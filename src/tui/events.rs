use crossterm::event::{
    Event, EventStream, KeyCode, KeyEvent, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};
use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::modes::{mode_prompt, language_instruction};
use crate::personas::resolve_persona;
use crate::streaming::StreamToken;
use crate::types::ChatMessage;

use super::agent;
use super::app::{App, Focus, Message, Role};

// ── Clipboard ─────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn copy_to_clipboard(text: &str) {
    use clipboard_win::{formats, set_clipboard};
    let _ = set_clipboard(formats::Unicode, text);
}

#[cfg(not(target_os = "windows"))]
fn copy_to_clipboard(_text: &str) {}

// ── Slash command handler ─────────────────────────────────────────────────────

/// Returns true if `text` was a slash command (caller should NOT send to AI).
fn handle_slash_command(text: &str, app: &mut App, focus: Focus) -> bool {
    if !text.starts_with('/') { return false; }

    let (cmd, arg) = match text.find(' ') {
        Some(i) => (&text[..i], text[i + 1..].trim()),
        None    => (text, ""),
    };
    let cmd_lc = cmd.to_ascii_lowercase();

    macro_rules! push {
        ($msg:expr) => {
            match focus {
                Focus::Coder    => app.coder.push_tool($msg),
                Focus::Observer => app.observer.push_tool($msg),
            }
        };
    }

    match cmd_lc.as_str() {
        "/model" => {
            if arg.is_empty() {
                let m = match focus {
                    Focus::Coder    => app.coder_cfg.model.clone(),
                    Focus::Observer => app.observer_cfg.model.clone(),
                };
                push!(format!("現在のモデル: {m}"));
            } else {
                match focus {
                    Focus::Coder => {
                        app.coder_cfg.model = arg.to_string();
                        app.coder.push_tool(format!("✓ Coder モデル → {arg}"));
                    }
                    Focus::Observer => {
                        app.observer_cfg.model = arg.to_string();
                        app.observer.push_tool(format!("✓ Observer モデル → {arg}"));
                    }
                }
            }
        }
        "/persona" => {
            if arg.is_empty() {
                push!(format!("現在のペルソナ: {}", app.coder_cfg.persona));
            } else {
                match resolve_persona(arg) {
                    Ok(p) => {
                        let key = p.key.to_string();
                        app.coder_cfg.persona    = key.clone();
                        app.observer_cfg.persona = key.clone();
                        push!(format!("✓ ペルソナ → {key} (両ペイン)"));
                    }
                    Err(e) => push!(format!("✗ {e}")),
                }
            }
        }
        "/temp" | "/temperature" => {
            if let Ok(t) = arg.parse::<f64>() {
                let t = t.clamp(0.0, 2.0);
                match focus {
                    Focus::Coder => {
                        app.coder_cfg.temperature = t;
                        app.coder.push_tool(format!("✓ 温度 → {t:.2}"));
                    }
                    Focus::Observer => {
                        app.observer_cfg.temperature = t;
                        app.observer.push_tool(format!("✓ 温度 → {t:.2}"));
                    }
                }
            } else {
                push!("使い方: /temp 0.0〜2.0".to_string());
            }
        }
        "/lang" => {
            if arg.is_empty() {
                push!(format!("言語: {}", app.lang));
            } else {
                let v = arg.trim().to_ascii_lowercase();
                if v == "ja" || v == "en" || v == "fr" {
                    app.lang = v.clone();
                    push!(format!("✓ 言語 → {v}"));
                } else {
                    push!("使い方: /lang ja|en|fr".to_string());
                }
            }
        }
        "/root" | "/wd" => {
            if arg.is_empty() {
                let r = app.tool_root.as_deref().unwrap_or("(未設定 = カレントディレクトリ)").to_string();
                push!(format!("作業ディレクトリ: {r}"));
            } else {
                app.tool_root = Some(arg.to_string());
                push!(format!("✓ 作業ディレクトリ → {arg}"));
            }
        }
        "/find" => {
            let q = arg.to_string();
            match focus {
                Focus::Coder => app.coder.find_query = q.clone(),
                Focus::Observer => app.observer.find_query = q.clone(),
            }
            if q.trim().is_empty() {
                push!("✓ 検索フィルタ OFF".to_string());
            } else {
                push!(format!("✓ 検索フィルタ → {q}"));
            }
        }
        "/help" | "/?" => {
            push!("\
 /model <name>       モデル変更 (例: /model gpt-4o)\n\
 /persona <name>     ペルソナ (default/cynical/cheerful/thoughtful/novelist)\n\
 /temp <0.0-2.0>     温度変更\n\
 /lang <ja|en|fr>    言語変更\n\
 /root <path>        作業ディレクトリ変更\n\
 /find <text>        履歴を検索フィルタ (空でOFF)\n\
 Ctrl+Y              最後のAI返答をクリップボードにコピー".to_string());
        }
        _ => push!(format!("不明なコマンド: {cmd}  /help で一覧")),
    }
    true
}

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
                maybe_observer_loop_retry(app, &observer_tx).await;
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

        // Yank (copy) last assistant message to clipboard
        KeyCode::Char('y') if ctrl => {
            let content = {
                let pane = match app.focus { Focus::Coder => &app.coder, Focus::Observer => &app.observer };
                pane.messages.iter().rev()
                    .find(|m| matches!(m.role, Role::Assistant) && m.complete)
                    .map(|m| m.content.clone())
            };
            if let Some(text) = content {
                copy_to_clipboard(&text);
                app.focused_pane_mut().push_tool("✓ クリップボードにコピー".to_string());
            }
        }

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
            // Detect repeated Observer replies and schedule a one-shot diff-only retry.
            // This prevents the common "template critique loop" when nothing new happened.
            if app.observer_loop_retry_budget > 0 {
                let asst: Vec<&Message> = app.observer.messages.iter()
                    .filter(|m| matches!(m.role, Role::Assistant) && m.complete && !m.content.trim().is_empty())
                    .collect();
                if asst.len() >= 2 {
                    let last = asst[asst.len() - 1];
                    if !crate::loop_detect::is_skippable_for_loop(&last.content) {
                        let mut max_sim: f64 = 0.0;
                        for prev in asst[..asst.len() - 1].iter().rev().take(4) {
                            max_sim = max_sim.max(crate::loop_detect::similarity(&last.content, &prev.content));
                        }
                        let detected = last.content.trim().len() >= 180 && max_sim >= 0.80;
                        if detected {
                            app.observer_loop_retry_budget = app.observer_loop_retry_budget.saturating_sub(1);
                            app.observer_loop_pending = Some(max_sim);
                        }
                    }
                }
            }
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

    // Handle slash commands before sending to AI.
    if handle_slash_command(&text, app, Focus::Coder) {
        app.coder.textarea = tui_textarea::TextArea::default();
        return;
    }

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
    // Default tool_root to the current working directory so the model knows where to create files.
    let tool_root = app.tool_root.clone().or_else(|| {
        std::env::current_dir().ok().map(|p| p.to_string_lossy().into_owned())
    });

    let persona_prompt = resolve_persona(&cfg.persona).map(|p| p.prompt).unwrap_or("");
    let lang = language_instruction(Some(&app.lang), &cfg.mode);
    let system = agent::coder_system(persona_prompt, lang);
    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    let hist_len = history.len();
    for m in history.iter().take(hist_len.saturating_sub(1)) {
        messages.push(m.clone());
    }
    messages.push(ChatMessage { role: "user".to_string(), content: text });

    let tx = tx.clone();
    let handle = tokio::spawn(async move {
        if let Err(e) = agent::run_agentic(messages, &cfg, tool_root.as_deref(), tx.clone()).await {
            let _ = tx.send(StreamToken::Error(format!("{e:#}"))).await;
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
            // Handle slash commands before sending to AI.
            if handle_slash_command(&t, app, Focus::Observer) {
                app.observer.textarea = tui_textarea::TextArea::default();
                return;
            }
            app.observer.textarea = tui_textarea::TextArea::default();
            t
        }
    };
    if app.observer.streaming { return; }

    if let Some(handle) = app.observer_task.take() { handle.abort(); }

    app.observer.scroll = 0;
    app.ignore_observer_tokens = false;
    // Each new Observer send gets a single anti-loop retry budget.
    app.observer_loop_retry_budget = 1;
    app.observer_loop_pending = None;
    app.observer.push_user(text.clone());
    app.observer.streaming = true;
    app.observer.messages.push(Message::new_streaming(Role::Assistant));

    let history = app.observer.chat_history();
    let coder_history = app.coder.chat_history();
    let cfg = app.observer_cfg.clone();

    // Inject recent Coder context into Observer system prompt.
    let persona_prompt = resolve_persona(&cfg.persona).map(|p| p.prompt).unwrap_or("");
    let lang = language_instruction(Some(&app.lang), &cfg.mode);
    let obs_mode_prompt = mode_prompt(&cfg.mode);
    let obs_system = format!("{obs_mode_prompt}\n\n[Language]\n{lang}\n\n[Persona]\n{persona_prompt}");
    let obs_system = obs_system.trim_end().to_string();
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

    let system = format!("{obs_system}{coder_context}").trim_end().to_string();
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
        let snippet = content.chars().take(800).collect::<String>();
        let prompt = match app.lang.as_str() {
            "fr" => format!(
                "[AUTO-OBSERVE] Le Coder vient de produire une nouvelle sortie. Fais une critique en mode commentaire live.\n\nDernière sortie du Coder:\n{snippet}"
            ),
            "en" => format!(
                "[AUTO-OBSERVE] The Coder produced new output. Commentate and critique live.\n\nLatest Coder output:\n{snippet}"
            ),
            _ => format!(
                "[AUTO-OBSERVE] コーダーが新しいアウトプットを生成した。実況しながら批評せよ。\n\n最新のコーダー出力:\n{snippet}"
            ),
        };
        send_observer_message(app, observer_tx, Some(prompt)).await;
    }
}

async fn maybe_observer_loop_retry(app: &mut App, observer_tx: &mpsc::Sender<StreamToken>) {
    let Some(sim) = app.observer_loop_pending.take() else { return; };
    if app.observer.streaming { return; }

    // Best-effort: abort the previous task handle (it should already be complete).
    if let Some(handle) = app.observer_task.take() { handle.abort(); }

    // Build the retry request using the already-visible history (including the repeated reply),
    // but do not add a visible user message for the loop fix.
    let history = app.observer.chat_history();
    let coder_history = app.coder.chat_history();
    let cfg = app.observer_cfg.clone();

    let persona_prompt = resolve_persona(&cfg.persona).map(|p| p.prompt).unwrap_or("");
    let lang = language_instruction(Some(&app.lang), &cfg.mode);
    let obs_mode_prompt = mode_prompt(&cfg.mode);
    let obs_system = format!("{obs_mode_prompt}\n\n[Language]\n{lang}\n\n[Persona]\n{persona_prompt}");
    let obs_system = obs_system.trim_end().to_string();
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

    let system = format!("{obs_system}{coder_context}").trim_end().to_string();
    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    for m in &history { messages.push(m.clone()); }

    let loop_fix = match app.lang.as_str() {
        "fr" => format!(
            "CORRECTION BOUCLE: Ton dernier message se répète (sim={}%). Fais une critique NOUVELLE basée UNIQUEMENT sur les informations NOUVELLES depuis le message précédent. Ne répète pas les mêmes proposals. S'il n'y a rien de nouveau, réponds exactement: [Observer] No new critique. Loop detected.",
            (sim * 100.0).round() as i64
        ),
        "en" => format!(
            "LOOP FIX: Your last message repeated the same critique (sim={}%). Write a NEW critique ONLY based on NEW information since your previous message. Do not restate the same proposals. If there is no new signal, reply exactly: [Observer] No new critique. Loop detected.",
            (sim * 100.0).round() as i64
        ),
        _ => format!(
            "LOOP FIX: 直前の批評がほぼ同一です (sim={}%)。前回から増えた情報に基づく「新しい」批評だけを書いてください。同じ提案の焼き直しは禁止。新しい指摘が無い場合は、次の1行だけを厳密に出力: [Observer] No new critique. Loop detected.",
            (sim * 100.0).round() as i64
        ),
    };
    messages.push(ChatMessage { role: "user".to_string(), content: loop_fix });

    // Overwrite the last assistant message if it's the tail; otherwise append a new streaming assistant.
    if let Some(idx) = app.observer.messages.iter().rposition(|m| matches!(m.role, Role::Assistant) && m.complete) {
        if idx + 1 == app.observer.messages.len() {
            if let Some(m) = app.observer.messages.get_mut(idx) {
                m.content.clear();
                m.complete = false;
            }
        } else {
            app.observer.messages.push(Message::new_streaming(Role::Assistant));
        }
    } else {
        app.observer.messages.push(Message::new_streaming(Role::Assistant));
    }

    app.observer.scroll = 0;
    app.ignore_observer_tokens = false;
    app.observer.streaming = true;

    let tx = observer_tx.clone();
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
