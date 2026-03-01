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
After every build/test: confirm exit_code == 0 before proceeding.";

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

    // Keep messages as serde_json::Value throughout to preserve tool_call_id.
    let mut messages: Vec<serde_json::Value> = messages_in
        .iter()
        .map(|m| json!({"role": m.role, "content": m.content}))
        .collect();

    for iter in 0..MAX_ITERS {
        // ── Prune old tool results before sending to save context tokens ───
        prune_old_tool_results(&mut messages);

        // ── Stream from model ──────────────────────────────────────────────
        let (token_tx, mut token_rx) = mpsc::channel::<StreamToken>(256);
        let cfg_clone = cfg.clone();
        let tools_clone = tools.clone();
        let client_clone = client.clone();
        let msgs_clone = messages.clone();

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
        let _ = tx.send(StreamToken::Delta(format!("\n\n[TOOL] {command}\n"))).await;

        let exec_result = exec::run_command(&command, cwd.as_deref()).await;

        let (stdout, stderr, exit_code) = match exec_result {
            Ok(r) => (r.stdout, r.stderr, r.exit_code),
            Err(e) => (String::new(), e.to_string(), -1),
        };

        let tool_output = if exit_code == 0 {
            build_ok_tool_output(&stdout)
        } else {
            build_failed_tool_output(&stdout, &stderr, exit_code)
        };

        let result_label = if exit_code == 0 {
            format!("[RESULT] exit=0\n")
        } else {
            format!("[RESULT] exit={exit_code} ⚠\n")
        };
        let _ = tx.send(StreamToken::Delta(result_label)).await;

        // ── Append tool result (with tool_call_id preserved) ───────────────
        messages.push(json!({
            "role": "tool",
            "tool_call_id": tc.id,
            "content": tool_output,
        }));

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
