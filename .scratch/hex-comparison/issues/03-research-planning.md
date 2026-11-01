# Research how Hex plans work

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

How does Hex plan work before an agent builds it, and how does it record its decisions? Where does this differ from wayfinder maps and `.scratch/` tickets?

## Context

- **Hex sources.**
  - `docs/plans/`: 18 files, 31k words, with screenshots in `docs/plans/rune-ui/`. `docs/plans/rune-ui.md` has a section "Needs A Person To Look".
  - `docs/specs/` (1 file) and `docs/research/` (8 files, 16k words)
  - `prototypes/app-shell/`: `README.md`, `DECISIONS.md`, `SYNTHESIS.md` and three variant HTML files
  - `design/capture_capsule.sigil`: 1363 words in a notation that I don't know
  - `ROADMAP.md`, which `AGENTS.md` calls "the authoritative work list"
  - `docs/releases/` and `release-notes/`
- **What to find.**
  - the structure of a plan
  - how a plan links to code and tests
  - how a plan ends: deleted, kept, or marked done
  - how a prototype turns into a decision
  - what `.sigil` is
- **Task ids.** Ids such as `t421` and `t1153` appear in 24 commit messages. Their tracker is not in the repo (inferred: it is in the private control repo).
- **Chronicler side.**
  - the `wayfinder`, `prototype` and `research` skills
  - the open `.scratch/` maps and `.scratch/_closed/`
  - `docs/plans/`
- **Candidates this touches** ([findings asset](../assets/findings.md)): `ROADMAP.md` as the one work list, and the `## Current Work` habit.

## Answer

Hex records one line of work as one long-lived file in `docs/plans/`. The file keeps the current state, not only the intent:

- a Status line that names the built steps, the evidence, and a numbered landing order with a done mark on each step;
- a limits section, a manual checklist for a person, and the decisions with their rejected alternatives.

`ROADMAP.md` is the one prioritized work list for the whole product. `## Current Work` in `AGENTS.md` holds the live constraints. A finished plan stays in place with its Status rewritten and is deleted only when its premise dies. Chronicler covers the Status line, the outward links, the decision table and the review record. It has no roadmap, no `## Current Work`, no limits section, no per-step order and no ADR folder, although it documents the ADR format.

Asset: [Research: How Hex plans work](../assets/research-03-planning.md). 14 practices: 4 covered, 5 partly covered, 5 gaps.
