# Extend /statechart: charts of a large model through a view, and sequence diagrams

Type: task
Status: open
Blocked by: —

## Question

Change `.agents/skills/statechart/` so that it draws the diagrams that ticket 09 decided (Q9 C, Q11 A):

1. **A state chart through a view, from a large model.** Today the limit `CHART_STATES = 500` (`scripts/ir.ts`) counts TLC states. So the tool draws no chart for any verify-gate config (769 to 147,768 states), even when a view merges them into a few chart states. Make the limit count the chart states after the view. Without a view, each TLC state is a chart state, so that case stays as it is.
2. **A sequence diagram of a scenario.** A scenario is a named list of model steps. TLC takes only those steps and checks every rule on the way. A step that the model does not allow is an error that names the step. So a diagram cannot show an order that the model does not have.
3. **A sequence diagram of each broken rule's path**, from the TLC counterexample that the report already reads.

The labels go in the view file: the lanes (the participants) and one message for each step (from, to, words, and whether a write was refused). The tool stays generic: no names of the gate go into it.

## The prototype (known)

[assets/09-diagram-prototype/](../assets/09-diagram-prototype/) has the views, the configs, the charts and `scenarios.py`. With the limit raised on a scratch copy, the busy state of one Game took 24 s: 60,976 TLC states became 4 chart states. `scenarios.py` builds a module that extends the TLA+ model with a step counter, and reads the TLC trace. Pele 1.9.0 renders state charts and sequence diagrams.

## Context

- **Time and memory.** With no TLC limit, a large config can run for a long time. Decide whether the tool keeps a limit on TLC states (it has `RERUN_STATES = 100_000` for another use), and say so in `SKILL.md`.
- **A fork.** The skill was copied from `cursor/plugins` (`pstack/skills/statechart`, map Notes). After this change it is a fork, with no sync from upstream. Write that into the map Notes when this ticket resolves.
- The skill has no tests (its `scripts/test/` folder was not copied). An outside check: the three TLA+ models in [assets/00-code-models/](../assets/00-code-models/) give the same reports and charts as before, and the prototype configs give the charts of the prototype.

## Checks

- The outside check above.
- `SKILL.md` and `references/render.md` of the skill describe the new options.
- Do not commit.
