# Map: dashboard UI review, phase 2

Labels: wayfinder:map

## Destination

Every finding in the [2026-10-09 re-review](../dashboard-ui-review/re-review-2026-10-09.md) is fixed, or ruled out of scope with a reason. [Re-review the dashboard after phase 2](issues/07-re-review-after-phase-2.md) finds no new P1/P2, and `python build.py` passes.

This map continues the [dashboard UI review](../dashboard-ui-review/map.md). That map closed with [Final re-review of the dashboard](../dashboard-ui-review/issues/24-final-re-review.md), which found one new P1 and eight new P2s.

## Notes

- **This map carries execution,** like phase 1. It overrides wayfinder's "plan, don't do": `task` tickets make the change, not only decide it. Decision tickets (`grilling`, `prototype`) graduate their implementation work into new tickets.
- **Domain:** the dashboard frontend: `assets/` (`index.html`, `styles.css`, the `dashboard-*.js` modules) and the Askama templates and handlers under `src/adapters/driving/http/`. Backend code changes only where a finding needs it. [Decide when a reasoning-only reply counts as an answer](issues/01-decide-reasoning-only-replies.md) is the expected exception: it touches the LLM transport.
- **Source of findings:** the [2026-10-09 re-review](../dashboard-ui-review/re-review-2026-10-09.md). Tickets cite findings by R-number (for example "R6") and screenshots by number (`tmp/t24/NN-*.png`, local only).
- **Tests:** the rules of phase 1 hold. Place every test by the placement rule in `tests/STRATEGY.md` and name the tier in the ticket answer. Specs describe the system, not changes. Pure CSS and copy fixes need no test, but update the existing assertions that a fix breaks. A tier-1 test asserts domain outcomes only (the "Domain outcome" section of `tests/STRATEGY.md`).
- **End of each execution ticket:** `python build.py` is green, then the user reviews the diff. After approval, commit through `/commit-and-push`. One ticket = one session = one commit.
- **Skills:**
  - grilling and prototype tickets: `/grilling`, `/domain-modeling`, `/prototype`
  - review tickets and visual checks: `/chronicler-ui-investigator`
- **Reference docs:** `docs/diataxis/reference/frontend/ui_design.md` (design tokens), `docs/diataxis/reference/frontend/dashboard.md` (including "Failure and health states"), `docs/diataxis/explanation/dashboard_design.md`.
- **Real LLM calls are fine.** The user prefers the real connections to a stub for review and manual checks; a few calls cost less than a cent.
- **User data:** review tickets run on a probe server with its own database (`python build.py run -- --world redmist_estate --port <port>` writes `target/debug/chronicler_<port>.db`). Restore any changed setting before the session ends.
- **Final re-review:** every ticket added to this map must also be added to the `Blocked by:` line of [Re-review the dashboard after phase 2](issues/07-re-review-after-phase-2.md).
- **Small follow-ups:** issues from code reviews or implementers that don't block a merge go under `## Items` in [Follow up on small issues found in the final re-review](issues/06-follow-up-re-review-p3s.md), with their source ticket, while it is open.
- Other agents may work in the repo at the same time. Do not touch unrelated changes.

## Decisions so far

<!-- the index — one line per closed ticket -->

## Not yet specified

- **Other snapshot-restore paths.** Carried from phase 1. [Reset the generation status and the options dock when a swipe switch restores a snapshot](../dashboard-ui-review/issues/_resolved/43-reset-status-and-dock-on-swipe-switch.md) fixed the swipe case only. Retrigger and history revert restore snapshots by other paths and may show the same stuck status. Unverified; the 2026-10-09 re-review could not reach Retrigger because no trigger fired. [Re-review the dashboard after phase 2](issues/07-re-review-after-phase-2.md) forces a trigger to test it.
- **Structured editing of Map and Scenarios JSON.** Carried from phase 1 (finding 4.8, pushed here by [Decide the panel layout convention and supported viewports](../dashboard-ui-review/issues/_resolved/19-decide-layout-convention.md)). Raw JSON sits in plain textareas. R15 (the placeholder JSON looks like a value) touches the same form. A separate question; it can wait.

## Out of scope

Carried from [Final re-review of the dashboard](../dashboard-ui-review/issues/24-final-re-review.md), which names an owner for each:

- **Two inert route-boundary guardrails.** `check_handler_return_type` and `check_server_layer_boundaries` can never fire, because the gate strips the `src/` prefix. Owner: the guardrail suite.
- **Panel padding at 48px instead of 24px.** Every panel nests a same-class fragment root inside a same-class shell. Pre-existing on `main`. R16 (the inset jump after Duplicate and World Edit) is its visible symptom.
- **Panel wipe on a failed preset add or activate.** Owner: `.scratch/architecture-deepening/issues/15-single-fragment-error-policy.md`.
- **`isPollRequest` reports a server-answered 500 as "unreachable".** Scenario 16.18 pins the banner on purpose; only the heuristic and the wording are open.
- **Changing the Quantifier's fallback behaviour.** A backend design question; the banner already reports the fallback.
