# Spec: Message Action Buttons Redesign

## Objective
Redesign the action buttons on message boxes in the Chronicler Engine dashboard to improve visibility, positioning, and functionality.

**User stories:**
- As a player, I want message action buttons at the top-right of each message so they are easy to find and consistent with modern chat UIs (like SillyTavern)
- As a player, I want buttons to be visually distinctive so I can quickly identify available actions
- As a player, I want to delete individual messages from the history

**Success criteria:**
- [x] Edit, check, retry, and delete buttons appear at the top-right of each message bubble
- [x] Buttons are visible (not hidden/low-opacity) and have distinctive hover states
- [x] Delete button removes the message from history and refreshes the story log
- [x] All existing tests pass
- [x] New tests cover delete functionality

## Tech Stack
- Rust (backend): Axum, Askama templates
- Frontend: HTMX, vanilla JS, CSS custom properties
- Testing: headless Chrome e2e tests, Rust unit tests

## Commands
```bash
# Full validation
python build.py

# Run e2e tests only
cargo test --test e2e_tests

# Run component tests
cargo test --test component_tests
```

## Project Structure (relevant files)
```
chronicler_engine/
├── src/
│   ├── model/state.rs           # GameState, LogEntry, delete_log method
│   ├── server/mod.rs            # Router, route definitions
│   ├── server/fragments.rs      # Handlers (edit_history_handler, delete_history_handler)
│   ├── server/templates.rs      # StoryLogTemplate (HTML generation)
│   └── server/templates_tests.rs # Template unit tests
├── assets/
│   ├── index.html               # Main page, JS functions
│   └── styles.css               # Message bubble and button styles
└── tests/
    └── e2e_tests.rs             # Browser automation tests
```

## Implementation Plan

### Phase 1: Backend - Delete Functionality
1. Add `delete_log(id: u64)` method to `GameState` in `src/model/state.rs`
2. Add `delete_history_handler` in `src/server/fragments.rs`
3. Add route `POST /history/:id/delete` in `src/server/mod.rs`

### Phase 2: Template - Restructure HTML
Modify `StoryLogTemplate` in `src/server/templates.rs`:
- Wrap each log entry's header (timestamp, sender) in a flex container
- Add action buttons container at top-right
- Buttons: Edit (all), Check (input only), Delete (all), Retry (last AI only)
- Remove old inline button placement after `.text`

### Phase 3: CSS - Visible Distinctive Buttons
Update `assets/styles.css`:
- New `.message-header` flex container with space-between
- New `.message-actions` container for top-right buttons
- Button styles with visible icons, hover states, and distinct colors
- Delete button styled in red/danger color

### Phase 4: JavaScript - Delete Handler
Update `assets/index.html`:
- Add `deleteMessage(id)` function with confirmation dialog
- POST to `/history/:id/delete`, then refresh story log

### Phase 5: Tests
- Update e2e tests for new button selectors and positions
- Add template tests for delete button rendering
- Add component test for delete endpoint

## Code Style
- Match existing Rust conventions: `snake_case`, Result propagation with `?`
- CSS: Use existing design tokens (`--color-*` variables)
- HTML: Keep Askama template syntax minimal

## Testing Strategy
- **Unit tests**: Template rendering tests in `templates_tests.rs`
- **Component tests**: Handler tests in `component_tests.rs` or `fragments_tests.rs`
- **e2e tests**: Browser tests verify button visibility, positioning, and functionality

## Boundaries
- **Always**: Run `python build.py` before considering done
- **Ask first**: Changes to log entry data model beyond simple delete
- **Never**: Break existing edit/retry functionality

## Open Questions
1. Should delete have a confirmation dialog? (Assuming: yes, via JS confirm())
2. Should the delete button appear on all entry types or exclude location/event headers? (Assuming: exclude location/event, same as edit)
