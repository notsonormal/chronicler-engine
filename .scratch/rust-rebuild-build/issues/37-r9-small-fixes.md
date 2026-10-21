# R9: the small fixes

Type: task (AFK)
Status: resolved
Blocked by: 36

## Question

Build §11 of [the build plan of the response](../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §11 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §11 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built on `wf/rebuild-response` in one commit, after a green full gate (`logs/build_20261021_045354.log`: 191 guardrail, 1365 integration, 29 browser tests pass).

- Finding 10: the 39.4 Options-call row asserts the status `options`; a new row stops at the analysis call of an event Narration (the one-shot Trigger is in the new Swipe state).
- Finding 16: `ActiveGame::set` returns `Result` and moves the pointer only after the `updated_at` touch lands; `ActiveGame::set_new` serves create and reset, which have no write that can fail after their unit. The wrong `updated_at` claim is gone from the docs of `set` and `touch_game_updated_at`. Scenario 18.5.
- Finding 22: a failed status read answers the missing-data line of `Record::Game` (HTTP 200, so htmx swaps it in), not "idle". Scenario 39.9.
- Finding 23: a reset with no active Game answers 400. Scenario 7.4.
- P-1: today's behaviour, as scenarios: with no active Game a stop answers "failed" (39.10) and a status poll answers "idle" (39.11).
- Finding 28: scenario 3.5 asserts 2 Narrations.
- Finding 29: no `render().unwrap_or_default()` is left under `src/adapters/driving/http/` (also `worlds.rs`, the saved-games list, the text-check card, the select options). One helper `render_or_error_fragment` and one 500 response shape (`HX-Reswap: none`) in `utils/fragment.rs`; `handler_helpers::render_template` is gone. A render failure has no tier-1 test: Askama has no seam. `architecture_system.md` lost its `AppState` field list.
- Carry-overs: `STORY_WINDOW` is `AGENT_HISTORY_WINDOW`; the dead `HookInput.player_text` is deleted.
- **For the owner to confirm** (a fill, as G1, G3 and G6 were): ticket 26 Q3 says a Game select whose pointer write fails answers with "the failure line". `CONTEXT.md` defines a failure line as the line stored on a Game, so the select answers a 500 error fragment that names the storage failure, as scenario 20.4 does.
