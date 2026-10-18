# The core step (C1 to C7)

Type: task
Status: open
Blocked by: 06

## Question

Build §9 of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) on branch `wf/rebuild-core`, from the current `main`. Read §1, §2 and §9 of the plan, and §8 (P5 wrote the target tests). The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §9 pass. The owner has approved the last commit, and the branch has merged to `main`.

## Comments

**2026-10-18, lead (handoff from P5).** (1) When C5 removes an `#[ignore]` from a target test, check that the test turns green for the right reason: change its expectation once, see it fail, and change it back. (2) The Job runner keeps the test-only Job-task tracker (`GenerationGate::watch_job`, `wait_for_jobs`) at its spawn site, in `job.rs` or `test_support/`, not in the new `gate.rs` (a second lock there is a rule-18 re-look trigger). (3) Run the target tests with `cargo nextest run --no-fail-fast --test http --run-ignored all -E 'test(races::)'`. (4) Ticket 06 lists the gaps of the stop matrix.
