use crossterm::event::{
    Event, EventStream, KeyCode, KeyEvent, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};
use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::modes::mode_prompt;
use crate::streaming::StreamToken;
use crate::types::ChatMessage;

use super::agent;
use super::app::{App, Focus, Message, Role};

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    CoderToken(StreamToken),
    ObserverToken(StreamToken),
    Tick,
}

pub async fn run_event_loop(
    app: &mut App,
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
) -> anyhow::Result<()> {
    let (coder_tx, mut coder_rx) = mpsc::channel::<StreamToken>(256);
    let (observer_tx, mut observer_rx) = mpsc::channel::<StreamToken>(256);
    let mut event_stream = EventStream::new();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(100));

    loop {
        terminal.draw(|f| super::ui::render(f, app))?;

        let ev = tokio::select! {
            maybe_key = event_stream.next() => {
                match maybe_key {
                    Some(Ok(Event::Key(k)))   => AppEvent::Key(k),
                    Some(Ok(Event::Mouse(m))) => AppEvent::Mouse(m),
                    Some(Err(e)) => return Err(e.into()),
                    _ => continue,
                }
            }
            Some(token) = coder_rx.recv() => AppEvent::CoderToken(token),
            Some(token) = observer_rx.recv() => AppEvent::ObserverToken(token),
            _ = tick.tick() => AppEvent::Tick,
        };

        match ev {
            AppEvent::Key(key) => {
                if handle_key(key, app, &coder_tx, &observer_tx).await? {
                    break;
                }
            }
            AppEvent::Mouse(m)                  => handle_mouse(m, app),
            AppEvent::CoderToken(token)         => handle_coder_token(token, app),
            AppEvent::ObserverToken(token)      => handle_observer_token(token, app),
            AppEvent::Tick => {
                app.tick_count = app.tick_count.wrapping_add(1);
                maybe_auto_observe(app, &observer_tx).await;
            }
        }

        if app.quit { break; }
    }

    // Abort any in-flight tasks on clean exit.
    if let Some(t) = app.coder_task.take() { t.abort(); }
    if let Some(t) = app.observer_task.take() { t.abort(); }

    Ok(())
}

// ── Mouse handler ─────────────────────────────────────────────────────────────

fn handle_mouse(mouse: MouseEvent, app: &mut App) {
    // Query terminal dimensions for hit-testing (fall back to 80×24).
    let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));

    // The layout produced by ui::render:
    //   row 0-1    → header (2 rows)
    //   row 2..h-4 → body panes
    //   row h-3..h → input box (3 rows)
    // Horizontal: left 55 % = Coder, right 45 % = Observer.
    let coder_w = (term_w as u32 * 55 / 100) as u16;
    let body_start: u16 = 2;
    let body_end: u16 = term_h.saturating_sub(3);

    match mouse.kind {
        // Scroll wheel: scroll whichever pane the cursor is over.
        MouseEventKind::ScrollUp => {
            let pane = if mouse.column < coder_w {
                &mut app.coder
            } else {
                &mut app.observer
            };
            pane.scroll = pane.scroll.saturating_add(3);
        }
        MouseEventKind::ScrollDown => {
            let pane = if mouse.column < coder_w {
                &mut app.coder
            } else {
                &mut app.observer
            };
            pane.scroll = pane.scroll.saturating_sub(3);
        }

        // Left-click in the body: focus that pane.
        MouseEventKind::Down(MouseButton::Left) => {
            if mouse.row >= body_start && mouse.row < body_end {
                app.focus = if mouse.column < coder_w {
                    Focus::Coder
                } else {
                    Focus::Observer
                };
            }
        }

        _ => {}
    }
}

// ── Key handler ───────────────────────────────────────────────────────────────

/// Returns true if the app should quit.
async fn handle_key(
    key: KeyEvent,
    app: &mut App,
    coder_tx: &mpsc::Sender<StreamToken>,
    observer_tx: &mpsc::Sender<StreamToken>,
) -> anyhow::Result<bool> {
    use crossterm::event::KeyEventKind;
    if key.kind == KeyEventKind::Release { return Ok(false); }

    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    match key.code {
        // Quit
        KeyCode::Char('c') if ctrl => return Ok(true),
        KeyCode::Esc => return Ok(true),

        // Switch focus
        KeyCode::Tab => app.toggle_focus(),

        // Toggle auto-observe
        KeyCode::Char('a') if ctrl => {
            app.auto_observe = !app.auto_observe;
        }

        // Trigger Observer manually
        KeyCode::Char('o') if ctrl => {
            send_observer_message(app, observer_tx, None).await;
        }

        // Clear current pane
        KeyCode::Char('l') if ctrl => {
            let pane = app.focused_pane_mut();
            pane.messages.clear();
            pane.scroll = 0;
        }

        // Stop streaming (Ctrl+K)
        KeyCode::Char('k') if ctrl => {
            match app.focus {
                Focus::Coder => {
                    if let Some(handle) = app.coder_task.take() {
                        handle.abort();
                    }
                    app.ignore_coder_tokens = true;
                    app.coder.finish_stream();
                    app.coder.push_tool("(ストリーミング停止)".to_string());
                }
                Focus::Observer => {
                    if let Some(handle) = app.observer_task.take() {
                        handle.abort();
                    }
                    app.ignore_observer_tokens = true;
                    app.observer.finish_stream();
                    app.observer.push_tool("(ストリーミング停止)".to_string());
                }
            }
        }

        // Scroll (lines-from-bottom semantics: 0 = pinned, N = above)
        KeyCode::PageUp => {
            app.focused_pane_mut().scroll = app.focused_pane_mut().scroll.saturating_add(5);
        }
        KeyCode::PageDown => {
            app.focused_pane_mut().scroll = app.focused_pane_mut().scroll.saturating_sub(5);
        }
        KeyCode::Home => {
            app.focused_pane_mut().scroll = usize::MAX; // jump to very top
        }
        KeyCode::End => {
            app.focused_pane_mut().scroll = 0; // re-pin to bottom
        }

        // Send message
        KeyCode::Enter if !shift => {
            match app.focus {
                Focus::Coder => send_coder_message(app, coder_tx).await,
                Focus::Observer => send_observer_message(app, observer_tx, None).await,
            }
        }

        // Insert newline
        KeyCode::Enter if shift => {
            app.focused_pane_mut().textarea.insert_newline();
        }

        // Pass everything else to tui-textarea
        _ => {
            app.focused_pane_mut().textarea.input(key);
        }
    }

    Ok(false)
}

// ── Token handlers ────────────────────────────────────────────────────────────

fn handle_coder_token(token: StreamToken, app: &mut App) {
    if app.ignore_coder_tokens { return; }
    match token {
        StreamToken::Delta(s) => {
            app.coder.push_delta(&s);
            // scroll = 0 means pinned; don't disturb if user has scrolled up.
        }
        StreamToken::ToolCall(_) => {
            app.coder_iter = app.coder_iter.saturating_add(1);
        }
        StreamToken::Done => {
            app.coder.finish_stream();
        }
        StreamToken::Error(e) => {
            app.coder.push_tool(format!("ERROR: {e}"));
            app.coder.finish_stream();
        }
    }
}

fn handle_observer_token(token: StreamToken, app: &mut App) {
    if app.ignore_observer_tokens { return; }
    match token {
        StreamToken::Delta(s) => {
            app.observer.push_delta(&s);
        }
        StreamToken::ToolCall(_) => {}
        StreamToken::Done => {
            app.observer.finish_stream();
        }
        StreamToken::Error(e) => {
            app.observer.push_tool(format!("ERROR: {e}"));
            app.observer.finish_stream();
        }
    }
}

// ── Send helpers ──────────────────────────────────────────────────────────────

async fn send_coder_message(app: &mut App, tx: &mpsc::Sender<StreamToken>) {
    let lines = app.coder.textarea.lines();
    let text = lines.join("\n");
    let text = text.trim().to_string();
    if text.is_empty() || app.coder.streaming { return; }

    // Abort any previous task before starting a new one.
    if let Some(handle) = app.coder_task.take() { handle.abort(); }

    // Reset state for the new send.
    app.coder.textarea = tui_textarea::TextArea::default();
    app.coder_iter = 0;
    app.coder.scroll = 0;         // pin to bottom for new output
    app.ignore_coder_tokens = false;

    app.coder.push_user(text.clone());
    app.coder.streaming = true;
    app.coder.messages.push(Message::new_streaming(Role::Assistant));

    let history = app.coder.chat_history();
    let cfg = app.coder_cfg.clone();
    let tool_root = app.tool_root.clone();

    let system = agent::coder_system(mode_prompt(&cfg.mode));
    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    let hist_len = history.len();
    for m in history.iter().take(hist_len.saturating_sub(1)) {
        messages.push(m.clone());
    }
    messages.push(ChatMessage { role: "user".to_string(), content: text });

    let tx = tx.clone();
    let handle = tokio::spawn(async move {
        if let Err(e) = agent::run_agentic(messages, &cfg, tool_root.as_deref(), tx.clone()).await {
            let _ = tx.send(StreamToken::Error(e.to_string())).await;
        }
    });
    app.coder_task = Some(handle);
}

async fn send_observer_message(
    app: &mut App,
    tx: &mpsc::Sender<StreamToken>,
    override_text: Option<String>,
) {
    let text = match override_text {
        Some(t) => t,
        None => {
            let t = app.observer.textarea.lines().join("\n").trim().to_string();
            if t.is_empty() { return; }
            app.observer.textarea = tui_textarea::TextArea::default();
            t
        }
    };
    if app.observer.streaming { return; }

    if let Some(handle) = app.observer_task.take() { handle.abort(); }

    app.observer.scroll = 0;
    app.ignore_observer_tokens = false;
    app.observer.push_user(text.clone());
    app.observer.streaming = true;
    app.observer.messages.push(Message::new_streaming(Role::Assistant));

    let history = app.observer.chat_history();
    let coder_history = app.coder.chat_history();
    let cfg = app.observer_cfg.clone();

    // Inject recent Coder context into Observer system prompt.
    let obs_system = mode_prompt(&cfg.mode);
    let coder_context = if !coder_history.is_empty() {
        let snippet = coder_history
            .iter()
            .rev()
            .take(6)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| {
                format!(
                    "[{}]: {}",
                    m.role,
                    m.content.chars().take(500).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\n[Recent Coder activity]\n{snippet}")
    } else {
        String::new()
    };

    let system = format!("{obs_system}{coder_context}");
    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    for m in &history { messages.push(m.clone()); }
    messages.push(ChatMessage { role: "user".to_string(), content: text });

    let tx = tx.clone();
    let handle = tokio::spawn(async move {
        use crate::config::ProviderKind;
        use crate::streaming::{stream_anthropic, stream_openai_compat};
        let client = reqwest::Client::new();
        let result = match cfg.provider {
            ProviderKind::Anthropic => stream_anthropic(&client, &cfg, &messages, tx.clone()).await,
            _ => stream_openai_compat(&client, &cfg, &messages, None, tx.clone()).await,
        };
        if let Err(e) = result {
            let _ = tx.send(StreamToken::Error(e.to_string())).await;
        }
    });
    app.observer_task = Some(handle);
}

// ── Auto-observe ──────────────────────────────────────────────────────────────

async fn maybe_auto_observe(app: &mut App, observer_tx: &mpsc::Sender<StreamToken>) {
    if let Some((idx, content)) = app.auto_observe_trigger() {
        app.last_auto_obs_idx = Some(idx);
        let prompt = format!(
            "[AUTO-OBSERVE] コーダーが新しいアウトプットを生成した。実況しながら批評せよ。\n\n最新のコーダー出力:\n{}",
            content.chars().take(800).collect::<String>()
        );
        send_observer_message(app, observer_tx, Some(prompt)).await;
    }
}
