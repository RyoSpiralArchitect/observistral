use super::{canonicalize_tool_call_command, compact_one_line};
use crate::streaming::ToolCallData;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FollowupKind {
    Docs,
    TuiReplay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FollowupRequirement {
    file_path: String,
    missing_literal: String,
    kind: FollowupKind,
}

pub(super) fn coerce_existing_followup_tool_call(
    messages: &[Value],
    tc: &ToolCallData,
    root_user_text: &str,
) -> Option<(ToolCallData, String, String)> {
    if !is_followup_candidate_tool(tc) {
        return None;
    }

    let reads = successful_read_contents(messages);
    let mutations = successful_literal_mutations(messages);
    let requirement = next_followup_requirement(messages, root_user_text, &reads, &mutations)?;
    let rewritten = if let Some(body) = reads.get(requirement.file_path.as_str()) {
        synthesize_followup_patch(&requirement, body)?
    } else {
        ToolCallData {
            id: tc.id.clone(),
            name: "read_file".to_string(),
            arguments: json!({ "path": requirement.file_path }).to_string(),
        }
    };
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

pub(super) fn matches_required_existing_followup(
    messages: &[Value],
    tc: &ToolCallData,
    root_user_text: &str,
) -> bool {
    let reads = successful_read_contents(messages);
    let mutations = successful_literal_mutations(messages);
    let Some(requirement) = next_followup_requirement(messages, root_user_text, &reads, &mutations)
    else {
        return false;
    };
    let Some(path) = tool_call_path(tc) else {
        return false;
    };
    if path != requirement.file_path {
        return false;
    }
    match tc.name.as_str() {
        "read_file" => !reads.contains_key(path.as_str()),
        "patch_file" | "apply_diff" | "write_file" => reads.contains_key(path.as_str()),
        _ => false,
    }
}

fn next_followup_requirement(
    messages: &[Value],
    root_user_text: &str,
    reads: &BTreeMap<String, String>,
    mutations: &BTreeMap<String, String>,
) -> Option<FollowupRequirement> {
    let docs_path = docs_followup_path(root_user_text);
    let tui_replay_path = tui_replay_followup_path(root_user_text);
    let mut pending = Vec::new();

    for pending_requirement in
        missing_followup_requirements(messages, docs_path.clone(), tui_replay_path.clone())
    {
        pending.push(pending_requirement);
    }

    if let Some(missing_literal) = target_src_literal(root_user_text) {
        if let Some(file_path) = docs_path.as_ref() {
            if reads.contains_key(file_path.as_str()) {
                pending.push(FollowupRequirement {
                    file_path: file_path.clone(),
                    missing_literal: missing_literal.clone(),
                    kind: FollowupKind::Docs,
                });
            }
        }
        if let Some(file_path) = tui_replay_path.as_ref() {
            if reads.contains_key(file_path.as_str()) {
                pending.push(FollowupRequirement {
                    file_path: file_path.clone(),
                    missing_literal,
                    kind: FollowupKind::TuiReplay,
                });
            }
        }
    }

    pending.reverse();
    pending
        .into_iter()
        .find(|requirement| !followup_requirement_is_satisfied(requirement, reads, mutations))
}

fn synthesize_followup_patch(
    requirement: &FollowupRequirement,
    body: &str,
) -> Option<ToolCallData> {
    if body.contains(requirement.missing_literal.as_str()) {
        return None;
    }
    let replace = match requirement.kind {
        FollowupKind::Docs => {
            transform_markdown_followup(body, requirement.missing_literal.as_str())
        }
        FollowupKind::TuiReplay => {
            transform_tui_replay_followup(body, requirement.missing_literal.as_str())
        }
    }?;
    Some(ToolCallData {
        id: "synthetic_followup_patch".to_string(),
        name: "patch_file".to_string(),
        arguments: json!({
            "path": requirement.file_path,
            "search": body,
            "replace": replace,
        })
        .to_string(),
    })
}

fn transform_markdown_followup(body: &str, missing_literal: &str) -> Option<String> {
    let mut lines = body.lines().map(str::to_string).collect::<Vec<_>>();
    let bullet = format!("- `{missing_literal}`");
    let insert_at = lines
        .iter()
        .enumerate()
        .rev()
        .find_map(|(idx, line)| {
            let trimmed = line.trim();
            (trimmed.starts_with("- `") && trimmed.ends_with('`')).then_some(idx + 1)
        })
        .unwrap_or(lines.len());
    lines.insert(insert_at, bullet);
    let mut out = lines.join("\n");
    if body.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

fn transform_tui_replay_followup(body: &str, missing_literal: &str) -> Option<String> {
    let mut value = serde_json::from_str::<Value>(body).ok()?;
    let paths = value.get_mut("paths")?.as_array_mut()?;
    if paths
        .iter()
        .any(|entry| entry.as_str() == Some(missing_literal))
    {
        return None;
    }
    paths.push(Value::String(missing_literal.to_string()));
    let mut out = serde_json::to_string_pretty(&value).ok()?;
    if body.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

fn docs_followup_path(root_user_text: &str) -> Option<String> {
    path_literals_in_text(root_user_text)
        .into_iter()
        .find(|path| path.starts_with("docs/") && path.ends_with(".md"))
}

fn tui_replay_followup_path(root_user_text: &str) -> Option<String> {
    path_literals_in_text(root_user_text)
        .into_iter()
        .find(|path| {
            path.ends_with(".obstral/tui_replay.json") || path.ends_with("tui_replay.json")
        })
}

fn target_src_literal(root_user_text: &str) -> Option<String> {
    path_literals_in_text(root_user_text)
        .into_iter()
        .find(|path| path.starts_with("src/") && path.ends_with(".rs"))
}

fn tool_call_path(tc: &ToolCallData) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(tc.arguments.as_str()).ok()?;
    parsed
        .get("path")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn is_followup_candidate_tool(tc: &ToolCallData) -> bool {
    matches!(
        tc.name.as_str(),
        "read_file" | "search_files" | "list_dir" | "glob" | "exec" | "patch_file" | "apply_diff"
    )
}

fn missing_followup_requirements(
    messages: &[Value],
    docs_path: Option<String>,
    tui_replay_path: Option<String>,
) -> Vec<FollowupRequirement> {
    let mut pending = Vec::new();
    for msg in messages.iter().rev() {
        if msg.get("role").and_then(|v| v.as_str()) != Some("tool") {
            continue;
        }
        let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(missing_literal) = trimmed.strip_prefix("missing docs follow-up for ") {
                if let Some(file_path) = docs_path.as_ref() {
                    pending.push(FollowupRequirement {
                        file_path: file_path.clone(),
                        missing_literal: missing_literal.trim().to_string(),
                        kind: FollowupKind::Docs,
                    });
                }
            } else if let Some(missing_literal) =
                trimmed.strip_prefix("missing tui replay follow-up for ")
            {
                if let Some(file_path) = tui_replay_path.as_ref() {
                    pending.push(FollowupRequirement {
                        file_path: file_path.clone(),
                        missing_literal: missing_literal.trim().to_string(),
                        kind: FollowupKind::TuiReplay,
                    });
                }
            }
        }
    }
    pending
}

fn followup_requirement_is_satisfied(
    requirement: &FollowupRequirement,
    reads: &BTreeMap<String, String>,
    mutations: &BTreeMap<String, String>,
) -> bool {
    reads
        .get(requirement.file_path.as_str())
        .is_some_and(|body| body.contains(requirement.missing_literal.as_str()))
        || mutations
            .get(requirement.file_path.as_str())
            .is_some_and(|body| body.contains(requirement.missing_literal.as_str()))
}

fn successful_read_contents(messages: &[Value]) -> BTreeMap<String, String> {
    let mut pending: BTreeMap<String, String> = BTreeMap::new();
    let mut out = BTreeMap::new();

    for msg in messages {
        match msg.get("role").and_then(|v| v.as_str()).unwrap_or("") {
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
                    if id.is_empty() || name != "read_file" {
                        continue;
                    }
                    let path = tc
                        .get("function")
                        .and_then(|v| v.get("arguments"))
                        .and_then(|v| v.as_str())
                        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
                        .and_then(|value| {
                            value
                                .get("path")
                                .and_then(|v| v.as_str())
                                .map(str::to_string)
                        })
                        .unwrap_or_default();
                    if !path.trim().is_empty() {
                        pending.insert(id.to_string(), path);
                    }
                }
            }
            "tool" => {
                let tool_call_id = msg
                    .get("tool_call_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                let Some(path) = pending.remove(tool_call_id) else {
                    continue;
                };
                let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
                if content.starts_with("ERROR ") || content.starts_with("GOVERNOR BLOCKED") {
                    continue;
                }
                let body = content
                    .split_once('\n')
                    .map(|(_, rest)| rest.to_string())
                    .unwrap_or_default();
                out.insert(path, body);
            }
            _ => {}
        }
    }

    out
}

fn successful_literal_mutations(messages: &[Value]) -> BTreeMap<String, String> {
    let mut pending: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut out = BTreeMap::new();

    for msg in messages {
        match msg.get("role").and_then(|v| v.as_str()).unwrap_or("") {
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
                    if id.is_empty() || !matches!(name, "patch_file" | "write_file") {
                        continue;
                    }
                    let Some(args) = tc
                        .get("function")
                        .and_then(|v| v.get("arguments"))
                        .and_then(|v| v.as_str())
                        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
                    else {
                        continue;
                    };
                    let Some(path) = args.get("path").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    let payload = match name {
                        "patch_file" => args.get("replace").and_then(|v| v.as_str()),
                        "write_file" => args.get("content").and_then(|v| v.as_str()),
                        _ => None,
                    };
                    if let Some(payload) = payload {
                        pending.insert(id.to_string(), (path.to_string(), payload.to_string()));
                    }
                }
            }
            "tool" => {
                let tool_call_id = msg
                    .get("tool_call_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                let Some((path, payload)) = pending.remove(tool_call_id) else {
                    continue;
                };
                let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
                if content.starts_with("OK:") {
                    out.insert(path, payload);
                }
            }
            _ => {}
        }
    }

    out
}

fn path_literals_in_text(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for token in text.split_whitespace() {
        let trimmed = token
            .trim_matches(|c: char| {
                (c.is_ascii_punctuation() && !matches!(c, '.' | '/' | '\\' | '_' | '-'))
                    || matches!(
                        c,
                        '「' | '」'
                            | '『'
                            | '』'
                            | '（'
                            | '）'
                            | '('
                            | ')'
                            | '['
                            | ']'
                            | '{'
                            | '}'
                            | '`'
                            | '"'
                            | '\''
                    )
            })
            .trim_end_matches(|c: char| matches!(c, '.' | ',' | ';' | ':' | '!' | '?'))
            .trim_matches(|c: char| {
                matches!(
                    c,
                    '「' | '」'
                        | '『'
                        | '』'
                        | '（'
                        | '）'
                        | '('
                        | ')'
                        | '['
                        | ']'
                        | '{'
                        | '}'
                        | '`'
                        | '"'
                        | '\''
                )
            });
        let has_path_sep = trimmed.contains('/') || trimmed.contains('\\');
        let has_extension = trimmed
            .split('/')
            .next_back()
            .is_some_and(|segment| segment.contains('.'));
        if (has_path_sep || has_extension) && !trimmed.is_empty() {
            let literal = trimmed.replace('\\', "/");
            if !out.contains(&literal) {
                out.push(literal);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{coerce_existing_followup_tool_call, matches_required_existing_followup};
    use crate::streaming::ToolCallData;
    use serde_json::json;

    #[test]
    fn coerces_missing_docs_followup_to_read_docs_first() {
        let messages = vec![json!({
            "role": "tool",
            "content": "[auto-test] ✗ FAILED (exit 1)\nmissing docs follow-up for src/tui/review_panel.rs"
        })];
        let tc = ToolCallData {
            id: "call_search".to_string(),
            name: "search_files".to_string(),
            arguments: json!({"pattern":"review_panel"}).to_string(),
        };

        let (rewritten, original, coerced) = coerce_existing_followup_tool_call(
            &messages,
            &tc,
            "Update docs/runtime-architecture.md and .obstral/tui_replay.json too.",
        )
        .expect("docs followup coercion");

        assert_eq!(original, "search_files(pattern=review_panel)");
        assert_eq!(rewritten.name, "read_file");
        assert_eq!(coerced, "read_file(path=docs/runtime-architecture.md)");
    }

    #[test]
    fn coerces_missing_replay_followup_to_patch_after_read() {
        let messages = vec![
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_read_replay",
                    "type":"function",
                    "function":{"name":"read_file","arguments":"{\"path\":\".obstral/tui_replay.json\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_read_replay",
                "content":"[.obstral/tui_replay.json] (10 lines, 120 bytes)\n{\n  \"version\": 1,\n  \"paths\": [\n    \"src/tui/events.rs\"\n  ]\n}\n"
            }),
            json!({
                "role":"tool",
                "content":"[auto-test] ✗ FAILED (exit 1)\nmissing tui replay follow-up for src/tui/review_panel.rs"
            }),
        ];
        let tc = ToolCallData {
            id: "call_exec".to_string(),
            name: "exec".to_string(),
            arguments: json!({"command":"cargo test 2>&1"}).to_string(),
        };

        let (rewritten, _original, coerced) = coerce_existing_followup_tool_call(
            &messages,
            &tc,
            "Update .obstral/tui_replay.json too.",
        )
        .expect("replay followup coercion");

        assert_eq!(rewritten.name, "patch_file");
        assert!(rewritten
            .arguments
            .contains("\"path\":\".obstral/tui_replay.json\""));
        assert!(rewritten.arguments.contains("src/tui/review_panel.rs"));
        assert!(coerced.starts_with("patch_file("));
    }

    #[test]
    fn coerces_prompt_required_docs_followup_to_patch_after_docs_read() {
        let messages = vec![
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_read_docs",
                    "type":"function",
                    "function":{"name":"read_file","arguments":"{\"path\":\"docs/runtime-architecture.md\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_read_docs",
                "content":"[docs/runtime-architecture.md] (8 lines, 120 bytes)\n# Runtime architecture\n\n- `src/tui/events.rs`\n- `src/tui/app.rs`\n"
            }),
        ];
        let tc = ToolCallData {
            id: "call_read_again".to_string(),
            name: "read_file".to_string(),
            arguments: json!({"path":"docs/runtime-architecture.md"}).to_string(),
        };

        let (rewritten, _original, coerced) = coerce_existing_followup_tool_call(
            &messages,
            &tc,
            "Update docs/runtime-architecture.md and .obstral/tui_replay.json to include `src/tui/review_panel.rs`.",
        )
        .expect("prompt-driven docs followup");

        assert_eq!(rewritten.name, "patch_file");
        assert!(rewritten
            .arguments
            .contains("\"path\":\"docs/runtime-architecture.md\""));
        assert!(rewritten.arguments.contains("src/tui/review_panel.rs"));
        assert!(coerced.starts_with("patch_file("));
    }

    #[test]
    fn redirects_repeated_fix_patch_to_required_docs_followup() {
        let messages = vec![
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_patch_rules",
                    "type":"function",
                    "function":{"name":"patch_file","arguments":"{\"path\":\"src/observer/repo_rules.rs\",\"search\":\"old\",\"replace\":\"new\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_patch_rules",
                "content":"OK: patched 'src/observer/repo_rules.rs' (+1 lines, 3 total)"
            }),
            json!({
                "role":"tool",
                "content":"FAILED (exit_code: 1)\nstderr:\nmissing docs follow-up for src/tui/review_panel.rs"
            }),
        ];
        let tc = ToolCallData {
            id: "call_patch_rules_again".to_string(),
            name: "patch_file".to_string(),
            arguments: json!({"path":"src/observer/repo_rules.rs","search":"old","replace":"new"})
                .to_string(),
        };

        let (rewritten, _original, coerced) = coerce_existing_followup_tool_call(
            &messages,
            &tc,
            "Update docs/runtime-architecture.md and .obstral/tui_replay.json to include `src/tui/review_panel.rs`.",
        )
        .expect("docs followup from repeated patch");

        assert_eq!(rewritten.name, "read_file");
        assert!(rewritten
            .arguments
            .contains("\"path\":\"docs/runtime-architecture.md\""));
        assert_eq!(coerced, "read_file(path=docs/runtime-architecture.md)");
    }

    #[test]
    fn advances_from_docs_patch_to_replay_patch_after_successful_followup() {
        let messages = vec![
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_read_docs",
                    "type":"function",
                    "function":{"name":"read_file","arguments":"{\"path\":\"docs/runtime-architecture.md\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_read_docs",
                "content":"[docs/runtime-architecture.md] (8 lines, 120 bytes)\n# Runtime architecture\n\n- `src/tui/events.rs`\n- `src/tui/app.rs`\n"
            }),
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_patch_docs",
                    "type":"function",
                    "function":{"name":"patch_file","arguments":"{\"path\":\"docs/runtime-architecture.md\",\"search\":\"- `src/tui/app.rs`\\n\",\"replace\":\"- `src/tui/app.rs`\\n- `src/tui/review_panel.rs`\\n\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_patch_docs",
                "content":"OK: patched 'docs/runtime-architecture.md' (+1 lines, 4 total)"
            }),
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_read_replay",
                    "type":"function",
                    "function":{"name":"read_file","arguments":"{\"path\":\".obstral/tui_replay.json\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_read_replay",
                "content":"[.obstral/tui_replay.json] (6 lines, 80 bytes)\n{\n  \"paths\": [\n    \"src/tui/events.rs\"\n  ]\n}\n"
            }),
        ];
        let tc = ToolCallData {
            id: "call_exec".to_string(),
            name: "exec".to_string(),
            arguments: json!({"command":"cargo test 2>&1"}).to_string(),
        };

        let (rewritten, _original, coerced) = coerce_existing_followup_tool_call(
            &messages,
            &tc,
            "Update docs/runtime-architecture.md and .obstral/tui_replay.json to include `src/tui/review_panel.rs`.",
        )
        .expect("replay followup after docs patch");

        assert_eq!(rewritten.name, "patch_file");
        assert!(rewritten
            .arguments
            .contains("\"path\":\".obstral/tui_replay.json\""));
        assert!(rewritten.arguments.contains("src/tui/review_panel.rs"));
        assert!(coerced.starts_with("patch_file("));
    }

    #[test]
    fn allows_pending_replay_followup_patch_during_verify() {
        let messages = vec![
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_read_docs",
                    "type":"function",
                    "function":{"name":"read_file","arguments":"{\"path\":\"docs/runtime-architecture.md\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_read_docs",
                "content":"[docs/runtime-architecture.md] (8 lines, 120 bytes)\n# Runtime architecture\n\n- `src/tui/events.rs`\n- `src/tui/app.rs`\n"
            }),
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_patch_docs",
                    "type":"function",
                    "function":{"name":"patch_file","arguments":"{\"path\":\"docs/runtime-architecture.md\",\"search\":\"- `src/tui/app.rs`\\n\",\"replace\":\"- `src/tui/app.rs`\\n- `src/tui/review_panel.rs`\\n\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_patch_docs",
                "content":"OK: patched 'docs/runtime-architecture.md' (+1 lines, 4 total)"
            }),
            json!({
                "role":"assistant",
                "tool_calls":[{
                    "id":"call_read_replay",
                    "type":"function",
                    "function":{"name":"read_file","arguments":"{\"path\":\".obstral/tui_replay.json\"}"}
                }]
            }),
            json!({
                "role":"tool",
                "tool_call_id":"call_read_replay",
                "content":"[.obstral/tui_replay.json] (6 lines, 80 bytes)\n{\n  \"paths\": [\n    \"src/tui/events.rs\"\n  ]\n}\n"
            }),
        ];
        let tc = ToolCallData {
            id: "call_patch_replay".to_string(),
            name: "patch_file".to_string(),
            arguments: json!({
                "path":".obstral/tui_replay.json",
                "search":"\"src/tui/events.rs\"",
                "replace":"\"src/tui/events.rs\",\n    \"src/tui/review_panel.rs\""
            })
            .to_string(),
        };

        assert!(matches_required_existing_followup(
            &messages,
            &tc,
            "Update docs/runtime-architecture.md and .obstral/tui_replay.json to include `src/tui/review_panel.rs`.",
        ));
    }
}
