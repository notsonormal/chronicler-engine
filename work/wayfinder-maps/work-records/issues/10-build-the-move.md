# Build the move to `work/`

Type: task
Status: resolved
Blocked by: 09, 11

## Question

Write the move script and run it: move the records to the layout of ticket 06 and the placement of ticket 07, rewrite the links, edit the docs, the skills and the config of ticket 08, and pass the link check of ticket 09 and the full `python build.py`. End with `/simple-after-plan`.

## Context

- **Layout and placement.** [Decide the layout of the folder](06-layout.md) and [Decide which maps go where](07-maps-to-expeditions.md). Ticket 07, "Facts for later tickets", lists the link counts, the files beside `map.md`, the proposed numbers and the rule to stop on a folder that its table does not name.
- **Docs, skills and config.** [Decide where the names are written and how skills find the folder](08-names-in-docs-and-skills.md), "Facts for later tickets", lists the files. Ticket 06 lists the files that name `docs/plans`.
- **No workstreams, and status folders for one-off issues.** [Decide what a workstream is for, or drop it](11-workstream-purpose.md) replaced the layout of ticket 06 and the workstreams of ticket 07. Read its answer first: the layout tree, the renumbering, the issue-move script, and which parts of tickets 06 to 09 still hold.
- **Link check and old path.** [Decide the link check, the broken links of today and the old path](09-link-check-and-old-path.md): write `scripts/validate_work_records.py` and wire it into the doc step of `build.py`; fix the 17 broken links in live maps and tickets; run the strict before-and-after link check in the move script; write the path table `path-moves.md`; remove `.scratch` with no symlink.
- **The script (map Notes).** The move is a script, so it can run again with other names or be reversed.
- **Concurrency (map Notes).** The move runs only when no other session runs. The owner decides the scope of each commit.

## Answer

Built on 2026-11-12. All work records are in `work/`. `.scratch/`, `docs/plans/` and the repo-root `plans` symlink are gone. The full `python build.py` passed (`logs/build_20261111_192443_3588644.log`). Nothing is committed.

- **The plan.** [The build plan](../assets/build-plan.md) was reviewed against the code with `/improve-ai-plan` before the build. The owner decided five points in that review. Those decisions are in its "Decisions" section: the run went ahead, the old status text of the four plans and the idea is kept as a `**Before the move:**` line, backticked `.scratch/` paths are rewritten outside `_closed/`, the dead `plans` exemptions are removed, and one-off issue assets go in the `assets/` folder of `work/issues/`.
- **The layout.** `work/wiki/index.md` (empty index), `work/wayfinder-maps/` (17 maps, each with a `Status:` line: 4 closed, 13 open), `work/issues/pending/` (22 issues, numbered 01 to 23) and `work/issues/closed/` (issue 04), `work/_closed/` (19 maps). `work/plans/`, `work/issues/ready/`, `work/issues/in-progress/` and `work/issues/assets/` are empty and hold a `.gitkeep` file.
- **The links.** Before the move, 7,598 links resolved. After it, 7,593 resolve, and no link that resolved before broke. The difference is the links into git-ignored `tmp/`, which became backticked text, and the 5 links from the two dropped files (`autoplay-follow-ups/map.md`, `state-machines-work/README.md`). The path table is [path-moves.md](../assets/path-moves.md).
- **The scripts.** `scripts/work_records.py` (shared link code), `scripts/validate_work_records.py` (the gate step `validate-work-records`), `scripts/move_issue.py` (a change of status), and the one-off [move script](../assets/move_to_work.py) with `--dry-run` and `--reverse`.
- **The docs and config.** `docs/agents/issue-tracker.md` (names, layout tree, status table, old-path line), `docs/agents/triage-labels.md` (roles to folders), `AGENTS.md`, `docs/AGENTS.md`, six repo skills, `.pi/plan-mode.json`, `arch-lint.toml`, `.gitignore`, `.vale.ini`, `validate_docs.py`, `vale_lint.py`, and the DOC-anchor guardrail (a DOC anchor may not point into `work/`).

Departures from the plan:

- **Ticket 09 was wrong about one link.** The `fable-audit` ticket 02 link to `invariant-audit` points at the live map, not a closed one.
- **More `_closed/` links were rewritten.** 17 links in 16 files changed, not 3: 14 of them pointed into `docs/plans/`. No backticked text in `_closed/` changed.
- **The gate check is its own build step**, `validate-work-records`, not a part of `validate-docs`. Each gate check in `build.py` is one script in one step.
- **Empty folders are kept with `.gitkeep`.** The plan and decision 5 left `plans/`, `ready/`, `in-progress/` and `assets/` to be created on first use. The tracker doc names `work/plans/` and `work/issues/assets/`, so the gate check fails on a fresh clone without them. Added after the document review.
- **`move_issue.py` also rewrites the links of the moved issue.** The plan said that they need no rewrite. That was wrong: a link to an issue in the old folder breaks after the move.
- **The two sentences of the `state-machines-work` README were not carried over.** Ticket 07 meant them for a workstream, which ticket 11 dropped. Git history keeps them.

Facts for later:

- **Backup.** The untracked files of `.scratch/` (700 MB, every `local-only/` folder included) are copied to `tmp/work-records-backup/`. Delete the backup once the move is committed.
- **Old paths in prose.** Live records still name `.scratch/` paths as history: placeholders, deleted plans and the dissolved `state-machines-work` folder. The path table and the old-path line of the tracker doc explain them.
