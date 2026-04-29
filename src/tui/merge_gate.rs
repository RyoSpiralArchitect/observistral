use anyhow::{anyhow, Context, Result};

use crate::merge_gate::{self, MergeGateActionResponse, MergeGateBoard, MergeGateBoardEntry};

use super::app::App;

const MERGE_GATE_ENTRY_LINES: usize = 2;

fn workspace_root() -> Result<std::path::PathBuf> {
    std::env::current_dir().context("failed to resolve workspace root")
}

fn clamp_cursor(app: &mut App) {
    if app.merge_gate_cursor >= app.merge_gate.entries.len() {
        app.merge_gate_cursor = app.merge_gate.entries.len().saturating_sub(1);
    }
}

fn apply_action_response(app: &mut App, response: MergeGateActionResponse) -> String {
    let message = response.message;
    app.merge_gate = response.board;
    clamp_cursor(app);
    app.merge_gate_status = Some(message.clone());
    message
}

pub fn selected_entry(app: &App) -> Option<&MergeGateBoardEntry> {
    app.merge_gate.entries.get(app.merge_gate_cursor)
}

pub fn visible_window(
    board: &MergeGateBoard,
    cursor: usize,
    viewport_rows: usize,
) -> (usize, usize) {
    if board.entries.is_empty() {
        return (0, 0);
    }
    let capacity = (viewport_rows / MERGE_GATE_ENTRY_LINES).max(1);
    let cur = cursor.min(board.entries.len().saturating_sub(1));
    let start = cur.saturating_sub(capacity / 2);
    let end = (start + capacity).min(board.entries.len());
    (start, end)
}

pub fn select_visible_row(app: &mut App, viewport_rows: usize, row_offset: usize) -> bool {
    if app.merge_gate.entries.is_empty() {
        return false;
    }
    let (start, end) = visible_window(&app.merge_gate, app.merge_gate_cursor, viewport_rows);
    let idx = start + (row_offset / MERGE_GATE_ENTRY_LINES);
    if idx >= end {
        return false;
    }
    app.merge_gate_cursor = idx;
    true
}

pub fn primary_action_label(entry: &MergeGateBoardEntry) -> Option<&'static str> {
    if entry.can_approve {
        Some("approve")
    } else {
        None
    }
}

pub fn primary_action_hint(entry: &MergeGateBoardEntry) -> &'static str {
    match primary_action_label(entry) {
        Some("approve") => "Enter=approve",
        _ if entry.can_preview_rollback => "Ctrl+Y=copy rollback preview",
        _ => "Enter=no-op",
    }
}

pub fn run_primary_action(app: &mut App) -> Result<String> {
    let entry = selected_entry(app).ok_or_else(|| anyhow!("no merge gate case selected"))?;
    if entry.can_approve {
        approve_selected(app)
    } else if entry.can_preview_rollback {
        Err(anyhow!(
            "rollback is preview-only; press Ctrl+Y to copy the rollback command"
        ))
    } else {
        Err(anyhow!("selected merge gate case has no primary action"))
    }
}

pub fn selected_entry_clipboard_text(app: &App) -> Option<String> {
    let entry = selected_entry(app)?;
    let mut lines = vec![
        format!("merge_gate_case: {}", entry.id),
        format!("case_status: {}", entry.case_status),
        format!("review_status: {}", entry.review_badge),
        format!("root: {}", entry.root),
        format!("gate_path: {}", entry.gate_path),
        format!("report_path: {}", entry.report_path),
    ];
    if let Some(path) = entry.promoted_overlay_path.as_deref() {
        lines.push(format!("promoted_overlay_path: {path}"));
    }
    if let Some(command) = entry.rollback_command.as_deref() {
        lines.push("rollback_preview:".to_string());
        lines.push(command.to_string());
    }
    Some(lines.join("\n"))
}

pub fn refresh_merge_gate(app: &mut App) -> Result<String> {
    let root = workspace_root()?;
    let board = merge_gate::load_board(&root)?;
    let message = board
        .status_message
        .clone()
        .unwrap_or_else(|| format!("loaded {} merge gate case(s)", board.summary.total));
    app.merge_gate = board;
    clamp_cursor(app);
    app.merge_gate_status = Some(message.clone());
    Ok(message)
}

pub fn approve_selected(app: &mut App) -> Result<String> {
    let root = workspace_root()?;
    let entry = selected_entry(app).ok_or_else(|| anyhow!("no merge gate case selected"))?;
    let response = merge_gate::approve(&root, &entry.id)?;
    Ok(apply_action_response(app, response))
}

pub fn hold_selected(app: &mut App) -> Result<String> {
    let root = workspace_root()?;
    let entry = selected_entry(app).ok_or_else(|| anyhow!("no merge gate case selected"))?;
    let response = merge_gate::hold(&root, &entry.id)?;
    Ok(apply_action_response(app, response))
}

#[cfg(test)]
mod tests {
    use super::super::app::App;
    use super::*;
    use crate::config::{ProviderKind, RunConfig};
    use crate::merge_gate::{MergeGateBoardStatus, MergeGateBoardSummary};
    use crate::modes::Mode;

    fn test_cfg(mode: Mode) -> RunConfig {
        RunConfig {
            provider: ProviderKind::OpenAiCompatible,
            model: "test-model".to_string(),
            chat_model: "test-model".to_string(),
            code_model: "test-model".to_string(),
            api_key: None,
            base_url: "https://api.openai.com/v1".to_string(),
            mode,
            persona: "default".to_string(),
            temperature: 0.2,
            max_tokens: 1024,
            timeout_seconds: 30,
            hf_device: "cpu".to_string(),
            hf_local_only: false,
        }
    }

    fn test_entry(
        id: &str,
        status: MergeGateBoardStatus,
        can_approve: bool,
        rollback_command: Option<&str>,
    ) -> MergeGateBoardEntry {
        MergeGateBoardEntry {
            id: id.to_string(),
            case_status: if can_approve { "passed" } else { "failed" }.to_string(),
            review_status: status,
            review_badge: match status {
                MergeGateBoardStatus::NeedsReview => "needs_review",
                MergeGateBoardStatus::Approved => "approved",
                MergeGateBoardStatus::Held => "held",
                MergeGateBoardStatus::RollbackAvailable => "rollback_available",
                MergeGateBoardStatus::Blocked => "blocked",
            }
            .to_string(),
            can_approve,
            can_hold: true,
            can_preview_rollback: rollback_command.is_some(),
            review_record: None,
            root: "/tmp/repo".to_string(),
            gate_path: "/tmp/repo/.tmp/runtime_eval/merge_gate.json".to_string(),
            report_path: "/tmp/repo/.tmp/runtime_eval/report.json".to_string(),
            rollback_command: rollback_command.map(str::to_string),
            promoted_overlay_path: None,
        }
    }

    fn test_app(entries: Vec<MergeGateBoardEntry>) -> App {
        let mut app = App::new(
            test_cfg(Mode::Jikkyo),
            test_cfg(Mode::Observer),
            test_cfg(Mode::Chat),
            None,
            None,
            false,
            "en".to_string(),
            None,
        );
        app.merge_gate.entries = entries;
        app.merge_gate.summary = MergeGateBoardSummary {
            total: app.merge_gate.entries.len(),
            passed: 0,
            failed: 0,
            needs_review: 0,
            approved: 0,
            held: 0,
            rollback_available: 0,
            blocked: 0,
        };
        app
    }

    #[test]
    fn visible_window_centers_cursor_by_entry_capacity() {
        let app = test_app(vec![
            test_entry("a", MergeGateBoardStatus::NeedsReview, true, None),
            test_entry("b", MergeGateBoardStatus::NeedsReview, true, None),
            test_entry(
                "c",
                MergeGateBoardStatus::RollbackAvailable,
                false,
                Some("git reset"),
            ),
            test_entry("d", MergeGateBoardStatus::Blocked, false, None),
        ]);
        assert_eq!(visible_window(&app.merge_gate, 2, 4), (1, 3));
    }

    #[test]
    fn select_visible_row_maps_rows_to_entries() {
        let mut app = test_app(vec![
            test_entry("a", MergeGateBoardStatus::NeedsReview, true, None),
            test_entry("b", MergeGateBoardStatus::Approved, false, None),
            test_entry(
                "c",
                MergeGateBoardStatus::RollbackAvailable,
                false,
                Some("git reset"),
            ),
        ]);
        app.merge_gate_cursor = 1;
        assert!(select_visible_row(&mut app, 6, 4));
        assert_eq!(app.merge_gate_cursor, 2);
    }

    #[test]
    fn primary_action_only_approves_green_cases() {
        let green = test_entry("a", MergeGateBoardStatus::NeedsReview, true, None);
        let rollback = test_entry(
            "b",
            MergeGateBoardStatus::RollbackAvailable,
            false,
            Some("git reset"),
        );
        assert_eq!(primary_action_label(&green), Some("approve"));
        assert_eq!(primary_action_hint(&green), "Enter=approve");
        assert_eq!(primary_action_label(&rollback), None);
        assert_eq!(
            primary_action_hint(&rollback),
            "Ctrl+Y=copy rollback preview"
        );
    }
}
