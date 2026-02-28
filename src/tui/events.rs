use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::modes::mode_prompt;
use crate::streaming::StreamToken;
use crate::types::ChatMessage;

use super::agent;
use super::app::{App, Focus, Message, Role};

pub enum AppEvent {
    Key(KeyEvent),
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

    // Track pending stop signals
    let mut coder_stop = false;
    let mut observer_stop = false;

    loop {
        terminal.draw(|f| super::ui::render(f, app))?;

        let ev = tokio::select! {
            maybe_key = event_stream.next() => {
                match maybe_key {
                    Some(Ok(Event::Key(k))) => AppEvent::Key(k),
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
                if handle_key(key, app, &coder_tx, &observer_tx, &mut coder_stop, &mut observer_stop).await? {
                    break;
                }
            }
            AppEvent::CoderToken(token) => {
                handle_coder_token(token, app);
            }
            AppEvent::ObserverToken(token) => {
                handle_observer_token(token, app);
            }
            AppEvent::Tick => {
                maybe_auto_observe(app, &observer_tx).await;
            }
        }

        if app.quit {
            break;
        }
    }

    Ok(())
}

/// Returns true if the app should quit.
async fn handle_key(
    key: KeyEvent,
    app: &mut App,
    coder_tx: &mpsc::Sender<StreamToken>,
    observer_tx: &mpsc::Sender<StreamToken>,
    _coder_stop: &mut bool,
    _observer_stop: &mut bool,
) -> anyhow::Result<bool> {
    use crossterm::event::KeyEventKind;
    // Only react on press (not release) to avoid double-firing on some terminals.
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }

    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    match key.code {
        KeyCode::Char('c') if ctrl => return Ok(true),
        KeyCode::Esc => return Ok(true),

        KeyCode::Tab => {
            app.toggle_focus();
        }

        // Ctrl+A — toggle auto-observe
        KeyCode::Char('a') if ctrl => {
            app.auto_observe = !app.auto_observe;
        }

        // Ctrl+O — manually trigger observer
        KeyCode::Char('o') if ctrl => {
            send_observer_message(app, observer_tx, None).await;
        }

        // Ctrl+L — clear current pane history
        KeyCode::Char('l') if ctrl => {
            app.focused_pane_mut().messages.clear();
            app.focused_pane_mut().scroll = 0;
        }

        // PageUp / PageDown — scroll
        KeyCode::PageUp => {
            let s = app.focused_pane_mut().scroll;
            app.focused_pane_mut().scroll = s.saturating_sub(5);
        }
        KeyCode::PageDown => {
            app.focused_pane_mut().scroll = app.focused_pane_mut().scroll.saturating_add(5);
        }

        // Enter — send message
        KeyCode::Enter if !shift => {
            let focus = app.focus;
            match focus {
                Focus::Coder => send_coder_message(app, coder_tx).await,
                Focus::Observer => send_observer_message(app, observer_tx, None).await,
            }
        }

        // Shift+Enter — insert newline
        KeyCode::Enter if shift => {
            let pane = app.focused_pane_mut();
            pane.textarea.insert_newline();
        }

        // All other keys — delegate to tui-textarea
        _ => {
            let pane = app.focused_pane_mut();
            pane.textarea.input(key);
        }
    }

    Ok(false)
}

fn handle_coder_token(token: StreamToken, app: &mut App) {
    match token {
        StreamToken::Delta(s) => {
            app.coder.push_delta(&s);
            // Auto-scroll
            app.coder.scroll = usize::MAX;
        }
        StreamToken::ToolCall(_) => {}
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
    match token {
        StreamToken::Delta(s) => {
            app.observer.push_delta(&s);
            app.observer.scroll = usize::MAX;
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

async fn send_coder_message(app: &mut App, tx: &mpsc::Sender<StreamToken>) {
    let lines = app.coder.textarea.lines();
    let text: String = lines.join("\n");
    let text = text.trim().to_string();
    if text.is_empty() {
        return;
    }
    if app.coder.streaming {
        return;
    }

    // Clear textarea
    app.coder.textarea = tui_textarea::TextArea::default();
    app.coder.push_user(text.clone());
    app.coder.streaming = true;
    // Start streaming message slot
    app.coder.messages.push(Message::new_streaming(Role::Assistant));

    let history = app.coder.chat_history();
    let cfg = app.coder_cfg.clone();
    let tool_root = app.tool_root.clone();

    // Build system + history + user message list.
    let system = agent::coder_system(mode_prompt(&cfg.mode));
    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    // Push all history except the last user msg (it's already in history from push_user).
    let hist_len = history.len();
    for m in history.iter().take(hist_len.saturating_sub(1)) {
        messages.push(m.clone());
    }
    // The last user turn
    messages.push(ChatMessage { role: "user".to_string(), content: text });

    let tx = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = agent::run_agentic(messages, &cfg, tool_root.as_deref(), tx.clone()).await {
            let _ = tx.send(StreamToken::Error(e.to_string())).await;
        }
    });
}

async fn send_observer_message(app: &mut App, tx: &mpsc::Sender<StreamToken>, override_text: Option<String>) {
    let text = match override_text {
        Some(t) => t,
        None => {
            let lines = app.observer.textarea.lines();
            let t = lines.join("\n").trim().to_string();
            if t.is_empty() {
                return;
            }
            app.observer.textarea = tui_textarea::TextArea::default();
            t
        }
    };

    if app.observer.streaming {
        return;
    }

    app.observer.push_user(text.clone());
    app.observer.streaming = true;
    app.observer.messages.push(Message::new_streaming(Role::Assistant));

    let history = app.observer.chat_history();
    let coder_history = app.coder.chat_history();
    let cfg = app.observer_cfg.clone();

    // Build context: Observer system + recent coder context + observer history + new msg.
    let obs_system = mode_prompt(&cfg.mode);
    let coder_context = if !coder_history.is_empty() {
        let snippet: String = coder_history
            .iter()
            .rev()
            .take(6)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| format!("[{}]: {}", m.role, m.content.chars().take(500).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\n[Recent Coder activity]\n{snippet}")
    } else {
        String::new()
    };

    let system = format!("{obs_system}{coder_context}");
    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    for m in &history {
        messages.push(m.clone());
    }
    messages.push(ChatMessage { role: "user".to_string(), content: text });

    let tx = tx.clone();
    tokio::spawn(async move {
        use crate::config::ProviderKind;
        use crate::streaming::{stream_openai_compat, stream_anthropic};

        let client = reqwest::Client::new();
        let result = match cfg.provider {
            ProviderKind::Anthropic => stream_anthropic(&client, &cfg, &messages, tx.clone()).await,
            _ => stream_openai_compat(&client, &cfg, &messages, None, tx.clone()).await,
        };
        if let Err(e) = result {
            let _ = tx.send(StreamToken::Error(e.to_string())).await;
        }
    });
}

async fn maybe_auto_observe(app: &mut App, observer_tx: &mpsc::Sender<StreamToken>) {
    if let Some(coder_content) = app.auto_observe_trigger() {
        // Record we've handled this message (find index again).
        let idx = app.coder.messages
            .iter()
            .enumerate()
            .rev()
            .find(|(_, m)| {
                matches!(m.role, Role::Assistant) && m.complete && m.content.trim().len() > 40
            })
            .map(|(i, _)| i);
        app.last_auto_obs_idx = idx;

        let prompt = format!(
            "[AUTO-OBSERVE] コーダーが新しいアウトプットを生成した。実況しながら批評せよ。\n\n最新のコーダー出力:\n{}",
            coder_content.chars().take(800).collect::<String>()
        );
        send_observer_message(app, observer_tx, Some(prompt)).await;
    }
}
