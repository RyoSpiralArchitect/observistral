mod agent_session;
mod approvals;
mod chatbot;
mod config;
mod exec;
mod file_tools;
mod governor_contract;
mod lang_detect;
mod loop_detect;
mod modes;
mod observer;
mod pending_commands;
mod pending_edits;
mod personas;
mod project;
mod providers;
mod repl;
mod server;
mod streaming;
mod task_graph;
mod trace_writer;
mod tui;
mod types;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::json;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::chatbot::ChatBot;
use crate::config::{PartialConfig, ProviderKind};
use crate::server::ServeArgs;
use crate::tui::TuiArgs;

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

    /// Headless coding agent (agentic loop with tools; like TUI Coder but CLI-only)
    Agent(AgentArgs),

    /// Review `git diff` with Observer (or diff批評) and print critique
    Review(ReviewArgs),

    /// Generate `.obstral.md` template in tool_root (project instructions + test_cmd)
    Init(InitArgs),

    /// Interactive REPL
    Repl,

    /// Local web UI (React) + JSON API
    Serve(ServeArgs),

    /// Terminal UI (dual-pane Coder + Observer, like Claude Code / Aider)
    Tui(TuiArgs),

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

#[derive(Args, Debug, Clone)]
struct AgentArgs {
    /// Prompt text. If omitted, pass `--stdin` to read the prompt from stdin.
    prompt: Option<String>,

    /// Working directory for file/exec tools (defaults to current directory)
    #[arg(long, short = 'C', alias = "root")]
    tool_root: Option<String>,

    /// UI / response language hint (ja/en/fr)
    #[arg(long, default_value = "ja")]
    lang: String,

    /// Max tool-call iterations (default: 12, max: 64)
    #[arg(long)]
    max_iters: Option<usize>,

    /// Auto-approve all commands and edits (no prompts)
    #[arg(long, short = 'y')]
    yes: bool,

    /// Disable ALL approval prompts (commands + edits)
    #[arg(long, alias = "no-approvals")]
    no_approval: bool,

    /// Disable approval prompts for `exec` tool calls (and implied commands)
    #[arg(long)]
    no_command_approval: bool,

    /// Disable approval prompts for file edits (write_file/patch_file/apply_diff)
    #[arg(long)]
    no_edit_approval: bool,

    /// Save and resume an agent session from this JSON file.
    /// If the file exists, OBSTRAL loads it and continues the conversation.
    /// If `-C/--root` is set and the session path is relative, it is resolved under `tool_root`.
    #[arg(long, short = 's', num_args = 0..=1, default_missing_value = ".tmp/obstral_session.json")]
    session: Option<PathBuf>,

    /// Start a new session even if `--session` already exists.
    #[arg(long)]
    new_session: bool,

    /// Auto-fix loop: after the agent finishes, run an Observer diff review and feed it back
    /// to the agent as a follow-up prompt.
    /// Usage:
    /// - `--autofix` (runs 1 round)
    /// - `--autofix 3` (runs 3 rounds)
    /// - `--autofix-rounds 3` (alias)
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "1",
        value_name = "ROUNDS",
        alias = "autofix-rounds"
    )]
    autofix: Option<usize>,

    /// Write a JSONL trace of the agent run (tool calls, checkpoints, errors, done).
    /// If `-C/--root` is set and the path is relative, it is resolved under `tool_root`.
    #[arg(long, alias = "trace_out")]
    trace_out: Option<PathBuf>,

    /// Write the final session snapshot as JSON (messages + tool_calls + tool results).
    /// If `-C/--root` is set and the path is relative, it is resolved under `tool_root`.
    #[arg(long, alias = "json_out")]
    json_out: Option<PathBuf>,

    /// Write an execution graph (nodes+edges) derived from the final session messages.
    /// If `-C/--root` is set and the path is relative, it is resolved under `tool_root`.
    #[arg(long, alias = "graph_out")]
    graph_out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
struct ReviewArgs {
    /// Optional guidance for the review (e.g. "focus on security and tests")
    prompt: Option<String>,

    /// Git repository directory to review (defaults to current directory)
    #[arg(long, short = 'C', alias = "root")]
    tool_root: Option<String>,

    /// Review only staged changes (`git diff --staged`)
    #[arg(long)]
    staged: bool,

    /// Review only unstaged changes (`git diff`)
    #[arg(long)]
    unstaged: bool,

    /// Base revision for a combined diff (`git diff <base>`). Cannot be combined with --staged/--unstaged.
    /// Useful with the agent's printed checkpoint hash.
    #[arg(long)]
    base: Option<String>,

    /// Maximum diff characters to include in the prompt (default: 24000)
    #[arg(long)]
    max_diff_chars: Option<usize>,
}

#[derive(Args, Debug, Clone)]
struct InitArgs {
    /// Directory to write `.obstral.md` into (defaults to current directory)
    #[arg(long, short = 'C', alias = "root")]
    tool_root: Option<String>,

    /// Overwrite `.obstral.md` if it already exists
    #[arg(long)]
    force: bool,
}

#[derive(Parser, Clone, Debug)]
struct CommonArgs {
    /// Apply VIBE preset defaults (provider=mistral, model=codestral-latest, mode=VIBE)
    #[arg(long, global = true)]
    vibe: bool,

    #[arg(long, value_enum, global = true)]
    provider: Option<ProviderKind>,

    #[arg(long, global = true)]
    model: Option<String>,

    /// Model for chat-like modes (実況/壁打ち). Defaults to --model when omitted.
    #[arg(long, global = true)]
    chat_model: Option<String>,

    /// Model for coding-like modes (VIBE/diff批評/ログ解析). Defaults to --model when omitted.
    #[arg(long, global = true)]
    code_model: Option<String>,

    /// API key (prefer env vars to avoid shell history)
    #[arg(long, global = true)]
    api_key: Option<String>,

    /// Provider base URL (OpenAI-compatible: .../v1, Anthropic: .../v1)
    #[arg(long, global = true)]
    base_url: Option<String>,

    #[arg(long, value_enum, global = true)]
    mode: Option<modes::Mode>,

    #[arg(long, global = true)]
    persona: Option<String>,

    #[arg(long, default_value_t = 0.4, global = true)]
    temperature: f64,

    #[arg(long, default_value_t = 1024, global = true)]
    max_tokens: u32,

    #[arg(long, default_value_t = 120, global = true)]
    timeout_seconds: u64,

    /// Read a diff/patch file and inject it into the prompt (for diff批評)
    #[arg(long, global = true)]
    diff_file: Option<PathBuf>,

    /// Path to a log file (for ログ解析 mode). Defaults to stdin when omitted.
    #[arg(long, global = true)]
    log_file: Option<PathBuf>,

    /// HF: device (auto|cpu|cuda)
    #[arg(long, global = true)]
    device: Option<String>,

    /// HF: local_files_only
    #[arg(long, global = true)]
    hf_local_only: bool,

    /// Read stdin and append it to the prompt
    #[arg(long, global = true)]
    stdin: bool,
}

impl CommonArgs {
    fn to_partial_config(&self) -> PartialConfig {
        PartialConfig {
            vibe: self.vibe,
            provider: self.provider.clone(),
            model: self.model.clone(),
            chat_model: self.chat_model.clone(),
            code_model: self.code_model.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            mode: self.mode.clone(),
            persona: self.persona.clone(),
            temperature: Some(self.temperature),
            max_tokens: Some(self.max_tokens),
            timeout_seconds: Some(self.timeout_seconds),
            hf_device: self.device.clone(),
            hf_local_only: if self.hf_local_only { Some(true) } else { None },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Chat { prompt }) => run_chat(prompt, cli.common).await,
        Some(Command::Agent(args)) => run_agent(args, cli.common).await,
        Some(Command::Review(args)) => run_review(args, cli.common).await,
        Some(Command::Init(args)) => run_init(args, cli.common).await,
        Some(Command::Repl) => repl::run(cli.common.to_partial_config()).await,
        Some(Command::Serve(args)) => server::run(args, cli.common.to_partial_config()).await,
        Some(Command::Tui(args)) => tui::run(args, cli.common.to_partial_config()).await,
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
            for p in config::supported_providers() {
                println!("{p}");
            }
        }
        ListWhat::Modes => {
            for m in modes::supported_modes() {
                println!("{m}");
            }
        }
        ListWhat::Personas => {
            for p in personas::supported_personas() {
                let label = personas::resolve_persona(p).map(|d| d.label).unwrap_or("");
                if label.trim().is_empty() {
                    println!("{p}");
                } else {
                    println!("{p}\t{label}");
                }
            }
        }
    }
}

async fn run_chat(prompt: String, common: CommonArgs) -> Result<()> {
    let mut user_input = prompt;

    if common.stdin && common.log_file.is_none() {
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
    let provider = providers::build_provider(client, &cfg);
    let bot = ChatBot::new(provider);

    let log_text = match (&cfg.mode, &common.log_file) {
        (modes::Mode::LogAnalysis, Some(p)) => Some(
            std::fs::read_to_string(p)
                .with_context(|| format!("failed to read log file: {}", p.display()))?,
        ),
        (modes::Mode::LogAnalysis, None) if common.stdin => {
            Some(read_stdin_to_string().context("failed to read stdin")?)
        }
        _ => None,
    };

    let resp = bot
        .run(
            &user_input,
            &[],
            &cfg.mode,
            &cfg.persona,
            None,
            "brief",
            cfg.temperature,
            cfg.max_tokens,
            diff_text.as_deref(),
            log_text.as_deref(),
        )
        .await?;

    println!("{}", resp.content);
    Ok(())
}

fn truncate_middle(s: &str, max_chars: usize) -> String {
    let s = s.trim_end();
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let head_len = max_chars / 2;
    let tail_len = max_chars.saturating_sub(head_len);
    let head: String = s.chars().take(head_len).collect();
    let tail: String = s
        .chars()
        .skip(char_count.saturating_sub(tail_len))
        .collect();
    let total_lines = s.lines().count();
    format!(
        "{head}\n...[truncated — {total_lines} lines total, {char_count} chars total]...\n{tail}"
    )
}

fn git_cmd_output(root: &str, args: &[&str]) -> Result<(String, String, i32)> {
    let out = std::process::Command::new("git")
        .args(["-C", root])
        .args(args)
        .output()
        .with_context(|| format!("failed to run git: git -C {root} {}", args.join(" ")))?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let exit = out.status.code().unwrap_or(-1);
    Ok((stdout, stderr, exit))
}

fn git_change_signal_since_base(root: &str, base: &str) -> Result<bool> {
    let (status_out, status_err, status_exit) =
        git_cmd_output(root, &["status", "--porcelain=v1"])?;
    if status_exit != 0 {
        anyhow::bail!("git status failed (exit {status_exit}).\n{status_err}");
    }

    let (diff_out, diff_err, diff_exit) =
        git_cmd_output(root, &["diff", "--no-color", "--name-only", base])?;
    if diff_exit != 0 {
        anyhow::bail!("git diff --name-only failed (exit {diff_exit}).\n{diff_err}");
    }

    Ok(!(status_out.trim().is_empty() && diff_out.trim().is_empty()))
}

async fn review_git_diff(
    tool_root: &str,
    staged: bool,
    unstaged: bool,
    base: Option<&str>,
    max_diff_chars: usize,
    guidance: &str,
    common: &CommonArgs,
    lang: Option<&str>,
) -> Result<String> {
    // Auto-scan project context (stack/git/tree + .obstral.md/AGENTS.md) for better reviews.
    let (project_context, agents_md) =
        if let Some(ctx) = crate::project::ProjectContext::scan(tool_root).await {
            (Some(ctx.to_context_text()), ctx.agents_md)
        } else {
            (None, None)
        };

    // Git status and diff payload.
    let (status_out, status_err, status_exit) =
        git_cmd_output(tool_root, &["status", "--porcelain=v1"])?;
    if status_exit != 0 {
        anyhow::bail!("git status failed (exit {status_exit}).\n{status_err}");
    }

    let base = base.map(|s| s.trim()).filter(|s| !s.is_empty());
    let max_chars = max_diff_chars.max(2_000).min(200_000);

    // Build diff + stat.
    let (diff_label, stat_args, diff_args): (&str, Vec<&str>, Vec<&str>) = if staged {
        (
            "staged",
            vec!["diff", "--no-color", "--stat", "--staged"],
            vec!["diff", "--no-color", "--staged"],
        )
    } else if unstaged {
        (
            "unstaged",
            vec!["diff", "--no-color", "--stat"],
            vec!["diff", "--no-color"],
        )
    } else if let Some(b) = base {
        (
            "combined",
            vec!["diff", "--no-color", "--stat", b],
            vec!["diff", "--no-color", b],
        )
    } else {
        // Prefer a combined diff vs HEAD when it exists; otherwise fall back to staged+unstaged sections.
        let (_h_out, _h_err, h_exit) =
            git_cmd_output(tool_root, &["rev-parse", "--verify", "HEAD"])?;
        if h_exit == 0 {
            (
                "combined",
                vec!["diff", "--no-color", "--stat", "HEAD"],
                vec!["diff", "--no-color", "HEAD"],
            )
        } else {
            ("mixed", vec![], vec![])
        }
    };

    let mut git_context = String::new();
    git_context.push_str(&format!("[repo]\nroot: {tool_root}\n"));
    if let Some(b) = base {
        git_context.push_str(&format!("base: {b}\n"));
    }
    git_context.push_str("\n[git status --porcelain=v1]\n");
    git_context.push_str(status_out.trim_end());
    if !status_out.trim().is_empty() {
        git_context.push('\n');
    }

    let mut diff_payload = git_context.clone();
    let mut diff_patch = String::new();

    if diff_label == "mixed" {
        // Empty repo / no HEAD: show staged + unstaged separately.
        let (stat_s, err_s, exit_s) =
            git_cmd_output(tool_root, &["diff", "--no-color", "--stat", "--staged"])?;
        if exit_s == 0 && !stat_s.trim().is_empty() {
            diff_payload.push_str("\n[git diff --staged --stat]\n");
            diff_payload.push_str(stat_s.trim_end());
            diff_payload.push('\n');

            git_context.push_str("\n[git diff --staged --stat]\n");
            git_context.push_str(stat_s.trim_end());
            git_context.push('\n');
        } else if exit_s != 0 && !err_s.trim().is_empty() {
            diff_payload.push_str("\n[git diff --staged --stat ERROR]\n");
            diff_payload.push_str(err_s.trim_end());
            diff_payload.push('\n');
        }

        let (diff_s, err_sd, exit_sd) =
            git_cmd_output(tool_root, &["diff", "--no-color", "--staged"])?;
        if exit_sd == 0 && !diff_s.trim().is_empty() {
            diff_payload.push_str("\n[git diff --staged]\n");
            let t = truncate_middle(&diff_s, max_chars);
            diff_payload.push_str(&t);
            diff_payload.push('\n');

            diff_patch.push_str("[git diff --staged]\n");
            diff_patch.push_str(&t);
            diff_patch.push('\n');
        } else if exit_sd != 0 && !err_sd.trim().is_empty() {
            diff_payload.push_str("\n[git diff --staged ERROR]\n");
            diff_payload.push_str(err_sd.trim_end());
            diff_payload.push('\n');
        }

        let (stat_u, err_u, exit_u) = git_cmd_output(tool_root, &["diff", "--no-color", "--stat"])?;
        if exit_u == 0 && !stat_u.trim().is_empty() {
            diff_payload.push_str("\n[git diff --stat]\n");
            diff_payload.push_str(stat_u.trim_end());
            diff_payload.push('\n');

            git_context.push_str("\n[git diff --stat]\n");
            git_context.push_str(stat_u.trim_end());
            git_context.push('\n');
        } else if exit_u != 0 && !err_u.trim().is_empty() {
            diff_payload.push_str("\n[git diff --stat ERROR]\n");
            diff_payload.push_str(err_u.trim_end());
            diff_payload.push('\n');
        }

        let (diff_u, err_ud, exit_ud) = git_cmd_output(tool_root, &["diff", "--no-color"])?;
        if exit_ud == 0 && !diff_u.trim().is_empty() {
            diff_payload.push_str("\n[git diff]\n");
            let t = truncate_middle(&diff_u, max_chars);
            diff_payload.push_str(&t);
            diff_payload.push('\n');

            diff_patch.push_str("[git diff]\n");
            diff_patch.push_str(&t);
            diff_patch.push('\n');
        } else if exit_ud != 0 && !err_ud.trim().is_empty() {
            diff_payload.push_str("\n[git diff ERROR]\n");
            diff_payload.push_str(err_ud.trim_end());
            diff_payload.push('\n');
        }
    } else {
        let (stat, stat_err, stat_exit) = git_cmd_output(tool_root, &stat_args)?;
        if stat_exit == 0 && !stat.trim().is_empty() {
            diff_payload.push_str(&format!("\n[git diff --stat] ({diff_label})\n"));
            diff_payload.push_str(stat.trim_end());
            diff_payload.push('\n');

            git_context.push_str(&format!("\n[git diff --stat] ({diff_label})\n"));
            git_context.push_str(stat.trim_end());
            git_context.push('\n');
        } else if stat_exit != 0 && !stat_err.trim().is_empty() {
            diff_payload.push_str("\n[git diff --stat ERROR]\n");
            diff_payload.push_str(stat_err.trim_end());
            diff_payload.push('\n');
        }

        let (diff, diff_err, diff_exit) = git_cmd_output(tool_root, &diff_args)?;
        if diff_exit == 0 && !diff.trim().is_empty() {
            diff_payload.push_str(&format!("\n[git diff] ({diff_label})\n"));
            let t = truncate_middle(&diff, max_chars);
            diff_payload.push_str(&t);
            diff_payload.push('\n');

            diff_patch.push_str(&t);
            diff_patch.push('\n');
        } else if diff_exit != 0 {
            anyhow::bail!("git diff failed (exit {diff_exit}).\n{diff_err}");
        }
    }

    // Resolve config. Default review mode to Observer unless the user explicitly set one.
    let mut partial = common.to_partial_config();
    if partial.mode.is_none() {
        partial.mode = Some(crate::modes::Mode::Observer);
    }
    let cfg = partial.resolve()?;

    let client = reqwest::Client::new();
    let provider = providers::build_provider(client, &cfg);
    let bot = ChatBot::new(provider);

    // For diff批評 mode, pass diff via diff_text so compose_user_text wraps it in ```diff```.
    // For Observer mode, include the diff in the user input (diff_text injection is ignored there).
    let user_input = if matches!(cfg.mode, crate::modes::Mode::DiffReview) {
        let mut s = String::new();
        if let Some(ctx) = project_context.as_deref() {
            if !ctx.trim().is_empty() {
                s.push_str(ctx.trim_end());
                s.push_str("\n\n");
            }
        }
        if let Some(a) = agents_md.as_deref() {
            if !a.trim().is_empty() {
                s.push_str("[Project Instructions]\n");
                s.push_str(a.trim_end());
                s.push_str("\n\n");
            }
        }
        if !git_context.trim().is_empty() {
            s.push_str("[git context]\n");
            s.push_str(git_context.trim_end());
            s.push_str("\n\n");
        }
        s.push_str(guidance);
        s
    } else {
        let mut s = String::new();
        if let Some(ctx) = project_context.as_deref() {
            if !ctx.trim().is_empty() {
                s.push_str(ctx.trim_end());
                s.push_str("\n\n");
            }
        }
        if let Some(a) = agents_md.as_deref() {
            if !a.trim().is_empty() {
                s.push_str("[Project Instructions]\n");
                s.push_str(a.trim_end());
                s.push_str("\n\n");
            }
        }
        s.push_str(guidance);
        s.push_str("\n\n[git diff payload]\n");
        s.push_str(diff_payload.trim_end());
        s
    };

    let cot = if matches!(cfg.mode, crate::modes::Mode::Observer) {
        "off"
    } else {
        "brief"
    };

    let resp = bot
        .run(
            &user_input,
            &[],
            &cfg.mode,
            &cfg.persona,
            lang,
            cot,
            cfg.temperature,
            cfg.max_tokens,
            if matches!(cfg.mode, crate::modes::Mode::DiffReview) {
                Some(diff_patch.as_str())
            } else {
                None
            },
            None,
        )
        .await?;

    Ok(resp.content)
}

async fn run_review(args: ReviewArgs, common: CommonArgs) -> Result<()> {
    let tool_root = args.tool_root.clone().or_else(|| {
        std::env::current_dir()
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
    });
    let tool_root = tool_root
        .as_deref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ".".to_string());

    if args.base.is_some() && (args.staged || args.unstaged) {
        anyhow::bail!("--base cannot be combined with --staged/--unstaged");
    }
    if args.staged && args.unstaged {
        anyhow::bail!(
            "--staged and --unstaged are mutually exclusive (omit both for combined review)"
        );
    }

    // Read optional prompt guidance from stdin.
    let mut guidance = args.prompt.unwrap_or_default();
    if common.stdin {
        let stdin_text = read_stdin_to_string().context("failed to read stdin")?;
        if !stdin_text.trim().is_empty() {
            if guidance.trim().is_empty() {
                guidance = stdin_text;
            } else {
                guidance = format!("{guidance}\n\n[stdin]\n{stdin_text}");
            }
        }
    }
    if guidance.trim().is_empty() {
        guidance = "Review this git diff and critique it ruthlessly and concretely.".to_string();
    }

    let max_chars = args
        .max_diff_chars
        .unwrap_or(24_000)
        .max(2_000)
        .min(200_000);
    let out = review_git_diff(
        &tool_root,
        args.staged,
        args.unstaged,
        args.base.as_deref(),
        max_chars,
        &guidance,
        &common,
        None,
    )
    .await?;

    println!("{out}");
    Ok(())
}

fn parse_at_refs(text: &str) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for word in text.split_whitespace() {
        if !word.starts_with('@') {
            continue;
        }
        let path = word.trim_start_matches('@');
        let path = path.trim_end_matches(|c: char| matches!(c, ',' | ')' | ']' | ';' | ':' | '.'));
        if path.is_empty() {
            continue;
        }
        if seen.insert(path.to_string()) {
            refs.push(path.to_string());
        }
    }
    refs
}

fn normalize_path_for_compare(s: &str) -> String {
    let mut out = s.trim().replace('\\', "/");
    while out.ends_with('/') && out.len() > 1 {
        out.pop();
    }
    if cfg!(target_os = "windows") {
        out = out.to_ascii_lowercase();
    }
    out
}

fn normalize_tool_root(tool_root: Option<String>) -> Option<String> {
    let raw = tool_root?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let pb = std::path::PathBuf::from(raw);
    let abs = if pb.is_absolute() {
        pb
    } else {
        std::env::current_dir().ok()?.join(pb)
    };

    // Ensure the directory exists so canonicalize can remove `..` components.
    let _ = std::fs::create_dir_all(&abs);
    let abs = std::fs::canonicalize(&abs).unwrap_or(abs);

    Some(abs.to_string_lossy().into_owned())
}

fn resolve_session_path(session_path: PathBuf, tool_root: Option<&str>) -> PathBuf {
    if session_path.is_absolute() {
        return session_path;
    }
    let Some(root) = tool_root.map(|s| s.trim()).filter(|s| !s.is_empty()) else {
        return session_path;
    };
    std::path::PathBuf::from(root).join(session_path)
}

async fn run_agent(args: AgentArgs, common: CommonArgs) -> Result<()> {
    let AgentArgs {
        prompt,
        tool_root: tool_root_arg,
        lang,
        max_iters,
        yes,
        no_approval,
        no_command_approval,
        no_edit_approval,
        session: session_path,
        new_session,
        autofix,
        trace_out,
        json_out,
        graph_out,
    } = args;

    let session_path = session_path.map(|sp| resolve_session_path(sp, tool_root_arg.as_deref()));

    let mut loaded_session: Option<crate::agent_session::AgentSession> = None;
    let mut start_messages_json: Option<Vec<serde_json::Value>> = None;
    let mut start_checkpoint: Option<String> = None;
    let mut start_cwd: Option<String> = None;
    let mut create_checkpoint = true;
    let mut resuming = false;

    if let Some(ref sp) = session_path {
        if sp.exists() && !new_session {
            let mut sess = crate::agent_session::AgentSession::load(sp)?;
            if let Some(warn) = sess.repair_for_resume() {
                eprintln!("[session] WARN: {warn}");
            }
            let msg_count = sess.messages.len();
            let ckpt_short = sess
                .checkpoint
                .as_deref()
                .map(|h| &h[..h.len().min(8)])
                .unwrap_or("-");
            eprintln!(
                "[session] resuming: {} (messages={msg_count}, checkpoint={ckpt_short})",
                sp.display()
            );
            start_checkpoint = sess.checkpoint.clone();
            start_cwd = sess.cur_cwd.clone();
            create_checkpoint = sess.checkpoint.is_none();
            start_messages_json = Some(std::mem::take(&mut sess.messages));
            loaded_session = Some(sess);
            resuming = true;
        } else if new_session && sp.exists() {
            eprintln!(
                "[session] --new-session: starting fresh and overwriting {}",
                sp.display()
            );
        } else if new_session {
            eprintln!(
                "[session] --new-session: starting fresh at {}",
                sp.display()
            );
        }
    }

    let saved_tool_root = loaded_session.as_ref().and_then(|s| s.tool_root.clone());
    let saved_tool_root_norm = normalize_tool_root(saved_tool_root.clone());

    let tool_root = normalize_tool_root(
        tool_root_arg
            .clone()
            .or_else(|| saved_tool_root.clone())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            }),
    );

    if resuming && tool_root_arg.is_some() {
        if let (Some(ref chosen), Some(ref saved)) =
            (tool_root.as_ref(), saved_tool_root_norm.as_ref())
        {
            if normalize_path_for_compare(chosen) != normalize_path_for_compare(saved) {
                eprintln!(
                    "WARN: --root/--tool-root ({}) differs from saved session tool_root ({}). Continuing with provided tool root.",
                    chosen,
                    saved
                );
            }
        }
    }

    let trace_out_path = trace_out.map(|p| resolve_session_path(p, tool_root.as_deref()));
    let json_out_path = json_out.map(|p| resolve_session_path(p, tool_root.as_deref()));
    let graph_out_path = graph_out.map(|p| resolve_session_path(p, tool_root.as_deref()));

    // Build the user prompt text.
    fn default_continue_prompt(lang: &str) -> String {
        let l = lang.trim().to_ascii_lowercase();
        if l == "fr" {
            "Continue depuis l’état précédent. Reprends la tâche et vérifie avec des commandes/tests.".to_string()
        } else if l == "en" {
            "Continue from the previous state. Resume the task and verify with commands/tests."
                .to_string()
        } else {
            "前回の状態から続けて。作業を再開して、コマンド/テストで検証して。".to_string()
        }
    }
    let pending_user_turn = resuming
        && start_messages_json
            .as_ref()
            .and_then(|m| m.last())
            .and_then(|v| v.get("role").and_then(|r| r.as_str()))
            == Some("user");
    let mut used_stdin_as_prompt = false;
    let mut append_user_message = true;
    let mut user_input = match (prompt, common.stdin) {
        (Some(p), _) => p,
        (None, true) => {
            used_stdin_as_prompt = true;
            read_stdin_to_string().context("failed to read stdin")?
        }
        (None, false) if pending_user_turn => {
            // Resume by answering the pending user message in the session. Do not append
            // a synthetic "continue" prompt that could override the unfinished turn.
            append_user_message = false;
            String::new()
        }
        (None, false) if resuming => default_continue_prompt(&lang),
        (None, false) => {
            anyhow::bail!("missing prompt. Provide a prompt argument or pass --stdin.")
        }
    };

    // If --stdin and a prompt was provided, append stdin as extra context (like `run_chat`).
    if common.stdin && !used_stdin_as_prompt && !user_input.trim().is_empty() {
        let stdin_text = read_stdin_to_string().context("failed to read stdin")?;
        if !stdin_text.trim().is_empty() {
            user_input = format!("{user_input}\n\n[stdin]\n{stdin_text}");
        }
    }

    // Expand @file references into injected system messages.
    let at_refs = parse_at_refs(&user_input);
    let mut at_ref_messages_json: Vec<serde_json::Value> = Vec::new();
    for ref_path in &at_refs {
        let (content, is_err) = crate::file_tools::tool_read_file(ref_path, tool_root.as_deref());
        if is_err {
            eprintln!("[@{ref_path}] not found");
        } else {
            let header = content
                .lines()
                .next()
                .unwrap_or(ref_path.as_str())
                .to_string();
            eprintln!("📎 injected: {header}");
            at_ref_messages_json.push(json!({
                "role": "system",
                "content": format!("[@{ref_path}]\n{content}"),
            }));
        }
    }

    // Resolve config. For the headless coding agent, default to a code-like mode
    // so `--code-model` is selected when users set different models per mode.
    let mut partial = common.to_partial_config();
    if !partial.vibe && partial.mode.is_none() {
        partial.mode = Some(crate::modes::Mode::Vibe);
    }
    let cfg = partial.resolve()?;

    let trace: Option<crate::trace_writer::TraceWriter> = match trace_out_path.clone() {
        Some(p) => {
            eprintln!("[trace] writing: {}", p.display());
            Some(crate::trace_writer::TraceWriter::new(p)?)
        }
        None => None,
    };
    if let Some(ref tw) = trace {
        let _ = tw.event(
            "agent_start",
            json!({
                "resuming": resuming,
                "lang": lang.as_str(),
                "max_iters": max_iters.unwrap_or(crate::tui::agent::DEFAULT_MAX_ITERS).max(1).min(64),
                "tool_root": tool_root.as_deref(),
                "session": session_path.as_ref().map(|p| p.display().to_string()),
                "json_out": json_out_path.as_ref().map(|p| p.display().to_string()),
                "graph_out": graph_out_path.as_ref().map(|p| p.display().to_string()),
                "cfg": {
                    "provider": cfg.provider.key(),
                    "base_url": cfg.base_url.as_str(),
                    "mode": cfg.mode.label(),
                    "persona": cfg.persona.as_str(),
                    "model": cfg.model.as_str(),
                    "chat_model": cfg.chat_model.as_str(),
                    "code_model": cfg.code_model.as_str(),
                    "temperature": cfg.temperature,
                    "max_tokens": cfg.max_tokens,
                    "timeout_seconds": cfg.timeout_seconds,
                }
            }),
        );
    }

    // Build initial message list (OpenAI-compatible JSON).
    let mut messages_json: Vec<serde_json::Value> = if let Some(m) = start_messages_json.take() {
        m
    } else {
        // New session: build system prompt (same as TUI Coder).
        let persona_prompt = personas::resolve_persona(&cfg.persona)
            .map(|p| p.prompt)
            .unwrap_or("");
        let lang_instruction = crate::modes::language_instruction(Some(&lang), &cfg.mode);
        let system = crate::tui::agent::coder_system(persona_prompt, lang_instruction);
        vec![json!({"role":"system","content": system})]
    };
    messages_json.extend(at_ref_messages_json);
    if append_user_message {
        messages_json.push(json!({"role":"user","content": user_input}));
    }

    // Scan project context (stack/git/tree + .obstral.md/AGENTS.md + test_cmd).
    let (project_context, agents_md, test_cmd) = if let Some(ref root) = tool_root {
        if let Some(ctx) = project::ProjectContext::scan(root).await {
            (Some(ctx.to_context_text()), ctx.agents_md, ctx.test_cmd)
        } else {
            (None, None, None)
        }
    } else {
        (None, None, None)
    };

    let autofix_rounds = match autofix {
        Some(n) => n.max(1).min(8),
        None => 0usize,
    };

    // Stream tokens to stdout.
    let max_iters = max_iters
        .unwrap_or(crate::tui::agent::DEFAULT_MAX_ITERS)
        .max(1)
        .min(64);
    let command_approval = !no_command_approval && !no_approval;
    let edit_approval = !no_edit_approval && !no_approval;
    let approver = std::sync::Arc::new(crate::approvals::CliApprover::new(
        command_approval,
        edit_approval,
        yes,
    ));

    use std::io::Write;
    let mut stdout = std::io::stdout();
    let mut checkpoint: Option<String> = start_checkpoint.clone();
    let mut cur_cwd: Option<String> = start_cwd.clone();
    let mut create_checkpoint_round = create_checkpoint;
    let mut result: Result<()> = Ok(());

    let autosaver = session_path.as_ref().map(|sp| {
        std::sync::Arc::new(crate::agent_session::SessionAutoSaver::new(
            sp.clone(),
            loaded_session.as_ref(),
        ))
    });
    if let Some(ref saver) = autosaver {
        if !resuming {
            if let Some(ref sp) = session_path {
                eprintln!("[session] saving: {} (autosave enabled)", sp.display());
            }
        }
        saver.save_or_error(
            tool_root.as_deref(),
            checkpoint.as_deref(),
            cur_cwd.as_deref(),
            &messages_json,
        )?;
    }

    let total_rounds = 1usize + autofix_rounds;
    let mut interrupted = false;
    for round in 0..total_rounds {
        if let Some(ref tw) = trace {
            let _ = tw.event(
                "round_start",
                json!({
                    "round": round,
                    "total_rounds": total_rounds,
                    "autofix": round > 0,
                }),
            );
        }
        if round > 0 {
            eprintln!(
                "\n\n[autofix] round {round}/{autofix_rounds} — applying Observer proposals\n"
            );
        }

        let mut saw_error = false;
        let (tx, mut rx) = mpsc::channel::<streaming::StreamToken>(128);

        let start_state = crate::tui::agent::AgenticStartState {
            messages: messages_json.clone(),
            checkpoint: checkpoint.clone(),
            cur_cwd: cur_cwd.clone(),
            create_checkpoint: create_checkpoint_round,
        };
        create_checkpoint_round = false;

        let tool_root_for_task = tool_root.clone();
        let cfg_for_task = cfg.clone();
        let project_context_for_task = project_context.clone();
        let agents_md_for_task = agents_md.clone();
        let test_cmd_for_task = test_cmd.clone();
        let autosaver_for_task = autosaver.clone();
        let approver_for_task = approver.clone();

        let handle = tokio::spawn(async move {
            crate::tui::agent::run_agentic_json(
                start_state,
                &cfg_for_task,
                tool_root_for_task.as_deref(),
                max_iters,
                tx,
                project_context_for_task,
                agents_md_for_task,
                test_cmd_for_task,
                command_approval,
                autosaver_for_task,
                approver_for_task.as_ref(),
            )
            .await
        });

        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);
        loop {
            tokio::select! {
                token = rx.recv() => {
                    let Some(token) = token else { break };
                    match token {
                        streaming::StreamToken::Delta(s) => {
                            stdout.write_all(s.as_bytes()).ok();
                            stdout.flush().ok();
                        }
                        streaming::StreamToken::ToolCall(tc) => {
                            if let Some(ref tw) = trace {
                                let _ = tw.event(
                                    "tool_call",
                                    json!({
                                        "id": tc.id,
                                        "name": tc.name,
                                        "arguments": tc.arguments,
                                    }),
                                );
                            }
                        }
                        streaming::StreamToken::GovernorState(s) => {
                            if let Some(ref tw) = trace {
                                let _ = tw.event("governor_state", json!(s));
                            }
                        }
                        streaming::StreamToken::Checkpoint(hash) => {
                            checkpoint = Some(hash.clone());
                            if let Some(ref tw) = trace {
                                let _ = tw.event("checkpoint", json!({"hash": hash}));
                            }
                        }
                        streaming::StreamToken::Done => {
                            if let Some(ref tw) = trace {
                                let _ = tw.event("done", json!({}));
                            }
                            break
                        },
                        streaming::StreamToken::Error(e) => {
                            saw_error = true;
                            eprintln!("ERROR: {e}");
                            if let Some(ref tw) = trace {
                                let _ = tw.event("error", json!({"message": e}));
                            }
                        }
                    }
                }
                _ = &mut ctrl_c => {
                    interrupted = true;
                    eprintln!("\n\n[agent] interrupted (Ctrl+C). Session can be resumed.\n");
                    if let Some(ref tw) = trace {
                        let _ = tw.event("interrupted", json!({}));
                    }
                    handle.abort();
                    if let Some(ref saver) = autosaver {
                        let _ = saver.save_best_effort(
                            tool_root.as_deref(),
                            checkpoint.as_deref(),
                            cur_cwd.as_deref(),
                            &messages_json,
                        );
                    }
                    break;
                }
            }
        }

        if interrupted {
            result = Err(anyhow::anyhow!("interrupted"));
            break;
        }

        let end_state = match handle.await {
            Err(e) => {
                result = Err(anyhow::anyhow!("agent task panicked: {e}"));
                break;
            }
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                result = Err(e);
                break;
            }
        };

        messages_json = end_state.messages;
        cur_cwd = end_state.cur_cwd;
        checkpoint = checkpoint.or(end_state.checkpoint);
        if let Some(ref tw) = trace {
            let last_reflection =
                crate::agent_session::last_reflection_summary_from_messages(&messages_json);
            let _ = tw.event(
                "round_end",
                json!({
                    "round": round,
                    "messages_len": messages_json.len(),
                    "checkpoint": checkpoint.as_deref(),
                    "cur_cwd": cur_cwd.as_deref(),
                    "last_reflection": last_reflection,
                }),
            );
        }

        if saw_error {
            result = Err(anyhow::anyhow!("agent finished with errors"));
            break;
        }

        if round + 1 == total_rounds {
            break;
        }

        // ── Observer diff review → feed back into Coder (auto-fix loop) ───
        let Some(ref root) = tool_root else {
            eprintln!("[autofix] skipped: no tool_root set");
            break;
        };
        let Some(ref base_hash) = checkpoint else {
            eprintln!("[autofix] skipped: no git checkpoint available (not a git repo?)");
            break;
        };

        match git_change_signal_since_base(root, base_hash) {
            Ok(false) => {
                eprintln!("[autofix] no changes since checkpoint; stopping.");
                break;
            }
            Err(e) => {
                eprintln!("[autofix] skipped: {e:#}");
                break;
            }
            Ok(true) => {}
        }

        let short = &base_hash[..base_hash.len().min(8)];
        eprintln!("\n\n[autofix] Observer review (base {short})\n");
        let guidance = "Review changes since the session checkpoint. Output concrete, actionable proposals for the Coder to implement.";

        let review = match review_git_diff(
            root,
            false,
            false,
            Some(base_hash.as_str()),
            24_000,
            guidance,
            &common,
            Some(&lang),
        )
        .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[autofix] Observer review failed: {e:#}");
                result = Err(e);
                break;
            }
        };

        eprintln!("\n[Observer review]\n{review}\n");

        let next_prompt = format!(
            "[Auto-fix requested]\n\
The Observer has reviewed the code and identified the following issues. Fix ALL of them.\n\
For each proposal you address, verify with commands/tests. When finished, call done.\n\n\
{review}"
        );
        messages_json.push(json!({"role":"user","content": next_prompt}));
        if let Some(ref saver) = autosaver {
            if let Some(warn) = saver.save_best_effort(
                tool_root.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages_json,
            ) {
                eprintln!("[autosave] WARN: {warn}");
            }
        }
    }

    // Save session file (if requested).
    if let Some(ref saver) = autosaver {
        saver.save_or_error(
            tool_root.as_deref(),
            checkpoint.as_deref(),
            cur_cwd.as_deref(),
            &messages_json,
        )?;
    }

    // Optional: write a final session snapshot separate from --session autosave.
    if let Some(ref out_path) = json_out_path {
        let mut sess = crate::agent_session::AgentSession::new(
            tool_root.clone(),
            checkpoint.clone(),
            cur_cwd.clone(),
            messages_json.clone(),
        );
        if let Some(ref loaded) = loaded_session {
            sess.created_at_ms = loaded.created_at_ms;
        }
        match crate::agent_session::AgentSession::save_atomic(out_path, &sess) {
            Ok(_) => eprintln!("[json_out] wrote: {}", out_path.display()),
            Err(e) => {
                eprintln!("[json_out] ERROR: {e:#}");
                if result.is_ok() {
                    result = Err(e);
                }
            }
        }
    }

    // Optional: write an execution graph derived from the final messages.
    if let Some(ref out_path) = graph_out_path {
        let graph = crate::task_graph::TaskGraph::from_session_messages(
            tool_root.clone(),
            checkpoint.clone(),
            cur_cwd.clone(),
            &messages_json,
        );
        match crate::task_graph::save_graph_atomic(out_path, &graph) {
            Ok(_) => eprintln!("[graph_out] wrote: {}", out_path.display()),
            Err(e) => {
                eprintln!("[graph_out] ERROR: {e:#}");
                if result.is_ok() {
                    result = Err(e);
                }
            }
        }
    }

    // CLI nicety: show a compact git diff summary from the auto-created checkpoint.
    if result.is_ok() {
        let checkpoint_final = checkpoint.as_deref().map(|s| s.to_string());
        if let (Some(ref root), Some(ref hash)) = (tool_root, checkpoint_final.as_deref()) {
            let stat = std::process::Command::new("git")
                .args(["-C", root, "diff", hash, "--stat"])
                .output();
            let names = std::process::Command::new("git")
                .args(["-C", root, "diff", hash, "--name-status"])
                .output();
            if let (Ok(s), Ok(n)) = (stat, names) {
                if s.status.success() && n.status.success() {
                    let s_txt = String::from_utf8_lossy(&s.stdout).trim().to_string();
                    let n_txt = String::from_utf8_lossy(&n.stdout).trim().to_string();
                    if !s_txt.is_empty() || !n_txt.is_empty() {
                        println!(
                            "\n\n[git diff from checkpoint]\n{}\n\nFiles:\n{}",
                            if s_txt.is_empty() {
                                "(no changes)".to_string()
                            } else {
                                s_txt
                            },
                            if n_txt.is_empty() {
                                "(no files)".to_string()
                            } else {
                                n_txt
                            },
                        );
                    }
                }
            }
        }
    }

    if let Some(ref tw) = trace {
        let ok = result.is_ok();
        let err = result.as_ref().err().map(|e| format!("{e:#}"));
        let _ = tw.event(
            "agent_end",
            json!({
                "ok": ok,
                "error": err,
                "messages_len": messages_json.len(),
                "checkpoint": checkpoint.as_deref(),
                "cur_cwd": cur_cwd.as_deref(),
            }),
        );
    }

    result
}

async fn run_init(args: InitArgs, _common: CommonArgs) -> Result<()> {
    let root = args.tool_root.clone().or_else(|| {
        std::env::current_dir()
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
    });
    let root = root.unwrap_or_else(|| ".".to_string());
    let root_p = std::path::Path::new(&root);
    let obstral_path = root_p.join(".obstral.md");

    if obstral_path.exists() && !args.force {
        anyhow::bail!(
            ".obstral.md already exists at {} (use --force to overwrite)",
            obstral_path.display()
        );
    }

    // Detect stack and test_cmd synchronously (same logic as TUI /init).
    let has_cargo = root_p.join("Cargo.toml").exists();
    let has_pkg = root_p.join("package.json").exists();
    let has_py = root_p.join("pyproject.toml").exists() || root_p.join("requirements.txt").exists();
    let has_go = root_p.join("go.mod").exists();
    let stack = [
        if has_cargo { Some("Rust") } else { None },
        if has_pkg { Some("Node/React") } else { None },
        if has_py { Some("Python") } else { None },
        if has_go { Some("Go") } else { None },
    ]
    .iter()
    .flatten()
    .cloned()
    .collect::<Vec<_>>()
    .join(", ");
    let test_cmd = if has_cargo {
        "cargo test 2>&1"
    } else if has_pkg {
        "npm test --passWithNoTests 2>&1"
    } else if has_py {
        "pytest -q 2>&1"
    } else if has_go {
        "go test ./... 2>&1"
    } else {
        "# add your test command here"
    };
    let stack_line = if stack.is_empty() {
        "# auto-detected: unknown".to_string()
    } else {
        stack.clone()
    };
    let content = format!(
        "# .obstral.md — Project Instructions for OBSTRAL Coder
#
# This file is automatically injected into the Coder's system prompt.
# Edit it to set project rules, test commands, and coding conventions.

## Stack
{stack_line}

## Test Command
test_cmd: {test_cmd}

## Development Rules
- Always run tests after modifying source files
- Use patch_file or apply_diff for targeted edits (safer than exec+sed)
- Keep git commits small and focused
- Check for compilation errors before marking a task done

## Forbidden Commands
# List commands that should never be run automatically:
# - git push --force
# - rm -rf /
# - DROP TABLE

## Notes
# Add any project-specific context, architecture notes, or constraints here.
"
    );

    std::fs::create_dir_all(root_p)
        .with_context(|| format!("failed to create tool_root: {}", root_p.display()))?;
    std::fs::write(&obstral_path, content.as_bytes())
        .with_context(|| format!("failed to write {}", obstral_path.display()))?;

    println!("✓ wrote {}", obstral_path.display());
    Ok(())
}

fn read_stdin_to_string() -> Result<String> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
