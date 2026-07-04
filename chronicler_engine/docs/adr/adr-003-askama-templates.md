# ADR-003: Askama Template Engine for HTML Rendering

**Date:** 2025-04-12
**Status:** Accepted

---

## Context

Initial HTML rendering used manual Rust `format!` strings in `src/server/fragments.rs`.

Problems:
1. **No compile-time validation** — Missing fields and typos were discovered at runtime
2. **Test brittleness** — UI tests required Playwright browser automation (slow, fragile)
3. **No type connection** — Gap between Rust types and HTML output

---

## Decision

**Adopt Askama template engine for compile-time validated HTML rendering.**

### Why Askama over alternatives

| Feature | Maud | Tera | Askama |
|---------|------|------|--------|
| Error messages | Good | Varying | Best |
| File separation | `.html` files | Jinja-like | `.html` files or inline |
| HTMX pattern | Good | Good | Best |
| Type safety | None | None | Compile-time |

Askama was chosen for its compile-time type checking (missing fields = compiler error) and the best-in-class error messages. This directly addressed the Playwright dependency: with Askama, template correctness is verified at compile time, so unit tests don't need a browser.

### Implementation approach

Owned `String` types (not `&str`) to sidestep lifetime complexity. Boolean flags instead of `Option`. Empty string `""` for absent values.

Full implementation details: [`docs/system/dashboard.md`](../system/dashboard.md)

---

## Consequences

### Positive
- Compile-time validation — missing fields are caught by the compiler
- Unit tests run in milliseconds without a browser
- Automatic XSS escaping

### Negative
- Inline templates require Rust string escaping
- Askama-specific template syntax to learn

### Trade-offs
- Chose compile-time errors (Askama) over runtime flexibility (Tera)
- Chose owned types to simplify lifetimes

---

## Related ADRs

- [ADR-001: HTMX Web Dashboard Architecture](./adr-001-htmx-web-dashboard.md) — Uses templates

---

## History

- **2025-04-12**: Phase 1 — Pilot with `HeaderTemplate`
- **2025-04-12**: Full migration — all 4 fragments migrated to Askama