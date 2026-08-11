# Spec and migrate fragment.rs

Type: task (AFK)
Status: pending
Blocked by: 14

## Question

Write the spec(s) for the HTTP fragment endpoints in `tests/http/requires_migration/fragment.rs` and migrate the file back to `tests/http/fragment.rs` with `SCENARIO:` tags.

## Work

- Decide which existing or new spec(s) cover the fragment tests (e.g. dashboard fragments, games list, status, swipe switch, text check). This file may split across multiple specs.
- Write or update the relevant spec file(s) under `docs/specs/`.
- Move `requires_migration/fragment.rs` to `tests/http/fragment.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Tag each test with the appropriate spec scenario.
- Rewrite or drop any test whose assertions no longer match the spec.
- Run `cargo test --test http`.

## Output

- Spec file(s) updated/created.
- `tests/http/fragment.rs` is tagged and compiles/tests green.
- `requires_migration/fragment.rs` is gone.
