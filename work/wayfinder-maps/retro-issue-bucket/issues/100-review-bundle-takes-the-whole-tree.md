# The review bundle takes the whole tree, also the files of another session

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`/code-review` builds its bundle with `scripts/prepare_review_bundle.py --uncommitted`. When another session edits the same checkout, the bundle also holds that session's files. How does a session review only its own files?

## Context

Source: `/retro` 2026-11-02, the session of layout-trials ticket 11 (`work/wayfinder-maps/layout-trials/issues/11-session-per-agent.md`).

- **known** At review time the tree held my 3 files and 8 files of a ticket-12 session (`data/characters/…`, `data/worlds/redmist_estate/…`, `data/personas/julian.json`, `work/wayfinder-maps/layout-trials/issues/12-…`).
- **known** `prepare_review_bundle.py --help`: `--uncommitted | --ref REF`, `--worktree`, `--out DIR`. It has no path filter.
- **known** I built the bundle by hand: `git diff HEAD -- <3 paths> > tmp/review/lt11/change.patch`. The README of the script and its per-area patches were then missing.
- **known** `comment_finder.py` already has `--files`, so `/simple-after-plan` step 1 can name a file scope there but not for the review.
- **known** Again on 2026-11-09 (statechart-models ticket 14): the bundle held the untracked `work/wayfinder-maps/invariant-audit/`, `.scratch/state-machines-*/` and a `.scratch/weekly-retro-*` edit of other sessions. `tooling.patch` was 830 KB. Both reviewer briefs had to name the paths to ignore.

## Options to decide

- **A.** `prepare_review_bundle.py` takes a pathspec (`-- <paths>`), passed to every `git diff` and to the untracked list.
- **B.** A line in `.agents/skills/code-review/SKILL.md`: with a shared tree, build the bundle by hand with `git diff HEAD -- <paths>`.
- **C.** Nothing: commit first, then review with `--ref`.

## Recurrence (2026-11-02, layout-trials ticket 15)

Source: `/retro` of the coordinator session of [Try the history in the ME format](../../layout-trials/issues/15-history-in-me-format.md).

- **known** The tree held 5 changed and 14 untracked files of `work/wayfinder-maps/character-art-from-code/` from another session. `prepare_review_bundle.py --uncommitted` put them in `tooling.patch` (1174 lines, of which 400 were this ticket's).
- **known** The coordinator wrote `tooling-lt15.patch` with `awk` and told both reviewers to ignore the other files. The comment pass used `comment_finder.py --files` with a hand-built list (`tmp/lt15/code-files.txt`).
- **Won't do check.** D2 lists "a review bundle holds other efforts' files (32)", with the reason "many efforts share one checkout". Option A does not fight the shared checkout. It lets a session review its own files in it. Two hand-built bundles in one day are the new evidence.
