# Map: dashboard UI review

Labels: wayfinder:map

## Destination

The dashboard has no open finding from the [UI review](review-2026-09-29.md). Each finding is fixed, or ruled out of scope with a reason. This includes new findings from the areas the first review skipped. The error and health display is redesigned (Theme 1). The [final re-review](issues/24-final-re-review.md) finds no new P1/P2, and `python build.py` passes.

The test rules the map's tickets rely on hold: `tests/STRATEGY.md` states one checkable rule for what a tier-1 test may observe, and the test standards docs match the code. This was added after the [test audit](assets/test-audit/), because this map's tickets write specs and tests by those rules.

## Notes

- **This map carries execution.** It overrides wayfinder's "plan, don't do": `task` tickets make the change, not only decide it. Decision tickets (`grilling`, `prototype`) graduate their implementation work into new tickets.
- **Domain:** the dashboard frontend. That means `assets/index.html`, `assets/styles.css`, and the Askama templates and handlers under `src/adapters/driving/http/`. Backend code changes only where a finding needs it.
- **Source of findings:** [review-2026-09-29.md](review-2026-09-29.md). Tickets cite findings by number (for example "finding 2.3") and screenshots by number (`tmp/ui-review/NN-*.png`, local only).
- **Tests:**
  - Place every test by the placement rule in `tests/STRATEGY.md`. The ticket answer names the tier.
  - Specs describe the system, not changes. Add or change a spec scenario only when the spec is incomplete or wrong.
  - Pure CSS and copy fixes need no test. Do update existing assertions that the fix breaks.
  - Until [Decide what a tier-1 test may observe](issues/31-decide-tier-1-observations.md) resolves, write new spec Givens and Thens in `CONTEXT.md` terms rather than field names. That holds under every option it weighs.
  - The test-rule tickets (31–34) can run apart from the dashboard tickets. They share `tests/` and `docs/specs/`, so check `git status` for overlapping edits first.
- **End of each execution ticket:** `python build.py` is green, then the user reviews the diff. After approval, commit through `/commit-and-push`. One ticket = one session = one commit.
- **Skills:**
  - grilling and prototype tickets: `/grilling`, `/domain-modeling`, `/prototype`
  - review tickets and visual checks: `/chronicler-ui-investigator`
- **Reference docs:** `docs/diataxis/reference/frontend/ui_design.md` (design tokens), `docs/diataxis/reference/frontend/dashboard.md`, `docs/diataxis/explanation/dashboard_design.md`.
- **User data:** review tickets run against the user's real `data/` and settings. Use throwaway games, worlds, and presets, and restore any changed setting before the session ends.
- **Final re-review:** every ticket added to this map must also be added to the `Blocked by:` line of [Final re-review of the dashboard](issues/24-final-re-review.md).
- **Small review follow-ups:** issues from code reviews or implementers that don't block a merge go under `## Items` in [Follow up on small issues found during review, round 2](issues/37-follow-up-small-review-issues-2.md), with their source ticket. Bigger ones get their own ticket.
- Other agents may work in the repo at the same time. Do not touch unrelated changes.

## Decisions so far

<!-- the index — one line per closed ticket -->

- [05](issues/05-review-create-save-delete.md) — review of create/save/delete flows; report at `tmp/ui-review/t05-report.md`; findings F1–F8 graduate to 25–29 and extend 08/16.

01 [Stop the story-log fragment nesting a second #story-log](issues/01-fix-nested-story-log.md): resolved — the narrative-log fragment no longer ships the `#story-log` wrapper (shell keeps sole ownership); tier 1 `test_story_log_fragment_declares_no_log_container` (scenario 8.5) + aligned stub fixture; commit 60d1ef65.

02 [Rebind action-area handles after swaps](issues/02-rebind-action-area-handles.md): resolved — use-time `#submit-btn` lookup + body-level MutationObserver delegation; tier 2 stub tests 16.9/16.10 (`tests/browser/stub/dashboard.rs`); commit 262a4857.

29 [Stop world creation from silently overwriting an existing key](issues/29-world-key-silent-overwrite.md): resolved — user create refuses an existing key (`Storage::create_world` → `WorldAlreadyExists` → 400 toast); bootstrap `seed_world` still replaces, InMemory now matches; tier 1 scenario 25.7 + storage pair tests; graduated [35](issues/35-sqlite-reseed-keeps-world-id.md).

03 [Keep the command form when a text-check result shows](issues/03-keep-command-form-on-text-check.md): resolved — ✓ results render into their own `#text-check-result`, never replacing the command form; tier 2 scenario 16.11; 16.9/16.10 rewritten to drive submit → preview → confirm.

30 [Stop the story-log delete test flaking under parallel load](issues/30-fix-flaky-story-log-delete-test.md): resolved — `wait_idle` saw persisted Idle before the generation slot was released, so follow-up actions were dropped as `ConcurrentGeneration`; the helper now also waits for the slot (all 87 call sites).

33 [Make the test standards docs match the code](issues/33-fix-test-standards-drift.md): resolved — ~30 phantom names and false claims fixed across the three test standards docs; `integration_test_standards.md` pruned 352→250 lines to pointers plus the rationale the code cannot state; guardrail message names `TestAppBuilder`.

27 [Recover from a failed message save instead of freezing the story log](issues/27-recover-failed-message-save.md): resolved — failed save/swipe/retrigger show the toast, revert the entry or status, and always resume polling; tier 2 scenarios 30.4/30.5; restoring entry actions on revert handed to 11.

35 [Keep the world id stable when SQLite re-seeds a world](issues/35-sqlite-reseed-keeps-world-id.md): resolved — SQLite re-seed now upserts in place (`ON CONFLICT(key) DO UPDATE`), keeping the world id, map and characters; `create_world` refusal now enforced by the `worlds.key UNIQUE` constraint.

17 [Collapse the Prompt Presets add forms](issues/17-collapse-preset-add-forms.md): resolved — each Add form is a closed `<details>` disclosure (no JS); tier 1 scenario 21.28; failed-add error display left to 25.

11 [Fix edit mode: size, focus, keys and locked controls](issues/11-fix-edit-mode.md): resolved — auto-growing textarea capped at 50vh, autofocus, Escape/Cmd+Enter, entry controls locked while editing and restored by every revert; tier 2 scenarios 30.6–30.9.

22 [Show Character names in the visual sidebar](issues/22-show-npc-names.md): resolved — each portrait shows its Character name in the reinstated `.image-label` (escaped, ellipsised); tier 1 scenarios 32.1/32.2 in a new `visual_sidebar.md` spec.

20 [Copy sweep](issues/20-copy-sweep.md): resolved — type names, plurals, jargon and unlabelled selects fixed per findings 5.1, 5.2, 5.4, 5.5; tier 1 scenario 25.8 plus extended 20.8.

28 [Decide whether duplicate connection and preset names are allowed](issues/28-decide-duplicate-names.md): resolved — rejected on add (trimmed, case-insensitive; connections global, presets per category), edits keep their own name, Duplicate picks a free "(Copy N)"; tier 1 scenarios 20.9–20.11, 21.29–21.34.

36 [Follow up on small issues found during review](issues/36-follow-up-small-review-issues.md): resolved — 21 of 28 items fixed or closed, 7 dropped with reasons; leftovers now in [round 2](issues/37-follow-up-small-review-issues-2.md), and the edit-another-entry freeze is [its own ticket](issues/38-edit-another-entry-freezes-poll.md).

34 [Fix the weak dashboard and settings tests](issues/34-fix-weak-dashboard-and-settings-tests.md): resolved — 16.5, 16.7/16.8, 16.10 (B2) and 20.2–20.4 now fail on the regression they name, each proven by a temporary break; `StubActionOutcome::Idle` deleted.

37 [Follow up on small issues found during review, round 2](issues/37-follow-up-small-review-issues-2.md): resolved — 17 of 20 items fixed, the 16.9 acknowledgement flake explained and fixed, 3 deferred or dropped with reasons; judgement leftovers listed in its answer.

09 [Decide whether failed LLM calls appear in LLM Messages](issues/09-decide-failed-llm-calls.md): resolved — every LLM attempt is recorded, success or failure; the failure marker is the existing nullable `error_message`; the error text is stored and shown verbatim; no attempt number, no degraded-outcome row, no duration or tokens; graduated [Record failed LLM attempts and show them in LLM Messages](issues/41-record-failed-llm-attempts.md).

04 [Review the text-check-enabled flow](issues/04-review-text-check-enabled.md): resolved — 8 findings, one P1: a confirm through the send preview leaves the command input permanently disabled while the status reads "Ready". Graduated [Rework the action area so a text check cannot strand it](issues/42-rework-the-action-area.md); the ✓-while-disabled half of C5 joins 15, the tofu ✍ joins 13.

06 [Review swipes, the options dock and the Thinking states](issues/06-review-swipes-options-thinking.md): resolved — 6 findings, one P1: switching a swipe restores the snapshot written mid-narration, so `/status/generating` answers `narrating` for good and the dashboard sticks on "Generating narration..." with no in-page way out. Graduated [Reset the generation status and the options dock when a swipe switch restores a snapshot](issues/43-reset-status-and-dock-on-swipe-switch.md); the Stop-label finding went into [Rework the action area so a text check cannot strand it](issues/42-rework-the-action-area.md), and the ▶ glyph instance joins 13. Thinking/generating states are now captured with a slow-stub connection (`tmp/ui-review/t06-llm-stub.py`).

07 [Review keyboard use and screen-reader output](issues/07-review-keyboard-screen-reader.md): resolved — 11 findings, two P1: the 2s story-log poll ejects keyboard focus to `<body>`, and LLM Messages rows are mouse-only (`div onclick`, zero focusable elements). Graduated [Make LLM Messages rows keyboard-operable](issues/44-keyboard-operable-llm-messages.md), [Restore keyboard focus after in-place htmx swaps](issues/45-restore-focus-after-inline-swaps.md), [Expose the dashboard to assistive technology](issues/46-expose-dashboard-to-assistive-technology.md) and [Announce dynamic state changes to assistive technology](issues/47-announce-state-changes-to-at.md) (blocked by 08); the story-log focus findings K1/K7 joined [Decide how the story-log poll keeps DOM state](issues/10-decide-story-log-poll-swap.md).

10 [Decide how the story-log poll keeps DOM state](issues/10-decide-story-log-poll-swap.md): resolved — morph the poll swap (vendored idiomorph) instead of a server no-swap or a selection pause. Keeps selection, hover, focus and scroll on idle polls; closes the switchSwipe hole for free; no client-side hash to maintain. K7 joins as `tabindex="0"` on `#story-log`. Tier 2 tests. Graduated [Morph the story-log poll swap](issues/48-morph-story-log-poll-swap.md) and [Delete the dead `htmx:refresh` calls and fix the docs](issues/49-delete-dead-htmx-refresh.md).

42 [Rework the action area so a text check cannot strand it](issues/42-rework-the-action-area.md): resolved — the action area is a static shell driven by one client state machine (`checking → generating → preview → idle`); confirm retargets `#status-display` instead of swapping a disabled input back in; `/check-text` renders a read-only result; the preview lives in its own `#action-preview`; focus follows the preview; the primary button is a disabled "Generating…" indicator, not a fake "Stop"; no cancel route built. 16.9–16.15, tier-3 proof 16.12.

41 [Record failed LLM attempts and show them in LLM Messages](issues/41-record-failed-llm-attempts.md): resolved — every LLM attempt persists one row (success or failure); the failure text is stored verbatim in `error_message` and rendered in the panel; the row names its agent and backend/model; new `docs/specs/llm_messages.md` 33.1/33.2 with tier-1 HTTP tests.

39 [Enforce preset name uniqueness in storage](issues/39-storage-level-preset-name-uniqueness.md): resolved — `Storage::save_preset` checks and writes under one backend lock (no TOCTOU); the service delegates to it; no `UNIQUE` constraint, so legacy duplicate-holding DBs still open; predicate moved to `domain/model/utils` to satisfy `arch-lint.toml`; driven-adapter InMemory/SQLite pair + legacy-duplicates test.

40 [Drive the retrigger control in a browser test](issues/40-drive-retrigger-in-a-browser-test.md): resolved — worth a tier-2 scenario; 30.10 drives the shipped `submitRetrigger` against the fixture's real control and the stub's counted `/retrigger` → 500 route, proving the URL and the shared recovery. (The earlier planted-control/fetch-intercept version was replaced during the review follow-up.)

44 [Make LLM Messages rows keyboard-operable](issues/44-keyboard-operable-llm-messages.md): resolved — each row header is a real `<button aria-expanded>` with `aria-controls` the body, so Enter/Space use native activation and the state is exposed to AT; the restore hook moved to `hx-on::after-settle` after a swap race left the class on a replaced node, and a stable header id keeps focus through the 4s poll; tier-2 35.1–35.3 in a new `browser_llm_messages.md`.

46 [Expose the dashboard to assistive technology](issues/46-expose-dashboard-to-assistive-technology.md): resolved — skip link plus `<main>` landmark; tablist/tab/tabpanel with `aria-selected`/`aria-controls`; the slash menu is a listbox/option pattern with the input a combobox tracking `aria-activedescendant` (the `.active` highlight is mirrored, not replaced); a real label names the command input; a forced-colors `:focus-visible` ring restores what `outline: none` removed; tier-2 16.16/16.17 and extended 31.1/31.3.

08 [Decide how the dashboard shows each kind of failure](issues/08-decide-failure-display.md): resolved — quiet by default. The hardcoded "Connected" is deleted; a banner under the header appears only while a role's newest LLM attempt has failed or the server is unreachable (the latter client-owned), with per-role detail behind a popover and on-demand health in Settings. Four always-on per-role chips were mocked, measured at ~310px, and rejected. The toast retires in sequence after the banner, the inline slots and the clamped status display exist; a failure never swaps into the region it describes; error text is a short message plus an anchored popover. Graduated [51](issues/51-add-failure-banner.md), [52](issues/52-failed-request-keeps-its-region.md), [53](issues/53-clamp-status-error-with-popover.md), [54](issues/54-retire-toast-and-route-callers.md).

Candidates raised while closing the AFK tickets, now tickets: [Stop editing a second entry from freezing the story-log poll](issues/38-edit-another-entry-freezes-poll.md) (grilling), [Enforce preset name uniqueness in storage](issues/39-storage-level-preset-name-uniqueness.md), [Drive the retrigger control in a browser test](issues/40-drive-retrigger-in-a-browser-test.md).

## Not yet specified

- **Other snapshot-restore paths.** [Reset the generation status and the options dock when a swipe switch restores a snapshot](issues/43-reset-status-and-dock-on-swipe-switch.md) fixes the swipe case, where the snapshot was written mid-generation. Retrigger and history revert restore snapshots by different paths and may show the same stuck status. Unverified; [Drive the retrigger control in a browser test](issues/40-drive-retrigger-in-a-browser-test.md) drove the failure path only and did not surface it.
- **Implementation of the other decision tickets:**
  - icon buttons (13)
  - palette (14)
  - Settings panel (15)
  - save model (16)
  - Options presets (18)
  - layout convention (19)
  - game names (21)
- **Browser suite cost.** Each stub-tier test launches its own Chromium, and Theme 1/Theme 2 fixes will add several. Check the cost at the final re-review. Decide then whether it needs action.
- **Spec and test migration after the tier-1 decision.** 65 of 146 scenarios name internal state, and about 40 tier-1 test lines read raw `GameState` fields ([test audit](assets/test-audit/)). How much of this moves, and when, depends on [Decide what a tier-1 test may observe](issues/31-decide-tier-1-observations.md). It may graduate into one ticket per spec file, into a migrate-on-touch rule, or into nothing.
- **A validator check for spec prose.** `scripts/validate_feature_spec.py` never reads Givens or Thens. One cheap check might flag dotted identifiers in backticks, such as `narrative.last_trigger`. Its false-hit rate is unknown. It takes shape with [Rewrite the test strategy around one checkable tier-1 rule](issues/32-rewrite-test-strategy.md).

## Out of scope

- **Changing the Quantifier's fallback behaviour.** The engine falling back to fallback NPC IDs is now reported by the failure banner ([51](issues/51-add-failure-banner.md)); whether the fallback itself should behave differently is a backend design question and this map covers the dashboard frontend.
