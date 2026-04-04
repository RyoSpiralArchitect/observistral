# TUI Agent Split Plan

`src/tui/agent.rs` is the current orchestration center for the Coder loop.
That is useful for iteration speed, but it is no longer the right long-term
shape for enterprise-grade maintenance.

This document tracks the next extraction targets.

## Why split now

`src/tui/agent.rs` currently owns too many concerns:

- provider compatibility repair
- plan/think/done gate validation
- read-only rescue and tool-first coercion
- evidence scoring and `done` auto-finalize
- resolution memory integration
- runtime loop orchestration

Those concerns change at different rates and should not all compete in one file.

## Extraction targets

### 1. `src/tui/agent/done_gate.rs`

Move:

- `done` validation helpers
- acceptance/evidence canonicalization
- read-only auto-finalize logic
- evidence scoring and `read -> done` landing

Reason:

- this is a coherent policy surface
- it changes with eval feedback
- it has many direct tests and should own them

### 2. `src/tui/agent/read_only.rs`

Move:

- read-only task detection
- tool-first policy
- plan/think rescue for observation tasks
- task-specific search/read coercion

Reason:

- this logic is now a substantial policy subsystem
- it is provider-agnostic and should be testable in isolation

### 3. `src/tui/agent/provider_compat.rs`

Move:

- Mistral/OpenAI compatibility repair helpers
- malformed tool-call normalization
- synthetic plan/think repair for provider-specific drift

Reason:

- provider quirks should not be interleaved with core runtime logic
- this area is likely to grow as more providers are added

### 4. `src/tui/agent/memory.rs`

Move:

- resolution memory plumbing
- observation-backed canonical path helpers
- future evidence-memory and command-memory glue

Reason:

- typed memory is now a first-class subsystem
- it should stay auditable and not be buried in loop code

### 5. `src/tui/agent/loop.rs`

Keep as the orchestration entrypoint:

- drive the model/tool loop
- call gates, rescue, and compat helpers
- own turn sequencing and stop conditions

This file should become thinner over time and mostly coordinate modules.

### 6. `src/tui/agent/task_harness.rs`

Move:

- task-lane inference such as `fix` / `create_file` / `init_repo`
- artifact-shape hints that tell the runtime what “progress” looks like
- progress-gate policy for repeated observation loops in action tasks

Reason:

- this is typed runtime state, not provider compatibility
- it should stay auditable and testable without growing the main loop
- action-task harnessing is now a distinct policy surface with its own eval cases

## Suggested extraction order

1. `done_gate.rs`
2. `read_only.rs`
3. `provider_compat.rs`
4. `memory.rs`
5. `task_harness.rs`
6. optional `telemetry.rs` if loop instrumentation keeps growing

This order minimizes risk because the first two already have strong eval/replay
pressure and the clearest ownership boundaries.

## Quality gate for each extraction

Each extraction should preserve:

- `cargo test -q ... tui::agent::tests::`
- relevant `runtime_eval` cases
- relevant `tui_replay` cases when Observer/soft-hint behavior is involved

Do not extract purely for aesthetics. Extract around ownership boundaries and
keep the tests close to the new module.
