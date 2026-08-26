# Fix minimal_app_no_game to use the canonical helper

Type: task
Status: resolved
Blocked by: —

## Question

In `src/application/games/view_query_tests.rs`, the `minimal_app_no_game` helper manually constructs `MessageService`, `AppSettings`, and calls `ActionPipeline::with_backends` directly. This duplicates the canonical `make_test_pipeline_with_backends` helper from `test_support::context.rs` — the exact duplication the original plan was meant to eliminate. It violates `unit_test_standards.md` Pattern 5.

Rewrite `minimal_app_no_game` to build its pipeline via `make_test_pipeline_with_backends(Arc::clone(&storage), narrator_recorder, AgentRegistry::default())`, keeping the same `storage` (which is pre-seeded with the world + snapshot). Verify the 9 `GameViewQuery` tests still pass.

## Answer

Rewrote `minimal_app_no_game` to build the pipeline via `make_test_pipeline_with_backends(Arc::clone(&storage), narrator_recorder, AgentRegistry::default())`, removing the duplicated `MessageService` and `AppSettings` construction and the direct `ActionPipeline::with_backends` call. The storage remains pre-seeded with the world and snapshot as before. `cargo test --lib view_query` passes (11 tests).
