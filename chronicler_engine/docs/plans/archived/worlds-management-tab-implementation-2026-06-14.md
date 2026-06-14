# Implementation Plan: Worlds Management Tab UI

## Overview
Implements a dedicated "Worlds" tab in the dashboard for full CRUD operations on worlds. Builds on Plan 1 (data foundation) which added WorldCard model and storage operations. Adds UI + server handlers for create, read, update, delete worlds with proper validation and game reference checking.

## Architecture Decisions
- **Fragment-based UI**: Follow existing pattern from `games_fragment` and `settings_fragment` - HTMX fragments replace content in-place
- **Vertical slice**: Each CRUD operation is complete (storage → service → handler → UI) before next
- **Delete protection**: Cannot delete worlds with referencing games (checked in storage layer, validated in handler)
- **Persona dropdown**: Fetch personas for `player_key` selection in create/edit form

## Task List

### Phase 1: Foundation
- [ ] Task 1: Add Operation::DeleteWorld to storage/backend/core.rs
- [ ] Task 2: Implement Storage::delete_world() in storage/backend/worlds.rs
- [ ] Task 3: Add ApplicationService world CRUD methods (list_worlds, create_world, update_world, delete_world)
- [ ] Task 4: Add ApplicationService::list_personas() method

### Checkpoint: Foundation
- [ ] cargo clippy passes on changed files
- [ ] cargo nextest run storage + application tests pass

### Phase 2: Server Layer - worlds_fragment Module
- [ ] Task 5: Create src/server/worlds_fragment/mod.rs
- [ ] Task 6: Create src/server/worlds_fragment/fragments.rs (renderers: render_worlds_panel, render_world_edit_form, render_error)
- [ ] Task 7: Create src/server/worlds_fragment/handlers.rs (list_worlds_fragment, create_world_handler, edit_world_handler, update_world_handler, delete_world_handler, get_persona_list)
- [ ] Task 8: Add unit tests for fragment renderers
- [ ] Task 9: Add integration tests for handlers

### Checkpoint: Server Layer
- [ ] cargo fmt + clippy passes
- [ ] All new unit tests pass
- [ ] Integration tests compile

### Phase 3: Router + UI Integration
- [ ] Task 10: Add worlds_fragment module to router.rs imports
- [ ] Task 11: Register worlds routes in build_router() (list, create, edit, update, delete, persona list)
- [ ] Task 12: Add "Worlds" tab button to assets/index.html (after "Prompt Presets" tab)
- [ ] Task 13: Add worlds-tab content panel to assets/index.html with hx-get to /fragment/worlds
- [ ] Task 14: Add modal HTML for create/edit world form (hidden by default, shown via JS)

### Checkpoint: Integration
- [ ] Server compiles without errors
- [ ] Worlds tab appears in UI
- [ ] Tab navigation works (switches to worlds-tab panel)

### Phase 4: End-to-End Verification
- [ ] Task 15: Start server, verify Worlds tab renders with existing worlds (redmist_estate, test)
- [ ] Task 16: Test create world via UI with valid data
- [ ] Task 17: Test edit world updates correctly
- [ ] Task 18: Test delete blocked when games reference world
- [ ] Task 19: Test delete succeeds when no games reference world
- [ ] Task 20: Test persona dropdown populates with available personas
- [ ] Task 21: Run python build.py for full validation

### Checkpoint: Complete
- [ ] All acceptance criteria met
- [ ] Manual verification complete
- [ ] build.py passes

### Phase 5: Cleanup & Documentation
- [ ] Task 22: Update docs/architecture/system.md with worlds tab architecture
- [ ] Task 23: Create docs/system/worlds.md documenting the worlds management system
- [ ] Task 24: Move plan to docs/plans/archived/
- [ ] Task 25: Update CHANGELOG.md with "Worlds Management Tab UI" entry

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Missing Operation::DeleteWorld breaks delete | High | Add to Operation enum first, compile will fail if missed |
| Persona list empty | Medium | Check storage::list_personas() returns data, verify PlayerCard model |
| HTMX fragment doesn't refresh | Medium | Use hx-swap="outerHTML" or return ok_refresh() |
| Modal JS conflicts with existing tab JS | Low | Use unique modal IDs, check for conflicts in index.html script |
| Delete cascade misses references | High | Check games table with WHERE world_key = ? before delete |

## Open Questions
None - plan is fully specified.

## Files to Touch
- `src/storage/backend/core.rs` - Operation enum
- `src/storage/backend/worlds.rs` - delete_world()
- `src/application/application_service.rs` - world CRUD + list_personas
- `src/server/worlds_fragment/mod.rs` (new)
- `src/server/worlds_fragment/fragments.rs` (new)
- `src/server/worlds_fragment/handlers.rs` (new)
- `src/server/worlds_fragment/fragments_tests.rs` (new)
- `src/server/worlds_fragment/handlers_tests.rs` (new)
- `src/server/router.rs` - imports + routes
- `assets/index.html` - tab button + panel + modal
- `docs/architecture/system.md` - architecture update
- `docs/system/worlds.md` (new) - domain documentation
- `docs/CHANGELOG.md` - changelog entry

## Estimated Total Scope
~12 files modified/created, 6 test files added. Large implementation requiring careful coordination between layers.
