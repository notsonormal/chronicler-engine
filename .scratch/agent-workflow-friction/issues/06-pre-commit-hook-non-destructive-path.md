# Give the pre-commit hook a non-destructive path

Type: task
Status: resolved
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

## Answer

Resolved 2026-10-05 by implementation. The hook now delegates to a new
`scripts/precommit_regenerate.py`, which decides per file from **prose outside the
generated blocks**, not from a whole-file snapshot comparison.

### Why the prescribed compare was replaced

The ticket's snapshot → regenerate → compare (S == fresh → own output; S == index →
clean; else foreign) fails the ticket's own criteria (an independent design review
confirmed both):

- **It sweeps foreign prose.** All four generators replace only their marker-delimited
  region and preserve prose (`generate_structure_index.py:166-169`,
  `generate_docs_index.py:102-112`, `generate_tests_structure_index.py:113-121`,
  `generate_guardrails_doc.py:214-259`). So for a bystander prose edit with an unchanged
  block, regeneration leaves `R == S`, the hook reads that as "own output", stages the
  whole file, and commits the other agent's prose — the exact F12 safety property.
  `docs/AGENTS.md` reaches the same state via its no-write path (`:107-109`).
- **It still false-aborts on leftover own output.** Stale leftover output plus a
  subsequent source change gives `S != R` and `S != I`, so case (C) fires on a file with
  no foreign bytes — the acceptance criterion this ticket exists to fix.

### What was built

- `scripts/precommit_regenerate.py`
  - Pinned marker keys per file (`AUTO-STRUCTURE`, `AUTO-STRUCTURE-TESTS`, `AUTO-INDEX`,
    `AUTO-GUARDRAILS: {clippy,arch-lint,syn}`). A file must carry exactly those pairs, in
    order; anything else aborts, so a deleted or malformed block is never silently
    rewritten.
  - Safe iff the worktree's prose equals the index's prose. Then it regenerates and
    `git add`s. Foreign prose (or a missing/malformed block) prints the file name and
    aborts before anything is written.
  - Any failure after generation starts (generator error or `git add` error) restores all
    four files from an in-memory snapshot, so an abort always leaves the worktree
    byte-identical to how it was found.
  - Prints a note when an uncommitted change lay entirely inside a block
    ("regenerated `<file>`..."), which is the leftover-own-output case.
- `scripts/git-hooks/pre-commit` — thin caller for the helper. A compatibility path for
  worktrees that predate the helper was added and then removed: it existed only for the
  three orphaned dashboard worktrees (`wt63`, `wt65`, `wt67`), which were deleted in the
  same session.
- `scripts/tests/test_precommit_regenerate.py` — 12 tests: prose stripping, marker
  rejection, stale-block staging, foreign-prose abort, in-block regeneration, and
  restore-on-generator-failure, all against a real temp git repo.
- `.agents/skills/commit-and-push/SKILL.md` — the hook-behavior paragraph corrected:
  it aborts on prose *outside* the block, not on any uncommitted change.

### Trade-off recorded

A hand edit *inside* a generated block is indistinguishable from stale own output (no
provenance), so it is treated as generator-owned and regenerated rather than restored.
The report note makes the overwrite visible. All prose traffic — the safety property — is
protected.

### Verification

- `python build.py` full gate green: 1566 integration, 62 browser, 165 guardrail, 1
  architecture; Python tests 285/285.
- Against the live checkout, with another effort's uncommitted `AGENTS.md` prose, the
  hook aborts naming `AGENTS.md` and leaves it untouched.
- Pinned keys confirmed to match all four real files.
- Throwaway temp-repo run exercised the helper path end-to-end, including abort-untouched.
