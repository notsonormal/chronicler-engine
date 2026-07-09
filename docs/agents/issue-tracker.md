# Issue tracker: Local Markdown

Issues and PRDs for this repo live as markdown files in `.scratch/`.

## Conventions

- One effort per directory: `.scratch/<effort-slug>/`
- The PRD (or effort map) is `.scratch/<effort-slug>/map.md`
- Tickets are `.scratch/<effort-slug>/issues/NN-<slug>.md`, numbered from `01`
- Triage state is recorded as a `Status:` line near the top of each ticket (see `triage-labels.md` for role strings)
- Comments / resolution append to the bottom under a `## Comments` (ticket) or `## Decisions so far` (map) heading

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/<effort-slug>/issues/` (creating the directory if needed).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user normally passes the path or the issue number directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` — Destination, Notes, Decisions-so-far, Not-yet-specified, Out-of-scope.
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, numbered from `01`. A `Type:` line records ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:` line records `claimed`/`resolved`.
- **Blocking**: `Blocked by: NN, NN` line near the top of the child body. A ticket is unblocked when every listed blocker is `resolved`.
- **Frontier**: scan `.scratch/<effort>/issues/` for files that are open, unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under `## Answer`, set `Status: resolved`, then append a context pointer (gist + link) to the map's Decisions-so-far in `map.md`.
