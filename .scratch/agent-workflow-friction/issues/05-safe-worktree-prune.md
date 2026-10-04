# Prune worktrees safely

Type: task
Status: open
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
