# Decide the layout of the folder

Type: grilling
Status: resolved
Blocked by: 01, 02, 03, 04

## Question

Which subfolders does the top folder have, and which does each initiative have? What closes an initiative, and who says it is done? Where do closed records go, and what happens to the wiki of a closed initiative? Is a one-off issue a file or a folder, and how is it numbered? How does `docs/plans/` fold in? May a map live outside an initiative?

## Context

- **One wiki (ticket 05, 2026-11-11).** [Decide what a wiki holds and what it never holds](05-wiki-contents.md) decided that the repo has one wiki, which follows the chief-of-staff skill as written. The sketches below with `wiki/` in each initiative are older. This ticket decides where the one wiki is, how its `index.md` shows the pages of one initiative, and what `general` holds without a wiki. A history wiki has no close step, so "the wiki of a closed initiative" is now a question about its pages in the shared wiki.

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
- **Closed records.** Today `work/_closed/` holds 19 closed maps. Charting suggested that they stay as they are, because moving them costs links and gains nothing.

## Answer

Grilled with the owner on 2026-11-11.

```
work/
├── wiki/                        the only wiki: flat, with the skill's index.md
├── plans/                       where plan mode writes an approved plan
├── workstreams/
│   ├── general/                 the catch-all, with the same shape as the others
│   │   ├── workstream.md
│   │   ├── wayfinder-maps/<map>/   map.md  issues/  assets/
│   │   ├── issues/                 NN-<slug>.md   (one-off issues)
│   │   └── assets/
│   └── <workstream>/            (same shape)
└── _closed/                     the 19 closed maps of today, unchanged
```

1. **The word is workstream, not initiative.** A workstream is the folder of one subject of work. It may close, but it need not. The owner overturned the premise of [Decide the names of the work records](04-names.md) that the unit has an end: a subject such as "environmental issues", or a Claude project such as "taxes", never closes. "Initiative" implies a push toward something, and "workstream" is true for both kinds. So `general` is a workstream. It differs only as the catch-all for what fits nowhere else.
2. **The workstreams have their own folder, `work/workstreams/`.** The owner found it strange that a folder beside `wiki/` was only implicitly a workstream. The top level is then fixed (`wiki/`, `plans/`, `workstreams/`, `_closed/`), so it needs no cap.
3. **The wiki is `work/wiki/`.** It is a sibling of the workstreams, because a map and a ticket are sources and not wiki pages. It has no structure for workstreams: a page names the workstreams that it affects, and an agent searches the wiki for the name. Nothing in the skill changes.
4. **Each workstream has a `workstream.md`.** It says what the workstream is for and carries a `Status:` line. It gives a done condition only if the workstream can close. It lists no maps, because the list would copy what the folder shows.
5. **A map carries its own state.** `map.md` gets a `Status: open` or `Status: closed` line beside its `Labels:` line. The tracker doc must say so. The upstream wayfinder skill does not change.
6. **Closing, as the default for now.** A closed workstream stays in `workstreams/` with `Status: closed`. A closed map stays in its `wayfinder-maps/`. The owner is not sure what to do with a closed workstream, so this is open for the review after use.
7. **`work/_closed/` holds only the 19 closed maps of today.** They move unchanged. `_closed/` is the owner's temporary store before a delete, and no new map goes there.
8. **A one-off issue is always a file**, `issues/NN-<slug>.md`, numbered from `01` in its `issues/` folder, with a sibling `assets/` folder. It has the shape of a ticket. The number is unique only in its folder, so a reference uses the path.
9. **Plans land in `work/plans/`** (`planFolder` in `.pi/plan-mode.json`). A plan that is implemented at once is archived as today. A plan that must wait moves to the `issues/` of its workstream as `NN-<slug>.md`. The four plans in `docs/plans/` become issues, and the `plans` symlink at the repo root is removed.
10. **Every map is in a workstream.** A map with no better home goes to `general`.
11. **The ticket folder inside a map keeps the name `issues/`.** A rename to `tickets/` costs 221 links and a local change to the upstream skill after each sync.

Rejected:

- **`work/` as the wiki, with the workstreams inside it.** Maps and tickets would become wiki content, under the index, the lint and the history voice.
- **A hub page or an index section for each workstream.** The first is a list that goes stale, and the second changes the skill's index.
- **Two kinds as in PARA** (`projects/` that end, `areas/` that do not). Each new folder needs a choice, and a goal that becomes a standing subject needs a move.
- **area**, **project**: `CONTEXT.md` lists "area" under _Avoid_ for **Room** (1,322 uses in `.scratch/`), and "project" also means a repository.
- **A folder-level `AGENTS.md`.** `workstream.md` does that work.
- **The destination of a map as a wiki page** (draft PR #1369). The destination stays in `map.md`.

Facts for later tickets:

- **07.** Place each live map and each of the four plans in a workstream. The 19 closed maps are not placed.
- **08.** The tracker doc must define "workstream", the `Status:` line of `map.md` and `workstream.md`, and the path rule `work/workstreams/<name>/…`.
- **The build.** Plan mode writes `<planFolder>/<slug>.md` with no number (`.pi/extensions/pi-plan-mode/src/index.ts`, `writePlanFile`), so the change is `planFolder` and `scratchFolders` in `.pi/plan-mode.json`. These files also name `docs/plans`: `ENVIRONMENT.md`, `.vale.ini`, `scripts/validate_docs.py` and its test, `scripts/tests/test_prepare_review_bundle.py`, `tests/infrastructure/guardrails/structure.rs` and `structure_tests.rs`, `.pi/extensions/smoke-break/index.ts`, the plan-mode README and test, `docs/AGENTS.md`, and the skills `chronicler-after-plan-workflow`, `chronicler-after-plan-workflow-plus-review` and `document-review`.

Changed later (2026-11-11): [Decide what a workstream is for, or drop it](11-workstream-purpose.md) dropped the workstream tier. The layout is `work/wiki/`, `work/plans/`, `work/wayfinder-maps/<map>/`, `work/issues/` and `work/_closed/`. Points 1, 2, 4, 6 (the closed workstream), 8 (a one-off issue now sits in a status folder and has no `Status:` line) and 10 no longer hold. Points 3, 5, 7, 9 and 11 hold, with `work/issues/pending/` in place of "the `issues/` of its workstream".
