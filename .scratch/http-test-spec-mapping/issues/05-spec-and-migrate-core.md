# Spec and migrate core.rs

Type: task (AFK)
Status: pending
Blocked by: 14

## Question

Write or update the spec(s) covering the behaviour in `tests/http/requires_migration/core.rs` and migrate the file back to `tests/http/core.rs` with `SCENARIO:` tags.

## Work

- Decide which existing or new spec(s) cover `core.rs` tests (e.g. reset behaviour may belong in `reset.md`).
- Write or update the relevant spec file(s) under `docs/specs/`.
- Move `requires_migration/core.rs` to `tests/http/core.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Tag each test with the appropriate spec scenario.
- Rewrite or drop any test whose assertions no longer match the spec.
- Run `cargo test --test http`.

## Output

- Spec file(s) updated/created.
- `tests/http/core.rs` is tagged and compiles/tests green.
- `requires_migration/core.rs` is gone.
