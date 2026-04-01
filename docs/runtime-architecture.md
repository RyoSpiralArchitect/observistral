# OBSTRAL Runtime Architecture (WIP)

OBSTRAL is not "a chat app that sometimes runs commands".
It is a **controlled execution runtime** for LLMs, with human gates and a safety governor.

This document describes the target structure we are converging on:

```text
CLI
 │
 ▼
Session Manager
 │
 ▼
Task Graph
 │
 ▼
Agent Loop
 │
 ▼
Safety Governor
 │
 ▼
Tool Router
 │
 ▼
Tools (exec / file ops / approvals / ...)
```

The point of the layering is to make failures legible and prevent the classic agent drift:
- repeating the same command
- nested git repo disasters
- "Done" without verification
- phase drift (polish advice during core failures)
- approvals and actions getting entangled

Related docs:

- `docs/state-schema.md` — typed state ownership and persistence boundaries
- `docs/tui-agent-split-plan.md` — extraction plan for `src/tui/agent.rs`
- `AGENTS.md` — contributor replay/eval policy and high-touch file rules

## Mapping To Current Code (2026-03)

The runtime is already present, but parts are still "in one file". This mapping makes it explicit.

### 1) CLI

- `src/main.rs` (subcommands: `agent`, `tui`, `serve`, `review`, ...)
- `src/tui/mod.rs` (TUI entrypoint, tool_root isolation)
- `web/app.js` (Web UI, longrun agent loop client-side)

### 2) Session Manager

Responsibilities:
- load/resume session
- atomic save + crash resume
- track tool_root / checkpoint / cwd
- (optionally) trace output

Current code:
- `src/agent_session.rs` (`AgentSession`, `SessionAutoSaver`)
- `src/project.rs` (project scan: stack/git/test_cmd)

### 3) Task Graph

Responsibilities:
- represent "what work exists" and "what is done"
- allow routing to Coder vs Observer tasks
- persist across runs

Current code:
- TUI tasks tab + TaskRouter: `src/tui/app.rs`, `src/tui/events.rs`
- Web UI tasks list: `web/app.js` (thread tasks)

### 4) Agent Loop

Responsibilities:
- iterative model calls
- append messages (OpenAI tool-call format)
- execute tools and feed results back
- stop conditions + goal checks

Current code:
- TUI/CLI agentic loop: `src/tui/agent.rs::run_agentic_json`
- Streaming adapter: `src/streaming.rs`
- Web longrun loop: `web/app.js::runCoderAgentic`

### 5) Safety Governor

Responsibilities:
- stop repetition loops (same cmd / same output / same error)
- classify failures to route recovery strategy
- detect suspicious "success" (exit=0 but error markers)
- sandbox constraints (cwd/tool_root)
- phase gating (core/feature/polish)

Current code:
- `src/tui/agent.rs` (AgentState + FailureMemory + error classifier + stuck hints)
- `src/exec.rs` (dangerous command checks, cwd validation)
- `web/core/exec.js` (bash→PowerShell normalization, dangerous command guard)
- `web/app.js` (loop governor + goal_check probes + recent-runs memory)

### 6) Tool Router

Responsibilities:
- map tool names to tool implementations
- enforce sandbox + approvals
- normalize OS-specific command behavior

Current code:
- `src/tui/agent.rs` (routes tool_calls to exec / read/write/patch/apply_diff / glob / search)
- `src/exec.rs` (exec runner)
- `src/file_tools.rs` (file tools)
- `src/approvals.rs` + `src/pending_*` (approval gating)

### 7) Tools

Current code:
- exec: `src/exec.rs`
- files: `src/file_tools.rs`
- approvals: `src/approvals.rs`, `src/pending_commands.rs`, `src/pending_edits.rs`

## Next Refactor Milestones

1. Introduce `src/engine/` module to make the layers explicit in code.
2. Move session + trace concerns behind a single `SessionManager` API.
3. Promote TaskRouter outputs into a DAG (dependencies + statuses), persisted into the session.
4. Formalize governor decisions as structured events (so UI can visualize "why we stopped").
5. Keep tools stable (their safety properties are the foundation), improve orchestration above them.
