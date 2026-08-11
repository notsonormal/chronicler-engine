# T2-ARCH: Narration Deepening

**Parent Plan:** [abstraction-fixes-followup-superplan.md](./abstraction-fixes-followup-superplan.md)
**Status:** Planning — needs G2–G5 grilling before interface design
**Date:** 2026-06-28
**Depends on:** T1 (soft — error shape), R2 in `reliability-and-cancellation-plan.md` (cancellation plumbing — coordinate G4), T9 (ADR-018 doc update)
**Blocks:** none
**Priority:** P1
**Findings owned:** N5

---

## Summary

The original T2 framed the gap as "two reimplementations of pipeline logic." Architecture-lens reframe: this is **one deep Narration module split across two adapters**. `phase_narrate` (`application/action_pipeline/phases.rs`) and `ArrivalTaskContext::run` (`bootstrap/init_game.rs:146`) both implement the same narration pipeline (load → context → assemble → complete → persist) behind disconnected interfaces. Per LANGUAGE.md: two adapters = real seam. The deep module (Narration) does not exist as an explicit module.

A third LLM-call site (`phase_trigger_continuation_raw`) is a **different seam** — it replays pre-assembled stored prompts (bypasses the assembler). Not in scope here.

## Constraints Any Deepened Module Must Resolve

1. **State ownership** — pipeline receives pre-loaded `GameState`; arrival owns `Arc<Storage>` and loads snapshot inside `run()`.
2. **Preset acquisition** — pipeline calls `self.service.load_preset_and_response_length()` at call time; arrival receives `arrival_preset` + `response_length` baked into struct at spawn time.
3. **Persistence policy** — both now route through `save_message_and_snapshot` (closed in T2-Q1 / `db8ab25`).
4. **Cancellation** — pipeline checks `cancel_token` pre+post LLM call; arrival task's token registration + checks owned by R2 in `reliability-and-cancellation-plan.md`.
5. **Status reporting** — pipeline writes `GenerationStatus::Generating` and transitions through phases (`GeneratingEvent`, `Quantifying`); arrival only sets `Generating` then `Idle`/`Error`.
6. **Backwards data flow** — pipeline returns `(text, backend_name, model_name)` for caller to persist `LlmMessage` forensics; arrival discards `backend_name`/`model_name`.
7. **Domain role** — pipeline = "narrate user input" (always has `input: String`); arrival = "narrate arrival scene" (empty input, only when `!has_scenario`). Different domain meaning.

## Grilling Decisions (resolve before designing interface)

- **G2 (naming).** `Narrator` (conflicts with `AGENT_NARRATOR`?), `NarrationDriver`, `SceneNarration`, or two modules `ArrivalNarration` + `InputNarration` (rejects deepening).
- **G3 (state ownership shape).** (a) always receive `GameState`; (b) always own storage + load internally; (c) two entry points sharing internals.
- **G4 (cancellation).** (a) require `cancel_token` param; (b) `Option<&CancellationToken>`; (c) always require token with spawner supplying it. **Coordinate with R2** — token registration gap must close first.
- **G5 (ADR-018 conflict).** `architecture/system.md:208-209` currently documents `ArrivalTaskContext` as deliberate; revisit if deepening proceeds. Re-flag prevention covered by super-plan Finding State table + T9 task 5.

(G1 — persistence difference — was the bug fixed in `db8ab25`.)

## Key Changes

To be determined after G2–G5 grilling. Do NOT propose interfaces yet — sub-plan runs `improve-codebase-architecture` skill grilling loop first.

## Side Concerns

- `ArrivalTaskContext` has 12 fields — coupling smell. Deepened module may swallow fewer.
- `phase_narrate` returns `backend_name`/`model_name` for forensics (`LlmMessage`). Deepened module must expose them OR persist `LlmMessage` itself (more locality, larger interface).
- `spawn_blocking` is caller's concern (pipeline + `message_editing.rs:145,189` + `init_game.rs:317` all spawn). Deepened module stays sync; callers spawn.

## Blast Radius

`bootstrap/init_game.rs` + `application/action_pipeline/phases.rs` + possibly new `application/narration.rs` (or `narrative/narration.rs`) — placement decision deferred to grilling. High churn but mechanical-ish.

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean.
- Ensure `phase_narrate` + `ArrivalTaskContext::run` tests still pass — they exercise both adapters via existing scaffolding; deepened module must preserve their coverage.
- Integration test coverage for both narration paths (user input + arrival scene) — required before merge (structural track).
- Confirm `architecture/system.md:208-209` + ADR-018 updated if deepening proceeds.

## Pre-Implementation Checklist

- [ ] R2 in reliability plan must land first (token registration gap + arrival cancellation checks). Without R2, deepened module's cancellation contract is decorative.
- [ ] Run `improve-codebase-architecture` skill grilling loop to resolve G2–G5.
- [ ] Verify ADR-018 conflict outcome with user before writing code.
