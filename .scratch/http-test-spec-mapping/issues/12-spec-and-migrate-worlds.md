# Spec and migrate worlds_fragment_handlers.rs

Type: task (AFK)
Status: pending
Blocked by: 14

## Question

Write the spec for the worlds management fragment handlers and migrate `tests/http/requires_migration/worlds_fragment_handlers.rs` back to `tests/http/worlds_fragment_handlers.rs` with `SCENARIO:` tags.

## Work

- Write `docs/specs/worlds.md` covering the behaviour tested in `worlds_fragment_handlers.rs`.
- Move `requires_migration/worlds_fragment_handlers.rs` to `tests/http/worlds_fragment_handlers.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Tag each test in `worlds_fragment_handlers.rs` with `// [docs/specs/worlds.md] SCENARIO: N.N`.
- Rewrite or drop any test whose assertions no longer match the spec.
- Run `cargo test --test http`.

## Output

- `docs/specs/worlds.md` exists and is internally consistent.
- `tests/http/worlds_fragment_handlers.rs` is tagged and compiles/tests green.
- `requires_migration/worlds_fragment_handlers.rs` is gone.
