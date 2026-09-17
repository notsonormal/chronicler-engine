# Tier-2 rollout: move the remaining class-1 tests onto the stub-server tier

Type: task
Status:
Blocked by: 06

## Question

Roll out the tier-2 fixture validated by ticket 06 to the rest of the class-1
set, per the ratified placement rule ("if the server behind this were fake,
does the behaviour change?" — no → tier 2). The audit
(`assets/harness-audit.md` §1) is the source of per-test facts.

Move, keeping every assertion identical:

- `slash_menu.rs` tests 2–7 (opens / filters / arrows / enter / escape /
  click) — pure client JS on the shell.
- `options.rs` test 9 (`options_dock_edit_fills_without_submitting`).
- `story_log.rs` tests 10–12 (edit activates, cancel restores, polling pauses
  during edit) — stories as canned entries; the poll hits the stub's canned
  response.
- `dashboard.rs` test 13 (error toast — synthetic `htmx:beforeSwap`).
- `invariants.rs` test 1 (9 sub-checks) — move onto the tier-2 fixture, one
  shared browser with fresh pages via `run_subtest`.

Judge individually — these two are not automatic:

- `slash_menu.rs` test 8 (`reopens_after_action_area_rerender`): port via a
  synthetic `#action-area` innerHTML transplant + re-arm check, **or** leave in
  tier 3 if the transplant reads as faking the thing under test. Whichever,
  say why in `## Answer`. Ticket 09 expects the answer.
- Any test where the canned fragment visibly diverges from the real template:
  stop, note the drift, and ask whether it belongs in tier 3 instead. This is
  the accepted drift tax (ticket 04 Answer) — pay it with a fixture update,
  not by stretching the assertion.

The tier-2 fixture itself (stub server + shared browser, fresh page per test)
is extended, not rebuilt, from ticket 06. Keep it serverless from the app's
perspective: no `TestServer`, no engine boot.

Ticket 06's spike ported 2 tests onto the fixture
(`tier2::test_slash_menu_opens_on_slash_tier2`,
`tier2::test_error_toast_on_action_failure_tier2`) as mirrors of their tier-3
twins. Reuse them; do not move them twice. The duplicates are deleted here,
not in ticket 06 — state the deletion in `## Answer`.

**Exit check (ratified with ticket 06's verdict):** both spike tests are
server-independent, so the slice never measured the drift tax for a test that
*reads* served content. The `story_log.rs` ports (tests 10–12: canned entries
in the story log, polling against the stub's canned response) are the first
fragment-reading ports. Characterise the drift there — does the test still
exercise behaviour, or has the canned fragment quietly become the assertion? —
before porting anything else. Record the finding in `## Answer`; if any port
reads as faking the thing under test, stop and say so rather than multiplying
it.

Record under `## Answer`: the per-test placement table (moved → which fixture
mode / or left in tier 3 + why), the wall time of the tier-2 binary, and the
drift items found.
