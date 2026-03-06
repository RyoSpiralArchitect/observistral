/// Coder agentic loop: calls the model with an `exec` tool, runs commands,
/// and loops until finish_reason == "stop" or max iterations are reached.
///
/// Reasoning improvements applied here:
///   1. Scratchpad protocol  — model outputs <think>goal/risk/doubt/next/verify</think>
///      before every tool call (~50 tokens).  Prevents wrong-direction errors.
///   2. Tool output truncation  — stdout > 1500 chars / stderr > 600 chars are trimmed.
///      This is the single largest token-saver on long runs.
///   3. Context pruning  — tool results older than KEEP_RECENT_TOOL_TURNS are collapsed
///      to their first line.  Keeps the context window from exploding across sessions.
///   4. Error classification  — on exit_code != 0, `classify_error()` identifies the
///      error type (env/syntax/path/dep/network/logic) and injects a targeted recovery
///      hint before the generic diagnosis protocol.
///   5. tool_call_id preserved  — messages stay as serde_json::Value all the way to the
///      provider, so the id field is never silently dropped.
///   6. Progress checkpoints  — at iter 3/6/9 the model self-evaluates goal distance
///      (DONE / REMAINING / ON_TRACK) before continuing.
use anyhow::{anyhow, Result};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::approvals::{ApprovalOutcome, ApprovalRequest, Approver};
use crate::config::RunConfig;
use crate::exec;
use crate::streaming::{stream_openai_compat_json, StreamToken, ToolCallData};
use crate::types::ChatMessage;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AgenticStartState {
    pub messages: Vec<serde_json::Value>,
    pub checkpoint: Option<String>,
    pub cur_cwd: Option<String>,
    pub create_checkpoint: bool,
}

#[derive(Debug, Clone)]
pub struct AgenticEndState {
    pub messages: Vec<serde_json::Value>,
    pub checkpoint: Option<String>,
    pub cur_cwd: Option<String>,
}

async fn autosave_best_effort(
    autosaver: &Option<Arc<crate::agent_session::SessionAutoSaver>>,
    tx: &mpsc::Sender<StreamToken>,
    tool_root_abs: Option<&str>,
    checkpoint: Option<&str>,
    cur_cwd: Option<&str>,
    messages: &[serde_json::Value],
) {
    let Some(ref saver) = autosaver else {
        return;
    };
    let Some(warn) = saver.save_best_effort(tool_root_abs, checkpoint, cur_cwd, messages) else {
        return;
    };
    let _ = tx
        .send(StreamToken::Delta(format!("\n[autosave] WARN: {warn}\n")))
        .await;
}

// ── Tunables ─────────────────────────────────────────────────────────────────

pub const DEFAULT_MAX_ITERS: usize = 12;
const MAX_STDOUT_CHARS: usize = 1500;
const MAX_STDERR_CHARS: usize = 600;
const KEEP_RECENT_TOOL_TURNS: usize = 4;
const TOKEN_BUDGET_WARN_TOKENS: usize = 9000;

/// Marker appended to every exec call so we can persist working directory across tool runs.
///
/// IMPORTANT: Each `exec` runs in a fresh process. Without this, `cd` is lost between calls,
/// which causes nested-git disasters and "why did it run in the repo root?" failures.
const PWD_MARKER: &str = "__OBSTRAL_PWD__=";

// ── System prompt addons ──────────────────────────────────────────────────────

/// Injected for Windows PowerShell environments.
const WIN_SYSTEM_ADDON: &str = "\n\n[Windows execution rules]\n\
- You are on Windows. Use PowerShell syntax.\n\
- NEVER use here-strings (@' or @\"). Write files line-by-line with Set-Content / Add-Content, or use Out-File.\n\
- Use single-line commands or semicolons to chain statements.\n\
- Check $LASTEXITCODE after commands that may fail.\n\
- Prefer relative paths inside the project directory.";

/// Compact reasoning protocol injected into every agentic system prompt.
///
/// Cost: ~50 tokens per iteration in the response.
/// Benefit: avoids wrong-direction mistakes that cost 300-500 tokens to correct.
/// The <plan> block runs once; <think> runs before every tool call.
const SCRATCHPAD_ADDON: &str = "\n\n\
[Planning Protocol — emit ONCE before your very first tool call]\n\
<plan>\n\
goal: <one sentence: what the finished task looks like when done>\n\
steps: 1) ... 2) ... 3) ... (3-7 concrete, ordered steps)\n\
risks: <the 2 most likely failure modes for this specific task>\n\
assumptions: <what you are taking as given>\n\
</plan>\n\
\n\
[Reasoning Protocol — emit before EVERY tool call]\n\
 <think>\n\
 goal:   <≤12 words: what must succeed right now>\n\
 step:   <which plan step number (1-7) this tool call belongs to>\n\
 risk:   <≤12 words: most likely failure mode>\n\
doubt:  <≤12 words: one reason this approach could be wrong>\n\
next:   <≤12 words: exact command or step>\n\
verify: <≤12 words: how to confirm this step succeeded>\n\
</think>\n\
Keep each field under 15 words. This 6-line check (~60 tokens) prevents\n\
wrong-direction errors that cost 300+ tokens to recover from.\n\
\n\
[Error Protocol]\n\
If exit_code ≠ 0: STOP. Quote the exact error. State root cause in one sentence.\n\
Fix with one corrected command. If the SAME approach fails 3 consecutive times:\n\
abandon it, explain why, and propose a completely different strategy.";

// ── Tool definition ───────────────────────────────────────────────────────────

pub fn exec_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "exec",
            "description": "Execute a shell command on the local machine and return stdout/stderr/exit_code.\n\
                            On Windows: PowerShell.  On Linux/macOS: sh.\n\
                            ALWAYS check exit_code — 0 = success, non-zero = failure.\n\
                            If exit_code != 0: STOP, diagnose the error, then fix it.\n\
                            Do NOT proceed to the next step while any command is failing.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute. Single-line preferred."
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Optional working directory (absolute or relative path)."
                    }
                },
                "required": ["command"]
            }
        }
    })
}

pub fn read_file_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Read the full content of a file. Use before editing to see the exact current text. \
                            Path is relative to tool_root (or absolute within tool_root). \
                            Large files are truncated automatically.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to tool_root."
                    }
                },
                "required": ["path"]
            }
        }
    })
}

pub fn write_file_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "write_file",
            "description": "Atomically create or overwrite a file with the given content. \
                            Creates parent directories automatically. \
                            More reliable than exec+echo for file creation (handles encoding, special chars, newlines). \
                            Path is relative to tool_root.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to tool_root."
                    },
                    "content": {
                        "type": "string",
                        "description": "Complete file content to write."
                    }
                },
                "required": ["path", "content"]
            }
        }
    })
}

pub fn patch_file_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "patch_file",
            "description": "Edit a file by replacing an exact text snippet. \
                            The search string MUST appear exactly once in the file. \
                            Call read_file first to see the exact current text if unsure. \
                            For whole-file rewrites use write_file instead. \
                            Path is relative to tool_root.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to tool_root."
                    },
                    "search": {
                        "type": "string",
                        "description": "Exact text to find (must match character-for-character, including whitespace and newlines)."
                    },
                    "replace": {
                        "type": "string",
                        "description": "Text to substitute in place of the search string."
                    }
                },
                "required": ["path", "search", "replace"]
            }
        }
    })
}

pub fn search_files_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "search_files",
            "description": "Search file contents for a literal text pattern (like grep -rn). \
                            Returns matching lines as 'file:line: content'. \
                            PREFER this over exec+grep — it is faster, safer, and token-efficient. \
                            Use to find function definitions, TODO items, error strings, or any \
                            pattern across the codebase. Dir is relative to tool_root.",
            "parameters": {
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Literal text to search for (not a regex)."
                    },
                    "dir": {
                        "type": "string",
                        "description": "Subdirectory to search in (relative to tool_root). \
                                        Omit or leave empty to search all files under tool_root."
                    },
                    "case_insensitive": {
                        "type": "boolean",
                        "description": "If true, search ignores case. Default: false."
                    }
                },
                "required": ["pattern"]
            }
        }
    })
}

pub fn apply_diff_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "apply_diff",
            "description": "Apply a unified diff to a file. More reliable than patch_file for \
                            complex edits that span many lines or have multiple hunks. \
                            Use standard @@ unified diff format. Each hunk is matched by \
                            content (context + remove lines), so exact line numbers are not required. \
                            Multiple hunks per call are supported. \
                            ALWAYS include 2-3 context lines around changes for reliable matching.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to tool_root."
                    },
                    "diff": {
                        "type": "string",
                        "description": "Unified diff string with @@ hunks. Example:\n@@ -10,5 +10,6 @@\n context\n-old line\n+new line\n context"
                    }
                },
                "required": ["path", "diff"]
            }
        }
    })
}

pub fn list_dir_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "list_dir",
            "description": "List a directory (non-recursive). Use this to quickly discover the project structure.\n\
                            Prefer this over glob when you don't know the exact pattern yet.\n\
                            Dir is relative to tool_root. If dir is empty, list tool_root/current directory.",
            "parameters": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Directory to list (relative to tool_root). Empty = tool_root/current directory."
                    },
                    "max_entries": {
                        "type": "integer",
                        "description": "Max entries to return. Default: 200. Max: 500."
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "description": "Include dotfiles/directories. Default: false."
                    }
                }
            }
        }
    })
}

pub fn glob_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "glob",
            "description": "Find files by name/path pattern (like find -name). \
                            Supports * (single component), ** (any depth), ? (single char). \
                            Examples: '**/*.rs', 'src/*.ts', 'test_*'. \
                            Returns relative paths sorted alphabetically. \
                            PREFER this over exec+find/ls — OS-agnostic and token-efficient.",
            "parameters": {
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern, e.g. '**/*.rs' or 'src/*.ts'"
                    },
                    "dir": {
                        "type": "string",
                        "description": "Subdirectory to search in (relative to tool_root). \
                                        Omit to search all of tool_root."
                    }
                },
                "required": ["pattern"]
            }
        }
    })
}

pub fn done_tool_def() -> serde_json::Value {
    json!({
        "type": "function",
        "function": {
            "name": "done",
            "description": "Signal that the task is complete and end the agent loop. \
                            Use only after verifying with commands/tests.",
            "parameters": {
                "type": "object",
                "properties": {
                    "summary": {
                        "type": "string",
                        "description": "Brief [DONE] summary: what was built/changed and where it lives."
                    },
                    "next_steps": {
                        "type": "string",
                        "description": "How to run/verify, or any follow-up work."
                    }
                },
                "required": ["summary"]
            }
        }
    })
}

// ── System prompt builders ────────────────────────────────────────────────────

/// Fixed base for the TUI Coder pane — always an agentic executor, not a chat bot.
const CODER_BASE_SYSTEM: &str = "\
You are an autonomous coding agent with 9 tools:\n\
  exec(command, cwd?)                       — run shell commands (build, test, git, installs)\n\
  read_file(path)                           — read a file's exact content\n\
  write_file(path, content)                 — create or overwrite a file reliably\n\
  patch_file(path, search, replace)         — replace an exact snippet in a file\n\
  apply_diff(path, diff)                    — apply a unified @@ diff (multiple hunks)\n\
  search_files(pattern, dir?, ci?)          — find text across files (like grep -rn)\n\
  list_dir(dir?, max_entries?, include_hidden?) — list a directory (non-recursive)\n\
  glob(pattern, dir?)                       — find files by name pattern (like find -name)\n\
  done(summary, next_steps?)                — finish the task and end the loop\n\
\n\
RULE: You MUST call a tool on every single turn. Never respond with text only.\n\
\n\
PRIORITY (safety-first default when unsure):\n\
  1) read_file                  — before editing any existing file\n\
  2) list_dir/glob/search_files  — discover files (prefer list_dir for quick structure)\n\
  3) patch_file/apply_diff       — prefer for edits (less destructive)\n\
  4) write_file                  — ONLY for new files, or after read_file if overwriting\n\
  5) exec                        — build/test/git/install; avoid for file I/O\n\
\n\
Plan enforcement:\n\
  - Every tool call MUST map to a step in your <plan>. If not, update <plan> first.\n\
\n\
Choose the right tool:\n\
  Quick directory listing  → list_dir      (structure discovery, low token)\n\
  List files by pattern    → glob          (NOT exec+ls/find; OS-agnostic, token-efficient)\n\
  Find text in files       → search_files  (NOT exec+grep; safer, token-efficient)\n\
  Create/overwrite file    → write_file    (handles encoding; more reliable than exec+echo)\n\
  Small targeted edit      → read_file → patch_file  (simple single-snippet replacement)\n\
  Complex multi-hunk edit  → read_file → apply_diff  (multiple changes, spans many lines)\n\
  Run programs/tests       → exec\n\
  Git / installs           → exec\n\
\n\
apply_diff format (include 2-3 context lines for reliable matching):\n\
  @@ -10,4 +10,5 @@\n\
   context line\n\
  -old line to remove\n\
  +new line to add\n\
   context line\n\
\n\
After every file edit: tests run automatically if configured — check the result.\n\
After every build/test: confirm exit_code == 0 before proceeding.\n\
\n\
When ALL steps from your <plan> are verified complete:\n\
  call exec one final time to run a smoke test or confirm the deliverable exists,\n\
  then call done with a brief summary: what was built, where it lives, how to run it.\n\
  Do NOT call done while any command is still failing.";

/// Build the full Coder system prompt: base + scratchpad + OS rules + persona + language.
pub fn coder_system(persona_prompt: &str, lang_instruction: &str) -> String {
    let mut s = CODER_BASE_SYSTEM.to_string();
    s.push_str(SCRATCHPAD_ADDON);
    if cfg!(target_os = "windows") {
        s.push_str(WIN_SYSTEM_ADDON);
    }
    if !persona_prompt.is_empty() {
        s.push_str("\n\n");
        s.push_str(persona_prompt);
    }
    if !lang_instruction.is_empty() {
        s.push_str("\n\n");
        s.push_str(lang_instruction);
    }
    s
}

// ── Token-efficient helpers ───────────────────────────────────────────────────

/// Trim output that exceeds `max_chars`, appending a truncation notice.
/// This is the biggest single token-saver: a `cargo build` or `ls -R` can
/// return tens of thousands of characters; we only need the key lines.
fn truncate_output(s: &str, max_chars: usize) -> String {
    let s = s.trim_end();
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_chars).collect();
    let total_lines = s.lines().count();
    format!("{truncated}\n[…truncated — {total_lines} lines total, first {max_chars} chars shown]")
}

/// Tail-biased truncation: errors appear at the END of compiler output, not the beginning.
/// E.g. `cargo build` prints 100+ "Compiling …" lines before the actual error.
/// Showing the last `max_chars` puts the real error in view.
fn truncate_output_tail(s: &str, max_chars: usize) -> String {
    let s = s.trim_end();
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let skip = char_count - max_chars;
    let tail: String = s.chars().skip(skip).collect();
    let total_lines = s.lines().count();
    let shown_lines = tail.lines().count();
    let hidden = total_lines.saturating_sub(shown_lines);
    format!("[…{hidden} earlier lines omitted — showing last {max_chars} chars]\n{tail}")
}

/// Line- and char-bounded preview truncation (used for approval prompts).
fn truncate_preview(s: &str, max_chars: usize, max_lines: usize) -> String {
    let mut out = String::new();
    let mut lines = 0usize;
    for line in s.lines() {
        if lines >= max_lines {
            out.push_str("\n...truncated...\n");
            break;
        }
        if out.len() + line.len() + 1 > max_chars {
            out.push_str("\n...truncated...\n");
            break;
        }
        out.push_str(line);
        out.push('\n');
        lines += 1;
    }
    out.trim_end().to_string()
}

fn approx_tokens_text(s: &str) -> usize {
    // Rough heuristic:
    // - ASCII-ish text: ~4 chars/token
    // - Non-ASCII (CJK etc): closer to 1 char/token
    let mut ascii = 0usize;
    let mut non_ascii = 0usize;
    for ch in s.chars() {
        if ch.is_ascii() {
            ascii += 1;
        } else {
            non_ascii += 1;
        }
    }
    (ascii + 3) / 4 + non_ascii
}

fn approx_tokens_messages(messages: &[serde_json::Value]) -> usize {
    messages
        .iter()
        .map(|m| {
            m.get("content")
                .and_then(|c| c.as_str())
                .map(approx_tokens_text)
                .unwrap_or(0)
        })
        .sum()
}

fn simple_before_after(old: &str, new: &str) -> String {
    if old == new {
        return "(no changes)".to_string();
    }
    let old_p = truncate_preview(old, 1400, 60);
    let new_p = truncate_preview(new, 1400, 60);
    format!("--- before ---\n{old_p}\n\n--- after ---\n{new_p}\n")
        .trim_end()
        .to_string()
}

/// Extract a compact digest of error lines from command output.
/// Helps the model see ALL errors even when stdout is very long.
/// Returns None when no clear error lines are found.
fn extract_error_digest(stdout: &str, stderr: &str) -> Option<String> {
    let patterns: &[&str] = &[
        "error[e",         // Rust: error[E0XXX]
        "error: aborting", // Rust: summary line
        " --> ",           // Rust: file:line pointer
        "syntaxerror:",    // Python / JS
        "typeerror:",
        "nameerror:",
        "attributeerror:",
        "valueerror:",
        "runtimeerror:",
        "importerror:",
        "modulenotfounderror:",
        "referenceerror:", // JS
        "traceback (most recent call last)",
        "error: ", // generic (space avoids false positives)
        "fatal: ",
        "fatal error:",
    ];

    let mut lines: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for src in [stderr, stdout] {
        for line in src.lines() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            let low = t.to_ascii_lowercase();
            if patterns.iter().any(|p| low.contains(p)) {
                if seen.insert(t.to_string()) {
                    lines.push(t.to_string());
                    if lines.len() >= 20 {
                        break;
                    }
                }
            }
        }
        if lines.len() >= 20 {
            break;
        }
    }

    if lines.is_empty() {
        return None;
    }
    Some(format!(
        "[ERROR DIGEST — {} line(s)]\n{}",
        lines.len(),
        lines.join("\n")
    ))
}

/// Returns true if a tool result can safely be pruned (= it was a success).
/// Failures are never pruned because they are critical context for recovery.
fn is_prunable_tool_result(content: &str) -> bool {
    let c = content.trim_start();
    // exec success
    c.starts_with("OK (exit_code: 0)")
        // write_file / patch_file success
        || c.starts_with("OK: wrote '")
        || c.starts_with("OK: patched '")
        // read_file success — header starts with "[path] (N lines"
        || (c.starts_with('[') && c.contains("] (") && c.contains(" lines,"))
}

/// Collapse tool result messages older than KEEP_RECENT_TOOL_TURNS to a
/// one-line summary.  Each collapsed result saves ~200-2000 tokens.
/// Only the content field is modified; tool_call_id stays intact.
fn prune_old_tool_results(messages: &mut Vec<serde_json::Value>) {
    let tool_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m["role"].as_str() == Some("tool"))
        .map(|(i, _)| i)
        .collect();

    if tool_indices.len() <= KEEP_RECENT_TOOL_TURNS {
        return;
    }

    let prune_count = tool_indices.len() - KEEP_RECENT_TOOL_TURNS;
    for &idx in tool_indices.iter().take(prune_count) {
        if let Some(content) = messages[idx]["content"].as_str() {
            // Never prune failures: they are the most important context for recovery.
            // Prune successful exec outputs and file tool outputs.
            if !is_prunable_tool_result(content) {
                continue;
            }
            let line_count = content.lines().count();
            if line_count > 2 {
                let first = content.lines().next().unwrap_or("[done]").to_string();
                messages[idx]["content"] =
                    serde_json::Value::String(format!("{first} [pruned {line_count}L]"));
            }
        }
    }
}

// ── Error classification ──────────────────────────────────────────────────────

/// Classifies the kind of error from stderr/stdout so that targeted recovery
/// hints can be injected before the generic diagnosis protocol.
#[derive(Debug, Clone, PartialEq)]
enum ErrorClass {
    Environment, // command not found, permission denied, not recognized
    Syntax,      // parse error, unexpected token, syntax error
    Path,        // no such file, cannot find path, path not found
    Dependency,  // module not found, missing package/crate
    Network,     // connection refused, timeout, could not connect
    Logic,       // assertion failed, test failed, expected X got Y
    Unknown,
}

impl Default for ErrorClass {
    fn default() -> Self {
        ErrorClass::Unknown
    }
}

fn classify_error(stderr: &str, stdout: &str) -> ErrorClass {
    let combined = format!("{stderr}\n{stdout}");
    let low = combined.to_ascii_lowercase();

    if low.contains("command not found")
        || low.contains("is not recognized as the name")
        || low.contains("is not recognized as an internal")
        || low.contains("permission denied")
        || low.contains("access is denied")
        || low.contains("access denied")
        || low.contains("commandnotfoundexception")
    {
        ErrorClass::Environment
    } else if low.contains("syntax error")
        || low.contains("unexpected token")
        || low.contains("parse error")
        || low.contains("parsererror")
        || low.contains("invalid syntax")
        || low.contains("missing expression")
        || low.contains("unexpected end of")
    {
        ErrorClass::Syntax
    } else if low.contains("no such file")
        || low.contains("cannot find path")
        || low.contains("path not found")
        || (low.contains("does not exist") && !low.contains("package"))
    {
        ErrorClass::Path
    } else if low.contains("modulenotfounderror")
        || low.contains("cannot find module")
        || low.contains("no module named")
        || low.contains("package not found")
        || low.contains("no such package")
        || (low.contains("could not find") && (low.contains("package") || low.contains("crate")))
    {
        ErrorClass::Dependency
    } else if low.contains("connection refused")
        || low.contains("timed out")
        || low.contains("network unreachable")
        || low.contains("could not connect")
        || low.contains("name resolution failed")
    {
        ErrorClass::Network
    } else if low.contains("assertion")
        || low.contains("test failed")
        || (low.contains("expected") && low.contains("actual"))
    {
        ErrorClass::Logic
    } else {
        ErrorClass::Unknown
    }
}

/// Returns a targeted one-line recovery hint for each error class,
/// or an empty string for Unknown (no hint injected).
fn error_class_hint(class: &ErrorClass) -> &'static str {
    match class {
        ErrorClass::Environment =>
            "⚠ ENVIRONMENT ERROR: The binary/permission is missing. Fix the environment first — do NOT modify source code.",
        ErrorClass::Syntax =>
            "⚠ SYNTAX ERROR: There is a language/parser mistake. Fix the exact line — do NOT change unrelated code.",
        ErrorClass::Path =>
            "⚠ PATH ERROR: A file or directory is missing. Verify paths with ls/Get-ChildItem before creating or reading.",
        ErrorClass::Dependency =>
            "⚠ DEPENDENCY ERROR: A required package is missing. Install it first, then retry the original command.",
        ErrorClass::Network =>
            "⚠ NETWORK ERROR: Cannot reach a remote service. Check if the service is running and proxy vars are clear.",
        ErrorClass::Logic =>
            "⚠ LOGIC ERROR: The code ran but produced wrong results. Re-read the relevant logic before re-running.",
        ErrorClass::Unknown => "",
    }
}

/// Extra targeted hints for known high-frequency Windows/Git failure modes.
fn specific_recovery_hint(stderr: &str, stdout: &str) -> &'static str {
    let combined = format!("{stderr}\n{stdout}");
    let low = combined.to_ascii_lowercase();

    // GitHub HTTPS routed via a dead local proxy (common in locked-down networks).
    // Example: "Failed to connect to github.com port 443 via 127.0.0.1 ... Could not connect to server"
    let git_github_proxy = (low.contains("github.com") || low.contains("ssh.github.com"))
        && low.contains("port 443")
        && (low.contains("via 127.0.0.1") || low.contains("via localhost"))
        && (low.contains("could not connect") || low.contains("failed to connect"));
    if git_github_proxy {
        return "HINT: Git HTTPS appears to be routed via a dead local proxy (127.0.0.1/localhost).\n\
- Try clearing proxy env vars: HTTP_PROXY / HTTPS_PROXY / ALL_PROXY / GIT_HTTP_PROXY / GIT_HTTPS_PROXY.\n\
- For GitHub, prefer SSH-over-443:\n\
  git push ssh://git@ssh.github.com:443/<owner>/<repo>.git main\n\
  (and similarly for fetch/pull).";
    }

    // Windows: `cargo run` cannot overwrite a running .exe (locked file handle).
    // Example: "failed to remove file ... obstral.exe ... access is denied (os error 5)"
    let cargo_exe_lock = low.contains("failed to remove file")
        && low.contains("obstral.exe")
        && (low.contains("os error 5") || low.contains("access is denied"));
    if cargo_exe_lock {
        return "HINT: On Windows, a running .exe cannot be overwritten.\n\
- Stop the running process (`Stop-Process -Name obstral -Force`) OR restart the terminal.\n\
- Or run cargo with an isolated target dir to avoid the lock:\n\
  $env:CARGO_TARGET_DIR = '.tmp/cargo-target-tui'; cargo run -- tui";
    }

    ""
}

// ── Tool output builders ──────────────────────────────────────────────────────

/// Build the tool result string for a failed command with structured
/// diagnosis guidance.  Forces the model to reason about the error rather
/// than blindly retrying or continuing.
fn build_failed_tool_output(stdout: &str, stderr: &str, exit_code: i32) -> String {
    let class = classify_error(stderr, stdout);

    let specific_hint = specific_recovery_hint(stderr, stdout);
    let class_hint = error_class_hint(&class);

    let mut hint_prefix = String::new();
    if !specific_hint.is_empty() {
        hint_prefix.push_str(specific_hint);
        hint_prefix.push_str("\n\n");
    }
    if !class_hint.is_empty() {
        hint_prefix.push_str(class_hint);
        hint_prefix.push_str("\n\n");
    }
    let mut out = format!(
        "{hint_prefix}FAILED (exit_code: {exit_code})\n\
         \n\
         ⚠ STOP — diagnosis required before your next action:\n\
         1. Quote the exact line causing the error.\n\
         2. Identify the root cause in one sentence.\n\
         3. Fix it with a single corrected command.\n\
         Do NOT continue the original plan until the fix succeeds.\n"
    );

    // Error digest first: compact list of all error lines so the model sees
    // ALL errors even when stdout is long (e.g. cargo build with many "Compiling" lines).
    if let Some(digest) = extract_error_digest(stdout, stderr) {
        out.push_str(&format!("\n{digest}\n"));
    }

    // Tail-biased: errors appear at the END of compiler output, not the beginning.
    // Showing the last N chars ensures the real errors are in view.
    let stdout_t = truncate_output_tail(stdout, MAX_STDOUT_CHARS);
    let stderr_t = truncate_output(stderr, MAX_STDERR_CHARS);
    if !stdout_t.is_empty() {
        out.push_str(&format!("\nstdout (tail):\n{stdout_t}\n"));
    }
    if !stderr_t.is_empty() {
        out.push_str(&format!("\nstderr:\n{stderr_t}\n"));
    }
    out
}

fn build_ok_tool_output(stdout: &str) -> String {
    let stdout_t = truncate_output(stdout, MAX_STDOUT_CHARS);
    if stdout_t.is_empty() {
        "OK (exit_code: 0)".to_string()
    } else {
        format!("OK (exit_code: 0)\nstdout:\n{stdout_t}")
    }
}

fn wrap_exec_with_pwd(cmd: &str) -> String {
    let raw = cmd.trim();
    if raw.is_empty() {
        return String::new();
    }

    if cfg!(target_os = "windows") {
        // Emit marker even on failures to keep recovery loops from losing cwd.
        // NOTE: if this wrapped script triggers bash->PowerShell translation, the translator MUST
        // preserve standalone `}` lines (see exec.rs).
        return [
            "$ErrorActionPreference = 'Stop'",
            "try {",
            raw,
            "} finally {",
            &format!("Write-Output (\"{}\" + (Get-Location).Path)", PWD_MARKER),
            "}",
        ]
        .join("\n");
    }

    // POSIX: keep behavior simple (do not `set -e`).
    format!("{raw}\necho \"{PWD_MARKER}$(pwd)\"")
}

fn strip_pwd_marker(stdout_raw: &str) -> (String, Option<String>) {
    let raw = stdout_raw.replace("\r\n", "\n");
    if raw.is_empty() {
        return (String::new(), None);
    }
    let mut kept: Vec<&str> = Vec::new();
    let mut pwd: Option<String> = None;
    for ln in raw.split('\n') {
        if let Some(rest) = ln.strip_prefix(PWD_MARKER) {
            let p = rest.trim();
            if !p.is_empty() {
                pwd = Some(p.to_string());
            }
            continue;
        }
        kept.push(ln);
    }
    (kept.join("\n").trim_end().to_string(), pwd)
}

fn normalize_path_sep(s: &str) -> String {
    s.replace('\\', "/")
}

fn is_within_root(path: &str, root: &str) -> bool {
    let p = normalize_path_sep(path)
        .replace('\u{0}', "")
        .trim()
        .trim_end_matches('/')
        .to_string();
    let r = normalize_path_sep(root)
        .replace('\u{0}', "")
        .trim()
        .trim_end_matches('/')
        .to_string();
    if p.is_empty() || r.is_empty() {
        return false;
    }
    if cfg!(target_os = "windows") {
        let p = p.to_ascii_lowercase();
        let r = r.to_ascii_lowercase();
        p == r || p.starts_with(&(r + "/"))
    } else {
        p == r || p.starts_with(&(r + "/"))
    }
}

fn absolutize_path(path: &str) -> Option<String> {
    let p = path.trim();
    if p.is_empty() {
        return None;
    }
    let pb = std::path::PathBuf::from(p);
    let abs = if pb.is_absolute() {
        pb
    } else {
        let cwd = std::env::current_dir().ok()?;
        cwd.join(pb)
    };
    Some(abs.to_string_lossy().into_owned())
}

fn inject_cwd(tool_output: &str, cwd_line: &str, note: Option<&str>) -> String {
    let t = tool_output.trim_end_matches('\n');
    if t.is_empty() {
        return String::new();
    }
    let (first, rest) = match t.split_once('\n') {
        Some((a, b)) => (a, b),
        None => (t, ""),
    };

    let mut out = String::new();
    out.push_str(first);
    out.push('\n');
    out.push_str(cwd_line);
    if let Some(n) = note {
        if !n.trim().is_empty() {
            out.push('\n');
            out.push_str(n.trim_end());
        }
    }
    if !rest.trim().is_empty() {
        out.push('\n');
        out.push_str(rest);
    }
    out
}

// ── Loop Governor (Coder strengthening) ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentState {
    Planning,
    Executing,
    Verifying,
    Recovery,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryStage {
    Diagnose,
    Fix,
    Verify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecKind {
    Diagnostic,
    Action,
    Verify,
}

#[derive(Debug, Default)]
struct RecoveryGovernor {
    stage: Option<RecoveryStage>,
}

impl RecoveryGovernor {
    fn stage_label(&self) -> &'static str {
        match self.stage {
            None => "none",
            Some(RecoveryStage::Diagnose) => "diagnose",
            Some(RecoveryStage::Fix) => "fix",
            Some(RecoveryStage::Verify) => "verify",
        }
    }

    fn in_recovery(&self) -> bool {
        self.stage.is_some()
    }

    fn restore_from_session(mem: &FailureMemory, messages: &[serde_json::Value]) -> Self {
        let mut g = RecoveryGovernor::default();
        if mem.consecutive_failures > 0 || last_tool_looks_failed(messages) {
            g.stage = Some(RecoveryStage::Diagnose);
        }
        g
    }

    fn maybe_block_tool(&self, tc: &ToolCallData, test_cmd: Option<&str>) -> Option<String> {
        let Some(stage) = self.stage else {
            return None;
        };
        let name = tc.name.as_str();

        // Note: `done` is handled earlier in the main loop.
        match stage {
            RecoveryStage::Diagnose => {
                if is_diagnostic_tool_name(name) {
                    return None;
                }
                if name == "exec" {
                    let cmd =
                        parse_exec_command_from_args(tc.arguments.as_str()).unwrap_or_default();
                    if is_diagnostic_command(cmd.as_str()) {
                        return None;
                    }
                }
                Some(format!(
                    "[Recovery Gate] stage=diagnose\n\
You are in recovery mode. Do NOT start new work yet.\n\
Required now: run diagnostics first (e.g. `pwd`, `ls`/`dir`, `git status`, `git rev-parse --show-toplevel`)."
                ))
            }
            RecoveryStage::Fix => None, // allow edits/commands to fix
            RecoveryStage::Verify => {
                if name == "exec" {
                    let cmd =
                        parse_exec_command_from_args(tc.arguments.as_str()).unwrap_or_default();
                    if is_verify_command(cmd.as_str(), test_cmd) {
                        return None;
                    }
                }
                Some(format!(
                    "[Recovery Gate] stage=verify\n\
You already applied a fix. Verify before continuing.\n\
Required now: run ONE verification command (tests or `git status`)."
                ))
            }
        }
    }

    fn on_diagnostic_result(&mut self, ok: bool) {
        if !self.in_recovery() && !ok {
            self.stage = Some(RecoveryStage::Diagnose);
            return;
        }
        if !ok {
            self.stage = Some(RecoveryStage::Diagnose);
            return;
        }
        if self.stage == Some(RecoveryStage::Diagnose) {
            self.stage = Some(RecoveryStage::Fix);
        }
    }

    fn on_fix_result(&mut self, ok: bool, verified: bool) {
        if !self.in_recovery() && !ok {
            self.stage = Some(RecoveryStage::Diagnose);
            return;
        }
        if !ok {
            self.stage = Some(RecoveryStage::Diagnose);
            return;
        }
        if verified {
            self.stage = None;
            return;
        }
        match self.stage {
            Some(RecoveryStage::Diagnose) | Some(RecoveryStage::Fix) => {
                self.stage = Some(RecoveryStage::Verify);
            }
            _ => {}
        }
    }

    fn on_exec_result(&mut self, kind: ExecKind, ok: bool) {
        if !ok {
            self.stage = Some(RecoveryStage::Diagnose);
            return;
        }
        if kind == ExecKind::Verify {
            // A successful verification ends recovery regardless of the current stage.
            self.stage = None;
            return;
        }
        match (self.stage, kind) {
            (Some(RecoveryStage::Diagnose), ExecKind::Diagnostic) => {
                self.stage = Some(RecoveryStage::Fix);
            }
            (Some(RecoveryStage::Fix), ExecKind::Action) => {
                self.stage = Some(RecoveryStage::Verify);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
struct FailureMemory {
    consecutive_failures: usize,

    last_command_sig: Option<String>,
    same_command_repeats: usize,

    last_error_sig: Option<String>,
    same_error_repeats: usize,

    last_output_hash: Option<u64>,
    same_output_repeats: usize,

    last_error_class: ErrorClass,
}

fn last_tool_looks_failed(messages: &[serde_json::Value]) -> bool {
    let Some(last_tool) = messages
        .iter()
        .rev()
        .find(|m| m.get("role").and_then(|v| v.as_str()) == Some("tool"))
    else {
        return false;
    };
    let content = last_tool
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let low = content.to_ascii_lowercase();
    low.contains("failed (exit_code:")
        || low.contains("governor blocked")
        || low.contains("rejected by user")
        || low.contains("[result_file_err]")
}

fn is_diagnostic_tool_name(name: &str) -> bool {
    matches!(name, "read_file" | "search_files" | "list_dir" | "glob")
}

fn is_fix_tool_name(name: &str) -> bool {
    matches!(name, "write_file" | "patch_file" | "apply_diff")
}

fn is_diagnostic_command(command: &str) -> bool {
    let c = command_sig(command);
    let pats = [
        "pwd",
        "cd",
        "set-location",
        "pushd",
        "popd",
        "ls",
        "dir",
        "get-location",
        "get-childitem",
        "where",
        "which",
        "get-command",
        "echo",
        "write-output",
        "whoami",
        "hostname",
        "git status",
        "git rev-parse",
        "git remote",
        "git branch",
        "git diff",
        "cargo --version",
        "rustc --version",
        "python --version",
        "node --version",
        "npm --version",
        "pnpm --version",
        "yarn --version",
        "go version",
        "dotnet --info",
    ];
    pats.iter().any(|p| c.contains(p))
}

fn is_verify_command(command: &str, test_cmd: Option<&str>) -> bool {
    let c = command_sig(command);
    let pats = [
        "cargo test",
        "cargo build",
        "cargo check",
        "npm test",
        "pnpm test",
        "yarn test",
        "pytest",
        "go test",
        "dotnet test",
        "git status",
    ];
    if pats.iter().any(|p| c.contains(p)) {
        return true;
    }
    if let Some(t) = test_cmd {
        let t_sig = command_sig(t);
        if !t_sig.is_empty() && c.contains(&t_sig) {
            return true;
        }
    }
    false
}

fn classify_exec_kind(command: &str, test_cmd: Option<&str>) -> ExecKind {
    if is_verify_command(command, test_cmd) {
        return ExecKind::Verify;
    }
    if is_diagnostic_command(command) {
        return ExecKind::Diagnostic;
    }
    ExecKind::Action
}

fn normalize_for_signature(s: &str) -> String {
    // Keep this tiny: lowercased + digits collapsed removes most "At line:123" noise.
    let mut out = String::with_capacity(s.len().min(160));
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            out.push('#');
        } else {
            out.push(ch.to_ascii_lowercase());
        }
        if out.len() >= 160 {
            break;
        }
    }
    out
}

fn command_sig(command: &str) -> String {
    // Single line, trimmed, collapsed whitespace.
    let normalized = command.replace("\r\n", "\n");
    let one = normalized
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    let collapsed = one.split_whitespace().collect::<Vec<_>>().join(" ");
    normalize_for_signature(&collapsed)
}

fn pick_interesting_error_line(stdout: &str, stderr: &str) -> String {
    let keywords = [
        "error",
        "fatal",
        "exception",
        "traceback",
        "parsererror",
        "unexpected token",
        "not recognized",
        "commandnotfoundexception",
        "missing expression",
        "unable to",
        "could not",
        "access is denied",
        "permission denied",
    ];

    for src in [stderr, stdout] {
        for ln in src.lines() {
            let t = ln.trim();
            if t.is_empty() {
                continue;
            }
            let low = t.to_ascii_lowercase();
            if keywords.iter().any(|k| low.contains(k)) {
                return normalize_for_signature(t);
            }
        }
    }

    // Fall back to the first non-empty line.
    for src in [stderr, stdout] {
        for ln in src.lines() {
            let t = ln.trim();
            if !t.is_empty() {
                return normalize_for_signature(t);
            }
        }
    }

    String::new()
}

fn error_signature(command: &str, stdout: &str, stderr: &str, exit_code: i32) -> String {
    let cmd = command_sig(command);
    let line = pick_interesting_error_line(stdout, stderr);
    format!("exit={exit_code}|cmd={cmd}|err={line}")
}

fn hash_output(stdout: &str, stderr: &str) -> u64 {
    let mut h = DefaultHasher::new();
    stdout.trim_end().hash(&mut h);
    stderr.trim_end().hash(&mut h);
    h.finish()
}

fn hash_text(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.trim_end().hash(&mut h);
    h.finish()
}

fn fmt_hash(h: u64) -> String {
    format!("{h:016x}")
}

fn suspicious_success_reason(stdout: &str, stderr: &str) -> Option<String> {
    // PowerShell often exits 0 even when it printed errors (non-terminating error records).
    // Cargo warnings also go to stderr, so we only trigger on strong error markers.
    let bad = [
        "parsererror",
        "unexpected token",
        "missing expression",
        "commandnotfoundexception",
        "not recognized",
        "error:",
        "fatal:",
        "exception",
        "traceback",
        "access is denied",
        "permission denied",
        "does not have a commit checked out",
        "unable to index file",
        "could not find a part of the path",
    ];

    for src in [stderr, stdout] {
        let low = src.to_ascii_lowercase();
        if bad.iter().any(|k| low.contains(k)) {
            // Return just enough to explain why we treated this as failure.
            let line = pick_interesting_error_line(stdout, stderr);
            if !line.is_empty() {
                return Some(format!(
                    "exit_code was 0, but output contained error markers (e.g. `{line}`)"
                ));
            }
            return Some("exit_code was 0, but output contained error markers".to_string());
        }
    }

    None
}

fn hint_for_known_failure(command: &str, stdout: &str, stderr: &str) -> Option<String> {
    let mut s = String::new();
    s.push_str(stdout);
    s.push('\n');
    s.push_str(stderr);
    let low = s.to_ascii_lowercase();
    let cmd_low = command.to_ascii_lowercase();

    if low.contains("the term '$' is not recognized")
        || (low.contains("the term '$'") && low.contains("not recognized"))
    {
        return Some(
            "Your command includes a transcript prompt marker like `$` / `PS>`.\n\
Fix: send ONLY the command (e.g. `git status`), not `$ git status`."
                .to_string(),
        );
    }
    if low.contains("unexpected token '}'")
        || (low.contains("unexpected token") && low.contains('}'))
    {
        return Some(
            "PowerShell saw a stray `}` in the command.\n\
Fix: remove the trailing `}` and retry."
                .to_string(),
        );
    }
    if low.contains("adding embedded git repository")
        || low.contains("does not have a commit checked out")
    {
        return Some(
            "You are trying to `git add` a nested repo directory.\n\
Fix: `cd` into the intended repo before `git add .`, or add the nested repo dir to `.gitignore` (or use `git submodule add ...`)."
                .to_string(),
        );
    }
    if low.contains("failed to remove file")
        && (low.contains("access is denied") || low.contains("permission denied"))
        && cmd_low.contains("cargo")
    {
        return Some(
            "Rust build failed because `obstral.exe` is locked.\n\
Fix: stop the running process (or close the TUI/serve), then rebuild.\n\
Tip (Windows): use `scripts/run-tui.ps1` / `scripts/run-ui.ps1` which build in an isolated CARGO_TARGET_DIR and auto-kill old processes."
                .to_string(),
        );
    }
    if low.contains("could not connect to server")
        && low.contains("127.0.0.1")
        && cmd_low.contains("git")
    {
        return Some(
            "Git network failed via a dead local proxy (127.0.0.1).\n\
Fix: clear proxy env vars: `Remove-Item Env:HTTP_PROXY,Env:HTTPS_PROXY,Env:ALL_PROXY -ErrorAction SilentlyContinue`."
                .to_string(),
        );
    }
    if low.contains("incorrect api key")
        || low.contains("invalid_api_key")
        || (low.contains("http 401") && low.contains("api key"))
    {
        return Some(
            "Provider returned HTTP 401 (bad/missing API key).\n\
Fix: update the configured API key for the selected provider/model, then retry."
                .to_string(),
        );
    }

    None
}

fn is_git_repo_root(dir: &str) -> bool {
    let p = Path::new(dir);
    let dot_git = p.join(".git");
    dot_git.is_dir() || dot_git.is_file()
}

fn gitmodules_lists_path(repo_root: &str, rel_path: &str) -> bool {
    let p = Path::new(repo_root).join(".gitmodules");
    let Ok(text) = std::fs::read_to_string(&p) else {
        return false;
    };
    let needle = format!("path = {rel_path}");
    text.lines().any(|l| l.trim() == needle)
}

fn nested_git_dirs_shallow(repo_root: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let root = Path::new(repo_root);
    let Ok(rd) = std::fs::read_dir(root) else {
        return out;
    };
    for ent in rd.flatten() {
        let path = ent.path();
        if !path.is_dir() {
            continue;
        }
        let dot_git = path.join(".git");
        if !(dot_git.is_dir() || dot_git.is_file()) {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        // Ignore the repo root itself (we only scan immediate children anyway).
        if name == ".git" {
            continue;
        }
        out.push(name.to_string());
    }
    out.sort();
    out.dedup();
    out
}

fn should_block_git_landmines(command: &str, tool_root_abs: Option<&str>) -> Option<String> {
    let root = tool_root_abs?;
    if !is_git_repo_root(root) {
        return None;
    }

    let cmd_low = command.to_ascii_lowercase();

    // 1) Never create a new git repo inside an existing repo; this causes embedded repos and breaks `git add`.
    if cmd_low.contains("git init") {
        return Some(format!(
            "Refusing to run `git init` inside an existing git repo (tool_root has .git): {root}\n\
This is a common agent failure mode that creates embedded repos and breaks `git add`.\n\
Fix: set tool_root to a fresh directory outside this repo (e.g. `.tmp/newrepo`), or remove `git init` and just create files."
        ));
    }

    // 2) Block `git add` when we detect nested git directories under tool_root (unless they are declared submodules).
    if cmd_low.contains("git add") {
        let nested = nested_git_dirs_shallow(root);
        let mut offenders: Vec<String> = Vec::new();
        for d in nested {
            if !gitmodules_lists_path(root, &d) {
                offenders.push(d);
            }
        }
        if !offenders.is_empty() {
            let list = offenders.join(", ");
            return Some(format!(
                "Nested git repo(s) detected under tool_root: {list}\n\
Running `git add` will trigger embedded-repo errors or index failures.\n\
Fix: move those directories outside tool_root, add them to `.gitignore`, or add them properly as submodules (`git submodule add <url> <path>`)."
            ));
        }
    }

    None
}

fn wants_local_actions(user_text: &str) -> bool {
    let s = user_text.to_ascii_lowercase();
    let kws = [
        "repo",
        "repository",
        "scaffold",
        "bootstrap",
        "init",
        "setup",
        "create",
        "generate",
        "implement",
        "build",
        "test",
        "run",
        "install",
        "folder",
        "directory",
        "file",
        "git",
        "commit",
        "push",
        // Japanese
        "リポ",
        "リポジトリ",
        "フォルダ",
        "ディレクトリ",
        "ファイル",
        "作成",
        "作る",
        "実装",
        "実行",
        "コミット",
        "プッシュ",
        // French
        "dépôt",
        "depot",
        "répertoire",
        "repertoire",
        "fichier",
        "commande",
        "créer",
        "creer",
        "générer",
        "generer",
        "exécuter",
        "executer",
        "installer",
    ];
    kws.iter().any(|k| s.contains(k))
}

#[derive(Debug, Clone)]
struct ImpliedExecScript {
    lang_hint: String,
    script: String,
}

fn is_shell_fence_lang(lang_hint: &str) -> bool {
    matches!(
        lang_hint,
        "bash" | "sh" | "shell" | "zsh" | "powershell" | "pwsh" | "ps1" | "ps" | "console"
    )
}

fn looks_like_shell_command(script: &str) -> bool {
    let low = script.to_ascii_lowercase();
    let pats = [
        // PowerShell
        "new-item",
        "set-content",
        "add-content",
        "remove-item",
        "copy-item",
        "move-item",
        "get-content",
        "test-path",
        // Common CLIs
        "\ngit ",
        "git ",
        "\ncargo ",
        "cargo ",
        "\npython",
        "python ",
        "\nnode ",
        "node ",
        "\nnpm ",
        "npm ",
        "\npnpm ",
        "pnpm ",
        "\nyarn ",
        "yarn ",
        // Shell-ish
        "\ncd ",
        "\nmkdir",
        "mkdir ",
    ];
    pats.iter().any(|p| low.contains(p))
}

fn extract_implied_exec_scripts(text: &str) -> Vec<ImpliedExecScript> {
    let raw = text.replace("\r\n", "\n");
    let mut out: Vec<ImpliedExecScript> = Vec::new();

    let mut in_fence = false;
    let mut fence_lang = String::new();
    let mut buf: Vec<String> = Vec::new();

    for line0 in raw.lines() {
        let line = line0.trim_end_matches('\r');
        let t = line.trim_start();
        if !in_fence {
            if let Some(rest) = t.strip_prefix("```") {
                in_fence = true;
                fence_lang = rest.trim().to_ascii_lowercase();
                buf.clear();
            }
            continue;
        }

        // in_fence
        if t.starts_with("```") {
            let body = buf.join("\n");
            let script = body.trim().to_string();
            if !script.is_empty() {
                let lang = fence_lang.trim().to_string();
                if is_shell_fence_lang(lang.as_str()) || looks_like_shell_command(&script) {
                    out.push(ImpliedExecScript {
                        lang_hint: lang,
                        script,
                    });
                    if out.len() >= 3 {
                        return out;
                    }
                }
            }
            in_fence = false;
            fence_lang.clear();
            buf.clear();
            continue;
        }

        buf.push(line.to_string());
    }

    // Fallback: some models omit code fences but still paste PS prompt lines.
    if out.is_empty() {
        let mut lines: Vec<String> = Vec::new();
        for line0 in raw.lines() {
            let t = line0.trim_start();
            if t.starts_with("PS>") || t.starts_with("$ ") || t.starts_with("> ") {
                lines.push(t.to_string());
                if lines.len() >= 24 {
                    break;
                }
            }
        }
        let joined = lines.join("\n").trim().to_string();
        if !joined.is_empty() && looks_like_shell_command(&joined) {
            out.push(ImpliedExecScript {
                lang_hint: "powershell".to_string(),
                script: joined,
            });
        }
    }

    out
}

impl FailureMemory {
    fn from_recent_messages(messages: &[serde_json::Value]) -> Self {
        let mut mem = FailureMemory::default();

        // Map tool_call_id -> command for exec calls.
        let mut exec_by_id: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for msg in messages {
            let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");

            if role == "assistant" {
                let Some(tcs) = msg.get("tool_calls").and_then(|v| v.as_array()) else {
                    continue;
                };
                for tc in tcs {
                    let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("").trim();
                    if id.is_empty() {
                        continue;
                    }
                    let name = tc
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    if name != "exec" {
                        continue;
                    }
                    let args = tc
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    if let Some(cmd) = parse_exec_command_from_args(args) {
                        exec_by_id.insert(id.to_string(), cmd);
                    }
                }
                continue;
            }

            if role == "tool" {
                let tcid = msg
                    .get("tool_call_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim();
                if tcid.is_empty() {
                    continue;
                }
                let Some(command) = exec_by_id.remove(tcid) else {
                    continue;
                };
                let content = msg
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let (exit_code, stdout, stderr) = parse_exec_tool_output_sections(&content);
                let Some(mut effective_exit_code) = exit_code else {
                    continue;
                };
                if effective_exit_code == 0
                    && suspicious_success_reason(stdout.as_str(), stderr.as_str()).is_some()
                {
                    effective_exit_code = 1;
                }
                let _ = mem.on_tool_result(
                    command.as_str(),
                    stdout.as_str(),
                    stderr.as_str(),
                    effective_exit_code,
                );
            }
        }

        mem
    }

    fn on_tool_result(
        &mut self,
        command: &str,
        stdout: &str,
        stderr: &str,
        effective_exit_code: i32,
    ) -> Option<String> {
        // Track repeated identical commands (common loop symptom).
        let cmd_sig = command_sig(command);
        if self.last_command_sig.as_deref() == Some(&cmd_sig) {
            self.same_command_repeats = self.same_command_repeats.saturating_add(1);
        } else {
            self.last_command_sig = Some(cmd_sig);
            self.same_command_repeats = 1;
        }

        // Track output hash (stuck detection).
        let oh = hash_output(stdout, stderr);
        if self.last_output_hash == Some(oh) {
            self.same_output_repeats = self.same_output_repeats.saturating_add(1);
        } else {
            self.last_output_hash = Some(oh);
            self.same_output_repeats = 1;
        }

        if effective_exit_code == 0 {
            self.consecutive_failures = 0;
            self.last_error_sig = None;
            self.same_error_repeats = 0;
            self.last_error_class = ErrorClass::Unknown;
            return None;
        }

        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.last_error_class = classify_error(stderr, stdout);

        let sig = error_signature(command, stdout, stderr, effective_exit_code);
        if self.last_error_sig.as_deref() == Some(&sig) {
            self.same_error_repeats = self.same_error_repeats.saturating_add(1);
        } else {
            self.last_error_sig = Some(sig);
            self.same_error_repeats = 1;
        }

        // Emit hints only when crossing key thresholds to avoid spamming context.
        if self.same_error_repeats == 2 {
            if let Some(h) = hint_for_known_failure(command, stdout, stderr) {
                return Some(h);
            }
            return Some(
                "The SAME error happened twice.\n\
Action: stop repeating; gather diagnostics (`pwd`, `ls`, `git status`) then change strategy."
                    .to_string(),
            );
        }

        if self.same_command_repeats == 3 {
            return Some(
                "You ran the SAME command 3 times.\n\
Action: abandon this approach and try a different strategy (different cwd, different command, or add diagnostics)."
                    .to_string(),
            );
        }

        if self.consecutive_failures >= 3 {
            let class_ctx = error_class_hint(&self.last_error_class);
            let context = if class_ctx.is_empty() {
                String::new()
            } else {
                format!("\nLast error type: {class_ctx}")
            };
            return Some(format!(
                "3 consecutive failures.{context}\n\
Action: change strategy now; do NOT retry the same approach again."
            ));
        }

        if self.same_output_repeats >= 2 && self.same_command_repeats >= 2 {
            return Some(
                "Stuck detected: repeated identical output.\n\
Action: print diagnostics and change strategy; do not repeat the same command."
                    .to_string(),
            );
        }

        None
    }
}

fn parse_exec_command_from_args(args: &str) -> Option<String> {
    // Standard tool schema uses JSON arguments: {"command":"...","cwd":"..."}.
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(args) {
        if let Some(cmd) = v.get("command").and_then(|x| x.as_str()) {
            let t = cmd.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }

    // Fallback: some providers/models might pass a raw string.
    let t = args.trim();
    if !t.is_empty() {
        return Some(t.to_string());
    }
    None
}

fn parse_exit_code_from_tool_text(s: &str) -> Option<i32> {
    let low = s.to_ascii_lowercase();
    let key = "exit_code:";
    let idx = low.find(key)?;
    let after = low[idx + key.len()..].trim_start();
    let mut num = String::new();
    for ch in after.chars() {
        if ch == '-' || ch.is_ascii_digit() {
            num.push(ch);
        } else {
            break;
        }
    }
    if num.is_empty() {
        return None;
    }
    num.parse::<i32>().ok()
}

fn parse_exec_tool_output_sections(tool_content: &str) -> (Option<i32>, String, String) {
    let t = tool_content.replace("\r\n", "\n");
    let exit_code = parse_exit_code_from_tool_text(t.as_str());

    // Prefer parsing known markers so the governor sees the "real" stdout/stderr.
    // - OK: "OK (exit_code: 0)\nstdout:\n..."
    // - FAILED: "...FAILED (exit_code: X)\n...stdout (tail):\n...\nstderr:\n..."
    let mut stdout = String::new();
    let mut stderr = String::new();

    let (before_stderr, stderr_part) = match t.split_once("\nstderr:\n") {
        Some((a, b)) => (a, Some(b)),
        None => (t.as_str(), None),
    };
    if let Some(b) = stderr_part {
        stderr = b.trim_end().to_string();
    }

    // stdout markers can exist with or without stderr.
    if let Some((_, s_out)) = before_stderr.rsplit_once("\nstdout (tail):\n") {
        stdout = s_out.trim_end().to_string();
    } else if let Some((_, s_out)) = before_stderr.rsplit_once("\nstdout:\n") {
        stdout = s_out.trim_end().to_string();
    }

    // If we can't parse anything meaningful, treat the whole tool output as stderr on failures.
    if stdout.is_empty() && stderr.is_empty() {
        if exit_code.unwrap_or(0) != 0 {
            stderr = t.trim_end().to_string();
        }
    }

    (exit_code, stdout, stderr)
}

// ── Git helpers ───────────────────────────────────────────────────────────────

/// Create a git checkpoint commit in `root` (if it is a git repo).
/// Returns the HEAD hash after the commit, or None if not a git repo / git unavailable.
async fn git_create_checkpoint(root: &str) -> Option<String> {
    // Only proceed if this is a git repo.
    let head = run_git_cmd(root, &["rev-parse", "HEAD"]).await;
    if head.is_none() {
        return None;
    }

    // Stage all current changes (untracked included).
    let _ = run_git_cmd(root, &["add", "-A"]).await;

    // Commit with --allow-empty so we always get a clean ref even if nothing changed.
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let msg = format!("obstral: pre-session checkpoint {epoch}");
    let _ = run_git_cmd(root, &["commit", "--allow-empty", "-m", &msg]).await;

    // Return the new HEAD hash.
    run_git_cmd(root, &["rev-parse", "HEAD"]).await
}

/// Run `git -C root <args>` with a 5-second timeout. Returns trimmed stdout or None.
async fn run_git_cmd(root: &str, args: &[&str]) -> Option<String> {
    let fut = tokio::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();
    let out = tokio::time::timeout(std::time::Duration::from_secs(5), fut)
        .await
        .ok()?
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Run the project's test command after a file edit. Returns a formatted result string.
/// Capped at 120 seconds; stdout/stderr truncated to MAX_STDOUT_CHARS.
async fn run_test_cmd(cmd: &str, cwd: &str) -> String {
    let fut = tokio::process::Command::new(if cfg!(target_os = "windows") {
        "powershell"
    } else {
        "sh"
    })
    .args(if cfg!(target_os = "windows") {
        vec!["-Command", cmd]
    } else {
        vec!["-c", cmd]
    })
    .current_dir(cwd)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .output();

    let result = match tokio::time::timeout(std::time::Duration::from_secs(120), fut).await {
        Ok(Ok(out)) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            let exit = out.status.code().unwrap_or(-1);
            (combined, exit)
        }
        Ok(Err(e)) => (format!("error running test: {e}"), -1),
        Err(_) => ("test timed out after 120s".to_string(), -1),
    };

    let (combined, exit) = result;
    let truncated = truncate_output_tail(&combined, 1200);

    if exit == 0 {
        format!("\n\n[auto-test] ✓ PASSED (exit 0)\n{truncated}")
    } else {
        format!("\n\n[auto-test] ✗ FAILED (exit {exit})\n{truncated}\nFix the test failure before proceeding.")
    }
}

// ── Agentic loop ──────────────────────────────────────────────────────────────

/// Run the agentic loop.  Sends StreamToken events to `tx` for the TUI to display.
/// The caller builds the initial messages (system + history + user).
pub async fn run_agentic(
    messages_in: Vec<ChatMessage>,
    cfg: &RunConfig,
    tool_root: Option<&str>,
    max_iters: usize,
    tx: mpsc::Sender<StreamToken>,
    project_context: Option<String>,
    agents_md: Option<String>,
    // Command to run after every successful file edit (e.g. "cargo test 2>&1").
    test_cmd: Option<String>,
    approver: &dyn Approver,
) -> Result<AgenticEndState> {
    let messages_json: Vec<serde_json::Value> = messages_in
        .iter()
        .map(|m| json!({"role": m.role, "content": m.content}))
        .collect();
    let start = AgenticStartState {
        messages: messages_json,
        checkpoint: None,
        cur_cwd: None,
        create_checkpoint: true,
    };
    run_agentic_json(
        start,
        cfg,
        tool_root,
        max_iters,
        tx,
        project_context,
        agents_md,
        test_cmd,
        None,
        approver,
    )
    .await
}

pub async fn run_agentic_json(
    start: AgenticStartState,
    cfg: &RunConfig,
    tool_root: Option<&str>,
    max_iters: usize,
    tx: mpsc::Sender<StreamToken>,
    project_context: Option<String>,
    agents_md: Option<String>,
    // Command to run after every successful file edit (e.g. "cargo test 2>&1").
    test_cmd: Option<String>,
    autosaver: Option<Arc<crate::agent_session::SessionAutoSaver>>,
    approver: &dyn Approver,
) -> Result<AgenticEndState> {
    let client = reqwest::Client::new();
    let tools = json!([
        exec_tool_def(),
        read_file_tool_def(),
        write_file_tool_def(),
        patch_file_tool_def(),
        apply_diff_tool_def(),
        search_files_tool_def(),
        list_dir_tool_def(),
        glob_tool_def(),
        done_tool_def(),
    ]);
    let mut state = AgentState::Planning;
    let mut pending_system_hint: Option<String> = None;
    let mut forced_tool_once = false;
    // C — token budget guardian
    let mut budget_warned = false;
    // D — consecutive file-tool failure escalation
    let mut file_tool_consec_failures: usize = 0;

    let root_user_text = start
        .messages
        .iter()
        .rev()
        .find(|m| m["role"].as_str() == Some("user"))
        .and_then(|m| m["content"].as_str())
        .unwrap_or("")
        .to_string();
    let goal_wants_actions = wants_local_actions(&root_user_text);

    // Keep messages as serde_json::Value throughout to preserve tool_call_id.
    let mut messages: Vec<serde_json::Value> = start.messages;
    // Rebuild loop governor memory from the existing session so resuming runs doesn't
    // repeat the same failures from scratch.
    let mut mem = FailureMemory::from_recent_messages(&messages);
    let mut recovery = RecoveryGovernor::restore_from_session(&mem, &messages);
    if recovery.in_recovery() {
        state = AgentState::Recovery;
    }

    // Resolve tool_root once (absolute path) and track cwd across tool calls.
    // This prevents the classic "cd didn't persist, so git add ran in the wrong repo" failure.
    let tool_root_abs = tool_root
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .and_then(absolutize_path);

    fn has_system_prefix(messages: &[serde_json::Value], prefix: &str) -> bool {
        messages.iter().any(|m| {
            m.get("role").and_then(|r| r.as_str()) == Some("system")
                && m.get("content")
                    .and_then(|c| c.as_str())
                    .map(|s| s.trim_start().starts_with(prefix))
                    .unwrap_or(false)
        })
    }

    let mut checkpoint = start.checkpoint.clone();
    // D — git checkpoint: snapshot HEAD so the user can /rollback if the session goes wrong.
    if let Some(ref root) = tool_root_abs {
        if checkpoint.is_none() && start.create_checkpoint {
            if let Some(hash) = git_create_checkpoint(root).await {
                checkpoint = Some(hash.clone());
                let short = hash[..hash.len().min(8)].to_string();
                let _ = tx.send(StreamToken::Checkpoint(hash)).await;
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "[git checkpoint] {short} saved — use /rollback to restore\n\n"
                    )))
                    .await;
            }
        } else if let Some(ref hash) = checkpoint {
            // Resume: re-emit checkpoint token for UI/CLI consumers.
            let _ = tx.send(StreamToken::Checkpoint(hash.clone())).await;
        }
    }

    if let Some(ref root) = tool_root_abs {
        let _ = std::fs::create_dir_all(root);
        let note = format!(
            "[Working directory]\n\
Working directory (tool_root): {root}\n\
IMPORTANT: Each exec runs in a fresh process; `cd` does NOT persist unless the tool reports cwd.\n\
 Always operate under tool_root. Create new repos under tool_root (fresh directory).\n\
 NEVER create a git repo inside another git repo. If you see 'embedded git repository', STOP and relocate."
        );
        if !has_system_prefix(&messages, "[Working directory]") {
            if messages.first().and_then(|m| m["role"].as_str()) == Some("system") {
                messages.insert(1, json!({"role":"system","content": note}));
            } else {
                messages.insert(0, json!({"role":"system","content": note}));
            }
        }
    }
    // Project context — inject once at position 2 (after [Working directory] note).
    if let Some(ctx_text) = project_context {
        if !ctx_text.is_empty() {
            if !has_system_prefix(&messages, "[Project Context") {
                let pos = messages.len().min(2);
                messages.insert(pos, json!({"role":"system","content": ctx_text}));
            }
        }
    }
    // AGENTS.md / .obstral.md — project-specific rules injected right after project context.
    // These take precedence over generic instructions and can override coding conventions.
    if let Some(agents_text) = agents_md {
        if !agents_text.is_empty() {
            if !has_system_prefix(
                &messages,
                "[Project Instructions — .obstral.md / AGENTS.md]",
            ) {
                let pos = messages.len().min(3);
                messages.insert(pos, json!({
                    "role": "system",
                    "content": format!("[Project Instructions — .obstral.md / AGENTS.md]\n{agents_text}")
                }));
            }
        }
    }

    let mut cur_cwd: Option<String> = start.cur_cwd.clone().or_else(|| tool_root_abs.clone());
    if let (Some(ref root), Some(ref cwd)) = (tool_root_abs.as_ref(), cur_cwd.as_ref()) {
        if !is_within_root(cwd, root) {
            cur_cwd = tool_root_abs.clone();
        }
    }

    // Seed the session file early so long runs can resume even if interrupted.
    autosave_best_effort(
        &autosaver,
        &tx,
        tool_root_abs.as_deref(),
        checkpoint.as_deref(),
        cur_cwd.as_deref(),
        &messages,
    )
    .await;

    // Session-scoped file read cache.
    // Key: canonical path string.  Invalidated on write_file / patch_file success.
    let mut file_cache: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    let max_iters = max_iters.max(1).min(64);
    for iter in 0..max_iters {
        // ── Prune old tool results before sending to save context tokens ───
        prune_old_tool_results(&mut messages);

        // C — Token budget guardian: warn once when context grows large.
        let approx_tokens = approx_tokens_messages(&messages);
        if !budget_warned
            && approx_tokens >= TOKEN_BUDGET_WARN_TOKENS
            && pending_system_hint.is_none()
        {
            budget_warned = true;
            pending_system_hint = Some(format!(
                "[Token Budget] Context approx {approx_tokens} tokens ({} messages).\n\
Be concise: prefer tool calls over long explanations. Summarise intermediate results in 1-2 lines.",
                messages.len()
            ));
        }

        // ── Progress checkpoint every 3 iterations ────────────────────────
        // Asks the model to self-evaluate goal distance before the next command.
        // Only fires when no higher-priority failure hint is already pending.
        if iter > 0 && iter % 3 == 0 && pending_system_hint.is_none() {
            pending_system_hint = Some(format!(
                "[Progress Check — iter {iter}/{max_iters}]\n\
 Before your next command, answer in ONE line each:\n\
 1. DONE: which steps from your <plan> are verified complete (exit_code=0)?\n\
 2. REMAINING: which steps are left?\n\
 3. ON_TRACK: yes/no — if no, re-evaluate your plan before proceeding."
            ));
        }

        // ── Final iteration handoff ─────────────────────────────────────────
        // If we hit the iteration cap, force a clean handoff via `done()` so
        // long sessions are resumable (session JSON keeps full history).
        if iter + 1 == max_iters {
            let final_hint = format!(
                "[Final Iteration — iter {}/{}]\n\
This is the LAST model call for this run.\n\
- If the task is fully done AND verified: run ONE final smoke test (if needed), then call `done`.\n\
- If the task is NOT done: call `done` with (1) verified-complete items, (2) what remains, and (3) the exact next commands/files to continue on the next run.",
                iter + 1,
                max_iters
            );
            match pending_system_hint.as_mut() {
                Some(existing) => {
                    existing.push_str("\n\n");
                    existing.push_str(&final_hint);
                }
                None => pending_system_hint = Some(final_hint),
            }
        }

        // ── Stream from model ──────────────────────────────────────────────
        // Inject a one-shot governor hint if we detected a repeated failure pattern.
        let mut msgs_for_call = messages.clone();
        if let Some(h) = pending_system_hint.take() {
            let note = format!(
                "[Loop Governor]\nstate: {:?}\nrecovery_stage: {}\n{}\n\nYou MUST incorporate this hint in your next tool call.\nDo not repeat the same failing command.",
                state,
                recovery.stage_label(),
                h
            );
            let _ = tx
                .send(StreamToken::Delta(format!("\n[governor] {h}\n")))
                .await;
            msgs_for_call.push(json!({"role":"system","content": note}));
        }

        let (token_tx, mut token_rx) = mpsc::channel::<StreamToken>(256);
        let cfg_clone = cfg.clone();
        let tools_clone = tools.clone();
        let client_clone = client.clone();
        let msgs_clone = msgs_for_call;

        let stream_task = tokio::spawn(async move {
            stream_openai_compat_json(
                &client_clone,
                &cfg_clone,
                &msgs_clone,
                Some(&tools_clone),
                token_tx,
            )
            .await
        });

        let mut assistant_text = String::new();
        let mut tool_call: Option<ToolCallData> = None;
        let mut stream_error: Option<String> = None;

        while let Some(token) = token_rx.recv().await {
            match token {
                StreamToken::Delta(s) => {
                    assistant_text.push_str(&s);
                    let _ = tx.send(StreamToken::Delta(s)).await;
                }
                StreamToken::ToolCall(tc) => {
                    tool_call = Some(tc);
                }
                StreamToken::Done => break,
                StreamToken::Error(e) => {
                    let _ = tx.send(StreamToken::Error(e.clone())).await;
                    stream_error = Some(e);
                    break;
                }
                StreamToken::Checkpoint(_) => {} // not emitted by inner stream
            }
        }

        match stream_task.await {
            Err(join_err) => {
                let msg = format!("stream task panicked: {join_err}");
                if stream_error.is_none() {
                    let _ = tx.send(StreamToken::Error(msg.clone())).await;
                }
                stream_error = Some(msg);
            }
            Ok(Err(e)) => {
                // Stream failed (network error, bad status, etc.) — surface it.
                let msg = format!("{e:#}");
                if stream_error.is_none() {
                    let _ = tx.send(StreamToken::Error(msg.clone())).await;
                }
                stream_error = Some(msg);
            }
            Ok(Ok(())) => {}
        }

        if stream_error.is_some() {
            let _ = tx
                .send(StreamToken::Delta(
                    "\n[agent] aborted due to stream error; session can be resumed.\n".to_string(),
                ))
                .await;
            break;
        }

        // ── Append assistant turn ──────────────────────────────────────────
        if let Some(ref tc) = tool_call {
            state = AgentState::Executing;
            messages.push(json!({
                "role": "assistant",
                "content": assistant_text,
                "tool_calls": [{
                    "id": tc.id,
                    "type": "function",
                    "function": {
                        "name": tc.name,
                        "arguments": tc.arguments
                    }
                }]
            }));
        } else {
            messages.push(json!({"role": "assistant", "content": assistant_text}));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;

            // Model didn't call tools. If the user asked for local actions, try implied scripts
            // (PowerShell/bash code fences) as a fallback so non-tool-calling models can still act.
            if goal_wants_actions && iter + 1 < max_iters {
                let implied = extract_implied_exec_scripts(&assistant_text);
                if !implied.is_empty() {
                    let _ = tx
                        .send(StreamToken::Delta(
                            "\n[governor] tool_call missing; executing implied commands\n"
                                .to_string(),
                        ))
                        .await;

                    for im in implied {
                        let command = im.script.trim().to_string();
                        if command.is_empty() {
                            continue;
                        }

                        state = AgentState::Executing;
                        let cwd_used: Option<String> =
                            cur_cwd.clone().or_else(|| tool_root_abs.clone());
                        let cwd_used_label = cwd_used
                            .as_deref()
                            .unwrap_or("(workspace root)")
                            .to_string();

                        let _ = tx
                            .send(StreamToken::Delta(format!(
                                "\n\n[IMPLIED_TOOL][{:?}] lang={}\n{command}\n[cwd] {cwd_used_label}\n",
                                state, im.lang_hint
                            )))
                            .await;

                        if let Some(block) =
                            should_block_git_landmines(&command, tool_root_abs.as_deref())
                        {
                            state = AgentState::Recovery;
                            let _ = tx
                                .send(StreamToken::Delta(format!(
                                    "[RESULT][Recovery] GOVERNOR BLOCK\n{block}\n"
                                )))
                                .await;

                            let cwd_line = format!("cwd: {cwd_used_label}");
                            let tool_output = inject_cwd(
                                &format!("GOVERNOR BLOCKED\n\n{block}\n\ncommand:\n{command}"),
                                &cwd_line,
                                None,
                            );

                            // NOTE: do not fabricate tool_call_id. Feed the block back as user text.
                            messages.push(json!({
                                "role": "user",
                                "content": format!(
                                    "[implied_exec]\nlang_hint: {}\ncommand:\n{}\n\n{}",
                                    im.lang_hint, command, tool_output
                                ),
                            }));
                            autosave_best_effort(
                                &autosaver,
                                &tx,
                                tool_root_abs.as_deref(),
                                checkpoint.as_deref(),
                                cur_cwd.as_deref(),
                                &messages,
                            )
                            .await;

                            // Update memory so repeating the same blocked command triggers stronger hints.
                            let _ = mem.on_tool_result(&command, "", &block, 1);
                            pending_system_hint = Some(block);
                            break;
                        }

                        let approval = approver
                            .approve(ApprovalRequest::Command {
                                command: command.clone(),
                                cwd: cwd_used.clone(),
                            })
                            .await?;
                        if approval == ApprovalOutcome::Rejected {
                            state = AgentState::Recovery;
                            let _ = tx
                                .send(StreamToken::Delta(
                                    "[RESULT][Recovery] REJECTED by user\n".to_string(),
                                ))
                                .await;

                            let cwd_line = format!("cwd: {cwd_used_label}");
                            let tool_output =
                                format!("REJECTED BY USER\n{cwd_line}\ncommand:\n{command}");
                            // NOTE: do not fabricate tool_call_id. Feed the rejection back as user text.
                            messages.push(json!({
                                "role": "user",
                                "content": format!(
                                    "[implied_exec]\nlang_hint: {}\ncommand:\n{}\n\n{}",
                                    im.lang_hint, command, tool_output
                                ),
                            }));
                            autosave_best_effort(
                                &autosaver,
                                &tx,
                                tool_root_abs.as_deref(),
                                checkpoint.as_deref(),
                                cur_cwd.as_deref(),
                                &messages,
                            )
                            .await;
                            pending_system_hint = Some(
                                "The user rejected the command. Choose a safer alternative or explain why it is necessary before retrying."
                                    .to_string(),
                            );
                            break;
                        }

                        let exec_cmd = wrap_exec_with_pwd(&command);
                        let exec_result = exec::run_command(&exec_cmd, cwd_used.as_deref()).await;
                        let (stdout, stderr, exit_code) = match exec_result {
                            Ok(r) => (r.stdout, r.stderr, r.exit_code),
                            Err(e) => (String::new(), e.to_string(), -1),
                        };

                        // Update cwd from marker output.
                        let (stdout, pwd_after) = strip_pwd_marker(&stdout);
                        let mut cwd_after_note: Option<String> = None;
                        if let Some(p) = pwd_after {
                            if let Some(ref root) = tool_root_abs {
                                if is_within_root(&p, root) {
                                    cur_cwd = Some(p);
                                } else {
                                    cwd_after_note = Some(format!(
                                        "NOTE: cwd_after was outside tool_root; ignored: {p}"
                                    ));
                                }
                            } else {
                                cur_cwd = Some(p);
                            }
                        }

                        let escaped_tool_root = cwd_after_note.is_some();
                        let cwd_after_label = cur_cwd
                            .as_deref()
                            .unwrap_or(cwd_used_label.as_str())
                            .to_string();
                        let cwd_line = if cwd_used_label == cwd_after_label {
                            format!("cwd: {cwd_used_label}")
                        } else {
                            format!("cwd: {cwd_used_label}\ncwd_after: {cwd_after_label}")
                        };

                        state = AgentState::Verifying;
                        let suspicious_reason = if exit_code == 0 {
                            suspicious_success_reason(&stdout, &stderr)
                        } else {
                            None
                        };
                        let effective_exit_code = if exit_code == 0
                            && (suspicious_reason.is_some() || escaped_tool_root)
                        {
                            1
                        } else {
                            exit_code
                        };

                        let note = cwd_after_note.as_deref();
                        let tool_output = if effective_exit_code == 0 {
                            let base = build_ok_tool_output(&stdout);
                            inject_cwd(&base, &cwd_line, note)
                        } else {
                            let mut out =
                                build_failed_tool_output(&stdout, &stderr, effective_exit_code);
                            if let Some(reason) = suspicious_reason {
                                out = format!(
                                    "NOTE: command returned exit_code=0 but was treated as failure.\nreason: {reason}\n\n{out}"
                                );
                            }
                            if escaped_tool_root && exit_code == 0 {
                                out = format!(
                                    "NOTE: command escaped tool_root and was treated as failure.\n\
This is blocked to prevent nested-repo / accidental repo-root modifications.\n\n{out}"
                                );
                            }
                            inject_cwd(&out, &cwd_line, note)
                        };

                        let result_label = if effective_exit_code == 0 {
                            format!("[RESULT][{:?}] exit=0\n", state)
                        } else {
                            format!("[RESULT][{:?}] exit={effective_exit_code} !\n", state)
                        };
                        let _ = tx.send(StreamToken::Delta(result_label)).await;

                        // NOTE: do not fabricate tool_call_id. Feed the result back as user text.
                        messages.push(json!({
                            "role": "user",
                            "content": format!(
                                "[implied_exec]\nlang_hint: {}\ncommand:\n{}\n\n{}",
                                im.lang_hint, command, tool_output
                            ),
                        }));
                        autosave_best_effort(
                            &autosaver,
                            &tx,
                            tool_root_abs.as_deref(),
                            checkpoint.as_deref(),
                            cur_cwd.as_deref(),
                            &messages,
                        )
                        .await;

                        if effective_exit_code != 0 {
                            state = AgentState::Recovery;
                        } else {
                            state = AgentState::Planning;
                        }

                        pending_system_hint = if escaped_tool_root {
                            Some(
                                "SANDBOX BREACH: Your command ended outside tool_root.\n\
Action: re-run from tool_root, avoid `cd ..` / absolute paths, and verify `pwd` stays under tool_root."
                                    .to_string(),
                            )
                        } else {
                            mem.on_tool_result(&command, &stdout, &stderr, effective_exit_code)
                        };

                        if effective_exit_code != 0 {
                            break;
                        }
                    }

                    continue;
                }
            }

            // Common failure mode: model "explains what to do" but never calls tools.
            // Try once to force a tool call so long sessions keep moving.
            if !forced_tool_once && iter + 1 < max_iters {
                forced_tool_once = true;
                state = AgentState::Recovery;
                let note = if goal_wants_actions {
                    "\
[Tool enforcement]\n\
You MUST call ONE tool now to act locally (exec/read_file/write_file/patch_file/apply_diff/search_files/list_dir/glob/done).\n\
Do NOT respond with instructions only.\n\
Start with ONE minimal safe action, then verify and continue."
                } else {
                    "\
[Tool enforcement]\n\
You responded without calling any tool.\n\
You MUST call ONE tool now (exec/read_file/write_file/patch_file/apply_diff/search_files/list_dir/glob/done).\n\
Do NOT respond with text-only instructions."
                };
                let _ = tx
                    .send(StreamToken::Delta(
                        "\n[governor] tool_call missing; forcing tool call\n".to_string(),
                    ))
                    .await;
                messages.push(json!({"role":"system","content": note}));
                autosave_best_effort(
                    &autosaver,
                    &tx,
                    tool_root_abs.as_deref(),
                    checkpoint.as_deref(),
                    cur_cwd.as_deref(),
                    &messages,
                )
                .await;
                continue;
            }

            state = AgentState::Done;
            let _ = tx
                .send(StreamToken::Delta(format!(
                    "\n[agent] state: {:?}\n",
                    state
                )))
                .await;
            break; // Model finished without tool call
        }

        // ── Execute the tool ───────────────────────────────────────────────
        let tc = tool_call.unwrap();

        // Plan gate: require a <plan> at least once before doing any real work.
        // The model should include <plan> in the same assistant message as its first tool call.
        let has_plan = messages.iter().any(|m| {
            m.get("role").and_then(|r| r.as_str()) == Some("assistant")
                && m.get("content")
                    .and_then(|c| c.as_str())
                    .map(|t| t.contains("<plan>"))
                    .unwrap_or(false)
        });
        if !has_plan {
            state = AgentState::Recovery;
            recovery.stage = Some(RecoveryStage::Diagnose);
            let block = "[Plan Gate] Missing <plan>.\n\
Required now: in your next assistant message, include a <plan> (goal/steps/risks/assumptions), then call ONE diagnostic tool (list_dir/search_files/glob/read_file) to start."
                .to_string();

            let _ = tx
                .send(StreamToken::Delta(format!(
                    "[RESULT][Recovery] GOVERNOR BLOCK\n{block}\n"
                )))
                .await;

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": format!(
                    "GOVERNOR BLOCKED\n\n{block}\n\ntool:\n{}\narguments:\n{}",
                    tc.name, tc.arguments
                ),
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;

            pending_system_hint = Some(block);
            continue;
        }

        // ── done tool ──────────────────────────────────────────────────────
        if tc.name.as_str() == "done" {
            let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
            let summary = args["summary"].as_str().unwrap_or("").trim();
            let next_steps = args["next_steps"].as_str().unwrap_or("").trim();

            let mut final_text = String::new();
            final_text.push_str("[DONE]\n");
            if !summary.is_empty() {
                final_text.push_str(summary);
            }
            if !next_steps.is_empty() {
                final_text.push_str("\n\nNext:\n");
                final_text.push_str(next_steps);
            }

            // Close out the tool call so session JSON remains valid on resume.
            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": "OK: done"
            }));
            messages.push(json!({"role": "assistant", "content": final_text.clone()}));

            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;

            let _ = tx
                .send(StreamToken::Delta(format!("\n\n{final_text}\n")))
                .await;
            break;
        }

        // ── apply_diff tool ───────────────────────────────────────────────
        // Recovery gate: while recovering from failures, enforce a strict
        // Diagnose -> Fix -> Verify workflow to prevent phase drift.
        if let Some(block) = recovery.maybe_block_tool(&tc, test_cmd.as_deref()) {
            state = AgentState::Recovery;
            let _ = tx
                .send(StreamToken::Delta(format!(
                    "[RESULT][Recovery] GOVERNOR BLOCK\n{block}\n"
                )))
                .await;

            let cwd_label = cur_cwd
                .as_deref()
                .or(tool_root_abs.as_deref())
                .unwrap_or("(workspace root)")
                .to_string();
            let tool_output = inject_cwd(
                &format!(
                    "GOVERNOR BLOCKED\n\n{block}\n\ntool:\n{}\narguments:\n{}",
                    tc.name, tc.arguments
                ),
                &format!("cwd: {cwd_label}"),
                None,
            );

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": tool_output,
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;

            pending_system_hint = Some(block);
            continue;
        }

        if tc.name.as_str() == "apply_diff" {
            let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
            let path = args["path"].as_str().unwrap_or("").to_string();
            let diff = args["diff"].as_str().unwrap_or("").to_string();

            let _ = tx
                .send(StreamToken::Delta(format!("\n\n[APPLY_DIFF] {path}\n")))
                .await;
            let _ = tx.send(StreamToken::ToolCall(tc.clone())).await;

            let base = tool_root_abs.as_deref();
            let mut rejected_by_user = false;

            // B — capture old content for diff anchoring
            let old_for_cache = base.and_then(|b| {
                crate::file_tools::resolve_safe_path(&path, Some(b))
                    .ok()
                    .and_then(|abs| std::fs::read_to_string(&abs).ok())
            });

            let preview = truncate_preview(&diff, 2800, 140);
            let approval = approver
                .approve(ApprovalRequest::Edit {
                    action: "apply_diff".to_string(),
                    path: path.clone(),
                    preview,
                })
                .await?;
            let (mut result, is_error) = if approval == ApprovalOutcome::Approved {
                crate::file_tools::tool_apply_diff(&path, &diff, base)
            } else {
                rejected_by_user = true;
                (
                    format!(
                        "REJECTED BY USER\naction: apply_diff\npath: {path}\n(no changes applied)"
                    ),
                    true,
                )
            };

            // Invalidate file cache on success + auto-test
            if !is_error {
                let cache_key = crate::file_tools::resolve_safe_path(&path, base)
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.clone());
                file_cache.remove(&cache_key);
                let before_hash = old_for_cache.as_deref().map(hash_text);
                // Seed cache with new content + hash verification
                if let Ok(abs) = crate::file_tools::resolve_safe_path(&path, base) {
                    if let Ok(new_content) = std::fs::read_to_string(&abs) {
                        let after_hash = hash_text(&new_content);
                        match before_hash {
                            Some(bh) => result.push_str(&format!(
                                "\n[hash] before={} after={}",
                                fmt_hash(bh),
                                fmt_hash(after_hash)
                            )),
                            None => result
                                .push_str(&format!("\n[hash] after={}", fmt_hash(after_hash))),
                        }
                        file_cache.insert(cache_key, new_content);
                    }
                }
                // A — auto-run tests
                if let Some(ref cmd) = test_cmd {
                    if let Some(ref root) = tool_root_abs {
                        let test_out = run_test_cmd(cmd, root).await;
                        result.push_str(&test_out);
                    }
                }
            }

            let verified = result.contains("PASSED (exit 0)");
            if is_error {
                recovery.on_fix_result(false, false);
            } else {
                recovery.on_fix_result(true, verified);
            }

            let first_line = result.lines().next().unwrap_or("").to_string();
            if is_error {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "[RESULT_FILE_ERR] {first_line}\n"
                    )))
                    .await;
                state = AgentState::Recovery;
                if !rejected_by_user {
                    file_tool_consec_failures += 1;
                    pending_system_hint = Some(format!("apply_diff error: {first_line}"));
                } else {
                    pending_system_hint = Some(
                        "The user rejected the edit. Choose a safer alternative or ask again with a smaller change."
                            .to_string(),
                    );
                }
            } else {
                let _ = tx
                    .send(StreamToken::Delta(format!("[RESULT_FILE] {first_line}\n")))
                    .await;
                state = if recovery.in_recovery() {
                    AgentState::Recovery
                } else {
                    AgentState::Planning
                };
                file_tool_consec_failures = 0;
                pending_system_hint = if recovery.stage == Some(RecoveryStage::Verify) {
                    Some("Recovery stage=verify: run ONE verification command (tests or `git status`) before continuing.".to_string())
                } else {
                    None
                };
            }

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result,
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;
            continue;
        }

        // ── glob tool ─────────────────────────────────────────────────────
        // ── list_dir tool ────────────────────────────────────────────────────
        if tc.name.as_str() == "list_dir" {
            let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
            let dir = args["dir"].as_str().unwrap_or("").to_string();
            let max_entries = args["max_entries"].as_u64().unwrap_or(200) as usize;
            let include_hidden = args["include_hidden"].as_bool().unwrap_or(false);

            let dir_label = if dir.trim().is_empty() { "." } else { dir.as_str() };
            let _ = tx
                .send(StreamToken::Delta(format!("\n\n[LIST_DIR] {dir_label}\n")))
                .await;
            let _ = tx.send(StreamToken::ToolCall(tc.clone())).await;

            let base = tool_root_abs.as_deref();
            let (result, is_error) =
                crate::file_tools::tool_list_dir(&dir, max_entries, include_hidden, base);
            recovery.on_diagnostic_result(!is_error);

            let first_line = result.lines().next().unwrap_or("").to_string();
            if is_error {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "[RESULT_FILE_ERR] {first_line}\n"
                    )))
                    .await;
                state = AgentState::Recovery;
            } else {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "[RESULT_LIST_DIR] {first_line}\n"
                    )))
                    .await;
                state = if recovery.in_recovery() {
                    AgentState::Recovery
                } else {
                    AgentState::Planning
                };
                pending_system_hint = if recovery.stage == Some(RecoveryStage::Fix) {
                    Some("Recovery stage=fix: apply a minimal fix now (edit files or run a corrected command).".to_string())
                } else if recovery.stage == Some(RecoveryStage::Verify) {
                    Some("Recovery stage=verify: run ONE verification command (tests or `git status`) before continuing.".to_string())
                } else {
                    None
                };
            }

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result,
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;
            continue;
        }

        if tc.name.as_str() == "glob" {
            let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
            let pattern = args["pattern"].as_str().unwrap_or("").to_string();
            let dir = args["dir"].as_str().unwrap_or("").to_string();

            let _ = tx
                .send(StreamToken::Delta(format!("\n\n[GLOB] {pattern}\n")))
                .await;
            let _ = tx.send(StreamToken::ToolCall(tc.clone())).await;

            let base = tool_root_abs.as_deref();
            let (result, is_error) = crate::file_tools::tool_glob_files(&pattern, &dir, base);
            recovery.on_diagnostic_result(!is_error);

            let first_line = result.lines().next().unwrap_or("").to_string();
            if is_error {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "[RESULT_FILE_ERR] {first_line}\n"
                    )))
                    .await;
                state = AgentState::Recovery;
            } else {
                let _ = tx
                    .send(StreamToken::Delta(format!("[RESULT_GLOB] {first_line}\n")))
                    .await;
                state = if recovery.in_recovery() {
                    AgentState::Recovery
                } else {
                    AgentState::Planning
                };
                pending_system_hint = if recovery.stage == Some(RecoveryStage::Fix) {
                    Some("Recovery stage=fix: apply a minimal fix now (edit files or run a corrected command).".to_string())
                } else if recovery.stage == Some(RecoveryStage::Verify) {
                    Some("Recovery stage=verify: run ONE verification command (tests or `git status`) before continuing.".to_string())
                } else {
                    None
                };
            }

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result,
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;
            continue;
        }

        // ── search_files tool ─────────────────────────────────────────────
        if tc.name.as_str() == "search_files" {
            let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
            let pattern = args["pattern"].as_str().unwrap_or("").to_string();
            let dir = args["dir"].as_str().unwrap_or("").to_string();
            let ci = args["case_insensitive"].as_bool().unwrap_or(false);

            let _ = tx
                .send(StreamToken::Delta(format!(
                    "\n\n[SEARCH_FILES] {pattern}\n"
                )))
                .await;
            let _ = tx.send(StreamToken::ToolCall(tc.clone())).await;

            let base = tool_root_abs.as_deref();
            let (result, is_error) = crate::file_tools::tool_search_files(&pattern, &dir, ci, base);
            recovery.on_diagnostic_result(!is_error);

            let first_line = result.lines().next().unwrap_or("").to_string();
            if is_error {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "[RESULT_FILE_ERR] {first_line}\n"
                    )))
                    .await;
                state = AgentState::Recovery;
            } else {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "[RESULT_SEARCH] {first_line}\n"
                    )))
                    .await;
                state = if recovery.in_recovery() {
                    AgentState::Recovery
                } else {
                    AgentState::Planning
                };
                pending_system_hint = if recovery.stage == Some(RecoveryStage::Fix) {
                    Some("Recovery stage=fix: apply a minimal fix now (edit files or run a corrected command).".to_string())
                } else if recovery.stage == Some(RecoveryStage::Verify) {
                    Some("Recovery stage=verify: run ONE verification command (tests or `git status`) before continuing.".to_string())
                } else {
                    None
                };
            }

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result,
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;

            continue;
        }

        // ── File tools: read_file / write_file / patch_file ────────────────
        if matches!(tc.name.as_str(), "read_file" | "write_file" | "patch_file") {
            let args: serde_json::Value = serde_json::from_str(&tc.arguments).unwrap_or(json!({}));
            let path = args["path"].as_str().unwrap_or("").to_string();

            // Emit annotation (visible in TUI).
            let tool_upper = tc.name.to_ascii_uppercase();
            let _ = tx
                .send(StreamToken::Delta(format!("\n\n[{tool_upper}] {path}\n")))
                .await;
            let _ = tx.send(StreamToken::ToolCall(tc.clone())).await;

            let base = tool_root_abs.as_deref();

            // Cache key: canonical absolute path string.
            let cache_key = crate::file_tools::resolve_safe_path(&path, base)
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| format!("{}|{}", path, base.unwrap_or("")));
            let mut rejected_by_user = false;

            let (result, is_error) = match tc.name.as_str() {
                "read_file" => {
                    // ── Gap 6: serve from cache if file hasn't changed ──────
                    if let Some(cached) = file_cache.get(&cache_key) {
                        let header = cached.lines().next().unwrap_or(&path).to_string();
                        let _ = tx
                            .send(StreamToken::Delta(format!("[CACHE_HIT] {header}\n")))
                            .await;
                        (
                            format!(
                                "{} [⚡ cached — unchanged since last read]\n{cached}",
                                header
                            ),
                            false,
                        )
                    } else {
                        let (content, err) = crate::file_tools::tool_read_file(&path, base);
                        if !err {
                            file_cache.insert(cache_key.clone(), content.clone());
                        }
                        (content, err)
                    }
                }
                "write_file" => {
                    let content = args["content"].as_str().unwrap_or("").to_string();

                    // Safety: do not overwrite an existing file unless we already read it in this session.
                    // This prevents accidental destructive writes when the agent guessed the path/content.
                    let file_exists = crate::file_tools::resolve_safe_path(&path, base)
                        .ok()
                        .and_then(|abs| abs.metadata().ok())
                        .map(|m| m.is_file())
                        .unwrap_or(false);
                    if file_exists && !file_cache.contains_key(&cache_key) {
                        (
                            format!(
                                "GOVERNOR BLOCKED: write_file refused because '{path}' already exists.\n\
Action required: call read_file(path) first to confirm current contents, then retry with patch_file/apply_diff (preferred) or write_file."
                            ),
                            true,
                        )
                    } else {
                    // Approval: show a compact before/after preview.
                    let old = file_cache
                        .get(&cache_key)
                        .cloned()
                        .or_else(|| {
                            crate::file_tools::resolve_safe_path(&path, base)
                                .ok()
                                .and_then(|abs| std::fs::read_to_string(&abs).ok())
                        })
                        .unwrap_or_default();
                    let preview = simple_before_after(&old, &content);
                    let before_hash = hash_text(&old);
                    let after_hash = hash_text(&content);
                    let approval = approver
                        .approve(ApprovalRequest::Edit {
                            action: "write_file".to_string(),
                            path: path.clone(),
                            preview,
                        })
                        .await?;
                    if approval == ApprovalOutcome::Rejected {
                        rejected_by_user = true;
                        (
                            format!(
                                "REJECTED BY USER\naction: write_file\npath: {path}\n(no changes applied)"
                            ),
                            true,
                        )
                    } else {
                        let (mut r_text, r_err) =
                            crate::file_tools::tool_write_file(&path, &content, base);
                        if !r_err {
                            file_cache.insert(cache_key.clone(), content.clone());
                            r_text.push_str(&format!(
                                "\n[hash] before={} after={}",
                                fmt_hash(before_hash),
                                fmt_hash(after_hash)
                            ));
                            // A — auto-test after write
                            if let Some(ref cmd) = test_cmd {
                                if let Some(ref root) = tool_root_abs {
                                    r_text.push_str(&run_test_cmd(cmd, root).await);
                                }
                            }
                        }
                        (r_text, r_err)
                    }
                    }
                }
                _ => {
                    // ── patch_file ─────────────────────────────────────────
                    let search = args["search"].as_str().unwrap_or("").to_string();
                    let replace = args["replace"].as_str().unwrap_or("").to_string();

                    // B — capture old content for diff preview before patching.
                    let old_content_for_diff = file_cache.get(&cache_key).cloned().or_else(|| {
                        crate::file_tools::resolve_safe_path(&path, base)
                            .ok()
                            .and_then(|abs| std::fs::read_to_string(&abs).ok())
                    });
                    let before_hash = old_content_for_diff
                        .as_deref()
                        .map(hash_text)
                        .unwrap_or_else(|| hash_text(""));

                    // Approval: show the computed patch diff (or search/replace when diff can't be made).
                    let mut preview = old_content_for_diff
                        .as_deref()
                        .map(|old| crate::file_tools::make_patch_diff(old, &search, &replace))
                        .unwrap_or_default();
                    if preview.trim().is_empty() {
                        let s = truncate_preview(&search, 700, 28);
                        let r = truncate_preview(&replace, 700, 28);
                        preview = format!(
                            "(no context diff available)\n--- search ---\n{s}\n\n--- replace ---\n{r}\n"
                        );
                    }
                    let approval = approver
                        .approve(ApprovalRequest::Edit {
                            action: "patch_file".to_string(),
                            path: path.clone(),
                            preview,
                        })
                        .await?;
                    if approval == ApprovalOutcome::Rejected {
                        rejected_by_user = true;
                        (
                            format!(
                                "REJECTED BY USER\naction: patch_file\npath: {path}\n(no changes applied)"
                            ),
                            true,
                        )
                    } else {
                        let (mut patch_result, patch_err) =
                            crate::file_tools::tool_patch_file(&path, &search, &replace, base);

                        if !patch_err {
                            file_cache.remove(&cache_key); // invalidate stale cache

                            // B — append diff preview (shows exactly what changed).
                            if let Some(ref old) = old_content_for_diff {
                                let diff =
                                    crate::file_tools::make_patch_diff(old, &search, &replace);
                                if !diff.is_empty() {
                                    patch_result.push_str(&format!("\n{diff}"));
                                }
                            }

                            // Gap 7: auto-verify patch was applied correctly.
                            if let Ok(abs) = crate::file_tools::resolve_safe_path(&path, base) {
                                if let Ok(new_content) = std::fs::read_to_string(&abs) {
                                    let after_hash = hash_text(&new_content);
                                    patch_result.push_str(&format!(
                                        "\n[hash] before={} after={}",
                                        fmt_hash(before_hash),
                                        fmt_hash(after_hash)
                                    ));
                                    if replace.is_empty() || new_content.contains(&replace) {
                                        patch_result
                                            .push_str("\n✓ auto-verify: patch confirmed in file");
                                    } else {
                                        patch_result.push_str(
                                            "\n✗ auto-verify FAILED: replacement text not found — \
                                             file may be in unexpected state; call read_file to inspect",
                                        );
                                    }
                                    // Seed cache with the freshly written content.
                                    file_cache.insert(cache_key.clone(), new_content);
                                }
                            }
                            // A — auto-test after successful patch
                            if let Some(ref cmd) = test_cmd {
                                if let Some(ref root) = tool_root_abs {
                                    patch_result.push_str(&run_test_cmd(cmd, root).await);
                                }
                            }
                        }
                        (patch_result, patch_err)
                    }
                }
            };

            // D — track consecutive file-tool failures for escalation.
            if is_error {
                if !rejected_by_user {
                    file_tool_consec_failures += 1;
                }
            } else {
                file_tool_consec_failures = 0;
            }

            let verified = result.contains("PASSED (exit 0)");
            match tc.name.as_str() {
                "read_file" => recovery.on_diagnostic_result(!is_error),
                "write_file" | "patch_file" => {
                    if is_error {
                        recovery.on_fix_result(false, false);
                    } else {
                        recovery.on_fix_result(true, verified);
                    }
                }
                _ => {}
            }

            // Emit result label.
            let first_line = result.lines().next().unwrap_or("").to_string();
            if is_error {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "[RESULT_FILE_ERR] {first_line}\n"
                    )))
                    .await;
                state = AgentState::Recovery;
                // D — escalate after 3 consecutive file-tool failures.
                let hint = if rejected_by_user {
                    "The user rejected the edit. Choose a safer alternative or ask again with a smaller change."
                        .to_string()
                } else if file_tool_consec_failures >= 3 {
                    format!(
                        "CRITICAL: {file_tool_consec_failures} consecutive file-tool errors.\n\
                         You MUST abandon the current approach. Do NOT retry the same operation.\n\
                         Instead: call read_file to inspect the actual file state, then choose \
                         a completely different strategy (e.g. write_file instead of patch_file)."
                    )
                } else {
                    format!(
                        "File tool error: {first_line}\n\
                         Read the error message carefully and fix the issue before proceeding."
                    )
                };
                pending_system_hint = Some(hint);
            } else {
                let _ = tx
                    .send(StreamToken::Delta(format!("[RESULT_FILE] {first_line}\n")))
                    .await;
                state = if recovery.in_recovery() {
                    AgentState::Recovery
                } else {
                    AgentState::Planning
                };
                pending_system_hint = if recovery.stage == Some(RecoveryStage::Fix) {
                    Some("Recovery stage=fix: apply a minimal fix now (edit files or run a corrected command).".to_string())
                } else if recovery.stage == Some(RecoveryStage::Verify) {
                    Some("Recovery stage=verify: run ONE verification command (tests or `git status`) before continuing.".to_string())
                } else {
                    None
                };
            }

            // Append tool result to conversation.
            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result,
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;

            if iter + 1 == max_iters {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "\n[agent] iteration cap ({max_iters}) reached.\n"
                    )))
                    .await;
            }
            continue; // skip exec block below
        }

        if tc.name != "exec" {
            return Err(anyhow!("unknown tool: {}", tc.name));
        }

        let args: serde_json::Value =
            serde_json::from_str(&tc.arguments).unwrap_or(json!({"command": tc.arguments}));
        let command = args["command"].as_str().unwrap_or("").to_string();

        // Resolve an optional cwd (absolute or relative). Relative cwd is resolved against
        // the current tracked directory (or tool_root).
        let mut cwd_note: Option<String> = None;
        let cwd_used: Option<String> = if let Some(c0) = args["cwd"]
            .as_str()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            let pb = std::path::PathBuf::from(c0);
            let candidate = if pb.is_absolute() {
                pb
            } else if let Some(base) = cur_cwd.as_deref().or(tool_root_abs.as_deref()) {
                std::path::PathBuf::from(base).join(pb)
            } else {
                pb
            };
            let cand_str = candidate.to_string_lossy().into_owned();
            if let Some(ref root) = tool_root_abs {
                if is_within_root(&cand_str, root) {
                    Some(cand_str)
                } else {
                    cwd_note = Some(format!(
                        "NOTE: requested cwd is outside tool_root; ignoring: {c0}"
                    ));
                    cur_cwd.clone().or_else(|| tool_root_abs.clone())
                }
            } else {
                Some(cand_str)
            }
        } else {
            cur_cwd.clone().or_else(|| tool_root_abs.clone())
        };
        let cwd_used_label = cwd_used
            .as_deref()
            .unwrap_or("(workspace root)")
            .to_string();

        // Emit tool-call annotation to TUI (dim line the UI will colour differently).
        let _ = tx
            .send(StreamToken::Delta(format!(
                "\n\n[TOOL][{:?}] {command}\n[cwd] {cwd_used_label}\n",
                state
            )))
            .await;

        if let Some(block) = should_block_git_landmines(&command, tool_root_abs.as_deref()) {
            state = AgentState::Recovery;
            recovery.on_exec_result(ExecKind::Action, false);
            let _ = tx
                .send(StreamToken::Delta(format!(
                    "[RESULT][Recovery] GOVERNOR BLOCK\n{block}\n"
                )))
                .await;

            let cwd_line = format!("cwd: {cwd_used_label}");
            let tool_output = inject_cwd(
                &format!("GOVERNOR BLOCKED\n\n{block}\n\ncommand:\n{command}"),
                &cwd_line,
                cwd_note.as_deref(),
            );

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": tool_output,
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;

            // Update memory so repeating the same blocked command triggers stronger hints.
            let _ = mem.on_tool_result(&command, "", &block, 1);
            pending_system_hint = Some(block);

            if iter + 1 == max_iters {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "\n[agent] iteration cap ({max_iters}) reached.\n"
                    )))
                    .await;
            }
            continue;
        }

        let approval = approver
            .approve(ApprovalRequest::Command {
                command: command.clone(),
                cwd: cwd_used.clone(),
            })
            .await?;
        if approval == ApprovalOutcome::Rejected {
            state = AgentState::Recovery;
            recovery.on_exec_result(ExecKind::Action, false);
            let _ = tx
                .send(StreamToken::Delta(
                    "[RESULT][Recovery] REJECTED by user\n".to_string(),
                ))
                .await;
            let cwd_line = format!("cwd: {cwd_used_label}");
            let tool_output = format!("REJECTED BY USER\n{cwd_line}\ncommand:\n{command}");

            messages.push(json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": tool_output,
            }));
            autosave_best_effort(
                &autosaver,
                &tx,
                tool_root_abs.as_deref(),
                checkpoint.as_deref(),
                cur_cwd.as_deref(),
                &messages,
            )
            .await;
            pending_system_hint = Some(
                "The user rejected the command. Choose a safer alternative or explain why it is necessary before retrying."
                    .to_string(),
            );

            if iter + 1 == max_iters {
                let _ = tx
                    .send(StreamToken::Delta(format!(
                        "\n[agent] iteration cap ({max_iters}) reached.\n"
                    )))
                    .await;
            }
            continue;
        }

        let exec_cmd = wrap_exec_with_pwd(&command);
        let exec_result = exec::run_command(&exec_cmd, cwd_used.as_deref()).await;

        let (stdout, stderr, exit_code) = match exec_result {
            Ok(r) => (r.stdout, r.stderr, r.exit_code),
            Err(e) => (String::new(), e.to_string(), -1),
        };

        // Update cwd from marker output.
        let (stdout, pwd_after) = strip_pwd_marker(&stdout);
        let mut cwd_after_note: Option<String> = None;
        if let Some(p) = pwd_after {
            if let Some(ref root) = tool_root_abs {
                if is_within_root(&p, root) {
                    cur_cwd = Some(p);
                } else {
                    cwd_after_note = Some(format!(
                        "NOTE: cwd_after was outside tool_root; ignored: {p}"
                    ));
                }
            } else {
                cur_cwd = Some(p);
            }
        }
        let escaped_tool_root = cwd_after_note.is_some();
        let cwd_after_label = cur_cwd
            .as_deref()
            .unwrap_or(cwd_used_label.as_str())
            .to_string();
        let cwd_line = if cwd_used_label == cwd_after_label {
            format!("cwd: {cwd_used_label}")
        } else {
            format!("cwd: {cwd_used_label}\ncwd_after: {cwd_after_label}")
        };

        let mut note_lines: Vec<String> = Vec::new();
        if let Some(n) = cwd_note.take() {
            note_lines.push(n);
        }
        if let Some(n) = cwd_after_note.take() {
            note_lines.push(n);
        }
        let note = if note_lines.is_empty() {
            None
        } else {
            Some(note_lines.join("\n"))
        };

        state = AgentState::Verifying;
        let suspicious_reason = if exit_code == 0 {
            suspicious_success_reason(&stdout, &stderr)
        } else {
            None
        };
        let effective_exit_code =
            if exit_code == 0 && (suspicious_reason.is_some() || escaped_tool_root) {
                1
            } else {
                exit_code
            };

        let tool_output = if effective_exit_code == 0 {
            let base = build_ok_tool_output(&stdout);
            inject_cwd(&base, &cwd_line, note.as_deref())
        } else {
            let mut out = build_failed_tool_output(&stdout, &stderr, effective_exit_code);
            if let Some(reason) = suspicious_reason {
                out = format!(
                    "NOTE: command returned exit_code=0 but was treated as failure.\nreason: {reason}\n\n{out}"
                );
            }
            if escaped_tool_root && exit_code == 0 {
                out = format!(
                    "NOTE: command escaped tool_root and was treated as failure.\n\
This is blocked to prevent nested-repo / accidental repo-root modifications.\n\n{out}"
                );
            }
            inject_cwd(&out, &cwd_line, note.as_deref())
        };

        let result_label = if effective_exit_code == 0 {
            format!("[RESULT][{:?}] exit=0\n", state)
        } else {
            format!("[RESULT][{:?}] exit={effective_exit_code} !\n", state)
        };
        let _ = tx.send(StreamToken::Delta(result_label)).await;

        // ── Append tool result (with tool_call_id preserved) ───────────────
        messages.push(json!({
            "role": "tool",
            "tool_call_id": tc.id,
            "content": tool_output,
        }));
        autosave_best_effort(
            &autosaver,
            &tx,
            tool_root_abs.as_deref(),
            checkpoint.as_deref(),
            cur_cwd.as_deref(),
            &messages,
        )
        .await;

        // Update failure memory + recovery governor + possibly inject a system hint.
        let exec_kind = classify_exec_kind(&command, test_cmd.as_deref());
        let mut hint = if escaped_tool_root {
            Some(
                "SANDBOX BREACH: Your command ended outside tool_root.\n\
Action: re-run from tool_root, avoid `cd ..` / absolute paths, and verify `pwd` stays under tool_root."
                    .to_string(),
            )
        } else {
            mem.on_tool_result(&command, &stdout, &stderr, effective_exit_code)
        };

        recovery.on_exec_result(exec_kind, effective_exit_code == 0 && !escaped_tool_root);

        if effective_exit_code == 0 && !escaped_tool_root {
            if recovery.stage == Some(RecoveryStage::Fix) {
                hint = Some(
                    "Recovery stage=fix: apply a minimal fix now (edit files or run a corrected command)."
                        .to_string(),
                );
            } else if recovery.stage == Some(RecoveryStage::Verify) {
                hint = Some(
                    "Recovery stage=verify: run ONE verification command (tests or `git status`) before continuing."
                        .to_string(),
                );
            }
        }

        pending_system_hint = hint;
        state = if effective_exit_code != 0 || recovery.in_recovery() {
            AgentState::Recovery
        } else {
            AgentState::Planning
        };

        // Safety: stop if we've hit the iteration cap.
        if iter + 1 == max_iters {
            let _ = tx
                .send(StreamToken::Delta(format!(
                    "\n[agent] iteration cap ({max_iters}) reached.\n"
                )))
                .await;
        }
    }

    let _ = tx.send(StreamToken::Done).await;
    Ok(AgenticEndState {
        messages,
        checkpoint,
        cur_cwd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_git_init_inside_existing_repo() {
        let td = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(td.path().join(".git")).expect("mkdir .git");
        let root = td.path().to_string_lossy();
        let msg = should_block_git_landmines("git init Foo", Some(root.as_ref()))
            .expect("expected block");
        assert!(msg.to_ascii_lowercase().contains("refusing"));
    }

    #[test]
    fn blocks_git_add_when_nested_repo_detected() {
        let td = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(td.path().join(".git")).expect("mkdir .git");
        std::fs::create_dir_all(td.path().join("MazeGame").join(".git")).expect("mkdir nested");
        let root = td.path().to_string_lossy();
        let msg =
            should_block_git_landmines("git add -A", Some(root.as_ref())).expect("expected block");
        assert!(msg.contains("MazeGame"));
    }

    #[test]
    fn allows_git_add_when_nested_is_submodule() {
        let td = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(td.path().join(".git")).expect("mkdir .git");
        std::fs::create_dir_all(td.path().join("MazeGame").join(".git")).expect("mkdir nested");
        std::fs::write(
            td.path().join(".gitmodules"),
            "[submodule \"MazeGame\"]\n\tpath = MazeGame\n\turl = https://example.invalid/MazeGame.git\n",
        )
        .expect("write .gitmodules");
        let root = td.path().to_string_lossy();
        assert!(
            should_block_git_landmines("git add -A", Some(root.as_ref())).is_none(),
            "should not block when nested repo is declared as submodule"
        );
    }

    #[test]
    fn injects_git_proxy_hint_for_github_via_localhost() {
        let stderr = "fatal: unable to access 'https://github.com/x/y.git/': Failed to connect to github.com port 443 via 127.0.0.1 after 2041 ms: Could not connect to server";
        let out = build_failed_tool_output("", stderr, 1);
        assert!(
            out.contains("ssh.github.com"),
            "should suggest ssh-over-443"
        );
        assert!(
            out.contains("HTTP_PROXY") || out.contains("HTTPS_PROXY"),
            "should mention proxy env vars"
        );
    }

    #[test]
    fn injects_cargo_exe_lock_hint_on_windows_style_error() {
        let stderr = "error: failed to remove file `C:\\\\Users\\\\user\\\\observistral\\\\target\\\\debug\\\\obstral.exe`\nCaused by: Access is denied. (os error 5)";
        let out = build_failed_tool_output("", stderr, 1);
        assert!(
            out.to_ascii_lowercase().contains("cargo_target_dir"),
            "should suggest isolated target dir"
        );
    }

    #[test]
    fn rebuilds_failure_memory_from_session_messages() {
        let messages = vec![
            json!({"role":"system","content":"sys"}),
            json!({"role":"user","content":"do thing"}),
            json!({"role":"assistant","content":"","tool_calls":[{"id":"call_1","type":"function","function":{"name":"exec","arguments":"{\"command\":\"git status\"}"}}]}),
            json!({"role":"tool","tool_call_id":"call_1","content":"FAILED (exit_code: 1)\nstderr:\nfatal: nope"}),
            json!({"role":"assistant","content":"","tool_calls":[{"id":"call_2","type":"function","function":{"name":"exec","arguments":"{\"command\":\"git status\"}"}}]}),
            json!({"role":"tool","tool_call_id":"call_2","content":"FAILED (exit_code: 1)\nstderr:\nfatal: nope"}),
        ];

        let mem = FailureMemory::from_recent_messages(&messages);
        assert_eq!(mem.consecutive_failures, 2);
        assert_eq!(mem.same_command_repeats, 2);
        assert_eq!(mem.same_error_repeats, 2);
    }
}
