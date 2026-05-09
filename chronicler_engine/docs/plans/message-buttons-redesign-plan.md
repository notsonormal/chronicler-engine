# Implementation Plan: Message Action Buttons Redesign

## Overview
Move edit, spellcheck, and retry buttons from the bottom-left to the top-right of message bubbles. Add a delete message button. Make all buttons more visible and distinctive, inspired by SillyTavern's message action bar design.

## Architecture Decisions
- **Stable IDs for deletion**: Log entries already have stable `id: u64` fields, so deleting from the middle of `narration_history` is safe (no index-based references)
- **POST for delete**: Using `POST /history/:id/delete` (not DELETE method) for HTMX compatibility and consistency with existing edit endpoint (`POST /history/:id`)
- **Top-right placement**: Following SillyTavern's `.mes_buttons` pattern — buttons live in a flex container at the top-right of the message header, always visible
- **Button visibility**: Always-visible with subtle background pills and distinct hover colors per action type

## Task List

### Phase 1: Foundation — Backend Delete Support
- [ ] **Task 1** → `@fixer`: Add `delete_log()` method to `GameState`
  - Add to `src/model/state.rs`
  - Remove entry by ID, return `Result<()>`
  - Files: `src/model/state.rs`
  - Estimated scope: Small
  
- [ ] **Task 2** → `@fixer`: Add delete history endpoint
  - Add `delete_history_handler` to `src/server/fragments.rs`
  - Add route `POST /history/:id/delete` to `src/server/mod.rs`
  - Return empty 200 for HTMX to trigger client-side refresh
  - Files: `src/server/fragments.rs`, `src/server/mod.rs`
  - Estimated scope: Small

**Checkpoint 1**: Backend compiles, existing tests pass
- `cargo check` passes
- `cargo test` passes

### Phase 2: Frontend — Template, CSS, JS
- [ ] **Task 3** → `@fixer`: Restructure `StoryLogTemplate` HTML
  - Create `.message-header` flex container with sender/timestamp on left
  - Create `.message-actions` container on the right
  - Move edit, check, retry buttons into `.message-actions`
  - Add delete button to `.message-actions`
  - Remove old inline button placement after `.text`
  - Files: `src/server/templates.rs`
  - Estimated scope: Medium

- [ ] **Task 4** → `@designer`: Add CSS for new message header and visible buttons
  - `.message-header`: flex, justify-content: space-between, align-items: center
  - `.message-actions`: flex row, gap between buttons
  - `.action-btn`: visible button style with subtle background, border, icon
  - `.action-btn.delete`: distinct styling
  - `.action-btn:hover`: distinct color changes per button type
  - Update existing button styles to use new structure
  - Files: `assets/styles.css`
  - Estimated scope: Medium

- [ ] **Task 5** → `@fixer`: Add `deleteMessage()` JavaScript handler
  - Add to `assets/index.html`
  - Confirm dialog, then POST to `/history/:id/delete`
  - On success, trigger `htmx:refresh` on `#story-log`
  - Files: `assets/index.html`
  - Estimated scope: Small

**Checkpoint 2**: Frontend renders correctly
- `cargo test` passes
- Visual inspection: buttons appear at top-right of messages

### Phase 3: Tests
- [ ] **Task 6** → `@fixer`: Update template unit tests
  - Update `templates_tests.rs` for new HTML structure
  - Add test: delete button appears on entries
  - Add test: action buttons are in `.message-actions` container
  - Files: `src/server/templates_tests.rs`
  - Estimated scope: Small

- [ ] **Task 7** → `@fixer`: Update e2e tests
  - Update selectors: `.edit-btn` → `.message-actions .edit-btn` or similar
  - Update `test_edit_button_exists_on_entries`
  - Update `test_edit_mode_activates_on_click`
  - Update `test_edit_cancel_restores_original`
  - Update `test_retry_button_on_last_ai_message`
  - Add new test: `test_delete_button_exists`
  - Add new test: `test_delete_removes_message`
  - Files: `tests/e2e_tests.rs`
  - Estimated scope: Medium

- [ ] **Task 8** → `@fixer`: Add component test for delete endpoint
  - Test `POST /history/:id/delete` returns 200 for valid ID
  - Test returns 404 for non-existent ID
  - Test message is actually removed from history
  - Files: `tests/component_tests.rs` or `src/server/fragments_tests.rs`
  - Estimated scope: Small

**Checkpoint 3**: All tests pass
- `python build.py` passes (fmt + clippy + tests)

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| HTMX polling refreshes during delete | Med | The delete handler returns quickly; HTMX refresh will show updated state |
| Button repositioning breaks e2e selectors | High | All e2e tests that query `.edit-btn`, `.retry-btn` need selector updates |
| Delete breaks retry logic (which relies on history indices) | High | Retry uses `get_last_ai_response_index()` which scans by type, not index — safe |

## Open Questions (Resolved)
- ✅ Buttons always visible (not hover-only)
- ✅ Delete button on ALL message types (including location/event/system)
- ✅ Normal (not red) trash can icon for delete
