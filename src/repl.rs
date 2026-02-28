use anyhow::{Context, Result, anyhow};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::prompts::{Mode, Persona};
use crate::providers::{self, ChatMessage, PartialConfig, ProviderKind};

pub async fn run(mut partial: PartialConfig) -> Result<()> {
    let client = reqwest::Client::new();
    let mut history: Vec<ChatMessage> = Vec::new();

    let mut rl = DefaultEditor::new().context("failed to initialize line editor")?;
    let history_path = std::path::PathBuf::from(".obstral_history");
    let _ = rl.load_history(&history_path);

    println!("OBSTRAL REPL");
    println!("  /help  show commands");
    println!("  /exit  quit");

    loop {
        let cfg = partial.clone().resolve()?;
        let prompt = format!(
            "obstral[{}|{}|{}]> ",
            cfg.mode.label(),
            cfg.persona.key(),
            cfg.provider
        );

        let line = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                eprintln!("(interrupted; use /exit to quit)");
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => return Err(anyhow!(err).context("readline failed")),
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        rl.add_history_entry(line).ok();

        if line.starts_with('/') {
            if handle_command(line, &mut partial, &mut history)? {
                break;
            }
            continue;
        }

        let user_msg = ChatMessage {
            role: "user".to_string(),
            content: line.to_string(),
        };

        match providers::chat(&client, &cfg, &history, &user_msg.content, None).await {
            Ok(answer) => {
                println!("\n{answer}\n");
                history.push(user_msg);
                history.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: answer,
                });
            }
            Err(err) => {
                eprintln!("Error: {err}");
            }
        }
    }

    let _ = rl.save_history(&history_path);
    Ok(())
}

// Returns true if the REPL should exit.
fn handle_command(
    cmd: &str,
    partial: &mut PartialConfig,
    history: &mut Vec<ChatMessage>,
) -> Result<bool> {
    let mut parts = cmd.split_whitespace();
    let head = parts.next().unwrap_or("");
    let rest = parts.collect::<Vec<_>>().join(" ");

    match head {
        "/help" => {
            println!("Commands:");
            println!("  /help");
            println!("  /exit | /quit");
            println!("  /reset                    clear conversation history");
            println!("  /config                   show effective config (no API keys)");
            println!("  /vibe                     apply VIBE preset defaults");
            println!("  /provider <name>          openai-compatible | mistral | anthropic");
            println!("  /model <name>");
            println!("  /base-url <url>");
            println!("  /mode <name>              実況 | 壁打ち | diff批評 | VIBE");
            println!(
                "  /persona <name>           default | novelist | cynical | cheerful | thoughtful"
            );
            println!("  /temperature <0..2>");
            println!("  /max-tokens <n>");
        }
        "/exit" | "/quit" => return Ok(true),
        "/reset" => {
            history.clear();
            println!("(history cleared)");
        }
        "/config" => {
            let cfg = partial.clone().resolve()?;
            println!("provider        = {}", cfg.provider);
            println!("model           = {}", cfg.model);
            println!("base_url        = {}", cfg.base_url);
            println!("mode            = {}", cfg.mode.label());
            println!("persona         = {}", cfg.persona.key());
            println!("temperature     = {}", cfg.temperature);
            println!("max_tokens      = {}", cfg.max_tokens);
            println!("timeout_seconds = {}", cfg.timeout_seconds);
        }
        "/vibe" => {
            partial.vibe = true;
            partial.provider = None;
            partial.model = None;
            partial.base_url = None;
            partial.mode = None;
            println!("(VIBE preset enabled: provider=mistral, model=devstral-2, mode=VIBE)");
        }
        "/provider" => {
            let p = parse_provider(&rest)?;
            partial.provider = Some(p);
            partial.model = None;
            partial.base_url = None;
            partial.api_key = None;
            println!("(provider set; model/base_url/api_key reset to defaults/env)");
        }
        "/model" => {
            if rest.trim().is_empty() {
                return Err(anyhow!("usage: /model <name>"));
            }
            partial.model = Some(rest);
        }
        "/base-url" => {
            if rest.trim().is_empty() {
                return Err(anyhow!("usage: /base-url <url>"));
            }
            partial.base_url = Some(rest);
        }
        "/mode" => {
            let m = parse_mode(&rest)?;
            partial.mode = Some(m);
        }
        "/persona" => {
            let p = parse_persona(&rest)?;
            partial.persona = Some(p);
        }
        "/temperature" => {
            let v: f64 = rest.trim().parse().context("usage: /temperature <0..2>")?;
            partial.temperature = Some(v);
        }
        "/max-tokens" => {
            let v: u32 = rest.trim().parse().context("usage: /max-tokens <n>")?;
            partial.max_tokens = Some(v);
        }
        other => return Err(anyhow!("unknown command: {other} (try /help)")),
    }

    Ok(false)
}

fn parse_provider(s: &str) -> Result<ProviderKind> {
    let s = s.trim().to_ascii_lowercase();
    match s.as_str() {
        "openai-compatible" | "openai" | "openai_compat" => Ok(ProviderKind::OpenAiCompatible),
        "mistral" => Ok(ProviderKind::Mistral),
        "anthropic" => Ok(ProviderKind::Anthropic),
        _ => Err(anyhow!(
            "unsupported provider: {s}. Available: openai-compatible, mistral, anthropic"
        )),
    }
}

fn parse_mode(s: &str) -> Result<Mode> {
    let t = s.trim();
    if t.is_empty() {
        return Err(anyhow!("usage: /mode <name>"));
    }
    match t {
        "実況" | "jikkyo" | "live" => Ok(Mode::Jikkyo),
        "壁打ち" | "kabeuchi" | "ideation" => Ok(Mode::Kabeuchi),
        "diff批評" | "diff" | "review" => Ok(Mode::DiffReview),
        "VIBE" | "vibe" => Ok(Mode::Vibe),
        _ => Err(anyhow!(
            "unsupported mode: {t}. Available: 実況, 壁打ち, diff批評, VIBE"
        )),
    }
}

fn parse_persona(s: &str) -> Result<Persona> {
    let s = s.trim().to_ascii_lowercase();
    match s.as_str() {
        "default" => Ok(Persona::Default),
        "novelist" => Ok(Persona::Novelist),
        "cynical" => Ok(Persona::Cynical),
        "cheerful" => Ok(Persona::Cheerful),
        "thoughtful" => Ok(Persona::Thoughtful),
        _ => Err(anyhow!(
            "unsupported persona: {s}. Available: default, novelist, cynical, cheerful, thoughtful"
        )),
    }
}
