# Move context_tests.rs into utils/

Type: task
Status: resolved
Blocked by: —

## Question

`src/application/prompting/context_tests.rs` is an orphan: it tests `crate::application::prompting::utils::context::{fit_messages_to_context, trim_history_to_budget}`, whose source lives at `src/application/prompting/utils/context.rs`. The test file sits in the parent directory, so it has no matching `context.rs` sibling.

Move `context_tests.rs` to `src/application/prompting/utils/context_tests.rs` so it pairs with `utils/context.rs`. Adjust the `mod` declaration in `src/application/prompting/utils/mod.rs` (and remove the old declaration from `src/application/prompting/mod.rs` if present). Verify imports inside the test file still resolve. Run `cargo test --lib` for the prompting module.

## Answer

Moved `src/application/prompting/context_tests.rs` → `src/application/prompting/utils/context_tests.rs`; added `#[cfg(test)] mod context_tests;` to `src/application/prompting/utils/mod.rs`; removed the same declaration from `src/application/prompting/mod.rs`. Test imports resolved unchanged. `cargo test --lib prompting`: 54 passed.
