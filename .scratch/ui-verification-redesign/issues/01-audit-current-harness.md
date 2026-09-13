# Audit the current browser-test design: what does each test actually verify, and through which layer?

Type: research
Status:
Blocked by:

## Question

Read [browser-tier-inventory.md](../assets/browser-tier-inventory.md) and
[flake-investigation-2026-09-13.md](../assets/flake-investigation-2026-09-13.md)
first. Then audit the 26 tests in `tests/browser/` against
`tests/STRATEGY.md`'s placement rule (browser tier only for assertions only a
browser can observe: DOM, CSS, JS, rendering).

Classify each test into one of:

1. **Browser-only by nature** — asserts rendering/layout/client JS behaviour
   (e.g. slash menu keyboard nav, CSS invariants). Correct tier.
2. **Browser-observable behaviour that is really the app's wiring** — e.g.
   an `hx-trigger="change"` select producing a POST and a status fragment.
   The *application* contract is the HTTP exchange; the browser is one client
   of it.
3. **Already covered at the HTTP tier** — an equivalent request-level contract
   test exists (e.g. http-tier posture merge in tests/http/worlds.rs vs
   browser posture autosave in tests/browser/worlds.rs).

Also audit the harness itself: `with_test_page`, the shared story-log gate at
browser.rs:87, `wait.rs` primitives, `goto_with_connection_check`. Ask of each
piece: what flake does it prevent, what flake does it cause, and would it exist
in a simpler design? Note that the shared gate is what turned one CDN hiccup
into 6 different "flaky tests".

Two further things to inventory while auditing:

1. **Prior art.** Read `.scratch/test-strategy-execution/issues/07-browser-tier-design.md`
   (Answer section), `08-browser-tier-execution.md`, and
   `14-browser-missing-tests.md`. Tier placement was already grilled out
   (13 keep / 17 move-down; behaviour.rs specced + invariants.rs exempt). Do not
   re-litigate placement from scratch — your audit re-reads the current tier
   through a *robustness* lens and notes where the tier has drifted since the
   per-surface split (9 files, 26 tests).
2. **The app's existing readiness surface.** Inventory every readiness /
   observability hook the dashboard already exposes, since ticket 04 (Decide
   the UI verification design) will want them as raw material: `/status/generating`,
   `/debug/state`, `/debug/is_generating`, `HX-Trigger` response headers,
   htmx lifecycle events already plumbed, WebSocket state channels (see
   `docs/diataxis/explanation/two-state-channels.md`). Say which are observed
   by tests today and which go unused.

Deliverable: a linked markdown asset — a per-test classification table, the
readiness-hook inventory, + a short verdict paragraph on whether the harness
complexity is load-bearing or accidental. Make no code changes.
