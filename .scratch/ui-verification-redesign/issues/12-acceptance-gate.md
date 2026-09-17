# Acceptance gate: retries off, 5 green full-gate runs, stress loop, serialization override deleted

Type: task
Status:
Blocked by: 07, 08, 09, 10, 11

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
