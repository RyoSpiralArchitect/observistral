use super::{canonicalize_tool_call_command, compact_one_line, RecoveryStage};
use crate::streaming::ToolCallData;
use serde_json::Value;
use std::collections::BTreeMap;

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

#[derive(Debug, Default)]
struct ObservationHistory {
    total_successes: usize,
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
}
