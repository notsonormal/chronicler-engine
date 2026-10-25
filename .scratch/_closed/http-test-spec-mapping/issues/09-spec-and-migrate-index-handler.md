# Spec and migrate index_handler.rs

Type: task (AFK)
Status: wontfix
Blocked by: 14

## Question

Write or update the spec for the HTTP index/dashboard handler and migrate `tests/http/requires_migration/index_handler.rs` back to `tests/http/index_handler.rs` with `SCENARIO:` tags.

## Work

- Decide whether `index_handler.rs` belongs in `dashboard_fragments.md` or another spec.
- Write or update the relevant spec file under `docs/specs/`.
- Move `requires_migration/index_handler.rs` to `tests/http/index_handler.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Tag each test with the appropriate spec scenario.
- Rewrite or drop any test whose assertions no longer match the spec.
- Run `cargo test --test http`.

## Output

- Spec file updated/created.
- `tests/http/index_handler.rs` is tagged and compiles/tests green.
- `requires_migration/index_handler.rs` is gone.

## Comments

- 2026-10-25 — wontfix. Stale: the specs it plans were never written, and 36 specs now exist without it. Superseded by the plan [`docs/plans/drain-requires-migration-quarantine.md`](../../../../docs/plans/drain-requires-migration-quarantine.md).
