# Harness deletion pass: the ratified removal list

Type: task
Status: resolved
Blocked by: 06, 07, 08, 08b, 09

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

## Answer

Resolved 2026-09-20. Every item is deleted, folded, or declined with evidence.
Full gate green: 1604 integration / 137 guardrails / 20 browser / 1
architecture, 2 skipped; validator 141 declared, 141 covered, 0 gap(s), 0
orphan(s), 0 untagged, 0 surface mismatch(es), quarantine 85/85.

**Line counts.** `tests/test_utils/wait.rs` 260 → **236**. 
`tests/test_utils/browser.rs` 394 → **392** (the duplicated probe's 4 lines left,
3 lines of rationale arrived, and `cargo fmt` re-collapsed the shortened import
to one line).

### 1. Dead wait primitives — **deleted**

`wait_for_condition_sync` (`wait.rs:248`) and the private
`extract_port_from_url` (`wait.rs:114`) had zero call sites in the post-rollout
tree, verified by `rg` across `tests/`. The audit's line numbers were still
accurate for these two. Nothing else in `wait.rs` was deletable: every surviving
primitive has a live caller — `wait_for_llm_idle` and `wait_for_status_ready_or_error`
(llm flows), `wait_until_hidden` (tier2 ×3), `wait_for_element_persist`
(`tier2.rs:418`), `wait_for_condition_async` (settle gate ×2, `invariants.rs`,
`http/test_helpers.rs`), `wait_for_story_log`, `wait_for_status_ready`,
`wait_for_status_generating`, `wait_for_element_children`, and the private
`element_count`.

The `wait.rs` module summary line already named only surviving helpers, so it
needed no edit (and the auto-generated `tests/AGENTS.md` index entry is
consequently unchanged).

### 2. `GET /status/ready` — **deleted**

Registered static, never requested by the UI. Deleted: the route
(`router.rs`), `status_ready_handler` (`layout/handlers/endpoints.rs`), its
re-export (`layout/handlers/mod.rs`), the `src` unit test
(`endpoints_tests.rs`, import list trimmed), and the quarantine HTTP test
(`tests/http/requires_migration/fragment.rs`). The existing
`fetch_body(.., "/status/ready")` test existed only to cover the endpoint.

`REQUIRES_MIGRATION_TEST_COUNT` lowered **86 → 85** (the count may only go down;
deletions lower it deliberately). The `http_routes.md` regeneration is
**mandatory**, not hygiene: the gate's `http-routes-check` step runs
`extract_http_routes.py --check` and fails on a stale doc. Regenerated via
`python scripts/extract_http_routes.py` — the diff is exactly one removed row
(56 routes).

Blast radius confirmed exhaustive across `src/`, `tests/`, `assets/`, `docs/`,
and `scripts/`: no reference survives. The two candidate traps were checked and
both clear — the boot probe (`server.rs::probe_http`) hits `GET /`, not this
endpoint, and the browser helper `wait_for_status_ready` polls the
`#status-display` DOM, so neither depends on the route.

**One gate failure surfaced, and the test behind it was deleted.**
`scripts/tests/test_extract_http_routes.py::test_rendered_doc_table_rows_sum_to_route_count`
hardcoded 57 route rows; deleting a route legitimately lowered it. The count was
first updated to 56, then the whole test was removed on review as a maintenance
trap. Its useful assertion — parsed routes all appear as table rows — is
already covered by `test_table_row_count_matches_routes`, which derives the
expected count from the fixture instead of pinning it. The sibling
`test_real_router_route_count_matches_grep` stays and is safe: it cross-checks
parsed routes against `grep -c '\.route('` on the same file, so it is relative,
not pinned. Python tests 125 → 124; suite green.

### 3. Duplicated server-ready probe — **folded into `TestServer`**

`goto_with_connection_check` called `wait_for_server(port, 100)`;
`TestServer::start` already probes with `wait_for_server(port, 300)` and panics
on failure before returning. Both callers of `goto_with_connection_check`
(`with_test_page`, `llm/flow_llm_tests.rs`) construct a `TestServer` first, so
the second probe was pure duplication. Deleted from `browser.rs:27`; the
`wait_for_server` import went with it. The function itself stays — `TestServer`
still uses it.

### 4. `networkidle` — **declined here, evidence passed to ticket 11**

Zero current uses: `rg -i networkidle` over the test tree returns nothing, so
there was nothing to replace. The convention write belongs to ticket 11, whose
question explicitly owns the fold-in ("Also fold in ticket 10's networkidle
ban (one convention, one place)"), matching this ticket's own "the same edit as
ticket 11's rule write-up; go together". Ticket 11 writes the ban with this
grep result as its evidence; ticket 10 touches no `STRATEGY.md`.

### 5. Story-log gate twin and `wait.rs` re-verify — **declined (twin no longer
exists)**

The "now-unused twin at `invariants.rs:35`" does not exist. Ticket 07
restructured `invariants.rs` into the `run_subtest` shape, and every check
now reaches its page through `SharedBrowser::open_page`, which waits for the
canned entry once. There is no second unconditional gate to remove. Ticket 09
had already dropped the `with_test_page` copy, so `wait_for_story_log` survives
with exactly one caller — `SharedBrowser::open_page` (tier-2 stub readiness),
which is load-bearing rather than residual.

The `wait.rs` re-verification is recorded under item 1: the candidate list
collapsed to exactly this ticket's item-1 pair, and nothing more.

### 6. Sequenced last among 07–09 — **satisfied**

06, 07, 08, 08b, and 09 were all `resolved` at claim time, so the deletions
traded on `wait.rs` only once. Ticket 11 (docs) ran no concurrent edits to
files this pass touched. Verification: full gate green after the pass, with the
browser binary at 20/20.
