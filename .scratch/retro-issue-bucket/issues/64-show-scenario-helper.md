# Give agents one way to print a spec scenario by number

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Spec scenarios are `#### Scenario <n>: <title>` blocks in `docs/specs/*.md`. In this run the scouts wrote six separate extractors, and one guessed the wrong heading level. Shell loops looked up 25 scenario numbers. Should the repo ship the lookup?

- **A script**: `python scripts/show_scenario.py 39.4` prints the block and its file.
- **One line in `docs/AGENTS.md`** that names the heading form, so a `rg -A` call is enough.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Scout `e3690c86` entries 121, 128, 140, 172, 198, 203; wrong guess 47 to 48. Scout `69910b82` entries 36, 94. Scout `69dd27b4` entries 107, 109.
