# Prune worktrees safely

Type: task
Status: resolved
Blocked by: 02

## Question

Nothing safely prunes worktrees. A careless prune can delete a live agent's tree.

## Work

Add a prune script that enumerates worktrees with `git worktree list` and refuses to
remove a tree while any of these hold:

- the worktree is dirty (`git status --porcelain` is not empty);
- it has commits not pushed to its upstream;
- a live process holds the tree (its cwd is inside it).

There is no fixed home convention. `git worktree list` finds every tree, so the script
does not need one. Report each refusal with its reason.

The worktree-create script (F10) was dropped in
[Decide which mechanisms to build](02-decide-mechanisms-to-build.md): the varied homes
(`/workspace/wt63`, `/workspace/ce-wt`) cause no measured harm, and the prune does not
depend on where a tree lives.

## Done when

- The script exists and refuses all three cases.
- Each refusal has a test or a documented manual check.
- The script never removes a tree it did not first inspect.

## Answer

Built [`scripts/remove_worktrees.py`](../../../scripts/remove_worktrees.py), tested by
[`scripts/tests/test_remove_worktrees.py`](../../../scripts/tests/test_remove_worktrees.py)
(14 tests, part of the `py-tests` gate step).

- **Dry run by default.** It prints `safe`/`refuse <reason>` per tree. `--apply` removes the
  safe ones with plain `git worktree remove` (never `--force`, so git's own dirty check is a
  second guard). Optional `PATH` args limit which trees are considered.
- **Refuses all three cases from the ticket,** each with its own test: dirty tree (untracked
  files count), unpushed commits, and a live process whose cwd is inside the tree.
- **"Unpushed", decided here:** with an upstream, `@{u}..HEAD` must be empty. Without one
  (every current `wf/*` branch), `HEAD` must have no commits that a remote-tracking ref
  can't reach. This is stricter than the ticket's wording, since none of the live trees have
  an upstream.
- **Also refused:** the main worktree, locked trees, and trees whose directory is missing
  (it points at `git worktree prune` instead).
- **Fail-closed process check:** it reads `/proc/<pid>/cwd`. With no `/proc`, every tree is
  refused. Process entries it can't read (other users) are skipped. That is a known gap on a
  multi-user host, but it doesn't apply in this container.
- **Never removes an uninspected tree:** each tree is inspected right before its own
  removal. Branches are kept; only the directory goes.

Live check on 2026-10-05: the dry run refused all four real trees: main for being main, and
`wt63`, `wt65`, `wt67` for being dirty.

**Follow-up (same day, user decision):** renamed from `prune_worktrees.py` to
`remove_worktrees.py`. "Prune" clashed with `git worktree prune`, which only clears records
of trees that are already gone. Wired into `build.py` as the `remove-worktrees` step, and the
full gate now runs it with `--apply` as step 2, right after hook install. Like hook install,
it is best-effort: a failed removal warns and the build continues. The tree the gate runs in
is always refused, because the build's own cwd is inside it. A removed tree takes its ignored
files with it, including its own warm `target/`. Full gate on 2026-10-05: green, and step 2
refused all four real trees.

**Age rule (same day, user decision):** a tree younger than `--min-age-hours` (default 24) is
refused as `too new`. Age counts from the newer mtime of the tree's `HEAD` and `gitdir` admin
files, which are written at creation and checkout. This covers a fresh, clean tree that an
agent edits by path while its process runs from another directory, which the cwd check can't
see. The admin directory and `index` are deliberately not used. The gate's own `git status`
bumped `.git/worktrees/wt63/` to 18:39:10 on 2026-10-05, so either one would stop a tree
from ever ageing out. 18 tests; `py-tests` 273 passed.
