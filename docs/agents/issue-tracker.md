# Issue tracker: Local Markdown

The work records of this repo are markdown files in `work/`. They are maps, tickets, one-off issues, plans and the wiki.

Upstream skills and older records say `.scratch/`. In this repo, that means `work/`. The [path table of the move](../../work/wayfinder-maps/work-records/assets/path-moves.md) gives the new path of each old `.scratch/` and `docs/plans/` path.

## Names

```
work/
├── wiki/                  the one wiki: index.md and its pages
├── plans/                 plan mode writes plans here
├── wayfinder-maps/<map>/  map.md, other .md files, issues/, assets/
├── issues/                one-off issues, in four status folders
│   ├── pending/
│   ├── ready/
│   ├── in-progress/
│   ├── closed/
│   └── assets/            files that one-off issues link
└── _closed/               the closed maps of the old layout, unchanged
```

- **Map**: the plan of one piece of work, in `work/wayfinder-maps/<map>/map.md`. Its Destination is the end of that piece of work. Here, a map means a wayfinder map, not a game Map. A map that closes gets `Status: closed` and stays in `work/wayfinder-maps/`. No new map goes to `work/_closed/`.
- **Ticket**: one step of a map, in `work/wayfinder-maps/<map>/issues/NN-<slug>.md`, numbered from `01` in each map.
- **One-off issue**: an issue that belongs to no map, in `work/issues/<status>/NN-<slug>.md`.
- **Wiki**: the one wiki of the repo. `work/wiki/index.md` lists its pages. Pages follow the Wiki section of the upstream chief-of-staff skill (`mattpocock/skills`, `skills/in-progress/chief-of-staff/SKILL.md`): its page kinds, the history voice, and the ingest, query and lint operations.
- **Plan**: a file that plan mode writes in `work/plans/`. A plan that is implemented at once moves to `old-docs/archived-plans/`. A plan that must wait becomes a one-off issue in `work/issues/pending/`.
- **Closed map of the old layout**: a map in `work/_closed/`. Its text keeps the paths of the old layout, as history.

## Status

| Record | Path | Status |
|---|---|---|
| Map | `work/wayfinder-maps/<map>/map.md` | a `Status:` line: `open` or `closed` |
| Ticket | `work/wayfinder-maps/<map>/issues/NN-<slug>.md` | a `Status:` line: `open`, `claimed` or `resolved` |
| One-off issue | `work/issues/<status>/NN-<slug>.md` | its folder: `pending/`, `ready/`, `in-progress/` or `closed/` |

A one-off issue has no `Status:` line. Its folder is its status. `triage-labels.md` maps each triage role to a folder.

## Conventions

- A map's `Status:` line is under its `Labels:` line, or under its title when it has no `Labels:` line.
- Assets that a map or a ticket links are in `work/wayfinder-maps/<map>/assets/`. Assets that a one-off issue links are in `work/issues/assets/`.
- A map keeps a file that stays on this machine in its `assets/local-only/` folder, for example raw data or large trial output. Git ignores that folder, and `rg` skips it, so cite each file in it by its path.
- Comments go at the bottom, under a `## Comments` heading. A ticket's resolution goes under `## Answer`, and a map's under `## Decisions so far`.
- The gate step `validate-work-records` checks that each link of a live record resolves. It also checks that each backticked `work/` path in `AGENTS.md`, `docs/` and `.agents/skills/` exists. Assets and `work/_closed/` are history and are not checked.

## One-off issues

The number of a one-off issue is unique across all of `work/issues/`. A new issue takes one more than the highest number in `work/issues/`. The number stays with the issue when it moves, so `work/issues/*/NN-*` finds it.

A change of status is a move to another folder. Move the file with the script, which also rewrites the links to it:

```
python scripts/move_issue.py <NN> <pending|ready|in-progress|closed>
```

A move by hand leaves broken links, and the gate step `validate-work-records` fails.

## When a skill says "publish to the issue tracker"

Create a new one-off issue in `work/issues/pending/`, with the next free number. For a ticket of a map, create the file in `work/wayfinder-maps/<map>/issues/`.

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user normally passes the path or the issue number directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `work/wayfinder-maps/<map>/map.md`, with the sections Destination, Notes, Decisions so far, Not yet specified and Out of scope.
- **Child ticket**: `work/wayfinder-maps/<map>/issues/NN-<slug>.md`, numbered from `01`. A `Type:` line records the ticket type (`research`, `prototype`, `grilling` or `task`). A `Status:` line records `open`, `claimed` or `resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top of the child body. A ticket is unblocked when every listed blocker is `resolved`.
- **Frontier**: scan `work/wayfinder-maps/<map>/issues/` for files that are open, unblocked and unclaimed. The first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under `## Answer` and set `Status: resolved`. Then add the context pointer (gist and link) to the Decisions so far of `map.md`. In parallel runs, the coordinator owns that edit.
