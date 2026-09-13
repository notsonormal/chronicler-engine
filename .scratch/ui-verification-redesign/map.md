# Map: UI verification redesign — from per-flake patching to robust-by-design

Labels: wayfinder:map

## Destination

The redesign is **shipped**: the browser tier verifies the UI the way the design
decision settles it, the flake mechanisms are structurally removed (not merely
diagnosed), the harness surface that doesn't earn its keep is deleted, and the
full gate runs green with the relevant races impossible rather than retried
away. The application and its browser tests work today; the way is clear when
they work *robustly by construction*, the code shows it — and **5 consecutive
green full-gate runs with zero legacy flake signatures** (story-log CDN gate
timeout, posture-change no-fire) confirm it on the machine.

## Notes

- Domain: the dashboard is HTMX fragments over axum; htmx 1.9.10 is now vendored
  at `assets/htmx.min.js` (was fetched from unpkg per page load — a flake root
  cause, fixed 2026-09-13).
- Uncommitted tree state to know about: 10 modified files + `assets/htmx.min.js`
  (build-speedup + vendoring work) are sitting uncommitted as of chart date.
- Skills to consult: `research` for tickets 01–03; `/grilling` +
  `/domain-modeling` for ticket 04 (Decide the UI verification design).
- Related prior work: the [test-strategy-execution map](../test-strategy-execution/map.md)
  already designed and executed a browser-tier consolidation — its
  [ticket 07](../test-strategy-execution/issues/07-browser-tier-design.md)
  (target state: `behaviour.rs` specced + `invariants.rs` code-is-definition)
  and [ticket 08](../test-strategy-execution/issues/08-browser-tier-execution.md)
  settled tier *placement* (13 keep / 17 move-down), and
  [ticket 14](../test-strategy-execution/issues/14-browser-missing-tests.md)
  grilled missing behaviours. This map does NOT re-litigate placement; it
  answers the robustness question that effort never asked: how a test knows
  the UI is ready. The tier has since re-split per-surface (9 files, 26
  tests), so the prior answers need re-reading in that light.
- Assets from the flake investigation (read these first):
  - [Flake investigation evidence](./assets/flake-investigation-2026-09-13.md)
  - [Browser tier inventory](./assets/browser-tier-inventory.md)
- Execution override: this effort **carries execution into the map** (overrides
  plan-don't-do). Tickets 01–04 still resolve in order — research the evidence,
  then grill the design — and the design's Answer graduates implementation
  tickets, which subsequent sessions resolve one at a time until the
  destination is shipped.
- Standing preferences (from the user):
  - First-principles robustness over per-flake patching. Fixing one flaky test
    while the design invites the next one is failure.
  - Either the application design or the browser-test design may change; nothing
    is sacred. The project is deliberately not complicated — prefer
    simplification over added machinery.
  - Be suspicious of accumulated diagnostics and harness structure; judge
    whether each piece earns its keep rather than assuming it does.

## Decisions so far

_None yet._

## Not yet specified

- Implementation tickets applying the chosen design — an initial worlds-panel
  implementation as the worked example, then one per remaining surface —
  graduate from "Decide the UI verification design"'s Answer.
- Whether the nextest browser serialization override
  (`threads-required = num-test-threads`, added for a 2-core box) and the
  `retries = 1` policy survive the redesign — graduates after the design
  decision.
- Spec/test-tag bookkeeping for any tests that move between tiers
  (`validate_feature_spec.py` gates on this).
- The shared `#story-log .log-entry` precondition in `with_test_page`
  (tests/test_utils/browser.rs:87) — whether it becomes an explicit readiness
  contract or disappears. Depends on the design decision.

## Out of scope

- The CDN-vendoring fix itself — already done (assets/htmx.min.js +
  `assets/index.html`), recorded in the investigation asset.
- A top-to-bottom re-litigation of tier *placement* — settled by
  test-strategy-execution tickets 07/08 (see Notes). Exception: the tier has
  drifted since that ruling (26 tests now, 13 kept then), so ticket 01 flags
  drift and ticket 04 asks the user explicitly what to do about it; the
  default is that the 07/08 ruling stands. Note the later growth was partly
  deliberate — test-strategy-execution tickets 14/15 grilled + implemented
  new browser tests (responsive layout, error toast) — and the per-surface
  re-split (ticket 20) reorganized behaviour.rs into files; ticket 01 notes
  provenance per file rather than treating all growth as drift.
- Build-pipeline work (gate steps, flaky epilogue, coverage) — separate effort,
  mostly completed 2026-09-13.
- Environment tuning (the move to an 8-core WSL guest) — done, recorded in the
  investigation asset.
- Narrative/LLM pipeline behaviour — verification of *what* the LLM generates,
  vs this map's concern: *that* the UI wires and renders it correctly.
