# Show how to slice one file out of a bundle patch

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Three reviewers in this run each wrote `awk '/^diff --git a\/X/,/^diff --git a\/Y/'` by hand to read one file from `src.patch`, in seven calls. The bundle README says to page smallest-first and that `stat.txt` is the index. Should the bundle offer the slice?

- **One command in the bundle README**.
- **A `scripts/patch_slice.py <bundle> <path>`** helper.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Reviewer `161683c2` entries 69, 73, 77, 93, 95; reviewer `20247c8f` entry 93.
