# Give each branch of a design rule a test before the review passes

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Target design §3.2 says: "When a step fails, the generations after it do not run. The Options run still runs when a Narration was saved." C3 built `Job::run` so that a failed **event** generation, after the main Narration was saved, skipped the Options run. The C3 two-axis code review, the C5 reviews and the C6 reviews did not find it. The C7 doc review found it, because the rewritten `game_flow.md` copied the rule and the reviewer checked the sentence against the code path. The fix was one line plus scenario `options.md` 24.22.

The rule had a test for the failed main generation, not for the failed event generation. Where should the branch check live?

- **The spec-axis review brief.** For each design rule the diff implements, the reviewer lists its branches (here: which generation failed) and the test of each branch. A branch with no test is a finding.
- **The implementer brief.** The brief names the design rules of the step, and the implementer writes one scenario per branch before the code.
- **Leave it.** The doc review is the net, and it worked.

## Context

- Source: `/retro` 2026-10-19 on the core step (C1 to C7) of the Rust rebuild, ticket `.scratch/rust-rebuild-build/issues/07-core-step.md`, sections "C7" and "C7 fix pass".
- Evidence: commit `4eeadba2` changed `src/application/pipeline/job.rs` `Job::run` (`ended.narration_saved |= saved_a_narration`); the test `tests/http/options.rs::test_options_run_follows_a_failed_event_narration_http` failed before the fix with `left: []`.
