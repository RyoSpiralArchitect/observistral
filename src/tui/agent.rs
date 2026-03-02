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
use anyhow::{Result, anyhow};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::sync::mpsc;

use crate::config::RunConfig;
use crate::exec;
use crate::streaming::{StreamToken, ToolCallData, stream_openai_compat_json};
use crate::types::ChatMessage;

// ── Tunables ─────────────────────────────────────────────────────────────────

const MAX_ITERS: usize = 12;
const MAX_STDOUT_CHARS: usize = 1500;
const MAX_STDERR_CHARS: usize = 600;
const KEEP_RECENT_TOOL_TURNS: usize = 4;

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
[Planning Protocol — emit ONCE before your very first exec call]\n\
<plan>\n\
goal: <one sentence: what the finished task looks like when done>\n\
steps: 1) ... 2) ... 3) ... (3-7 concrete, ordered steps)\n\
risks: <the 2 most likely failure modes for this specific task>\n\
assumptions: <what you are taking as given>\n\
</plan>\n\
\n\
[Reasoning Protocol — emit before EVERY exec call]\n\
<think>\n\
goal:   <≤12 words: what must succeed right now>\n\
risk:   <≤12 words: most likely failure mode>\n\
doubt:  <≤12 words: one reason this approach could be wrong>\n\
next:   <≤12 words: exact command or step>\n\
verify: <≤12 words: how to confirm this step succeeded>\n\
</think>\n\
Keep each field under 15 words. This 5-line check (~50 tokens) prevents\n\
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

// ── System prompt builders ────────────────────────────────────────────────────

/// Fixed base for the TUI Coder pane — always an agentic executor, not a chat bot.
const CODER_BASE_SYSTEM: &str = "\
You are an autonomous coding agent with an exec tool that runs shell commands.\n\
\n\
RULE: You MUST call exec on every single turn. Never respond with text only.\n\
If you need to create a file — call exec. If you need to run code — call exec.\n\
If you are done — call exec to verify the result, then say done.\n\
\n\
File creation (Windows PowerShell):\n\
  New-Item -ItemType Directory -Path 'src' -Force\n\
  @('print(\"hello\")') | Set-Content -Path 'main.py' -Encoding UTF8\n\
\n\
File creation (Unix sh):\n\
  mkdir -p src && cat > main.py << 'EOF'\n\
  print(\"hello\")\n\
  EOF\n\
\n\
After every file write: verify with Get-Content or cat.\n\
After every build/test: confirm exit_code == 0 before proceeding.\n\
\n\
When ALL steps from your <plan> are verified complete:\n\
  call exec one final time to run a smoke test or confirm the deliverable exists,\n\
  then reply with a brief [DONE] summary: what was built, where it lives, how to run it.\n\
  Do NOT reply with [DONE] while any command is still failing.";

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
            // We only prune successful tool outputs we generated ("OK (exit_code: 0) ...").
            if !content.trim_start().starts_with("OK (exit_code: 0)") {
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
        || (low.contains("could not find")
            && (low.contains("package") || low.contains("crate")))
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

// ── Tool output builders ──────────────────────────────────────────────────────

/// Build the tool result string for a failed command with structured
/// diagnosis guidance.  Forces the model to reason about the error rather
/// than blindly retrying or continuing.
fn build_failed_tool_output(stdout: &str, stderr: &str, exit_code: i32) -> String {
    let class = classify_error(stderr, stdout);
    let hint = error_class_hint(&class);
    let hint_prefix = if hint.is_empty() {
        String::new()
    } else {
        format!("{hint}\n\n")
    };
    let mut out = format!(
        "{hint_prefix}FAILED (exit_code: {exit_code})\n\
         \n\
         ⚠ STOP — diagnosis required before your next action:\n\
         1. Quote the exact line causing the error.\n\
         2. Identify the root cause in one sentence.\n\
         3. Fix it with a single corrected command.\n\
         Do NOT continue the original plan until the fix succeeds.\n"
    );
    let stdout_t = truncate_output(stdout, MAX_STDOUT_CHARS);
    let stderr_t = truncate_output(stderr, MAX_STDERR_CHARS);
    if !stdout_t.is_empty() {
        out.push_str(&format!("\nstdout:\n{stdout_t}\n"));
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
    let p = normalize_path_sep(path).replace('\u{0}', "").trim().trim_end_matches('/').to_string();
    let r = normalize_path_sep(root).replace('\u{0}', "").trim().trim_end_matches('/').to_string();
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
    if low.contains("unexpected token '}'") || (low.contains("unexpected token") && low.contains('}')) {
        return Some(
            "PowerShell saw a stray `}` in the command.\n\
Fix: remove the trailing `}` and retry."
                .to_string(),
        );
    }
    if low.contains("adding embedded git repository") || low.contains("does not have a commit checked out") {
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
    if low.contains("could not connect to server") && low.contains("127.0.0.1") && cmd_low.contains("git") {
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

impl FailureMemory {
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

// ── Agentic loop ──────────────────────────────────────────────────────────────

/// Run the agentic loop.  Sends StreamToken events to `tx` for the TUI to display.
/// The caller builds the initial messages (system + history + user).
pub async fn run_agentic(
    messages_in: Vec<ChatMessage>,
    cfg: &RunConfig,
    tool_root: Option<&str>,
    tx: mpsc::Sender<StreamToken>,
) -> Result<()> {
    let client = reqwest::Client::new();
    let tools = json!([exec_tool_def()]);
    let mut state = AgentState::Planning;
    let mut mem = FailureMemory::default();
    let mut pending_system_hint: Option<String> = None;
    let mut forced_tool_once = false;

    let root_user_text = messages_in
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();
    let goal_wants_actions = wants_local_actions(&root_user_text);

    // Keep messages as serde_json::Value throughout to preserve tool_call_id.
    let mut messages: Vec<serde_json::Value> = messages_in
        .iter()
        .map(|m| json!({"role": m.role, "content": m.content}))
        .collect();

    // Resolve tool_root once (absolute path) and track cwd across tool calls.
    // This prevents the classic "cd didn't persist, so git add ran in the wrong repo" failure.
    let tool_root_abs = tool_root
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .and_then(absolutize_path);
    if let Some(ref root) = tool_root_abs {
        let _ = std::fs::create_dir_all(root);
        let note = format!(
            "[Working directory]\n\
Working directory (tool_root): {root}\n\
IMPORTANT: Each exec runs in a fresh process; `cd` does NOT persist unless the tool reports cwd.\n\
Always operate under tool_root. Create new repos under tool_root (fresh directory).\n\
NEVER create a git repo inside another git repo. If you see 'embedded git repository', STOP and relocate."
        );
        if messages.first().and_then(|m| m["role"].as_str()) == Some("system") {
            messages.insert(1, json!({"role":"system","content": note}));
        } else {
            messages.insert(0, json!({"role":"system","content": note}));
        }
    }
    let mut cur_cwd: Option<String> = tool_root_abs.clone();

    for iter in 0..MAX_ITERS {
        // ── Prune old tool results before sending to save context tokens ───
        prune_old_tool_results(&mut messages);

        // ── Progress checkpoint every 3 iterations ────────────────────────
        // Asks the model to self-evaluate goal distance before the next command.
        // Only fires when no higher-priority failure hint is already pending.
        if iter > 0 && iter % 3 == 0 && pending_system_hint.is_none() {
            pending_system_hint = Some(format!(
                "[Progress Check — iter {iter}/{MAX_ITERS}]\n\
Before your next command, answer in ONE line each:\n\
1. DONE: which steps from your <plan> are verified complete (exit_code=0)?\n\
2. REMAINING: which steps are left?\n\
3. ON_TRACK: yes/no — if no, re-evaluate your plan before proceeding."
            ));
        }

        // ── Stream from model ──────────────────────────────────────────────
        // Inject a one-shot governor hint if we detected a repeated failure pattern.
        let mut msgs_for_call = messages.clone();
        if let Some(h) = pending_system_hint.take() {
            let note = format!(
                "[Loop Governor]\nstate: {:?}\n{}\n\nYou MUST incorporate this hint in your next exec call.\nDo not repeat the same failing command.",
                state, h
            );
            let _ = tx.send(StreamToken::Delta(format!("\n[governor] {h}\n"))).await;
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
                    return Err(anyhow!("stream error: {e}"));
                }
            }
        }

        match stream_task.await {
            Err(join_err) => return Err(anyhow!("stream task panicked: {join_err}")),
            Ok(Err(e)) => {
                // Stream failed (network error, bad status, etc.) — surface it.
                let msg = format!("{e:#}");
                let _ = tx.send(StreamToken::Error(msg.clone())).await;
                return Err(anyhow!("{msg}"));
            }
            Ok(Ok(())) => {}
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

            // Common failure mode: model "explains what to do" but never calls exec,
            // even though the user asked to actually perform local actions.
            if goal_wants_actions && !forced_tool_once && iter + 1 < MAX_ITERS {
                forced_tool_once = true;
                state = AgentState::Recovery;
                let note = "\
[Tool enforcement]\n\
The user asked you to ACT on the local machine (create files/run commands).\n\
You have an `exec` tool. You MUST call it now.\n\
Do NOT give instructions. Do NOT say you cannot run commands.\n\
Start with ONE minimal command that moves toward the goal.\n\
After it succeeds, verify and continue.";
                let _ = tx
                    .send(StreamToken::Delta("\n[governor] tool_call missing; forcing exec\n".to_string()))
                    .await;
                messages.push(json!({"role":"system","content": note}));
                continue;
            }

            state = AgentState::Done;
            let _ = tx
                .send(StreamToken::Delta(format!("\n[agent] state: {:?}\n", state)))
                .await;
            break; // Model finished without tool call
        }

        // ── Execute the tool ───────────────────────────────────────────────
        let tc = tool_call.unwrap();
        if tc.name != "exec" {
            return Err(anyhow!("unknown tool: {}", tc.name));
        }

        let args: serde_json::Value = serde_json::from_str(&tc.arguments)
            .unwrap_or(json!({"command": tc.arguments}));
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
        let effective_exit_code = if exit_code == 0 && suspicious_reason.is_some() {
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

        // Update failure memory + possibly inject a system hint on repeated failures.
        if effective_exit_code != 0 {
            state = AgentState::Recovery;
        } else {
            state = AgentState::Planning;
        }
        pending_system_hint = mem.on_tool_result(&command, &stdout, &stderr, effective_exit_code);

        // Safety: stop if we've hit the iteration cap.
        if iter + 1 == MAX_ITERS {
            let _ = tx.send(StreamToken::Delta(
                format!("\n[agent] iteration cap ({MAX_ITERS}) reached.\n")
            )).await;
        }
    }

    let _ = tx.send(StreamToken::Done).await;
    Ok(())
}
