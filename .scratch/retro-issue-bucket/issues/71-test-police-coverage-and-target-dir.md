# Scope the coverage step and the target dir of `/test-police`

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`/test-police` runs `python build.py --coverage --target-dir target/test-police` "isolated (avoid lock conflicts)". The build slot already serializes builds, and in this run the slot was free. The run seeded two cold target dirs (7.4 GiB each), left 19.5 GiB in `target/`, and the coverage gate took 877 s of the 28 minutes. The skill also says `cd chronicler_engine` (the repo is `chronicler-engine`). The brief did not list `tests/AGENTS.md`, which the skill says to always read, and the session did not read it. What should the skill ask for?

- **Coverage from the lead's gate**: the lead runs `--coverage` once, and test-police reads `parse_coverage.py` output.
- **A targeted coverage run** over the touched files only.
- **Drop the `--target-dir` advice**, or reuse one dir and remove it at the end. Fix the `cd` line.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- `.agents/skills/test-police/SKILL.md` lines 13, 84 to 86, 113.
- Test-police `d448fab2` entries 19 (`Build slot: free.`), 173, 174, 253, 279; `logs/build_20261019_224303.log` `Total: 877.51s`.
