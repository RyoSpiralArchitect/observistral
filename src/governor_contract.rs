use std::collections::BTreeMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorField {
    pub key: String,
    pub hint: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub allowed_values: Vec<String>,
    #[serde(default)]
    pub allowed_values_from: Option<String>,
    #[serde(default)]
    pub value_aliases: BTreeMap<String, String>,
    #[serde(default)]
    pub min_items: Option<usize>,
    #[serde(default)]
    pub max_items: Option<usize>,
    #[serde(default)]
    pub min_value: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorBlock {
    pub title: String,
    pub tag: String,
    pub fields: Vec<GovernorField>,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoneContract {
    pub required_args: Vec<String>,
    pub acceptance_evidence_fields: Vec<String>,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptLayout {
    pub block_order: Vec<String>,
    pub done_title: String,
    pub done_args_template: String,
    pub error_title: String,
    pub error_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationContract {
    #[serde(default)]
    pub intent_doc_terms: Vec<String>,
    #[serde(default)]
    pub intent_behavioral_terms: Vec<String>,
    #[serde(default)]
    pub goal_test_terms: Vec<String>,
    #[serde(default)]
    pub goal_build_terms: Vec<String>,
    #[serde(default)]
    pub goal_repo_terms: Vec<String>,
    #[serde(default)]
    pub goal_check_runners: Vec<GoalCheckRunner>,
    #[serde(default)]
    pub repo_goal_requirements: Vec<RepoGoalRequirement>,
    #[serde(default)]
    pub goal_check_policy: GoalCheckPolicy,
    #[serde(default)]
    pub plan_build_terms: Vec<String>,
    #[serde(default)]
    pub plan_behavioral_terms: Vec<String>,
    #[serde(default)]
    pub doc_path_terms: Vec<String>,
    #[serde(default)]
    pub behavioral_path_extensions: Vec<String>,
    #[serde(default)]
    pub ignore_command_signatures: Vec<String>,
    #[serde(default)]
    pub build_command_signatures: Vec<String>,
    #[serde(default)]
    pub behavioral_command_signatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalCheckRunner {
    #[serde(default)]
    pub detect_files_any: Vec<String>,
    #[serde(default)]
    pub test_command: Option<String>,
    #[serde(default)]
    pub build_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoGoalRequirement {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub probe: String,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoalCheckPolicy {
    #[serde(default)]
    pub run_on_stop: bool,
    #[serde(default)]
    pub require_longrun: bool,
    #[serde(default)]
    pub require_exec_feature: bool,
    #[serde(default)]
    pub require_command_approval_off: bool,
    #[serde(default)]
    pub max_attempts_per_goal: usize,
    #[serde(default)]
    pub goal_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorMessages {
    pub multiple_tool_calls: String,
    pub plan_invalid: String,
    pub plan_missing: String,
    pub plan_missing_goal: String,
    pub plan_missing_steps: String,
    pub plan_min_steps: String,
    pub plan_max_steps: String,
    pub plan_missing_acceptance: String,
    pub plan_min_acceptance: String,
    pub plan_max_acceptance: String,
    pub plan_missing_risks: String,
    pub plan_missing_assumptions: String,
    pub plan_empty_step: String,
    pub plan_empty_acceptance: String,
    pub think_missing: String,
    pub think_invalid: String,
    pub think_missing_goal: String,
    pub think_invalid_step: String,
    pub think_requires_plan: String,
    pub think_step_out_of_range: String,
    pub think_invalid_tool: String,
    pub think_missing_risk: String,
    pub think_missing_doubt: String,
    pub think_missing_next: String,
    pub think_missing_verify: String,
    pub think_tool_mismatch: String,
    pub think_exec_prefix_mismatch: String,
    pub reflection_missing: String,
    pub reflection_invalid: String,
    pub reflection_one_tool: String,
    pub reflection_stop: String,
    pub reflection_missing_last_outcome: String,
    pub reflection_missing_wrong_assumption: String,
    pub reflection_missing_next_minimal_action: String,
    pub reflection_invalid_goal_delta: String,
    pub reflection_invalid_strategy_change: String,
    pub reflection_requires_strategy_change: String,
    pub reflection_non_improving_requires_change: String,
    pub impact_missing: String,
    pub impact_invalid: String,
    pub impact_one_tool: String,
    pub impact_stop: String,
    pub impact_missing_changed: String,
    pub impact_missing_progress: String,
    pub impact_missing_remaining_gap: String,
    pub impact_requires_plan: String,
    pub impact_invalid_progress_reference: String,
    pub done_invalid_acceptance: String,
    pub done_requires_plan: String,
    pub done_missing_criteria: String,
    pub done_completed_invalid_reference: String,
    pub done_remaining_invalid_reference: String,
    pub done_duplicate_criteria: String,
    pub done_incomplete_coverage: String,
    pub done_evidence_incomplete: String,
    pub done_evidence_invalid_reference: String,
    pub done_evidence_only_completed: String,
    pub done_evidence_duplicate_criteria: String,
    pub done_evidence_unknown_command: String,
    pub goal_check_repo_start: String,
    pub goal_check_repo_ok: String,
    pub goal_check_exec_run: String,
    pub goal_check_exec_ok: String,
    pub goal_check_exec_fail: String,
    pub goal_check_all_passed: String,
    pub goal_check_supported_runners: String,
    pub goal_check_tests_runner_fallback: String,
    pub goal_check_build_runner_fallback: String,
    pub goal_check_repo_missing: String,
    pub goal_check_tests_no_runner: String,
    pub goal_check_tests_failed: String,
    pub goal_check_build_no_runner: String,
    pub goal_check_build_failed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorContract {
    pub tool_names: Vec<String>,
    pub diagnostic_tools: Vec<String>,
    pub plan: GovernorBlock,
    pub think: GovernorBlock,
    pub reflect: GovernorBlock,
    pub impact: GovernorBlock,
    pub done: DoneContract,
    pub verification: VerificationContract,
    pub prompt_layout: PromptLayout,
    pub messages: GovernorMessages,
}

static CONTRACT: Lazy<GovernorContract> = Lazy::new(|| {
    serde_json::from_str(include_str!("../shared/governor_contract.json"))
        .expect("shared governor_contract.json must be valid")
});

pub fn contract() -> &'static GovernorContract {
    &CONTRACT
}

pub fn browser_fallback_script() -> String {
    let json = serde_json::to_string(contract())
        .expect("shared governor contract must serialize for browser fallback");
    format!("window.__OBSTRAL_GOVERNOR_CONTRACT_FALLBACK__ = {json};\n")
}

pub fn diagnostic_tools_hint() -> String {
    contract().diagnostic_tools.join("/")
}

pub fn diagnostic_tool_names() -> &'static [String] {
    &contract().diagnostic_tools
}

pub fn done_required_args() -> &'static [String] {
    &contract().done.required_args
}

pub fn done_acceptance_evidence_fields() -> &'static [String] {
    &contract().done.acceptance_evidence_fields
}

pub fn verification() -> &'static VerificationContract {
    &contract().verification
}

pub fn block(tag: &str) -> Option<&'static GovernorBlock> {
    match tag {
        "plan" => Some(&contract().plan),
        "think" => Some(&contract().think),
        "reflect" => Some(&contract().reflect),
        "impact" => Some(&contract().impact),
        _ => None,
    }
}

pub fn find_block_field<'a>(block: &'a GovernorBlock, raw_key: &str) -> Option<&'a GovernorField> {
    let want = raw_key.trim().to_ascii_lowercase();
    if want.is_empty() {
        return None;
    }
    block.fields.iter().find(|field| {
        field.key.eq_ignore_ascii_case(&want)
            || field
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(&want))
    })
}

pub fn block_field(tag: &str, raw_key: &str) -> Option<&'static GovernorField> {
    let block = block(tag)?;
    find_block_field(block, raw_key)
}

pub fn field_allowed_values(tag: &str, raw_key: &str) -> Vec<String> {
    let Some(field) = block_field(tag, raw_key) else {
        return Vec::new();
    };
    if field.allowed_values_from.as_deref() == Some("tool_names") {
        return contract().tool_names.clone();
    }
    field.allowed_values.clone()
}

pub fn canonical_field_value(tag: &str, raw_key: &str, value: &str) -> Option<String> {
    let field = block_field(tag, raw_key)?;
    let normalized = value.trim().to_ascii_lowercase().replace(' ', "_");
    if normalized.is_empty() {
        return None;
    }
    if let Some(mapped) = field.value_aliases.get(&normalized) {
        return Some(mapped.clone());
    }
    let allowed = field_allowed_values(tag, raw_key);
    if allowed.iter().any(|candidate| candidate == &normalized) {
        return Some(normalized);
    }
    if allowed.is_empty() && matches!(field.kind.as_deref(), Some("string")) {
        return Some(value.trim().to_string());
    }
    None
}

fn field_keys(block: &GovernorBlock) -> String {
    block
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<Vec<_>>()
        .join("/")
}

fn render_template(template: &str, replacements: &[(&str, String)]) -> String {
    let mut out = template.to_string();
    for (key, value) in replacements {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

pub fn multiple_tool_calls_message(count: usize) -> String {
    render_template(
        &contract().messages.multiple_tool_calls,
        &[("count", count.to_string())],
    )
}

pub fn invalid_plan_message(error: &str) -> String {
    render_template(
        &contract().messages.plan_invalid,
        &[
            ("error", error.to_string()),
            ("plan_fields", field_keys(&contract().plan)),
        ],
    )
}

pub fn missing_plan_message() -> String {
    render_template(
        &contract().messages.plan_missing,
        &[
            ("plan_fields", field_keys(&contract().plan)),
            ("diagnostic_tools", diagnostic_tools_hint()),
        ],
    )
}

pub fn missing_think_message() -> String {
    render_template(
        &contract().messages.think_missing,
        &[("think_fields", field_keys(&contract().think))],
    )
}

pub fn invalid_think_message(error: &str) -> String {
    render_template(
        &contract().messages.think_invalid,
        &[("error", error.to_string())],
    )
}

pub fn plan_missing_goal_message() -> String {
    contract().messages.plan_missing_goal.clone()
}

pub fn plan_missing_steps_message() -> String {
    contract().messages.plan_missing_steps.clone()
}

pub fn plan_min_steps_message(min_steps: usize) -> String {
    render_template(
        &contract().messages.plan_min_steps,
        &[("min_steps", min_steps.to_string())],
    )
}

pub fn plan_max_steps_message(max_steps: usize) -> String {
    render_template(
        &contract().messages.plan_max_steps,
        &[("max_steps", max_steps.to_string())],
    )
}

pub fn plan_missing_acceptance_message() -> String {
    contract().messages.plan_missing_acceptance.clone()
}

pub fn plan_min_acceptance_message(min_acceptance: usize) -> String {
    render_template(
        &contract().messages.plan_min_acceptance,
        &[("min_acceptance", min_acceptance.to_string())],
    )
}

pub fn plan_max_acceptance_message(max_acceptance: usize) -> String {
    render_template(
        &contract().messages.plan_max_acceptance,
        &[("max_acceptance", max_acceptance.to_string())],
    )
}

pub fn plan_missing_risks_message() -> String {
    contract().messages.plan_missing_risks.clone()
}

pub fn plan_missing_assumptions_message() -> String {
    contract().messages.plan_missing_assumptions.clone()
}

pub fn plan_empty_step_message() -> String {
    contract().messages.plan_empty_step.clone()
}

pub fn plan_empty_acceptance_message() -> String {
    contract().messages.plan_empty_acceptance.clone()
}

pub fn think_missing_goal_message() -> String {
    contract().messages.think_missing_goal.clone()
}

pub fn think_invalid_step_message() -> String {
    contract().messages.think_invalid_step.clone()
}

pub fn think_step_out_of_range_message(step: usize, plan_steps: usize) -> String {
    render_template(
        &contract().messages.think_step_out_of_range,
        &[
            ("step", step.to_string()),
            ("plan_steps", plan_steps.to_string()),
        ],
    )
}

pub fn think_invalid_tool_message() -> String {
    contract().messages.think_invalid_tool.clone()
}

pub fn think_missing_risk_message() -> String {
    contract().messages.think_missing_risk.clone()
}

pub fn think_missing_doubt_message() -> String {
    contract().messages.think_missing_doubt.clone()
}

pub fn think_missing_next_message() -> String {
    contract().messages.think_missing_next.clone()
}

pub fn think_missing_verify_message() -> String {
    contract().messages.think_missing_verify.clone()
}

pub fn think_tool_mismatch_message(think_tool: &str, actual_tool: &str) -> String {
    render_template(
        &contract().messages.think_tool_mismatch,
        &[
            ("think_tool", think_tool.to_string()),
            ("actual_tool", actual_tool.to_string()),
        ],
    )
}

pub fn think_exec_prefix_mismatch_message() -> String {
    contract().messages.think_exec_prefix_mismatch.clone()
}

pub fn reflection_missing_message(reason: &str) -> String {
    render_template(
        &contract().messages.reflection_missing,
        &[("reason", reason.to_string())],
    )
}

pub fn reflection_invalid_message(error: &str, reason: &str) -> String {
    render_template(
        &contract().messages.reflection_invalid,
        &[("error", error.to_string()), ("reason", reason.to_string())],
    )
}

pub fn reflection_one_tool_message(count: usize) -> String {
    render_template(
        &contract().messages.reflection_one_tool,
        &[("count", count.to_string())],
    )
}

pub fn reflection_missing_last_outcome_message() -> String {
    contract().messages.reflection_missing_last_outcome.clone()
}

pub fn reflection_missing_wrong_assumption_message() -> String {
    contract()
        .messages
        .reflection_missing_wrong_assumption
        .clone()
}

pub fn reflection_missing_next_minimal_action_message() -> String {
    contract()
        .messages
        .reflection_missing_next_minimal_action
        .clone()
}

pub fn reflection_invalid_goal_delta_message() -> String {
    contract().messages.reflection_invalid_goal_delta.clone()
}

pub fn reflection_invalid_strategy_change_message() -> String {
    contract()
        .messages
        .reflection_invalid_strategy_change
        .clone()
}

pub fn reflection_requires_strategy_change_message() -> String {
    contract()
        .messages
        .reflection_requires_strategy_change
        .clone()
}

pub fn reflection_non_improving_requires_change_message() -> String {
    contract()
        .messages
        .reflection_non_improving_requires_change
        .clone()
}

pub fn impact_missing_message(reason: &str) -> String {
    render_template(
        &contract().messages.impact_missing,
        &[("reason", reason.to_string())],
    )
}

pub fn impact_invalid_message(error: &str, reason: &str) -> String {
    render_template(
        &contract().messages.impact_invalid,
        &[("error", error.to_string()), ("reason", reason.to_string())],
    )
}

pub fn impact_one_tool_message(count: usize) -> String {
    render_template(
        &contract().messages.impact_one_tool,
        &[("count", count.to_string())],
    )
}

pub fn impact_missing_changed_message() -> String {
    contract().messages.impact_missing_changed.clone()
}

pub fn impact_missing_progress_message() -> String {
    contract().messages.impact_missing_progress.clone()
}

pub fn impact_missing_remaining_gap_message() -> String {
    contract().messages.impact_missing_remaining_gap.clone()
}

pub fn impact_requires_plan_message() -> String {
    contract().messages.impact_requires_plan.clone()
}

pub fn impact_invalid_progress_reference_message() -> String {
    contract()
        .messages
        .impact_invalid_progress_reference
        .clone()
}

pub fn done_invalid_acceptance_message(error: &str) -> String {
    render_template(
        &contract().messages.done_invalid_acceptance,
        &[("error", error.to_string())],
    )
}

pub fn done_requires_plan_message() -> String {
    contract().messages.done_requires_plan.clone()
}

pub fn done_missing_criteria_message() -> String {
    contract().messages.done_missing_criteria.clone()
}

pub fn done_completed_invalid_reference_message() -> String {
    contract().messages.done_completed_invalid_reference.clone()
}

pub fn done_remaining_invalid_reference_message() -> String {
    contract().messages.done_remaining_invalid_reference.clone()
}

pub fn done_duplicate_criteria_message() -> String {
    contract().messages.done_duplicate_criteria.clone()
}

pub fn done_incomplete_coverage_message() -> String {
    contract().messages.done_incomplete_coverage.clone()
}

pub fn done_evidence_incomplete_message() -> String {
    contract().messages.done_evidence_incomplete.clone()
}

pub fn done_evidence_invalid_reference_message() -> String {
    contract().messages.done_evidence_invalid_reference.clone()
}

pub fn done_evidence_only_completed_message() -> String {
    contract().messages.done_evidence_only_completed.clone()
}

pub fn done_evidence_duplicate_criteria_message() -> String {
    contract().messages.done_evidence_duplicate_criteria.clone()
}

pub fn done_evidence_unknown_command_message() -> String {
    contract().messages.done_evidence_unknown_command.clone()
}

fn goal_check_repo_requirements_summary() -> String {
    let labels = verification()
        .repo_goal_requirements
        .iter()
        .map(|requirement| {
            let label = requirement.label.trim();
            if label.is_empty() {
                requirement.key.trim()
            } else {
                label
            }
        })
        .filter(|label| !label.is_empty())
        .collect::<Vec<_>>();
    if labels.is_empty() {
        ".git / HEAD / README.md".to_string()
    } else {
        labels.join(" / ")
    }
}

pub fn goal_check_repo_start_message() -> String {
    render_template(
        &contract().messages.goal_check_repo_start,
        &[("requirements", goal_check_repo_requirements_summary())],
    )
}

pub fn goal_check_repo_ok_message() -> String {
    contract().messages.goal_check_repo_ok.clone()
}

pub fn goal_check_exec_run_message(label: &str, command: &str) -> String {
    render_template(
        &contract().messages.goal_check_exec_run,
        &[
            ("label", label.to_string()),
            ("command", command.to_string()),
        ],
    )
}

pub fn goal_check_exec_ok_message(label: &str, command: &str) -> String {
    render_template(
        &contract().messages.goal_check_exec_ok,
        &[
            ("label", label.to_string()),
            ("command", command.to_string()),
        ],
    )
}

pub fn goal_check_exec_fail_message(label: &str, command: &str, digest_line: &str) -> String {
    render_template(
        &contract().messages.goal_check_exec_fail,
        &[
            ("label", label.to_string()),
            ("command", command.to_string()),
            ("digest_line", digest_line.to_string()),
        ],
    )
}

pub fn goal_check_all_passed_message() -> String {
    contract().messages.goal_check_all_passed.clone()
}

pub fn goal_check_supported_runners_message(summary: &str) -> String {
    render_template(
        &contract().messages.goal_check_supported_runners,
        &[("summary", summary.to_string())],
    )
}

pub fn goal_check_tests_runner_fallback_message() -> String {
    contract().messages.goal_check_tests_runner_fallback.clone()
}

pub fn goal_check_build_runner_fallback_message() -> String {
    contract().messages.goal_check_build_runner_fallback.clone()
}

pub fn goal_check_repo_missing_message(missing: &str) -> String {
    render_template(
        &contract().messages.goal_check_repo_missing,
        &[("missing", missing.to_string())],
    )
}

pub fn goal_check_tests_no_runner_message(supported_runners_line: &str) -> String {
    render_template(
        &contract().messages.goal_check_tests_no_runner,
        &[("supported_runners_line", supported_runners_line.to_string())],
    )
}

pub fn goal_check_tests_failed_message(class_line: &str, digest_line: &str) -> String {
    render_template(
        &contract().messages.goal_check_tests_failed,
        &[
            ("class_line", class_line.to_string()),
            ("digest_line", digest_line.to_string()),
        ],
    )
}

pub fn goal_check_build_no_runner_message(supported_runners_line: &str) -> String {
    render_template(
        &contract().messages.goal_check_build_no_runner,
        &[("supported_runners_line", supported_runners_line.to_string())],
    )
}

pub fn goal_check_build_failed_message(class_line: &str, digest_line: &str) -> String {
    render_template(
        &contract().messages.goal_check_build_failed,
        &[
            ("class_line", class_line.to_string()),
            ("digest_line", digest_line.to_string()),
        ],
    )
}

fn render_block(block: &GovernorBlock) -> String {
    let mut out = String::new();
    out.push_str(&format!("[{}]\n", block.title));
    out.push_str(&format!("<{}>\n", block.tag));
    for field in &block.fields {
        out.push_str(&format!("{}: {}\n", field.key, field.hint));
    }
    out.push_str(&format!("</{}>", block.tag));
    if !block.rules.is_empty() {
        out.push('\n');
        for rule in &block.rules {
            out.push_str(rule);
            out.push('\n');
        }
        out.pop();
    }
    out
}

pub fn system_reasoning_prompt() -> String {
    let c = contract();
    let mut parts: Vec<String> = Vec::new();

    for tag in &c.prompt_layout.block_order {
        if let Some(block) = block(tag.as_str()) {
            parts.push(render_block(block));
        }
    }

    let done_args = c.done.required_args.join(", ");
    let done_line = render_template(
        &c.prompt_layout.done_args_template,
        &[("done_args", done_args)],
    );
    let mut done_section = vec![format!("[{}]", c.prompt_layout.done_title)];
    done_section.extend(c.done.rules.iter().cloned());
    done_section.push(done_line);
    parts.push(done_section.join("\n"));

    let mut error_section = vec![format!("[{}]", c.prompt_layout.error_title)];
    error_section.extend(c.prompt_layout.error_rules.iter().cloned());
    parts.push(error_section.join("\n"));

    format!("\n\n{}", parts.join("\n\n"))
}

pub fn scratchpad_addon() -> String {
    system_reasoning_prompt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_contract_has_core_blocks() {
        let c = contract();
        assert!(!c.tool_names.is_empty());
        assert!(!c.diagnostic_tools.is_empty());
        assert_eq!(c.plan.tag, "plan");
        assert_eq!(c.think.tag, "think");
        assert_eq!(c.reflect.tag, "reflect");
        assert_eq!(c.impact.tag, "impact");
        assert!(!c.prompt_layout.block_order.is_empty());
        assert!(!c.prompt_layout.error_rules.is_empty());
        assert!(!c.verification.intent_doc_terms.is_empty());
        assert!(!c.verification.behavioral_command_signatures.is_empty());
        assert!(!c.verification.goal_repo_terms.is_empty());
        assert!(!c.verification.goal_check_runners.is_empty());
        assert!(!c.verification.repo_goal_requirements.is_empty());
        assert!(c.verification.goal_check_policy.run_on_stop);
        assert!(c.verification.goal_check_policy.max_attempts_per_goal > 0);
        assert!(!c.verification.goal_check_policy.goal_order.is_empty());
        assert!(c
            .done
            .required_args
            .iter()
            .any(|arg| arg == "acceptance_evidence"));
        assert!(c.messages.plan_invalid.contains("{error}"));
        assert!(c.messages.think_missing.contains("<think>"));
        assert!(c.messages.plan_min_steps.contains("{min_steps}"));
        assert!(c.messages.think_tool_mismatch.contains("{actual_tool}"));
        assert!(c
            .messages
            .done_evidence_unknown_command
            .contains("known successful verification command"));
        assert!(c
            .messages
            .reflection_invalid_goal_delta
            .contains("goal_delta"));
        assert!(c
            .messages
            .impact_invalid_progress_reference
            .contains("impact.progress"));
        assert!(c.messages.goal_check_repo_start.contains("{requirements}"));
        assert!(c.messages.goal_check_exec_run.contains("{command}"));
        assert!(c
            .messages
            .goal_check_all_passed
            .contains("all requested stop checks passed"));
        assert!(c
            .messages
            .goal_check_supported_runners
            .contains("{summary}"));
        assert!(c.messages.goal_check_repo_missing.contains("[goal_check]"));
        assert!(c
            .messages
            .goal_check_tests_failed
            .contains("Tests are failing"));
    }

    #[test]
    fn shared_contract_exposes_alias_and_enum_metadata() {
        let acceptance = block_field("plan", "acceptance_criteria").expect("acceptance alias");
        assert_eq!(acceptance.key, "acceptance");

        let tool = canonical_field_value("think", "tool", "read").expect("tool alias");
        assert_eq!(tool, "read_file");

        let delta = canonical_field_value("reflect", "goal_delta", "further").expect("enum alias");
        assert_eq!(delta, "farther");
    }

    #[test]
    fn browser_fallback_script_bootstraps_window_contract() {
        let js = browser_fallback_script();
        assert!(js.contains("window.__OBSTRAL_GOVERNOR_CONTRACT_FALLBACK__"));
        assert!(js.contains("\"tool_names\""));
        assert!(js.contains("\"messages\""));
    }

    #[test]
    fn system_reasoning_prompt_uses_shared_layout() {
        let prompt = system_reasoning_prompt();
        assert!(prompt.contains("[Done Protocol]"));
        assert!(prompt.contains("[Error Protocol]"));
        assert!(prompt.contains("done must include"));
        assert!(prompt.contains("<reflect>"));
    }

    #[test]
    fn goal_check_log_messages_render_from_contract() {
        assert!(goal_check_repo_start_message().contains(".git"));
        assert_eq!(goal_check_repo_ok_message(), "[goal_check:repo] OK");
        assert_eq!(
            goal_check_exec_run_message("tests", "cargo test -q"),
            "[goal_check:tests] run `cargo test -q`"
        );
        assert_eq!(
            goal_check_exec_ok_message("build", "cargo check"),
            "[goal_check:build] OK `cargo check`"
        );
        assert_eq!(
            goal_check_exec_fail_message("tests", "cargo test -q", "1 test failed"),
            "[goal_check:tests] FAIL `cargo test -q`\n1 test failed"
        );
        assert_eq!(
            goal_check_all_passed_message(),
            "[goal_check] all requested stop checks passed"
        );
        assert_eq!(
            goal_check_supported_runners_message("Cargo.toml -> cargo test -q"),
            "Supported runners: Cargo.toml -> cargo test -q."
        );
        assert_eq!(
            goal_check_tests_runner_fallback_message(),
            "If tests are required, configure a supported test runner and re-run."
        );
        assert_eq!(
            goal_check_build_runner_fallback_message(),
            "If build is required, add build instructions/scripts for this repo and run them."
        );
    }
}
