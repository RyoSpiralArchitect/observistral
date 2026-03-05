# OBSTRAL

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)
![UI](https://img.shields.io/badge/UI-web%20%2B%20TUI-2dd4bf)

> **One prompt box is not enough.**
> OBSTRAL gives your AI a second brain — and makes them argue.

Languages: [English](README.md) | [日本語](README.ja.md) | [Français](README.fr.md)

---

Every AI coding tool has the same problem: the model that writes your code also reviews it.

That's not a review. That's a self-defense monologue.

OBSTRAL fixes this by running Coder and Observer in **completely separate contexts**. The Observer has never seen a single line of your code being written. It only sees the output. That's what makes it honest.

---

## Why OBSTRAL Exists

Most LLM tools optimize for conversation.
OBSTRAL optimizes for controlled execution loops: separate roles, approval gates, and critique that accumulates instead of resetting every turn.

This is not a chat client.
It's a development control engine.

---

## Three Roles. Three Contexts. Zero Conflicts.

| Role | What it does | What it never does |
|---|---|---|
| **Coder** | Acts — files, shell commands, agentic loop (up to 12 steps), 5 built-in tools | Review or second-guess its own work |
| **Observer** | Critiques — scores every proposal, escalates what you ignore | Touch any code. It only reads. |
| **Chat** | Thinks with you — design, rubber duck, tradeoffs | Interrupt the execution loop |

Different roles. Different models if you want. Different contexts always.

---

## What OBSTRAL Knows Before You Say Anything

When you set `tool_root`, OBSTRAL automatically scans the project:

```
[Project Context — auto-detected]
stack: Rust, React (no bundler)
git:   branch=main  modified=2  untracked=1
recent: "fix(observe): require all 4 blocks" · "feat(agent): error classifier"
tree:
  src/          12 files  (Rust source)
  web/           4 files  (JS/CSS)
  scripts/       8 files  (PowerShell)
key:  Cargo.toml · web/app.js · README.md
```

This context is injected into the Coder's system message **before your first prompt**. The Coder already knows the stack, current branch, modified files, and directory layout when you start typing.

In the TUI header you'll see a live badge: `▸ Rust · React · git:main`
In the Web UI, the stack label appears below the toolRoot field in Settings.

**Stack detection** — OBSTRAL looks for manifest files:
- `Cargo.toml` → Rust
- `package.json` → Node / React / TypeScript (inspects deps)
- `pyproject.toml` / `requirements.txt` → Python
- `go.mod` → Go
- `pom.xml` → Java

The scan runs once per session, takes under 200 ms, and silently skips anything it can't read.

---

## What Makes OBSTRAL Different

### The Observer Has No Skin in the Game

Other tools: same model writes code → same model reviews code → model defends its own choices.

OBSTRAL: fresh context for every Observer run. The Observer doesn't know what it *would have* written. It only judges what it sees.

Result: sharper feedback, honest risk assessment, no defensive hedging.

### Proposals Don't Disappear

When the Observer flags an issue and you don't act on it, the proposal escalates:

```
new  →  [UNRESOLVED] +10pts  →  [ESCALATED] +20pts, pinned to top
```

The Observer remembers what it said. If you ignore a `critical` warning twice, it becomes the loudest card on the board.

### Error Classification, Not Just Exit Codes

When a command fails, OBSTRAL doesn't hand the model a raw `exit_code: 1` and hope for the best. It classifies the error first:

| Error type | Recovery hint injected |
|---|---|
| `ENVIRONMENT` | Fix the environment. Don't touch source code. |
| `SYNTAX` | Fix the exact file. Don't change unrelated code. |
| `PATH` | Verify paths first. Don't create until you confirm. |
| `DEPENDENCY` | Install the package first. Then retry. |
| `NETWORK` | Check service status and proxy vars. |
| `LOGIC` | Re-read the logic. Don't just re-run. |

PowerShell caveat: `exit_code` can be `0` even when it printed errors (non-terminating error records).
OBSTRAL flags this as `SUSPICIOUS_SUCCESS` and treats it as failure to stop false-progress drift.

### The Coder Has Five Tools

The Coder isn't limited to shell commands. It has five purpose-built tools:

| Tool | When to use it |
|---|---|
| `exec(command, cwd?)` | Build, test, git, install packages — anything shell-based |
| `read_file(path)` | Read exact file content without shell quoting issues |
| `write_file(path, content)` | Atomically create or overwrite a file (parent dirs auto-created) |
| `patch_file(path, search, replace)` | Replace an exact snippet — fails loudly on ambiguity |
| `apply_diff(path, diff)` | Apply a unified `@@` diff (multiple hunks) — best for larger edits when `patch_file` is too small |

`write_file`, `patch_file`, and `apply_diff` use a temp-file → rename pattern, so a crash mid-write never leaves corrupt output.

`patch_file` requires the search string to appear **exactly once**. If it appears zero times, you get a preview of the file so the model can self-correct. If it appears more than once, you get the count. Ambiguity is an error, not a guess.

**Visual markers in the TUI** show which tool fired at a glance:
- `📄 READ` (teal) — file was read
- `✎ WRITE` (blue) — file was created or overwritten
- `⟳ PATCH` (magenta) — exact snippet replaced
- `✓` (green) / `✗` (red) — result OK or error

### The Coder Doubts Itself

Before every tool call, the Coder fills out a 5-line scratchpad:

```
<think>
goal:   what must succeed right now
risk:   most likely failure mode
doubt:  one reason this approach could be wrong   ← the unusual field
next:   exact command or operation
verify: how to confirm it worked
</think>
```

The `doubt:` field forces the model to surface one self-criticism before acting. ~50 tokens. It prevents the failure mode where the model is confidently wrong.

### @file References: Skip the Read Turn

Type `@path` anywhere in your message to inject that file's content as context before your prompt reaches the Coder:

```
@src/main.rs what does run_chat do?
@Cargo.toml @package.json show me the dependency versions side by side
fix the bug in @src/server.rs line 400
```

The TUI shows a notification for each injected file:
```
📎 injected: [src/main.rs] (276 lines, 8192 bytes)
```

The Web UI shows chips in the composer as you type:
```
📎 @src/main.rs   📎 @Cargo.toml
```

The Coder sees the file content immediately — no extra `read_file` round-trip needed. On a tight 12-iteration budget, skipping one read turn can be the difference between success and timeout.

### Phase Gating: Silence the Right Noise

Tell the Observer which phase you're in (`core` / `feature` / `polish`). Proposals that don't match are automatically dimmed. CSS tweaks don't interrupt you when your auth is broken.

### Health at a Glance

Every Observer response ends with a score:

```
--- health ---
score: 74  rationale: auth is solid, tests cover happy path only
```

❤ **74** → green (production-ready zone). The badge updates live as you build.

### Progress Checkpoints

At iterations 3, 6, and 9, the Coder pauses for self-evaluation:

```
1. DONE: which plan steps are verified complete (exit_code=0)?
2. REMAINING: what's left?
3. ON_TRACK: yes/no — if no, re-evaluate before the next command.
```

This is the difference between an agent that keeps circling and one that knows when it's lost.

### Windows-First (Really)

Most AI coding tools are designed on Mac, tested on Linux, and "should work" on Windows.

OBSTRAL was built on Windows. It handles:
- WDAC-blocked binaries → Python Lite fallback server (pure stdlib)
- Automatic PowerShell syntax translation (bash → PS)
- Corporate proxy environments
- `sh.exe` Win32 error 5 on interactive git prompts

### Plugin Registry

Extend OBSTRAL without forking it:

```js
registerObserverPlugin({ name: "my-plugin", onProposal, onHealth, onPhase })
registerPhase("security-review", { label: "Security Review", color: "#f97316" })
registerValidator(proposals => proposals.filter(p => p.score > 20))
```

Load your plugin via `<script>` before `app.js`. Done.

---

## The Observer Output Contract

The Observer doesn't free-write. It speaks a structured format that the UI parses into live cards:

```
--- phase ---
core

--- proposals ---
title: Input validation missing
toCoder: Validate length and character type before processing user input.
severity: critical
score: 88
phase: core
cost: low
impact: prevents crash on malformed input
quote: user_input = input()

--- critical_path ---
Fix input validation before adding any new features.

--- health ---
score: 41  rationale: core logic works but injection surface is wide open
```

Every field is intentional. `quote` pins the exact offending line to the card. `cost` tells you how hard the fix is before you read the details. `phase` controls visibility.

---

## Quickstart

**Web UI (recommended)**
```powershell
.\scripts\run-ui.ps1
# → http://127.0.0.1:18080/
```

**TUI (terminal)**
```powershell
.\scripts\run-tui.ps1
```

**Headless Coder (CLI)**
```powershell
# (optional) generate .obstral.md template (stack + test_cmd)
obstral init -C .

# run the coding agent in your project
obstral agent "fix the failing test" -C . --vibe

# persist and resume a session (default: .tmp/obstral_session.json)
obstral agent "fix the failing test" -C . --vibe --session
# resume later (omit prompt -> auto "continue")
obstral agent -C . --vibe --session

# auto-fix loop (Coder → Observer diff review → Coder)
obstral agent "fix the failing test" -C . --vibe --autofix
obstral agent "fix the failing test" -C . --vibe --autofix 3

# auto-approve tool actions (no prompts)
obstral agent "fix the failing test" -C . --vibe -y

# review your current git diff with Observer
obstral review -C .

# review changes since a checkpoint (hash printed by `obstral agent`)
obstral review -C . --base <checkpoint_hash>
 ```

**Python Lite (WDAC / no Rust binary)**
```powershell
python .\scripts\serve_lite.py
# → http://127.0.0.1:18080/
```

---

## Key Concepts

### tool_root

Every agent action runs inside a working directory.

Defaults:
- **Web UI**: `.tmp/<thread-id>` (isolated per thread)
- **TUI**: `.tmp/tui_<epoch>` (isolated per session)
- **CLI**: current directory

To work on your actual project, set `tool_root` to your project path:
- **TUI**: `-C .` / `--tool-root .` flag, or `/root <path>` slash command at runtime
- **Web UI**: Settings → toolRoot field
- **CLI**: `obstral agent "<prompt>" -C .`

When `tool_root` is set, OBSTRAL scans it on first use to build the project context block (stack, git, tree). Subsequent sends in the same session skip the scan.

Path traversal is blocked: paths with `..` components are rejected at every tool boundary.

### Language

- **UI language**: TUI `/lang ja|en|fr` (also affects prompts).
- **Observer language (Web UI)**: `auto` (default) follows your last user message language even if the UI is English; `ui` follows UI; or force `ja`/`en`/`fr`.

### Sessions (CLI)

`obstral agent` can save and resume the full conversation (including tool calls) with `--session[=<path>]`.

- Default path: `.tmp/obstral_session.json`
- If `-C/--root` is set, relative `--session` paths are resolved under `tool_root`
- Autosaves during the run (after tool calls)
- Resume without a prompt: run `obstral agent -C . --session` again
- Start fresh: add `--new-session` (overwrites the file)

Session JSON may contain code and tool outputs — treat it as sensitive.

### Approvals

- **Web UI**: edits/commands can queue as pending items. Approve/reject from the browser.
- **CLI (`obstral agent`)**: prompts before running `exec` and applying file edits (`write_file` / `patch_file` / `apply_diff`). Use `-y/--yes` or `--no-approvals` to skip prompts.
- **TUI**: currently auto-approves tool actions.

### Providers

OBSTRAL speaks OpenAI-compatible APIs. It also supports Mistral, Anthropic, Gemini, and local HF models via a `ChatProvider` trait.

Set a different model per role: fast model for Coder iteration, powerful model for Observer analysis. Common gotchas: `401` (bad key), `429` (rate limit), `max_tokens` vs `max_completion_tokens` mismatch.

### Chat Personas

Five chips above the Chat composer — switch anytime, independent of Coder/Observer:

| Chip | Style |
|---|---|
| 😊 Cheerful | Upbeat and encouraging |
| 🤔 Thoughtful | Checks assumptions, answers carefully |
| 🧙 Sensei | Guides with questions, not answers |
| 😏 Cynical | Points straight to the uncomfortable truth |
| 🦆 Duck | Never answers — just asks "Why?" to unblock your thinking |

### /slash Commands (TUI)

| Command | Effect |
|---|---|
| `/model <name>` | Switch model mid-session |
| `/persona <key>` | Switch Coder persona |
| `/temp <0.0–1.0>` | Adjust temperature |
| `/root <path>` | Change tool_root for subsequent sends |
| `/lang ja\|en\|fr` | Switch UI + prompt language |
| `/find <query>` | Filter messages in the current pane |
| `/help` | Show all commands |

---

## Security

`127.0.0.1` only by default. Shell execution is real — keep approvals enabled.

File tool paths are validated against `tool_root` at every call: absolute paths outside `tool_root` and any `..` component are rejected with an error (never silently).

If you expose to a network, add authentication and harden tool execution.

---

## Troubleshooting

**"Failed to connect to github.com via 127.0.0.1"** — dead proxy in env vars:
```powershell
Remove-Item Env:HTTP_PROXY,Env:HTTPS_PROXY,Env:ALL_PROXY,Env:GIT_HTTP_PROXY,Env:GIT_HTTPS_PROXY -ErrorAction SilentlyContinue
```

**Push without interactive prompts** (WDAC / sh.exe Win32 error 5):
```powershell
$env:GITHUB_TOKEN = "ghp_..."
.\scripts\push.ps1
```

**Push via SSH over 443** (corporate network):
```powershell
.\scripts\push_ssh.ps1
```

**"access denied" on obstral.exe** — binary still running:
```powershell
.\scripts\kill-obstral.ps1
```

---

## License

MIT
