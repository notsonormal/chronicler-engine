# DOM stuck-state test gap

Linked asset for [Close the browser-test gap for stuck DOM states](../issues/04-browser-test-gap-stuck-dom-states.md).

Source: finding F25, row A19 of the 2026-10-04 reflect sweep (divergent lens row 10). F25 said
the stuck-state bugs the window kept producing went unverified while "full gate green" stood
as proof, because the stub fixtures do not model DOM state.

Facts here were re-checked on 2026-10-04 against the committed tree (`8771d7c1`) and confirmed
by `python build.py browser` — 62 passed, 0 failed.

## What A19 named

A19's evidence names three stuck-state bugs: "action area stranded, poll frozen on second
edit, snapshot restore forgetting 'Generating…'". Two more share the class in the same window
(ticket 27's failed save and failed new-swipe/retrigger).

## Coverage today

| # | Bug | What got stuck | Ticket that fixed it | Test that covers it today | Tier |
|---|---|---|---|---|---|
| 1 | Text-check confirm strands the command input (04.C1, P1) | The server-rendered `#command-form` came back with `input disabled`; nothing re-rendered it when generation ended, so the input stayed dead while the status read Ready. Reload was the only recovery. | [Rework the action area so a text check cannot strand it](../../dashboard-ui-review/issues/_resolved/42-rework-the-action-area.md) | `tests/browser/dashboard.rs::test_confirm_preview_leaves_form_ready_and_usable` (16.12) asserts `!input.disabled` and a fresh command; `tests/browser/stub/dashboard.rs::test_primary_button_locks_and_unlocks_after_confirm` (16.9) asserts the form/status nodes survive and the button unlocks | 3 (16.12), 2 (16.9) |
| 2 | Swipe switch stuck on "Generating narration..." (06.6.1, P1) | A Swipe's snapshot was written mid-narration, so the restore carried `Generating`/`Narrating`; `/status/generating` answered `narrating` for good, Send became a disabled Stop, and Enter produced no request. Reload did not clear it. | [Reset the generation status and the options dock when a swipe switch restores a snapshot](../../dashboard-ui-review/issues/43-reset-status-and-dock-on-swipe-switch.md) | `tests/browser/stub/swipes.rs::test_switch_swipe_leaves_dashboard_ready_and_drops_options` (37.1); `tests/http/swipe_switch.rs::test_switch_swipe_leaves_the_game_idle_http` (36.1) and `::test_switch_swipe_drops_the_offered_option_set_http` (36.2); unit `message_service_tests::test_switch_swipe_restores_a_settled_snapshot` | 2 (37.1), 1 (36.x), unit |
| 3 | Second Edit click freezes the story-log poll | Clicking Edit on a second entry re-ran `showEditForm`; `pausePolling` overwrote `originalTrigger` with `"none"`, so the poll never resumed and the first entry stayed in edit mode. | [Stop editing a second entry from freezing the story-log poll](../../dashboard-ui-review/issues/38-edit-another-entry-freezes-poll.md) via [Keep DOM state and focus through in-place swaps](../../dashboard-ui-review/issues/65-keep-dom-state-through-swaps.md) | `tests/browser/stub/story_log.rs::test_edit_locks_the_other_entries_edit_controls` (30.15) pins the lock that makes the second click unreachable; `::test_successful_save_holds_the_edit_lock_until_the_poll` (30.17) only passes if the resumed poll re-renders, so it also proves the trigger was restored | 2 |
| 4 | Failed message save pauses the poll forever (05.F3, P1) | `resumePolling()` lived only in `.then()`, so a failed save left `#story-log` on `hx-trigger="none"` forever; the textarea stayed and ✓/✗ went dead. | [Recover from a failed message save instead of freezing the story log](../../dashboard-ui-review/issues/_resolved/27-recover-failed-message-save.md) | `tests/browser/stub/story_log.rs::test_failed_save_restores_entry_and_resumes_polling` (30.4) | 2 |
| 5 | Failed new-swipe / retrigger leaves "Thinking..." with a disabled Stop (05) | The shared `submitGenerationRequest` failure path never reset the status, so the dashboard stayed on "Thinking..." with a dead button. | Same ticket 27; retrigger wiring pinned by [Drive the retrigger control in a browser test](../../dashboard-ui-review/issues/_resolved/40-drive-retrigger-in-a-browser-test.md) | `tests/browser/stub/story_log.rs::test_failed_retry_clears_pending_status_and_re_enables_send` (30.5) and `::test_failed_retrigger_posts_to_retrigger_and_recovers` (30.10) | 2 |

Adjacent stuck-state finding from the same window, also covered: a stale option could be
clicked mid-generation (`useOption` checked only `input.disabled`), fixed by ticket 43 and
covered by `tests/browser/stub/options.rs::test_option_click_during_generation_does_not_submit`
(26.5).

## Why stub fixtures miss it

The tier-2 stub serves the shipped `assets/index.html` verbatim (`stub_server.rs`,
`DASHBOARD_SHELL`), so client-JS stuck states *are* reproducible at tier 2 — the poll freeze
(30.15–30.17) and the failed-save freeze (30.4) are both tier 2. What the stub cannot model is
server-rendered fragment shape: it serves canned markup for the polled fragments.

[Ticket 50](../../dashboard-ui-review/issues/50-render-stub-fixtures-from-templates.md) closed
that hole for the two fixtures the accessibility work churned — the story log and LLM Messages
now render through `NarrativeLogTemplate` / `LlmMessagesTemplate`, and the options dock and
text-check preview already did. A shipped-template hook change therefore reaches those stub
tests.

The remaining hand-copied fixtures are `action_area`, `header`, `settings`, `prompt_presets`,
`worlds`, `games` and `visual_sidebar`. Ticket 50 deferred them deliberately. For the strand
bug (#1) the exposure is small: its shipped markup is covered at tier 3 (16.12), so a
`ActionAreaTemplate` drift would have to escape tier 3 to go unnoticed. The tier-2-only
action-area scenarios (16.10, 16.11, 16.13–16.15) still read the canned fixture.

## Verdict

**None of the stuck-state bugs A19 names lacks an automated test today.** All five now carry a
browser test, four of them at tier 2 over the shipped shell. The gap A19 described was real
when it was written; it closed through the dashboard-ui-review tickets 27, 42, 43, 65 — each of
which made "a browser test covers X" part of its Done-when.

## Residual same-class gap (uncovered)

One stuck-state path in the class still has no test, and may not be reachable from the page:

**A persisted `Generating` status with no live generation slot has no integration test, and the
dashboard may not be able to send the action that heals it.**

- The mechanism exists: `GenerationGate::heal_stale` resets a persisted `Generating` when no
  slot owns the game, and `docs/diataxis/explanation/two-state-channels.md` documents recovery
  as happening "on the next `process_action`".
- The wiring is untested: `heal_stale` is called only from the action path
  (`action_pipeline/action.rs`) and bootstrap (`wiring.rs`). Its only test is the unit test
  `gate_tests::test_heal_stale_resets_generating_status_when_no_active_slot`; no HTTP or
  browser test exercises the recovery end to end.
- The poll does not heal: `generating_status_handler` reads the persisted status and reports it
  (`endpoints.rs`). A stale `Generating` therefore keeps `/status/generating` on `narrating`.
- The client blocks the healing action: `onStatusPoll` maps `narrating` to `.status.thinking`,
  `statusIsGenerating()` then returns true, and `applyActionState` disables `#submit-btn`. The
  input stays enabled, but the submit button is the form's default button, and A19's own
  evidence for bug #2 recorded that a real CDP Enter produced no request while the button was
  disabled.
- Reachability is narrow but real: `GenerationGuard::Drop` releases the slot on a mid-flight
  panic without resetting the persisted status, so a panicked generation leaves exactly this
  state. A server restart heals it via bootstrap; a page reload does not.

### Proposed test

Tier 1 (`tests/http/actions.rs`): seed a snapshot whose status is `GenerationStatus::Generating`
with no live slot, POST a command, and assert the action runs and the persisted status leaves
`Generating`. This pins the documented self-healing wiring, which today only a unit test covers.

Tier 2/3: a page whose status reads generating must be able to reach the recovery — either the
command form can still submit, or a recovery control is present. If it cannot, that is a product
defect, and the fix is a design choice (heal in the poll, which contradicts the documented
single-writer rule; let the client submit while generating and let the server answer
`ConcurrentGeneration`; or add a recovery control wired to `/status/reset-generating`).

Because the fix is a design decision, the work is graduated as a grilling ticket:
[Recover a dashboard stuck on Generating with no live generation](../issues/09-recover-stuck-generating.md).

## Not graduated

- **The deferred stub-fixture conversions** (`action_area`, `header`, `settings`,
  `prompt_presets`, `worlds`, `games`, `visual_sidebar`). Ticket 50 owns that decision; it is a
  fixture-fidelity concern, not a stuck-state bug, and duplicating it here would fragment the
  work.
- **A direct test of the `pausePolling` double-pause guard.** The guard is defense in depth; the
  UI lock makes the path unreachable, and 30.15 already fails if the lock is removed. A test of
  the unreachable guard would pin the mechanism but no user-visible behaviour.
