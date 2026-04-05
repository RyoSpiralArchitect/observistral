# State Schema

This document defines the main state surfaces in `observistral` and which layer
owns each kind of data.

The goal is simple: new features should not invent "just one more place" to
store state without first choosing the correct owner.

## State layers

| Layer | Owner | Lifetime | Backing store | Examples |
|---|---|---|---|---|
| Provider/runtime config | `src/config.rs` | process / launch | CLI args + env | `PartialConfig`, `RunConfig` |
| Project-local TUI prefs | `src/tui/prefs.rs` | cross-session | `.obstral/tui_prefs.json` | `TuiPrefs`, `PanePrefs`, `coder_realize_preset`, pane model/provider/mode |
| Session persistence | `src/agent_session.rs` | resumable run | `session.json` | `AgentSession`, `ObservationCache`, recent reflections |
| Project-local reflection ledger | `src/reflection_ledger.rs` | cross-session | `.obstral/reflection_ledger.json` | recurring wrong assumptions, next minimal actions, reflection counts |
| In-memory orchestration state | `src/tui/app.rs` + `src/tui/agent/task_harness.rs` + `src/tui/agent/meta_harness.rs` + `src/tui/agent/evaluator_loop.rs` | live TUI session / live coder loop | memory only | `App`, `pending_auto_fix`, `TaskHarness`, `TaskLane`, `ArtifactMode`, `MetaHarness`, `FailurePattern`, `PolicyDelta`, `EvaluatorLoop`, `EvaluatorFinding`, `PolicyPatch` |
| Intent state | `src/tui/intent.rs` | live session, optionally persisted later | memory only today | `IntentAnchor`, `IntentUpdateKind`, normalized constraints/success criteria |
| Replay/eval fixtures | `.obstral/*.json` + `src/runtime_eval.rs` + `src/tui_replay.rs` | versioned test input/output | repo files + `.tmp/` artifacts | runtime eval spec, TUI replay spec, reports |

## Current ownership map

### 1. Runtime/provider config

Code:

- `src/config.rs`

Owns:

- provider/model/base URL selection
- mode/persona defaults
- temperature/max_tokens/timeout
- provider-specific defaulting and normalization

Should not own:

- live session memory
- replay artifacts
- intent drift state

### 2. Persistent TUI prefs

Code:

- `src/tui/prefs.rs`

File:

- `.obstral/tui_prefs.json`

Owns:

- per-pane preferences that should survive restarts
- global TUI knobs like language, auto-observe, realize preset, right tab

Examples:

- `TuiPrefs`
- `PanePrefs`
- `coder_realize_preset`
- `ui_lang`
- pane `provider/mode/model/base_url/persona/temperature`

Should not own:

- evidence from tool use
- stuck-case history
- observer suggestions from the current live run

### 3. Session persistence

Code:

- `src/agent_session.rs`

Owns:

- resumable chat/tool history
- recent reflection summaries
- observation-backed memory that is useful across resumes

Examples:

- `AgentSession`
- `ObservationCache`
- `ObservationReadCache`
- `ObservationSearchCache`
- `ObservationResolutionCache`

This is the right home for typed operational memory such as:

- canonical path resolution
- evidence-backed observations
- recent successful commands used for `done` citation

### 4. Project-local reflection ledger

Code:

- `src/reflection_ledger.rs`

File:

- `.obstral/reflection_ledger.json`

Owns:

- recurring wrong assumptions that have already been refuted
- previously effective next minimal actions
- lightweight cross-session reflection counts

Examples:

- `"broad search was unnecessary" => "read src/tui/prefs.rs"`
- `"cargo test was necessary first" => "run targeted tests"`

This layer is intentionally bias-only memory:

- it should guide the next probe
- it should not be treated as proof
- it must yield to current tool output when contradicted

### 5. In-memory orchestration state

Code:

- `src/tui/app.rs`
- `src/tui/agent/task_harness.rs`
- `src/tui/agent/meta_harness.rs`
- `src/tui/agent/evaluator_loop.rs`

Owns:

- live UI state
- transient task handles
- pending one-shot actions between panes

Examples:

- `pending_auto_fix`
- `pending_observer_hint`
- `last_observer_suggestion`
- `coder_realize_state`
- running task handles
- `TaskHarness`
- `TaskLane`
- `ArtifactMode`
- `MetaHarness`
- `FailurePattern`
- `PolicyDelta`
- `EvaluatorLoop`
- `EvaluatorFinding`
- `PolicyPatch`

This layer should stay transient. If a field must survive restart/resume, it
likely belongs in prefs or session persistence instead.

### 6. Intent state

Code:

- `src/tui/intent.rs`

Owns:

- normalized user intent, not raw conversation text
- scope-preserving updates such as `Replace`, `Refine`, `Continue`,
  `VagueModifier`

Examples:

- `IntentAnchor`
- `IntentUpdateKind`
- normalized `constraints`
- normalized `success_criteria`
- `optimization_hints`

Important rule:

- vague modifiers may refine quality but must not widen `goal` or `target`

### 7. Replay and eval fixtures

Code:

- `src/runtime_eval.rs`
- `src/tui_replay.rs`

Files:

- `.obstral/runtime_eval.json`
- `.obstral/tui_replay.json`
- `.tmp/runtime_eval_*`
- `.tmp/tui_replay_*`

Owns:

- repeatable behavior probes
- diagnostics and artifact capture
- per-case copied worktrees for mutation-oriented eval runs
- quality gates for changes to agent behavior

This layer should never become a substitute for runtime state. It is the place
to measure behavior, not to drive live orchestration.

## Rules for adding new state

Before adding a field, answer these questions:

1. Is it user preference, resumable operational memory, or live transient state?
2. Does it need auditability or cross-session persistence?
3. Is it derived from observed tool results, or is it inferred policy?
4. Which existing struct should own it?

If the answer is not clear, document it here first.

## Immediate follow-ups

- Move more evidence-backed memory behind `ObservationCache` rather than ad hoc
  in `App`.
- Keep the reflection ledger project-local and bias-oriented; do not let it
  silently override current evidence.
- Keep `IntentAnchor` memory-first for now; only persist it after replay/eval
  proves the shape is stable.
- Keep replay/eval specs versioned and human-editable.
