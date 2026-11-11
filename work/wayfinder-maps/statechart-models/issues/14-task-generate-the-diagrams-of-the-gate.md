# Generate the diagrams of the gate into docs/diataxis/reference/diagrams/

Type: task
Status: resolved
Blocked by: 10, 11, 12, 13, 16

## Question

Make `verify-gate` generate its diagrams with the tool of tickets 13 and 16, and write them into `docs/diataxis/reference/diagrams/` (ticket 09, Q9 C and Q10). `game_flow.md` links to them after ticket 15.

## What to generate

- **A state chart for each part, one axis each.** Start list from [the prototype](../assets/09-diagram-prototype/): the busy state of one Game, the life of one Job, the life of one Autoplay run. A chart that mixes two axes is hard to read (the prototype's `gate` chart).
- **A sequence diagram for each named order.** Start list: a stop during the narrator call, the late end of a stopped Job, the end of a stopped run, the reset in one hold. Add the orders that `game_flow.md` explains today in prose. Each scenario names the rule and the spec scenario that it shows.
- **A sequence diagram of each broken rule's path.** These go to the trial log or to a bug ticket, not into `docs/`. `check.py` writes them when a rule breaks.

## Decisions for this ticket

- **Who writes the files.** `check.py` of `verify-gate`, because it has TLC through `mise`. `build.py` has no JDK (ticket 04), so it cannot generate them.
- **A stale file.** Each generated file carries a hash of the TLA+ model, the views and the scenarios. A check with git and Python only finds a file that is older than the model. The default is a warning in the end summary of the build log, as in ticket 08. The two checks can share code.
- **The format.** Mermaid in markdown pages, or `.mmd` files with SVG. The default is Mermaid in markdown: markdown viewers draw it, and an agent reads the text.
- `scripts/validate_docs.py` and Vale treat the folder as generated (`GENERATED_DOC_PATHS`; the work added a folder list, `GENERATED_DOC_DIRS`, instead).
- `SKILL.md` of `verify-gate`: a maintain pass generates the diagrams again.

## Checks

- `python .agents/skills/verify-gate/check.py` passes and writes the diagrams.
- `python build.py` passes.
- Do not commit.

## Answer

**Done (known).** `check.py` of `verify-gate` generates 13 pages into `docs/diataxis/reference/diagrams/`: 4 state charts and 9 sequence diagrams (after the reviews). Each page holds one Mermaid block in markdown. A rule that breaks gets its path as a sequence diagram under `tmp/verify-gate/paths/`.

### What changed

- `.agents/skills/verify-gate/diagrams/` (new): `diagrams.toml` (the list of pages and the output folder), four chart configs and views (`busy`, `failure`, `job-life`, `run-life`), `scenarios.mjs` with `scenarios.cfg`, and `sequence.view.mjs` (`lanes` and `message()` for every action of the model).
- `check.py`: a run of every config ends with the diagrams. It writes them only when every config gives its expected result and an input changed. `--diagrams` writes them in every case. For each break that is not known, it runs `/statechart` with `model/VerifyGate.view.mjs` and prints the path to `sequence-RULE.mmd`. It passes its own `mise` TLC and JDK to `/statechart` (`TLA2TOOLS`, `JAVA`). It adds `classDef` fills for the group colors, because `chart.mmd` leaves them to the CSS of Pele.
- `model/VerifyGate.view.mjs`: takes `lanes` and `message` from `diagrams/sequence.view.mjs`.
- `scripts/verify_coverage.py`: `diagram_inputs_hash`, `diagram_marker`, `diagram_pages`, `diagram_warning`. The inputs are `model/*.tla`, every file in `diagrams/`, `check.py` and the scripts of `/statechart`. The configs of `model/` are not inputs. `warning_lines` adds one line per stale skill, so the `build.py` end summary shows it. It uses no git, as in ticket 08.
- `scripts/validate_docs.py`: `GENERATED_DOC_DIRS` and `is_generated_doc`. `scripts/vale_lint.py`: `GENERATED_DIRS`. The `build.py` docstring names the second warning.
- Tests: 8 in `test_verify_coverage.py` (`DiagramTests`), 3 in `test_validate_docs.py`.
- `SKILL.md`: a new section *The diagrams*, maintain step 6 (update the diagrams), the output lines, and *Read a break* now uses the path that `check.py` draws.
- `docs/AGENTS.md`: index regenerated (the new folder is listed).

### The diagrams

| Page | Kind | Source | Size |
|---|---|---|---|
| `busy_state` | chart | `busy.cfg` (the prototype `gate.cfg`) | 60,976 TLC states, 4 chart states |
| `failure_line_and_options_failure` | chart (new) | `failure.cfg`: 1 run, Action and Options Jobs, stop, story writes | 75,619 TLC states, 4 chart states |
| `job_life` | chart | prototype `job-life.cfg` | 11 chart states |
| `run_life` | chart | prototype `run-life.cfg` | 14 chart states |
| `stop_during_the_narrator_call` | sequence | rule `StoppedJobWritesNothing`, dashboard.md 39.5 | 5 steps |
| `late_end_of_a_stopped_job` | sequence | `NextJobKeepsHolder`, dashboard.md 39.5 | 9 steps |
| `stop_on_an_idle_game` | sequence | `StopLeavesIdle`, dashboard.md 39.17 | 7 steps |
| `reset_during_a_job` | sequence | `StoppedJobWritesNothing`, reset.md 7.3 | 6 steps |
| `delete_during_a_job` | sequence | `StoppedJobWritesNothing`, games.md 19.5 | 6 steps |
| `switch_then_delete` | sequence | `DeleteNotActive`, games.md 19.2 | 4 steps |
| `autoplay_run_of_one_cycle` | sequence | `BudgetBound`, autoplay.md 41.1 | 23 steps |
| `late_end_of_a_stopped_run` | sequence | `RunHoldsItsGame`, autoplay.md 41.17 | 10 steps |

The three charts of the prototype are equal byte for byte to `assets/09-diagram-prototype/charts/` on the model of today (known). The prototype scenario `delete-races-a-switch` is gone: ticket 10 fixed that race, so the model no longer allows its order.

### Choices the ticket left open

- **One page per diagram**, named in snake_case like the other docs. A scenario key `a-b` gives the page `a_b.md`.
- **The marker line.** `<!-- Generated by verify-gate from its TLA+ model, inputs HASH. Do not edit. -->`. A page without the marker is not read, so a page written by hand is never removed.
- **One hash for all pages.** An edit of any input makes every page old, and the next run writes all 12 again (about 90 s). A run with current pages costs nothing more.
- **No spec numbers on a page.** `docs/AGENTS.md` says that a Diátaxis doc cites no spec scenario number. So a page names its rule. The spec scenario is in `scenarios.mjs` (`spec`) and in the `/statechart` report.
- **The messages read the state, not the intent.** For example, the note of a stop says "No failure line stays" only when the new state has none. So the path of a broken rule shows the bug. A mutant of `StopNoFailureLine` drew "A failure line stays" before the break note (known).
- **Each page links back to `game_flow.md`** in a `## Document References` section.

### Checks and results

| Check | Result |
|---|---|
| `python3 .agents/skills/verify-gate/check.py` | Known. Exit 0. 7 configs hold, then `diagrams: wrote 12 pages` (2 min in all). A second run prints `diagrams: current` (30 s). |
| `check.py --mutants` | Known. Every mutant `caught`. |
| A new break (the mutant `stop-keeps-failure-line` on `jobs`, in a scratch copy of `model/`) | Known. Exit 1, and the line `path to the break: tmp/verify-gate/paths/jobs/sequence-StopNoFailureLine.mmd`. |
| Pele 1.9.0 renders the Mermaid block of each of the 12 pages | Known. All 12 render. I did not test GitHub's Mermaid renderer (guessed: it reads the same syntax). |
| `python3 scripts/verify_coverage.py check` after the run | Known. No warning. The unit tests cover an old page, no page, and a page of another skill. |
| `python build.py` | Known. Pass (`logs/build_20261109_212444_2191883.log`), with no `WARNING` line. |

Scratch: `tmp/t14/` (hand runs, the Pele renders in `tmp/t14/pele/`). `tmp/verify-gate/diagrams/` holds the `/statechart` outputs of the last run.

### Left open

- **For ticket 15 (known).** `game_flow.md` does not link to the pages yet. The prose orders of `game_flow.md` that a diagram now shows are: the stop during a model call, the reset in one hold, the late save after a stop, a reset or a delete, the loop of the Autoplay accept, the clear of the failure lines, and the switch of the active Game. A config write during a run and the busy refusal of an edit have no diagram: the model has no step for a refused request, so no scenario can draw one.
- **One page per diagram (decided, owner, 2026-11-09).** The document review asked whether the pages with the same lead should become two pages, "Gate state charts" and "Gate sequences". The owner decided: keep one page per diagram. Ticket 15 links to each page.
- **The `autoplay_run_of_one_cycle` page is long (inferred).** It has about 40 lines, because one cycle is an Impersonate Job and then a Narration Job. It can be too long for one screen. Ticket 15 decides whether it stays.

### After the review

The comment fixer deleted one comment. The document review gave 1 must, 15 should and 9 nit findings. The changes:

- **The lead of a page (must).** A scenario run checks the INVARIANTs only, not a PROPERTY. The lead now says "every invariant held". When the `rule` of a scenario is a PROPERTY (`StopLeavesIdle`), the page says that TLC checks it in the configs, not along the order.
- **The lead states the model, the scope and the rule in words.** A chart lead takes its scope from the first line of its config (`\* Scope: ...`). A scenario has a new `says` field. `check.py` gives `/statechart` a copy of the scenarios without `says`, because `/statechart` refuses a key that it does not read.
- **The hash inputs** now include `check.py` and the `/statechart` scripts. The warning tells the agent to run `check.py`, because `--diagrams` checks no config.
- **The words of the views follow `CONTEXT.md`:** "Autoplay run" (not "run"), "the player", "Quantifier call", "Not started", and one wording for each action. The sequence diagrams have a `GameCatalogue` lane for the second half of a reset, a create, a delete and a switch. The page `failure_lines` is now `failure_line_and_options_failure`.
- **`SKILL.md`:** the output table has the `diagrams: not written` and `no path to the break` lines, and two sentences that only repeat a fact are gone.

Rejected: the joined edge labels ("A or B or C"). `/statechart` joins the labels of parallel edges with "or", so a view cannot change that. The trim of the `check.py` docstring was done, but the `scenarios.mjs` header keeps its format line.

### After the code review

- **Standards.** `_diagrams` is split into `_chart_page`, `_sequence_pages` and one `_NotWritten` failure. The path of `/statechart` is in one place (`verify_coverage.statechart_scripts`). The warning says "older than their inputs" in every place, and `SKILL.md` names the other two warnings. The `except` of `diagram_warning` is narrow. Rejected: a separate diagram module, one constant for the generated folder across `validate_docs.py` and `vale_lint.py` (they already mirror each other on purpose), and one shared action switch for all views (each view has its own words for its own axis).
- **Spec.** A new scenario, `config-write-during-a-job` (13 pages now), draws the prose order "A config write passes while a plain Job runs". A broken PROPERTY now prints `path to the break, as steps: .../report.md`, because `/statechart` draws a sequence diagram for a broken INVARIANT only. A scratch mutant of `StopLeavesIdle` on `jobs` showed the line.
- **Left open (known).** `/statechart` draws no sequence diagram for a broken PROPERTY. A change to `/statechart` (`pipeline.ts`, `compile`) can add it.

