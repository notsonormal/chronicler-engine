# Spec and migrate games_fragment_handlers.rs

Type: task (AFK)
Status: pending
Blocked by: 14

## Question

Write or update the spec for the games list fragment handlers and migrate `tests/http/requires_migration/games_fragment_handlers.rs` back to `tests/http/games_fragment_handlers.rs` with `SCENARIO:` tags.

## Work

- Decide whether `games_fragment_handlers.rs` belongs in `games_list.md`, `games.md`, or another spec.
- Write or update the relevant spec file under `docs/specs/`.
- Move `requires_migration/games_fragment_handlers.rs` to `tests/http/games_fragment_handlers.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Tag each test with the appropriate spec scenario.
- Rewrite or drop any test whose assertions no longer match the spec.
- Run `cargo test --test http`.

## Output

- Spec file updated/created.
- `tests/http/games_fragment_handlers.rs` is tagged and compiles/tests green.
- `requires_migration/games_fragment_handlers.rs` is gone.
