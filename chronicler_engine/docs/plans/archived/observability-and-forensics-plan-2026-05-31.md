# Plan: Observability & Automated Forensics

**Date:** 2026-05-09
**Status:** Planned
**Goal:** Reduce mean-time-to-diagnosis (MTTD) by making the system explain its own state on failure.

---

## Overview

Currently, when a test fails or the engine misbehaves, diagnosis requires a human or agent to:
1. Reproduce the failure (often involving LLM nondeterminism or complex state setup)
2. Read source code to infer what `GameState` looked like
3. Cross-reference `DEBUGGING.md`, `error_catalog.md`, and `docs/system/*.md`
4. Add temporary `println!` or log lines and rerun

This plan replaces that inferential loop with structured tracing and automatic forensics capture. When something breaks, the system emits a complete snapshot of the decision path, state, and context.

---

## Background

**Current logging:** `log = "0.4"` + `env_logger = "0.11"` provide flat string logging. There are no spans, no correlation IDs, and no structured context fields.

**Current test failures:** A panic or assertion failure in `components.rs` (1,504 lines) or `browser.rs` (781 lines) gives only the assertion message. The actual `GameState`, quantifier result, and LLM response context are lost.

**Documented pain points:**
- `DEBUGGING.md` instructs checking `GET /debug/state` — but this requires a running server and manual HTTP calls.
- `error_catalog.md` lists variants but agents still grep for strings to find causes.
- State mutation order in `action_processing.rs` is "load-bearing" and documented invariants exist, but violations are only detectable by reading code.

---

## Architecture Decisions

1. **Use `tracing` for structured instrumentation.** `tracing` spans and events provide structured fields, automatic context propagation, and subscriber-based output formatting. It interoperates with `log` so existing log lines continue to work.
2. **Forensics are opt-in for tests, always-on for debug builds.** We will not add runtime overhead to release builds. Tests will use a `#[cfg(test)]` forensics subscriber that serializes to JSON on failure.
3. **Preserve privacy in forensics.** LLM API keys and raw user prompts must be redacted from forensics snapshots.
4. **No breaking changes to `EngineError` or public APIs.** This plan extends observability; it does not change error variants or function signatures (except adding `#[instrument]` attributes).

---

## Phase 1: Investigation — Baseline Current Diagnostic Experience

**Goal:** Measure how long diagnosis currently takes so we can verify improvement.

### Task 1.1: Baseline Measurement
- Introduce a controlled failure in a mock backend (e.g., `MockBackend::with_empty_response()`) and time how long it takes an agent to locate the root cause using only existing tooling.
- Record: time to first relevant file, time to correct diagnosis, number of files read.
- **Files:** `tests/flow_mock_tests.rs`, `src/application/game_service/actions.rs`
- **Acceptance criteria:**
  - [ ] Documented baseline MTTD in this plan (target: >5 minutes)
  - [ ] List of files an agent typically reads before finding the cause

### Task 1.2: Tracing Audit
- Identify every location where a decision is made that affects game state or user-visible output.
- **Priority locations:**
  - `src/engine/action_processing.rs` — state mutation order
  - `src/engine/trigger_eval.rs` — trigger firing decisions
  - `src/narrative/agents/quantifier/core.rs` — quantifier confidence and NPC resolution
  - `src/narrative/llm_client.rs` — LLM request/response lifecycle
  - `src/application/game_service/actions.rs` — action dispatch and error mapping
- **Deliverable:** A markdown table of "decision points" with current log coverage (none/partial/good).

---

## Phase 2: Implementation — Structured Tracing

### Task 2.1: Add `tracing` Dependency and Subscribers
- Add `tracing` and `tracing-subscriber` to `Cargo.toml`.
- Configure a default subscriber in `main.rs` for human-readable output.
- Configure a JSON subscriber for test forensics.
- **Files:**
  - `Cargo.toml`
  - `src/main.rs`
  - `src/lib.rs` (test subscriber)
- **Acceptance criteria:**
  - [ ] `RUST_LOG=info cargo run` produces formatted output
  - [ ] `cargo nextest run` runs without trace pollution in default output
  - [ ] No regression in existing test pass rate

### Task 2.2: Instrument Core Decision Paths
Add `#[instrument(skip(...), fields(...))]` spans and `tracing::info!`/`warn!` events to:

1. **Action Processing**
   - `execute_freeaction_impl`: span with `room_id`, `action_type`, `input_length`
   - Each state mutation step: `handle_movement`, `resolve_npcs`, `add_log`, `evaluate_triggers`, `apply_npc_events`
   - **File:** `src/engine/action_processing.rs`

2. **Trigger Evaluation**
   - `evaluate_trigger`: span with `trigger_id`, `room_id`, `times_met`, `repeatable`
   - Events for `trigger_fired=true/false` with reason (room mismatch, already fired, condition false)
   - **File:** `src/engine/trigger_eval.rs`

3. **Quantifier**
   - `determine_npcs_in_room`: span with `room_id`, `backend_type`
   - Events for `High`/`Medium`/`Low` confidence with parsed NPC list
   - Events for fallback to static NPCs
   - **File:** `src/narrative/agents/quantifier/core.rs`

4. **LLM Client**
   - `narrate_action`: span with `backend`, `model`, `prompt_token_estimate`
   - Events for request start, response received, parse success/failure, empty response
   - **File:** `src/narrative/llm_client.rs`

5. **Game Service**
   - `execute_action`: span with `player_name`, `input`
   - `map_llm_error`: event with original `LlmFailure` variant before stringification
   - **File:** `src/application/game_service/actions.rs`

- **Acceptance criteria:**
  - [ ] Running with `RUST_LOG=info` shows a coherent trace of a player action from input to narration
  - [ ] Each major decision point emits a structured event with relevant IDs

### Task 2.3: Add Forensics Capture to Tests
- Create `src/test_support/forensics.rs` with:
  - `ForensicsCollector` subscriber that buffers spans/events during a test
  - `capture_on_failure()` that serializes the buffer to `tmp/diagnostics/<test_name>_<timestamp>.json` on assertion failure or panic
  - Redaction of sensitive fields (API keys, prompt text)
- Integrate into `tests/test_utils.rs` so all integration tests automatically capture forensics.
- **Files:**
  - `src/test_support/forensics.rs` (new)
  - `src/test_support/mod.rs`
  - `tests/test_utils.rs`
- **Acceptance criteria:**
  - [ ] A failing test produces a JSON file in `tmp/diagnostics/`
  - [ ] JSON contains: test name, timestamp, GameState snapshot, last 20 log entries, active span tree
  - [ ] No API keys or raw prompts appear in the JSON

---

## Phase 3: Implementation — State Replay

### Task 3.1: Serialize GameState for Replay
- Ensure `GameState` and all nested types implement `Serialize` (most already do via `serde`).
- Add a `ReplaySnapshot` struct that captures:
  - `world_id`
  - `game_state: GameState`
  - `last_action_input`
  - `last_llm_response`
  - `last_quantifier_result`
- **File:** `src/test_support/replay.rs` (new)
- **Acceptance criteria:**
  - [ ] Any `GameState` can be serialized to pretty-printed JSON
  - [ ] Snapshot round-trips through serialization without data loss

### Task 3.2: Build Replay Helper
- Add a `load_replay(path)` helper in tests that deserializes a `ReplaySnapshot` and constructs a `DefaultGameService` with the captured state.
- **File:** `src/test_support/replay.rs`
- **Acceptance criteria:**
  - [ ] A test can load a forensics snapshot and re-run the last action against the captured state
  - [ ] Re-running the action reproduces the same failure mode (deterministic for mock backends)

---

## Phase 4: Verification

### Task 4.1: Re-run Baseline Failure
- Re-run the controlled failure from Task 1.1 with the new observability stack.
- Measure time to diagnosis using forensics output only.
- **Acceptance criteria:**
  - [ ] MTTD reduced by ≥50% compared to baseline
  - [ ] Agent diagnoses the issue without reading more than 3 source files

### Task 4.2: Add Observability Guardrails
- Add a custom guardrail test that verifies every function in `action_processing.rs`, `trigger_eval.rs`, and `application/game_service/actions.rs` has either an `#[instrument]` attribute or a `// [OBS: no-instrument-reason]` comment.
- **File:** `tests/guardrails.rs` (extend existing)
- **Acceptance criteria:**
  - [ ] Build fails if a new decision-path function is added without instrumentation

---

## Dependencies

| Task | Depends on | Blocks |
|------|-----------|--------|
| 1.1 Baseline | None | 4.1 |
| 1.2 Tracing Audit | None | 2.2 |
| 2.1 Add tracing | None | 2.2, 2.3 |
| 2.2 Instrument paths | 2.1, 1.2 | 2.3, 4.1 |
| 2.3 Forensics | 2.1 | 3.1, 4.1 |
| 3.1 Serialize state | None | 3.2 |
| 3.2 Replay helper | 3.1, 2.3 | 4.1 |
| 4.1 Re-run baseline | 1.1, 2.2, 2.3, 3.2 | — |
| 4.2 Guardrails | 2.2 | — |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| `tracing` adds compile-time overhead | Use `tracing` with static level filters; disable in release with `release-max-level-info` |
| Forensics JSON leaks sensitive data | Redact all fields named `api_key`, `prompt`, `raw_response` during serialization |
| GameState serialization is too large | Limit history to last 20 entries; truncate long strings |
| Agent ignores traces and still reads code | Add guardrail (Task 4.2) and require forensics review in `DEBUGGING.md` |

---

## Success Criteria

1. A failing integration test produces a machine-readable forensics artifact within 1 second of failure.
2. An agent can diagnose the root cause of a mock backend failure using only the forensics artifact and `error_catalog.md`.
3. No regressions in test pass rate or build time (>10%).
