---
diataxis: reference
title: Code Comments
---

## File headers

Every file in `src/` (excluding `*_tests.rs` unit tests and `src/test_support/*.rs`) carries:

- Line 1: `//! [DOC: docs/diataxis/reference/<area>/<name>.md]` (links to reference documentation; `reference/` only — no `explanation/`, `how-to/`, or `tutorials/` targets). `src/test_support/*.rs` files must NOT carry a DOC anchor — the structure guardrail exempts them.
- Line 2: `//! Human-readable summary` (used for auto-generating the source structure index; required for `src/test_support/*.rs` too).

Function-level anchors are removed.

## What comments are for

Never write comments that paraphrase what the code does. If the code isn't clear, rename the symbols rather than comment. Comments explain the WHY only when non-obvious: a hidden constraint, behavior that would surprise a reader.

Never reference the task in code comments ("used by X flow", "added for issue Y", "TODO from review"). Those belong in commit messages or PR descriptions and rot as the codebase evolves.
