# Find how other tools and people lay out agent work in a repo

Type: research
Status: resolved
Blocked by: —

## Question

How do other tools and people lay out their work records: long goals, plans, one-off items and memory? Which layouts meet [the needs](../map.md#notes)? Which ideas does this map not have yet?

## Context

- The owner does not know how other people handle this (2026-11-04). This ticket is where approaches that neither [Find how Claude Projects and similar tools organize work and memory](01-claude-projects.md) nor [Find how Pocock's chief of staff and the LLM wiki keep memory](02-chief-of-staff-wiki.md) covers can show up.
- The mrn-wiki has pages on firstmate (`entities/firstmate.md`), OpenRig (`entities/openrig.md`) and Elephant-Goldfish (`entities/elephant-goldfish.md`). Start from them, and do not redo them.
- Answer the map, not the world (map Notes). One row for each approach, not a survey.

Points to cover:

1. **Agent tools that keep records in a repo.** Cline's Memory Bank, Kiro specs, GitHub Spec Kit, Taskmaster, Backlog.md, firstmate, OpenRig and Elephant-Goldfish. For each one: the folder layout, the names of its units, where its memory is, how a finished item is closed, and how a fresh agent finds a record. Add at most two more that the sources show.
2. **Personal systems.** PARA (Projects, Areas, Resources, Archives) and Johnny.Decimal. How each one handles work with no project, and the archive.
3. **Hidden or visible.** Which tools use a dot folder, and the reason they give.
4. **The needs.** A table: each approach against each of the six needs.
5. **New ideas.** Ideas that the map's working views do not have, each with the approach it comes from.

Write the result to `assets/other-layouts.md`, and link it from the answer.

## Answer

Asset: [Other layouts: how other tools and people lay out agent work records](../assets/other-layouts.md). It holds one section for each of 8 tools (Cline, Kiro, Spec Kit, Taskmaster, Backlog.md, firstmate, OpenRig, Elephant-Goldfish), PARA and Johnny.Decimal, the hidden-or-visible table, the table of ten approaches against the six needs, ten new ideas, and the facts for tickets 04 to 08. Access date of every web source: 2026-11-05. Repo commits are in the asset.

- **known** No approach meets all six needs. firstmate (a home for each chief, `data/` memory, one `AGENTS.md`, teardown) and Johnny.Decimal (an inbox, index and archive at every level) come closest. The marks in the table are **inferred**.
- **known** Need 5 (close a goal as one unit) is the weakest. For six of ten approaches I found no close step. Backlog.md archives task by task. PARA moves a project folder to Archives. firstmate runs a teardown.
- **known** Backlog.md moved from `.backlog/` to `backlog/` because the hidden folder "causes issues with file referencing (e.g., Claude's @ tool)" (task `back-105`). Taskmaster hides its folder to "keep your project directories clean". Spec Kit hides tool files in `.specify/` and shows features in `specs/`. **I don't know** the reason for the dot in Kiro, Cline and Pocock's `.scratch/`.
- **known** OpenRig reads one filename at every level, from general to specific (`rig context trace`). firstmate shares one `captain-shared.md` with its secondmates. Both are close to "every chief reads the `general` wiki first". Neither source covers copied rules in maps.
- **known** Three tools keep a folder path out of their instructions: OpenRig (`rig config get topology.root`), Spec Kit (a script prints `FEATURE_DIR`) and Backlog.md (`backlog instructions`, a configurable directory). This supports "skills point at the tracker doc".
- **known** No tool uses "expedition", "errand" or "general". OpenRig uses "mission" (project → mission → slice). Backlog.md uses "milestone" as one file. PARA and Johnny.Decimal use "area" for standing work.
- **known** PARA splits by "has an end". Projects end and go to Archives. Areas have no end. Johnny.Decimal puts an item in its most specific home and falls back to the inbox of the level above ("Prefer the most specific zero").
- **inferred** Best new ideas: (1) "most specific home first" as the rule for a one-off item. (2) The same fixed entries (index, inbox, wiki, archive) in every expedition and in `general`. (3) A status view that a script computes from the files. (4) An archive with the same layout as the live folder. (5) One path resolver, so no skill names the folder.
- **inferred** Cline's Memory Bank splits memory by rate of change (brief, active state, progress). It also requires reading all files in each task. This gives a shape for each wiki and a cost to cap with a size budget. firstmate's `/stow` enforces a budget. **I don't know** a good size number.
- **inferred** A one-off could be a file that becomes a folder when it gets assets (Backlog.md one file for each task, Spec Kit one folder for each item). This avoids empty folders.
- **inferred** The two bucket maps (`autoplay-follow-ups`, `retro-issue-bucket`) match an inbox (Johnny.Decimal) or `drafts/` (Backlog.md). PARA's "has an end" test can place the other 13 live folders. Ticket 07 has to test both.
- **I don't know** How any of these layouts behaves after months of use. The sources describe layouts. They do not report how well a layout holds up.
