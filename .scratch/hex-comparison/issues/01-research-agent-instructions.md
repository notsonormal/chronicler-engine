# Research how Hex writes and evolves its agent instructions

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

How does Hex write its agent instructions, and how did they change from the first commit to now? Which practices does this show, and which of them does Chronicler already have?

## Context

- **Hex `AGENTS.md`.** 1486 lines. Read it in full. The first look read only lines 1–400, lines 1440–1486 and the headings: Private Kit Tool, Purpose, Feature Map, Architecture, Invariants, Development, Diagnostics, Future Direction, Current Work.
- **Its history.** 225 commits from 2026-07-14 to 2026-10-31, with 3068 lines added and 1582 deleted (`git log -p --follow -- AGENTS.md`). Look for:
  - what started each change: an incident, a review, or a new feature
  - what was removed, and why
  - the rules that keep the file current, for example "update in the same change"
  - how the size changed over time
  If the diffs do not fit in one session, sample the first 20 commits, the 20 largest and the last 20.
- **Other instruction files.**
  - `src/personal_commands/AGENTS.md`
  - `sdk/commands/workspace-template/AGENTS.md` and its skill
  - `skills/hex-personal-commands/SKILL.md`, a skill for Hex users, not for developers
  - `AGENTS.md` points to `~/code/control/AGENTS.md`, which is not public
- **The Swift original** (`tmp/hex-compare/kitlangton-hex/`). It has `CLAUDE.md` (`AGENTS.md` is a symlink to it), `.cursor/rules/`, and changeset rules for agents. These files are short. Note what carried over into the Rust rewrite and what did not.
- **Chronicler side.** `AGENTS.md`, `CODING_STANDARDS.md`, `CONTEXT.md`, `tests/AGENTS.md`, the skill list in `.agents/skills/`, and the `writing-for-agents` skill.
- **Candidates this touches** ([findings asset](../assets/findings.md)): 2 (invariants in prose), 7 (the seam rule), and the `## Current Work` habit.

## Answer
Hex runs one dense always-loaded spec: `AGENTS.md`, 1486 lines, with purpose, a hand-written module map (97 bullets) and 73 invariants, 31 of them numeric. Workflow rules stay short and point to a private global file.
Agents update the spec inside the change that makes it true: only 7 of 225 such commits touch nothing else, and the median touches 12 files. Rules enter with the incident or review that caused them, and a removal takes its rules out.
I found 10 practices; 5 are `new`. Chronicler covers the seam rule (candidate 7) in substance but in a skill, and covers candidate 2 only partly: `docs/AGENTS.md:149` and `:155-158` write rules that oppose the Hex style.

Asset: [research-01-agent-instructions.md](../assets/research-01-agent-instructions.md)
