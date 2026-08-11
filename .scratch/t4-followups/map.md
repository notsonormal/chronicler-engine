# wayfinder:map — T4 Followups: post-PhaseError cleanup

Source: surfaced by ticket 04's post-plan-workflow analysis (subagent 2 — coverage gap + code-quality pass). 3 sharp items found by closing the coverage gap: the Cancelled-coverage work exposed misleading names and tests that were passing for the wrong reason.

## Destination

`phase_trigger_continuation_with_cancel_handling` named honestly (or actually checks the cancel token); two misleading-cancellation tests either deleted or rewritten to drive real cancellation; dead `None` arm at `retry.rs:79-82` removed or asserted. Build green, retry.rs coverage stays ≥ 80%.

## Notes

- **Domain:** Chronicler Engine, Rust 2024 edition (Rust 1.85+).
- **Build validation:** `python build.py`.
- **AFK task ticket.** Single 2 SP task → `general-purpose` subagent, primary agent verifies.
- **Origin:** items surfaced by closing ticket 04's coverage gap (subagent 2 report). Not in original T4 scope; the tests were latent bad-documentation tests before the coverage pass exposed them.
- **Touch only:** `phases.rs` (rename or semantically fix `phase_trigger_continuation_with_cancel_handling`) + `retry_tests.rs` (rewrite or delete 2 tests) + `retry.rs` (dead branch removal). No `run_from_input`, no `ActionPipeline` public surface changes.
- **Standards:** No `// What` comments. Preserve all existing test contracts (happy-path coverage must not drop).

## Decisions so far

*(none yet — map freshly charted)*

## Not yet specified

*(none — all 3 items are sharp enough to ticket in one task ticket)*

## Out of scope

- Anything beyond the 3 surfaced items. General cancellation-mechanism redesign (proper `cancel_token` propagation through all phases) is a separate, larger effort — not in scope here.
