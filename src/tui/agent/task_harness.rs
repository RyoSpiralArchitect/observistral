use super::repo_scaffold::{
    default_repo_gitignore, repo_root_from_test_cmd, required_repo_files_from_test_cmd,
    resolve_repo_file_path, resolve_repo_scaffold_path, scaffold_repo_file_content,
};
use super::{canonicalize_tool_call_command, compact_one_line, RecoveryStage};
use crate::streaming::ToolCallData;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TaskLane {
    ReadOnlyObserve,
    FixExisting,
    EditExisting,
    CreateFile,
    InitRepo,
    ScaffoldRepo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ArtifactMode {
    ObserveOnly,
    ExistingFiles,
    NewFiles,
    NewRepo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TaskHarness {
    pub lane: TaskLane,
    pub artifact_mode: ArtifactMode,
}

impl TaskHarness {
    pub(super) fn infer(root_user_text: &str, root_read_only: bool) -> Self {
        if root_read_only {
            return Self {
                lane: TaskLane::ReadOnlyObserve,
                artifact_mode: ArtifactMode::ObserveOnly,
            };
        }

        let low = root_user_text.to_ascii_lowercase();
        let mentions_repo = text_contains_any(
            low.as_str(),
            &[
                "repo",
                "repository",
                "git repo",
                "git repository",
                "project",
                "workspace",
                "service",
                "app",
                "リポ",
                "リポジトリ",
                "新規プロジェクト",
                "プロジェクト",
            ],
        );

        if text_contains_any(
            low.as_str(),
            &[
                "failing test",
                "fix the failing",
                "smallest code change",
                "bug",
                "regression",
                "broken",
                "repair",
                "debug",
                "panic",
                "fix ",
                "修正",
                "直す",
                "バグ",
                "壊れ",
                "失敗テスト",
                "デバッグ",
            ],
        ) {
            return Self {
                lane: TaskLane::FixExisting,
                artifact_mode: ArtifactMode::ExistingFiles,
            };
        }

        if mentions_repo
            && text_contains_any(
                low.as_str(),
                &[
                    "git init",
                    "initialize",
                    "initialise",
                    "initialize a new repo",
                    "initialize git",
                    "init a new repo",
                    "init repo",
                    "初期化",
                    "git リポジトリ",
                ],
            )
        {
            return Self {
                lane: TaskLane::InitRepo,
                artifact_mode: ArtifactMode::NewRepo,
            };
        }

        if mentions_repo
            && text_contains_any(
                low.as_str(),
                &["create", "new", "make", "add", "作成", "作る", "新規"],
            )
        {
            return Self {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            };
        }

        if mentions_repo
            && text_contains_any(
                low.as_str(),
                &[
                    "scaffold",
                    "bootstrap",
                    "starter",
                    "boilerplate",
                    "template",
                    "skeleton",
                    "雛形",
                    "テンプレート",
                    "ひな形",
                    "スキャフォールド",
                ],
            )
        {
            return Self {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            };
        }

        if text_contains_any(
            low.as_str(),
            &[
                "create file",
                "new file",
                "write file",
                "add file",
                "create a file",
                "write a file",
                "create ",
                "write ",
                "ファイルを作成",
                "新規ファイル",
                "ファイルを追加",
                "作成",
                "作る",
            ],
        ) && (text_contains_any(low.as_str(), &["file", "ファイル"])
            || prompt_mentions_path_literal(root_user_text))
        {
            return Self {
                lane: TaskLane::CreateFile,
                artifact_mode: ArtifactMode::NewFiles,
            };
        }

        Self {
            lane: TaskLane::EditExisting,
            artifact_mode: ArtifactMode::ExistingFiles,
        }
    }

    pub(super) fn lane_label(self) -> &'static str {
        match self.lane {
            TaskLane::ReadOnlyObserve => "read_only_observe",
            TaskLane::FixExisting => "fix_existing_files",
            TaskLane::EditExisting => "edit_existing_files",
            TaskLane::CreateFile => "create_file",
            TaskLane::InitRepo => "init_repo",
            TaskLane::ScaffoldRepo => "scaffold_repo",
        }
    }

    pub(super) fn artifact_mode_label(self) -> &'static str {
        match self.artifact_mode {
            ArtifactMode::ObserveOnly => "observe_only",
            ArtifactMode::ExistingFiles => "existing_files",
            ArtifactMode::NewFiles => "new_files",
            ArtifactMode::NewRepo => "new_repo",
        }
    }

    fn progress_shape_hint(self) -> &'static str {
        match self.lane {
            TaskLane::ReadOnlyObserve => {
                "Observe -> confirm -> done. Do not mutate or create files."
            }
            TaskLane::FixExisting => {
                "Inspect just enough to confirm the bug, then mutate, then verify."
            }
            TaskLane::EditExisting => {
                "Inspect just enough to confirm the target, then edit, then verify."
            }
            TaskLane::CreateFile => {
                "If the destination is clear, create the file early instead of over-inspecting."
            }
            TaskLane::InitRepo | TaskLane::ScaffoldRepo => {
                "Create the repo/project artifact early, then verify the created structure."
            }
        }
    }

    fn deliverable_hint(self) -> &'static str {
        match self.artifact_mode {
            ArtifactMode::ObserveOnly => "Leave no file mutations behind.",
            ArtifactMode::ExistingFiles => {
                "Completion must leave a concrete change in an existing project file."
            }
            ArtifactMode::NewFiles => "Completion must leave the requested new file(s) on disk.",
            ArtifactMode::NewRepo => {
                "Completion must leave the requested repo/project directory on disk."
            }
        }
    }

    pub(super) fn prompt(self, test_cmd: Option<&str>) -> String {
        let mut out = format!(
            "[Task Harness]\n\
lane: {}\n\
artifact_mode: {}\n\
progress_shape: {}\n\
deliverable: {}\n",
            self.lane_label(),
            self.artifact_mode_label(),
            self.progress_shape_hint(),
            self.deliverable_hint()
        );
        match self.artifact_mode {
            ArtifactMode::ObserveOnly => {
                out.push_str(
                    "When the target path/context is confirmed, stop probing and call `done`.\n",
                );
            }
            ArtifactMode::ExistingFiles => {
                out.push_str(
                    "Do not stay in repeated read/search loops once the current target file is confirmed.\n\
Move to `patch_file`/`apply_diff`, or run verification if you believe the fix is already present.\n",
                );
            }
            ArtifactMode::NewFiles => {
                out.push_str(
                    "Do not treat this as read-only exploration. Create the requested file when the destination is clear.\n",
                );
            }
            ArtifactMode::NewRepo => {
                out.push_str(
                    "Do not treat this as read-only exploration. Create the repo/project directory and starter artifacts when the destination is clear.\n",
                );
            }
        }
        if let Some(cmd) = test_cmd.filter(|cmd| !cmd.trim().is_empty()) {
            out.push_str("Configured verification command:\n- ");
            out.push_str(&compact_one_line(cmd, 180));
            out.push('\n');
        }
        out.push_str("If your next tool call would only repeat already-successful observation on the same target, change phase instead.");
        out
    }

    pub(super) fn compact_prompt(self, test_cmd: Option<&str>) -> String {
        let verify = test_cmd
            .filter(|cmd| !cmd.trim().is_empty())
            .map(|cmd| compact_one_line(cmd, 80))
            .unwrap_or_else(|| "none".to_string());
        [
            "[Task Harness cache]".to_string(),
            format!("- lane: {}", self.lane_label()),
            format!("- artifact_mode: {}", self.artifact_mode_label()),
            format!(
                "- progress_shape: {}",
                compact_one_line(self.progress_shape_hint(), 100)
            ),
            format!("- verify: {verify}"),
        ]
        .join("\n")
    }
}

pub(super) fn build_progress_gate_block(
    harness: TaskHarness,
    tc: &ToolCallData,
    messages: &[Value],
    recovery_stage: Option<RecoveryStage>,
    test_cmd: Option<&str>,
) -> Option<String> {
    if recovery_stage != Some(RecoveryStage::Fix) {
        return None;
    }
    if !is_observation_tool(tc.name.as_str()) {
        return None;
    }
    if harness.artifact_mode == ArtifactMode::ObserveOnly {
        return None;
    }

    let attempted = canonicalize_tool_call_command(tc.name.as_str(), tc.arguments.as_str())
        .unwrap_or_else(|| {
            format!(
                "{}({})",
                tc.name,
                compact_one_line(tc.arguments.as_str(), 120)
            )
        });
    let history = observation_history(messages);
    let same_successes = history.by_command.get(&attempted).copied().unwrap_or(0);
    if same_successes == 0 && history.total_successes < 2 {
        return None;
    }

    let next_action = match harness.artifact_mode {
        ArtifactMode::ExistingFiles => {
            "apply the smallest edit now with `patch_file` or `apply_diff`"
        }
        ArtifactMode::NewFiles => {
            "create the requested file now with `write_file` or a minimal `exec`"
        }
        ArtifactMode::NewRepo => {
            "create the requested repo/project artifact now with `write_file` or `exec`"
        }
        ArtifactMode::ObserveOnly => unreachable!(),
    };
    let verify_hint = test_cmd
        .filter(|cmd| !cmd.trim().is_empty())
        .map(|cmd| format!("If the artifact is already present, run the configured verification command now: `{}`.\n", compact_one_line(cmd, 140)))
        .unwrap_or_else(|| {
            "If you believe the artifact is already present, run a real command that proves it before `done`.\n".to_string()
        });

    Some(format!(
        "[Progress Gate]\n\
Task lane: {}\n\
Recovery stage is already `fix`.\n\
Attempted next action: {}\n\
Successful observation commands so far: {}\n\
This is stalled inspection, not progress.\n\
Required now: {}.\n\
{}\
Do NOT call the same observation tool on the same target again until the target changes or a mutation lands.",
        harness.lane_label(),
        compact_one_line(&attempted, 180),
        history.total_successes,
        next_action,
        verify_hint
    ))
}

pub(super) fn coerce_artifact_creation_tool_call(
    harness: TaskHarness,
    messages: &[Value],
    tc: &ToolCallData,
) -> Option<(ToolCallData, String, String)> {
    if !matches!(
        harness.artifact_mode,
        ArtifactMode::NewFiles | ArtifactMode::NewRepo
    ) {
        return None;
    }
    if !is_observation_tool(tc.name.as_str()) {
        return None;
    }

    let rewritten = recent_blocked_artifact_action(messages, harness)?;
    let original = canonicalize_tool_call_command(tc.name.as_str(), tc.arguments.as_str())
        .unwrap_or_else(|| {
            format!(
                "{}({})",
                tc.name,
                compact_one_line(tc.arguments.as_str(), 120)
            )
        });
    let coerced =
        canonicalize_tool_call_command(rewritten.name.as_str(), rewritten.arguments.as_str())
            .unwrap_or_else(|| {
                format!(
                    "{}({})",
                    rewritten.name,
                    compact_one_line(rewritten.arguments.as_str(), 120)
                )
            });
    if original == coerced {
        None
    } else {
        Some((rewritten, original, coerced))
    }
}

pub(super) fn coerce_repo_goal_completion_tool_call(
    harness: TaskHarness,
    messages: &[Value],
    tc: &ToolCallData,
    test_cmd: Option<&str>,
    tool_root: Option<&str>,
) -> Option<(ToolCallData, String, String)> {
    if harness.artifact_mode != ArtifactMode::NewRepo || !is_observation_tool(tc.name.as_str()) {
        return None;
    }
    let goal_check_missing = latest_goal_check_missing(messages);
    let status = repo_scaffold_status(messages, test_cmd?, tool_root)?;
    if goal_check_missing.is_empty() && !has_repo_scaffold_progress(&status) {
        return None;
    }
    let rewritten = next_repo_scaffold_tool_call(tc.id.as_str(), &status)?;
    let original = canonicalize_tool_call_command(tc.name.as_str(), tc.arguments.as_str())
        .unwrap_or_else(|| {
            format!(
                "{}({})",
                tc.name,
                compact_one_line(tc.arguments.as_str(), 120)
            )
        });
    let coerced =
        canonicalize_tool_call_command(rewritten.name.as_str(), rewritten.arguments.as_str())
            .unwrap_or_else(|| compact_one_line(rewritten.arguments.as_str(), 200));
    if original == coerced {
        None
    } else {
        Some((rewritten, original, coerced))
    }
}

pub(super) fn repair_repo_scaffold_write_tool_call(
    harness: TaskHarness,
    messages: &[Value],
    tc: &ToolCallData,
    test_cmd: Option<&str>,
    tool_root: Option<&str>,
) -> Option<(ToolCallData, String, String)> {
    if harness.artifact_mode != ArtifactMode::NewRepo || tc.name != "write_file" {
        return None;
    }
    if !write_file_looks_malformed(tc) {
        return None;
    }
    let status = repo_scaffold_status(messages, test_cmd?, tool_root)?;
    let rewritten = next_repo_scaffold_tool_call(tc.id.as_str(), &status)?;
    let original = canonicalize_tool_call_command(tc.name.as_str(), tc.arguments.as_str())
        .unwrap_or_else(|| {
            format!(
                "{}({})",
                tc.name,
                compact_one_line(tc.arguments.as_str(), 120)
            )
        });
    let coerced =
        canonicalize_tool_call_command(rewritten.name.as_str(), rewritten.arguments.as_str())
            .unwrap_or_else(|| compact_one_line(rewritten.arguments.as_str(), 200));
    if original == coerced {
        None
    } else {
        Some((rewritten, original, coerced))
    }
}

pub(super) fn allows_artifact_creation_during_diagnose(
    harness: TaskHarness,
    tc: &ToolCallData,
) -> bool {
    matches!(
        harness.artifact_mode,
        ArtifactMode::NewFiles | ArtifactMode::NewRepo
    ) && matches!(tc.name.as_str(), "write_file" | "exec")
}

pub(super) fn allows_artifact_creation_during_verify(
    harness: TaskHarness,
    tc: &ToolCallData,
) -> bool {
    harness.artifact_mode == ArtifactMode::NewRepo
        && matches!(tc.name.as_str(), "write_file" | "exec")
}

pub(super) fn build_fix_stage_progress_hint(
    harness: TaskHarness,
    messages: &[Value],
    recovery_stage: Option<RecoveryStage>,
    test_cmd: Option<&str>,
) -> Option<String> {
    if recovery_stage != Some(RecoveryStage::Fix)
        || harness.artifact_mode == ArtifactMode::ObserveOnly
    {
        return None;
    }

    let history = observation_history(messages);
    let threshold = match harness.artifact_mode {
        ArtifactMode::ExistingFiles => {
            if history
                .by_command
                .keys()
                .any(|command| command.starts_with("read_file("))
            {
                1
            } else {
                2
            }
        }
        ArtifactMode::NewFiles | ArtifactMode::NewRepo => 1,
        ArtifactMode::ObserveOnly => unreachable!(),
    };
    if history.total_successes < threshold {
        return None;
    }

    let recent = history
        .last_successful_command
        .as_deref()
        .unwrap_or("recent observation");
    let action = match harness.artifact_mode {
        ArtifactMode::ExistingFiles => {
            "apply the smallest edit now with `patch_file` or `apply_diff`"
        }
        ArtifactMode::NewFiles => {
            "create the requested file now with `write_file` or a minimal `exec`"
        }
        ArtifactMode::NewRepo => {
            "create the requested repo/project artifact now with `write_file` or `exec`"
        }
        ArtifactMode::ObserveOnly => unreachable!(),
    };
    let verify_hint = test_cmd
        .filter(|cmd| !cmd.trim().is_empty())
        .map(|cmd| format!("Then verify with `{}`.", compact_one_line(cmd, 140)))
        .unwrap_or_else(|| "Then run a real command that proves the artifact exists.".to_string());

    Some(format!(
        "[Task Harness]\n\
Task lane: {}\n\
Recent successful observation: {}\n\
You already have enough context for the current phase.\n\
Required now: {}.\n\
{}\n\
Do NOT continue with more read/search/list calls unless the target is still genuinely ambiguous.",
        harness.lane_label(),
        compact_one_line(recent, 160),
        action,
        verify_hint
    ))
}

#[derive(Debug, Default)]
struct ObservationHistory {
    total_successes: usize,
    last_successful_command: Option<String>,
    by_command: BTreeMap<String, usize>,
}

fn observation_history(messages: &[Value]) -> ObservationHistory {
    let mut pending_by_id: BTreeMap<String, String> = BTreeMap::new();
    let mut out = ObservationHistory::default();

    for msg in messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
        match role {
            "assistant" => {
                let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) else {
                    continue;
                };
                for tc in tool_calls {
                    let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("").trim();
                    let name = tc
                        .get("function")
                        .and_then(|v| v.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    let arguments = tc
                        .get("function")
                        .and_then(|v| v.get("arguments"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    if id.is_empty() || !is_observation_tool(name) {
                        continue;
                    }
                    if let Some(command) = canonicalize_tool_call_command(name, arguments) {
                        pending_by_id.insert(id.to_string(), command);
                    }
                }
            }
            "tool" => {
                let tcid = msg
                    .get("tool_call_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                let content = msg
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                let Some(command) = pending_by_id.get(tcid) else {
                    continue;
                };
                if !looks_like_successful_observation_result(content) {
                    continue;
                }
                out.total_successes += 1;
                out.last_successful_command = Some(command.clone());
                *out.by_command.entry(command.clone()).or_insert(0) += 1;
            }
            _ => {}
        }
    }

    out
}

fn looks_like_successful_observation_result(content: &str) -> bool {
    let trimmed = content.trim_start();
    trimmed.starts_with('[') && !trimmed.starts_with("[RESULT")
}

fn is_observation_tool(name: &str) -> bool {
    matches!(name, "read_file" | "search_files" | "list_dir" | "glob")
}

fn recent_blocked_artifact_action(
    messages: &[Value],
    harness: TaskHarness,
) -> Option<ToolCallData> {
    let mut idx = messages.len();
    while idx >= 2 {
        let tool_msg = &messages[idx - 1];
        let assistant_msg = &messages[idx - 2];
        idx -= 2;

        if tool_msg.get("role").and_then(|v| v.as_str()) != Some("tool") {
            continue;
        }
        let content = tool_msg
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if !looks_like_missing_gate_block(content) {
            break;
        }
        let Some(candidate) = first_tool_call_from_message(assistant_msg) else {
            continue;
        };
        if !supports_artifact_action(harness, candidate.name.as_str()) {
            continue;
        }
        if candidate.arguments.trim().is_empty() {
            continue;
        }
        return Some(candidate);
    }
    None
}

fn first_tool_call_from_message(msg: &Value) -> Option<ToolCallData> {
    let tool_calls = msg.get("tool_calls")?.as_array()?;
    let tc = tool_calls.first()?;
    let id = tc.get("id").and_then(|v| v.as_str())?.trim();
    let function = tc.get("function")?;
    let name = function.get("name").and_then(|v| v.as_str())?.trim();
    let arguments = function.get("arguments").and_then(|v| v.as_str())?.trim();
    if id.is_empty() || name.is_empty() {
        return None;
    }
    Some(ToolCallData {
        id: id.to_string(),
        name: name.to_string(),
        arguments: arguments.to_string(),
    })
}

fn looks_like_missing_gate_block(content: &str) -> bool {
    content.contains("GOVERNOR BLOCKED")
        && (content.contains("Missing valid <plan>") || content.contains("Missing <think>"))
}

fn supports_artifact_action(harness: TaskHarness, name: &str) -> bool {
    match harness.artifact_mode {
        ArtifactMode::NewFiles => matches!(name, "write_file" | "exec"),
        ArtifactMode::NewRepo => matches!(name, "write_file" | "exec"),
        ArtifactMode::ObserveOnly | ArtifactMode::ExistingFiles => false,
    }
}

fn latest_goal_check_missing(messages: &[Value]) -> Vec<String> {
    for msg in messages.iter().rev() {
        if msg.get("role").and_then(|v| v.as_str()) != Some("user") {
            continue;
        }
        let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
        if !content.starts_with("[goal_check]") {
            continue;
        }
        for line in content.lines() {
            let trimmed = line.trim();
            let Some(rest) = trimmed.strip_prefix("Missing:") else {
                continue;
            };
            return rest
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(ToString::to_string)
                .collect();
        }
    }
    Vec::new()
}

#[derive(Debug, Clone, Default)]
struct RepoScaffoldStatus {
    repo_root: String,
    has_git: bool,
    has_gitignore: bool,
    has_readme: bool,
    required_files: Vec<String>,
    present_files: BTreeSet<String>,
}

fn repo_scaffold_status(
    messages: &[Value],
    test_cmd: &str,
    tool_root: Option<&str>,
) -> Option<RepoScaffoldStatus> {
    let repo_root = repo_root_from_test_cmd(test_cmd)?;
    let required_files = required_repo_files_from_test_cmd(test_cmd, repo_root.as_str());
    let mut pending_by_id: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut status = RepoScaffoldStatus {
        repo_root: repo_root.clone(),
        required_files,
        ..RepoScaffoldStatus::default()
    };

    for msg in messages {
        match msg.get("role").and_then(|v| v.as_str()).unwrap_or("") {
            "assistant" => {
                let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) else {
                    continue;
                };
                for tc in tool_calls {
                    let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("").trim();
                    if id.is_empty() {
                        continue;
                    }
                    let Some(function) = tc.get("function") else {
                        continue;
                    };
                    let name = function
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    let arguments = function
                        .get("arguments")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    match name {
                        "write_file" => {
                            let path =
                                serde_json::from_str::<Value>(arguments)
                                    .ok()
                                    .and_then(|value| {
                                        value
                                            .get("path")
                                            .and_then(|v| v.as_str())
                                            .map(str::to_string)
                                    });
                            if let Some(path) = path {
                                pending_by_id.insert(id.to_string(), (name.to_string(), path));
                            }
                        }
                        "exec" => {
                            let command =
                                serde_json::from_str::<Value>(arguments)
                                    .ok()
                                    .and_then(|value| {
                                        value
                                            .get("command")
                                            .and_then(|v| v.as_str())
                                            .map(str::to_string)
                                    });
                            if let Some(command) = command {
                                pending_by_id.insert(id.to_string(), (name.to_string(), command));
                            }
                        }
                        _ => {}
                    }
                }
            }
            "tool" => {
                let id = msg
                    .get("tool_call_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                if id.is_empty() {
                    continue;
                }
                let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let Some((kind, payload)) = pending_by_id.get(id) else {
                    continue;
                };
                match kind.as_str() {
                    "write_file" if content.trim_start().starts_with("OK: wrote '") => {
                        if payload.starts_with(&format!("{repo_root}/")) {
                            status.present_files.insert(payload.clone());
                        }
                        if payload.ends_with("/README.md") {
                            status.has_readme = true;
                        }
                        if payload.ends_with("/.gitignore") {
                            status.has_gitignore = true;
                        }
                    }
                    "exec" if content.trim_start().starts_with("OK (exit_code: 0)") => {
                        if payload.trim() == format!("git init {}", repo_root) {
                            status.has_git = true;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    if let Some(root) = tool_root {
        let repo_path = resolve_repo_scaffold_path(root, repo_root.as_str());
        status.has_git |= repo_path.join(".git").exists();
        status.has_gitignore |= repo_path.join(".gitignore").exists();
        status.has_readme |= repo_path.join("README.md").exists();
        for required_path in status.required_files.clone() {
            if resolve_repo_file_path(root, required_path.as_str()).exists() {
                if required_path.ends_with("/README.md") {
                    status.has_readme = true;
                }
                if required_path.ends_with("/.gitignore") {
                    status.has_gitignore = true;
                }
                status.present_files.insert(required_path);
            }
        }
    }

    Some(status)
}

fn write_file_looks_malformed(tc: &ToolCallData) -> bool {
    let parsed = serde_json::from_str::<Value>(tc.arguments.as_str()).ok();
    let path = parsed
        .as_ref()
        .and_then(|value| value.get("path").and_then(|v| v.as_str()))
        .map(str::trim)
        .unwrap_or("");
    let content = parsed
        .as_ref()
        .and_then(|value| value.get("content").and_then(|v| v.as_str()))
        .map(str::trim)
        .unwrap_or("");
    parsed.is_none()
        || path.is_empty()
        || content.is_empty()
        || path == ".git"
        || path.ends_with("/.git")
}

fn has_repo_scaffold_progress(status: &RepoScaffoldStatus) -> bool {
    status.has_git || status.has_gitignore || status.has_readme || !status.present_files.is_empty()
}

fn next_repo_scaffold_tool_call(
    tool_call_id: &str,
    status: &RepoScaffoldStatus,
) -> Option<ToolCallData> {
    if !status.has_git {
        return Some(ToolCallData {
            id: tool_call_id.to_string(),
            name: "exec".to_string(),
            arguments: serde_json::json!({
                "command": format!("git init {}", status.repo_root)
            })
            .to_string(),
        });
    }
    if let Some(target) = preferred_missing_repo_required_file(status) {
        let content = if target.ends_with("/.gitignore") {
            default_repo_gitignore().to_string()
        } else {
            scaffold_repo_file_content(target)
        };
        return Some(ToolCallData {
            id: tool_call_id.to_string(),
            name: "write_file".to_string(),
            arguments: serde_json::json!({
                "path": target,
                "content": content,
            })
            .to_string(),
        });
    }
    if !status.has_gitignore {
        return Some(ToolCallData {
            id: tool_call_id.to_string(),
            name: "write_file".to_string(),
            arguments: serde_json::json!({
                "path": format!("{}/.gitignore", status.repo_root),
                "content": default_repo_gitignore(),
            })
            .to_string(),
        });
    }
    if !status.has_readme {
        let readme_path = format!("{}/README.md", status.repo_root);
        let readme_content = scaffold_repo_file_content(readme_path.as_str());
        return Some(ToolCallData {
            id: tool_call_id.to_string(),
            name: "write_file".to_string(),
            arguments: serde_json::json!({
                "path": readme_path,
                "content": readme_content,
            })
            .to_string(),
        });
    }
    None
}

fn preferred_missing_repo_required_file(status: &RepoScaffoldStatus) -> Option<&str> {
    if let Some(gitignore) = status
        .required_files
        .iter()
        .find(|path| path.ends_with("/.gitignore") && !status.present_files.contains(*path))
    {
        return Some(gitignore.as_str());
    }

    status
        .required_files
        .iter()
        .find(|path| !status.present_files.contains(*path))
        .map(String::as_str)
}

fn text_contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn prompt_mentions_path_literal(text: &str) -> bool {
    text.split_whitespace().any(|token| {
        let trimmed = token.trim_matches(|c: char| {
            c.is_ascii_punctuation()
                || matches!(
                    c,
                    '「' | '」' | '『' | '』' | '（' | '）' | '`' | '"' | '\''
                )
        });
        let has_path_sep = trimmed.contains('/') || trimmed.contains('\\');
        let has_extension = trimmed
            .split('/')
            .next_back()
            .is_some_and(|segment| segment.contains('.'));
        has_path_sep || has_extension
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn infer_task_harness_detects_fix_lane() {
        let harness =
            TaskHarness::infer("Fix the failing test with the smallest code change.", false);
        assert_eq!(harness.lane, TaskLane::FixExisting);
        assert_eq!(harness.artifact_mode, ArtifactMode::ExistingFiles);
    }

    #[test]
    fn infer_task_harness_detects_new_file_lane() {
        let harness = TaskHarness::infer(
            "Create `notes/todo.txt` containing exactly `ship it`.",
            false,
        );
        assert_eq!(harness.lane, TaskLane::CreateFile);
        assert_eq!(harness.artifact_mode, ArtifactMode::NewFiles);
    }

    #[test]
    fn infer_task_harness_detects_init_repo_lane() {
        let harness = TaskHarness::infer(
            "Create a new git repo in `demo_repo/` and initialize it with README.md.",
            false,
        );
        assert_eq!(harness.lane, TaskLane::InitRepo);
        assert_eq!(harness.artifact_mode, ArtifactMode::NewRepo);
    }

    #[test]
    fn progress_gate_blocks_repeated_fix_read() {
        let messages = vec![
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_read",
                    "type": "function",
                    "function": {"name":"read_file","arguments":"{\"path\":\"src/lib.rs\"}"}
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_read",
                "content": "[src/lib.rs] (13 lines, 228 bytes)\npub fn greet(name: &str) -> String {"
            }),
        ];
        let tc = ToolCallData {
            id: "call_repeat".to_string(),
            name: "read_file".to_string(),
            arguments: json!({"path":"src/lib.rs"}).to_string(),
        };

        let block = build_progress_gate_block(
            TaskHarness {
                lane: TaskLane::FixExisting,
                artifact_mode: ArtifactMode::ExistingFiles,
            },
            &tc,
            &messages,
            Some(RecoveryStage::Fix),
            Some("cargo test 2>&1"),
        )
        .expect("progress gate block");

        assert!(block.contains("[Progress Gate]"));
        assert!(block.contains("patch_file"));
        assert!(block.contains("cargo test 2>&1"));
    }

    #[test]
    fn progress_gate_blocks_third_observation_for_create_file_lane() {
        let messages = vec![
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_list",
                    "type": "function",
                    "function": {"name":"list_dir","arguments":"{\"dir\":\".\",\"include_hidden\":true}"}
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_list",
                "content": "[list_dir: '.' ・ 2 item(s)]\nREADME.md\n.obstral.md"
            }),
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_read",
                    "type": "function",
                    "function": {"name":"read_file","arguments":"{\"path\":\"README.md\"}"}
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_read",
                "content": "[README.md] (2 lines, 20 bytes)\nfixture"
            }),
        ];
        let tc = ToolCallData {
            id: "call_search".to_string(),
            name: "search_files".to_string(),
            arguments: json!({"pattern":"todo","dir":"."}).to_string(),
        };

        let block = build_progress_gate_block(
            TaskHarness {
                lane: TaskLane::CreateFile,
                artifact_mode: ArtifactMode::NewFiles,
            },
            &tc,
            &messages,
            Some(RecoveryStage::Fix),
            Some("test -f notes/todo.txt && grep -Fx \"ship it\" notes/todo.txt"),
        )
        .expect("progress gate block");

        assert!(block.contains("create the requested file now"));
        assert!(block.contains("notes/todo.txt") || block.contains("Configured"));
    }

    #[test]
    fn build_fix_stage_progress_hint_prefers_creation_after_first_success() {
        let messages = vec![
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_list",
                    "type": "function",
                    "function": {"name":"list_dir","arguments":"{\"dir\":\".\",\"include_hidden\":true}"}
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_list",
                "content": "[list_dir: '.' ・ 2 item(s)]\nREADME.md\n.obstral.md"
            }),
        ];

        let hint = build_fix_stage_progress_hint(
            TaskHarness {
                lane: TaskLane::CreateFile,
                artifact_mode: ArtifactMode::NewFiles,
            },
            &messages,
            Some(RecoveryStage::Fix),
            Some("test -f notes/todo.txt && grep -Fx \"ship it\" notes/todo.txt"),
        )
        .expect("fix-stage hint");

        assert!(hint.contains("create the requested file now"));
        assert!(hint.contains("notes/todo.txt"));
    }

    #[test]
    fn coerce_artifact_creation_tool_call_restores_blocked_write_file() {
        let messages = vec![
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_write",
                    "type": "function",
                    "function": {
                        "name":"write_file",
                        "arguments":"{\"path\":\"notes/todo.txt\",\"content\":\"ship it\\n\"}"
                    }
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_write",
                "content": "GOVERNOR BLOCKED\n\n[Plan Gate] Missing valid <plan>.\n\ntool:\nwrite_file\narguments:\n{\"path\":\"notes/todo.txt\",\"content\":\"ship it\\n\"}"
            }),
        ];
        let tc = ToolCallData {
            id: "call_list".to_string(),
            name: "list_dir".to_string(),
            arguments: json!({"dir":".","include_hidden":true}).to_string(),
        };

        let (rewritten, original, coerced) = coerce_artifact_creation_tool_call(
            TaskHarness {
                lane: TaskLane::CreateFile,
                artifact_mode: ArtifactMode::NewFiles,
            },
            &messages,
            &tc,
        )
        .expect("rewritten");

        assert_eq!(original, "list_dir(dir=., include_hidden=true)");
        assert_eq!(rewritten.name, "write_file");
        assert!(rewritten.arguments.contains("notes/todo.txt"));
        assert!(coerced.contains("write_file"));
    }

    #[test]
    fn allows_artifact_creation_during_diagnose_for_new_file_write() {
        let tc = ToolCallData {
            id: "call_write".to_string(),
            name: "write_file".to_string(),
            arguments: json!({"path":"notes/todo.txt","content":"ship it\n"}).to_string(),
        };
        assert!(allows_artifact_creation_during_diagnose(
            TaskHarness {
                lane: TaskLane::CreateFile,
                artifact_mode: ArtifactMode::NewFiles,
            },
            &tc,
        ));
    }

    #[test]
    fn allows_artifact_creation_during_verify_for_new_repo_write() {
        let tc = ToolCallData {
            id: "call_write".to_string(),
            name: "write_file".to_string(),
            arguments: json!({"path":"demo_repo/.gitignore","content":"target/\n"}).to_string(),
        };
        assert!(allows_artifact_creation_during_verify(
            TaskHarness {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            },
            &tc,
        ));
    }

    #[test]
    fn coerce_repo_goal_completion_tool_call_rewrites_to_git_init() {
        let messages = vec![json!({
            "role": "user",
            "content": "[goal_check]\nThe task is NOT complete yet.\nMissing: .git\nFix it by using exec/write_file. Do NOT stop until the goals are satisfied."
        })];
        let tc = ToolCallData {
            id: "call_list".to_string(),
            name: "list_dir".to_string(),
            arguments: json!({"dir":"demo_repo","include_hidden":true}).to_string(),
        };

        let (rewritten, original, coerced) = coerce_repo_goal_completion_tool_call(
            TaskHarness {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            },
            &messages,
            &tc,
            Some("test -d demo_repo/.git && test -f demo_repo/README.md && test -f demo_repo/.gitignore"),
            None,
        )
        .expect("rewritten");

        assert_eq!(rewritten.name, "exec");
        assert!(rewritten.arguments.contains("git init demo_repo"));
        assert!(original.contains("list_dir"));
        assert_eq!(coerced, "git init demo_repo");
    }

    #[test]
    fn coerce_repo_goal_completion_tool_call_advances_to_gitignore_after_git_init() {
        let messages = vec![
            json!({
                "role": "user",
                "content": "[goal_check]\nThe task is NOT complete yet.\nMissing: .git\nFix it by using exec/write_file. Do NOT stop until the goals are satisfied."
            }),
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_init",
                    "type": "function",
                    "function": {
                        "name":"exec",
                        "arguments":"{\"command\":\"git init demo_repo\"}"
                    }
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_init",
                "content": "OK (exit_code: 0)\nInitialized empty Git repository in demo_repo/.git/"
            }),
        ];
        let tc = ToolCallData {
            id: "call_list".to_string(),
            name: "list_dir".to_string(),
            arguments: json!({"dir":"demo_repo","include_hidden":true}).to_string(),
        };

        let (rewritten, _original, coerced) = coerce_repo_goal_completion_tool_call(
            TaskHarness {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            },
            &messages,
            &tc,
            Some("test -d demo_repo/.git && test -f demo_repo/README.md && test -f demo_repo/.gitignore"),
            None,
        )
        .expect("rewritten");

        assert_eq!(rewritten.name, "write_file");
        assert!(rewritten.arguments.contains("demo_repo/.gitignore"));
        assert!(coerced.contains(".gitignore"));
    }

    #[test]
    fn coerce_repo_goal_completion_tool_call_prefers_filesystem_progress_over_stale_goal_check() {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("maze_game");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::write(repo.join(".gitignore"), "target/\n").unwrap();
        std::fs::write(repo.join("README.md"), "# Maze Game\n").unwrap();

        let messages = vec![json!({
            "role": "user",
            "content": "[goal_check]\nThe task is NOT complete yet.\nMissing: .git\nFix it by using exec/write_file. Do NOT stop until the goals are satisfied."
        })];
        let tc = ToolCallData {
            id: "call_list".to_string(),
            name: "list_dir".to_string(),
            arguments: json!({"dir":"maze_game","include_hidden":true}).to_string(),
        };

        let rewritten = coerce_repo_goal_completion_tool_call(
            TaskHarness {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            },
            &messages,
            &tc,
            Some(
                "test -d maze_game/.git && test -f maze_game/README.md && test -f maze_game/.gitignore",
            ),
            Some(dir.path().to_str().unwrap()),
        );

        assert!(rewritten.is_none());
    }

    #[test]
    fn coerce_repo_goal_completion_tool_call_advances_to_missing_required_file_after_binary_crate()
    {
        let dir = tempdir().unwrap();
        let repo = dir.path().join("maze_game");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join(".gitignore"), "target/\n").unwrap();
        std::fs::write(repo.join("Cargo.toml"), "[package]\nname = \"maze_game\"\n").unwrap();
        std::fs::write(repo.join("README.md"), "# Maze Game\n").unwrap();
        std::fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();

        let messages = vec![json!({
            "role": "user",
            "content": "[goal_check]\nThe task is NOT complete yet.\nMissing: .git\nFix it by using exec/write_file. Do NOT stop until the goals are satisfied."
        })];
        let tc = ToolCallData {
            id: "call_list".to_string(),
            name: "list_dir".to_string(),
            arguments: json!({"dir":"maze_game","include_hidden":true}).to_string(),
        };

        let (rewritten, _original, coerced) = coerce_repo_goal_completion_tool_call(
            TaskHarness {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            },
            &messages,
            &tc,
            Some(
                "test -d maze_game/.git && test -f maze_game/README.md && test -f maze_game/Cargo.toml && test -f maze_game/src/lib.rs && test -f maze_game/src/main.rs && cd maze_game && cargo test 2>&1",
            ),
            Some(dir.path().to_str().unwrap()),
        )
        .expect("rewritten");

        assert_eq!(rewritten.name, "write_file");
        assert!(rewritten.arguments.contains("maze_game/src/lib.rs"));
        assert!(coerced.contains("maze_game/src/lib.rs"));
    }

    #[test]
    fn repair_repo_scaffold_write_tool_call_rewrites_truncated_git_path_to_init() {
        let messages = vec![
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_write_readme",
                    "type": "function",
                    "function": {
                        "name":"write_file",
                        "arguments":"{\"path\":\"demo_repo/README.md\",\"content\":\"# demo_repo\\n\"}"
                    }
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_write_readme",
                "content": "OK: wrote 'demo_repo/README.md' (1 lines, 12 bytes)"
            }),
        ];
        let tc = ToolCallData {
            id: "call_bad".to_string(),
            name: "write_file".to_string(),
            arguments: "{\"path\":\"demo_repo/.git".to_string(),
        };

        let (rewritten, _original, coerced) = repair_repo_scaffold_write_tool_call(
            TaskHarness {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            },
            &messages,
            &tc,
            Some("test -d demo_repo/.git && test -f demo_repo/README.md && test -f demo_repo/.gitignore"),
            None,
        )
        .expect("rewritten");

        assert_eq!(rewritten.name, "exec");
        assert!(rewritten.arguments.contains("git init demo_repo"));
        assert_eq!(coerced, "git init demo_repo");
    }

    #[test]
    fn repair_repo_scaffold_write_tool_call_rewrites_to_gitignore_after_git_init() {
        let messages = vec![
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_init",
                    "type": "function",
                    "function": {
                        "name":"exec",
                        "arguments":"{\"command\":\"git init demo_repo\"}"
                    }
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_init",
                "content": "OK (exit_code: 0)\nInitialized empty Git repository in demo_repo/.git/"
            }),
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_write_readme",
                    "type": "function",
                    "function": {
                        "name":"write_file",
                        "arguments":"{\"path\":\"demo_repo/README.md\",\"content\":\"# demo_repo\\n\"}"
                    }
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_write_readme",
                "content": "OK: wrote 'demo_repo/README.md' (1 lines, 12 bytes)"
            }),
        ];
        let tc = ToolCallData {
            id: "call_bad".to_string(),
            name: "write_file".to_string(),
            arguments: json!({"path":"demo_repo/.git","content":"oops"}).to_string(),
        };

        let (rewritten, _original, coerced) = repair_repo_scaffold_write_tool_call(
            TaskHarness {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            },
            &messages,
            &tc,
            Some("test -d demo_repo/.git && test -f demo_repo/README.md && test -f demo_repo/.gitignore"),
            None,
        )
        .expect("rewritten");

        assert_eq!(rewritten.name, "write_file");
        assert!(rewritten.arguments.contains("demo_repo/.gitignore"));
        assert!(coerced.contains("write_file"));
        assert!(coerced.contains(".gitignore"));
    }

    #[test]
    fn repair_repo_scaffold_write_tool_call_rewrites_to_readme_after_gitignore() {
        let messages = vec![
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_init",
                    "type": "function",
                    "function": {
                        "name":"exec",
                        "arguments":"{\"command\":\"git init demo_repo\"}"
                    }
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_init",
                "content": "OK (exit_code: 0)\nInitialized empty Git repository in demo_repo/.git/"
            }),
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": "call_write_gitignore",
                    "type": "function",
                    "function": {
                        "name":"write_file",
                        "arguments":"{\"path\":\"demo_repo/.gitignore\",\"content\":\"node_modules/\\n\"}"
                    }
                }]
            }),
            json!({
                "role": "tool",
                "tool_call_id": "call_write_gitignore",
                "content": "OK: wrote 'demo_repo/.gitignore' (1 lines, 14 bytes)"
            }),
        ];
        let tc = ToolCallData {
            id: "call_bad".to_string(),
            name: "write_file".to_string(),
            arguments: "{\"path\":\"demo_repo/.git".to_string(),
        };

        let (rewritten, _original, coerced) = repair_repo_scaffold_write_tool_call(
            TaskHarness {
                lane: TaskLane::ScaffoldRepo,
                artifact_mode: ArtifactMode::NewRepo,
            },
            &messages,
            &tc,
            Some("test -d demo_repo/.git && test -f demo_repo/README.md && test -f demo_repo/.gitignore"),
            None,
        )
        .expect("rewritten");

        assert_eq!(rewritten.name, "write_file");
        assert!(rewritten.arguments.contains("demo_repo/README.md"));
        assert!(rewritten.arguments.contains("# demo_repo\\n"));
        assert!(coerced.contains("README.md"));
    }
}
