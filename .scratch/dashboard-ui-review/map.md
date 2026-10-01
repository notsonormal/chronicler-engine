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

## Not yet specified

- **Theme 1 implementation.** The work for header health, status-display errors, inline action errors, the toast's future, and the silent failures (finding 1.7). It takes shape when [Decide how the dashboard shows each kind of failure](issues/08-decide-failure-display.md) and [Decide whether failed LLM calls appear in LLM Messages](issues/09-decide-failed-llm-calls.md) resolve. A prototype may come first.
- **Fixes found by the review tickets.** Review tickets 04–07 will produce findings. Each becomes a ticket or joins an existing one.
- **Implementation of the other decision tickets:**
  - story-log poll (10)
  - icon buttons (13)
  - palette (14)
  - Settings panel (15)
  - save model (16)
  - Options presets (18)
  - layout convention (19)
  - game names (21)
- **Browser suite cost.** Each stub-tier test launches its own Chromium, and Theme 1/Theme 2 fixes will add several. Check the cost at the final re-review. Decide then whether it needs action.
- **Quantifier fallback.** Finding 1.1 is partly backend: the engine uses fallback NPC IDs silently. Showing it is in scope. Whether the fallback itself should change may surface during ticket 08.
- **Spec and test migration after the tier-1 decision.** 65 of 146 scenarios name internal state, and about 40 tier-1 test lines read raw `GameState` fields ([test audit](assets/test-audit/)). How much of this moves, and when, depends on [Decide what a tier-1 test may observe](issues/31-decide-tier-1-observations.md). It may graduate into one ticket per spec file, into a migrate-on-touch rule, or into nothing.
- **A validator check for spec prose.** `scripts/validate_feature_spec.py` never reads Givens or Thens. One cheap check might flag dotted identifiers in backticks, such as `narrative.last_trigger`. Its false-hit rate is unknown. It takes shape with [Rewrite the test strategy around one checkable tier-1 rule](issues/32-rewrite-test-strategy.md).

## Out of scope

<!-- none yet -->
