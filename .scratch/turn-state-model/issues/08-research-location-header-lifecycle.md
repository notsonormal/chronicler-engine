# Research: trace the location header from write to render

Type: research
Status: resolved
Blocked by: —

## Question

Where is a room name written (`location_header`, `pending_location`, `ArrivingLocationWrite`), on which Message or Swipe, at which phase, and where is it read? Which data would option B of the room ticket (derive the header from the Swipe's result Snapshot at render time) need, and is that data present today?

## Scope

1. Every write: `GameState::log_movement_completion`, `NarrativeState::arriving_location_write`, `MessageService::save_message_and_snapshot`, `Storage::update_swipe_location_header`, `pending_location` for the starting scenario, arrival narration, the event Message, the unknown-room case in `GameState::attempt_movement`.
2. Every read: the story log render, swipe switch (scenario 9.8 in `docs/specs/swipe_new.md`), any other reader.
3. Per case (normal move, Trigger move, starting scenario, arrival, unknown room, Retry of a moving generation, swipe switch to a non-moving Swipe), the header shown and the room restored.
4. The plan for `pending_location` in `docs/plans/subplan-c-mapless-enablement.md`, and whether it still holds.
5. What a Snapshot holds about the room before and after the move, so that a render can tell "the room changed".
6. Stored data in old Games: which rows carry a header, and the migration helpers in `adapters/driven/storage/utils/plumbing.rs`.

## Context

- [Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md): the question, the options A, B and C, and the ticket-15 history.
- [Put the room name on the Narration that arrives in the room](../../dashboard-ui-review-phase2/issues/15-room-name-on-arriving-narration.md): its Answer.
- `docs/diataxis/explanation/storage_design.md` §Per-swipe state binding, `docs/diataxis/reference/game_flow.md` §Movement Branch.

## Output

An asset `../assets/research-08-location-header.md`: a write table, a read table, the per-case table, and a short note on what each option of the room ticket would change in code. Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

[Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md).

## Answer

The header is a **change marker, not a per-Message room stamp**. It lives on one column, `message_swipes.location_header`, carried in the domain as `Swipe.location_header` behind the active Swipe. Only the Swipe of the generation that **moved** the player carries one. It is display-only: no prompt renders it — a prompt's room comes from `PromptContext.room` through `LayerRenderer::render_game_state_layer`. Full tables (write, read, per-case, options A/B/C): [asset](../assets/research-08-location-header.md).

**Ticket 08's core question — which data option B needs, and whether it is present today: no, four pieces are missing, and one of them cannot be fixed by a pointer alone.**

1. **No result pointer.** `save_message_and_snapshot` sets a Swipe's `snapshot_id` only while it is `None`, so the pointer lands at the pre-Quantifier save. The post-engine Snapshot — the first state that knows the new room — is written by `persist_snapshot_or_err`, which returns `Result<(), PhaseError>` and **discards the id**. Nothing writes it anywhere.
2. **No "before" side.** Even with a result pointer, "show it when the room differs from the room before" has no before: the Input's `Sc` and the moving Narration's `S1` both hold the **pre-turn** room. A snapshot-diff render built on today's pointers would label the *following* Message — the pre-fix one-Message-late bug of finding S6, moved one Message further along.
3. **Room-name resolution is unreachable from HTTP.** A Snapshot holds `movement.current_room_id`; the *name* needs `MapDef::get_room_by_id` plus `dynamic_rooms`. That logic exists in `GameState::log_movement_completion` but not on the read side. `AppState` holds no `storage`; `GameViewQuery` is the only read-side type that owns `Arc<Storage>`.
4. **The story log cannot see per-Swipe Snapshots.** `GameViewQuery::get_story_log_entries` → `MessageService::load_or_fresh` loads exactly one Snapshot (`load_latest_snapshot`). The per-Swipe ids are in memory but `MessageEntry`/`MessageEntryView` drop them.

**Option B fixes the display only.** `MessageService::switch_swipe` restores `swipe.snapshot_id`, so unless ticket 01 also changes the restore target, the mismatch survives: the header would still name the arrival room while the switch returns the player to the room it left. **Option A is the only one of the three that changes the restore.** Option C needs no new data at all — `GameViewQuery::get_current_room_view` already resolves the live room name; today the sidebar shows it only in `<img alt>`.

**The mismatch is exactly one class of Swipe:** the Swipe of the generation that moved the player (a Narration's Swipe 0, or the redo Swipe of a main/guided/re-impersonate Retry). It is reachable only while that Narration is the last Message *and* has 2+ Swipes — i.e. after a Retry — because `switch_swipe` refuses any Message that is not the last. Per case, it shows up in: a normal move, a Trigger move, the unknown-room case, and the redo Swipe of main and guided Retry. Every other Swipe either has no header (Inputs, event Swipes, stayed-put redos) or one that matches its own Snapshot (the starting scenario), so those cannot contradict.

Other findings the room ticket needs:

- **`pending_location` is alive, and is now only a bootstrap seed.** Its writers are exactly the two seed helpers (`GameState::inject_scenario_logs`, `MessageService::build_fresh_initial_state`); `log_movement_completion` clears it. Ticket 15's Answer still holds. **`docs/plans/subplan-c-mapless-enablement.md` does not hold**: every path it cites is gone (pre-hexagonal layout), Subplan B never shipped (`MovementJson.destination` is still one field, no `find_room_in_world_map`, no `has_map`), migration v15 is taken (`user_version` is at 26), and its `push_message`/`log_movement_completion` design **contradicts** the shipped ticket-15 behaviour — it would re-label the next Message (failing scenario 1.14) and make the moving Narration name the room the turn *started* in. `subplan-b-quantifier-field-split.md` is stale on the same point.
- **Old Games.** The only stored header is `message_swipes.location_header`. `plumbing.rs` neither adds nor backfills it: the column is born with the table (v9, `16bef514`), and `git log -S"ADD COLUMN location_header"` is empty across all branches, unlike `replay`/`impersonated`/`guide_text`. Inferred: a pre-fix Game shows the header on the Message *after* the moving Narration and none on the Narration itself, with no reader-side way to tell.
- **Nothing tests the mismatch.** Scenario 9.8 and 22.5 assert header strings only, never the room the switch restored.
- **Flagged contradiction:** `docs/diataxis/explanation/storage_design.md` §Per-swipe state binding ("Each swipe keeps the Snapshot of the state that its own narration produced") is **false in the code** — the pointer is pre-Quantifier. This is the root of the room ticket's mismatch and the reason option B's "result Snapshot" does not exist. `docs/diataxis/reference/game_flow.md` §Movement Branch is accurate after the ticket-15 fix, false at `d6a047cd`.
- **Unrelated tangent, recorded so it is not lost:** `AppState::render_story_log` takes the **first** 50 entries (`entries.into_iter().take(MAX_LOG_DISPLAY)` over the oldest-first `ORDER BY id ASC` history). A snapshot-diff render walking neighbours would have to decide what happens at that cut.

**Open / not determined.** Whether any deployed DB has a `message_swipes` table without the column (needs one `pragma_table_info` on an old DB; git history says it cannot). Whether pre-fix mis-placed headers are worth a migration (no version marker exists to detect them). Untested: the arrival Narration draining a leftover `pending_location`.
