# Keep a history of Autoplay runs

Type: task
Status: needs-triage
Blocked by: —

## Question

Should the engine keep an `autoplay_runs` table, so the player can judge the output of Autoplay across runs? What does a row hold, and who writes it?

## Context

- Today a run is transient. It lives in memory only, and a restart ends it (Q4 and Q18 of `old-docs/archived-plans/autoplay-plan.md`). The review-fixes plan keeps that (Q8).
- A table reverses Q4 and Q18. The gate stores no saved busy mark by design (`docs/diataxis/explanation/two-state-channels.md`), so a row would record history and never the busy state.
- The writer rule says only `GameWriter` writes the records of a Game. A history write goes through a save unit.
- The per-cycle tracing lines are the only record of a run today.
