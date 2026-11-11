# Decide what a workstream is for, or drop it

Type: grilling
Status: resolved
Blocked by: 09

## Question

Does the workstream tier (`work/workstreams/<name>/`) still have a job? Or does the layout drop it, so that maps, one-off issues, the wiki and plans sit directly under `work/`?

## Context

- **The origin.** [The chief-of-staff report](../assets/chief-of-staff-report.md), option C, proposed one chief and one goal wiki for each goal. Long-play was the example: three maps in a row serve one goal and copy the same standing rules into their Notes. The folder was there to hold that goal, its wiki and its chief.
- **The drift (owner, 2026-11-11).** [Decide what a wiki holds and what it never holds](05-wiki-contents.md) made one wiki for the repo. [Decide the layout of the folder](06-layout.md) let a workstream stay open forever, so it became a subject. [Decide which maps go where](07-maps-to-expeditions.md) gave no workstream a done condition, and said "the end of a goal is the Destination of its map". The owner: "I feel like we kind of drifted into it, rather than having a clear destination in mind."
- **The skill (`mattpocock/skills@4ae2317`, still the head of `main` on 2026-11-11).** `skills/in-progress/chief-of-staff/SKILL.md` has one chief session for each goal ("Pursue a long-running goal in a single session"). It says "Look for a shared wiki first … and write your pages into it". It defines no folder for a goal, and it does not use the word "workstream". Its other records go to the issue tracker (`to-spec`, `to-tickets`).

## Answer

Grilled with the owner on 2026-11-11, in five rounds. The workstream tier goes.

```
work/
├── wiki/                  the one wiki
├── plans/                 plan mode writes here
├── wayfinder-maps/<map>/  map.md  issues/  assets/
├── issues/                one-off issues, in status folders
│   ├── pending/
│   ├── ready/
│   ├── in-progress/
│   └── closed/
└── _closed/               the 19 closed maps of today, unchanged
```

1. **No workstreams.** The split rested on a misreading of the skill: need 4 ("let a chief own one folder"). In the skill, a chief is a session that starts with a goal, and its memory is a shared wiki. Several chiefs can run at once, each with its own goal, and share the one wiki with no folder of their own. In Pocock's posts, a "domain" agent has its own wiki, but the posts do not say what a domain is (a repo, a course cohort). So his practice does not decide this. Need 4 now reads: "A chief finds its goal and its memory in the one wiki, and its records from the tracker doc."
2. **A goal is not a map.** A chief's goal is long-running and has no end (PR #1369: "unbounded"), and many maps and issues serve it. So the sentence "The end of a goal is the Destination of its map" in [Decide which maps go where](07-maps-to-expeditions.md) is withdrawn. A map's Destination is the end of one piece of work.
3. **Where a chief's goal lives is not decided here.** The prompt alone (the skill) or a dated wiki page (PR #1369) are the candidates. No layout choice depends on it. It joins the fog item "The hand-off to the later chief-of-staff map".
4. **One-off issues sit in four status folders**: `pending/`, `ready/`, `in-progress/` and `closed/`. The folder is the status. A one-off issue has no `Status:` line and no other status detail: no "waiting on info", no "for a human", no close reason. The owner chose folders over one flat folder because hundreds of numbered files are hard to follow, and status is what a reader looks for.
5. **Numbers.** A one-off issue is `NN-<slug>.md`. Its number is unique across all of `work/issues/`, and it stays with the issue when it moves, so `work/issues/*/NN-*` finds it. A new issue takes the next free number.
6. **A change of status is a move.** A small script moves the file and rewrites the links to it. The gate's link check ([Decide the link check, the broken links of today and the old path](09-link-check-and-old-path.md)) catches a move that skipped the script.
7. **The triage roles map to folders.** `docs/agents/triage-labels.md` maps `needs-triage` and `needs-info` to `pending/`, `ready-for-agent` and `ready-for-human` to `ready/`, and `wontfix` to `closed/`. A finished issue also goes to `closed/`. Work that has started is in `in-progress/`.
8. **At the move.** The 23 one-off issues are renumbered from 01. The 16 `needs-triage` issues and the 1 `needs-info` issue of `state-machines-work` and `autoplay-follow-ups` go to `pending/`. The 1 `closed` issue goes to `closed/`. The four plans and the `story-scroll-back` idea go to `pending/`. The old `Status:` line of each is removed. The path table records each old and new path.

Rejected:

- **Workstreams as subject folders.** They had no job left after one wiki, and each new item needed a choice of folder.
- **One chief for each workstream.** A chief does not need a folder, so the folder is not needed for several chiefs.
- **A workstream only where one goal spans several maps.** Two shapes, as the PARA option that [Decide the layout of the folder](06-layout.md) rejected.
- **One flat `issues/` folder**, with a `Status:` line or a generated status index. Hard to follow at hundreds of issues.
- **Subfolders by origin or subject.** It brings the grouping back.
- **The seven folders of the five triage values plus `in-progress/` and `done/`.** The owner chose four.
- **A body line for the lost detail** (`Closed: wontfix — <why>` and the like).

Facts for later tickets:

- **The build.** The workstream folders, the six `workstream.md` files and the cross-workstream link rewrites of [Decide which maps go where](07-maps-to-expeditions.md) go away. Maps stay siblings, so most links between maps change only in their prefix. 23 links in 13 files point at the 18 issues of `state-machines-work` and `autoplay-follow-ups` today. No installed skill writes a triage status. Only `AGENTS.md`, `docs/agents/issue-tracker.md` and `docs/agents/triage-labels.md` name the triage values.
- **The scripts.** The move needs one more script, which moves one issue to another status folder and rewrites the links to it. The move script can share its link rewrite.
