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
| Session persistence | `src/agent_session.rs` | resumable run | `session.json` | `AgentSession`, `ObservationCache`, recent reflections, `SessionBridge` |
| Project-local reflection ledger | `src/reflection_ledger.rs` | cross-session | `.obstral/reflection_ledger.json` | recurring wrong assumptions, next minimal actions, reflection counts |
| Project-local harness evolution queue | `src/tui/agent/harness_evolution.rs` | cross-session | `.obstral/policy_patch_queue.json` | trace-derived runtime overlay proposals, seen/applied counts, promotion readiness |
| Project-local promoted governor overlay | `src/tui/agent/harness_evolution.rs` | cross-session | `.obstral/governor_contract.overlay.json` | eval-gated promoted harness policies, green case IDs, stable overlay defaults |
| Project-local contract promotion candidate | `src/harness_promotion.rs` | generated artifact | `.obstral/governor_contract.promotion.json` | UI-ready candidate list, patch previews, promotion decisions for `shared/governor_contract.json` |
| Project-local contract promotion review gate | `src/harness_gate.rs` | cross-session | `.obstral/governor_contract.promotion_gate.json` | human review decisions like approved/held/applied, GUI/TUI gate state for source-contract updates |
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
- typed resume bridge memory such as last good verification and repeated dead-ends

Examples:

- `AgentSession`
- `ObservationCache`
- `ObservationReadCache`
- `ObservationSearchCache`
- `ObservationResolutionCache`
- `SessionBridge`
- `SessionVerificationMemory`
- `SessionAcceptedStrategy`
- `SessionDeadEnd`

This is the right home for typed operational memory such as:

- canonical path resolution
- evidence-backed observations
- recent successful commands used for `done` citation
- accepted strategies that were already matched to successful follow-up actions
- repeated dead-end commands that should not be retried first after resume

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

### 5b. Project-local harness evolution queue

Code:

- `src/tui/agent/harness_evolution.rs`

File:

- `.obstral/policy_patch_queue.json`

Owns:

- trace-derived runtime policy overlay proposals that are not yet promoted into the source contract
- per-policy seen/applied counts and promotion readiness
- cross-session memory of which deterministic harness patches are repeatedly paying off

Examples:

- `fix_existing_files::force_mutation_after_observation_loop`
- `init_repo::advance_repo_scaffold_artifact`

Important rule:

- this layer may bias the live runtime with overlay prompts
- it must not directly rewrite `shared/governor_contract.json` during a normal run
- promotion into a source contract should stay gated by replay/eval health

### 5c. Project-local promoted governor overlay

Code:

- `src/tui/agent/harness_evolution.rs`

File:

- `.obstral/governor_contract.overlay.json`

Owns:

- harness policies that already passed replay/eval gating
- stable per-lane defaults that should load before the next live run starts drifting
- green eval case IDs that justify each promoted overlay rule

Important rule:

- this layer is stronger than the raw patch queue, but still weaker than current contradictory tool output
- it is the bridge between runtime-learned policy and eventual source-contract promotion

### 5d. Project-local contract promotion candidate

Code:

- `src/harness_promotion.rs`

File:

- `.obstral/governor_contract.promotion.json`

Owns:

- the reviewable candidate artifact that maps promoted overlays onto `shared/governor_contract.json`
- UI/TUI-friendly display cards, decision states, and patch previews
- the last generated promotion snapshot for humans or future GUI/TUI approval flows

Important rule:

- this file is candidate output, not live runtime policy
- it may be regenerated at any time from the promoted overlay plus the current source contract

### 5e. Project-local contract promotion review gate

Code:

- `src/harness_gate.rs`

File:

- `.obstral/governor_contract.promotion_gate.json`

Owns:

- human review decisions for source-contract promotion candidates
- the durable gate between "candidate exists" and "write `shared/governor_contract.json`"
- GUI/TUI audit state such as approved, held, and applied timestamps

Important rule:

- this file may authorize source-contract updates, but it does not replace the candidate artifact itself
- runtime overlays and promotion candidates should remain derivable even if this review gate is reset

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

Runtime eval cases may seed `session.json` when a regression only appears after
resume. Keep those seed sessions small, typed, and reviewable.

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
