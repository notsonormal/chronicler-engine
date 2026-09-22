# Acceptance gate: retries off, 5 green full-gate runs, stress loop, serialization override deleted

Type: task
Status: resolved
Blocked by: 07, 08, 08b, 09, 10, 11

## Question

The destination check, executed empirically per ticket 04's ratified
verification bar (option C). Nothing below is relieved by argument — the
numbers are the acceptance.

In order:

1. **Remove `retries = 1`** from `.config/nextest.toml` before any acceptance
   run. A succeeding run on a retry hides exactly what this gate measures.
   Keep the `slow-timeout` overrides.
2. **Five consecutive green full `build.py` runs**, zero legacy flake
   signatures (story-log CDN gate timeout, posture-change no-fire). Any
   retried-then-green test in a run = the run does not count and is itself a
   signature to investigate. Any new-signature flake: record counts, open it
   against the originating ticket with the evidence — do not paper over it.
3. **Stress loop, final state.** Re-run the ~50-run posture-flow loop from
   ticket 06 against the final suite, no retry masking. Acceptance = zero
   legacy signatures; each new signature gets a one-line characterisation
   (which tier, which mechanism guess [inferred], which ticket it belongs on).
4. **Serialization override decision.** With the race closed and the gate
   green, delete the browser-binary `threads-required = "num-test-threads"`
   override (added for a 2-core box; the box is now 8-core). **Measure wall
   time before/after** — the run either proves the speedup or shows a new
   spawn-contention flake, in which case reinstate and record that the
   override earns its keep *empirically* now (a different status than
   "stabilization knob on a stale assumption").
5. **Escalation path.** If a stress loop or a gate run fails after step 1:
   the ticket that introduced the most plausible cause is reopened with the
   evidence, and this ticket stays open. The destination is "races impossible",
   so a failure here is the design saying something true, not bad luck.

Wall-time reporting: record the browser binary's total and per-tier wall times
(step 4 before/after) — this is the user's 2nd objective (speed) finally being
measured rather than asserted.

Record all numbers verbatim under `## Answer` — don't summarise runs as "5
green"; paste the counts. When green, the map's destination is satisfied and
the map itself can be closed.

## Answer

**Resolved — all five steps pass. The destination is satisfied.**

Only file changed: `.config/nextest.toml` (−9 lines). `git diff`: `retries = 1`
removed, and the browser `threads-required = "num-test-threads"` override block
deleted. `slow-timeout` (default 60s + architecture 180s + llm 300s) kept. No
test source, no production source, no spec touched.

### Step 1 — retries removed

`.config/nextest.toml` before any acceptance run: `retries = 1` deleted from
`[profile.default]`; the three `slow-timeout` overrides left intact. Every run
below therefore had a single attempt per test.

### Step 2 — five consecutive green full-gate runs (override still in place)

Each run is a full `python build.py` (retries off). Verbatim epilogue lines:

| run | wall | architecture | guardrails | integration | browser | real `FLAKY` lines |
|---|---|---|---|---|---|---|
| 1 | 71s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |
| 2 | 66s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |
| 3 | 69s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |
| 4 | 67s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |
| 5 | 69s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |

All 5 exit code 0. Zero retried-then-green tests in any run (`^\s+FLAKY \d+/\d+`
grep = 0 per run) — so no run was disqualified. Zero legacy flake signatures:
`never settled '#world-posture-status'` = 0 across all runs; neither story-log
CDN gate timeout nor posture-change no-fire appeared. No new-signature flake,
so no originating ticket was reopened and no ticket was opened.

Logs: `tmp/acceptance/run_{1..5}.log`, `tmp/acceptance/results.txt`.

### Step 3 — stress loop, final suite, no retry masking

`scripts/stress_posture.sh 50` drives
`worlds::test_world_posture_change_autosaves_server_state` directly through the
test binary, bypassing nextest and its retries. Run twice: once with the
override in place (pre-deletion), once after deletion in the final config.

Both loops:

```
runs:               50
pass:               50
fail:               0
lost interaction:   0
failed runs:        none
```

Zero `never settled '#world-posture-status'` signatures, zero failures, so no
new-signature characterisation was required. Logs:
`tmp/acceptance/stress.out` and `tmp/acceptance/stress_final.out`.

### Step 4 — serialization override deleted: 2.8× faster, no spawn contention

**Before** (override in place), browser binary timed three ways:

| sample | wall |
|---|---|
| gate runs 1–5, browser step | 50.30s / 52.85s / 53.96s / 53.45s / 54.03s |
| direct `cargo nextest run -E 'binary(browser)'` ×3 | 54.33s / 54.28s / 54.02s |

**After** (override deleted), direct `cargo nextest run -E 'binary(browser)'` ×12:

```
19.20s  19.07s  19.16s  19.14s  19.32s  18.88s
19.70s  19.41s  19.30s  19.39s  19.39s  18.99s
```

All 12 exit code 0; zero real `FLAKY` lines. 54.2s → 19.2s ≈ **2.8× faster**
(35s saved per browser tier). The 4-way-parallelism spawn-contention risk the
override was added for did **not** reproduce in 12 consecutive direct runs on
the 8-core box plus 5 further full gates (below) = 17 runs. The override is
deleted and does **not** earn its keep; the speedup is proven, not asserted.

Full-gate wall time fell from 66–71s (override) to 33.7s total
(`Total: 33.71s`, final run 5).

### Final configuration — five more full gates, shipped state

Step 2's five runs predate step 4's deletion, so five further full `build.py`
runs confirm the **shipped** config (retries off **and** override deleted):

| run | wall | architecture | guardrails | integration | browser | real `FLAKY` lines |
|---|---|---|---|---|---|---|
| 1 | 36s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |
| 2 | 35s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |
| 3 | 34s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |
| 4 | 34s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |
| 5 | 34s | `nextest: 1 passed, 0 failed` | `nextest: 137 passed, 0 failed` | `nextest: 1605 passed, 0 failed, 2 skipped` | `nextest: 20 passed, 0 failed` | 0 |

Per-tier wall, final config: architecture 1.48–1.96s, guardrails 1.11–1.18s,
integration 10.39–10.89s, browser 19.17–20.04s. Zero FLAKY lines; zero legacy
signatures. Logs: `tmp/acceptance/final/run_{1..5}.log`.

**Acceptance bar: 5 consecutive green full-gate runs satisfied in the shipped
configuration** (10 total green full gates across steps 2 and 4), validator
`141 declared, 141 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface
mismatch(es), quarantine 85/85` unchanged — the change is config-only, so no
test or scenario moved.

### Step 5 — escalation path

Not exercised: no stress-loop or gate failure occurred after step 1. No ticket
was reopened.

### Speed objective (user's 2nd objective), measured

The 2.8× browser-tier speedup was **measured**, not asserted. The measurement
also retroactively explains ticket 07's "490s under nextest's serialization
override" observation: it was this override forcing the browser binary to run
one test at a time. With the race closed by the settle-gate design and the
override deleted, the browser tier runs its 20 tests 4-way parallel in ~19s.

### State left for ticket 13 (post-acceptance sweep)

`tmp/acceptance/` (24 logs + `results.txt` + stdouts) now joins the sweeps of
ticket 13, along with `scripts/stress_posture.sh`, which this ticket needed one
last time. `tmp/posture_stress/` holds the two 50-run loops' per-run logs.

