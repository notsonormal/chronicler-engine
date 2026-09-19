# Tier-3 conversion: settle-gate wrappers and the keeper set

Type: task
Status:
Blocked by: 06

## Question

Put every remaining full-stack browser test behind the settle-gate primitive
built in ticket 06, and finalise the keeper set per the ratified design
(ticket 04 Answer: readiness option A — enforcement inside the harness helpers,
`hx-preserve` as fallback for a surface that resists wrapping).

In order:

1. **Wrapper converts the suite, not vice versa.** Route every swap→interact
   call in `tests/test_utils/browser.rs` through the settle-gate:
   `select_option` and the `click`-then-swapped-interact paths wait for the
   `htmx:afterSettle` counter to advance past the baseline armed before the
   action. `wait_until_visible` stops being a readiness primitive — it may
   remain as a rendering assertion in tier 2, but it no longer licenses an
   interaction in tier 3. The ticket-06 slice recorded `detail.elt`
   target-scoping as **required** (poller noise satisfied the plain counter —
   measured, not hypothetical): wire that in now.

   **Registering-swap rule (ratified with ticket 06's verdict, decision 2
   option 1):** target-scoping alone failed 6/50 in the slice. Every helper
   that triggers a swap must await *that swap's* settle before returning —
   enforcement is structural, in the helper, not a documented convention.
   `open_world_edit`'s await of the `.worlds-panel` settle is the reference
   shape (ticket-03's 20 ms `defaultSettleDelay` mechanism was refuted; see
   the correction on that ticket's Answer).
2. **Keeper set converts.** Move the surviving tier-3 tests onto the wrapped
   helpers, unchanged assertions: tests 14/15 (kept per rule-07's client-wiring
   rationale), `test_options_dock_survives_reload` (17), test 8 from
   `slash_menu.rs` *if ticket 07 left it here*, and the wiring smoke guards
   (see 3).

   **Disposition required for tests 21 and 24.** Ticket 08b added stronger HTTP
   coverage for both (games.md 20.8 for the posture-fragment render,
   worlds.md 25.6 for the world edit-form render), so their assertions are now
   curl-observable — but their *click-hop wiring* (worlds tab → Edit htmx swap;
   games tab switch) is not, and this ticket owns the swap→interact helpers
   that would cover it. Decide each explicitly and record the call: either fold
   its wiring into the item-3 guard for that surface, or delete it with the
   guard as its replacement. Do not leave either undeclared.
3. **Wiring smoke guards** — convert from ticket 08's demotions (one per
   surface, assert the server actually changed, not the fragment): posture
   change in `worlds.rs` (the ticket-03 flake home — ticket 06's slice already
   ran its ~50-run stress loop; **this ticket owns the guard's final form**,
   and ticket 12 re-runs the loop against the final suite), posture change in
   `games.rs`, one preset-chain guard, one options-Use guard (assert the guard
   submits the option text *read from the rendered button*, not a hardcoded
   seed string — otherwise the dock-render → submit linkage stays uncovered).
   All behind the wrappers. These replace — not supplement — the browser tests
   ticket 08 deleted.
4. **Story-log gate becomes explicit.** The shared `#story-log .log-entry`
   precondition in `with_test_page` (`browser.rs:87`) is replaced by an
   explicit `wait_for_story_log()` called only by the tier-3 tests that need it
   — per ticket 04's ratified deletion item; a failure is then attributable to
   the test that needed the log, not to N unrelated tests.
5. Fallback path: if one surface genuinely resists wrapping (evidence, not
   hunch — record what failed), `hx-preserve` on that surface's template is
   the ratified structural fix. Name the surface and the evidence in
   `## Answer`.

Do not widen tier 3 while converting. If a keeper's assertions prove
curl-observable mid-work, move it in this ticket's record rather than leaving
it.

Record under `## Answer`: converted-helper surface (which helpers now gate,
and how test authors are prevented from opting out), the final keeper list with
paths, any per-surface fallback applied and why, and the tier-3 binary's wall
time before/after.
