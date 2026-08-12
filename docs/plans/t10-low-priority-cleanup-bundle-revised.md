# Plan: Low-priority cleanup bundle (refresh)

**Date:** 2026-08-12  
**Status:** Planning  
**Goal:** Pick off a small set of independent, low-risk cleanups. Each item is self-contained and should take <1 hour.

## Items

### A11: `MessageEntry` DTO mirroring
- **File:** `src/domain/model/state/message_types.rs`.
- **Current state:** `MessageEntry` duplicates fields of `Message`.
- **Decision:** either collapse into `Message` where possible, or add `impl From<&Message> for MessageEntry` and remove manual field copies.
- **Verification:** `cargo build --tests` and `python build.py` pass.

### M3: `response_length: Option<&str>` is stringly typed
- **Files:** `src/application/prompting/assembler.rs`, `src/application/prompting/builders/sections.rs`.
- **Current state:** `response_length` is passed as `Option<&str>` and matched against magic strings like `"short"`/`"medium"`/`"long"`.
- **Decision:** replace with a small enum (`ResponseLength { Short, Medium, Long }`) or a token-count struct. Keep the wire format unchanged.
- **Verification:** all prompt-layer tests pass; integration tests for response-length rendering pass.

### M4: `QuantifierParseResult::is_high()` only
- **File:** `src/domain/model/quantifier.rs`.
- **Current state:** `QuantifierParseResult` only exposes `is_high()`; the wrapped `QuantifierConfidence` already has `is_low`/`is_medium`/`is_high`.
- **Decision:** add `is_low()` and `is_medium()` to `QuantifierParseResult`, or delete `is_high()` and make callers use `result.confidence.is_low()` etc.
- **Verification:** callers in `phases.rs`, `game_state_action_processing_tests.rs`, `invariant_contract.rs` compile and tests pass.

## Out of scope
- `empty_to_none` (already removed).
- `push_section` (replaced by `Section` enum + `render_preset_xml_parts`).
- `trigger_eval.rs` cohesion (`evaluate_triggers` moved to `domain/model/state/game_state.rs`).
- `list_personas` passthrough (now consistent with `list_worlds`).
- `from_row` consistency (all storage models now have it).
- `Ok(_) => unreachable!()` arms (none remain in `src/`).

## Picking rule
Choose 1–2 items per sprint. Do not bundle structurally unrelated items.
