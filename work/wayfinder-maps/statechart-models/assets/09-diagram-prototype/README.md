# Diagram prototype of ticket 09

Diagrams generated from the unchanged gate model (`.agents/skills/verify-gate/model/VerifyGate.tla` at sync commit `e6715c60`). Made on 2026-11-08 in `tmp/t09/diagrams/`.

## State charts (`charts/`)

`/statechart` with one change on a scratch copy of the tool: in `scripts/ir.ts`, `CHART_STATES = Number(process.env.CHART_STATES ?? 500)`, run with `CHART_STATES=200000`. The limit counts TLC states today, so a view that merges states cannot help.

| Chart | Config | View | TLC states | Chart states | Arrows |
|---|---|---|---|---|---|
| `busy` | `gate.cfg` (`run-and-jobs.cfg` with `Env = {"plain", "run", "stop"}`) | `busy.view.mjs` | 60,976 | 4 | 7 |
| `job-life` | `job-life.cfg` (1 Game, 1 Action, Stop) | `job-life.view.mjs` | 134 | 11 | 16 |
| `run-life` | `run-life.cfg` (a copy of `runs.cfg`) | `run-life.view.mjs` | 769 | 14 | 18 |
| `gate` | `gate.cfg` | `gate.view.mjs` | 60,976 | 8 | 21 |

`gate` mixes two axes (the holder and the failure line), and it is hard to read. One chart for each axis reads better.

The command, from a folder that holds `VerifyGate.tla`, the view and the config:

    CHART_STATES=200000 <copy of statechart>/scripts/statechart VerifyGate.tla --config gate.cfg --view busy.view.mjs --out out-busy

Pele renders the `.mmd` files: `node /tmp/pele-render/render.mjs <file>.mmd`.

## Sequence diagrams (`scenarios/`)

`scenarios.py` holds four named orders, each a list of model actions. It writes a module `Scenario.tla` that lets TLC take only those steps, with every rule as an invariant and `NotDone` as the end mark. It reads the TLC trace and draws a Mermaid sequence diagram. A step that the model does not allow stops the script, so a diagram cannot show an order that the model does not have. The labels are in the script; the tool change of ticket 13 puts them in the view file.

- `stop-during-the-narrator-call`: the write after the stop is refused.
- `late-end-of-a-stopped-job`: the drop of Job 1 leaves the holder of Job 2.
- `late-end-of-a-stopped-run`: the end of run 1 leaves the holder of run 2.
- `delete-races-a-switch`: the run-1 code gap; the diagram ends at the broken rule `DeleteNotActive`.
