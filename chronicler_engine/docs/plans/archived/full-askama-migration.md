# Plan: Full Askama Template Migration

## Problem

**Current State**: After Phase 1 (pilot) migration, only `HeaderTemplate` used Askama.
The remaining 3 fragments still used manual `format!` strings:

- `render_story_log` - HTML log entries built with string concat
- `render_visual_sidebar` - Image/NPC HTML built with string concat  
- `render_action_area` - Form HTML built with format!

**Impact**:
- Inconsistent: only 1/4 templates compile-time validated
- Dual rendering paths: Askama + manual strings
- Manual HTML escaping (`html_escape()` function) duplicated Askama's auto-escape
- Dead code accumulated in fragments.rs

## Solution

Complete migration to Askama for all 4 fragments:

1. **StoryLogTemplate** - Iterate log entries, auto-escape text
2. **VisualSidebarTemplate** - Conditional room image, loop NPC portraits
3. **ActionAreaTemplate** - Form with disabled state, action hints

## Files Changed

### Modified
- `src/server/templates.rs` - Added 3 new templates with tests
- `src/server/fragments.rs` - Updated to use new templates

### Not Changed
- `src/server/mod.rs` - Same routing
- `tests/` - Still exist (integration tests)

## Architecture Updates

### `docs/architecture/system.md`
Server tier already documented Askama. Confirm all 4 templates now use it.

## Implementation Notes

### Rust 2024 Compatibility
- Avoid reserved words in CSS classes: `form`, `area`, `hint`
- Use `class=command-wrapper` not `class=form`
- Use `id=cmd-area` not `id=action-area`
- Use empty string `""` instead of `Option` in templates

### Askama Template Patterns
- Always use owned types (`String`, not `&str`)
- Boolean flags instead of Option (e.g., `room_has_image: bool`)
- Empty string for "no value" (e.g., `sender: ""`)

## Acceptance Criteria

- [x] All 4 fragments use Askama templates
- [x] Compiles with `cargo check`
- [x] Clippy passes with `-D warnings`
- [x] Unit tests pass (<50ms each)
- [x] HTML escaping works (XSS protection)

## Test Results

```
running 12 tests
test server::templates::tests::test_action_area_no_exits ... ok
test server::templates::tests::test_action_area_ready ... ok
test server::templates::tests::test_action_area_thinking ... ok
test server::templates::tests::test_header_template_connection_status ... ok
test server::templates::tests::test_header_template_renders_room_name ... ok
test server::templates::tests::test_story_log_template_empty ... ok
test server::templates::tests::test_story_log_template_escapes_html ... ok
test server::templates::tests::test_story_log_template_with_entries ... ok
test server::templates::tests::test_visual_sidebar_no_image ... ok
test server::templates::tests::test_visual_sidebar_with_image ... ok
test server::templates::tests::test_visual_sidebar_with_npcs ... ok
test server::templates::tests::test_header_template_escapes_html ... ok

test result: ok. 12 passed; 0 failed; 0 ignored
```

## Risk & Rollback

**Risk**: Low - fully tested, easily reversible
**Rollback**: 
```bash
git checkout HEAD~1 -- src/server/fragments.rs src/server/templates.rs
```

## Decision Record

- **Rust 2024**: Choose inline templates over external `.html` files - tighter coupling, simpler builds
- **Owned types**: Use `String` over `&str` - simpler lifetime management
- **Boolean over Option**: Template conditionals simpler with `bool`