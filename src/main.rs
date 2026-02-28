mod prompts;
mod providers;
mod repl;
mod server;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::providers::{PartialConfig, ProviderKind};
use crate::server::ServeArgs;

#[derive(Parser, Debug)]
#[command(
    name = "obstral",
    version,
    about = "OBSTRAL: provider-abstracted chat runtime (CLI + local UI)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Prompt (shorthand for `obstral chat \"<prompt>\"`)
    prompt: Option<String>,

    /// Force REPL (even if a prompt is provided)
    #[arg(long)]
    repl: bool,

    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// One-shot chat (prints the assistant response and exits)
    Chat {
        /// Prompt text
        prompt: String,
    },

    /// Interactive REPL
    Repl,

    /// Local web UI (React) + JSON API
    Serve(ServeArgs),

    /// List built-in values
    List {
        #[arg(value_enum)]
        what: ListWhat,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum ListWhat {
    Providers,
    Modes,
    Personas,
}

#[derive(Parser, Clone, Debug)]
struct CommonArgs {
    /// Apply VIBE preset defaults (provider=mistral, model=devstral-2, mode=VIBE)
    #[arg(long)]
    vibe: bool,

    #[arg(long, value_enum)]
    provider: Option<ProviderKind>,

    #[arg(long)]
    model: Option<String>,

    /// API key (prefer env vars to avoid shell history)
    #[arg(long)]
    api_key: Option<String>,

    /// Provider base URL (OpenAI-compatible: .../v1, Anthropic: .../v1)
    #[arg(long)]
    base_url: Option<String>,

    #[arg(long, value_enum)]
    mode: Option<prompts::Mode>,

    #[arg(long, value_enum)]
    persona: Option<prompts::Persona>,

    #[arg(long, default_value_t = 0.7)]
    temperature: f64,

    #[arg(long, default_value_t = 1024)]
    max_tokens: u32,

    #[arg(long, default_value_t = 120)]
    timeout_seconds: u64,

    /// Read a diff/patch file and inject it into the prompt (for diff批評)
    #[arg(long)]
    diff_file: Option<PathBuf>,

    /// Read stdin and append it to the prompt
    #[arg(long)]
    stdin: bool,
}

impl CommonArgs {
    fn to_partial_config(&self) -> PartialConfig {
        PartialConfig {
            vibe: self.vibe,
            provider: self.provider.clone(),
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            mode: self.mode.clone(),
            persona: self.persona.clone(),
            temperature: Some(self.temperature),
            max_tokens: Some(self.max_tokens),
            timeout_seconds: Some(self.timeout_seconds),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Chat { prompt }) => run_chat(prompt, cli.common).await,
        Some(Command::Repl) => repl::run(cli.common.to_partial_config()).await,
        Some(Command::Serve(args)) => server::run(args, cli.common.to_partial_config()).await,
        Some(Command::List { what }) => {
            run_list(what);
            Ok(())
        }
        None => {
            if let Some(prompt) = cli.prompt {
                return run_chat(prompt, cli.common).await;
            }
            if cli.repl {
                return repl::run(cli.common.to_partial_config()).await;
            }
            server::run(
                ServeArgs {
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                },
                cli.common.to_partial_config(),
            )
            .await
        }
    }
}

fn run_list(what: ListWhat) {
    match what {
        ListWhat::Providers => {
            for p in providers::supported_providers() {
                println!("{p}");
            }
        }
        ListWhat::Modes => {
            for m in prompts::supported_modes() {
                println!("{m}");
            }
        }
        ListWhat::Personas => {
            for p in prompts::supported_personas() {
                println!("{p}");
            }
        }
    }
}

async fn run_chat(prompt: String, common: CommonArgs) -> Result<()> {
    let mut user_input = prompt;

    if common.stdin {
        let stdin_text = read_stdin_to_string().context("failed to read stdin")?;
        if !stdin_text.trim().is_empty() {
            user_input = format!("{user_input}\n\n[stdin]\n{stdin_text}");
        }
    }

    let diff_text = match &common.diff_file {
        Some(path) => Some(
            std::fs::read_to_string(path)
                .with_context(|| format!("failed to read diff file: {}", path.display()))?,
        ),
        None => None,
    };

    let cfg = common.to_partial_config().resolve()?;
    let client = reqwest::Client::new();

    let content = providers::chat(&client, &cfg, &[], &user_input, diff_text.as_deref()).await?;
    println!("{content}");
    Ok(())
}

fn read_stdin_to_string() -> Result<String> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
