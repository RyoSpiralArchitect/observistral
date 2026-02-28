use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::app::{App, Focus, Role};

// ── Animation ─────────────────────────────────────────────────────────────────

const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

fn spinner_char(tick: u64) -> char {
    SPINNER[(tick as usize / 2) % SPINNER.len()] // /2 → each frame lasts 200ms
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header bar
            Constraint::Min(1),    // pane area
            Constraint::Length(3), // input box
        ])
        .split(area);

    render_header(frame, vert[0], app);
    render_body(frame, vert[1], app);
    render_input(frame, vert[2], app);
}

// ── Header ────────────────────────────────────────────────────────────────────

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let coder_spin = if app.coder.streaming {
        format!(" {}", spinner_char(app.tick_count))
    } else {
        String::new()
    };
    let obs_spin = if app.observer.streaming {
        format!(" {}", spinner_char(app.tick_count))
    } else {
        String::new()
    };
    let iter_badge = if app.coder_iter > 0 {
        format!(" [iter {}/12]", app.coder_iter)
    } else {
        String::new()
    };
    let auto_badge = if app.auto_observe { " AUTO:ON" } else { "" };
    let coder_model = truncate_model(&app.coder_cfg.model, 18);
    let obs_model = truncate_model(&app.observer_cfg.model, 18);

    let text = format!(
        " OBSTRAL  C:{coder_model}{coder_spin}{iter_badge}  O:{obs_model}{obs_spin}{auto_badge}  \
         Tab=切替  Ctrl+A=自動  Ctrl+K=停止  Ctrl+C=終了"
    );
    frame.render_widget(
        Paragraph::new(text).style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        area,
    );
}

fn truncate_model(name: &str, max: usize) -> String {
    if name.chars().count() <= max {
        name.to_string()
    } else {
        format!("{}…", name.chars().take(max - 1).collect::<String>())
    }
}

// ── Body: two-pane split ──────────────────────────────────────────────────────

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_pane(frame, horiz[0], app, Focus::Coder);
    render_pane(frame, horiz[1], app, Focus::Observer);
}

fn render_pane(frame: &mut Frame, area: Rect, app: &App, which: Focus) {
    let pane = match which {
        Focus::Coder => &app.coder,
        Focus::Observer => &app.observer,
    };
    let focused = app.focus == which;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let focus_dot = if focused { "◉" } else { "○" };
    let pane_name = match which {
        Focus::Coder => "CODER",
        Focus::Observer => "OBSERVER",
    };

    // Scroll position badge: show "↑N" when user has scrolled above the bottom.
    let scroll_badge = if pane.scroll > 0 {
        let lines = pane.scroll.min(9999);
        format!(" [↑{lines}]")
    } else {
        String::new()
    };

    // Streaming indicator in pane title.
    let stream_indicator = if pane.streaming {
        format!(" {}", spinner_char(app.tick_count))
    } else {
        String::new()
    };

    let title = format!(" {focus_dot} {pane_name}{stream_indicator}{scroll_badge} ");
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if pane.messages.is_empty() {
        render_welcome(frame, inner, which);
        return;
    }

    // Build display lines.
    let mut lines: Vec<Line> = Vec::new();
    for msg in &pane.messages {
        let (prefix, prefix_style, body_style) = match msg.role {
            Role::User => (
                "you › ",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Green),
            ),
            Role::Assistant => (
                match which {
                    Focus::Coder => "coder › ",
                    Focus::Observer => "obs › ",
                },
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::White),
            ),
            Role::Tool => (
                "  ",
                Style::default().fg(Color::Yellow),
                Style::default().fg(Color::Yellow),
            ),
        };

        lines.push(Line::from(vec![Span::styled(prefix, prefix_style)]));

        match (&msg.role, which) {
            (Role::Assistant, Focus::Observer) => {
                lines.extend(render_observer_content(&msg.content, body_style));
            }
            (Role::Assistant, Focus::Coder) => {
                lines.extend(render_coder_content(&msg.content));
            }
            _ => {
                for l in msg.content.lines() {
                    lines.push(Line::from(vec![Span::styled(l.to_string(), body_style)]));
                }
            }
        }

        // Streaming cursor.
        if !msg.complete {
            lines.push(Line::from(vec![Span::styled(
                format!("{} ", spinner_char(app.tick_count)),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )]));
        }
        lines.push(Line::default()); // blank separator
    }

    // ── Scroll (lines-from-bottom semantics) ──────────────────────────────────
    // scroll=0 → show bottom (auto-scroll). scroll=N → N lines above bottom.
    let total = lines.len();
    let visible = inner.height as usize;
    let max_scroll = total.saturating_sub(visible);
    let from_bottom = pane.scroll.min(max_scroll);
    let from_top = max_scroll.saturating_sub(from_bottom);

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((from_top as u16, 0)),
        inner,
    );
}

// ── Welcome / empty-pane hint ─────────────────────────────────────────────────

fn render_welcome(frame: &mut Frame, area: Rect, which: Focus) {
    let lines: Vec<Line> = match which {
        Focus::Coder => vec![
            Line::from(vec![Span::styled(
                "── CODER ──",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
            Line::from(vec![Span::styled(
                "タスクを入力して Enter で送信",
                Style::default().fg(Color::White),
            )]),
            Line::from(vec![Span::styled(
                "例: \"maze game を作って\"",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::default(),
            Line::from(vec![Span::styled(
                "  Tab          フォーカス切り替え",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "  Shift+Enter  改行",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "  Ctrl+K       ストリーミング停止",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "  Ctrl+L       履歴クリア",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "  PageUp/Down  スクロール  (End=最下部)",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "  Ctrl+A       自動実況 ON/OFF",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "  Ctrl+C / Esc  終了",
                Style::default().fg(Color::DarkGray),
            )]),
        ],
        Focus::Observer => vec![
            Line::from(vec![Span::styled(
                "── OBSERVER ──",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            )]),
            Line::default(),
            Line::from(vec![Span::styled(
                "質問を入力して Enter で送信",
                Style::default().fg(Color::White),
            )]),
            Line::from(vec![Span::styled(
                "Ctrl+O でコーダーの最新出力を自動レビュー",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::default(),
            Line::from(vec![Span::styled(
                "  Ctrl+A  自動実況を ON にすると",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                "  コーダーが出力するたびに自動でここに批評が入る",
                Style::default().fg(Color::DarkGray),
            )]),
        ],
    };

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
        Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        },
    );
}

// ── Observer content renderer ─────────────────────────────────────────────────

fn render_observer_content(content: &str, body_style: Style) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_proposals = false;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed == "--- proposals ---" {
            in_proposals = true;
            lines.push(Line::from(vec![Span::styled(
                "── proposals ──".to_string(),
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            )]));
            continue;
        }
        // Next top-level "--- foo ---" section closes proposals.
        if in_proposals && trimmed.starts_with("--- ") && trimmed.ends_with(" ---") {
            in_proposals = false;
        }

        let style = if in_proposals {
            proposal_line_style(trimmed)
        } else {
            body_style
        };
        lines.push(Line::from(vec![Span::styled(raw_line.to_string(), style)]));
    }
    lines
}

fn proposal_line_style(line: &str) -> Style {
    let lower = line.to_ascii_lowercase();
    // Score 80+ or crit severity → red bold
    if lower.contains("severity: crit")
        || lower.contains("score: 10")
        || lower.contains("score: 9")
        || lower.contains("score: 8")
    {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if lower.contains("severity: warn") {
        Style::default().fg(Color::Yellow)
    } else if lower.contains("severity: info") {
        Style::default().fg(Color::Blue)
    } else if lower.contains("to_coder:") {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    }
}

// ── Coder content renderer ────────────────────────────────────────────────────
//
// Visual zones:
//   <think>…</think>   DarkGray + Italic + Dim  (scratchpad, low visual weight)
//   [TOOL] …           Yellow + Bold            (command dispatched to exec)
//   [RESULT] exit=0    Green                    (success)
//   [RESULT] exit=N ⚠  Red                     (failure)
//   [agent] …          DarkGray                 (system annotations)
//   everything else    White

fn render_coder_content(content: &str) -> Vec<Line<'static>> {
    let think_style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::ITALIC | Modifier::DIM);
    let tool_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let ok_style = Style::default().fg(Color::Green);
    let err_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    let annotation_style = Style::default().fg(Color::DarkGray);
    let body_style = Style::default().fg(Color::White);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_think = false;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();

        // ── Think block state machine ─────────────────────────────────────
        if trimmed.starts_with("<think>") {
            in_think = true;
            lines.push(Line::from(vec![Span::styled(raw_line.to_string(), think_style)]));
            if trimmed.contains("</think>") { in_think = false; }
            continue;
        }
        if in_think {
            lines.push(Line::from(vec![Span::styled(raw_line.to_string(), think_style)]));
            if trimmed.contains("</think>") { in_think = false; }
            continue;
        }

        // ── Tool / result annotations ─────────────────────────────────────
        let style = if trimmed.starts_with("[TOOL]") {
            tool_style
        } else if trimmed.starts_with("[RESULT] exit=0") {
            ok_style
        } else if trimmed.starts_with("[RESULT]") {
            err_style
        } else if trimmed.starts_with("[agent]") {
            annotation_style
        } else {
            body_style
        };

        lines.push(Line::from(vec![Span::styled(raw_line.to_string(), style)]));
    }
    lines
}

// ── Input bar ─────────────────────────────────────────────────────────────────

fn render_input(frame: &mut Frame, area: Rect, app: &App) {
    let focused_label = match app.focus {
        Focus::Coder => "CODER",
        Focus::Observer => "OBSERVER",
    };
    let is_streaming = match app.focus {
        Focus::Coder => app.coder.streaming,
        Focus::Observer => app.observer.streaming,
    };
    let hint = if is_streaming {
        "  Ctrl+K=停止"
    } else {
        "  Enter=送信  Shift+Enter=改行  End=最下部"
    };
    let title = format!(" › {focused_label}{hint} ");

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_streaming { Color::Yellow } else { Color::Cyan }));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.focus {
        Focus::Coder => frame.render_widget(&app.coder.textarea, inner),
        Focus::Observer => frame.render_widget(&app.observer.textarea, inner),
    };
}
