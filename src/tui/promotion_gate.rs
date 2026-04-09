use anyhow::{anyhow, Context, Result};

use crate::harness_gate::{
    self, HarnessPromotionActionResponse, HarnessPromotionBoard, HarnessPromotionBoardEntry,
};

use super::app::App;

const PROMOTION_ENTRY_LINES: usize = 2;

fn workspace_root() -> Result<std::path::PathBuf> {
    std::env::current_dir().context("failed to resolve workspace root")
}

fn clamp_cursor(app: &mut App) {
    if app.harness_promotions_cursor >= app.harness_promotions.entries.len() {
        app.harness_promotions_cursor = app.harness_promotions.entries.len().saturating_sub(1);
    }
}

fn apply_action_response(app: &mut App, response: HarnessPromotionActionResponse) -> String {
    let message = response.message;
    app.harness_promotions = response.board;
    clamp_cursor(app);
    app.harness_promotions_status = Some(message.clone());
    message
}

pub fn selected_entry(app: &App) -> Option<&HarnessPromotionBoardEntry> {
    app.harness_promotions
        .entries
        .get(app.harness_promotions_cursor)
}

pub fn visible_window(
    board: &HarnessPromotionBoard,
    cursor: usize,
    viewport_rows: usize,
) -> (usize, usize) {
    if board.entries.is_empty() {
        return (0, 0);
    }
    let capacity = (viewport_rows / PROMOTION_ENTRY_LINES).max(1);
    let cur = cursor.min(board.entries.len().saturating_sub(1));
    let start = cur.saturating_sub(capacity / 2);
    let end = (start + capacity).min(board.entries.len());
    (start, end)
}

pub fn select_visible_row(app: &mut App, viewport_rows: usize, row_offset: usize) -> bool {
    if app.harness_promotions.entries.is_empty() {
        return false;
    }
    let (start, end) = visible_window(
        &app.harness_promotions,
        app.harness_promotions_cursor,
        viewport_rows,
    );
    let idx = start + (row_offset / PROMOTION_ENTRY_LINES);
    if idx >= end {
        return false;
    }
    app.harness_promotions_cursor = idx;
    true
}

pub fn primary_action_label(entry: &HarnessPromotionBoardEntry) -> Option<&'static str> {
    if entry.can_apply {
        Some("apply")
    } else if entry.can_approve {
        Some("approve")
    } else {
        None
    }
}

pub fn primary_action_hint(entry: &HarnessPromotionBoardEntry) -> &'static str {
    match primary_action_label(entry) {
        Some("apply") => "Enter=apply",
        Some("approve") => "Enter=approve",
        _ => "Enter=no-op",
    }
}

pub fn run_primary_action(app: &mut App) -> Result<String> {
    let entry = selected_entry(app).ok_or_else(|| anyhow!("no promotion selected"))?;
    if entry.can_apply {
        apply_selected(app)
    } else if entry.can_approve {
        approve_selected(app)
    } else {
        Err(anyhow!("selected promotion has no primary action"))
    }
}

pub fn selected_entry_clipboard_text(app: &App) -> Option<String> {
    let entry = selected_entry(app)?;
    let mut lines = vec![
        entry.title.clone(),
        format!("status: {}", entry.review_badge),
        format!("decision: {}", entry.badge),
        format!("contract_path: {}", entry.contract_path),
    ];
    if !entry.subtitle.trim().is_empty() {
        lines.push(format!("subtitle: {}", entry.subtitle));
    }
    if !entry.green_case_ids.is_empty() {
        lines.push(format!("green_cases: {}", entry.green_case_ids.join(", ")));
    }
    if !entry.reasons.is_empty() {
        lines.push("reasons:".to_string());
        lines.extend(entry.reasons.iter().map(|reason| format!("- {reason}")));
    }
    if let Some(path) = entry.patch_path.as_deref() {
        lines.push(format!("patch_path: {path}"));
    }
    Some(lines.join("\n"))
}

pub fn refresh_promotions(app: &mut App) -> Result<String> {
    let root = workspace_root()?;
    let board = harness_gate::load_board(&root)?;
    let message = board
        .status_message
        .clone()
        .unwrap_or_else(|| format!("loaded {} promotion candidate(s)", board.summary.total));
    app.harness_promotions = board;
    clamp_cursor(app);
    app.harness_promotions_status = Some(message.clone());
    Ok(message)
}

pub fn approve_selected(app: &mut App) -> Result<String> {
    let root = workspace_root()?;
    let entry = selected_entry(app).ok_or_else(|| anyhow!("no promotion selected"))?;
    let response = harness_gate::approve(&root, &entry.id)?;
    Ok(apply_action_response(app, response))
}

pub fn hold_selected(app: &mut App) -> Result<String> {
    let root = workspace_root()?;
    let entry = selected_entry(app).ok_or_else(|| anyhow!("no promotion selected"))?;
    let response = harness_gate::hold(&root, &entry.id)?;
    Ok(apply_action_response(app, response))
}

pub fn apply_selected(app: &mut App) -> Result<String> {
    let root = workspace_root()?;
    let entry = selected_entry(app).ok_or_else(|| anyhow!("no promotion selected"))?;
    let response = harness_gate::apply_to_contract(&root, &entry.id)?;
    Ok(apply_action_response(app, response))
}

#[cfg(test)]
mod tests {
    use super::super::app::App;
    use super::*;
    use crate::config::{ProviderKind, RunConfig};
    use crate::governor_contract::RuntimeOverlayTemplate;
    use crate::harness_gate::HarnessPromotionBoardStatus;
    use crate::harness_promotion::PromotionDecision;
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
        status: HarnessPromotionBoardStatus,
        can_approve: bool,
        can_apply: bool,
    ) -> HarnessPromotionBoardEntry {
        HarnessPromotionBoardEntry {
            id: id.to_string(),
            decision: PromotionDecision::Update,
            review_status: status,
            review_badge: match status {
                HarnessPromotionBoardStatus::NeedsReview => "needs_review",
                HarnessPromotionBoardStatus::Approved => "approved",
                HarnessPromotionBoardStatus::Held => "held",
                HarnessPromotionBoardStatus::Applied => "applied",
                HarnessPromotionBoardStatus::UpToDate => "up_to_date",
                HarnessPromotionBoardStatus::Blocked => "blocked",
            }
            .to_string(),
            can_approve,
            can_hold: true,
            can_apply,
            review_record: None,
            title: format!("entry {id}"),
            subtitle: "Promote runtime overlay".to_string(),
            badge: "update".to_string(),
            contract_path: format!("runtime_overlay_templates.{id}"),
            reasons: vec!["loop fixed".to_string()],
            green_case_ids: vec!["fix-failing-rust-test".to_string()],
            existing_template: None,
            proposed_template: RuntimeOverlayTemplate::default(),
            patch_path: Some(format!("{id}.json")),
        }
    }

    fn test_app(entries: Vec<HarnessPromotionBoardEntry>) -> App {
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
        app.harness_promotions.entries = entries;
        app.harness_promotions.summary.total = app.harness_promotions.entries.len();
        app
    }

    #[test]
    fn visible_window_centers_cursor_by_entry_capacity() {
        let app = test_app(vec![
            test_entry("a", HarnessPromotionBoardStatus::NeedsReview, true, false),
            test_entry("b", HarnessPromotionBoardStatus::NeedsReview, true, false),
            test_entry("c", HarnessPromotionBoardStatus::Approved, true, true),
            test_entry("d", HarnessPromotionBoardStatus::Held, false, false),
        ]);
        assert_eq!(visible_window(&app.harness_promotions, 2, 4), (1, 3));
    }

    #[test]
    fn select_visible_row_maps_rows_to_entries() {
        let mut app = test_app(vec![
            test_entry("a", HarnessPromotionBoardStatus::NeedsReview, true, false),
            test_entry("b", HarnessPromotionBoardStatus::Approved, true, true),
            test_entry("c", HarnessPromotionBoardStatus::Held, false, false),
        ]);
        app.harness_promotions_cursor = 1;
        assert!(select_visible_row(&mut app, 6, 4));
        assert_eq!(app.harness_promotions_cursor, 2);
    }

    #[test]
    fn primary_action_prefers_apply_after_approval() {
        let entry = test_entry("b", HarnessPromotionBoardStatus::Approved, true, true);
        assert_eq!(primary_action_label(&entry), Some("apply"));
        assert_eq!(primary_action_hint(&entry), "Enter=apply");
    }

    #[test]
    fn clipboard_text_includes_path_and_reasons() {
        let app = test_app(vec![test_entry(
            "scaffold_repo",
            HarnessPromotionBoardStatus::NeedsReview,
            true,
            false,
        )]);
        let text = selected_entry_clipboard_text(&app).expect("clipboard text");
        assert!(text.contains("runtime_overlay_templates.scaffold_repo"));
        assert!(text.contains("loop fixed"));
        assert!(text.contains("patch_path: scaffold_repo.json"));
    }
}
