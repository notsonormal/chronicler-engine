# 12 — Refactor AppState render methods to module-per-type

Type: task
Status: resolved
Blocked by: (none)

## Question

Move the stray `impl AppState` block from `adapters/driving/http/layout/renderers/fragment_renderers.rs` so it satisfies `guardrails_inherent_impl_locality`.

Current state:
- `AppState` defined in `adapters/driving/http/app_state.rs`.
- Inherent impl block with render helper methods (`render_header`, `render_story_log`, `render_visual_sidebar`, `render_action_area`, `render_character_headshots`, `render_llm_messages`) lives in `adapters/driving/http/layout/renderers/fragment_renderers.rs`.
- Tests for those methods live in `adapters/driving/http/layout/renderers/fragment_renderers_tests.rs`.

This is a cross-folder split: the impl file is under `layout/renderers/`, whose parent folder is not `app_state`.

Target shape — single-file consolidation (file length stays well under 2000 lines):
```text
adapters/driving/http/app_state.rs  # AppState struct + all inherent impls
adapters/driving/http/app_state_tests.rs  # existing + relocated render tests
```

Constraints:
- `build.py` green at every landed step.
- Preserve all method signatures and imports exactly.
- Delete `fragment_renderers.rs` once emptied and remove its `mod` declaration from `layout/renderers/mod.rs`.
- Relocate `fragment_renderers_tests.rs` to `app_state_tests.rs` (or merge into existing `app_state_tests.rs`) so test-file location stays consistent.
- Do NOT touch trait impls.

Acceptance:
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `AppState` violations.
- Full `cargo test --test guardrails` passes.
- Full `build.py` green.

Links:
- Map: `.scratch/inherent-impl-locality/map.md`
- Rule ticket: `issues/01-build-inherent-impl-locality-rule.md`

## Answer

Consolidated the stray `impl AppState` block back into the type's defining file.

Changes:
- Moved the render helpers (`render_header`, `render_story_log`, `render_visual_sidebar`, `render_action_area`, `render_character_headshots`, `render_llm_messages`) from `src/adapters/driving/http/layout/renderers/fragment_renderers.rs` into `src/adapters/driving/http/app_state.rs` inside the existing `impl AppState` block.
- Added the required imports to `app_state.rs`: `askama::Template`, `crate::error::{EngineError, Result}`, and the template/view-model/builder modules previously used only by the renderer file.
- Moved the two render helper tests from `fragment_renderers_tests.rs` into `app_state_tests.rs`, reordered imports to satisfy `guardrails_import_ordering`, and merged the duplicated `use std::sync::Arc;` import.
- Removed the empty `layout/renderers/` module: deleted `fragment_renderers.rs`, `fragment_renderers_tests.rs`, and `renderers/mod.rs`, and removed `pub mod renderers;` from `layout/mod.rs`.

Result:
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero violations.
- `cargo test --test guardrails` passes (112 tests).
- `build.py` is fully green.

