# Recommend a decision for each doc practice

Type: task (AFK)
Status: resolved
Blocked by: 19

## Question

For each of C-01, C-02, C-03, C-09, C-10, C-11 and C-32, what decision does the evidence support: a follow-on task, a brief, a question for a new or existing map, covered, or drop with a reason?

## Context

- **Source.** The map entry **Docs organized as feature maps** in `map.md`, Not yet specified. Ticket 08's answer holds the owner's first view of each practice. `assets/candidates.md`, part 2, holds the evidence rows and part 5 holds the G2 proposal.
- **Inputs.**
  - [Research how agents use the current docs and plans](./10-research-agent-use-of-docs-and-plans.md): questions 1, 2 and 4 of part 7 of its asset. Answer each from the data, and say what the data cannot settle.
  - [Research whether the docs mislead the agents that read them](./19-research-do-docs-mislead.md): C-03 and C-11.
  - [Find where evidence of a working feature lands today](./12-research-where-evidence-lands.md): its draft for `docs/specs/autoplay.md`, because a capability page (C-09) would hold those parts.
  - Ticket 11 and `statechart-models`: C-02 and C-10 may belong there (map, Not yet specified, "Other maps"). Read `.scratch/statechart-models/map.md` and `.scratch/code-concepts/map.md` if they exist.
- **The owner's position** (2026-11-01). An AI writes and reads the docs. He wants no rewrite that rests on a hypothetical. Feature maps appeal to him because the structure is focused and logical. He wants no more research on `pstack`.
- **What Chronicler has (known, from the map).** `docs/specs/` is already most of a feature map, and `chronicler-ui-investigator` uses the browser specs as its selector vocabulary. Four parts are missing: one page for each capability (Autoplay is split over two specs, by test surface), proof records and known gaps (ticket 12), and a rule for which docs exist.
- **Sources.** Research 04, and Lauren Tan's talk: [excerpt](../assets/excerpt-poteto-feature-maps.md) ([08:55]–[11:23]; the full transcript is `tmp/poteto-2500-prs-talk-2026-09-21.md`).
- **Written conflicts.** `docs/AGENTS.md` "Values stay in the code" and "No negative explaining" oppose C-02. Name each conflict beside the practice it touches.
- **Do.**
  1. For each practice, give the recommendation, the evidence (asset and part), and the cost if adopted (files, sessions).
  2. If a practice is a change of one session or less, name the follow-on task. If it is larger, draft the brief or the question for a new or existing map in the asset.
  3. If the evidence does not settle a practice, say so and turn it into one question. Ask at most 3 questions in total.
- **Not in scope.** Edit no doc, skill or `AGENTS.md`. Do not draft a full feature-map page.
- **Output.** Write `assets/task-20-doc-practices.md`. The `## Answer` gives one line for each practice and the questions.
- **When done.** Add `## Answer` and set `Status: resolved`.

## Answer

Asset: [task-20-doc-practices.md](../assets/task-20-doc-practices.md). Six of seven practices end as covered or dropped. No follow-on build and no brief.

- **C-01** One always-loaded spec: **covered**. `AGENTS.md` is 29,112 B, 74% generated; agents already pull domain docs (56% of code-work sessions). No change.
- **C-02** Invariants with exact values: **drop** for `docs/diataxis/` and the instruction files; **covered** in specs, where a test pins the value (Scenario 41.9: 0, 100, 101). Keeps "Values stay in the code" and "No negative explaining". Q2.
- **C-03** Docs change with the behaviour: **covered**. No new rule. The narrow rule that research 19 offers exists already (`docs/AGENTS.md:117` and `:149`). Gap named: `AGENTS.md` prose and skills are not measured.
- **C-09** Feature map by capability: **covered in substance**. No merge of specs into one page per capability (10 capabilities in 2 files each; the validator binds each file to one surface; a merge is 2 sessions or more). Proof records and gaps go to ticket 12 (C-12 to C-14). Q1.
- **C-10** Stable behaviour id: **covered** by the scenario number (561 ids, checked both ways). The open part, where a TLA+ rule id lives, stays in `statechart-models` ticket 04; the owner's leaning there fits. No new map.
- **C-11** Mechanical check on the map: **covered**. One constraint goes to the C-12 follow-on: each "Scenarios N.M" in a proof record resolves to a declared scenario. A check on ids catches 0 of 8 stale cases.
- **C-32** Test name carries the number, boundary triad: **drop**. The `SCENARIO` tag already joins a test to a value.

**Questions (3).**

1. C-09: confirm that each capability stays in two specs (`X.md`, `browser_X.md`)? Recommended: yes.
2. C-02: confirm that `docs/diataxis/` keeps "Values stay in the code" and "No negative explaining", and that a value a client sees lives in a spec scenario? Recommended: yes.
3. Confirm the doc practices in ticket 16, as one extra round? Recommended: yes.
