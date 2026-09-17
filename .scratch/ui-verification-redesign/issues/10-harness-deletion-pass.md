# Harness deletion pass: the ratified removal list

Type: task
Status:
Blocked by: 06, 07, 08, 09

## Question

Execute the deletion list ratified in ticket 04 (its Answer, Q5). Each item is
gone, replaced, or explicitly declined with evidence — per the user's standing
preference, harness surface that doesn't earn its keep does not survive the
redesign.

1. `wait_for_condition_sync` and `extract_port_from_url` (`tests/test_utils/wait.rs`)
   — the audit says zero uses. Verify against the *post-rollout* tree (tickets
   07–09 may have added uses of one) before deleting.
2. `GET /status/ready` endpoint (registered, static, never requested by the UI;
   the audit §3 names it) — delete the route, the handler, and any test that
   exists only to cover it. Confirm nothing in `assets/index.html` references it
   first.
3. The duplicated server-ready probe in `goto_with_connection_check`
   (`tests/test_utils/browser.rs:14`) — fold into `TestServer`'s own probe; one
   readiness check per boot, not two.
4. `networkidle` — ban as a wait strategy suite-wide: grep the test tree today
   to confirm no current use (the audit §3 shows five pollers make a quiet
   window unsatisfiable; the research asset bans it), write the convention into
   `tests/STRATEGY.md` (the same edit as ticket 11's rule write-up; go
   together), and if any current `WaitUntil::NetworkIdle` use exists, replace
   with the tier-appropriate wait.
5. The shared `#story-log .log-entry` gate — ticket 09 replaces it in
   `with_test_page`; this ticket removes the now-unused twin at
   `invariants.rs:35` after 07 moves invariants to tier 2, and deletes
   `wait.rs` wait-primitives that no surviving test uses post-rollout (audit §2
   names candidates — verify, don't trust the audit's line numbers).
6. Sort order with other tickets: do this pass **last** among 07–09 (except 11
   which is docs (+validator) — either order). Trading twice on the same
   `wait.rs` helpers is how deletions cause new flakes.

Sequencing rule from the map: the `Blocked by` line (06, 07, 08, 09) is the
claim gate — this ticket is claimed only after 07–09 are resolved, because
trading twice on the same `wait.rs` helpers is how deletions cause new flakes.

Record under `## Answer`: per item — deleted / folded into what / declined
(with use evidence), and the `wait.rs` / `browser.rs` line count before and
after.
