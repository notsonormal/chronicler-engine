---
name: tdd
description: Test-driven development. Use when the user wants to build features or fix bugs test-first, mentions "red-green-refactor", or wants integration tests.
---

# Test-Driven Development

TDD is the red → green loop. This skill is the reference that makes that loop produce tests worth keeping: what a good test is, where tests go, the anti-patterns, and the rules of the loop. Every section applies on every cycle — consult them before and during the loop, not after.

When exploring the codebase, read `CONTEXT.md` so test names and interface vocabulary match the project's domain language, and respect ADRs in the area you're touching.

## What a good test is

Tests verify behavior through public interfaces, not implementation details. Code can change entirely; tests shouldn't. A good test reads like a specification — "user can checkout with valid cart" tells you exactly what capability exists — and survives refactors because it doesn't care about internal structure.

See [tests.md](tests.md) for examples and [mocking.md](mocking.md) for mocking guidelines.

## Seams — where tests go

A **seam** is the public boundary you test at: the interface where you observe behavior without reaching inside. Tests live at seams, never against internals.

**Test only at pre-agreed seams.** Before writing any test, write down the seams under test and confirm them with the user. No test is written at an unconfirmed seam. You can't test everything — agreeing the seams up front is how testing effort lands on the critical paths and complex logic instead of every edge case.

Ask: "What's the public interface, and which seams should we test?"

When the shape of that interface is itself in question — how deep the module is, where the seam belongs, what the interface should expose — the `codebase-design` skill is the shared vocabulary for it. It is a reference to consult, not a session to run.

## Anti-patterns

- **Implementation-coupled** — mocks internal collaborators, tests private methods, or verifies through a side channel (querying the database instead of using the interface). The tell: the test breaks when you refactor but behavior hasn't changed.
- **Tautological** — the assertion recomputes the expected value the way the code does (`expect(add(a, b)).toBe(a + b)`, a snapshot derived by hand the same way, a constant asserted equal to itself), so it passes by construction and can never disagree with the code. Expected values must come from an independent source of truth — a known-good literal, a worked example, the spec.
- **Horizontal slicing** — writing all tests first, then all implementation. Bulk tests verify _imagined_ behavior: you test the _shape_ of things rather than user-facing behavior, the tests go insensitive to real changes, and you commit to test structure before understanding the implementation. Work in **vertical slices** instead — one test → one implementation → repeat, each test a **tracer bullet** that responds to what the last cycle taught you.

## Rules of the loop

- **Red before green.** Write the failing test first, then only enough code to pass it. Don't anticipate future tests or add speculative features.
- **One slice at a time.** One seam, one test, one minimal implementation per cycle.
- **Refactoring is not part of the loop.** It belongs to the review stage (see the `code-review` skill), not the red → green implementation cycle.

## Where tests live here

- Integration tests go in `tests/`, one file per area (`tests/http/`, `tests/browser/`, `tests/storage/`, …). A `*_tests.rs` must be declared in a sibling `mod.rs` or the parent `<stem>.rs`, and an inline `mod tests` is rejected — `scripts/check_test_structure.py` runs in the gate, so this structure is enforced, not advisory.
- Read [tests/STRATEGY.md](../../../tests/STRATEGY.md) before choosing a seam. It says which tier a test belongs in and why.
- When a test fails, [tests/AGENTS.md](../../../tests/AGENTS.md) carries the failure-handling protocol — never rationalise a failure away.
- For repo-specific conventions — the test inventory, wait helpers, what belongs in the browser tier — see the `test-police` skill.

## A note on tests that cannot fail

The tautological anti-pattern above is the one this repo keeps catching in review: fixtures where the assertion holds both before and after the change, stubs that echo the input so the output assertion passes either way, `assert!` on a value that is already true. A test that cannot fail is worse than no test, because it buys false confidence.

The countermeasure, when a test looks suspicious: **break the behaviour, watch the test fail, then revert**. If it still passes, the test proves nothing.
