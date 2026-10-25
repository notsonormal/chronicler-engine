# Research: survey the pipeline's shape for the fix-or-rewrite decision

Type: research
Status: resolved
Blocked by: —

## Question

What is the current shape of the generation pipeline (modules, entry paths, shared phases, seams to `MessageService` and `GenerationGate`), where do the entry paths duplicate or diverge, and how often has this code changed and broken? The answer gives the facts that the "Fix or rewrite" decision weighs.

## Scope

1. Module map of `src/application/pipeline/` (`pipeline_run.rs`, `narration_generation.rs`, `phase_error.rs`, `action_pipeline/*`), `message_service.rs`, `generation/`, `arrival_service.rs`: size, public seams, and callers.
2. Per entry path (action, each retry form, Retrigger, Options, arrival): which phases it shares through `PipelineRun`, and which it re-implements.
3. Where Snapshot saves and pointer rules live: in one module, or spread across phases (use the count from [Trace the Snapshots each generation path saves](04-research-trace-generation-snapshots.md) if it is resolved; otherwise count them).
4. History: `git log` churn of these files, and the closed tickets and plans that fixed defects here (`.scratch/_closed/`, `docs/plans/`).
5. Seams for tests: can a test drive one entry path without HTTP? Where would a model-level test sit?

Use the `/codebase-design` vocabulary (module depth, seams). Report facts. Do not recommend fix or rewrite; that is a later decision.

## Output

An asset `../assets/research-12-pipeline-shape.md`: the module map, the entry-path-by-phase matrix, the churn and defect history, and the test seams. Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

The map's fog patch "Fix or rewrite".

## Answer

Asset: [research-12-pipeline-shape.md](../assets/research-12-pipeline-shape.md). Code at `d6a047cd` plus the 2026-10-11 work tree. Facts are known from the code or git unless marked inferred. None is newly tested.

- **Shape:** ten entry paths run through five ordering functions (`run_from_input`, `retry_event_continuation`, `retry_user_regen`, `execute_options_refresh`, `ArrivalTaskContext::run_inner`). `run_from_input` (164 lines) serves six of the ten. `PipelineRun` has 17 phase methods and owns no order. `NarrationGeneration` is the one deepened phase.
- **Duplication:** the event tail (Trigger, reconciliation, Options, finalize) is written twice; the `retry_target` append three times; the world-bundle load three times; the narrator call twice (arrival uses its own prompt path); there are five failure writers plus the cancellation writer. User regen skips the pre-main save.
- **Saves and pointers:** 14 pointer-capable save sites in 7 files, plus 10 pointer-free Snapshot writers. The pointer rule has one main home (`save_message_and_snapshot`) and two copies for a Game's first Snapshot. Which Snapshot a pointer gets is set by phase order across the five ordering functions, together with `GameState::push_message` and two in-memory-only fields.
- **Status:** 33 hand-written `input_buffer.status`/`.phase` lines in 7 files.
- **History:** 108 commits (an upper bound) on these files in five months; 63 are refactors; a structural reshape about once a month. Ten defects fixed here; six open from this map. Six fixed and five open share the pointer-timing or writer root (inferred grouping).
- **Overlap:** the open map `architecture-deepening` has five grilling tickets on the same modules (04 lifecycle, 05 MessageService, 08 gate, 09 status transitions, 10 arrival). None is resolved. Commit `8449a491` already rejected a post-turn Swipe pointer because Retry anchors read it.
- **Test seams:** a unit test can drive each entry body without HTTP (in-memory `Storage`, `MockBackend`). No module answers "which Snapshot is each pointer, and what does it hold", so a model-level test can only read storage. Retry tests seed the save order by hand; that order has drifted from the real one before (phase-2 ticket 09) and still differs (the Input points at Sc, not S0).
- **New inferred fact for the writer rule:** each spawned body reads `current_game_id()` when it starts, not the game id the claim used. A game switch between claim and task start gives the run the new game.
