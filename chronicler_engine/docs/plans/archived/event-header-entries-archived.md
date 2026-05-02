# Spec: Event Header Entries

## Objective

When a chronicler engine trigger fires, add a special header entry to the story log with the event's name (e.g., "Gabriella Introduction"). This entry should behave similarly to location headers (e.g., "Entrance Hall") — appearing as a prominent inline header — but be visually distinct so players can clearly distinguish narrative events from room changes.

**User story:** As a player, when I encounter Gabriella for the first time and her trigger fires, I want to see a clear "Gabriella Introduction" event banner in the log, separate from the room location, so I can recognize important story moments at a glance.

**Success criteria:**
- Triggers can optionally define a `name` field
- When a named trigger fires, an event header entry appears in the story log before the trigger's narration
- Event headers use a distinct visual style from location headers
- Event headers do not show edit/retry buttons
- Existing triggers without names continue to work unchanged (backward compatible)
- All tests pass (`python build.py`)

## Tech Stack

- Rust (Edition 2024, Rust 1.85+)
- Chronicler Engine existing stack: Axum, Askama, HTMX, pulldown-cmark
- Test framework: built-in `cargo test` + browser-based integration tests

## Commands

```bash
# Validate (format + clippy + tests + coverage)
cd chronicler_engine && python build.py

# Run unit tests only
cargo test

# Run specific test
cargo test test_event_header_ -- --nocapture
```

## Project Structure

Files that will be modified:

```
src/model/trigger.rs          # Add `name` to TriggerAction
src/model/state.rs            # Add `LogType::Event` variant
src/engine/action_processing.rs  # Add event header before trigger narration
src/server/templates.rs       # Add `is_event` to LogEntryView, update template
assets/styles.css             # Add `.event-header` / `.event-timestamp` styles
data/worlds/*/characters/*.json  # Add names to existing triggers
docs/system/triggers.md       # Update trigger documentation
tests/                        # Add/update integration tests
```

## Code Style

Follow existing Rust conventions from `AGENTS.md`:
- Result over panic, propagate with `?`
- Doc anchors for complex blocks: `// [DOC: docs/system/triggers.md]`
- No "What" comments — rename symbols instead
- Import order: std → external → local

Example pattern for adding log entries:
```rust
state.add_log(
    String::new(),
    Some(event_name.to_string()),
    LogType::Event,
);
```

## Testing Strategy

- **Unit tests** in `src/model/trigger.rs` for serde of new `name` field
- **Unit tests** in `src/engine/action_processing.rs` for event header insertion
- **Unit tests** in `src/server/templates.rs` for `LogEntryView::is_event` detection
- **Integration tests** in `tests/` verifying event headers render with correct CSS class
- **Backward compatibility** tests: triggers without `name` still work

Coverage expectation: New code paths should be covered by at least one test each.

## Boundaries

- **Always:** Run `python build.py` before marking complete; update docs before code per spec-first policy
- **Ask first:** Changing the color palette (we'll pick a sensible default); adding new trigger conditions
- **Never:** Break existing world data (all JSON changes must be backward compatible); commit without tests passing

## Open Questions

1. **Visual color**: Location headers use green (`#4ade80`). Should event headers use amber/orange, purple, or another color?
2. **Event naming**: Should the event name be on `Trigger` or `TriggerAction`? `TriggerAction` makes sense since the action produces the event entry.
3. **Fallback behavior**: If a trigger has no name, it behaves exactly as before — no event header, just narration. Is this correct?

---

## Implementation Plan

### Phase 1: Data Model Changes
1. Add `pub name: Option<String>` to `TriggerAction` in `src/model/trigger.rs`
2. Add `Event` variant to `LogType` in `src/model/state.rs`
3. Update serde tests in `trigger.rs` for new field
4. Update `LogEntryView::from` in `src/server/templates.rs` to detect `is_event`

### Phase 2: Engine Logic
1. In `evaluate_and_narrate_triggers`, before adding trigger narration:
   - If `trigger.action.name` is `Some(name)`, add event header entry
   - Then add the LLM continuation text as before
2. Add unit test verifying event header is inserted when name is present
3. Add unit test verifying no event header when name is absent

### Phase 3: UI & Styling
1. Update `LogEntryView` to include `pub is_event: bool`
2. Update Askama `StoryLogTemplate` to render event entries with:
   - `<span class="event-header">{{ entry.sender }}</span>`
   - `<span class="event-timestamp">- {{ entry.timestamp }}</span>`
   - No edit/retry buttons (same as location)
3. Add CSS classes in `assets/styles.css`:
   - `.event-header` — bold, distinct color (suggest amber `#fbbf24`)
   - `.event-timestamp` — muted, same pattern as location timestamp
4. Add template unit test for event entry rendering

### Phase 4: Data & Documentation
1. Update existing world data files to add `name` to triggers:
   - `gabriella.json`: `"name": "Gabriella Introduction"`
   - `shopkeeper.json`: `"name": "Shopkeeper Greeting"`
   - `ranger.json`: `"name": "Ranger Warning"`
2. Update `docs/system/triggers.md` to document the `name` field
3. Add CHANGELOG entry

### Phase 5: Integration Tests
1. Add integration test verifying event header renders in browser
2. Verify backward compatibility: unnamed triggers still work
3. Run full `python build.py` validation

## Verification Checkpoints

- [ ] `cargo test` passes (all unit tests)
- [ ] `python build.py` passes (fmt + clippy + tests + coverage)
- [ ] Event headers render with distinct color from location headers
- [ ] Event headers show no edit/retry buttons
- [ ] Triggers without `name` behave identically to before
- [ ] World JSON files load without errors
