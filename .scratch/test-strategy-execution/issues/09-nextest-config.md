# Review nextest configuration (retries, parallelism, profiles)

Type: task (AFK)
Status: resolved

## Question

Migrated from the [planning map's ticket 11](../../test-strategy/issues/11-review-nextest-config.md).
The question is unchanged; this ticket carries the execution context
(post-consolidation measurements). Blocked by ticket 08 (browser tier
execution, the last known consolidation ticket) because the suite must be
fully consolidated before tuning. Also depends on any implementation
tickets that graduate from ticket 06 (lifecycle/sequencing/arrival
research) — add them as blockers when they're created.

- **`--retries 2` in build.py** masks flaky tests silently. With the
  polling loop gone (the main suspected flake source), decide: keep
  retries as a safety net, drop to `--retries 1`, or drop to 0 and let
  flakes fail loudly so they get fixed?
- **`-j 4` is conservative.** The polling tests were sleep-bound, not
  CPU-bound; after the consolidation the suite is mostly fast tests. Try
  higher parallelism (and check whether SQLite temp-file or port-lock
  contention appears — build.py mentions a `chronicler_test_ports` lock
  dir, so port allocation is already managed; verify it holds at higher
  -j).
- **tests/nextest.toml** has a 300s default timeout and a 600s
  `flow_mock_tests` profile. Once tests are fast, lower the default
  timeout so a hung test fails in reasonable time instead of burning 5
  minutes.
- Confirm the slow suites (http/browser) partition sensibly across
  nextest profiles so `build.py` fast runs stay fast.

## Decisions

- Retries: dropped from `--retries 2` to `1` in the default nextest profile.
  `--retries 0` surfaced a flaky browser test (`behaviour::test_delete_removes_message`)
  within the first run; `1` retry masked no failures across six subsequent runs.
  This keeps a small safety net while exposing repeated flakiness.
- Parallelism: kept `-j 4` / `test-threads = 4`. Benchmarked `-j 2` (~42s), `-j 4`
  (~30s), and `-j 6` (~39s) on a 2-core host; `-j 4` is the sweet spot for this
  mix of fast unit and I/O-bound browser/HTTP tests. Port locks held with no
  SQLite contention.
- Timeout: lowered default hard timeout from the old 300s warning-only setting to
  `slow-timeout = { period = "60s", terminate-after = 1, on-timeout = "fail" }`.
  A hung test now fails in ~70s instead of burning 5 minutes. LLM tests keep a
  dedicated `[profile.llm]` with a 300s timeout, used automatically when build.py
  includes ignored tests.
- Profiles: removed the unused `tests/nextest.toml` (it was not loaded by nextest;
  the active config is `.config/nextest.toml`). The stale `[profile.flow_mock_tests]`
  profile is gone.
- Fast-run partition: all default-profile tests stay fast; http/browser suites
  are naturally interleaved. No separate profile needed for `build.py` fast runs.

## Files changed

- `.config/nextest.toml` — new canonical config:
  `test-threads = 4`, `retries = 1`, default 60s hard timeout, `[profile.llm]`
  with 300s timeout for LLM runs.
- `tests/nextest.toml` — deleted stale duplicate.
- `build.py` — removed hardcoded `--retries 2 -j 4` and
  redundant `--features testing` from `get_test_cmd()` and `get_coverage_cmd()`;
  `include_llm` runs now select `--profile llm`.
- `AGENTS.md` — updated standard-build wall-clock expectation
  from "2-3 minutes" to "about 1 minute" in both the DEVELOPMENT LOOP prose and
  the Final Validation command comment.
- `.agents/skills/test-police/TEST_INVENTORY.md` — moved `nextest.toml` entry
  from the stale `tests/nextest.toml` location to `.config/nextest.toml`.

## Verification

- `python build.py --no-fmt`: **30.06s total**, all steps OK.
- `cargo nextest run --features testing --no-fail-fast`: **six consecutive green
  runs** (29-35s, 1350 passed, 2 skipped) after the config change.
- `cargo llvm-cov nextest --no-report --no-fail-fast --features testing`: green
  (29.6s, 1350 passed, 2 skipped).
- `cargo nextest show-config version` parses cleanly; no config errors.

Acceptance: build.py's documented wall-clock expectation updated; suite green
at the new settings; no increase in flaky failures over a few runs.
