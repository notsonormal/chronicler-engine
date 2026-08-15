# wayfinder:map — T4 Followups: post-PhaseError cleanup

Source: surfaced by ticket 04's post-plan-workflow analysis (subagent 2 — coverage gap + code-quality pass). 3 sharp items found by closing the coverage gap: the Cancelled-coverage work exposed misleading names and tests that were passing for the wrong reason.

## Destination

The remaining two sharp items from ticket 04's coverage gap are resolved: the misleading cancellation test in `retry_tests.rs` either drives a real `Err(PhaseError::Cancelled)` or is explicitly deleted with rationale; the dead `None` arm in `retry.rs` is removed or asserted. Build green, `retry.rs` coverage stays ≥ 80%.

## Notes

- **Domain:** Chronicler Engine, Rust 2024 edition (Rust 1.85+).
- **Build validation:** `python build.py`.
- **AFK task ticket.** Single 2 SP task → `general-purpose` subagent, primary agent verifies.
- **Origin:** items surfaced by closing ticket 04's coverage gap (subagent 2 report). Not in original T4 scope; the tests were latent bad-documentation tests before the coverage pass exposed them.
- **Touch only:** `src/application/pipeline/action_pipeline/retry_tests.rs` (rewrite/delete misleading cancellation test) + `src/application/pipeline/action_pipeline/retry.rs` (dead `None` arm removal). Optional `///` doc comment in `src/application/pipeline/pipeline_run.rs`. No `run_from_input`, no `ActionPipeline` public surface changes.
- **Standards:** No `// What` comments. Preserve all existing test contracts (happy-path coverage must not drop).

## Decisions so far

- [Ticket 01 — Cleanup misleading cancel-test plumbing + dead arm surfaced by ticket 04 coverage pass](issues/01-cleanup-misleading-cancel-tests-and-dead-arm.md) — `phase_trigger_continuation_with_cancel_handling` was renamed to `phase_trigger_continuation_llm_call` during the `031cf9b` pipeline refactor; the misleading-name concern is resolved. The misleading cancellation test and the dead `None` arm remain and are tracked in the same ticket.

## Not yet specified

*(none — the remaining work is tracked in ticket 01)*

## Out of scope

- Anything beyond the 3 surfaced items. General cancellation-mechanism redesign (proper `cancel_token` propagation through all phases) is a separate, larger effort — not in scope here.
