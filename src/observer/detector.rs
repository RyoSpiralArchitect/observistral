use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileScanTool {
    SearchFiles,
    Glob,
    ListDir,
}

#[derive(Debug, Clone)]
pub enum Event {
    CommandExecuted {
        command: String,
    },
    CommandTiming {
        command: String,
        duration_ms: u64,
    },
    CommandFailure {
        command: String,
        stderr: String,
    },
    SandboxBreach {
        command: String,
        detail: String,
    },
    DangerousCommand {
        command: String,
        reason: String,
    },
    LargeDiffApplied {
        path: String,
        diff_chars: usize,
        hunks: usize,
    },
    LargeCommandOutput {
        command: String,
        lines_total: Option<usize>,
        truncated_to_chars: Option<usize>,
    },
    FileScan {
        tool: FileScanTool,
        pattern: String,
        dir: String,
    },
    FileWritten {
        path: String,
    },
    LoopDetected {
        reason: String,
    },
}

#[derive(Debug, Clone)]
struct ToolCallMeta {
    name: String,
    args: Option<Value>,
}

fn normalize_command_key(cmd: &str) -> String {
    cmd.trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn dangerous_command_reason(command: &str) -> Option<String> {
    let cmd = command.trim();
    if cmd.is_empty() {
        return None;
    }
    let low = cmd.to_ascii_lowercase();

    let has_pipe_to_shell = low.contains("| sh")
        || low.contains("| bash")
        || low.contains("| zsh")
        || low.contains("| pwsh")
        || low.contains("| powershell")
        || low.contains("| iex")
        || low.contains("| invoke-expression");
    let has_remote_fetch = low.contains("curl ")
        || low.contains("curl\t")
        || low.contains("wget ")
        || low.contains("iwr ")
        || low.contains("invoke-webrequest")
        || low.contains("invoke-restmethod");
    if has_pipe_to_shell && has_remote_fetch {
        return Some("security:rce(remote script piped to shell)".to_string());
    }

    if low.contains("invoke-expression")
        || low.contains(" iex ")
        || low.starts_with("iex ")
        || low.contains("| iex")
    {
        return Some("security:rce(invoke-expression)".to_string());
    }

    // Obvious destructive operations (high severity).
    if low.contains("rm -rf /")
        || low.contains("rm -rf /*")
        || low.contains("rm -rf ~")
        || low.contains("rm -rf $home")
        || low.contains("rm -rf $env:home")
        || low.contains("rm -rf $env:userprofile")
        || low.contains("rm -rf .git")
        || low.contains("del /s /q")
        || low.contains("format ")
        || low.contains("mkfs")
        || low.contains("dd if=")
    {
        return Some("security:destructive(filesystem wipe)".to_string());
    }

    // Risky git operations (reliability / data loss).
    if low.contains("git push") && (low.contains("--force") || low.contains("--force-with-lease")) {
        return Some("reliability:git(force push)".to_string());
    }
    if low.contains("git reset") && low.contains("--hard") {
        return Some("reliability:git(reset --hard)".to_string());
    }
    if low.contains("git clean") && (low.contains("-fdx") || low.contains("-xdf")) {
        return Some("reliability:git(clean -fdx)".to_string());
    }

    // Overly permissive permissions (security).
    if low.contains("chmod 777") {
        return Some("security:permissions(chmod 777)".to_string());
    }

    None
}

fn parse_truncated_marker_line(line: &str) -> Option<(usize, usize)> {
    let low = line.to_ascii_lowercase();
    if !low.contains("truncated") || !low.contains("lines total") {
        return None;
    }
    let mut nums: Vec<usize> = Vec::new();
    let mut cur = String::new();
    for ch in low.chars() {
        if ch.is_ascii_digit() {
            cur.push(ch);
        } else if !cur.is_empty() {
            if let Ok(v) = cur.parse::<usize>() {
                nums.push(v);
                if nums.len() >= 2 {
                    break;
                }
            }
            cur.clear();
        }
    }
    if nums.len() < 2 && !cur.is_empty() {
        if let Ok(v) = cur.parse::<usize>() {
            nums.push(v);
        }
    }
    if nums.len() >= 2 {
        Some((nums[0], nums[1]))
    } else {
        None
    }
}

fn extract_truncated_marker(text: &str) -> Option<(usize, usize)> {
    for line in text.lines() {
        if let Some(info) = parse_truncated_marker_line(line.trim()) {
            return Some(info);
        }
    }
    None
}

fn parse_exit_code(text: &str) -> Option<i32> {
    fn parse_int_after_marker(s: &str, marker: &str) -> Option<i32> {
        let idx = s.find(marker)?;
        let rest = s[idx + marker.len()..].trim_start();
        if rest.is_empty() {
            return None;
        }
        let mut out = String::new();
        for (i, ch) in rest.chars().enumerate() {
            if i == 0 && ch == '-' {
                out.push(ch);
                continue;
            }
            if ch.is_ascii_digit() {
                out.push(ch);
            } else {
                break;
            }
        }
        if out.is_empty() || out == "-" {
            None
        } else {
            out.parse::<i32>().ok()
        }
    }

    parse_int_after_marker(text, "exit_code:")
        .or_else(|| parse_int_after_marker(text, "exit_code="))
        .or_else(|| parse_int_after_marker(text, "exit:"))
        .or_else(|| parse_int_after_marker(text, "exit="))
}

fn parse_duration_ms(text: &str) -> Option<u64> {
    fn parse_u64_after_marker(s: &str, marker: &str) -> Option<u64> {
        let low = s.to_ascii_lowercase();
        let idx = low.find(marker)?;
        let rest = s[idx + marker.len()..].trim_start();
        if rest.is_empty() {
            return None;
        }
        let mut out = String::new();
        for ch in rest.chars() {
            if ch.is_ascii_digit() {
                out.push(ch);
            } else {
                break;
            }
        }
        if out.is_empty() {
            None
        } else {
            out.parse::<u64>().ok()
        }
    }

    parse_u64_after_marker(text, "duration_ms:")
        .or_else(|| parse_u64_after_marker(text, "duration_ms="))
}

fn extract_first_stderr_line(text: &str) -> String {
    for line in text.lines() {
        let t = line.trim_start();
        if t.len() < 7 {
            continue;
        }
        if t[..7].eq_ignore_ascii_case("stderr:") {
            return t[7..].trim().to_string();
        }
    }
    String::new()
}

fn extract_sandbox_breach_detail(text: &str) -> Option<String> {
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.contains("SANDBOX BREACH") {
            let mut out = vec![line.trim().to_string()];
            if let Some(next) = lines.next() {
                let t = next.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
            }
            return Some(out.join("\n"));
        }
        if line
            .to_ascii_lowercase()
            .contains("cwd_after escaped tool_root")
        {
            return Some(line.trim().to_string());
        }
    }
    None
}

fn diff_hunk_count(diff: &str) -> usize {
    diff.lines()
        .filter(|l| l.trim_start().starts_with("@@"))
        .count()
}

fn should_flag_large_diff(diff_chars: usize, hunks: usize) -> bool {
    // Large apply_diff payloads are harder to review and more likely to be wrong.
    diff_chars >= 12_000 || hunks >= 10
}

fn is_tool_error(tool_name: &str, content: &str) -> bool {
    let t = content.trim_start();
    if t.is_empty() {
        return false;
    }
    let low = t.to_ascii_lowercase();
    if low.starts_with("error:") || low.starts_with("error ") || low.starts_with("error\n") {
        return true;
    }
    if low.starts_with("error reading")
        || low.starts_with("error writing")
        || low.starts_with("error patching")
        || low.starts_with("error applying")
        || low.starts_with("error listing")
        || low.starts_with("error searching")
        || low.starts_with("error glob")
    {
        return true;
    }

    // Server file tools return "ERROR: ..." (uppercase).
    if t.starts_with("ERROR") {
        return true;
    }

    if tool_name == "exec" {
        if low.contains("failed (exit_code") {
            return true;
        }
        if let Some(code) = parse_exit_code(t) {
            return code != 0;
        }
    }

    false
}

fn args_string_arg(args: &Value, key: &str) -> Option<String> {
    Some(args.get(key)?.as_str()?.to_string())
}

fn tool_command_from_args(args: Option<&Value>) -> Option<String> {
    let a = args?;
    args_string_arg(a, "command")
        .or_else(|| args_string_arg(a, "cmd"))
        .or_else(|| args_string_arg(a, "script"))
}

fn tool_path_from_args(args: Option<&Value>) -> Option<String> {
    let a = args?;
    args_string_arg(a, "path")
        .or_else(|| args_string_arg(a, "file_path"))
        .or_else(|| args_string_arg(a, "filename"))
}

fn args_search_pattern(args: Option<&Value>) -> Option<String> {
    let a = args?;
    args_string_arg(a, "pattern").or_else(|| args_string_arg(a, "query"))
}

fn args_dir(args: Option<&Value>) -> Option<String> {
    let a = args?;
    args_string_arg(a, "dir").or_else(|| args_string_arg(a, "root"))
}

fn index_tool_calls(messages: &[Value]) -> HashMap<String, ToolCallMeta> {
    let mut out: HashMap<String, ToolCallMeta> = HashMap::new();
    for msg in messages {
        if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") {
            continue;
        }
        let Some(tcs) = msg.get("tool_calls").and_then(|v| v.as_array()) else {
            continue;
        };
        for tc in tcs {
            let Some(id) = tc.get("id").and_then(|x| x.as_str()).map(|s| s.to_string()) else {
                continue;
            };
            let fn_obj = tc.get("function");
            let name = fn_obj
                .and_then(|f| f.get("name"))
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let arg_str = fn_obj
                .and_then(|f| f.get("arguments"))
                .and_then(|x| x.as_str())
                .unwrap_or("");
            let args = serde_json::from_str::<Value>(arg_str).ok();
            out.insert(id, ToolCallMeta { name, args });
        }
    }
    out
}

pub fn detect_events(messages: &[Value]) -> Vec<Event> {
    let tool_calls = index_tool_calls(messages);
    let mut events: Vec<Event> = Vec::new();

    let mut last_fail_key = String::new();
    let mut same_fail_repeats: u32 = 0;
    let mut loop_emitted = false;

    for msg in messages {
        if msg.get("role").and_then(|r| r.as_str()) != Some("tool") {
            continue;
        }

        let tool_call_id = msg
            .get("tool_call_id")
            .and_then(|x| x.as_str())
            .unwrap_or("");
        let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");

        let meta = tool_calls.get(tool_call_id);
        let tool_name = meta.map(|m| m.name.as_str()).unwrap_or("");

        if tool_name == "exec" {
            let command = tool_command_from_args(meta.and_then(|m| m.args.as_ref()))
                .unwrap_or_else(|| "(unknown command)".to_string());
            events.push(Event::CommandExecuted {
                command: command.clone(),
            });

            if let Some(ms) = parse_duration_ms(content) {
                events.push(Event::CommandTiming {
                    command: command.clone(),
                    duration_ms: ms,
                });
            }

            if let Some(reason) = dangerous_command_reason(&command) {
                events.push(Event::DangerousCommand {
                    command: command.clone(),
                    reason,
                });
            }

            if let Some(detail) = extract_sandbox_breach_detail(content) {
                events.push(Event::SandboxBreach {
                    command: command.clone(),
                    detail,
                });
            }

            if let Some((lines_total, truncated_to_chars)) = extract_truncated_marker(content) {
                events.push(Event::LargeCommandOutput {
                    command: command.clone(),
                    lines_total: Some(lines_total),
                    truncated_to_chars: Some(truncated_to_chars),
                });
            } else if content.to_ascii_lowercase().contains("output truncated") {
                events.push(Event::LargeCommandOutput {
                    command: command.clone(),
                    lines_total: None,
                    truncated_to_chars: None,
                });
            }

            if is_tool_error("exec", content) {
                let stderr = {
                    let s = extract_first_stderr_line(content);
                    if s.trim().is_empty() {
                        let first = content.lines().next().unwrap_or("").trim().to_string();
                        if first.is_empty() {
                            "(no stderr)".to_string()
                        } else {
                            first
                        }
                    } else {
                        s
                    }
                };
                events.push(Event::CommandFailure {
                    command: command.clone(),
                    stderr,
                });

                let key = normalize_command_key(&command);
                if !key.is_empty() && key == last_fail_key {
                    same_fail_repeats = same_fail_repeats.saturating_add(1);
                } else {
                    last_fail_key = key;
                    same_fail_repeats = 1;
                    loop_emitted = false;
                }
                if same_fail_repeats >= 3 && !loop_emitted {
                    loop_emitted = true;
                    events.push(Event::LoopDetected {
                        reason: format!(
                            "same failing command repeated {} times",
                            same_fail_repeats
                        ),
                    });
                }
            }

            continue;
        }

        if tool_name == "search_files" {
            let pattern = args_search_pattern(meta.and_then(|m| m.args.as_ref()))
                .unwrap_or_else(|| "".to_string());
            let dir =
                args_dir(meta.and_then(|m| m.args.as_ref())).unwrap_or_else(|| "".to_string());
            if !pattern.trim().is_empty() {
                events.push(Event::FileScan {
                    tool: FileScanTool::SearchFiles,
                    pattern,
                    dir,
                });
            }
        }
        if tool_name == "glob" {
            let pattern = args_search_pattern(meta.and_then(|m| m.args.as_ref()))
                .unwrap_or_else(|| "".to_string());
            let dir =
                args_dir(meta.and_then(|m| m.args.as_ref())).unwrap_or_else(|| "".to_string());
            if !pattern.trim().is_empty() {
                events.push(Event::FileScan {
                    tool: FileScanTool::Glob,
                    pattern,
                    dir,
                });
            }
        }
        if tool_name == "list_dir" {
            let dir =
                args_dir(meta.and_then(|m| m.args.as_ref())).unwrap_or_else(|| "".to_string());
            events.push(Event::FileScan {
                tool: FileScanTool::ListDir,
                pattern: String::new(),
                dir,
            });
        }

        if matches!(tool_name, "write_file" | "patch_file" | "apply_diff") {
            if is_tool_error(tool_name, content) {
                continue;
            }
            let Some(path) = tool_path_from_args(meta.and_then(|m| m.args.as_ref())) else {
                continue;
            };
            if path.trim().is_empty() {
                continue;
            }
            events.push(Event::FileWritten { path });

            if tool_name == "apply_diff" {
                if let Some(args) = meta.and_then(|m| m.args.as_ref()) {
                    let diff = args.get("diff").and_then(|d| d.as_str()).unwrap_or("");
                    let diff_chars = diff.len();
                    let hunks = diff_hunk_count(diff);
                    if should_flag_large_diff(diff_chars, hunks) {
                        events.push(Event::LargeDiffApplied {
                            path: tool_path_from_args(Some(args))
                                .unwrap_or_else(|| "(unknown path)".to_string()),
                            diff_chars,
                            hunks,
                        });
                    }
                }
            }
        }
    }

    events
}

pub fn detect_events_from_transcript(transcript: &str) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    let mut last_fail_key = String::new();
    let mut same_fail_repeats: u32 = 0;
    let mut loop_emitted = false;

    let normalized = transcript.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.lines().collect();
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i].trim_end();

        // File tool markers as shown in the web UI.
        if let Some(rest) = line.strip_prefix("🔍 search_files:") {
            let tail = rest.trim();
            let (pattern, dir) = match tail.rsplit_once("(dir=") {
                Some((a, b)) => (a.trim(), b.trim().trim_end_matches(')').trim()),
                None => (tail, "."),
            };
            if !pattern.is_empty() {
                events.push(Event::FileScan {
                    tool: FileScanTool::SearchFiles,
                    pattern: pattern.to_string(),
                    dir: dir.to_string(),
                });
            }
            i += 1;
            continue;
        }
        if let Some(rest) = line.strip_prefix("❖ glob:") {
            let tail = rest.trim();
            let (pattern, dir) = match tail.rsplit_once("(dir=") {
                Some((a, b)) => (a.trim(), b.trim().trim_end_matches(')').trim()),
                None => (tail, "."),
            };
            if !pattern.is_empty() {
                events.push(Event::FileScan {
                    tool: FileScanTool::Glob,
                    pattern: pattern.to_string(),
                    dir: dir.to_string(),
                });
            }
            i += 1;
            continue;
        }
        if let Some(rest) = line.strip_prefix("📁 list_dir:") {
            let dir = rest.trim();
            events.push(Event::FileScan {
                tool: FileScanTool::ListDir,
                pattern: String::new(),
                dir: if dir.is_empty() {
                    ".".to_string()
                } else {
                    dir.to_string()
                },
            });
            i += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("✎ write_file:") {
            let p = rest.trim();
            if !p.is_empty() {
                events.push(Event::FileWritten {
                    path: p.to_string(),
                });
            }
            i += 1;
            continue;
        }
        if let Some(rest) = line.strip_prefix("✎ patch_file:") {
            let p = rest.trim();
            if !p.is_empty() {
                events.push(Event::FileWritten {
                    path: p.to_string(),
                });
            }
            i += 1;
            continue;
        }
        if let Some(rest) = line.strip_prefix("⟁ apply_diff:") {
            let tail = rest.trim();
            if !tail.is_empty() {
                let (path, meta) = match tail.rsplit_once('(') {
                    Some((a, b)) if b.contains("chars") || b.contains("hunks") => (a.trim(), b),
                    _ => (tail, ""),
                };
                if !path.is_empty() {
                    events.push(Event::FileWritten {
                        path: path.to_string(),
                    });
                }

                let meta_low = meta.to_ascii_lowercase();
                let mut diff_chars: Option<usize> = None;
                let mut hunks: Option<usize> = None;
                for tok in meta_low.split(|c: char| !c.is_ascii_digit()) {
                    if tok.is_empty() {
                        continue;
                    }
                    if diff_chars.is_none() {
                        diff_chars = tok.parse::<usize>().ok();
                        continue;
                    }
                    if hunks.is_none() {
                        hunks = tok.parse::<usize>().ok();
                        break;
                    }
                }
                if let (Some(dc), Some(hk)) = (diff_chars, hunks) {
                    if should_flag_large_diff(dc, hk) {
                        events.push(Event::LargeDiffApplied {
                            path: path.to_string(),
                            diff_chars: dc,
                            hunks: hk,
                        });
                    }
                }
            }
            i += 1;
            continue;
        }

        if !line.starts_with("```") {
            i += 1;
            continue;
        }

        // Fence block.
        let mut fence_lines: Vec<&str> = Vec::new();
        i += 1;
        while i < lines.len() {
            let l = lines[i];
            if l.trim_end() == "```" {
                break;
            }
            fence_lines.push(l);
            i += 1;
        }

        // Skip closing fence if present.
        if i < lines.len() && lines[i].trim_end() == "```" {
            i += 1;
        }

        // Parse command from first prompt line.
        let mut command: Option<String> = None;
        for l in &fence_lines {
            let t = l.trim_start();
            if let Some(rest) = t.strip_prefix("$ ") {
                command = Some(rest.trim().to_string());
                break;
            }
            if let Some(rest) = t.strip_prefix("PS> ") {
                command = Some(rest.trim().to_string());
                break;
            }
        }
        let Some(cmd) = command else {
            continue;
        };
        if cmd.trim().is_empty() {
            continue;
        }

        events.push(Event::CommandExecuted {
            command: cmd.clone(),
        });

        if let Some(reason) = dangerous_command_reason(&cmd) {
            events.push(Event::DangerousCommand {
                command: cmd.clone(),
                reason,
            });
        }

        // Try to parse exit code from the next line ("exit: N") or any fence line.
        let mut exit_code: Option<i32> = None;
        if i < lines.len() {
            if let Some(code) = parse_exit_code(lines[i]) {
                exit_code = Some(code);
            }
        }
        if exit_code.is_none() {
            for l in &fence_lines {
                if let Some(code) = parse_exit_code(l) {
                    exit_code = Some(code);
                    break;
                }
            }
        }

        // Try to parse duration from next lines ("duration_ms: N") or any fence line.
        let mut duration_ms: Option<u64> = None;
        for off in 0..3 {
            if i + off < lines.len() {
                if let Some(ms) = parse_duration_ms(lines[i + off]) {
                    duration_ms = Some(ms);
                    break;
                }
            }
        }
        if duration_ms.is_none() {
            for l in &fence_lines {
                if let Some(ms) = parse_duration_ms(l) {
                    duration_ms = Some(ms);
                    break;
                }
            }
        }
        if let Some(ms) = duration_ms {
            events.push(Event::CommandTiming {
                command: cmd.clone(),
                duration_ms: ms,
            });
        }

        let stderr = fence_lines
            .iter()
            .find_map(|l| {
                let t = l.trim_start();
                if t.len() >= 7 && t[..7].eq_ignore_ascii_case("stderr:") {
                    Some(t[7..].trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let has_breach_marker = fence_lines.iter().any(|l| l.contains("SANDBOX_BREACH:"))
            || fence_lines.iter().any(|l| l.contains("SANDBOX BREACH"));
        let suspicious_marker = fence_lines
            .iter()
            .find(|l| l.contains("SUSPICIOUS_SUCCESS:"))
            .map(|l| l.trim().to_string());

        for l in &fence_lines {
            if let Some((lines_total, truncated_to_chars)) = parse_truncated_marker_line(l.trim()) {
                events.push(Event::LargeCommandOutput {
                    command: cmd.clone(),
                    lines_total: Some(lines_total),
                    truncated_to_chars: Some(truncated_to_chars),
                });
                break;
            }
        }

        let failed = match exit_code {
            Some(code) => code != 0,
            None => false,
        } || has_breach_marker
            || suspicious_marker.is_some();

        if has_breach_marker {
            let detail = fence_lines
                .iter()
                .find(|l| l.contains("SANDBOX_BREACH:") || l.contains("SANDBOX BREACH"))
                .map(|l| l.trim().to_string())
                .unwrap_or_else(|| "SANDBOX_BREACH".to_string());
            events.push(Event::SandboxBreach {
                command: cmd.clone(),
                detail,
            });
        }

        if failed {
            let stderr2 = if let Some(s) = suspicious_marker {
                s
            } else if stderr.trim().is_empty() {
                "(no stderr)".to_string()
            } else {
                stderr
            };
            events.push(Event::CommandFailure {
                command: cmd.clone(),
                stderr: stderr2,
            });

            let key = normalize_command_key(&cmd);
            if !key.is_empty() && key == last_fail_key {
                same_fail_repeats = same_fail_repeats.saturating_add(1);
            } else {
                last_fail_key = key;
                same_fail_repeats = 1;
                loop_emitted = false;
            }
            if same_fail_repeats >= 3 && !loop_emitted {
                loop_emitted = true;
                events.push(Event::LoopDetected {
                    reason: format!("same failing command repeated {} times", same_fail_repeats),
                });
            }
        }
    }

    events
}
