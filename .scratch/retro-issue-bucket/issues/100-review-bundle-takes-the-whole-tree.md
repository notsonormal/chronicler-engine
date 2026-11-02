# The review bundle takes the whole tree, also the files of another session

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`/code-review` builds its bundle with `scripts/prepare_review_bundle.py --uncommitted`. When another session edits the same checkout, the bundle also holds that session's files. How does a session review only its own files?

## Context

Source: `/retro` 2026-11-02, the session of layout-trials ticket 11 (`.scratch/layout-trials/issues/11-session-per-agent.md`).

- **known** At review time the tree held my 3 files and 8 files of a ticket-12 session (`data/characters/…`, `data/worlds/redmist_estate/…`, `data/personas/julian.json`, `.scratch/layout-trials/issues/12-…`).
- **known** `prepare_review_bundle.py --help`: `--uncommitted | --ref REF`, `--worktree`, `--out DIR`. It has no path filter.
- **known** I built the bundle by hand: `git diff HEAD -- <3 paths> > tmp/review/lt11/change.patch`. The README of the script and its per-area patches were then missing.
- **known** `comment_finder.py` already has `--files`, so `/simple-after-plan` step 1 can name a file scope there but not for the review.

## Options to decide

- **A.** `prepare_review_bundle.py` takes a pathspec (`-- <paths>`), passed to every `git diff` and to the untracked list.
- **B.** A line in `.agents/skills/code-review/SKILL.md`: with a shared tree, build the bundle by hand with `git diff HEAD -- <paths>`.
- **C.** Nothing: commit first, then review with `--ref`.
