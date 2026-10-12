# Apply or drop the reshape flags from the tickets 15/16 comment pass

Type: task (AFK)
Status: open
Blocked by: —

## Question

The comment pass over the uncommitted tickets 15 and 16 change set deleted 77 comment lines whose reason the code could carry, and offered a reshape for each. Dispose of all 17: apply each one, or drop it with a reason.

## Context

- Source: the comment pass run on 2026-10-11 by lead session `01a12d46-9ca5-7568-beff-572c92e3490a`, following `.agents/skills/chronicler-comment-fixer/SKILL.md`. Finder output `--uncommitted` fell from 455 to 356 comment lines. The five group reports live in that session's implementer results and its final finder output at `tmp/cmt-final.txt` (transient), so every item is restated below.
- Precedent: [Apply the six reshape flags the comment pass offered](16-apply-the-comment-pass-reshape-flags.md) disposed of the previous batch in this same shape. This ticket is that batch's successor, not a re-run of it.
- **Type.** A `task (AFK)`, not `grilling`. No item asks for a choice between product behaviours: each is either a mechanical refactor already covered by tests, or a fact settled by reading the code. The two judgement items (F2, F11) can record their judgement in the answer. The four recommended drops are dropped by rules this map already states, not by preference.
- Related: [Decide whether the comment pass renames or gates instead of delete-and-flag](../../retro-issue-bucket/issues/44-comment-pass-rename-or-gate.md) decides whether the pass applies a flag itself. This ticket disposes of this batch either way.
- **Warning.** A previous pass deleted two `// MEASURE_CLAMPED_TEXT` lines that are code inside a JavaScript probe in a Rust raw string; clippy passed and only the browser tier caught it ([Stop the comment pass from deleting code inside string literals](../../retro-issue-bucket/issues/49-comment-pass-code-inside-string-literals.md)). F14 rewrites that same marker, so that session must run `python build.py browser`.
- The comment named in each item is the reason the item exists. Keep the shape, and the comment has to stay.
- Sizes are measured occurrence counts, not estimates. "Recommend" is the pass's opinion; the session that runs this ticket may overrule it, but must record the reason.

## Items

| ID | Item | Where | Size | Recommend |
| --- | --- | --- | --- | --- |
| F1 | One `warn_orphaned_games(conn, label)` helper for the three copies of the orphan-repair warning block | `storage/utils/plumbing.rs:413`, `:593`, `:649` | SMALL | APPLY |
| F2 | Read `is_deleted` from column 5 instead of hardcoding `0` | `storage/models/message.rs`, `DbMessage::from_row` | SMALL | INVESTIGATE |
| F3 | Named struct for the 4-tuple expectation tables | `storage/utils/plumbing_tests.rs` (v22 and v26 tables) | SMALL | APPLY |
| F4 | `set_user_version_back(conn, n)` helper naming the legacy shape | `storage/utils/plumbing_tests.rs` (6 call sites) | SMALL | APPLY |
| F5 | Bind the stored-inputs precedence to a named value | `action_pipeline/core.rs`, `ActionPipeline::run_from_input` | SMALL | APPLY |
| F6 | Name the retry history window (`HistoryWindow`, or `history_len_for`) | `action_pipeline/retry.rs`, `reconstruct_retry_state` | SMALL | APPLY |
| F7 | Move `persisted_a_swipe` onto `RetryTarget` | `action_pipeline/retry.rs`, `durable_world_after_failed_redo` | SMALL | APPLY |
| F8 | Move the "snapshots exclude messages" harness contract onto the builder | `action_pipeline/options_tests.rs:84` | TRIVIAL | APPLY |
| F9 | `StoredInputs` struct for `(impersonated, guide_text, impersonate_direction)` | `state/game_state.rs`, `Message::set_stored_inputs`, `fixtures.rs` | MEDIUM | DROP |
| F10 | `last_input_text` returns the stored guide text when the last turn is guided | `domain/model/message_history.rs` | SMALL | APPLY |
| F11 | Derive the pending room write instead of storing it | `state/narrative_state.rs`, `arriving_location_write` | MEDIUM | DROP |
| F12 | Named struct for the `(Router, AppState, u64)` a fixture returns | `tests/http/swipe_new.rs:28` | SMALL | APPLY |
| F13 | Name the quantifier local `quantifier_tripwire` | `tests/http/swipe_new.rs:507` | TRIVIAL | APPLY |
| F14 | Make the substitution target non-comment-shaped (`__MEASURE_CLAMPED_TEXT__`) | `tests/browser/stub/failure_display.rs:797`, `:798` | SMALL | DROP |
| F15 | Key canned responses by role/agent instead of one shared `call_index` | `llm/providers/mock.rs`, `with_prompt_responses` | SMALL | APPLY |
| F16 | Seed `room_npcs` from the built map's room | `test_support/test_data_builder.rs:96` | SMALL | APPLY |
| F17 | Rename `evaluate_triggers` to carry its world-wide scope | `application/agents`, `evaluate_triggers` | TRIVIAL | DROP |

### F1–F4 (storage)

The orphan-repair `warn!` block appears three times with only the migration label changed. F1 is what the deleted header comment was justifying.

F2 asks a question the code does not answer: `from_row` hardcodes `is_deleted: 0` although the column exists since v9. Settle whether the reading `SELECT` already filters on that column. If it does, apply the change and note that it is defence in depth; if a soft-deleted row can reach `from_row`, apply it as a fix. Either way the answer records which. `tests/storage/message_storage.rs` covers soft-delete, restore and purge.

Recommended drops in this group: none.

### F5–F8 (pipeline)

F5 removes the comment `// A redo falls back to the retry target's stored inputs.`, which exists because a three-part boolean guard hides the precedence. F6 removes `// The tail re-appends the target from \`retry_target\`, so it must not stay in history: an anchor that IS the target truncates exclusively.` F7 removes the four-line `Attempt` variant doc on `DurableWorldAfterFailedRedo` (the pass trimmed it from eight lines on 2026-10-11; the enum and the ruling stay).

F7 changes no behaviour: tier-1 scenarios 11.9 and 11.10 pin it, and `retry_tests.rs` must stay green without edits.

F8 is a doc move: the kept line `/// Seeded apps reload with an empty messages table (snapshots exclude messages), so tests needing scene history run a narration turn first.` describes a harness fact one test carries for every test.

### F9–F11 (domain)

F9 (recommend DROP) duplicates the map's *Not yet specified* item **One type for a Swipe's stored inputs**, which proposes the sum type `Plain | Guided(String) | Impersonated(Option<String>)` rather than a struct. Two competing designs for the same three values is worse than one. Owner: promote that map item.

F11 (recommend DROP) removes the field the kept doc `/// Deferred write for a Swipe whose row already exists; the next save applies it as an update.` describes. That field is exactly "where the room lives" mid-generation, which [Decide where the room a generation arrives in lives](../../turn-state-model/issues/03-decide-where-the-room-lives.md) owns, and the map's *Not yet specified* carries the stale-header risk the derivation would have to cover. Owner: that ticket.

F10 keeps one of the two "a guided turn appends no Input row" comments instead of two, because `message_history.rs:124` and `message_service.rs:254` explain the same invariant at two layers.

### F12–F16 (tests and test support)

F12 removes `/// Returns the app, its state and the event Message's id.`. F13 removes `// The seeded response is a tripwire: a stray quantifier call would move the player back.`. F16 removes `// room_npcs defaults to ["npc_1"]; the shopkeeper is not in any room's`.

F15 removes the two `// Consumed in call order: …` comments the pass wrote into `swipe_new.rs`: canned text is matched to calls by one shared `call_index`, so a fixture's meaning lives in the order of two vectors. A keyed shape removes that coupling.

F14 (recommend DROP) is the same code as open retro ticket 49, which is a `grilling (HITL)` ticket that decides between "make it unrepresentable", "teach the finder about strings" and "widen the verify step". Its owner decides the shape; doing it here would pre-empt that ruling. Owner: ticket 49.

### F17 (recommend DROP)

Out of this map's domain. The map says backend code changes only where a dashboard finding needs it, and no finding needs this rename. The retained comment in `tests/http/actions.rs` states a fact the test relies on (`evaluate_triggers` iterates every world NPC, not just `npcs_in_area`), so the test keeps its explanation. If the rename is wanted, it belongs to an engine-naming effort.

## Sessions

Two sessions, because the map's one-ticket-one-session-one-commit rule cannot hold for both halves:

- **Session A, test tier only.** F3, F4, F8, F12, F13, F15, F16. Gate: `python build.py --no-browser`, then `python build.py browser` (F15 changes a stub the browser fixtures can reach).
- **Session B, `src/` refactors.** F1, F2, F5, F6, F7, F10. Gate: `python build.py`.

Either session may apply a drop with a recorded reason instead. Split the file if the coordinator wants one commit per item group.

## Done when

- Each of the 17 items is applied or dropped with a reason.
- `python build.py` is green for whatever was applied, and `python build.py browser` is green for the applied test-tier items.
- Each behavioural item names the test that pins it unchanged: F2 (`tests/storage/message_storage.rs`), F6 and F7 (tier-1 scenarios 11.9 and 11.10), F10 (`message_service_tests.rs`).
- Each dropped item names its owner: F9 the map's *One type for a Swipe's stored inputs*, F11 ticket 03, F14 retro ticket 49, F17 the engine-naming effort.
- Each item's comment is either gone because the code carries its reason, or still there and the item is dropped with a reason.
