# Consolidate game_state_*_tests.rs into game_state_tests.rs

Type: task
Status: resolved
Blocked by: —

## Answer

Consolidated `game_state_action_processing_tests.rs`, `game_state_logic_tests.rs`, and `game_state_trigger_eval_tests.rs` into the existing `game_state_tests.rs` (~1,418 lines, under the 2000-line guardrail). Merged imports into a single top-level `use` block, kept all helper names distinct (`make_test_state`, `setup_test_state`, `make_state`), deleted the three orphan files, and removed their `mod` declarations from `src/domain/model/state/mod.rs`. `cargo test --lib game_state` passes (61 tests); `cargo check --all-targets` is clean.

## Question

Three orphan test files all target `src/domain/model/state/game_state.rs` but have no matching source siblings:

- `src/domain/model/state/game_state_action_processing_tests.rs` (626 lines)
- `src/domain/model/state/game_state_logic_tests.rs` (164 lines)
- `src/domain/model/state/game_state_trigger_eval_tests.rs` (361 lines)

Consolidate all three into the existing `src/domain/model/state/game_state_tests.rs` (272 lines; combined total ~1423, under the 2000-line guardrail). Verified: no test-function name collisions across the four files, and helper names differ (`make_test_state`, `setup_test_state`, `make_state`) so merge them with care — rename on collision if any appears after re-reading in full.

Delete the three orphan files and remove their `mod` declarations from `src/domain/model/state/mod.rs`. Run `cargo test --lib` for the state module.
