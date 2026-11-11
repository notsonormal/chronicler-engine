# R8: the step rule and the narrator

Type: task (AFK)
Status: resolved
Blocked by: 35

## Question

Build §10 of [the build plan of the response](../../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §10 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §10 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built on `wf/rebuild-response` in one commit, after a green full gate (`logs/build_20261021_042102.log`: 191 guardrail, 1360 integration, 29 browser tests pass).

- The step rule: `Job::generate` runs `room_check → save_input → narrate → save_text → [analyse → update_state]`; the event order loops through the same list; then `options_step` and `end`. Each step loads by id, calls once, saves small. Only ids, texts and step results move between steps.
- The save units read nothing: `JobWriter::state_at`, `tail_of` and `dock` are gone, with the duplicate start-state read and `Record::Dock`. The step gives the start state and the `Tail` to the unit.
- `Tx::put_dock_notice` (`TxWrite::PutDockNotice`) is the notice-only write; `save_options` lost `after_narration`. Every failed Options run writes the notice only; after a Narration the text save has already emptied the offered set, so the result is as before (target design §4.2).
- `GameWriter::stop` reads the dock record before it removes the job entry; a failed read leaves the Job running and answers "failed" (scenario 39.8, test `stop_dock_read_fails`).
- `Narrator::narrate(&Generation, ..)` owns the user message, the event continuation text, the guide and Impersonate flags and the role; `Narrator::assemble` is private. `Generation::preset_type` is a domain fn.
- Each hook builds its own `HookInput`; the Options run reads its toggle and preset when it runs (R-59).
- Rule 7, parameter half: the one row on `pipeline/job.rs` uses `TypeCheckPosition::FieldsAndParameters` (the registry allows one row per rule). Helpers are associated fns (the free-fn guardrail bans free fns in `pipeline/`).
- Accepted cost of ticket 20 Q11 (plan §10 amendment): `save_text` loads the start state after the model call, so a start-state read fault there shows "the start state" and the Narration is not saved. Scenario 38.14 gained that row.
- Carried to R9: `HookInput.player_text` has no reader.
