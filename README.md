# OBSTRAL

Dual-pane "dual brain" coding cockpit:

- **Coder**: acts (files + commands) with approvals
- **Observer**: critiques + proposes next actions (with scoring)
- **Chat**: brainstorming / narration without derailing execution

Languages: [English](README.md) | [Japanese](README.ja.md) | [French](README.fr.md)

![OBSTRAL UI](docs/ui.png)

## Highlights (What Makes It Different)

OBSTRAL is not just a chat client. It's a **development control engine** for agentic coding.

- **Execution-first**: tools (`write_file`, `exec`) + approvals (human-in-the-loop)
- **Dual-agent tension**: Coder acts, Observer audits (and refuses repeated loops)
- **Proposal engine**: structured `--- proposals ---` cards with score/phase/cost/impact
- **Loop detection**: repeated critiques trigger warnings (and a UI hue-shift), repeated failing commands trigger a governor hint
- **Sandboxed by default**: per-thread `tool_root` isolation prevents nested-git disasters
- **Windows-realistic**: PowerShell-native, handles WDAC/blocked binaries with a Python Lite server
- **Provider-agnostic**: OpenAI-compatible + multiple providers; handles real-world API quirks

## What This Is

Most LLM tools optimize for conversation.

OBSTRAL optimizes for **controlled execution loops**:

- dual-agent tension (Coder vs Observer)
- proposal scoring + phase gating (core/feature/polish)
- loop detection (repeated critique / repeated failing commands)
- safety rails (edit approval, command approval, tool_root isolation)

## Observer Output Contract (Structured Critique)

Observer is not "another chat bot". It speaks a structured format that the UI parses into cards:

- `--- phase ---` (`core | feature | polish`)
- `--- proposals ---` (scored, actionable, can be sent back to Coder)
- `--- critical_path ---` (one-line "what to do next")
- `--- health ---` (a numeric score + short rationale)

## Quickstart (Rust Server)

### UI (Web)

```powershell
.\scripts\run-ui.ps1
```

Then open:

- `http://127.0.0.1:18080/`

### TUI

```powershell
.\scripts\run-tui.ps1
```

Note: The scripts build/run with isolated `CARGO_TARGET_DIR` so UI and TUI can coexist.

## Lite Server (Python)

If you cannot run the Rust binary (e.g. WDAC blocks new EXEs), the repo includes a Python fallback:

```powershell
python .\scripts\serve_lite.py
```

This is a pragmatic compatibility mode, not a full replacement.

## Key Concepts

### tool_root

OBSTRAL runs all agent actions under a scratch directory (`tool_root`).

Default: `.tmp/<thread-id>` so each thread is isolated and you avoid nested-git disasters.

### Approvals

- **Edit approval**: model requests `write_file` get queued as pending edits; you approve/reject.
- **Command approval**: model requests `exec` get queued as pending commands; you approve/reject (Coder resumes after approval).

## Providers

OBSTRAL speaks "OpenAI-compatible" APIs and also supports multiple providers via a `ChatProvider` trait.

Common gotchas (caught and surfaced in logs):

- `401 Unauthorized`: missing/incorrect API key
- `429 Too Many Requests`: rate limit; retry/backoff
- `max_tokens` vs `max_completion_tokens`: model-specific parameter mismatch

## Security Model (Local-First)

OBSTRAL is designed for `127.0.0.1` usage.

If you expose it to a network, you must add authentication and harden tool execution.

## Troubleshooting

### "Failed to connect to github.com via 127.0.0.1"

Your environment is likely forcing a dead proxy (`HTTP_PROXY/HTTPS_PROXY/ALL_PROXY`).

Clear it for the current PowerShell session:

```powershell
Remove-Item Env:HTTP_PROXY,Env:HTTPS_PROXY,Env:ALL_PROXY,Env:GIT_HTTP_PROXY,Env:GIT_HTTPS_PROXY -ErrorAction SilentlyContinue
```

### Push via SSH over 443 (locked-down networks)

If `git push` fails in corporate Windows environments, try:

```powershell
.\scripts\push_ssh.ps1
```

### Push without interactive prompts (WDAC-safe)

Some environments break interactive git prompts (e.g. `sh.exe` fails with Win32 error 5).

If you can use a GitHub token, do a one-shot non-interactive push:

```powershell
$env:GITHUB_TOKEN = "ghp_..."
.\scripts\push.ps1
```

### `cargo run` fails with "access denied" on `obstral.exe`

That means the binary is still running from the same target directory.

Use:

- `.\scripts\kill-obstral.ps1`
- or just run via `.\scripts\run-ui.ps1` / `.\scripts\run-tui.ps1`

## License

MIT
