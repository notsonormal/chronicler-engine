# Record contention and effect size in benchmark reports

Type: task
Status: resolved
Blocked by: 02

## Question

`scripts/diagnostic_benchmark.py` reports omit the contention condition and the effect
size, so a single run on a busy host reads as signal. In the window a 663 s gate was
described as "the only thing running on the box" when it was not, and a retro finding was
retracted late — "My 'slot acquisition' framing was hand-waving… Candidate 4 is not a
finding. Drop it" — after the operator had to ask for repeated runs.

## Work

Record in both the JSON and the markdown report:

- the contention condition at run time: load average, and whether the build slot was held;
- the run-to-run spread when a step is repeated;
- an effect-size field against the baseline.

Flag a delta below a stated threshold as within noise. Do not build the
minimum-effect-size gate that blocks filing a finding — that is judgement, and it was
dropped with the rest of F28 in
[Decide which mechanisms to build](02-decide-mechanisms-to-build.md).

## Done when

- A report shows the contention condition and the effect size.
- A sub-threshold delta is labelled as noise, not as a speedup.

## Answer

Resolved 2026-10-05. `scripts/diagnostic_benchmark.py` now records the three missing facts, and a
sub-threshold delta is labelled `within noise`, never as a speedup.

**Contention condition.** `capture_contention()` records the 1/5/15-minute load average
(`os.getloadavg`, `None` when unavailable) and whether the machine-wide build slot is held, by
probing `build_slot.probe_shared_lock` (`None` when the probe itself fails). It lands in the JSON
under `contention` and in the markdown under a `## Contention` table placed above the scores it
qualifies.

**Run-to-run spread.** `--repeat N` runs the suite N times; `merge_runs()` reports each scenario's
mean score plus max-minus-min spread, and the overall spread is taken across the run-level overall
averages, not across scenarios. A single run reports `runs: 1` and a zero spread.

**Effect size against the baseline.** `--compare BASELINE` now computes an `effect_size` section
instead of printing a table and returning, so it is written to both the JSON and the markdown. The
stated threshold is `NOISE_THRESHOLD_PCT = 20.0` percent of the baseline score — the figure the
2026-10-04 sweep put on contended-host micro-measurements — and `classify_delta` labels anything
below it `within noise`. A missing baseline scenario is `no baseline`, not a fabricated zero.

Verification (all 2026-10-05):

- 17 new unit tests in `scripts/tests/test_diagnostic_benchmark.py` cover the load-average and
  slot probes, the spread math (including that overall spread is across runs, not scenarios), the
  noise boundary (exactly 20% is a change; below it is noise), the markdown rendering, and the
  `--json` wiring. `python build.py py-tests`: 302 passed. `python build.py docstrings`: clean.
- A report generated from synthetic results shows both blocks:

  ```
  | Fact | Value |
  |------|-------|
  | Load average (1m / 5m / 15m) | 1.42 / 0.90 / 0.60 |
  | Build slot held | yes |
  | Benchmark runs | 2 |
  | Overall run-to-run spread | 0.2 |
  ...
  Stated noise threshold: **20%** of the baseline score. A delta below it is within noise, not a speedup.
  Overall: 5.0 -> 4.1 (-0.90, -18.0%) — **within noise**.
  | `storage_timeout` | 5.8 | 6.2 | +0.40 | +6.9% | within noise |
  ```

- `REPORT_DIR` was `Path(__file__).parent.parent.parent / "tmp" / "diagnostics"`, i.e.
  `/workspace/tmp/diagnostics` — outside the repo, a leftover from the tree move. It now resolves
  to `<repo>/tmp/diagnostics`.

**The report has no data source.** `run_benchmark()` shells
`cargo test --test diagnostic_benchmark`, and that target does not exist:

```
$ cargo test --test diagnostic_benchmark --no-run
error: no test target named `diagnostic_benchmark` in default-run packages
```

`tests/` has no top-level `diagnostic_benchmark.rs`, `Cargo.toml` declares no such `[[test]]`, and
nothing in the tree emits `BENCHMARK_RESULT`. `fe09985a` renamed the target to `diagnostic`
without updating the script; `d34c8785` then removed that target. Its old entry points
(`DefaultGameService`, `GameServiceContext`, `make_test_context`) are gone from `src/`, so it is a
rewrite, not a restore. `--diagnostic-benchmark` therefore exits 1 before writing anything. That is
a separate decision — restore, rebuild smaller, or drop the flag — so it graduated to
[Decide the fate of the dead diagnostic benchmark](10-dead-diagnostic-benchmark.md).
