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

## Three Roles. Three Contexts. Zero Conflicts.

| Role | What it does | What it never does |
|---|---|---|
| **Coder** | Acts — files, shell commands, agentic loop (up to 12 steps) | Review or second-guess its own work |
| **Observer** | Critiques — scores every proposal, escalates what you ignore | Touch any code. It only reads. |
| **Chat** | Thinks with you — design, rubber duck, tradeoffs | Interrupt the execution loop |

Different roles. Different models if you want. Different contexts always.

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

### The Coder Doubts Itself

Before every command, the Coder fills out a 5-line scratchpad:

```
<think>
goal:   what must succeed right now
risk:   most likely failure mode
doubt:  one reason this approach could be wrong   ← the unusual field
next:   exact command
verify: how to confirm it worked
</think>
```

The `doubt:` field forces the model to surface one self-criticism before acting. ~50 tokens. It prevents the failure mode where the model is confidently wrong.

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

**Python Lite (WDAC / no Rust binary)**
```powershell
python .\scripts\serve_lite.py
# → http://127.0.0.1:18080/
```

---

## Key Concepts

### tool_root

Every agent action runs inside a scratch directory. Default: `.tmp/<thread-id>`.

This prevents nested git repositories, stray files in your project root, and the "why did it run in the wrong directory?" failure mode. Each thread is fully isolated.

### Approvals

- **Edit approval**: `write_file` calls queue as pending edits. You approve or reject each one.
- **Command approval**: `exec` calls can be gated the same way (optional). The Coder pauses and resumes after your decision.

Neither mode requires you to stop working — they queue silently.

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

---

## Security

`127.0.0.1` only by default. Shell execution is real — keep approvals enabled.

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
