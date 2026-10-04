# Give the pre-commit hook a non-destructive path

Type: task
Status: open
Blocked by: 02

## Question

`scripts/git-hooks/pre-commit` aborts when one of its four generated files is dirty
versus the index. It cannot tell its own leftover regeneration from a foreign edit, so a
prior failed run makes the next run abort and blame the operator. Sessions answered with
`--no-verify`, which skips regeneration and lets a stale index land.

The guard runs before regeneration (`pre-commit:22-29`) and asks only "is the worktree
dirty versus the index?" It records no provenance, and the four generators write in place.

## Work

Snapshot the four generated files, regenerate, then compare per file:

- snapshot equals the fresh regeneration: the dirtiness was the hook's own output. Stage
  it. Do not abort.
- snapshot equals the index content (`git show :<path>`): the worktree was clean. Stage
  the fresh regeneration.
- neither: a foreign edit exists that regeneration would overwrite. Restore the snapshot,
  abort, and name the file.

The abort path must leave the worktree exactly as the hook found it.

## Done when

- The false abort (leftover own output) no longer fires.
- A genuine foreign edit is restored and reported, never overwritten.
- A clean or self-generated file is still regenerated and staged on success.
