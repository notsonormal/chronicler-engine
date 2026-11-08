# Decide the layout of the folder

Type: grilling
Status: open
Blocked by: 01, 02, 03, 04

## Question

Which subfolders does the top folder have, and which does each initiative have? What closes an initiative, and who says it is done? Where do closed records go, and what happens to the wiki of a closed initiative? Is a one-off issue a file or a folder, and how is it numbered? How does `docs/plans/` fold in? May a map live outside an initiative?

## Context

- **Research (tickets 01 to 03).** Read the `## Answer` of [01](01-claude-projects.md), [02](02-chief-of-staff-wiki.md) and [03](03-other-layouts.md) first. Each asset ends with "Facts for tickets 04 to 08". Zoom into the asset only for the detail.
- **The working layout (owner, 2026-11-04; names from ticket 04).**

  ```
  <top folder>/
    general/        wiki/  wayfinder-maps/  issues/
    long-play/      wiki/  wayfinder-maps/  issues/
    _closed/
  ```

- **A map stays whole.** A map, its tickets and its assets move together. The maps hold 513 `../assets/` links and 221 `../issues/` links. If the three long-play maps move together, 588 of their 600 links stay valid ([the charting facts](../assets/charting-facts.md), "Links").
- **The ticket folder.** Inside a map, `issues/` could become `tickets/`, to match the word that `docs/agents/issue-tracker.md` uses ("Child ticket"). Cost: the 221 links, the tracker doc and the wayfinder skill.
- **Plans.** `.pi/plan-mode.json` writes every plan to one folder (`"planFolder": "docs/plans"`). A one-off issue as a folder needs a change to the plan-mode extension, or plans first land in one folder. Step 2 of `chronicler-after-plan-workflow` archives a plan to `old-docs/archived-plans/`.
- **A folder-level `AGENTS.md` (from ticket 01).** Known: pi loads context files from the agent directory, the working directory and its parents (pi 1.5.0, `docs/configuration.md`, "Context files"). The docs name no load from a subfolder on demand, as Claude Code has. Inferred: an `AGENTS.md` in an initiative folder reaches a chief only when the chief starts in that folder. The repo-root `AGENTS.md` then still loads, as a parent. A short pi test would confirm it.
- **A map as a wiki page (from ticket 02).** Draft PR #1369 of `mattpocock/skills` (open on 2026-11-05) says chief-of-staff "may replace wayfinder, because the wiki outlives the session where a closed wayfinder map doesn't". It moves the destination, the fog and the out-of-scope list into a `destination.md` page of the wiki. So one option puts the destination of an initiative in its wiki, and keeps maps for the decisions under it. The chief-of-staff skill has no rule for a closed goal.
- **Closing (from ticket 03).** Need 5 is the weakest need in all sources: for six of ten approaches the researcher found no close step. PARA moves a project folder to Archives. firstmate runs a teardown. Backlog.md archives task by task.
- **A cap on the top level (from ticket 03).** Johnny.Decimal allows at most ten entries at each level. `.scratch/` has 15 live folders on 2026-11-04. Say whether the top level has a cap.
- **The `plans` symlink (from ticket 04).** The repo root has a tracked symlink `plans -> docs/plans`. The plans move must remove it or point it at the new place.
- **Closed records.** Today `.scratch/_closed/` holds 19 closed maps. Charting suggested that they stay as they are, because moving them costs links and gains nothing.
