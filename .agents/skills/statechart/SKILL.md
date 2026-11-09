---
name: statechart
description: "Check a TLA+ model of a system that moves between states, such as a product flow, a protocol, or a service, and draw it as a Mermaid statechart. The report gives the shortest path to each state that breaks a rule. Use for /statechart, or to show a bug as a path through states, or to draw a scenario as a Mermaid sequence diagram."
disable-model-invocation: true
---

# Statechart

The chart shows how a system moves between states, in the words of the person who reads it. That person, the reader, decides whether the behavior is right.

## Run

Work in a scratch folder outside the repo. Write a TLA+ spec and its config. Then run:

```sh
<skill dir>/scripts/statechart SPEC.tla [--config FILE] [--view FILE] [--out DIR]
<skill dir>/scripts/statechart SPEC.tla --scenarios FILE --view FILE [--config FILE] [--out DIR]
```

The second form draws sequence diagrams of scenarios, and no chart. See "Draw sequence diagrams".

`scripts/statechart.ts` is TypeScript, so run it with a runtime that can execute TypeScript.

The first run installs TLC and Java with mise and needs the network.

## Write the spec

- Write each rule that the reader agrees to as an INVARIANT. A rule can be about a screen, a result, a stored value, or a message. If the model has no variable for a rule, add the variable. Never weaken a rule to make it hold.
- Write `Next` as a disjunction of the actions, such as `YouPressPlay \/ YouPressStop`. If you do not, TLC names every step `Next`.
- A state with no step out breaks `NoDeadlock`, also when it is a correct end state. When an INVARIANT or a PROPERTY already has that name, the rule is `CHECK_DEADLOCK`. Add to `Next` an action that is true only in the end states and changes nothing, such as `Done == phase \in {"paid", "failed"} /\ UNCHANGED vars`, and give it a label in `step()`.

## Pick the chart

You do the work that needs judgment: the words on each state and arrow, the phases and their colors, the title, and the report to the reader. Put your work in a view module, and give it with `--view`.

To show the reader's words, the view exports `state(vars)`. The value that `state()` returns picks the form:

- A string gives a flat chart: `export const state = (v) => (v.paid ? "Paid" : "Paying");`
- An array gives parent states: `export const state = (v) => v.main.split(".");`
- An object gives one parallel region for each key: `export const state = (v) => ({ Playback: v.playback, Volume: v.volume });`

Label each state with what the reader can see or cares about, such as "Paying" or "Lease held, write pending". Keep variable names out of labels. In `vars`, a TLA+ sequence is an object with keys from 1, so `v.pc[1]` is `pc[1]`. A set is an array.

The view can also export `step(action, from, to)`. It gets the TLC action name and the values before and after the step, and it returns the label of the arrow. Write each step label as the user sees it happen, such as "you press play" or "the connect fails". Do not name the code or the TLA+ action. Two actions that end the same in the chart get the same label, so that they share one arrow.

When a step can occur only in some conditions, end its label with the condition in brackets, in the reader's words, such as `you press play [while the file loads]`. In a regions chart, the command writes the condition on the other regions on the arrow, in the words of their labels, such as `[when Offline]`. Your brackets replace it, and two arrows from one state can then look the same. So in a regions chart, do not write brackets for a condition on another region. When a value that no label shows decides where a step goes, name the result in the label, such as "the roster read fails".

The view can also export these:

- `group(vars)` returns the name of a phase. Each chart state gets the color of its group. When the model states behind one chart state are in different groups, that state gets no color.
- `colors` gives each group name a color class.
- `title` is a string that the chart shows at the top.

The command calls `state()` and `group()` at most once for each model state, and `step()` at most once for each step. The chart and the report both use those answers. So make each function depend only on its arguments.

The chart does not draw a step that stays in one label. Split a label when the reader must see such a step.

Aim for about 20 arrows or fewer, because a busier chart is hard to read. First return coarser labels from `state()`. Give one label to values that act the same, and leave out a detail that the reader does not need, such as a call in flight whose return changes nothing. Keep each state of a bug path, and keep each mark. When a merge changes a `breaks` state to `mayBreak`, it hides where a rule breaks. Do not merge two states that break different rules, because the reader then cannot see which rule breaks where. If the chart is still too busy, return `null` from `step()` for the steps that the reader does not need, such as a reset that every state has, or a step whose arrow repeats what other arrows show. Obey these rules when you hide steps:

- Keep each chart state reachable by arrows from a start state.
- Keep at least one arrow out of each chart state that has a step out in the model.
- Hide all the steps of an arrow, or none of them. When you hide only some, its condition gets narrower.
- In the hand-off, say which steps the chart hides.

## Read the report

- In the chart, a state that breaks a rule has the class `breaks`. When only some model states behind it break the rule, it has the class `mayBreak`. The Legend names the rules of each mark. When two rules break, it names both on one mark, so say in the hand-off which state breaks which rule. In a regions chart, a state gets a mark when each combination of region states that includes it can break a rule. When a combination can break a rule and none of its states has a mark, each state in that combination gets a mark.
- TLC goes past a state that breaks a rule, so the chart shows the states after a bug.
- Because TLC goes past bugs, a model with no limit on its states can run without end. Keep the constants small, or add a CONSTRAINT.
- With a VIEW or a SYMMETRY, TLC counts states that it treats as the same as one state, and the chart draws that one state. An arrow can lead to it rather than to the state that the step reaches, and the report says so.
- TLC checks the rules in each state past a CONSTRAINT or an ACTION_CONSTRAINT, but it does not explore that state or check it for deadlock. When TLC's shortest path for a rule ends in such a state, the chart draws that state, and the report gives that path. The chart does not draw the other states past the bound. A deadlock in such a state does not show. TLC also checks the rules after each step that an ACTION_CONSTRAINT excludes, also when it reaches the next state another way. When TLC's shortest path for a rule ends with such a step, the chart draws that step.
- The command checks the rules with TLC on the config as written, with one worker. When a rule breaks, the command removes that rule and runs TLC again, until no rule breaks. After the first break, each run stops at 100,000 distinct states, so that a large model does not run without end. When a run reaches that bound or stops on an error, the report lists the rules that it did not check under "Rules not checked", and the chart does not mark them. A rule that is not checked can hold or break. To check it, remove the rules that break from the config, or make the model smaller. The bound adds a module that changes TLC's order of the variables, so for a property TLC can find a different counterexample than a run without the bound.
- `tlc.out` is what TLC printed on the config as written. When the chart does not show the bug path of a rule that breaks, `tlc-RULE.out` is what TLC printed for the run that found it. When TLC repeats its first warning, plain TLC prints it twice, but `tlc.out` shows it once, because TLC leaves the second copy out of the output that this command reads. That happens only when Java runs with `-XX:+UseParallelGC`, since otherwise TLC's first warning is its advice to pass that flag, which it prints once.

## Show the chart

Show `chart.mmd` to the user as a `mermaid` code block. The chart has a budget of 500 chart states. A chart state is one state of the chart, after the view merges TLC states. Without a `state()` in the view, each TLC state is a chart state, so a config with more than 500 TLC states is over the budget. With a `state()`, the command draws the whole graph when TLC reaches at most 200,000 distinct states on the config as written, and then counts the chart states. The bound of 200,000 is for the drawing run only: the first run has no bound, and the later runs stop at 100,000 states (see "Read the report").

Above the budget or the bound, the chart draws only the paths to the rules that break, and when no rule breaks, the command writes no `chart.mmd`. The report says which limit the config is over. It still checks each rule, and the report gives the state count and each bug path. To see the whole graph, give a coarser `state()` or make the model smaller.

The first run has no bound, so a large config can run longer than one tool call allows. Run one config per call. Run a config that can reach more than about 1,000,000 states in the background, with its output in a file.

Render charts with Pele. When the user asks for an image, or a chart is too large to read as text, follow `references/render.md`. Do not use a different renderer unless the user asks for it.

## Draw sequence diagrams

A sequence diagram shows one path of the model as messages between lanes, such as the user, a store, and a worker. The command draws two kinds:

- **The path of each broken rule.** With `lanes` and `message()` in the view, a chart run also writes `sequence-RULE.mmd` for each INVARIANT that breaks, and for a deadlock. It draws TLC's counterexample for the rule. The command does not draw the path of a broken PROPERTY, because that path is a loop.
- **A scenario.** A scenario is a named list of TLA+ actions. With `--scenarios FILE`, TLC takes only those steps, in that order, and checks each INVARIANT of the config in each state on the way. It does not check a PROPERTY, because a scenario is a finite path. The command writes `NAME.mmd` and `tlc-NAME.out` for each scenario, and a `report.md`. It draws no chart.

The view exports these two for a sequence diagram:

- `lanes` is the ordered list of the lanes, such as `[{ id: "You" }, { id: "G", label: "Busy registry" }]`. An id has letters, digits and `_`, starts with a letter, is unique, and is not a Mermaid keyword such as `end` or `box`. The label is what the diagram shows. Without a label, the diagram shows the id. A lane shows only when an item of `message()` uses it, in the order of `lanes`.
- `message(action, from, to)` gets the TLC action name and the values before and after the step, as `step()` does. It returns a list of items, in order. Each item is one of these:
  - `{ call: TEXT, from: LANE, to: LANE }`: a call (`->>`).
  - `{ answer: TEXT, from: LANE, to: LANE }`: an answer (`-->>`).
  - `{ refused: TEXT, from: LANE, to: LANE }`: a call that the receiver refuses (`-x`).
  - `{ failed: TEXT, from: LANE, to: LANE }`: an answer that says the call failed (`--x`).
  - `{ note: TEXT, over: LANE }` or `{ note: TEXT, over: [LANE, LANE] }`: a note over one lane or two.

  Return `[]` for a step that the diagram does not show. `action` has no arguments, also in a scenario, because a counterexample gives none. So read the values that you need, such as the id of a Job, from the change between `from` and `to`. When a rule breaks, the command adds the note `rule RULE breaks here` over all the lanes in use, after the last step.

The scenarios file is a module that exports `scenarios`, an object with one entry for each scenario. The key is the name of the scenario, and the name of its files: lowercase letters, digits and `-`, and a letter first.

```js
export const scenarios = {
  "stop-during-the-upload": {
    title: "A stop while the upload runs",
    steps: ['YouUpload(1, "photo")', "TheUploadStarts(1)", "YouPressStop(1)", "TheUploadEnds(1)"],
    constants: { MaxUploads: "1" },
    rule: "StoppedUploadSavesNothing",
    spec: "upload.md, scenario 3",
  },
};
```

- `title` and `steps` are required. Write each step on one line, as a TLA+ action of the model that starts with its name, with its arguments.
- `constants` replace values of the config, as config text. The config must already set each of them with `=`.
- `rule` and `spec` are for the reader. The report shows them. `rule` must be an INVARIANT or a PROPERTY of the config. `spec` is free text, such as the spec scenario that the diagram shows.

The command adds a module that extends the model with the step counter `ChartStep`. So the model cannot define `ChartStep`, `ChartInit`, `ChartNext`, `ChartSpec` or `ChartEnd`. Each step must also be a step of `Next`. The command takes `Init` and `Next` from the `INIT` and `NEXT` lines of the config. Without them, it reads the definition that `SPECIFICATION` names, which must have the form `Init /\ [][Next]_vars`.

The scenario config keeps the CONSTANTS of the config and its INVARIANTS. It leaves out PROPERTY, CONSTRAINT, ACTION_CONSTRAINT, VIEW, SYMMETRY and CHECK_DEADLOCK. TLC checks for deadlock, so a step that the model does not allow stops the scenario in the state before it. TLC searches every way to take the steps. So when one step can end in more than one state, and the next step is not allowed from one of them, the scenario stops there too. A diagram cannot show an order that the model does not have.

The exit code of a run with `--scenarios`:

- 0: each scenario takes all its steps, and every INVARIANT holds.
- 1: a rule breaks in a scenario. Its diagram stops in the state that breaks the rule and marks it, and the report names the step.
- 2: the model does not allow a step, a step does not parse, or the files are wrong. The error names the scenario, and the step when a step is the cause. The command writes no output.

Show each sequence diagram as a `mermaid` code block, and render it with Pele as you do a chart (`references/render.md`).

## Hand off

Give each chart and each sequence diagram, with its Mermaid text, to the person who must decide. Add one line for each bug: the rule, the reader's sentence for that rule, and the path as the report numbers it. Say what the model leaves out. Give the command that makes the files again, with a new `--out` folder.

When the model describes code, replay each bug path against the real code, as a test or a live repro, one step for each action. If a bug does not occur, think that the model is wrong until you show that it is not.
