# R3: the room is one fact

Type: task (AFK)
Status: resolved
Blocked by: 30

## Question

Build §5 of [the build plan of the response](../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §5 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §5 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built with R2 in one commit on `wf/rebuild-response`, after a green full gate (`logs/build_20261021_011115.log`).

- `SwipeState::room` is the one room lookup (map room, else added room, else none); `room_by_id` serves `game_rules::walk`. The copies in `job.rs`, `view_query.rs`, `game_rules.rs` and `gate.rs` (`opening_room_report`) are gone.
- The marker is `StoryWindow::room_headers` (an associated fn: a free fn failed the free-fn-location guardrail). `story_window` reads the active Swipe state of each entry and the World map; each entry is `StoryWindowEntry { row, room_header }`, so `StoryRow` holds no derived data.
- Migration v29 drops `message_swipes.room_report`; `Swipe.room_report`, `StoryRow.room_report`, `RuleOutcome`, `arrived_room_name` and `is_the_opening` are gone.
- New scenarios 1.15 to 1.17 (`actions.md`) and 8.13 (`story_log.md`); the kept room scenarios use the "after a switch" form. Unit tests `room-added-room` and `room-marker-by-name`.
- Docs: target design (D-19 replaced, with its rejected option), clean spec (R-22, R-23), `game_flow.md` ("The Story Window"), `storage.md`, `integration_test_standards.md`.
- Carried to R9: `STORY_WINDOW = 4` in `domain/model/agent.rs` names the history window of the Agents, and it clashes with the term **Story window**.
