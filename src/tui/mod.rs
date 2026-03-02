pub mod app;
pub mod events;
pub mod ui;
pub mod agent;

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::PartialConfig;
use crate::modes::Mode;

#[derive(clap::Args, Debug, Clone)]
pub struct TuiArgs {
    /// Model to use for the Observer pane (defaults to same as Coder).
    #[arg(long)]
    pub observer_model: Option<String>,

    /// UI / response language hint (ja/en/fr).
    #[arg(long, default_value = "ja")]
    pub lang: String,

    /// Working directory for exec tool calls.
    #[arg(long)]
    pub tool_root: Option<String>,

    /// Enable auto-observe (Observer auto-fires after each Coder response).
    #[arg(long)]
    pub auto_observe: bool,
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    fn SetConsoleCP(wCodePageID: u32) -> i32;
}

fn default_tui_tool_root() -> String {
    // Isolate each TUI session to avoid collisions and nested git repo disasters.
    // Keep it relative so it stays under the user's workspace root by default.
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!(".tmp/tui_{epoch}")
}

pub async fn run(args: TuiArgs, partial_cfg: PartialConfig) -> Result<()> {
    // Resolve the Coder config from the shared partial config.
    let coder_cfg = partial_cfg.clone().resolve().context("failed to resolve coder config")?;

    // Chat uses Chat mode (no tools) and should default to chat_model (not code_model).
    let chat_cfg = {
        let mut chat_partial = partial_cfg.clone();
        chat_partial.mode = Some(Mode::Chat);
        chat_partial.resolve().context("failed to resolve chat config")?
    };

    // For the Observer we allow a different model via --observer-model.
    let observer_cfg = {
        let mut obs_partial = partial_cfg.clone();
        if let Some(obs_model) = &args.observer_model {
            obs_partial.model = Some(obs_model.clone());
            obs_partial.chat_model = None;
            obs_partial.code_model = None;
        }
        // Observer always uses Observer mode.
        obs_partial.mode = Some(crate::modes::Mode::Observer);
        obs_partial.resolve().context("failed to resolve observer config")?
    };

    let tool_root = args.tool_root.clone().or_else(|| Some(default_tui_tool_root()));
    if let Some(ref r) = tool_root {
        if !r.trim().is_empty() {
            std::fs::create_dir_all(r).context("failed to create tool_root")?;
        }
    }
    let auto_observe = args.auto_observe;
    let lang = args.lang.clone();

    // Windows: force UTF-8 so Japanese/French and box-drawing characters don't mojibake.
    #[cfg(target_os = "windows")]
    unsafe {
        const CP_UTF8: u32 = 65001;
        let _ = SetConsoleOutputCP(CP_UTF8);
        let _ = SetConsoleCP(CP_UTF8);
    }

    // ── Panic hook — restore terminal before printing the panic message ──────
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Best-effort restore; ignore errors (we're already panicking).
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture,
        );
        let _ = crossterm::execute!(std::io::stderr(), crossterm::cursor::Show);
        default_hook(info);
    }));

    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    // ── Run ───────────────────────────────────────────────────────────────────
    let mut app = app::App::new(coder_cfg, observer_cfg, chat_cfg, tool_root, auto_observe, lang);
    let result = events::run_event_loop(&mut app, &mut terminal).await;

    // ── Restore terminal ──────────────────────────────────────────────────────
    // Always restore even if run errored.
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal.show_cursor();

    result
}
