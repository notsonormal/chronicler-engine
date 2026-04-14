# Plan: Template Migration to Askama

## Problem

**Current State**: HTML fragments are hand-built using Rust `format!` strings in `src/server/fragments.rs`. This creates two issues:

1. **No compile-time validation** - Missing fields, typos in variable names, type mismatches only discovered at runtime
2. **Test brittleness** - UI tests require spawning HTTP server, launching Playwright browser, making DOM queries - tests are slow (seconds) and fragile (one CSS change = cascade failures)

**Impact**:
- Developers don't trust tests (they catch the wrong things)
- Tests take too long (nobody runs them during development)
- Gap between Rust types and HTML templates (no connection)

## Solution

**Migrate HTML fragment rendering from manual strings to Askama** - a compile-time template engine that:
- Validates templates at compile time (missing fields = compiler error)
- Tests render without HTTP server (pure unit tests, milliseconds)
- Bridges the type gap (templates declare required data shapes)

**Why Askama over Maud/Tera**:
- Best error messages (tells you exactly what's wrong)
- Keeps separate `.html` files (familiar workflow)
- Works perfectly with HTMX fragment pattern

## Scope (Pilot)

**Phase 1** (this plan): Migrate one template as proof of concept
- Convert `render_header` to Askama template
- Write unit test that renders without server

**Phase 2** (future): Migrate remaining fragments
- `story_log`, `visual_sidebar`, `action_area`

## Files to Change

### New Files
- `src/server/templates.rs` - Askama template definitions
- `tests/template_tests.rs` - Fast unit tests

### Modified Files
- `Cargo.toml` - Add `askama` dependency
- `src/server/fragments.rs` - Use Askama template instead of `format!`

### Not Changed (Phase 1)
- `src/server/mod.rs` - Same routing, just different render impl
- `tests/ui_tests.rs` - Keep as integration tests (will coexist)
- `tests/layout_tests.rs` - Keep as integration tests

## Architecture Updates

### Updated: `docs/architecture/system.md`
Add Server Tier section change:
```
- `fragments`: HTML fragment generators for HTMX partial updates.
  - Uses `pulldown-cmark` for markdown→HTML conversion (in progress).
  - Uses `askama` for compile-time validated templates (NEW).
```

### New: Template Module (`src/server/templates.rs`)
```rust
use askama::Template;

/// Header template with compile-time field validation
#[derive(Template)]
#[template(source = r#"
<div class="header">
    <span class="game-title">Chronicler Engine</span>
    <span class="location">| {{ room_name }}</span>
    <span class="connection-status connected" id="connection-status">Connected</span>
</div>
"#)]
pub struct HeaderTemplate {
    pub room_name: String,
}
```

## Implementation Steps

1. **Add Askama to Cargo.toml**
   ```toml
   askama = "0.12"
   ```

2. **Create `src/server/templates.rs`** with `HeaderTemplate`

3. **Update `src/server/fragments.rs`** to use `HeaderTemplate`

4. **Create `tests/template_tests.rs`** with unit tests

5. **Run validation**: `cargo fmt`, `cargo clippy`, `cargo test`

## Test Structure

**Before** (current):
```rust
#[tokio::test]
async fn test_header_renders() {
    let _server = TestServer::new_with_mock(PORT, WORLD);
    // ...spawn browser, make HTTP request, parse DOM...
}
```

**After** (Askama):
```rust
#[test]
fn test_header_template_room_name() {
    let t = HeaderTemplate { room_name: "Test Room".into() };
    assert!(t.render().unwrap().contains("Test Room"));
}
```

## Acceptance Criteria

- [ ] `HeaderTemplate` renders correctly
- [ ] Unit test runs without HTTP server (<50ms vs current ~5s)
- [ ] Missing fields produce compile errors
- [ ] Migration doesn't break existing tests
- [ ] `cargo fmt`, `cargo clippy`, `cargo test` pass

## Risk & Rollback

**Risk**: Low - pilot scope, easily reversible
**Rollback**: Revert to manual strings in `fragments.rs`, remove `askama` from Cargo.toml

## Decision Record

- **Chose Askama over Maud**: Better error messages, familiar Jinja-like syntax
- **Chose Askama over Tera**: Compile-time validation addresses our core pain
- **No runtime change**: Response format unchanged, handlers identical