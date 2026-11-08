# Map: Work records

Labels: wayfinder:map

## Destination

The work records of this repo live in one visible folder: the maps, the one-off items, the plans, and the memory of each long goal. Research on how others do this chose its names and layout. Each long goal has its own folder and wiki, and a general folder holds what has no goal and what applies to all goals. The records are moved, the docs and skills are updated, and the link check and the gate pass. After a period of use, the owner keeps the layout or changes it.

## Notes

- **Origin (owner, 2026-11-04).** A session on Pocock's `chief-of-staff` skill. Its report is [the chief-of-staff report](assets/chief-of-staff-report.md): the workflow in numbers, the five orchestration axes, the bitter lesson, and the adapted chief of staff. The facts that charting found about `.scratch/` are in [the charting facts](assets/charting-facts.md).
- **The problem (owner, charting).** You are the scheduler and the only memory across maps. Loose items have no home, so maps become buckets. Standing rules are copied forward into the Notes of each map. The name "scratch" says "throwaway", and the word already means two other things in this repo.
- **The needs (owner, charting).** Each research ticket and the review after use check each approach against this list. The layout must make these easy:
  1. See each goal and where it stands, without asking an agent.
  2. Put a one-off item in one obvious place.
  3. Keep the rules that span maps in one place, not copied into each map's Notes.
  4. Let a chief own one folder and find everything it needs from there.
  5. Close a goal as one unit.
  6. Let a fresh agent find any record from one doc (the tracker doc).
- **Working views (owner, 2026-11-04).** The research and grilling tickets test these views, and can overturn them:
  - The top folder is visible, with no dot.
  - The names: an **expedition** is the folder of one long goal. An **errand** is a one-off item with no map. **`general`** is the expedition of everything with no goal, and of what applies to all goals. **Map** and **ticket** keep their wayfinder meanings.
  - Each expedition has `wiki/`, `maps/` and `errands/`. Every chief reads the `general` wiki first, then its own.
  - An errand uses the triage statuses of `docs/agents/triage-labels.md`. A ticket keeps `open`, `claimed` and `resolved`.
  - `docs/plans/` folds into errands. `docs/` then holds only current-state docs.
  - The skills point at the tracker doc instead of naming the folder.
  - The names describe the workflow, so they go in `docs/agents/`, not in `CONTEXT.md` (the game glossary).
- **Scope (owner, charting).** `.scratch/` and `docs/plans/`, in this repo. A layout that also fits other repos is a plus, but it is not required.
- **Execution is in this map (owner, charting).** Research, then grilling, then build. A build ticket ends with `/simple-after-plan`.
- **The outside check (owner, charting).** A link check and the full `python build.py` after each move, then the owner's review after a period of use. The move is a script, so it can run again with other names or be reversed. The owner's words: "move it and then see if we don't like it and then change it". Broken links stay silent, so the link check is required.
- **Research answers the map, not the world (owner, charting).** A research ticket answers the needs above and the open grilling questions. It does not make a full inventory. The [hex-comparison map](../_closed/hex-comparison/map.md) made long lists of findings that ended as "already covered" or "not worth doing".
- **Concurrency.** Other sessions write to `.scratch/` at the same time. A move runs only when no other session runs. The owner decides the scope of each commit.
- **Skills.** `/research` for a research ticket. `/grilling` and `/domain-modeling` for a grilling ticket. `/writing-for-agents` for an edit to a skill or an `AGENTS.md`. `/simple-after-plan` for a build ticket.
- **Sources. Start from these; do not redo them.**
  - [The chief-of-staff report](assets/chief-of-staff-report.md) and [the charting facts](assets/charting-facts.md).
  - Pocock's skills at `mattpocock/skills@4ae2317`: `skills/in-progress/chief-of-staff/`, `skills/in-progress/software-factory/`, `skills/in-progress/loop-me/`, `skills/engineering/wayfinder/`. Clone with `git clone --depth 1 https://github.com/mattpocock/skills.git`.
  - The owner's research wiki `notsonormal/mrn-wiki` (private; at `37db57b` on 2026-11-04). Clone over SSH: `git clone --depth 1 git@github.com:notsonormal/mrn-wiki.git`. Pages: `concepts/chief-of-staff-pattern.md`, `concepts/wayfinder-decision-map.md`, `concepts/repo-as-memory.md`, `concepts/orchestration-axes.md`, `concepts/orchestration-principles.md`, `concepts/software-factory.md`, `entities/chronicler-engine.md` (the ADR history), `entities/firstmate.md`, `entities/openrig.md`. Its `SCHEMA.md` is an example of a wiki's rules.
  - Clone outside the repo, for example to `/tmp/research/`. `/tmp` can be cleared, so clone again when a clone is gone.
- **Names.** "Expedition", "errand", "`general`", "work records" and "the needs" are working names until the names ticket decides them.

## Decisions so far

<!-- the index — one line per closed ticket: enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [Find how Claude Projects and similar tools organize work and memory](issues/01-claude-projects.md) — "Project" means three things. Only the Claude Code project (public beta) is close to an expedition: a goal, instructions, memory files, a Library and threads, with a state view (Waiting on you … Resolved). The user writes instructions, the model writes memory. Claude Code memory is current-state: a short index (200 lines or 25KB) and topic files on demand. Work outside a project has no named place. The docs fit a project to "an area you keep feeding", which may make the bucket maps expeditions (inferred).
- [Find how Pocock's chief of staff and the LLM wiki keep memory](issues/02-chief-of-staff-wiki.md) — Karpathy's "LLM Wiki" gist (2026-04-04) is the origin, and its wiki is "kept current". "Write history, not the present" is Pocock's own rule (`5beb35b`), and no script checks it. The skill defines no goal folders. Pocock's posts show one wiki per person, per deliverable or per domain agent. Nothing is merged upstream after `4ae2317`, but draft PR #1369 says chief-of-staff "may replace wayfinder" and moves a map's destination and fog into a `destination.md` wiki page. Fits need 3, part of 1, 4 and 6, not 2 or 5. The report's "broken procedure paths" was wrong: they resolve after install (PR #1363).
- [Find how other tools and people lay out agent work in a repo](issues/03-other-layouts.md) — No approach meets all six needs. firstmate and Johnny.Decimal come closest, and need 5 (close a goal) is the weakest everywhere. Backlog.md dropped its dot because the hidden folder broke file references (`back-105`). OpenRig and firstmate layer a general file under the specific one. OpenRig, Spec Kit and Backlog.md keep the folder path out of their instructions. No tool uses expedition, errand or `general`. PARA and Johnny.Decimal say "area" for standing work. Ready rules: "has an end" (PARA) and "prefer the most specific zero" (Johnny.Decimal).

## Not yet specified

- **The build.** The move script, the link rewrite, and the edits to the docs, the skills, `.pi/plan-mode.json` (`planFolder` and `scratchFolders`), `.gitignore` line 215 and the source of the generated `guardrails.md`. It waits for the grilling tickets.
- **The link check.** Whether it becomes part of the gate (for example in `scripts/validate_docs.py`), or stays a step of the move script.
- **The broken links of today.** More than 100 relative links in live maps do not resolve ([the charting facts](assets/charting-facts.md), "Links"). Whether the move fixes them, or leaves them.
- **The review after use.** When it happens, and how the owner checks the needs. One candidate (ticket 03, idea 3): a status view that a script computes from the files, as the Backlog.md board and `task-master list` do. It would also serve need 1.
- **The old path.** Whether `.scratch` keeps working for a while after the move. Backlog.md wrote the old folder name into the config of old installs (`back-105`).
- **The hand-off to the later chief-of-staff map.** What this map leaves ready for it.

## Out of scope

- **How a chief asks fewer questions** (owner, charting). The "decided for you" log, the prework before a grilling round, and the one-week trial of [the chief-of-staff report](assets/chief-of-staff-report.md). A later map.
- **`old-docs/`, `tmp/` and the mrn-wiki** (owner, charting). `old-docs/` is history, `tmp/` stays a scratch area, and the mrn-wiki has its own `SCHEMA.md`.
- **A navigation test with a fresh scout** (owner, charting). The owner chose to move the records, use them, and change them, instead.
