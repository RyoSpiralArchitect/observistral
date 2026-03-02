/// Coder agentic loop: calls the model with an `exec` tool, runs commands,
/// and loops until finish_reason == "stop" or max iterations are reached.
///
/// Reasoning improvements applied here:
///   1. Scratchpad protocol  — model outputs <think>goal/risk/next</think> before every
///      tool call (~30 tokens).  Prevents wrong-direction errors that cost 300+ tokens to
///      recover from.
///   2. Tool output truncation  — stdout > 1500 chars / stderr > 600 chars are trimmed.
///      This is the single largest token-saver on long runs.
///   3. Context pruning  — tool results older than KEEP_RECENT_TOOL_TURNS are collapsed
///      to their first line.  Keeps the context window from exploding across sessions.
///   4. Error amplification  — on exit_code != 0, a structured diagnosis prompt is
///      injected so the model must identify the root cause before its next action.
///   5. tool_call_id preserved  — messages stay as serde_json::Value all the way to the
///      provider, so the id field is never silently dropped.
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
goal: <≤12 words: what must succeed right now>\n\
risk: <≤12 words: most likely failure mode>\n\
next: <≤12 words: exact command or step>\n\
verify: <≤12 words: how to confirm this step succeeded>\n\
</think>\n\
Keep each field under 15 words. This 4-line check (~40 tokens) prevents\n\
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

/// Build the tool result string for a failed command with structured
/// diagnosis guidance.  Forces the model to reason about the error rather
/// than blindly retrying or continuing.
fn build_failed_tool_output(stdout: &str, stderr: &str, exit_code: i32) -> String {
    let mut out = format!(
        "FAILED (exit_code: {exit_code})\n\
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
            return None;
        }

        self.consecutive_failures = self.consecutive_failures.saturating_add(1);

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
            return Some(
                "3 consecutive failures.\n\
Action: change strategy now; do NOT retry the same approach again."
                    .to_string(),
            );
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

    for iter in 0..MAX_ITERS {
        // ── Prune old tool results before sending to save context tokens ───
        prune_old_tool_results(&mut messages);

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
        let cwd = args["cwd"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| tool_root.map(|r| r.to_string()));

        // Emit tool-call annotation to TUI (dim line the UI will colour differently).
        let _ = tx
            .send(StreamToken::Delta(format!(
                "\n\n[TOOL][{:?}] {command}\n",
                state
            )))
            .await;

        let exec_result = exec::run_command(&command, cwd.as_deref()).await;

        let (stdout, stderr, exit_code) = match exec_result {
            Ok(r) => (r.stdout, r.stderr, r.exit_code),
            Err(e) => (String::new(), e.to_string(), -1),
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
            build_ok_tool_output(&stdout)
        } else {
            let mut out = build_failed_tool_output(&stdout, &stderr, effective_exit_code);
            if let Some(reason) = suspicious_reason {
                out = format!(
                    "NOTE: command returned exit_code=0 but was treated as failure.\nreason: {reason}\n\n{out}"
                );
            }
            out
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
