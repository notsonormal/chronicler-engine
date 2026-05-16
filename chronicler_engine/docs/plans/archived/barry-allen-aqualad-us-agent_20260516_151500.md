# Plan: Remove Meaningless Sync Actions from UI and Prompts

## Objective
Remove the "meaningless" sync actions (Look, Inventory) and directional exit hints (North/South/etc) from the bottom-left action hints UI. Remove inventory references from the system prompt and LLM game state context.

## Background

Current state:
- Action hints at bottom-left show `[Look] [Inventory] [North] [South] ...` via `/hints` endpoint (`render_action_hints()`)
- `Action::Look` and `Action::Inventory` bypass LLM generation (sync actions) and output canned responses
- `ActionAreaTemplate` also builds `available_actions` with Look/Inventory/exits, but this field is unused in the HTML template
- System prompt mentions "inventory" as part of state validation rules
- Game state layer sent to LLM includes a `--- Inventory ---` block
- Directional movement (`process_directional_movement`) is already dead code in production (only called from tests) — movement is handled via FreeAction → LLM pipeline

## Two Approaches

### Option A: UI & Prompt Only (Minimal)
Only remove visual hints and prompt references. Keep backend enum/parser/handlers so typed commands still work.

**Changes:**
1. `src/server/fragments/renderers.rs` — `render_action_hints()` returns empty string (no Look/Inventory/exit hints)
2. `src/server/templates.rs` — `ActionAreaTemplate::new()` sets `available_actions: vec![]` (field is unused in template but clean it up)
3. `src/narrative/prompt/templates.rs` — Remove "inventory" from line 7 state validation rules
4. `src/narrative/prompt/builder.rs` — Remove the entire `--- Inventory ---` block from `render_game_state_layer()`
5. Update affected tests:
   - `tests/browser/structure.rs::test_action_hints_visible` — verify hints container still exists but may be empty
   - `tests/browser/interaction.rs::test_form_submission` — change "look" to "quit" or a free action
   - `tests/browser/editing.rs` — change "look" to "quit" in 3 places
   - `tests/flow_mock/sequence.rs::test_sync_look_then_async_freeaction_then_retry` — rewrite to use `Action::Quit` as the sync action
   - `src/narrative/prompt/builder_tests.rs` — update tests that assert inventory presence in prompts

**Trade-offs:**
- ✅ Minimal code churn, lower risk
- ✅ Fast to implement and validate
- ❌ User can still type "look"/"inventory" and get canned responses (hidden feature)
- ❌ Dead code remains in enum/parser/handlers

### Option B: Full Engine Removal (Comprehensive)
Remove `Action::Look` and `Action::Inventory` entirely from the engine. Typing "look" or "inventory" will route through the LLM as `FreeAction` like any other input.

**Changes:**
Everything in Option A, plus:
1. `src/engine/action.rs` — Remove `Look` and `Inventory` variants from `Action` enum
2. `src/engine/parser.rs` — Remove `"l" | "look"` and `"i" | "inv" | "inventory"` match arms (they fall through to `FreeAction`)
3. `src/engine/game_service/actions.rs` — Remove `Action::Look` and `Action::Inventory` match arms
4. `src/server/fragments/actions.rs` — Update `is_sync` to only match `Action::Quit`; remove Look/Inventory arms from `process_sync_action()`
5. `src/engine/logic.rs` — Remove dead `get_available_exits()` and `process_directional_movement()` (only called from tests); keep `find_room_in_map`, `find_room_in_world_map`, `get_current_room`
6. `src/server/fragments/renderers.rs` — Remove `get_available_exits` import
7. Update tests:
   - All tests from Option A
   - `src/engine/parser_tests.rs` — Remove `test_parse_look`, `test_parse_inventory`; update `test_parse_mixed_case_commands` to not assert Inventory
   - `src/engine/logic_tests.rs` — Remove tests for `get_available_exits` and `process_directional_movement`
   - `tests/logic_tests.rs` — Remove directional movement tests

**Trade-offs:**
- ✅ Complete removal, no hidden features
- ✅ Cleaner architecture — no special-case sync actions for look/inventory
- ✅ Typing "look" now gets a real LLM narration instead of a canned room description
- ❌ Larger change, touches ~10 files
- ❌ More tests to update/adjust

## Recommendation

**Option B (Recommended).**

The user explicitly calls these "meaningless sync actions." If they are meaningless, they should not exist as first-class engine constructs. Removing them entirely means:
- `look` becomes a `FreeAction` that the LLM narrates organically (which is the desired UX)
- `inventory` becomes a `FreeAction` the LLM handles via the game state context
- The sync action path only handles `Quit`, which is the only action that genuinely needs instant client-side handling

The risk is manageable — the changes are mechanical and tests will catch any mistakes during validation.

## Verification Plan

After implementation:
1. `cd chronicler_engine && cargo test` — all tests pass
2. `cd chronicler_engine && python build.py` — full validation passes (fmt + clippy + tests + coverage)
3. Manual UI check: action hints area at bottom-left should be empty or absent
4. Manual prompt check: verify system prompt and game state XML contain no inventory references

## Files to Modify

| File | Change |
|------|--------|
| `src/server/fragments/renderers.rs` | Empty `render_action_hints()`, remove `get_available_exits` import |
| `src/server/templates.rs` | Empty `available_actions` in `ActionAreaTemplate::new()` |
| `src/narrative/prompt/templates.rs` | Remove "inventory" from state validation line |
| `src/narrative/prompt/builder.rs` | Remove `--- Inventory ---` block from `render_game_state_layer()` |
| `src/engine/action.rs` | Remove `Look`, `Inventory` variants (Option B only) |
| `src/engine/parser.rs` | Remove look/inventory match arms (Option B only) |
| `src/engine/game_service/actions.rs` | Remove Look/Inventory handlers (Option B only) |
| `src/server/fragments/actions.rs` | Update `is_sync` and `process_sync_action()` (Option B only) |
| `src/engine/logic.rs` | Remove `get_available_exits()` and `process_directional_movement()` (Option B only) |
| `tests/browser/structure.rs` | Update `test_action_hints_visible` |
| `tests/browser/interaction.rs` | Change "look" to "quit" in form submission test |
| `tests/browser/editing.rs` | Change "look" to "quit" in 3 places |
| `tests/flow_mock/sequence.rs` | Rewrite sync look test to use Quit |
| `src/engine/parser_tests.rs` | Remove look/inventory parser tests (Option B only) |
| `src/engine/logic_tests.rs` | Remove exit/directional tests (Option B only) |
| `tests/logic_tests.rs` | Remove directional movement tests (Option B only) |
| `src/narrative/prompt/builder_tests.rs` | Update prompt assertions for inventory removal |
