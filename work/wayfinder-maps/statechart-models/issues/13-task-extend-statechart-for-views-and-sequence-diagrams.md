# Extend /statechart: charts of a large model through a view

Type: task
Status: resolved
Blocked by: —

## Question

Change `.agents/skills/statechart/` so that it draws a state chart of a large TLA+ model through a view (ticket 09, Q9 C, Q11 A).

Today the limit `CHART_STATES = 500` (`scripts/ir.ts:139`) counts TLC states. So the tool draws no chart for any verify-gate config (769 to 147,768 states), even when a view merges them into a few chart states. Make the limit count the chart states after the view.

Today the count comes before the view: `check()` compares the TLC count with `CHART_STATES` (`scripts/run.ts:412`, `:427`), and the `ChartBudget` constraint stops the dump run at `CHART_STATES` (`scripts/backends/cfg.ts:28`). The view is applied later, in `compile()` (`scripts/pipeline.ts:46`). So the choice "whole chart or bug paths only" moves into `compile()`, and the dump run gets its own bound on TLC states. Also change the budget sentence in `scripts/backends/report.ts:85` and the usage text in `scripts/statechart.ts:9-10`. Without a view, each TLC state is a chart state, so that case stays as it is.

## The decisions (plan review, 2026-11-08)

- **The split** (owner, decision 9, Option B). This ticket keeps the limit after the view. [Ticket 16](16-task-add-sequence-diagrams-to-statechart.md) takes the sequence diagrams: the renderer, the scenarios and the path of each broken rule.
- **The bound of the dump run** (owner, decision 10, Option B). A new constant of 200,000 TLC states, the prototype value. The `games` config has 147,768 TLC states. Above the bound, the tool falls back to bug paths only, as today, and says so in `report.md`. `SKILL.md` names the bound. `RERUN_STATES = 100_000` (`ir.ts:137`) stays for its own use: it bounds only the runs after the first break (`run.ts:337`).

## The prototype (known)

[assets/09-diagram-prototype/](../assets/09-diagram-prototype/) has the views, the configs and the charts. With the limit raised on a scratch copy, the busy state of one Game took 24 s: 60,976 TLC states became 4 chart states.

## Context

- **Time and memory.** The first TLC run has no bound today (`scripts/run.ts:337`). The new cost is the `-dump dot` run and the graph that the tool keeps in memory. Measure both on the `games` config.
- **A fork.** The skill was copied from `cursor/plugins` (`pstack/skills/statechart`, map Notes). After this change it is a fork, with no sync from upstream. Write that into the map Notes when this ticket resolves.
- The skill has no tests (its `scripts/test/` folder was not copied). The outside check is in Checks.

## Checks

- The outside check, run in this order:
  1. Before the change, run the unchanged tool on each TLA+ model in `assets/00-code-models/` with the README command and `--out /workspace/chronicler-engine/tmp/t13/before-<name>`. The stored outputs came from Temurin 21, so a diff against them can come from the JDK.
  2. After the change, run again into `tmp/t13/after-<name>`. `diff` the `report.md` and `chart.mmd` files, but not `tlc.out`, which holds timestamps.
  3. For the prototype, use `git show 61a8855c:.agents/skills/verify-gate/model/VerifyGate.tla`, because tickets 10 and 11 can change the working tree first. Run the four configs with their views, and `diff` each output with `charts/<name>.mmd`.
  4. A config above the bound falls back to bug paths, and `report.md` says so.
- `.agents/skills/verify-gate/SKILL.md:52` (a chart only up to 500 states) is corrected.
- `SKILL.md` and `references/render.md` of the skill describe the new limit and the bound.
- Do not commit.

## Not in scope

- Sequence diagrams of any kind (ticket 16).
- Writing diagrams into `docs/diataxis/reference/diagrams/`, hashes, and stale-file checks (ticket 14).
- Changes to `check.py` or the views of `verify-gate` (ticket 14).
- Restoring the upstream `scripts/test/` folder, or syncing with `cursor/plugins`.
- Rendering to SVG inside the tool: Pele stays outside, as in `references/render.md`.
- Running `build.py`: it does not cover `.agents/skills/statechart/`.

## What already exists

- TLC lookup and runs: `findTlc`, `runJava`, `PINNED_FLAGS` (`scripts/run.ts`).
- Generated wrapper modules with `Chart*` names: `boundRun`, `chartRun`, `configText` (`scripts/backends/cfg.ts`).
- View checks and JSON values: `checkView`, `jsonVars` (`scripts/frontends/view.ts`).
- Bug paths over the budget: `pathGraph` (`scripts/passes/paths.ts`).
- Output folder handling and exit codes: `scripts/statechart.ts`, `scripts/worker.ts`.
- Pele render with the CSS: `assets/00-code-models/render.mjs`.

## Failure modes

| New code path | Failure | Handled? |
|---|---|---|
| Dump run at the bound | The TLC count reaches 200,000, so the graph is partial | Fall back to bug paths, and say so in `report.md`. |
| Labelling a large graph | Memory or time on about 150,000 states | The bound limits it. Measure on `games`. |
| Chart states after the view above `CHART_STATES` | A chart that nobody can read | Fall back to bug paths, as today. |

## Answer

**Done (known).** `/statechart` now counts the chart states after the view. The limit of 500 stays, but it counts chart states, not TLC states, when the view has a `state()`.

### What changed (all under `.agents/skills/`)

- `statechart/scripts/ir.ts`: new `DUMP_STATES = 200_000` (the bound of the dump run). `CHART_STATES` (500) and `RERUN_STATES` (100,000) stay. `CheckResult` has two new fields: `bound` and `fallback` (the bug-path graph, paths and properties).
- `statechart/scripts/worker.ts`: passes `view.state ? DUMP_STATES : CHART_STATES` to `check()`. Without a `state()` in the view, each TLC state is a chart state, so that case stays as it was.
- `statechart/scripts/run.ts`: `check(spec, config, bound)`. The dump run happens when TLC reaches at most `bound` states. The bug-path result is now built in every case (it costs little) and kept as `fallback` when the whole graph is drawn.
- `statechart/scripts/backends/cfg.ts`: `chartRun(spec, tag, bound)`. The `ChartBudget` constraint uses the bound.
- `statechart/scripts/pipeline.ts`: `compile()` draws the whole graph, counts the chart states (`chartSize`), and falls back to `fallback` when the count is above 500.
- `statechart/scripts/backends/report.ts`: three first sentences. (1) The view merges N TLC states into M chart states, over 500. (2) TLC reached more states than the bound of 200,000 for a chart through a view. (3) The old sentence for the 500 budget, unchanged.
- `statechart/scripts/statechart.ts`: the usage text.
- `statechart/SKILL.md` ("Show the chart") and `references/render.md`: the new limit and the bound.
- `verify-gate/SKILL.md:52`: corrected (one exact-text edit).

### Checks and results

| Check | Result |
|---|---|
| Step 1, before the change: the 3 models of `assets/00-code-models/` into `tmp/t13/before-*` | Known. `report.md`, `chart.mmd` equal the stored outputs (Temurin 25, so no JDK difference). Only `chart.png` and `chart.svg` are missing, as they are not tool output. |
| Step 2, after the change, into `tmp/t13/after-*` | Known. `diff -r` excluding `tlc.out` shows no difference for all 3. |
| Step 3, the model from `git show 61a8855c:...VerifyGate.tla`, the 4 prototype configs with their views | Known. `chart.mmd` equals `charts/busy.mmd`, `job-life.mmd`, `run-life.mmd` and `gate.mmd`, byte for byte. Times: busy 28 s, job-life 5 s, run-life 9 s, gate 61 s. |
| `games` config (147,768 TLC states) with `busy.view.mjs` | Known. 2 chart states, 2 arrows. 52 s, peak about 660 MiB (java and node together, sampled with `ps`; one other java of about 56 MiB was running). |
| Step 4a, above the bound: `games` with `MaxJobs = 2` (6,916,020 TLC states, the first run took about 11 min) and `busy.view.mjs` | Known. No dump run. `report.md` says: "TLC reached 6,916,020 states, more than the bound of 200,000 states for a chart through a view, so the command wrote no chart.mmd." Peak 1.66 GiB, from the first TLC run, which has no bound (old behavior). |
| Step 4b, chart states above 500: `runs.cfg` (769 TLC states) with a view that returns every variable (769 chart states) | Known. The report says the view merges 769 states into 769 chart states, over 500, and the command wrote no chart.mmd. |
| Step 4c, the same with a rule that breaks (`Bad.tla`, one invariant on top of `VerifyGate`) | Known. The chart draws only the bug path (2 states). Exit 1. |
| No view, a config over 500 with a broken rule (the same `Bad.tla`), old tool against new tool | Known. `report.md` and `chart.mmd` are equal. |
| `tsc --strict` on the scripts | Known. The only errors (`exit` typed `number`, in `run.ts`) are also in the original code with the same `tsc`. The change adds none. |

Scratch output: `tmp/t13/` (run script `run.sh`, `mem.sh`, the prototype copy in `proto/`).

### Left open

- **Map Notes (known).** The skill is now a fork of `cursor/plugins`. The coordinator must write this in the map Notes, as the ticket says. I did not edit the map.
- **The count is states only (known).** The budget counts chart states, not arrows. A view can still give 500 states and many arrows. This is as before.
- **Cost after the view (inferred).** The labelling of a large graph takes the most time: `gate` (60,976 TLC states) took 61 s, against 28 s for `busy` on the same graph, because its view has more states to label. I measured no config above 148,000 TLC states that stays under the bound.
- **The first TLC run is still unbounded (known).** A model of millions of states runs to the end before the tool falls back to bug paths. This is as before and the ticket did not ask to change it.
- Ticket 10 changed `VerifyGate.tla` while I worked. I used `61a8855c` for all runs, as the ticket says. I did not run `build.py` (no file outside `.agents/skills/` changed by me).

### Coordinator check (2026-11-08)

- `diff -rq -x tlc.out` of `tmp/t13/before-<name>` and `tmp/t13/after-<name>` shows no difference for `job-life`, `autoplay` and `swipe-state` (known). `after-proto-busy/chart.mmd` equals `assets/09-diagram-prototype/charts/busy.mmd` (known).
- The map Notes now say that the skill is a fork of `cursor/plugins`.
