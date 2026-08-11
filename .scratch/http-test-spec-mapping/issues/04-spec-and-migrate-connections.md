# Spec and migrate connections.rs

Type: task (AFK)
Status: pending
Blocked by: 14

## Question

Write the spec for the HTTP connection endpoints and migrate `tests/http/requires_migration/connections.rs` back to `tests/http/connections.rs` with `SCENARIO:` tags.

## Work

- Write `docs/specs/connections.md` covering the behaviour tested in `connections.rs`.
- Move `requires_migration/connections.rs` to `tests/http/connections.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Tag each test in `connections.rs` with `// [docs/specs/connections.md] SCENARIO: N.N`.
- Rewrite or drop any test whose assertions no longer match the spec.
- Run `cargo test --test http`.

## Output

- `docs/specs/connections.md` exists and is internally consistent.
- `tests/http/connections.rs` is tagged and compiles/tests green.
- `requires_migration/connections.rs` is gone.
