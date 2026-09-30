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
- Other agents may work in the repo at the same time. Do not touch unrelated changes.

## Decisions so far

<!-- the index — one line per closed ticket -->

- [05](issues/05-review-create-save-delete.md) — review of create/save/delete flows; report at `tmp/ui-review/t05-report.md`; findings F1–F8 graduate to 25–29 and extend 08/16.

01 [Stop the story-log fragment nesting a second #story-log](issues/01-fix-nested-story-log.md): resolved — the narrative-log fragment no longer ships the `#story-log` wrapper (shell keeps sole ownership); tier 1 `test_story_log_fragment_declares_no_log_container` (scenario 8.5) + aligned stub fixture; commit 60d1ef65.

02 [Rebind action-area handles after swaps](issues/02-rebind-action-area-handles.md): resolved — use-time `#submit-btn` lookup + body-level MutationObserver delegation; tier 2 stub tests 16.9/16.10 (`tests/browser/stub/dashboard.rs`); commit 262a4857.

29 [Stop world creation from silently overwriting an existing key](issues/29-world-key-silent-overwrite.md): resolved — user create refuses an existing key (`Storage::create_world` → `WorldAlreadyExists` → 400 toast); bootstrap `seed_world` still replaces, InMemory now matches; tier 1 scenario 25.7 + storage pair tests; graduated [35](issues/35-sqlite-reseed-keeps-world-id.md).

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
