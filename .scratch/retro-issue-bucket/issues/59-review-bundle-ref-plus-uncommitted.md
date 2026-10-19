# Let the review bundle diff a ref against the working tree

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

At step 12 of the plus-review workflow, the fixes of steps 4 to 10 are not committed. `prepare_review_bundle.py --ref <ref>` makes a three-dot diff against `HEAD`, so it leaves those fixes out, and `--uncommitted` diffs only against `HEAD`. This run built two bundles (`tmp/review/8036bbc3-ref` and `tmp/review/8036bbc3-uncommitted-fixes`) and told each reviewer to read both as one change. How should the bundle cover a fixed point plus the work in the tree?

- **A `--ref <ref> --worktree` mode**: diff `<ref>` against the working tree, including untracked files.
- **Keep two bundles**, and make the workflow say so in step 12.
- **Commit the fixes first**, then build one `--ref` bundle (conflicts with the rule that commits need the owner's approval).

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- `scripts/prepare_review_bundle.py`: `return f"{ref}...HEAD" if mode == "ref" else "HEAD"`.
- Related: [Let the review bundle take a path list](32-review-bundle-path-scope.md).
