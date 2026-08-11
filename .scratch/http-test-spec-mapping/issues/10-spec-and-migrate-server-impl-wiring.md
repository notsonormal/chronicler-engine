# Spec and migrate server_impl_wiring.rs

Type: task (AFK)
Status: pending
Blocked by: 14

## Question

Decide whether `tests/http/requires_migration/server_impl_wiring.rs` belongs in a spec or is exempt, and migrate it back to `tests/http/server_impl_wiring.rs` with either `SCENARIO:` tags or explicit exemption comments.

## Work

- Decide if server wiring tests are spec-covered behaviour (e.g. a bootstrap/wiring spec) or should be exempt.
- If covered, write the relevant spec under `docs/specs/` and tag the tests.
- If exempt, add explicit exemption comments and no `SCENARIO:` tags.
- Move `requires_migration/server_impl_wiring.rs` to `tests/http/server_impl_wiring.rs`.
- Update `tests/http/mod.rs` and `tests/http/requires_migration/mod.rs`.
- Run `cargo test --test http`.

## Output

- `tests/http/server_impl_wiring.rs` is either tagged or carries explicit exemption comments.
- `requires_migration/server_impl_wiring.rs` is gone.
