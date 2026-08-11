# Spec and migrate text_check.rs

Type: task (AFK)
Status: pending
Blocked by: 14

## Question

Write the spec for the HTTP text-check endpoints and migrate `tests/http/requires_migration/text_check.rs` back to `tests/http/text_check.rs` with `SCENARIO:` tags.

## Work

- Write `docs/specs/text_check.md` covering the behaviour tested in `text_check.rs`.
- Move `requires_migration/text_check.rs` to `tests/http/text_check.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Tag each test in `text_check.rs` with `// [docs/specs/text_check.md] SCENARIO: N.N`.
- Rewrite or drop any test whose assertions no longer match the spec.
- Run `cargo test --test http`.

## Output

- `docs/specs/text_check.md` exists and is internally consistent.
- `tests/http/text_check.rs` is tagged and compiles/tests green.
- `requires_migration/text_check.rs` is gone.
