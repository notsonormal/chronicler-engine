# ADR-003: Askama Template Engine for HTML Rendering

**Date:** 2025-04-12

---

## Context

Initial HTML rendering used manual Rust `format!` strings in `src/server/fragments.rs`:

```rust
fn render_header(room_name: &str) -> String {
    format!(r#"<div class="header">{}</div>"#, room_name)
}
```

Problems:
1. **No compile-time validation** - Missing fields, typos discovered at runtime
2. **Test brittleness** - UI tests required Playwright browser, slow and fragile
3. **No type connection** - Gap between Rust types and HTML templates

---

## Decision

**Adopt Askama template engine for compile-time validated HTML rendering.**

### Why Askama

| Feature | Maud | Tera | Askama |
|---------|------|------|-------|
| Error messages | Good | Varying | Best |
| File separation | `.html` files | Jinja-like | `.html` files or inline |
| HTMX pattern | Good | Good | Best |
| Type safety | None | None | Compile-time |

### Implementation

All 4 fragments migrated to Askama:

1. **HeaderTemplate** - Game title, location, connection status
2. **StoryLogTemplate** - Narration history with auto-escaping
3. **VisualSidebarTemplate** - Room image + NPC portraits
4. **ActionAreaTemplate** - Command input form

### Rust 2024 Patterns

```rust
#[derive(Template)]
#[template(source = r#"
<div class="header">
    <span class="location">{{ room_name }}</span>
    <span class="timestamp">{{ timestamp }}</span>
</div>
"#)]
pub struct HeaderTemplate {
    pub room_name: String,
    pub timestamp: String,
}
```

Using:
- Owned `String` types (not `&str`)
- Boolean flags instead of `Option`
- Empty string `""` for "no value"

---

## Consequences

### Positive
- **Compile-time validation** - Missing fields = compiler error
- **Fast unit tests** - No HTTP server needed (milliseconds vs seconds)
- **Type bridging** - Templates declare data shapes
- **HTML escaping** - Automatic XSS protection

### Negative
- Learn template syntax
- Inline templates require Rust escaping
- More dependencies

### Trade-offs
- Best error messages over file separation
- Owned types simplify lifetimes

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard Architecture](./adr-001-htmx-web-dashboard.md) - Uses templates

---

## History

- **2025-04-12**: Phase 1 - Pilot with HeaderTemplate
- **Later**: Full migration - all 4 fragments

---

## Historical Note

This was a two-phase migration: first a pilot with HeaderTemplate, then full migration to all 4 fragments.