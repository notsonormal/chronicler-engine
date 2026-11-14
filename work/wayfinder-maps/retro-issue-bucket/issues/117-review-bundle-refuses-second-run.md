# A second review bundle of the same commit and mode fails on its default path

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`prepare_review_bundle.py` refuses to clear a directory it did not create, and a finished run deletes its own marker. Must a second run pick a free path?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass).

- **known** The second `--uncommitted` run of the session answered "refusing to clear a directory this script did not create"; the lead re-ran it with `--out`.
- **known** By design: a successful run deletes its marker (`scripts/prepare_review_bundle.py:265`), so every later run on the same commit and mode refuses.

## Options to decide

- **A.** When the default path exists, use a free one with a numeric suffix and print it.
- **B.** Keep the marker after a successful run, so a rerun may clear it.
- **C.** Nothing.
