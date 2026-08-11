# Spec and migrate debug.rs

Type: task (AFK)
Status: pending
Blocked by: 14

## Question

Write the spec for the HTTP debug endpoints and migrate `tests/http/requires_migration/debug.rs` back to `tests/http/debug.rs` with `SCENARIO:` tags.

## Work

- Write `docs/specs/debug.md` covering the behaviour tested in `debug.rs`.
- Move `requires_migration/debug.rs` to `tests/http/debug.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Tag each test in `debug.rs` with `// [docs/specs/debug.md] SCENARIO: N.N`.
- Rewrite or drop any test whose assertions no longer match the spec.
- Run `cargo test --test http`.

## Output

- `docs/specs/debug.md` exists and is internally consistent.
- `tests/http/debug.rs` is tagged and compiles/tests green.
- `requires_migration/debug.rs` is gone.
