# Plan: Fix weak assertions in CSS, debug, and browser trigger tests

**Date:** 2026-06-16
**Status:** Implemented

## Context

Three test areas flagged in the test-police review for weak assertions or fragility:
1. **`tests/integration/model/css.rs`** — 2 tests only check CSS string containment (`:root`, `var(--`, `@media`, scrollbar selectors). No validation of CSS syntax, dark mode variables, or responsive breakpoints beyond "string exists somewhere."
2. **`tests/http/debug.rs`** — 1 test, happy path only. Missing: `is_generating` endpoint, `backend` endpoint, error-state debug, and value-type validation on the existing test.
3. **`tests/browser/trigger.rs:80-98`** — `test_second_encounter_does_not_refire` reads `shopkeeper.json` from disk and indexes `triggers[0].narration.narration_prompt` with hard-coded field path. If the JSON structure changes, the test silently fails or panics with a misleading message.

## Approach

### Step 1 — Strengthen CSS integration tests

- **Added `test_css_design_tokens_cover_core_areas`**: Asserts ≥5 of 6 core variable-prefix categories exist
- **Added `test_css_responsive_breakpoints`**: Parses `@media` queries, asserts breakpoint widths in 100–2000px range
- **Strengthened `test_css_valid`**: Added non-trivial length check (>1000 chars) and background variable presence (`--color-bg-` or `--bg`)

### Step 2 — Add missing debug endpoint tests

- **Strengthened `test_debug_state_endpoint_returns_json`**: Added type assertions for `current_room_id`, `npcs_in_area`, `generation_status` (Idle/Generating/Error), `generation_phase` (Narrating/Quantifying/GeneratingEvent)
- **Added `test_debug_state_endpoint_includes_all_documented_fields`**: Asserts all 13 DebugStateResponse fields exist (was 6)
- **Added `test_debug_is_generating_returns_false_by_default`**: Tests `GET /debug/is_generating` returns `"false"`
- **Added `test_debug_is_generating_reflects_state`**: Tests builder with `is_generating(true)` returns `"true"`
- **Added `test_debug_backend_returns_json`**: Tests `GET /debug/backend` returns JSON with string fields

### Step 3 — Harden browser trigger test

- Extracted `load_first_trigger_prompt(character_id)` helper with descriptive panic messages
- Replaced fragile 6-line fixture indexing block with single helper call

## Deviations from plan

- `test_debug_state_endpoint_validates_field_types` merged into existing `test_debug_state_endpoint_returns_json` (less duplication)
- `test_debug_is_generating_returns_string` named `test_debug_is_generating_returns_false_by_default` (more descriptive)
- `u64 >= 0` assertions removed (always true, clippy error); `is_number()` still validates type
- `backend_name`/`model_name` asserted as strings only, not `"Mock"` specifically (less brittle)
