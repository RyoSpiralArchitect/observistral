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

use crate::config::PartialConfig;

#[derive(clap::Args, Debug, Clone)]
pub struct TuiArgs {
    /// Model to use for the Observer pane (defaults to same as Coder).
    #[arg(long)]
    pub observer_model: Option<String>,

    /// Working directory for exec tool calls.
    #[arg(long)]
    pub tool_root: Option<String>,

    /// Enable auto-observe (Observer auto-fires after each Coder response).
    #[arg(long)]
    pub auto_observe: bool,
}

pub async fn run(args: TuiArgs, partial_cfg: PartialConfig) -> Result<()> {
    // Resolve the Coder config from the shared partial config.
    let coder_cfg = partial_cfg.clone().resolve().context("failed to resolve coder config")?;

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

    let tool_root = args.tool_root.clone();
    let auto_observe = args.auto_observe;

    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    // ── Run ───────────────────────────────────────────────────────────────────
    let mut app = app::App::new(coder_cfg, observer_cfg, tool_root, auto_observe);
    let result = events::run_event_loop(&mut app, &mut terminal).await;

    // ── Restore terminal ──────────────────────────────────────────────────────
    // Always restore even if run errored.
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal.show_cursor();

    result
}
