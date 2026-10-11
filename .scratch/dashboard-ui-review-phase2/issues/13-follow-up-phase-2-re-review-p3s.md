# Follow up on small issues found in the phase-2 re-review

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Fix each P3 from the [2026-10-11 re-review](../re-review-2026-10-11.md), or drop it with a reason.

## Items

- **S5** The command form's inline-error "Details" toggle is drawn as a second Send button (150x40), and its message is 16px. The `#command-form button` rule in `assets/styles.css` matches every button in the form; scope it to `#submit-btn`. Shot `tmp/p2-07/08-unreachable.png`.
- **S6** The location header lands one Message late: movement sets `pending_location`, and the next Message takes it (`src/domain/model/state/game_state.rs:123`, `:190`). This is a narrative-pipeline rule, not dashboard code. Decide whether the arriving narration should carry the header, or rule it out of scope with an owner.
- **S7** `truncated_preview` (`src/adapters/driving/http/builders/presets.rs`) appends "…" after a full stop ("intelligence.…"). Drop the trailing punctuation before the ellipsis.
- **S8** The text-check preview draws the Original with a line-through while its note says nothing changes unless the player chooses "Send with edits". Decide whether the line-through stays.

From [Make event Retry restore the trigger context and never roll the game back](09-fix-event-retry-snapshot.md):

- **F09-1** A retried event Swipe has no location header, so a swipe switch adds or removes the room name. Same class as S6; decide them together.
- **F09-2** `check_retry_anchor` still checks the anchor Narration's snapshot, while event Retry now reads the event Swipe's own snapshot. Kept on purpose (I.7: a Message that points at a missing snapshot is a data-integrity error). Confirm or drop.
- **F09-3** If a redo saves a Swipe but the room move and trigger effects are never saved, the game keeps the rewound state. Rare. Fix or drop with a reason.

From [Report a failed trigger continuation without a story-log Message](10-report-failed-trigger-without-message.md):

- **F10-1** Trailing `System` notices still hide Retrigger and the swipe controls, and reach the Narrator as `System: …`: the NPC-detection notice (`pipeline_run.rs`, the last row on any trigger-less turn with an empty or non-high Quantifier), the options-failure notices (`action_pipeline/options.rs`, one carries raw engine text) and "[System] Entered unknown location" (`game_state.rs`). Ticket 10 kept them on purpose (see its Answer). Decide whether a trailing notice should still hide the controls, or rule it out with a reason.
- **F10-2** `retrigger.rs` and `MessageService::switch_swipe` still use `messages.last()`, so a trailing notice counts as the last message. Align them with `Message::is_input_or_narration`, or drop with a reason.
- **F10-3** Rust identifiers remain in spec scenarios `docs/specs/retrigger.md` 14.5 and `docs/specs/swipe_new.md` 11.8 ("Spec vocabulary" rule).
- **F10-4** Optional: `tests/http/failure_display.rs` has no Trigger-role case for the failure banner.

From [Show the whole token-budget failure, remedy included](11-show-the-token-budget-remedy.md):

- **F11-1** The status sentence has no fallback when it grows past two lines: the cut is silent. Scenario 16.43 fails loudly for today's sentences only. Decide whether the clamp needs an ellipsis or a Details fallback.

From [Restore focus after preset and world deletes](12-restore-focus-after-card-deletes.md):

- **F12-1** A world delete refused with `WorldHasGames` answers 200 and replaces the row with an error row; focus lands on the neighbouring row or the panel, not on the error.
- **F12-2** A preset card GET that fails (`require_preset!` → 200 `div.error-message`) moves focus to the neighbouring card.
- **F12-3** Pre-existing race: `focusedBeforeSwap` (`assets/dashboard-panels.js`) is one global that every `htmx:beforeSwap` overwrites, so a 2–5 s poll inside htmx's settle delay can lose the saved element (estimated 2–4 % per click, never observed). Affects every `afterSettle` focus check.
- **F12-4** `scripts/prepare_review_bundle.py` files `assets/` under `docs.patch`, so a JS-only change reads as a docs change.

## Done when

- Every item is fixed or dropped with a reason.
- `python build.py` is green. Commit after user approval.

## Answer

All 16 items resolved: 11 fixed, 2 ruled out of the ticket as larger design work, 2 ruled with no code change, 1 dropped as rare.

| ID | Item | Outcome |
| --- | --- | --- |
| **S5** | The inline-error "Details" toggle is drawn as a second Send button (150x40), and its message is 16px | **Fixed.** The five `#command-form button` rules and the ≤768px responsive pair now target `#submit-btn`, and a new scoped rule puts the inline-error message on `var(--font-size-small)`. Tier 2: browser 16.19 (`read_inline_error_sizes`), which fails against the old CSS. |
| **S6** | The location header lands one Message late | **Ruled out of this ticket.** A narrative-pipeline rule, not dashboard code: the room name would have to move onto the message that arrives at the room, which changes `push_message`/engine-commit ordering, has no existing `location_header` test to lean on, and collides with `docs/plans/subplan-c-mapless-enablement.md`, which already plans to delete `pending_location`. Proposed as its own ticket. Ticket 14 must treat S6 as ruled out, not fixed. |
| **S7** | `truncated_preview` appends "…" after a full stop ("intelligence.…") | **Fixed.** Trailing whitespace and punctuation are trimmed in one pass (`builders/presets.rs`). Unit: `presets_tests.rs`, including the `"stanza . "` → `"stanza…"` case, which fails on the old order. |
| **S8** | The text-check preview strikes through the Original while the note says nothing changes unless the player chooses "Send with edits" | **Fixed.** The whole Original is rendered in one `<span>`, so the strike covered every visible word and read as "this will be deleted" even though "Send Original" is the primary action. The rule was removed; the note was left alone. No test: pure CSS, permitted by the map Notes. |
| **F09-1** | A retried event Swipe has no location header, so a swipe switch adds or removes the room name | **Fixed.** `reconstruct_retry_state` seeds `pending_location` from a sibling Swipe that carries the room name, so a redone Swipe matches its siblings. Unit (`retry_tests.rs`, seed plus no-invention case) and tier 1 SCENARIO 10.8 (`tests/http/swipe_new.rs`); both fail with the seed disabled. |
| **F09-2** | `check_retry_anchor` still checks the anchor Narration's snapshot | **Confirmed as-is, no code change.** The anchor guard is a strict superset — it can refuse an event retry whose own snapshot is fine — and the event branch validates its snapshot again at load; the case is the data-integrity error I.7 names. Both pins (`retry_tests.rs`) stay green. |
| **F09-3** | A redo saves a Swipe but the room move and trigger effects are never saved | **Dropped as rare, not fixed, and the drop is narrower than the ticket implies.** The window IS reachable: a failure between the pre-quantifier narration persist (which saves the new Swipe) and the post-engine Snapshot — an engine-commit error, or a write that fails once and then recovers — keeps the rewound world, deliberately. Reason recorded in the `retry.rs` doc comment; no test for it. |
| **F10-1** | Trailing `System` notices still hide Retrigger/swipe and reach the Narrator as `System: …` | **Ruled out of this ticket.** The ruling is a design decision (which entry owns the controls, and whether a notice should be prompt-visible at all), not a P3 fix. Proposed as its own ticket. |
| **F10-2** | `retrigger.rs` and `switch_swipe` still use `messages.last()` | **Ruled out together with F10-1.** The two must land together: done alone, it loosens a server guard whose control the UI still hides, so nothing user-visible changes. |
| **F10-3** | Rust identifiers in spec scenarios `retrigger.md` 14.5 and `swipe_new.md` 11.8 | **Fixed** for those two lines (reworded in domain language; scenario numbers and test tags untouched). The wider identifier sweep in those files is a follow-up docs ticket, not part of this one. |
| **F10-4** | No Trigger-role case for the failure banner | **Fixed.** Tier 1 SCENARIO 38.8 (`tests/http/failure_display.rs`) plus its spec line. |
| **F11-1** | The status sentence is cut silently past two lines | **Fixed.** A two-line clamp with a visible ellipsis; it needed the sentence in its own text wrapper (`utils/error.rs`, mirrored in `dashboard-errors.js`) and `text-align: left`, because Chrome draws no ellipsis for right-aligned clamped text. Review found the wrapper had disarmed the 16.41/16.43 "reads in full" guards; both now measure the clamped element and fail again on an over-long sentence. Tier 2: the new `layout.rs` clamp check plus 16.41/16.43. |
| **F12-1** | A world delete refused with `WorldHasGames` answers 200 and focus lands on the neighbour, not the error | **Fixed.** `dashboard-panels.js` focuses the error fragment that replaced the row. Tier 2: browser 29.5. |
| **F12-2** | A preset card GET that fails answers 200 and moves focus to the neighbouring card | **Fixed**, with the same mechanism. Tier 2: browser 28.5. |
| **F12-3** | `focusedBeforeSwap` is one global that every `beforeSwap` overwrites, so a poll can lose the saved element | **Fixed.** The saved element is paired with its swap target, and `afterSettle` acts and clears only for that target; the new error-fragment fallback is limited to row/error targets. Tier 2: 29.5 synchronously dispatches a poll's settle inside the refused delete's swap window — it fails without either guard. Residual, documented: two overlapping row/panel swaps can still overwrite one saved slot; narrower and pre-existing. |
| **F12-4** | `scripts/prepare_review_bundle.py` files `assets/` under `docs.patch` | **Fixed.** A new `assets` area before `docs`; the generated tables follow. Python tests updated. |

**Decisions and deviations worth recording.**

- F12-1/F12-2 make the error fragment focusable client-side (`tabindex="-1"` applied by the JS at the moment it becomes the landing spot) instead of changing the server-rendered `error_fragment`: same acceptance, and the server-side route would churn ~26 exact-markup assertions and 11 spec lines for no behavioural gain.
- F11-1's `text-align: left` is a deliberate, visible change: sentences that wrap to two lines were right-aligned and are now left-aligned. Chrome renders no ellipsis for right-aligned clamped text, so the cut signal requires it. Screenshot evidence in `tmp/screenshots/`.
- Review findings rejected, with reasons: (a) render the layout check's disclosure markup from the stub instead of by hand — the real server markup is measured by 16.41/16.43 after the B1 fix; (b) fix the `pending_location` leak on a game switch mid-redo — a rare path that is already broken, and the alternative costs an extra persist or a change to the cancellation path; (c) sweep all remaining Rust identifiers out of `retrigger.md`/`swipe_new.md` — a separate docs ticket.

**Tests, by tier.**

- Unit (`src/**/*_tests.rs`): `builders/presets_tests.rs` (S7), `action_pipeline/retry_tests.rs` (F09-1 pins), `settings/templates/settings_tests.rs` (F11-1 disclosure shape).
- Tier 1 (HTTP E2E): `tests/http/swipe_new.rs` SCENARIO 10.8, `tests/http/failure_display.rs` SCENARIO 38.8.
- Tier 2 (browser): `tests/browser/stub/failure_display.rs` 16.19/16.41/16.43, `tests/browser/stub/layout.rs` clamp check, `tests/browser/stub/worlds.rs` 29.5, `tests/browser/stub/prompt_presets.rs` 28.5.
- Python: `scripts/tests/test_prepare_review_bundle.py` (F12-4).
- Specs updated: `swipe_new.md`, `failure_display.md`, `retrigger.md`, `browser_dashboard.md`, `browser_worlds.md`, `browser_prompt_presets.md` (feature-spec validator 275/275 covered).

**Gate.** `python build.py` green on the frozen tree (`logs/build_20261011_185531.log`): all 19 steps OK; architecture `1 passed, 0 failed`; guardrails `162 passed, 0 failed`; integration `1625 passed, 0 failed, 2 skipped`; browser `29 passed, 0 failed, 0 skipped`.

**Follow-ups.**

1. **S6 + F09-1's class — location-header semantics.** Move the room name onto the arriving message (or delete `pending_location` per `docs/plans/subplan-c-mapless-enablement.md`), and decide the same question for the non-event retry paths: a guided turn's narration takes the previous turn's `pending_location`, so a retried guided narration after a movement loses the room name the same way F09-1 did. Needs a narrative-pipeline ruling, not a UI fix.
2. **F10-1 + F10-2 — trailing `System` notices.** Decide whether a trailing notice should hide Retrigger/swipe and whether it should stay prompt-visible as `System: …`; then move the controls onto the last input-or-narration entry and align `retrigger.rs`/`switch_swipe` with `Message::is_input_or_narration` in the same change.
3. `app_state.rs` renders the OLDEST 50 story-log entries (`take(50)`) while the retrigger/swipe/delete controls hang off `loop.last`, so in a game longer than 50 Messages the controls attach to the 50th-oldest entry.
4. The `pending_location` leak on a game switch mid-redo (review finding C5).
5. Sweep the remaining ~29 Rust-identifier lines out of `swipe_new.md` / `retrigger.md` (`tests/STRATEGY.md` "Spec vocabulary").
6. `games/templates/games.rs` hand-writes a `<div class="error-message">` instead of using `error_fragment`, so it silently misses changes to that fragment; the stub has no 200-refusal mode for games rows.
7. `dashboard-panels.js` comment nits: it cites "Scenario 29.5" (goes stale on renumber) and describes `detail.target` for `beforeSwap` where the code reads `detail.elt`; `text-overflow: clip` on `.error-disclosure-text` is inert under `white-space: normal` plus the clamp.

Committed on `phase2-13-p3-follow-ups` as the fix commit plus this ticket; not pushed.

### Follow-ups from the after-plan review (2026-10-11)

8. **Scenario 9.2's room name.** `docs/specs/swipe_new.md` names the room `"room1"` while `TestMap::two_rooms("room_1", …)` and `tests/http/swipe_new.rs:163-167` use `room_1`.
9. **`docs/diataxis/reference/game_flow.md:25`** still uses the deprecated term in a Mermaid label (`Options[turn-end Options rewrite]`).
10. **`tests/STRATEGY.md:128-129`'s layout-only carve-out against 16.19 and 16.43: KEEP AS IS.** 16.19 has five content assertions, so it is not layout-only; 16.43 pins the drawn sentence per `GenerationFailureKind`, which no other check covers, and `:94-96` expects cross-tier overlap while `:98-100` binds within a tier. The optional split of the size measurements into `tests/browser/stub/layout.rs` stays a follow-up, not a fix.
11. **`MEASURE_CLAMPED_TEXT` is duplicated** between `tests/browser/stub/layout.rs` and `tests/browser/stub/failure_display.rs`, done two ways; one shared copy in `stub/support.rs`.
12. **Two seeders re-implement the pipeline's save order:** `seed_event_flow` (`src/test_support/fixtures.rs`) and `setup_event_flow` (`retry_tests.rs`). The unit seeder could call the shared one once its 10 callers are checked.
13. **`message_location_header`** (`src/application/pipeline/action_pipeline/retry.rs`) reads only `message.swipes`; move it onto `Message` beside `location_header()`.
14. **Three sites each search for the last input-or-narration Message** — `message_service.rs`'s new `last_input_or_narration_idx`, `retry.rs`'s target rule and `message_history.rs`'s `last_turn_is_retryable`. One shared "last index" helper would make the "cannot drift apart" claim structural.
15. **`refusal_message_without_layer_prefix_for_validation_and_config` (`utils/error.rs`) is 63 characters**, and the length is the price of the truth: only Validation and Config strip the layer prefix, so a shorter name needs a qualifier the name no longer carries.
16. **Prose "turn" survives** in code comments, `fixtures.rs` doc blocks and about a dozen pre-existing test names (`MessageHistory::last_turn_is_retryable`, `rewrite_options_after_turn`, …); the identifier rename covered only the names this branch added or changed.

The comment pass over-deleted: the docs it dropped on `refusal_message` and on `Message::set_event_header` (now `#[cfg(test)] pub(crate)`) carried constraints their names did not. Both were corrected in place — a rename, and a visibility gate — not re-commented.
