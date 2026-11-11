# Tests (step 5/7) and consistency (step 8)

Source: scout `tests-and-consistency` (session e3690c86). Lead spot checks marked "lead: verified".

## Bad tests

| # | Finding | Where | Sev |
|---|---|---|---|
| T-1 | 14.1 test asserts only `400`; the "no generation is started" Then is unchecked (lead: verified, no model and no story assertion) | `tests/http/retrigger.rs::test_retrigger_no_trigger_context_returns_400_http` | medium |
| T-2 | Walker detection cases assert only the violation count; a case with `expected: 2` passes on two wrong hits | `tests/infrastructure/guardrails/walker_tests.rs::run_cases` | medium |
| T-3 | `dock_notice` helper answers `None` for "no notice" and for "markup renamed"; `None` assertions can pass on a rename | `tests/http/options.rs::dock_notice` | low |
| T-4 | 39.1 test covers four Given/When/Then blocks in one body | `tests/http/dashboard.rs::test_generating_status_answers_from_live_registry` | low |
| T-5 | `wait_settled(&state, 1000)` for 2–3 call Jobs; `settings.rs` uses 2000 for the same shape (latent flake) | `swipe_new.rs`, `actions.rs`, `retrigger.rs` | low |
| T-6 | Tier-1 observes stored state through `Storage::get_preset` (STRATEGY.md calls this a leak) | `tests/http/prompt_presets.rs` two asserts | low |

## Missing tests (production branches with no test at any tier)

| # | Branch | Where | Sev |
|---|---|---|---|
| M-1 | `Refusal::BadRequest("Swipe index out of bounds")` — no test, no scenario; a client reaches it with `POST /message/:id/swipe/99` (lead: verified by rg) | `gate.rs` `switch_swipe` | high |
| M-2 | "Message not found" on edit — lib handler test only, no scenario | `gate.rs` `edit` | medium |
| M-3 | "The story keeps its last message" on delete — lib handler test only, no scenario | `gate.rs` `delete_last` | medium |
| M-4 | The save-failed line ("The new text could not be saved.") — no test, no scenario (lead: verified by rg); `TestFailureHandle::set_write` exists but tier 1 never uses it | `job.rs::end`, `generation_status.rs` | medium |
| M-5 | `PromptTooLong` line — no tier-1 test, no scenario | `generation_status.rs`, `prompting/utils/context.rs` | medium |
| M-6 | The Job's own read failures (the story, the current state, budget) write a missing-data line, untested; event-record ones are unreachable | `job.rs` | low |
| M-7 | `JobWriter::state_at`/`tail_of`, `message_kind_of` failure branches untested; `tail_of` empty-story is unreachable | `gate.rs` | low |

## Duplicate families worth a helper (from the 438-pair report)

| # | Family | Helper |
|---|---|---|
| D-1 | Story read `game_view_query.story_messages(active_game_of(&state))` + filter (79 sites; private copies in `races.rs`, `swipe_new.rs`) | new `support/` story-read helper |
| D-2 | Busy refusal read: POST → body → `contains("Still thinking...")` | `assert_still_thinking`, export `STILL_THINKING` |
| D-3 | Triggered-NPC app arrange (7+ sites; private `triggered_npc()` in `races.rs`) | `app_with_a_triggered_npc` |
| D-4 | `settings.rs` 20.25/20.26 repeat a ~45-line arrange | `long_story_app(window)` |
| D-5 | `post_form_request` copies in `prompt_presets.rs` and `settings.rs` | existing `support::post_form` |
| D-6 | Two-Game arrange in `games_delete.rs`/`games_switch.rs` | `two_game_app()` |
| D-7 | Inline `NpcCard{.. triggers ..}` literals | `TestNpc` builder |
| D-8 | Agent test `world()` fixture duplicated across the two Agent contracts | optional |

Fine to keep: per-file read vocabulary, `wait_settled`+`wait_for_jobs` pairs, import blocks, walker tables, browser stub clones.

## Consistency

New code follows the patterns that stayed (Refusal mapping next to ApplicationError, `Response<Body>` handlers, `CurrentGame`, one `impl Storage` per file, failure injection seam, SCENARIO tags).

| # | Finding | Sev |
|---|---|---|
| K-1 | `/history` edit and delete refusals have no spec (same as M-2/M-3) | medium |
| K-2 | `Storage::stories()`/`docks()` handle pattern is new; older files use plain `impl Storage` methods; `DockReads` has one method (`GameReads` was deleted for the same reason) | low |
| K-3 | `generation_failure_summary` is now a one-line forwarder to `kind.line(raw)`; its clamp comment sits away from the table it explains | low |
| K-4 | `races.rs` R-59 test reads `storage.require_game` for arrange (allowed) | low, optional |
