# Coverage run reliability: browser-test flakes and gate ergonomics

> **Status:** Proposed. Surfaced by the test-police review of 2026-08-30 (branch `guided-generations`).
> **Scope:** 3 SP — root-cause investigation, one targeted fix, two one-line skill-doc updates.

## Summary

`python build.py --coverage` — the documented test-police coverage command — fails flakily on a cold
isolated run: 4 browser tests fail and 3 more are flaky, all with the same server-start-timeout
signature. The same tree passes the full non-instrumented suite (1463/1463). The coverage gate itself
passes (88.7% ≥ 80%); the problem is that *running* coverage is unreliable and slow enough that agents
mis-handle it (foreground timeout, empty logs). This plan covers the investigation, the fix, and the
small ergonomics gaps observed while running it.

## Evidence

### The failing run

From `logs/build_20260830_231929.log` (coverage run, isolated target dir):

```
Summary [ 192.166s] 1463 tests run: 1459 passed (3 flaky), 4 failed, 2 skipped
```

Failed (both retries exhausted):

- `behaviour::test_delete_removes_message`
- `behaviour::test_edit_cancel_restores_original`
- `behaviour::test_edit_mode_activates_on_click`
- `behaviour::test_error_toast_on_action_failure`

Flaky (failed try 1, passed try 2 — try-1 output is not captured in the log):

- `behaviour::test_form_stays_static_after_submission`
- `behaviour::test_polling_pauses_during_edit`
- `behaviour::test_slash_guide_does_not_persist_input_entry`

All four hard failures share one signature:

```
🛑 Server failed to start on port 3043 within 30s. Draining child output for diagnostics:
--- child stderr empty (binary may not have written anything yet) ---

thread 'behaviour::test_edit_mode_activates_on_click' panicked at tests/browser/../test_utils/server.rs:453:13:
Server failed to start on port 3043
```

(Ports differ per test: 3043, 3045 observed — consistent with `get_available_port` handing out
distinct ports from the 3010–3050 range, `tests/test_utils/server.rs:277`.)

### The three experiments that bound the cause

1. **Non-instrumented full suite, same tree:** `logs/build_20260830_225630.log` —
   `1463 tests run: 1463 passed, 2 skipped`. The branch diff is not the trigger.
2. **The 4 failed tests, non-instrumented, isolated rerun:** `4 tests run: 4 passed`
   (16–21s each — note server startup alone takes ~15s on this box).
3. **The 4 failed tests, instrumented, isolated rerun**
   (`CARGO_TARGET_DIR=target/test_police/llvm-cov-target cargo nextest run --test browser ...`):
   `4 tests run: 4 passed` (15–20s each).

Conclusion: the failures only occur under the *combination* of instrumented build + full-suite
parallel load. Isolated runs — instrumented or not — pass.

### Mechanism (inferred, with the three confirmations above)

`.config/nextest.toml` sets `test-threads = 4` (line 2) and `retries = 1` (line 3). Four browser
tests each spawn an engine server plus a Playwright browser simultaneously; under llvm-cov
instrumentation the engine binary's startup latency grows, and the four spawns contend. The 30s
budget in `tests/test_utils/server.rs:422` — `wait_for_server(port, 300).await; // 300 * 100ms = 30s
total — CI under load can take >10s` — is exceeded by the losers of the contention. The comment shows
this budget was already raised once from 10s to 30s for the same class of problem.

`retries = 1` did not save the four hard failures because both tries land in the same contention
window (the whole suite is still running when the retry fires).

## Open questions for the investigator

1. **Slow vs stalled.** Empty child stderr after 30s is ambiguous. Does the engine normally write
   startup output within seconds? If yes, empty output at 30s means the process *stalled*, not merely
   started slowly — and the fix differs (a stall needs a cause hunt: coverage-runtime at-exit writes,
   DB/port locking, tracing init in `bootstrap/logging.rs`). Instrument: add timestamps to
   `wait_for_server` polling, or run one instrumented server manually and time it.
2. **Reproducibility.** Does a cold `--coverage` run fail on other machines / in CI? A single
   reproduced failure per ~1 run is enough to develop against; zero on a faster box would explain why
   this went unnoticed.
3. **Flaky-test signatures.** Rerun coverage with `--retries 0` (or read nextest's saved output) to
   confirm the 3 flaky tests fail with the same server-start signature or a different wait timeout.
4. **Port contention.** Confirm `get_available_port` file-locking (`tests/test_utils/server.rs:277`)
   is not a factor — the failed ports were distinct, so probably not, but a quick look at the lock
   mechanism under 4-way parallelism is cheap.

## Fix options (in preference order)

1. **Dedicated nextest profile for coverage runs.** Add a `coverage` profile to `.config/nextest.toml`
   (inherits `default`, e.g. `test-threads = 2` or a `[[profile.coverage.overrides]]` with
   `filter = 'binary(browser)'` + `test-threads = 1`), and have `build.py --coverage` pass
   `--profile coverage`. Keeps the fast suite parallel; serializes only where the contention is.
   Trade-off: browser binary is 17 tests (`behaviour.rs` 16 + `invariants.rs` 1) at ~16s each under
   coverage — serializing adds ~2–4 min to a 15-min run. Deterministic wins over fast here.
2. **Raise the browser server-start budget.** Bump `wait_for_server(port, 300)` to 600 (60s), or make
   the budget an env override. Trade-off: a genuinely hung server takes 60s to fail instead of 30s.
   This treats the symptom; contention losses just move to 60s.
3. **Lower global threads under coverage only.** `test-threads = 2` in the coverage profile.
   Trade-off: the whole coverage suite slows, not just browser tests.
4. **Auto-retry browser failures in `build.py --coverage`.** Rejected as a primary fix — it masks the
   flake (and `retries = 1` already fails to save the hard failures).

Recommended: option 1, with option 2 as a cheap belt-and-braces if option 1 alone still flakes.

## Secondary issues noticed while running coverage (same plan, cheap fixes)

- **Foreground timeout.** A cold coverage build plus full suite takes ~15 min. A foreground call with
  a 600s timeout dies mid-build and wastes the whole window. Add one line to
  `.agents/skills/test-police/SKILL.md` (Development loop section): "Cold coverage builds take
  15–20 min; run in background."
- **Buffered background logs.** `nohup python build.py ... > log 2>&1` produced an empty log until
  completion (the only progress signal was `pgrep`). Add to the same skill section: "use
  `python -u build.py` for background runs."
- **Stale skill reference.** `.agents/skills/test-police/TEST_INVENTORY.md` describes
  `tests/integration/{adapters,application,flow,model}` (dissolved per `tests/STRATEGY.md`) and
  browser files (`editing.rs`, `interaction.rs`, `structure.rs`, `trigger.rs`) that no longer exist
  (actual: `behaviour.rs`, `invariants.rs`). Prune it to drift-proof content (tier purposes, severity
  bands) and point at `tests/AGENTS.md` (auto-generated, current) as the manifest source of truth.
- **Coverage-quality note (observed, pre-existing, not gating).** The overall gate passes at 88.7%,
  but `src/adapters/driven/llm/transport/utils/client.rs` is 0% (88 lines) and `openrouter.rs` 29.2% /
  `ollama.rs` 32.2% / `response.rs` 38.5% — LLM transport paths are only exercised by the
  `#[ignore]`d LLM suite. Per-file numbers are reference, not gates; listed here so the investigator
  does not mistake them for regression.

## Acceptance criteria

- A cold `python build.py --coverage --target-dir target/test_police --no-fmt` completes with
  **0 failed, 0 flaky** browser tests, twice in a row.
- The fix does not slow the default (non-coverage) suite.
- The two skill-doc one-liners and the TEST_INVENTORY.md prune are landed with the fix.
- The investigation result (slow vs stalled, and why `retries = 1` cannot recover) is recorded here or
  in an ADR if it changes server-start design.
