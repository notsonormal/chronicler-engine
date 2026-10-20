# Decide the shape of the Job: one run loop, its failure lines and the narrator's prompt rules

Type: grilling (HITL)
Status: resolved
Blocked by: 19

## Question

Given the shape of a generation (ticket 19), what is the shape of the Job and of the narrator behind it? Decide:

1. **`Job::run` as one loop** (finding 6): one fact in place of the three booleans, and whether the Options run follows a `Stopped::SaveFailed`.
2. **The failure lines** (finding 2): what replaces the blanket `impl From<EngineError> for Failure`, so one fault shows one line whether `accept` or the Job reads it, and a read inside `JobWriter` does not show "The new text could not be saved."
3. **The error class of the new storage code**: the 34 rusqlite errors wrapped by hand as `EngineError::Config`, which `GenerationFailure::from_engine_error` cannot classify.
4. **The narration prompt rules behind the narrator** (candidates M3, S5, finding 19): `continuation_user_message` and the hook-input assembly move out of the Job; the `agent-edge` rule (the narrator gets values, not storage) stays.

## Context

- Inputs: [synthesis.md](../assets/architecture-review/synthesis.md) rows M3, S5; [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md) findings 2, 6, 19; thermo M7 in `../assets/whole-rebuild-review/findings/review-thermo.md`.
- Skills: `/grilling`, `/domain-modeling`, `/codebase-design` (its design-it-twice pattern for the interfaces). Terms: `CONTEXT.md`. Design: `.scratch/turn-state-model/assets/target-design.md`.
- From ticket 19 (resolved): the order is `GenerationOrder { generation: Generation, place, start: SwipeAt }`; the Game rules return the fired event record and the Job builds the event order with `GenerationOrder::event_after`; the start state is read twice today (`job.rs` `start_state`, `JobWriter::state_at`), which this ticket decides; the narrator gets `&Generation`, so the direction, the guide and the event direction come with no missing-data branch.

## Answer

(The rounds, 2026-10-20.) Round 1 owner answers: Q2 A (one line rule for a read in `accept` and the Job: "missing: <record name>", raw error in the disclosure; only a failed write shows "could not be saved"); Q3 A (save units read nothing; the Job passes values); Q4 A (typed `EngineError::Database { context, source }` for every rusqlite wrap under `storage/`). Round 2: Q1 A (the Options run follows every failure after a saved Narration except `EntryMoved`; F4 and F7 included, as design §4.2 says); Q5 A (the Job reads one input bundle per generation; the narrator owns the prompt rules of each kind; `Generation::preset_type` in the domain; one `HookInput` build).

Round 3 (drafts: [design 1, minimal](../assets/job-shape/design-1-minimal.md) (Opus), [design 2, common caller](../assets/job-shape/design-2-common-caller.md) (GLM), [design 3, types](../assets/job-shape/design-3-types.md) (Qwen), from [the brief](../assets/job-shape/brief.md)). The owner rejected the drafts' direction: they pass loaded data from step to step, against sketch 25 §2.1 "pass ids, load late, save small". Q6 to Q10 of round 3 are withdrawn. Round 4: Q11 yes (Q5 revised: no input bundle; each step loads by id what it uses, makes one call, saves small, and returns its result; only ids, texts and step results move between steps; a later step may read a record again, so finding 19 is the accepted cost; the narrator still owns the prompt rules of each kind, and `Generation::preset_type` stays); Q12 A (guardrail row 7 gains the parameter half: a `Job` method takes no record type as a parameter; a free helper fn that builds an input from values is allowed); Q13 A (ticket 19 Q14 revised: `PreAnalysisState` deleted, the Game-rules step loads the Swipe state at `saved.at`, R-18 is a match on the order).

### The answer (confirmed by the owner, 2026-10-20)

**Behaviour.**

| Q | Decision |
|---|---|
| Q1 | The Options run follows every failure after a saved Narration or event Narration, except `EntryMoved`. A `SaveFailed` of the state update or of the event text save gets it too (target design §4.2; today's code skips it). The status shows the first failure; a failure of that Options run is only logged. |
| Q2 | One line rule for a read, in `accept` and in the Job: "The data for this request is missing: <record>.", the raw error in the disclosure. Only a failed write shows "The new text could not be saved." The record names are one `Record` enum (`Game`, `Story`, `StartState`, `World`, `Persona`, `Settings`, `Preset`, ...). `impl From<EngineError> for Failure` is deleted, so a read that names no record does not compile. |
| Q4 | `EngineError::Database { context: &'static str, source: rusqlite::Error }` replaces every hand wrap of a rusqlite error under `storage/` (about 64 to 95 sites; serde and datetime wraps take `Serde` and `Parse`). `Config` stays for a setting with no valid value. `from_engine_error` then classifies only model and prompt-assembly errors; its `Database | Io` arm goes. |

**Data: the step rule** (Q3, Q11, Q12, Q13; sketch 25 §2.1).
- Each step loads by id what it uses, makes one call, saves small, and returns its result. Between steps only ids, texts and step results move. A later step may read a record again (finding 19 is the accepted cost).
- The save units read nothing; the step that calls a unit loads the values and gives them to it. Deleted from `JobWriter`: `state_at`, `tail_of`, `dock`, `opening_room_report`. An on-demand Options failure writes the notice only (a new notice-only `Tx` write), so `save_options` loses `after_narration`.
- Guardrail row 7 gains its parameter half: a method of `Job` takes no record type (`Game`, `Message`, `SwipeState`, `Tail`, `DockRecord`, `WorldBundle`) as a parameter. A free helper fn that builds an input from values is allowed.
- Ticket 19 amended (see its note): `PreAnalysisState` is deleted; the Game-rules step loads the Swipe state at `saved.at`; R-18 is a match on the order.

```text
generate(order, input, &mut narration_saved)
  room_check(order)                   loads World, start state
  save_input(text, order.start)       loads start state                       -> SwipeAt
  narrate(order)                      loads Game, settings, preset, World,
                                      start state, Tail, story window          -> text
  save_text(order, text)              loads start state, Tail                  -> SavedText
  if user regen: end (R-18)
  analyse(order, saved, text)         loads World, start state, window         -> QuantifierResult
  update_state(order, saved, result)  loads World, the state at saved.at;
                                      game_rules::apply; save                  -> the event order, or none
```

**The narrator** (Q5).
- The narrate step gives the values it loaded to `Narrator::narrate(&Generation, ..)`. The narrator owns the prompt rules of each kind: the user message (a user regen's Input text from the Tail the step read), the event continuation text, the guide and Impersonate flags, the role for health. `Generation::preset_type` is a domain fn. `Narrator::assemble` becomes private. The `agent-edge` rule stays.
- The after-narration hook and the end-of-Job hook each load their own values and build their `HookInput`; the Options run reads the toggle when it runs.

**The loop** (Q14).

```rust
enum JobFailure { EntryMoved, Line(GenerationFailure) }

fn run(&self, plan: JobPlan) {
    let mut narration_saved = false;
    let mut failure = None;
    let mut next = plan.first;
    while let Some(order) = next.take() {
        match self.generate(&order, &mut narration_saved) {
            Ok(event) => next = event,
            Err(f) => failure = Some(f.of(order.kind())),   // the one event wrap
        }
    }
    if options_follow(plan.options, narration_saved, failure.as_ref()) {
        self.options_step(plan.options);
    }
    self.end(failure);                                      // the one exit
}
```

Deleted: `Failure`, `Ended`, `Failure::missing`, `event_narration_failed`, `into_event_failure` (2 sites → 1), `failed`, the three holders of "a Narration was saved" (→ one local), `Stopped::SaveFailed` as a separate path (a unit error becomes `Line(save_failed)` at the unit).

**Not taken:** the input bundle (all three drafts), `UserRegen { text }`, `CheckedRoom`, the move of `WorldBundle` to the domain, `JobOutcome`. Round 3 Q6 to Q10 were withdrawn.

**Tests that pin it.**

| Tier | CASE id | Asserts |
|---|---|---|
| unit, `job_tests.rs` | `options-follow` (6 rows) | on demand; Narration saved with no failure, with a line, with `EntryMoved`; no Narration with and without a failure |
| unit | `failure-of-event` | an event's line becomes "The event narration failed." with the inner line in the raw text; `EntryMoved` and other kinds pass |
| unit, `generation_status_tests.rs` | `record-names` | each `Record` gives its fixed name in the line; a miss and a read error give the same line |
| seam contract, `narrator_tests.rs` | `narrator-user-message-by-kind` (4 rows), `narrator-kind-role` | the user message, the flags and the role of each `Generation`, with no storage |
| storage | `notice-keeps-set`, `db-error-context` | the notice-only write keeps the offered set; a failed query gives `Database { context, .. }` |
| tier 1, `options.rs` | `options-after-state-save-failed`, `options-after-event-text-save-failed` | the save line (or the event line) shows, and the Options run ran |
| tier 1, `failure_display.rs` | `job-read-names-record` (rows) | an injected read error in a Job shows "missing: <record>", not the save line |
| guardrail | row 7, parameter half | no `Job` method takes a record type |

Existing tier-1 tests that must stay green: `options.rs` (the three dock tests), `failure_display.rs` 38.x, `races.rs` (stop at each model call; the event narration uses the preset active when it runs), `retrigger.rs` and `swipe_new.rs` event-failure rows.

**Doc changes that come with the build:** build plan row 7 (the parameter half); `game_flow.md` (the Options run after a save failure).

