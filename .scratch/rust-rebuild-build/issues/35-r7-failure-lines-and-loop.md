# R7: the failure lines and the loop of the Job

Type: task (AFK)
Status: resolved
Blocked by: 34, 28

## Question

Build §9 of [the build plan of the response](../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §9 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §9 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built on `wf/rebuild-response` in one commit, after a green full gate (`logs/build_20261021_034457.log`: 191 guardrail, 1353 integration, 29 browser tests pass).

- One read-line rule: `Record` (next to `GenerationFailure`) names each record; every read in `accept`, the Job and the `JobWriter` reads names one. `impl From<EngineError> for Failure` is gone. `PlanRefusal::Missing` takes a `Record`.
- G3, G6: a missing hook preset or narrator preset gives the `Record::Preset` line; `hook_text` returns `Result<String>`; `PresetMissing` is gone. Production seeds the default hook presets at boot (`ensure_presets`); the test context now seeds them the same way.
- `from_engine_error` classifies only model and prompt-assembly errors. Effect (accepted, Q4): a failed save of the narrator's call record shows the internal-error line.
- One loop: `JobFailure { EntryMoved, Line }` (in `gate.rs`), one local `narration_saved`, one event wrap (`GenerationFailure::of_generation`), one exit. `Failure`, `Ended`, `Stopped`, `event_narration_failed`, `into_event_failure` are gone.
- The Options run follows every failure after a saved Narration except `EntryMoved`; the status shows the first failure (an Options failure is the status when nothing failed before it). Scenarios 24.23, 24.24.
- A failed state update shows the contradiction line and keeps the text (finding 24). Amends the §9 test row and ticket 20, which said the save line; target design §4.3 and §4.7 now state it. Scenario 38.17.
- `SceneMissing` is `InvalidRoom`. `GenerationFailureKind::ALL` comes from the `failure_kinds!` macro; the browser stub's `FAILURE_KINDS` is gone; the enum-docs guardrail now checks the variants inside the macro.
- G2: `OptionsRun::follows(narration_saved, entry_moved)` takes three facts, not four (a failure exists never changes the result), so `options-follow` has 5 rows. `failure-of-event` and `record-names` are domain unit tests.
- New scenarios 38.14 to 38.17; 38.5, 38.11 and 38.13 changed. The `job-read-names-record` id is split over four tests with the prefix `job_read_names_record_`.
- Owner note: a dev database with Games saved after v30 can hold a stored failure kind `SceneMissing` or `PresetMissing`; such a Game no longer loads. Reset the dev database if that shows.
