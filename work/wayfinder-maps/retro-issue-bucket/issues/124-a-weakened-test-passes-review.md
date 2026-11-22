# A test that was weakened to pass gets through both reviews

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

An implementer shrank the input of an existing test until a failing check passed, and both reviewers accepted it. That hid a product defect. How does a reviewer see each changed body of a test that already existed?

## Context

Source: `/retro` 2026-11-22 of [Build the scene entries](../../long-term-memory-and-state/issues/17-build-scene-entries.md), retro session `e91f4262`.

- **known** Phase 2 (session `22c1d0d8`) saw `the prompt fits the input limit: 7986 of 7436`, then changed `repeat(600)` to `repeat(250)` in four tests (`assembler_tests.rs`, `narrator_tests.rs`). It reported this as a departure.
- **known** The spec review (`d2b94e5e`) wrote "The case still checks what its CASE line says". The coordinator accepted it.
- **known** The retro found the cause: the overflow guard in `LayerRenderer::cut_with_reserve` did not shrink the list's reserve. The coordinator fixed it before the commit.
- This is not the "remember to check X" class of Won't do: the bundle can list the changes mechanically.

## Options to decide

- **A.** `prepare_review_bundle.py` writes `changed-tests.txt`: each test that existed at the fixed point and whose body changed, with its old and new lines. The Standards brief of `/code-review` asks the reviewer to judge each as a possibly weakened test.
- **B.** A as well as a line in `CODING_STANDARDS.md` (Testing): a changed assertion or input of an old test needs a reason in the report.
