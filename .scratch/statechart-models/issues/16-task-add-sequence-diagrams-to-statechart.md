# Add sequence diagrams to /statechart: checked scenarios and the paths of broken rules

Type: task
Status: resolved
Blocked by: 13

## Question

Change `.agents/skills/statechart/` so that it draws the two kinds of sequence diagram that ticket 09 decided (Q9 C, Q11 A). Split from ticket 13 by its plan review (owner, 2026-11-08, decision 9, Option B).

1. **A sequence diagram of a scenario.** A scenario is a named list of TLA+ actions. TLC takes only those steps and checks every INVARIANT of the config on the way. A PROPERTY is not checked, because a scenario is a finite path. A step that the TLA+ model does not allow is an error that names the step: keep TLC's deadlock check on, so that TLC stops in the last state that the scenario reaches (the prototype does not do this, `scenarios.py:92`). TLC names each scenario step by the generated next-state action and prints no parameter values, so the tool reads the action of step k from the scenario list, through the step counter. Give the counter a `Chart` prefix, as `cfg.ts` does, so that it cannot clash with a variable of the TLA+ model. So a diagram cannot show an order that the TLA+ model does not have.
2. **A sequence diagram of each broken rule's path**, from the TLC counterexample that the report already reads (`readCounterexample`, `scripts/frontends/marks.ts:80`).

The tool stays generic: no names of the gate go into it.

## The decisions (plan review of ticket 13, 2026-11-08)

- **Scenarios live in one `scenarios.mjs` file, passed with `--scenarios`** (owner, decision 11, Option C). Each scenario has its steps, its constants and its title. The prototype scenarios change constants (`MaxJobs`, `Games0`, `Env`), and ticket 14 wants a rule and a spec scenario on each one. That data is not labels, so it does not go in the view.
- **The message API** (owner, decision 12, Option B). The view exports an ordered list `lanes` and a function `message(action, from, to)` that returns a list of calls, answers, refused arrows and notes. One function serves both scenarios and counterexamples. A counterexample gives no action arguments, so the function must not need them. `checkView` throws on any export except `state`, `step`, `group`, `colors` and `title` (`scripts/frontends/view.ts:58`, `:70`), so the new exports go into `VIEW_EXPORTS` with the same checks. The prototype needs a call and its answer, notes over one or two lanes, refused arrows, and lanes that show only when used (`scenarios.py:114-207`).
- **The path of a broken PROPERTY (a loop) is out of scope** (owner, decision 13, Option A). Only rules of the INVARIANT kind broke in run 1.

## The prototype (known)

[assets/09-diagram-prototype/](../assets/09-diagram-prototype/) has `scenarios.py` and four scenarios. `scenarios.py` builds a module that extends the TLA+ model with a step counter (`scenarios.py:55-66`), and reads the TLC trace (`:89-107`). It hard-codes 14 invariants (`:19`) and leaves out `TypeOK` and the PROPERTY `StopLeavesIdle`. Pele 1.9.0 renders the sequence diagrams (`scenarios/*.svg` have the class `pele-sequence`). No sequence backend exists in the tool today (known: `rg sequence scripts/` finds nothing).

## Checks

- The four prototype scenarios, written in the new format, give the message order of `scenarios/<name>.mmd`. `delete-races-a-switch` ends at `DeleteNotActive`. Use `git show 61a8855c:.agents/skills/verify-gate/model/VerifyGate.tla`, because tickets 10 and 11 can change the working tree first.
- A scenario with `TheTextIsSaved(1)` before `AJobStartsAGeneration(1)` stops with an error that names that step.
- `delete-race.cfg` with a view gives a sequence diagram of the path of `DeleteNotActive`.
- The three TLA+ models in `assets/00-code-models/` still give the same `report.md` and `chart.mmd` as after ticket 13.
- `SKILL.md` and `references/render.md` of the skill describe `--scenarios`, the new view exports and the new outputs.
- Do not commit.

## Not in scope

- The state chart limit (ticket 13).
- The path of a broken PROPERTY (a loop).
- Writing diagrams into `docs/diataxis/reference/diagrams/`, hashes, and stale-file checks (ticket 14).
- Changes to `check.py` or the views of `verify-gate` (ticket 14).
- Rendering to SVG inside the tool: Pele stays outside, as in `references/render.md`.
- Running `build.py`: it does not cover `.agents/skills/statechart/`.

## What already exists

- TLC lookup and runs: `findTlc`, `runJava`, `PINNED_FLAGS` (`scripts/run.ts`).
- Generated wrapper modules with `Chart*` names: `boundRun`, `chartRun`, `configText` (`scripts/backends/cfg.ts`).
- Trace reading: `readCounterexample` (`scripts/frontends/marks.ts:80`), `readTrace` (`scripts/frontends/trace.ts`), `readMessages`, `actionOf`, `messageState` (`scripts/frontends/messages.ts`).
- Config sections: `readConfig` (`scripts/frontends/config.ts`), `spec.written`.
- View checks and JSON values: `checkView`, `jsonVars` (`scripts/frontends/view.ts`).
- Mermaid text escaping: `scripts/backends/mermaid.ts`.
- Output folder handling and exit codes: `scripts/statechart.ts`, `scripts/worker.ts`.
- The prototype runner and drawer: `assets/09-diagram-prototype/scenarios.py`.

## Failure modes

| New code path | Failure | Handled? |
|---|---|---|
| Scenario: a step that the TLA+ model does not allow | Deadlock in the generated module | Error that names step k, exit 2. |
| Scenario: a rule breaks before the end | The trace stops early | Draw up to the rule and mark it, as the prototype does. Exit 1. |
| Scenario: a step expression does not parse | SANY error | Show the SANY tail, exit 2 (existing `tail()`). |
| Counter name clashes with a variable | SANY error | Use a `Chart` prefix. |
| `message()` throws, or returns a lane that is not declared | Bad diagram | Throw with the action name, as `group()` does for an unknown color. |
| View with the new exports in chart mode | Rejected by `checkView` today | Add them to `VIEW_EXPORTS`. |
| `--out` not empty when called from `check.py` | Exit 2 | Existing behaviour. Ticket 14 must use a new folder. |

## Answer

**Done (known).** `/statechart` now draws the two kinds of sequence diagram. (1) With `--scenarios FILE`, it runs each scenario through TLC and writes one diagram for each scenario. (2) In a chart run, a view with `lanes` and `message()` also gives `sequence-RULE.mmd` for each broken INVARIANT and for the deadlock rule. The tool has no gate names.

### What changed (all under `.agents/skills/statechart/`)

- `scripts/frontends/scenarios.ts` (new): `checkScenarios` reads and checks the scenarios file. `actionName` takes the action name from a step.
- `scripts/backends/sequence.ts` (new): `sequenceLines` writes the Mermaid `sequenceDiagram` from `lanes`, the steps and `message()`.
- `scripts/backends/cfg.ts`: `scenarioRun` writes the wrapper module (`ChartStep`, `ChartInit`, `ChartNext`, `ChartSpec`, `ChartEnd`). `scenarioConfig` writes its config.
- `scripts/frontends/spec.ts`: `initAndNext` finds `Init` and `Next`. It reads them from `INIT` and `NEXT`, or from the SANY XML of the `SPECIFICATION`. `readSany` now keeps the symbol table.
- `scripts/frontends/view.ts`: `lanes` and `message` are in `VIEW_EXPORTS`, with the same kind of checks.
- `scripts/run.ts`: the TLC setup moved out of `check()` into `prepare()`, with no change of behaviour. The new `runScenarios()` uses it.
- `scripts/pipeline.ts`: `compile()` adds `sequence-RULE.mmd`, and a "Sequence diagrams" section at the end of `report.md`. `compileScenarios()` is new.
- `scripts/backends/mermaid.ts`: `esc` is exported and `titleLines` is split out, so both diagram kinds use the same escaping.
- `scripts/worker.ts`, `scripts/statechart.ts`: the `--scenarios` flag and the usage text.
- `scripts/ir.ts`: the types `Lane`, `Said`, `Scenario`, `ScenarioRun`, `ScenarioResult`, and `View.lanes` / `View.message`.
- `SKILL.md` (a new section, "Draw sequence diagrams", and the second command form) and `references/render.md`.

### Format choices (the details that the ticket left open)

- **The scenarios file.** `export const scenarios = { "<name>": { title, steps, constants?, rule?, spec? } }`. The key is the name and also the file name (`^[a-z][a-z0-9-]*$`). Each step is one line, a TLA+ action that starts with its name. `constants` are config text, and each one must replace a `NAME = value` that the config already has. That rule catches typos. `rule` must be an INVARIANT or a PROPERTY of the config. `spec` is free text. The report shows both, so ticket 14 can find the rule and the spec scenario of each diagram there. The file must export only `scenarios`.
- **The view.** `lanes = [{ id, label? }, ...]`, in order. The id is a Mermaid participant id (a letter, then letters, digits or `_`, and not a Mermaid keyword). `message(action, from, to)` returns a list of `{ call | answer | refused | failed: TEXT, from, to }` and `{ note: TEXT, over: LANE | [LANE, LANE] }`. I added `failed` (`--x`) for a failed answer, because the prototype needs `L--xJ1: the call fails`. `action` never has arguments, also in a scenario, so one `message()` serves both kinds of diagram.
- **The broken-rule mark.** `Note over <first lane in use>,<last lane in use>: rule RULE breaks here`.
- **A run with `--scenarios` draws no chart.** It runs only the scenarios. It needs `--view` with `lanes` and `message()`, and it checks this before TLC runs. Default `--out`: `SPEC-scenarios`.
- **Output files.** For a scenario run: `report.md`, `NAME.mmd` and `tlc-NAME.out`. For a chart run: also `sequence-RULE.mmd`. The title of a counterexample diagram is `Path to the RULE bug`.
- **Exit codes with `--scenarios`.** 0 when all scenarios end and every rule holds. 1 when a rule breaks in a scenario (the diagram is drawn up to the break and marked). 2 when a step is not allowed, a step does not parse, a file is wrong, or a constant or a rule is unknown. As before, exit 2 writes nothing.
- **The scenario config.** It keeps the CONSTANTS (with the overrides), TYPE and TYPE_CONSTRAINT, and every INVARIANT, with `ChartEnd` last. It leaves out PROPERTY, CONSTRAINT, ACTION_CONSTRAINT, VIEW, SYMMETRY and CHECK_DEADLOCK. TLC runs without `-deadlock`, so the deadlock check stays on. Each step arm is `ChartStep = k /\ (step) /\ Next /\ ChartStep' = k+1`. So a step must also be a step of `Next`, which is stricter than the prototype.
- **Why the module needs `Init` and `Next` (known).** TLC refuses a spec with two `[][..]_v` conjuncts (tested on a toy model). So the wrapper cannot reuse `Spec` as it is. If the SPECIFICATION does not have the form `Init /\ [][Next]_vars` (it may be nested), the command stops and asks for `INIT` and `NEXT`.

### Checks and results

| Check | Result |
|---|---|
| The 4 prototype scenarios in the new format (`tmp/t16/scn/gate.scenarios.mjs`, view `gate.sequence.view.mjs`, base config `scn.cfg` = the BASE constants of `scenarios.py`), on `61a8855c:VerifyGate.tla` | Known. Exit 1, 27 s on the first run and 7 s on a second run. The message order equals `scenarios/<name>.mmd` for all four. `stop-during-the-narrator-call` and `late-end-of-a-stopped-job` are equal byte for byte. Two differences remain. `late-end-of-a-stopped-run` has no `participant J2`: the prototype declared J2 from a note over `G`, and no item uses J2. `delete-races-a-switch` has its break note over `You,G`, not over `G,P` (the generic span). It ends at `DeleteNotActive`. |
| `TheTextIsSaved(1)` before `AJobStartsAGeneration(1)` (`bad.scenarios.mjs`) | Known. Exit 2: "The scenario save-before-start cannot take step 2, TheTextIsSaved(1): the TLA+ model does not allow it after step 1. TLC stops in the state after step 1." No output. |
| `delete-race.cfg` with a view that has `state()`, `lanes` and `message()` | Known. Exit 1. The chart is as before, and `sequence-DeleteNotActive.mmd` gives the path delete check → switch → delete, with the break note. |
| The 3 TLA+ models of `assets/00-code-models/` against `tmp/t13/after-*` | Known. `diff -rq -x tlc.out`: no difference for `job-life`, `autoplay` and `swipe-state` (`swipe-state` breaks a rule, exit 1). The prototype `job-life` chart also equals `charts/job-life.mmd` and `tmp/t13/after-proto-job-life`. |
| A rule breaks before the last step (`edge.scenarios.mjs`) | Known. Exit 1. The diagram stops after step 3 of 4, and the report says so. |
| Other failure paths | Known. A step that does not parse: exit 2, with the TLC/SANY tail and the line of each step. An unknown constant, an unknown rule, a lane that `lanes` does not declare (the error names the action), and a view with no `lanes`: exit 2 with a clear message each time. |
| `INIT Init` / `NEXT Next` in place of `SPECIFICATION Spec` | Known. The diagram is the same. |
| Pele 1.9.0 renders the 5 new `.mmd` files, and a test diagram with `;`, `#`, `%%` and `<` in its texts | Known. All render, and the texts show unchanged. |
| `tsc --strict` | Known. The same 2 errors as before the change (`exit` typed `number` in `run.ts`). No new error. |

Scratch: `tmp/t16/` (`scn/run.sh`, the scenarios files, the views, `out-*`, `times.txt`, `tool-before/` = the tool after ticket 13).

### Left open

- **A step with more than one outcome (inferred).** TLC searches every way to take the steps. So if one outcome of step k does not allow step k+1, the scenario stops with the step error, even when another outcome would go on. No action of `VerifyGate` has a choice inside it once its arguments are given, so I could not test this. `SKILL.md` says it.
- **Step form (known).** A step must start with an action name, so `\E x \in S : A(x)` is refused. Give the arguments instead.
- **The rule and the spec scenario are only in `report.md` (known).** Ticket 14 must parse the report, or read `scenarios.mjs` with Node. I did not add a JSON summary.
- **The view of the gate (known).** `gate.sequence.view.mjs` is a scratch port of `scenarios.py` `draw()`. Ticket 14 owns the real view of `verify-gate`.
- `SKILL.md` has no rule for how many items a sequence diagram may have (guessed: not needed yet).

### Coordinator check (2026-11-08)

- The three TLA+ models of `assets/00-code-models/` give the same outputs as after ticket 13 (`diff -rq -x tlc.out` of `tmp/t13/after-<name>` and `tmp/t16/reg-<name>`, known).
- `tmp/t16/out-scn/` holds the four scenario diagrams and their TLC outputs. `tmp/t16/out-delrace/` holds `sequence-DeleteNotActive.mmd`. The step error names the step: "The scenario save-before-start cannot take step 2, TheTextIsSaved(1)" (known).
- For ticket 14: the rule and the spec scenario of each diagram are in `report.md` and in `scenarios.mjs`. The real view of `verify-gate` is ticket 14's work.
