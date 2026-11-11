# Move parser_tests.rs into utils/

Type: task
Status: resolved
Blocked by: —

## Answer
Moved `src/application/agents/quantifier/parser_tests.rs` to `src/application/agents/quantifier/utils/parser_tests.rs`. Removed `#[cfg(test)] mod parser_tests;` from `src/application/agents/quantifier/mod.rs`; added it to `src/application/agents/quantifier/utils/mod.rs`. Existing imports still resolve via the `pub use utils::parser` re-export in the parent module. `cargo test --lib quantifier` passes: 72 passed.

## Question

`src/application/agents/quantifier/parser_tests.rs` is an orphan: it tests `crate::application::agents::quantifier::parser`, whose source lives at `src/application/agents/quantifier/utils/parser.rs`. The test file sits in the parent directory, so it has no matching `parser.rs` sibling.

Move `parser_tests.rs` to `src/application/agents/quantifier/utils/parser_tests.rs` so it pairs with `utils/parser.rs`. Adjust the `mod` declaration in `src/application/agents/quantifier/utils/mod.rs` (and remove the old declaration from `src/application/agents/quantifier/mod.rs` if present). Verify imports inside the test file still resolve. Run `cargo test --lib` for the quantifier module.
