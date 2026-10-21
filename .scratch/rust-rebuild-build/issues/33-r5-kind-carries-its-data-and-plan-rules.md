# R5: the kind carries its data, and the plan rules

Type: task (AFK)
Status: resolved
Blocked by: 32

## Question

Build §7 of [the build plan of the response](../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §7 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §7 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built on `wf/rebuild-response` in one commit, after a green full gate (`logs/build_20261021_024048.log`: 189 guardrail, 1346 integration, 29 browser tests pass).

- `domain/model/generation.rs` holds the data-carrying `Generation`, `GenerationOrder { generation, place, start }`, `kind()`, `event_after`, `is_analysed()` (R-18 as a match on the order) and `write`. `write` is the one home of the Message kind, type, maker, stored inputs and event header, and marks an event's one-shot Trigger fired once.
- Deleted: `GenerationOrder.inputs` and `.event`, the missing-data guards, `message_kind_of`, `HookInput.kind` and `.event`, `SwipeState::pre_analysis`, `generation_record.rs`, and `StoredInputs` / `Swipe::stored_inputs` (test-only after the change).
- A bare `/impersonate` and the Retry of its Input run with `direction: None` (scenarios 1.18, 22.6).
- R-65: `Tail::retrigger_record` is the one rule, and `retrigger_allowed` calls it. The story-log template lost its own extra conditions (UI F3). Scenarios 13.4 and 8.14; 14.3 says "a typed Input"; 14.4 also asserts no button.
- `JobPlan::for_request(&JobRequest, StoryEnd)` is a pure domain function. `JobRequest`, `JobPlan`, `OptionsRun`, `PlanRefusal` and `StoryEnd` live in the domain. `gate.rs` keeps the guard, the storage checks (it always reads `has_text`, so no request planning is left) and its save units.
- `ScriptedModel::analysis` is renamed `after_narration`. Target design §4.5 gains D-36.
- The term "pre-analysis state" stays: ticket 20 Q13, the later owner ruling, uses it.
- Review finding rejected: the second Trigger reply of the 14.3 test is used. The first event Narration fails, so its fired mark is never saved, and the Trigger fires again.
