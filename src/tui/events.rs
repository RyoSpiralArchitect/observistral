use crossterm::event::{
    Event, EventStream, KeyCode, KeyEvent, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use crate::modes::{mode_prompt, language_instruction};
use crate::personas::resolve_persona;
use crate::providers;
use crate::streaming::StreamToken;
use crate::types::{ChatMessage, ChatRequest};

use super::agent;
use super::app::{App, Focus, RightTab, Task, TaskPhase, TaskTarget, Message, Role};

// ── Clipboard ─────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn copy_to_clipboard(text: &str) {
    use clipboard_win::{formats, set_clipboard};
    let _ = set_clipboard(formats::Unicode, text);
}

#[cfg(not(target_os = "windows"))]
fn copy_to_clipboard(_text: &str) {}

// ── Slash command handler ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneId {
    Coder,
    Observer,
    Chat,
}

/// Returns true if `text` was a slash command (caller should NOT send to AI).
fn handle_slash_command(text: &str, app: &mut App, pane: PaneId) -> bool {
    if !text.starts_with('/') {
        return false;
    }

    let (cmd, arg) = match text.find(' ') {
        Some(i) => (&text[..i], text[i + 1..].trim()),
        None => (text, ""),
    };
    let cmd_lc = cmd.to_ascii_lowercase();

    macro_rules! push {
        ($msg:expr) => {
            match pane {
                PaneId::Coder => app.coder.push_tool($msg),
                PaneId::Observer => app.observer.push_tool($msg),
                PaneId::Chat => app.chat.push_tool($msg),
            }
        };
    }

    match cmd_lc.as_str() {
        "/model" => {
            if arg.is_empty() {
                let m = match pane {
                    PaneId::Coder => app.coder_cfg.model.clone(),
                    PaneId::Observer => app.observer_cfg.model.clone(),
                    PaneId::Chat => app.chat_cfg.model.clone(),
                };
                push!(format!("model: {m}"));
            } else {
                match pane {
                    PaneId::Coder => {
                        app.coder_cfg.model = arg.to_string();
                        push!(format!("coder model <- {arg}"));
                    }
                    PaneId::Observer => {
                        app.observer_cfg.model = arg.to_string();
                        push!(format!("observer model <- {arg}"));
                    }
                    PaneId::Chat => {
                        app.chat_cfg.model = arg.to_string();
                        push!(format!("chat model <- {arg}"));
                    }
                }
            }
        }
        "/persona" => {
            if arg.is_empty() {
                let p = match pane {
                    PaneId::Coder => app.coder_cfg.persona.as_str(),
                    PaneId::Observer => app.observer_cfg.persona.as_str(),
                    PaneId::Chat => app.chat_cfg.persona.as_str(),
                };
                push!(format!("persona: {p}"));
            } else {
                match resolve_persona(arg) {
                    Ok(p) => {
                        let key = p.key.to_string();
                        match pane {
                            PaneId::Coder => app.coder_cfg.persona = key.clone(),
                            PaneId::Observer => app.observer_cfg.persona = key.clone(),
                            PaneId::Chat => app.chat_cfg.persona = key.clone(),
                        }
                        push!(format!("persona <- {key}"));
                    }
                    Err(e) => push!(format!("error: {e}")),
                }
            }
        }
        "/temp" | "/temperature" => {
            if arg.is_empty() {
                let t = match pane {
                    PaneId::Coder => app.coder_cfg.temperature,
                    PaneId::Observer => app.observer_cfg.temperature,
                    PaneId::Chat => app.chat_cfg.temperature,
                };
                push!(format!("temperature: {t:.2}"));
            } else if let Ok(t0) = arg.parse::<f64>() {
                let t = t0.clamp(0.0, 2.0);
                match pane {
                    PaneId::Coder => app.coder_cfg.temperature = t,
                    PaneId::Observer => app.observer_cfg.temperature = t,
                    PaneId::Chat => app.chat_cfg.temperature = t,
                }
                push!(format!("temperature <- {t:.2}"));
            } else {
                push!("usage: /temp 0.0-2.0".to_string());
            }
        }
        "/lang" => {
            if arg.is_empty() {
                push!(format!("lang: {}", app.lang));
            } else {
                let v = arg.trim().to_ascii_lowercase();
                if v == "ja" || v == "en" || v == "fr" {
                    app.lang = v.clone();
                    push!(format!("lang <- {v}"));
                } else {
                    push!("usage: /lang ja|en|fr".to_string());
                }
            }
        }
        "/root" | "/wd" => {
            if arg.is_empty() {
                let r = app.tool_root.as_deref().unwrap_or("(default: current dir)");
                push!(format!("tool_root: {r}"));
            } else {
                app.tool_root = Some(arg.to_string());
                push!(format!("tool_root <- {arg}"));
            }
        }
        "/find" => {
            let q = arg.to_string();
            match pane {
                PaneId::Coder => app.coder.find_query = q.clone(),
                PaneId::Observer => app.observer.find_query = q.clone(),
                PaneId::Chat => app.chat.find_query = q.clone(),
            }
            if q.trim().is_empty() {
                push!("find: off".to_string());
            } else {
                push!(format!("find: {q}"));
            }
        }
        "/help" | "/?" => {
            push!(
                "/model <name>       set model\n\
/persona <name>     set persona\n\
/temp <0.0-2.0>     set temperature\n\
/lang <ja|en|fr>    set language\n\
/root <path>        set tool_root\n\
/find <text>        filter history\n\
Ctrl+R              cycle right tab\n"
                    .to_string()
            );
        }
        _ => push!(format!("unknown command: {cmd} (try /help)")),
    }

    true
}

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    CoderToken(StreamToken),
    ObserverToken(StreamToken),
    ChatToken(StreamToken),
    TasksPlanned(Vec<Task>),
    TaskPlanError(String),
    Tick,
}

#[derive(Debug, Deserialize)]
struct PlannedTasks {
    tasks: Vec<PlannedTask>,
}

#[derive(Debug, Deserialize)]
struct PlannedTask {
    target: String,
    title: String,
    body: String,
    phase: Option<String>,
    priority: Option<u8>,
}

pub async fn run_event_loop(
    app: &mut App,
    terminal: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
) -> anyhow::Result<()> {
    let (coder_tx, mut coder_rx) = mpsc::channel::<StreamToken>(256);
    let (observer_tx, mut observer_rx) = mpsc::channel::<StreamToken>(256);
    let (chat_tx, mut chat_rx) = mpsc::channel::<StreamToken>(256);
    // Internal app events (background planners, etc.).
    let (internal_tx, mut internal_rx) = mpsc::channel::<AppEvent>(64);
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
            Some(token) = chat_rx.recv() => AppEvent::ChatToken(token),
            Some(ev2) = internal_rx.recv() => ev2,
            _ = tick.tick() => AppEvent::Tick,
        };

        match ev {
            AppEvent::Key(key) => {
                if handle_key(key, app, &coder_tx, &observer_tx, &chat_tx, &internal_tx).await? {
                    break;
                }
            }
            AppEvent::Mouse(m)                  => handle_mouse(m, app),
            AppEvent::CoderToken(token)         => handle_coder_token(token, app),
            AppEvent::ObserverToken(token)      => handle_observer_token(token, app),
            AppEvent::ChatToken(token)          => handle_chat_token(token, app),
            AppEvent::TasksPlanned(tasks)       => handle_tasks_planned(tasks, app),
            AppEvent::TaskPlanError(e)          => handle_task_plan_error(e, app),
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
    if let Some(t) = app.chat_task.take() { t.abort(); }

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
    // Horizontal: left 55 % = Coder, right 45 % = Right tab (Observer/Chat/Tasks).
    let coder_w = (term_w as u32 * 55 / 100) as u16;
    let body_start: u16 = 2;
    let body_end: u16 = term_h.saturating_sub(3);

    match mouse.kind {
        // Scroll wheel: scroll whichever pane the cursor is over.
        MouseEventKind::ScrollUp => {
            if mouse.column < coder_w {
                app.coder.scroll = app.coder.scroll.saturating_add(3);
            } else {
                match app.right_tab {
                    RightTab::Observer => app.observer.scroll = app.observer.scroll.saturating_add(3),
                    RightTab::Chat => app.chat.scroll = app.chat.scroll.saturating_add(3),
                    RightTab::Tasks => app.tasks_cursor = app.tasks_cursor.saturating_sub(1),
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if mouse.column < coder_w {
                app.coder.scroll = app.coder.scroll.saturating_sub(3);
            } else {
                match app.right_tab {
                    RightTab::Observer => app.observer.scroll = app.observer.scroll.saturating_sub(3),
                    RightTab::Chat => app.chat.scroll = app.chat.scroll.saturating_sub(3),
                    RightTab::Tasks => {
                        if !app.tasks.is_empty() {
                            app.tasks_cursor = (app.tasks_cursor + 1).min(app.tasks.len().saturating_sub(1));
                        }
                    }
                }
            }
        }

        // Left-click in the body: focus that pane.
        MouseEventKind::Down(MouseButton::Left) => {
            if mouse.row >= body_start && mouse.row < body_end {
                app.focus = if mouse.column < coder_w { Focus::Coder } else { Focus::Right };
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
    chat_tx: &mpsc::Sender<StreamToken>,
    internal_tx: &mpsc::Sender<AppEvent>,
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

        // Cycle right-side tab (Observer/Chat/Tasks)
        KeyCode::Char('r') if ctrl => app.cycle_right_tab(),

        // Yank (copy) last assistant message to clipboard
        KeyCode::Char('y') if ctrl => {
            let content = {
                let pane = match app.focus {
                    Focus::Coder => &app.coder,
                    Focus::Right => match app.right_tab {
                        RightTab::Observer => &app.observer,
                        RightTab::Chat => &app.chat,
                        RightTab::Tasks => &app.observer,
                    },
                };
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
        KeyCode::Char('k') if ctrl && app.focus == Focus::Right && app.right_tab == RightTab::Chat => {
            if let Some(handle) = app.chat_task.take() {
                handle.abort();
            }
            app.ignore_chat_tokens = true;
            app.chat.finish_stream();
            app.chat.push_tool("(stream canceled)".to_string());
        }
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
                Focus::Right => {
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
            if app.focus == Focus::Right && app.right_tab == RightTab::Tasks {
                app.tasks_cursor = app.tasks_cursor.saturating_sub(5);
            } else {
                app.focused_pane_mut().scroll = app.focused_pane_mut().scroll.saturating_add(5);
            }
        }
        KeyCode::PageDown => {
            if app.focus == Focus::Right && app.right_tab == RightTab::Tasks {
                if !app.tasks.is_empty() {
                    app.tasks_cursor = (app.tasks_cursor + 5).min(app.tasks.len().saturating_sub(1));
                }
            } else {
                app.focused_pane_mut().scroll = app.focused_pane_mut().scroll.saturating_sub(5);
            }
        }
        KeyCode::Home => {
            if app.focus == Focus::Right && app.right_tab == RightTab::Tasks {
                app.tasks_cursor = 0;
            } else {
                app.focused_pane_mut().scroll = usize::MAX; // jump to very top
            }
        }
        KeyCode::End => {
            if app.focus == Focus::Right && app.right_tab == RightTab::Tasks {
                if !app.tasks.is_empty() {
                    app.tasks_cursor = app.tasks.len().saturating_sub(1);
                }
            } else {
                app.focused_pane_mut().scroll = 0; // re-pin to bottom
            }
        }

        // Tasks selection
        KeyCode::Up if app.focus == Focus::Right && app.right_tab == RightTab::Tasks => {
            app.tasks_cursor = app.tasks_cursor.saturating_sub(1);
        }
        KeyCode::Down if app.focus == Focus::Right && app.right_tab == RightTab::Tasks => {
            if !app.tasks.is_empty() {
                app.tasks_cursor = (app.tasks_cursor + 1).min(app.tasks.len().saturating_sub(1));
            }
        }
        KeyCode::Char(' ') if app.focus == Focus::Right && app.right_tab == RightTab::Tasks => {
            if let Some(t) = app.tasks.get_mut(app.tasks_cursor) {
                t.done = !t.done;
            }
        }

        // Send message
        KeyCode::Enter if !shift => {
            match app.focus {
                Focus::Coder => send_coder_message(app, coder_tx).await,
                Focus::Right => match app.right_tab {
                    RightTab::Observer => send_observer_message(app, observer_tx, None).await,
                    RightTab::Chat => send_chat_message(app, chat_tx, internal_tx).await,
                    RightTab::Tasks => dispatch_selected_task(app, coder_tx, observer_tx).await,
                },
            }
        }

        // Insert newline
        KeyCode::Enter if shift => {
            match app.focus {
                Focus::Coder => app.coder.textarea.insert_newline(),
                Focus::Right => match app.right_tab {
                    RightTab::Observer => app.observer.textarea.insert_newline(),
                    RightTab::Chat => app.chat.textarea.insert_newline(),
                    RightTab::Tasks => {}
                },
            }
        }

        // Pass everything else to tui-textarea
        _ => {
            match app.focus {
                Focus::Coder => {
                    app.coder.textarea.input(key);
                }
                Focus::Right => match app.right_tab {
                    RightTab::Observer => {
                        app.observer.textarea.input(key);
                    }
                    RightTab::Chat => {
                        app.chat.textarea.input(key);
                    }
                    RightTab::Tasks => {}
                },
            }
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

fn handle_chat_token(token: StreamToken, app: &mut App) {
    if app.ignore_chat_tokens {
        return;
    }
    match token {
        StreamToken::Delta(s) => {
            app.chat.push_delta(&s);
        }
        StreamToken::ToolCall(_) => {}
        StreamToken::Done => {
            app.chat.finish_stream();
        }
        StreamToken::Error(e) => {
            app.chat.push_tool(format!("ERROR: {e}"));
            app.chat.finish_stream();
        }
    }
}

fn handle_tasks_planned(tasks: Vec<Task>, app: &mut App) {
    app.planning_tasks = false;
    if tasks.is_empty() {
        app.chat.push_tool("(task router) no tasks planned".to_string());
        return;
    }
    let n = tasks.len();
    app.tasks.extend(tasks);
    if app.tasks_cursor >= app.tasks.len() {
        app.tasks_cursor = app.tasks.len().saturating_sub(1);
    }
    app.chat.push_tool(format!(
        "(task router) planned {n} task(s) - Ctrl+R to view Tasks"
    ));
}

fn handle_task_plan_error(e: String, app: &mut App) {
    app.planning_tasks = false;
    app.chat.push_tool(format!("(task router error) {e}"));
}

/// Extract `@path` tokens from a message (unique, ordered).
/// Accepts patterns like @src/main.rs, @README.md, @dir/sub/file.txt.
fn parse_at_refs(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for word in text.split_whitespace() {
        if !word.starts_with('@') { continue; }
        let path = word.trim_start_matches('@');
        // Strip common trailing punctuation.
        let path = path.trim_end_matches(|c: char| matches!(c, ',' | ')' | ']' | ';' | ':' | '.'));
        if path.is_empty() { continue; }
        if seen.insert(path.to_string()) {
            refs.push(path.to_string());
        }
    }
    refs
}

async fn send_coder_with_text(app: &mut App, tx: &mpsc::Sender<StreamToken>, text: String) {
    let text = text.trim().to_string();
    if text.is_empty() || app.coder.streaming {
        return;
    }

    // Handle slash commands before sending to AI.
    if handle_slash_command(&text, app, PaneId::Coder) {
        return;
    }

    // Abort any previous task before starting a new one.
    if let Some(handle) = app.coder_task.take() { handle.abort(); }

    // Reset state for the new send.
    app.coder_iter = 0;
    app.coder.scroll = 0;         // pin to bottom for new output
    app.ignore_coder_tokens = false;

    // Resolve tool_root early (needed for @ref file reads below).
    let tool_root = app.tool_root.clone().or_else(|| {
        std::env::current_dir().ok().map(|p| p.to_string_lossy().into_owned())
    });

    // Expand @file references: read files and collect system messages to inject.
    let at_refs = parse_at_refs(&text);
    let mut at_ref_messages: Vec<ChatMessage> = Vec::new();
    app.coder.push_user(text.clone());
    for ref_path in &at_refs {
        let (content, is_err) = crate::file_tools::tool_read_file(ref_path, tool_root.as_deref());
        if is_err {
            app.coder.push_tool(format!("📎 @{ref_path}: not found"));
        } else {
            // Header line is "[path] (N lines, B bytes)" — use it as the notification.
            let header = content.lines().next().unwrap_or(ref_path.as_str()).to_string();
            app.coder.push_tool(format!("📎 injected: {header}"));
            at_ref_messages.push(ChatMessage {
                role: "system".to_string(),
                content: format!("[@{ref_path}]\n{content}"),
            });
        }
    }
    app.coder.streaming = true;
    app.coder.messages.push(Message::new_streaming(Role::Assistant));

    let history = app.coder.chat_history();
    let cfg = app.coder_cfg.clone();
    let max_iters = app.coder_max_iters.unwrap_or(agent::DEFAULT_MAX_ITERS);

    let persona_prompt = resolve_persona(&cfg.persona).map(|p| p.prompt).unwrap_or("");
    let lang = language_instruction(Some(&app.lang), &cfg.mode);
    let system = agent::coder_system(persona_prompt, lang);
    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    let hist_len = history.len();
    for m in history.iter().take(hist_len.saturating_sub(1)) {
        messages.push(m.clone());
    }
    // Inject @file system messages immediately before the user turn.
    messages.extend(at_ref_messages);
    messages.push(ChatMessage { role: "user".to_string(), content: text });

    // Scan project context once per session (guarded by stack_label being None).
    let project_context: Option<String> = if app.project_stack_label.is_none() {
        if let Some(ref root) = tool_root {
            if let Some(ctx) = crate::project::ProjectContext::scan(root).await {
                app.project_stack_label = Some(ctx.stack_label());
                Some(ctx.to_context_text())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let tx = tx.clone();
    let handle = tokio::spawn(async move {
        if let Err(e) = agent::run_agentic(messages, &cfg, tool_root.as_deref(), max_iters, tx.clone(), project_context).await {
            let _ = tx.send(StreamToken::Error(format!("{e:#}"))).await;
        }
    });
    app.coder_task = Some(handle);
}

async fn send_coder_message(app: &mut App, tx: &mpsc::Sender<StreamToken>) {
    let text = app.coder.textarea.lines().join("\n").trim().to_string();
    if text.is_empty() || app.coder.streaming {
        return;
    }

    // Handle slash commands before sending to AI.
    if handle_slash_command(&text, app, PaneId::Coder) {
        app.coder.textarea = tui_textarea::TextArea::default();
        return;
    }

    app.coder.textarea = tui_textarea::TextArea::default();
    send_coder_with_text(app, tx, text).await;
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
            if handle_slash_command(&t, app, PaneId::Observer) {
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

    // Build Observer system prompt. Persona is intentionally excluded: the Observer mode
    // has its own strict critique tone ("NO padding, ruthlessly honest") that must not be
    // softened by a cheerful/novelist/etc persona assigned to the Coder pane.
    let lang = language_instruction(Some(&app.lang), &cfg.mode);
    let obs_mode_prompt = mode_prompt(&cfg.mode);
    let obs_system = format!("{obs_mode_prompt}\n\n[Language]\n{lang}");
    let obs_system = obs_system.trim_end().to_string();
    // Inject recent Coder history. Use more messages and wider window than before so
    // Observer can see the full arc of what the Coder did, not just a narrow snippet.
    let coder_context = if !coder_history.is_empty() {
        let snippet = coder_history
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| {
                format!(
                    "[{}]: {}",
                    m.role,
                    m.content.chars().take(800).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\n[Recent Coder activity — last 10 turns]\n{snippet}")
    } else {
        String::new()
    };

    let system = format!("{obs_system}{coder_context}").trim_end().to_string();
    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    // Exclude the last entry in history (current user message, already pushed to pane)
    // to avoid sending a duplicate user message, matching the pattern in send_coder_message.
    let hist_len = history.len();
    for m in history.iter().take(hist_len.saturating_sub(1)) {
        messages.push(m.clone());
    }
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

async fn send_chat_message(
    app: &mut App,
    tx: &mpsc::Sender<StreamToken>,
    internal_tx: &mpsc::Sender<AppEvent>,
) {
    let text = app.chat.textarea.lines().join("\n").trim().to_string();
    if text.is_empty() || app.chat.streaming {
        return;
    }

    // Handle slash commands before sending to AI.
    if handle_slash_command(&text, app, PaneId::Chat) {
        app.chat.textarea = tui_textarea::TextArea::default();
        return;
    }

    // Abort any previous chat task before starting a new one.
    if let Some(handle) = app.chat_task.take() {
        handle.abort();
    }

    // Reset state for the new send.
    app.chat.textarea = tui_textarea::TextArea::default();
    app.chat.scroll = 0;
    app.ignore_chat_tokens = false;

    app.chat.push_user(text.clone());
    app.chat.streaming = true;
    app.chat.messages.push(Message::new_streaming(Role::Assistant));

    // Start background task planning (TaskRouter) in parallel.
    spawn_task_planner(app, internal_tx);

    let history = app.chat.chat_history();
    let cfg = app.chat_cfg.clone();
    let persona_prompt = resolve_persona(&cfg.persona).map(|p| p.prompt).unwrap_or("");
    let lang = language_instruction(Some(&app.lang), &cfg.mode);
    let mode = mode_prompt(&cfg.mode);

    let mut system = mode.to_string();
    if !lang.trim().is_empty() {
        system.push_str("\n\n[Language]\n");
        system.push_str(lang);
    }
    if !persona_prompt.trim().is_empty() {
        system.push_str("\n\n[Persona]\n");
        system.push_str(persona_prompt);
    }

    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: system,
    }];
    let hist_len = history.len();
    for m in history.iter().take(hist_len.saturating_sub(1)) {
        messages.push(m.clone());
    }
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: text,
    });

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
    app.chat_task = Some(handle);
}

fn spawn_task_planner(app: &mut App, internal_tx: &mpsc::Sender<AppEvent>) {
    if app.planning_tasks {
        return;
    }
    app.planning_tasks = true;

    let cfg = app.chat_cfg.clone();
    let lang = app.lang.clone();
    let chat_hist = app.chat.chat_history();
    let coder_hist = app.coder.chat_history();
    let observer_hist = app.observer.chat_history();
    let tx = internal_tx.clone();

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let provider = providers::build_provider(client, &cfg);

        let lang_instr = language_instruction(Some(&lang), &cfg.mode);
        let system = format!(
            "You are TaskRouter.\n\
Return STRICT JSON only (no markdown, no commentary).\n\
Schema: {{\"tasks\":[{{\"target\":\"coder|observer\",\"title\":\"...\",\"body\":\"...\",\"phase\":\"core|feature|polish|any\",\"priority\":0..100}}]}}\n\
Rules:\n\
- Prefer concrete next actions (files/commands) over abstract advice.\n\
- If there is no actionable work, return {{\"tasks\":[]}}.\n\n\
[Language]\n{lang_instr}"
        );

        let mut messages: Vec<ChatMessage> = Vec::new();
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system,
        });

        fn fmt_recent(label: &str, hist: &[ChatMessage], take: usize) -> String {
            let recent = hist
                .iter()
                .rev()
                .take(take)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>();
            let joined = recent
                .iter()
                .map(|m| format!("[{}] {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n");
            format!("{label}:\n{joined}")
        }

        let state = format!(
            "{}\n\n{}\n\n{}",
            fmt_recent("Recent chat", &chat_hist, 6),
            fmt_recent("Recent coder", &coder_hist, 6),
            fmt_recent("Recent observer", &observer_hist, 3),
        );
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: state,
        });

        let req = ChatRequest {
            messages,
            temperature: Some(0.2),
            max_tokens: Some(512),
            metadata: Some(json!({"response_format": {"type": "json_object"}})),
        };

        let resp = match provider.chat(&req).await {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(AppEvent::TaskPlanError(e.to_string())).await;
                return;
            }
        };

        let raw = resp.content;
        let json_str = extract_first_json_object(&raw).unwrap_or(raw);
        let parsed: PlannedTasks = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => {
                let _ = tx
                    .send(AppEvent::TaskPlanError(format!("invalid JSON: {e}")))
                    .await;
                return;
            }
        };

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let mut out: Vec<Task> = Vec::new();
        for (i, t) in parsed.tasks.into_iter().enumerate() {
            let target = match t.target.to_ascii_lowercase().as_str() {
                "observer" | "obs" => TaskTarget::Observer,
                _ => TaskTarget::Coder,
            };
            let phase = match t
                .phase
                .unwrap_or_else(|| "any".to_string())
                .to_ascii_lowercase()
                .as_str()
            {
                "core" => TaskPhase::Core,
                "feature" => TaskPhase::Feature,
                "polish" => TaskPhase::Polish,
                _ => TaskPhase::Any,
            };
            let prio = t.priority.unwrap_or(50).min(100);
            out.push(Task {
                id: format!("t{now_ms}-{i}"),
                target,
                title: t.title,
                body: t.body,
                phase,
                priority: prio,
                done: false,
            });
        }

        let _ = tx.send(AppEvent::TasksPlanned(out)).await;
    });
}

fn extract_first_json_object(s: &str) -> Option<String> {
    let start = s.find('{')?;
    let end = s.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(s[start..=end].to_string())
}

async fn dispatch_selected_task(
    app: &mut App,
    coder_tx: &mpsc::Sender<StreamToken>,
    observer_tx: &mpsc::Sender<StreamToken>,
) {
    let Some(t) = app.tasks.get(app.tasks_cursor).cloned() else {
        return;
    };
    let msg = format!(
        "[TASK]\nphase: {:?}\npriority: {}\ntitle: {}\n\n{}",
        t.phase, t.priority, t.title, t.body
    );
    match t.target {
        TaskTarget::Coder => send_coder_with_text(app, coder_tx, msg).await,
        TaskTarget::Observer => send_observer_message(app, observer_tx, Some(msg)).await,
    }
}

async fn maybe_auto_observe(app: &mut App, observer_tx: &mpsc::Sender<StreamToken>) {
    if let Some((idx, content)) = app.auto_observe_trigger() {
        app.last_auto_obs_idx = Some(idx);
        let snippet = content.chars().take(2000).collect::<String>();
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

    let lang = language_instruction(Some(&app.lang), &cfg.mode);
    let obs_mode_prompt = mode_prompt(&cfg.mode);
    let obs_system = format!("{obs_mode_prompt}\n\n[Language]\n{lang}");
    let obs_system = obs_system.trim_end().to_string();
    let coder_context = if !coder_history.is_empty() {
        let snippet = coder_history
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| {
                format!(
                    "[{}]: {}",
                    m.role,
                    m.content.chars().take(800).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\n[Recent Coder activity — last 10 turns]\n{snippet}")
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
