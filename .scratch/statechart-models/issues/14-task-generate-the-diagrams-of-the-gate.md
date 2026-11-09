# Generate the diagrams of the gate into docs/diataxis/reference/diagrams/

Type: task
Status: open
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
- `scripts/validate_docs.py` and Vale treat the folder as generated (`GENERATED_DOC_PATHS`).
- `SKILL.md` of `verify-gate`: a maintain pass generates the diagrams again.

## Checks

- `python .agents/skills/verify-gate/check.py` passes and writes the diagrams.
- `python build.py` passes.
- Do not commit.
