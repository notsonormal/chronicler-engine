# Say how to find the fixed point of a review for a multi-branch wayfinder effort

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The after-plan workflow uses a `<fixed-point>` for the review bundle (step 12) and scopes the gate's duplicate report to `main` (step 7). For the Rust rebuild, P0 to P5 had already merged to `main`, so neither `main` nor the merge-base was the right start. The user had to name it: "find the first point before we added code changes from the rust-rebuild". The lead found `bc7d6cd4` from the merge graph. The gate's duplicate step still covered only the core step, and a second run with `--ref bc7d6cd4` was needed. Where does the fixed point come from?

- **The map or the plan records it**: a wayfinder execution map writes its base commit in its Notes when the first slice starts, and the workflow reads it from there.
- **The workflow says how to derive it**: the parent of the first slice commit of the effort (`git log --first-parent`), and it passes the same ref to `healthcheck.py duplicates --ref`.
- **Leave it to the user** and say so in step 12.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Lead session `01a15614`, first user message; `git log --graph` shows `bc7d6cd4` as the parent of P0 `977ba3f7`.
- `build.py` registry entry `duplicates`: "scoped to files changed vs main".
