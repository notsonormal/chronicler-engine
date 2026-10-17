# Never issue a commit in the same step as the edit it commits

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

One assistant message carried the `edit` that appended to ticket 13 and `git commit -qam "docs(scratch): add ticket 09, 11 and 12 follow-ups to ticket 13"`. The commit printed the pre-commit hook's four generator lines and then `On branch phase2-integration / nothing to commit, working tree clean`, exit 1 — while the file was tracked, unignored and 17 lines ahead of HEAD (7 s later `git status --short --ignored` showed ` M …` and `git diff --stat` the 17 insertions). The owner inferred a hook that stashes; it does not. The same session's earlier commit, where the edit and the commit shared one bash command (`sed -i … && git commit -qam …`), committed fine.

Reproduced here: one message carried `edit` on this very file plus a read-only `grep` sibling; the edit reported "Successfully replaced 1 block(s)…" and the sibling still read the old file (`grep -c` → 0). Sibling tool calls in one assistant message can run concurrently — the pi docs say so (`extensions.md` in the `@earendil-works/pi-coding-agent` package: "Tool calls from one assistant message can run in parallel") — so a commit's staging snapshot can predate the edit's write.

How should this be prevented?

- **A commit is its own step.** Guidance and briefs say: never issue a commit in the same assistant message as the edit it commits; confirm the paths you edited appear in `git status --porcelain` before the commit runs.
- **Teach the symptom too.** "Nothing to commit, working tree clean" beside hook output, in a tree `git diff` calls dirty, means the stage preceded the write — re-run the commit as its own step. The hook does not stash; `.scratch` is not ignored.
- **Nothing repo-side.** The existing brief already says "stage with explicit paths, never `git commit -am`". Accepted cost: one wasted commit attempt per occurrence, with a misleading diagnosis.

## Context

- Source: `/retro` 2026-10-11; Manager session `01a12b7d-c321-761a-8192-b094ca97d492` (cwd `/workspace/chronicler-engine/tmp/wt-phase2-integration`), entries 82–90 at 16:57:39.
- Refutation of the recorded inference: `scripts/git-hooks/pre-commit` only runs `scripts/precommit_regenerate.py`, which regenerates the four generated files and `git add --` those four (`_snapshot` / `_restore` / the `add` call); it never stashes, resets, or reads `.scratch`. The hook's output sitting above the abort is git's order — the hook runs before git's emptiness re-check.
- Recovery that worked: `git add .scratch/…/13-follow-up-phase-2-re-review-p3s.md && git commit -qm …` → `5def7292` (17 insertions, 1 file). The earlier same-command edit+commit → `870b8eba`.
- The Manager's list for this run already forbids `-am` in every Lead brief; the failing commit was the coordinator's own, which is why the wording matters more than the flag.
- Related: [Stop `commit-and-push` from staging the whole shared tree](13-commit-staging-shared-tree.md), [Let the review bundle take a path list](32-review-bundle-path-scope.md).
