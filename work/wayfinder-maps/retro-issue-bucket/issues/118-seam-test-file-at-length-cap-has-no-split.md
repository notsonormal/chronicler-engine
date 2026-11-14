# A seam-contract test file at 2,000 lines has no allowed split

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

When a `_tests.rs` file reaches the 2,000-line cap, the pairing check rejects every split name. What is the legal split?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass), with the build sessions `a09ca82b` and `48a609ae`.

- **known** `narrator_tests.rs` passed the cap. `narrator_story_memory_tests.rs` failed as "Orphan test file". The agent used the `mod` exemption (`tests/infrastructure/guardrails/location.rs:59`) and created `narration/mod_tests.rs`, which the Standards review called a hard violation of the Test Mirror Convention. The fix round trimmed helpers to 1,983 lines.
- **known** The file has 1,978 non-blank lines on 2026-11-13; the next story memory change reaches the cap again.
- **known** The length message says "Consider splitting into smaller modules" (`tests/infrastructure/guardrails/structure.rs:316-322`) and names no legal shape.

## Options to decide

- **A.** The pairing check accepts `<base>_<topic>_tests.rs` when `<base>.rs` exists and the file is a seam-contract file (`unit_test_allowlist.rs`); the length message names that shape; the `mod` exemption stops covering seam files.
- **B.** A higher cap for seam-contract test files.
- **C.** Nothing: split the source file instead.
