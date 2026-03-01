use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use super::app::{App, Focus, Role};

// ── Brand palette (mirrors web UI) ────────────────────────────────────────────

const CODER_BLUE: Color = Color::Rgb(96, 165, 250);   // blue-400
const OBS_MAG:    Color = Color::Rgb(217, 70, 239);   // fuchsia-500
const ACCENT:     Color = Color::Rgb(45, 212, 191);   // teal-400
const WARN:       Color = Color::Rgb(251, 191, 36);   // amber-400
const DANGER:     Color = Color::Rgb(248, 113, 113);  // red-400
const SUCCESS:    Color = Color::Rgb(74, 222, 128);   // green-400
const MUTED:      Color = Color::Rgb(100, 116, 139);  // slate-500
const TEXT_BODY:  Color = Color::Rgb(226, 232, 240);  // slate-200
const BG_DARK:    Color = Color::Rgb(15, 23, 42);     // slate-950
const UNFOCUSED:  Color = Color::Rgb(51, 65, 85);     // slate-700

// ── Animation ─────────────────────────────────────────────────────────────────

const SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

fn spinner_char(tick: u64) -> char {
    SPINNER[(tick as usize / 2) % SPINNER.len()]
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // header (2 rows)
            Constraint::Min(1),
            Constraint::Length(3), // input box
        ])
        .split(area);

    render_header(frame, vert[0], app);
    render_body(frame, vert[1], app);
    render_input(frame, vert[2], app);
}

// ── Header ────────────────────────────────────────────────────────────────────

fn iter_progress(n: u32) -> String {
    const MAX: u32 = 12;
    let n = n.min(MAX) as usize;
    let bar: String = (0..MAX as usize)
        .map(|i| if i < n { '█' } else { '░' })
        .collect();
    format!(" {bar} {n}/{MAX}")
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let c_spin = if app.coder.streaming {
        format!(" {}", spinner_char(app.tick_count))
    } else {
        String::new()
    };
    let o_spin = if app.observer.streaming {
        format!(" {}", spinner_char(app.tick_count))
    } else {
        String::new()
    };
    let iter = if app.coder_iter > 0 {
        iter_progress(app.coder_iter)
    } else {
        String::new()
    };

    let c_m = truncate_model(&app.coder_cfg.model, 20);
    let o_m = truncate_model(&app.observer_cfg.model, 20);

    let row1 = Line::from(vec![
        Span::styled(
            "  ◈ OBSTRAL ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ ", Style::default().fg(UNFOCUSED)),
        Span::styled("C: ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{c_m}{c_spin}"),
            Style::default()
                .fg(CODER_BLUE)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(iter, Style::default().fg(ACCENT)),
        Span::styled("  │  O: ", Style::default().fg(MUTED)),
        Span::styled(
            format!("{o_m}{o_spin}"),
            Style::default()
                .fg(OBS_MAG)
                .add_modifier(Modifier::BOLD),
        ),
        if app.auto_observe {
            Span::styled(
                "  ◉ AUTO",
                Style::default().fg(WARN).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
    ]);

    let row2 = Line::from(Span::styled(
        "  Tab=切替  Ctrl+A=自動  Ctrl+K=停止  Ctrl+O=実況  Ctrl+L=クリア  Ctrl+C=終了",
        Style::default().fg(MUTED),
    ));

    frame.render_widget(
        Paragraph::new(Text::from(vec![row1, row2]))
            .style(Style::default().bg(BG_DARK)),
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
    let brand = match which {
        Focus::Coder => CODER_BLUE,
        Focus::Observer => OBS_MAG,
    };

    let border_style = if focused {
        Style::default().fg(brand)
    } else {
        Style::default().fg(UNFOCUSED)
    };

    let spin = if pane.streaming {
        format!(" {}", spinner_char(app.tick_count))
    } else {
        String::new()
    };
    let scroll_badge = if pane.scroll > 0 {
        format!(" ↑{}", pane.scroll.min(9999))
    } else {
        String::new()
    };
    let focus_dot = if focused { "◉" } else { "○" };
    let label = match which {
        Focus::Coder => "CODER",
        Focus::Observer => "OBSERVER",
    };

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled(focus_dot, Style::default().fg(brand)),
        Span::raw(" "),
        Span::styled(
            label,
            Style::default().fg(brand).add_modifier(Modifier::BOLD),
        ),
        Span::styled(spin, Style::default().fg(ACCENT)),
        Span::styled(scroll_badge, Style::default().fg(WARN)),
        Span::raw(" "),
    ]);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if pane.messages.is_empty() {
        render_welcome(frame, inner, which);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    for msg in &pane.messages {
        match msg.role {
            Role::User => {
                lines.push(Line::from(vec![
                    Span::styled(
                        "  you ",
                        Style::default()
                            .fg(SUCCESS)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("›", Style::default().fg(MUTED)),
                ]));
                for l in msg.content.lines() {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(l.to_string(), Style::default().fg(SUCCESS)),
                    ]));
                }
            }
            Role::Assistant => {
                let (lbl, lbl_color) = match which {
                    Focus::Coder => ("coder", CODER_BLUE),
                    Focus::Observer => ("obs", OBS_MAG),
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {lbl} "),
                        Style::default()
                            .fg(lbl_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("›", Style::default().fg(MUTED)),
                ]));
                let content_lines = match which {
                    Focus::Coder => render_coder_content(&msg.content),
                    Focus::Observer => render_observer_content(&msg.content),
                };
                lines.extend(content_lines);
            }
            Role::Tool => {
                // Tool messages already formatted as "[TOOL] cmd" / "[RESULT] …"
                lines.extend(render_coder_content(&msg.content));
            }
        }

        // Streaming cursor.
        if !msg.complete {
            lines.push(Line::from(Span::styled(
                format!("  {} ", spinner_char(app.tick_count)),
                Style::default()
                    .fg(ACCENT)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        // Thin separator between messages.
        lines.push(Line::from(Span::styled(
            "  ·",
            Style::default().fg(UNFOCUSED),
        )));
        lines.push(Line::default());
    }

    // Scroll: scroll=0 pins to bottom; scroll=N shows N lines above bottom.
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

fn key_row(key: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {key:<18}"),
            Style::default().fg(ACCENT),
        ),
        Span::styled(desc, Style::default().fg(MUTED)),
    ])
}

fn render_welcome(frame: &mut Frame, area: Rect, which: Focus) {
    let (brand, heading, hint1, hint2) = match which {
        Focus::Coder => (
            CODER_BLUE,
            " ◈ CODER",
            "タスクを入力して Enter で送信",
            "例: \"maze game を作って\"",
        ),
        Focus::Observer => (
            OBS_MAG,
            " ◈ OBSERVER",
            "質問を入力して Enter で送信",
            "Ctrl+O でコーダーの最新出力をレビュー",
        ),
    };

    let mut lines = vec![
        Line::from(Span::styled(
            heading,
            Style::default().fg(brand).add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from(Span::styled(hint1, Style::default().fg(TEXT_BODY))),
        Line::from(Span::styled(hint2, Style::default().fg(MUTED))),
        Line::default(),
    ];

    for (key, desc) in [
        ("Tab", "フォーカス切り替え"),
        ("Enter", "送信"),
        ("Shift+Enter", "改行"),
        ("Ctrl+K", "ストリーミング停止"),
        ("Ctrl+L", "履歴クリア"),
        ("Ctrl+A", "自動実況 ON/OFF"),
        ("Ctrl+O", "Observer 手動トリガー"),
        ("PageUp/Down", "スクロール"),
        ("End", "最下部へ"),
        ("Ctrl+C / Esc", "終了"),
    ] {
        lines.push(key_row(key, desc));
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        },
    );
}

// ── Observer content renderer ─────────────────────────────────────────────────
//
// Sections:
//   --- phase ---         → ACCENT banner
//   --- proposals ---     → OBS_MAG banner + card-like rendering
//   --- critical_path --- → DANGER banner  (⚠ highlights)
//   --- health ---        → health score bar

#[derive(Clone, Copy, PartialEq)]
enum ObsSection {
    Body,
    Phase,
    Proposals,
    CriticalPath,
    Health,
}

fn obs_section_header(label: &'static str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("── {label} "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("─".repeat(18), Style::default().fg(UNFOCUSED)),
    ])
}

fn render_observer_content(content: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut section = ObsSection::Body;

    for raw in content.lines() {
        let trimmed = raw.trim();

        // Section boundaries.
        match trimmed {
            "--- phase ---" => {
                section = ObsSection::Phase;
                lines.push(obs_section_header("PHASE", ACCENT));
                continue;
            }
            "--- proposals ---" => {
                section = ObsSection::Proposals;
                lines.push(obs_section_header("PROPOSALS", OBS_MAG));
                continue;
            }
            "--- critical_path ---" => {
                section = ObsSection::CriticalPath;
                lines.push(obs_section_header("CRITICAL PATH", DANGER));
                continue;
            }
            "--- health ---" => {
                section = ObsSection::Health;
                lines.push(obs_section_header("HEALTH", SUCCESS));
                continue;
            }
            _ => {}
        }
        if trimmed.starts_with("--- ") && trimmed.ends_with(" ---") {
            section = ObsSection::Body;
        }

        let line = match section {
            ObsSection::Body => {
                Line::from(Span::styled(raw.to_string(), Style::default().fg(TEXT_BODY)))
            }
            ObsSection::Phase => Line::from(vec![
                Span::styled("  ◈ ", Style::default().fg(ACCENT)),
                Span::styled(
                    trimmed.to_string(),
                    Style::default()
                        .fg(ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            ObsSection::Proposals => proposal_line(trimmed),
            ObsSection::CriticalPath => {
                if trimmed.is_empty() {
                    Line::default()
                } else {
                    Line::from(vec![
                        Span::styled(
                            "  ⚠ ",
                            Style::default()
                                .fg(DANGER)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(trimmed.to_string(), Style::default().fg(DANGER)),
                    ])
                }
            }
            ObsSection::Health => health_line(trimmed),
        };
        lines.push(line);
    }
    lines
}

fn parse_score_from(s: &str) -> Option<u32> {
    let low = s.to_ascii_lowercase();
    let idx = low.find("score:")?;
    low[idx + 6..].trim().split_whitespace().next()?.parse().ok()
}

fn score_color(score: u32) -> Color {
    if score >= 70 {
        DANGER
    } else if score >= 40 {
        WARN
    } else {
        SUCCESS
    }
}

fn proposal_line(trimmed: &str) -> Line<'static> {
    let lower = trimmed.to_ascii_lowercase();

    // Numbered header: "1) title: ..."
    let first_char_digit = trimmed.chars().next().map_or(false, |c| c.is_ascii_digit());
    if first_char_digit && trimmed.contains(") ") {
        return Line::from(vec![
            Span::raw("  "),
            Span::styled(
                trimmed.to_string(),
                Style::default()
                    .fg(TEXT_BODY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
    }

    // score: N  →  score bar
    if lower.starts_with("score:") {
        if let Some(s) = parse_score_from(&lower) {
            let c = score_color(s);
            let filled = (s / 10) as usize;
            let bar: String = (0..10)
                .map(|i| if i < filled { '█' } else { '░' })
                .collect();
            return Line::from(vec![
                Span::styled(
                    format!("    score: {s:>3} "),
                    Style::default().fg(c).add_modifier(Modifier::BOLD),
                ),
                Span::styled(bar, Style::default().fg(c)),
            ]);
        }
    }

    // severity: crit|warn|info
    if lower.starts_with("severity:") {
        let style = if lower.contains("crit") {
            Style::default().fg(DANGER).add_modifier(Modifier::BOLD)
        } else if lower.contains("warn") {
            Style::default().fg(WARN)
        } else {
            Style::default().fg(MUTED)
        };
        return Line::from(Span::styled(format!("    {trimmed}"), style));
    }

    // to_coder: msg
    if lower.starts_with("to_coder:") {
        let val = trimmed["to_coder:".len()..].trim().to_string();
        return Line::from(vec![
            Span::styled(
                "    ↳ ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(val, Style::default().fg(ACCENT)),
        ]);
    }

    // quote: snippet
    if lower.starts_with("quote:") {
        let q = trimmed["quote:".len()..].trim().to_string();
        return Line::from(vec![
            Span::styled("    ❝ ", Style::default().fg(MUTED)),
            Span::styled(q, Style::default().fg(WARN).add_modifier(Modifier::ITALIC)),
        ]);
    }

    // status: addressed|[UNRESOLVED]|[ESCALATED]
    if lower.starts_with("status:") {
        let val = lower["status:".len()..].trim().to_string();
        let style = if val.contains("unresolved") || val.contains("escalated") {
            Style::default().fg(DANGER)
        } else if val.contains("addressed") {
            Style::default().fg(SUCCESS)
        } else {
            Style::default().fg(MUTED)
        };
        return Line::from(Span::styled(format!("    {trimmed}"), style));
    }

    // impact / cost / phase labels (secondary info)
    if lower.starts_with("impact:") || lower.starts_with("cost:") || lower.starts_with("phase:") {
        return Line::from(Span::styled(
            format!("    {trimmed}"),
            Style::default().fg(MUTED),
        ));
    }

    // Default proposal line
    Line::from(Span::styled(
        format!("  {trimmed}"),
        Style::default().fg(TEXT_BODY),
    ))
}

fn health_line(trimmed: &str) -> Line<'static> {
    if trimmed.is_empty() {
        return Line::default();
    }
    if let Some(score) = parse_score_from(&trimmed.to_ascii_lowercase()) {
        let c = if score >= 70 {
            SUCCESS
        } else if score >= 40 {
            WARN
        } else {
            DANGER
        };
        let filled = (score / 5) as usize; // 20-block bar
        let bar: String = (0..20)
            .map(|i| if i < filled { '█' } else { '░' })
            .collect();
        return Line::from(vec![
            Span::styled("  ❤ ", Style::default().fg(c).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{score:>3}/100 "),
                Style::default().fg(c).add_modifier(Modifier::BOLD),
            ),
            Span::styled(bar, Style::default().fg(c)),
        ]);
    }
    Line::from(Span::styled(
        format!("    {trimmed}"),
        Style::default().fg(MUTED),
    ))
}

// ── Coder content renderer ────────────────────────────────────────────────────
//
// Visual zones:
//   <plan>…</plan>      Blue box-drawing frame (CODER_BLUE/UNFOCUSED)
//   <think>…</think>    Gray box-drawing frame; per-field colors
//   ```lang…```         Bordered code block
//   [TOOL] cmd          ▶ EXEC label (WARN)
//   [RESULT] exit=0     ✓ OK (SUCCESS)
//   [RESULT] exit=N ⚠   ✗ exit=N (DANGER)
//   [agent] …           dim annotation (UNFOCUSED)
//   diff …              unified-diff coloring
//   else                TEXT_BODY

fn render_coder_content(content: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_think = false;
    let mut in_plan = false;
    let mut in_diff = false;
    let mut in_code = false;
    let mut code_lang = String::new();

    for raw in content.lines() {
        let trimmed = raw.trim();

        // ── <plan> block ──────────────────────────────────────────────────────
        if trimmed.starts_with("<plan>") {
            in_plan = true;
            in_think = false;
            lines.push(Line::from(vec![
                Span::styled("  ╭── ", Style::default().fg(CODER_BLUE)),
                Span::styled(
                    "PLAN",
                    Style::default()
                        .fg(CODER_BLUE)
                        .add_modifier(Modifier::BOLD | Modifier::ITALIC),
                ),
                Span::styled(" ─────────────────", Style::default().fg(UNFOCUSED)),
            ]));
            if trimmed.contains("</plan>") {
                in_plan = false;
                lines.push(Line::from(Span::styled(
                    "  ╰─────────────────────────",
                    Style::default().fg(UNFOCUSED),
                )));
            }
            continue;
        }
        if in_plan {
            if trimmed.contains("</plan>") {
                in_plan = false;
                lines.push(Line::from(Span::styled(
                    "  ╰─────────────────────────",
                    Style::default().fg(UNFOCUSED),
                )));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("  │ ", Style::default().fg(UNFOCUSED)),
                    Span::styled(
                        trimmed.to_string(),
                        Style::default()
                            .fg(MUTED)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
            continue;
        }

        // ── <think> block ─────────────────────────────────────────────────────
        if trimmed.starts_with("<think>") {
            in_think = true;
            in_diff = false;
            lines.push(Line::from(vec![
                Span::styled("  ╭── ", Style::default().fg(UNFOCUSED)),
                Span::styled(
                    "think",
                    Style::default()
                        .fg(MUTED)
                        .add_modifier(Modifier::ITALIC),
                ),
                Span::styled(" ──────────────────", Style::default().fg(UNFOCUSED)),
            ]));
            if trimmed.contains("</think>") {
                in_think = false;
                lines.push(Line::from(Span::styled(
                    "  ╰─────────────────────────",
                    Style::default().fg(UNFOCUSED),
                )));
            }
            continue;
        }
        if in_think {
            if trimmed.contains("</think>") {
                in_think = false;
                lines.push(Line::from(Span::styled(
                    "  ╰─────────────────────────",
                    Style::default().fg(UNFOCUSED),
                )));
            } else {
                let (label, rest) = split_field(trimmed);
                let (label_style, rest_style) = match label {
                    "goal" => (
                        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(ACCENT)
                            .add_modifier(Modifier::ITALIC),
                    ),
                    "risk" => (
                        Style::default().fg(WARN).add_modifier(Modifier::BOLD),
                        Style::default().fg(WARN).add_modifier(Modifier::ITALIC),
                    ),
                    "next" => (
                        Style::default()
                            .fg(CODER_BLUE)
                            .add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(CODER_BLUE)
                            .add_modifier(Modifier::ITALIC),
                    ),
                    "verify" => (
                        Style::default()
                            .fg(SUCCESS)
                            .add_modifier(Modifier::BOLD),
                        Style::default()
                            .fg(SUCCESS)
                            .add_modifier(Modifier::ITALIC),
                    ),
                    _ => (
                        Style::default()
                            .fg(MUTED)
                            .add_modifier(Modifier::ITALIC),
                        Style::default()
                            .fg(MUTED)
                            .add_modifier(Modifier::ITALIC),
                    ),
                };
                if label.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("  │ ", Style::default().fg(UNFOCUSED)),
                        Span::styled(trimmed.to_string(), rest_style),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled("  │ ", Style::default().fg(UNFOCUSED)),
                        Span::styled(format!("{label}: "), label_style),
                        Span::styled(rest.to_string(), rest_style),
                    ]));
                }
            }
            continue;
        }

        // ── Code fences ───────────────────────────────────────────────────────
        if trimmed.starts_with("```") {
            if in_code {
                in_code = false;
                lines.push(Line::from(Span::styled(
                    "  ╰──────────────────────────",
                    Style::default().fg(UNFOCUSED),
                )));
                code_lang.clear();
            } else {
                in_code = true;
                code_lang = trimmed.trim_start_matches('`').to_string();
                let lang_label = if code_lang.is_empty() {
                    String::new()
                } else {
                    format!(" {}", code_lang)
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  ╭──{lang_label} "),
                        Style::default().fg(UNFOCUSED),
                    ),
                    Span::styled("──────────────────", Style::default().fg(UNFOCUSED)),
                ]));
            }
            continue;
        }
        if in_code {
            let style = code_line_style(&code_lang, trimmed);
            lines.push(Line::from(vec![
                Span::styled("  │ ", Style::default().fg(UNFOCUSED)),
                Span::styled(raw.to_string(), style),
            ]));
            continue;
        }

        // ── Special markers ───────────────────────────────────────────────────
        if trimmed.starts_with("[TOOL]") {
            in_diff = false;
            let cmd = trimmed.trim_start_matches("[TOOL]").trim();
            lines.push(Line::from(vec![
                Span::styled(
                    "  ▶ EXEC ",
                    Style::default().fg(WARN).add_modifier(Modifier::BOLD),
                ),
                Span::styled(cmd.to_string(), Style::default().fg(TEXT_BODY)),
            ]));
            continue;
        }
        if trimmed.starts_with("[RESULT] exit=0") {
            in_diff = false;
            lines.push(Line::from(Span::styled(
                "  ✓ OK",
                Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
            )));
            continue;
        }
        if trimmed.starts_with("[RESULT]") {
            in_diff = false;
            let rest = trimmed.trim_start_matches("[RESULT]").trim();
            lines.push(Line::from(vec![
                Span::styled(
                    "  ✗ ",
                    Style::default().fg(DANGER).add_modifier(Modifier::BOLD),
                ),
                Span::styled(rest.to_string(), Style::default().fg(DANGER)),
            ]));
            continue;
        }
        if trimmed.starts_with("[agent]") {
            in_diff = false;
            lines.push(Line::from(Span::styled(
                format!("  {trimmed}"),
                Style::default().fg(UNFOCUSED),
            )));
            continue;
        }

        // ── Diff ──────────────────────────────────────────────────────────────
        if trimmed.starts_with("diff --git ")
            || trimmed.starts_with("diff -u ")
            || trimmed.starts_with("diff -r ")
        {
            in_diff = true;
        }
        if in_diff {
            let style = if trimmed.starts_with("diff ") {
                Style::default()
                    .fg(CODER_BLUE)
                    .add_modifier(Modifier::BOLD)
            } else if trimmed.starts_with("+++ ")
                || trimmed.starts_with("--- ")
                || trimmed.starts_with("index ")
                || trimmed.starts_with("new file")
                || trimmed.starts_with("deleted file")
            {
                Style::default()
                    .fg(TEXT_BODY)
                    .add_modifier(Modifier::BOLD)
            } else if trimmed.starts_with("@@ ") {
                Style::default().fg(ACCENT)
            } else if trimmed.starts_with('+') && !trimmed.starts_with("+++") {
                Style::default().fg(SUCCESS)
            } else if trimmed.starts_with('-') && !trimmed.starts_with("---") {
                Style::default().fg(DANGER)
            } else {
                Style::default().fg(TEXT_BODY)
            };
            lines.push(Line::from(Span::styled(raw.to_string(), style)));
            continue;
        }

        // Default
        lines.push(Line::from(Span::styled(
            raw.to_string(),
            Style::default().fg(TEXT_BODY),
        )));
    }
    lines
}

/// Split "label: rest" into ("label", "rest").  Returns ("", line) if no colon found.
fn split_field(line: &str) -> (&str, &str) {
    if let Some(idx) = line.find(':') {
        (line[..idx].trim(), line[idx + 1..].trim())
    } else {
        ("", line)
    }
}

/// Minimal per-language code line styling.
fn code_line_style(lang: &str, line: &str) -> Style {
    let l = lang.to_ascii_lowercase();
    if l == "diff" {
        if line.starts_with('+') {
            return Style::default().fg(SUCCESS);
        }
        if line.starts_with('-') {
            return Style::default().fg(DANGER);
        }
        if line.starts_with('@') {
            return Style::default().fg(ACCENT);
        }
    }
    if l == "powershell" || l == "ps1" || l == "ps" || l == "pwsh" {
        if line.trim_start().starts_with('#') {
            return Style::default()
                .fg(MUTED)
                .add_modifier(Modifier::ITALIC);
        }
        if line.trim_start().starts_with("$") || line.contains(" $") {
            return Style::default().fg(CODER_BLUE);
        }
    }
    if l == "bash" || l == "sh" || l == "shell" || l == "console" {
        if line.trim_start().starts_with('#') {
            return Style::default()
                .fg(MUTED)
                .add_modifier(Modifier::ITALIC);
        }
        if line.starts_with("$ ") || line.starts_with("PS> ") {
            return Style::default()
                .fg(CODER_BLUE)
                .add_modifier(Modifier::BOLD);
        }
    }
    Style::default().fg(TEXT_BODY)
}

// ── Input bar ─────────────────────────────────────────────────────────────────

fn render_input(frame: &mut Frame, area: Rect, app: &App) {
    let is_coder = app.focus == Focus::Coder;
    let is_streaming = if is_coder {
        app.coder.streaming
    } else {
        app.observer.streaming
    };
    let brand = if is_coder { CODER_BLUE } else { OBS_MAG };
    let label = if is_coder { "CODER" } else { "OBSERVER" };
    let hint = if is_streaming {
        "Ctrl+K=停止"
    } else {
        "Enter=送信  Shift+Enter=改行  End=最下部"
    };

    let title = Line::from(vec![
        Span::raw(" "),
        Span::styled("›", Style::default().fg(brand).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(label, Style::default().fg(brand).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(hint, Style::default().fg(MUTED)),
        Span::raw(" "),
    ]);

    let border_color = if is_streaming { WARN } else { brand };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.focus {
        Focus::Coder => frame.render_widget(&app.coder.textarea, inner),
        Focus::Observer => frame.render_widget(&app.observer.textarea, inner),
    };
}
