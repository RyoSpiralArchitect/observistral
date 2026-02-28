use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::app::{App, Focus, Role};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // ── Vertical slices: header / body / footer ─────────────────────────────
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header bar
            Constraint::Min(1),    // main content
            Constraint::Length(3), // input box (tui-textarea renders 3 lines tall)
        ])
        .split(area);

    render_header(frame, vert[0], app);
    render_body(frame, vert[1], app);
    render_input(frame, vert[2], app);
}

// ── Header ────────────────────────────────────────────────────────────────────
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let coder_model = &app.coder_cfg.model;
    let obs_model = &app.observer_cfg.model;
    let auto = if app.auto_observe { "AUTO:ON" } else { "AUTO:OFF" };
    let text = format!(" OBSTRAL  [C:{coder_model}] [O:{obs_model}] [{auto}]  Tab=切替  Ctrl+A=自動  Ctrl+O=Observer  Ctrl+C=終了");
    let para = Paragraph::new(text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(para, area);
}

// ── Body: left/right pane split ───────────────────────────────────────────────
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
    let label = match which {
        Focus::Coder => if focused { "◉ CODER" } else { "○ CODER" },
        Focus::Observer => if focused { "◉ OBSERVER" } else { "○ OBSERVER" },
    };
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(label)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build lines from messages.
    let mut lines: Vec<Line> = Vec::new();
    for msg in &pane.messages {
        let (prefix, prefix_style, body_style) = match msg.role {
            Role::User => (
                "you> ",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Green),
            ),
            Role::Assistant => (
                if which == Focus::Coder { "coder: " } else { "obs: " },
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::White),
            ),
            Role::Tool => (
                "$ ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Yellow),
            ),
        };

        lines.push(Line::from(vec![Span::styled(prefix, prefix_style)]));
        // Render proposals for Observer, think-block + tool annotations for Coder.
        if which == Focus::Observer && msg.role == Role::Assistant {
            lines.extend(render_observer_content(&msg.content, body_style));
        } else if which == Focus::Coder && msg.role == Role::Assistant {
            lines.extend(render_coder_content(&msg.content));
        } else {
            for content_line in msg.content.lines() {
                lines.push(Line::from(vec![Span::styled(content_line.to_string(), body_style)]));
            }
        }

        if !msg.complete {
            lines.push(Line::from(vec![Span::styled("▌", Style::default().fg(Color::Cyan))]));
        }
        lines.push(Line::default()); // blank separator
    }

    let total_lines = lines.len() as u16;
    let visible = inner.height;
    let scroll = if total_lines > visible {
        // Auto-scroll to bottom, unless user has manually scrolled.
        let auto = (total_lines - visible) as usize;
        pane.scroll.min(auto)
    } else {
        0
    };
    // Store the max_scroll value via clamped index.
    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(para, inner);
}

/// Render Observer assistant content: colour-code `--- proposals ---` blocks.
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
        if trimmed.starts_with("--- ") && trimmed.ends_with(" ---") {
            in_proposals = false;
        }
        if in_proposals {
            lines.push(Line::from(vec![Span::styled(
                raw_line.to_string(),
                proposal_line_style(trimmed),
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(raw_line.to_string(), body_style)]));
        }
    }
    lines
}

/// Render Coder assistant content with three visual zones:
///
///   <think>…</think>  — dim gray italic  (scratchpad, low visual weight)
///   [TOOL] …          — yellow bold       (command being executed)
///   [RESULT] exit=0   — green / red       (outcome)
///   everything else   — white             (normal response text)
fn render_coder_content(content: &str) -> Vec<Line<'static>> {
    let think_style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::ITALIC | Modifier::DIM);
    let tool_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let ok_style = Style::default().fg(Color::Green);
    let err_style = Style::default().fg(Color::Red);
    let body_style = Style::default().fg(Color::White);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_think = false;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();

        // Think block open
        if trimmed.starts_with("<think>") {
            in_think = true;
            lines.push(Line::from(vec![Span::styled(raw_line.to_string(), think_style)]));
            if trimmed.contains("</think>") {
                in_think = false;
            }
            continue;
        }
        // Think block close
        if in_think {
            lines.push(Line::from(vec![Span::styled(raw_line.to_string(), think_style)]));
            if trimmed.contains("</think>") {
                in_think = false;
            }
            continue;
        }

        // Tool annotation lines emitted by the agent
        let style = if trimmed.starts_with("[TOOL]") {
            tool_style
        } else if trimmed.starts_with("[RESULT] exit=0") {
            ok_style
        } else if trimmed.starts_with("[RESULT]") {
            err_style
        } else if trimmed.starts_with("[agent]") {
            Style::default().fg(Color::DarkGray)
        } else {
            body_style
        };

        lines.push(Line::from(vec![Span::styled(raw_line.to_string(), style)]));
    }
    lines
}

fn proposal_line_style(line: &str) -> Style {
    let lower = line.to_ascii_lowercase();
    if lower.contains("severity: crit") || lower.contains("score: 9") || lower.contains("score: 10") {
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

// ── Input ─────────────────────────────────────────────────────────────────────
fn render_input(frame: &mut Frame, area: Rect, app: &App) {
    let focused_label = match app.focus {
        Focus::Coder => "CODER",
        Focus::Observer => "OBSERVER",
    };
    let is_streaming = match app.focus {
        Focus::Coder => app.coder.streaming,
        Focus::Observer => app.observer.streaming,
    };
    let hint = if is_streaming { "  [Ctrl+K=停止]" } else { "  [Enter=送信  Shift+Enter=改行]" };
    let title = format!(" > {focused_label}{hint}");

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Render the focused pane's textarea inside the block.
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.focus {
        Focus::Coder => frame.render_widget(&app.coder.textarea, inner),
        Focus::Observer => frame.render_widget(&app.observer.textarea, inner),
    };
}
