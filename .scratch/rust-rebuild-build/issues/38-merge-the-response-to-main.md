# Merge the response to `main`

Type: task (AFK)
Status: open
Blocked by: 37

## Question

Do §13 of [the build plan of the response](../../../docs/plans/rust-rebuild-response-build-plan.md): the last checks, `git merge --no-ff wf/rebuild-core` into `main`, the full gate on `main`, and the removal of `wf/rebuild-review` and its worktree `../chronicler-engine-rebuild-review` (ticket 26 Q6). This reaches the map's Destination.

## Done when

`main` holds `wf/rebuild-core`, the full gate on `main` is green, `wf/rebuild-review` and its worktree are gone, and the map records the Destination as reached.
