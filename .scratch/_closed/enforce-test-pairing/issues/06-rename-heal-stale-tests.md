# Rename heal_stale_* tests to test_ prefix

Type: task
Status: closed
Blocked by: —

## Question

In `src/application/generation/gate_tests.rs`, three pre-existing tests lack the required `test_` prefix, violating `unit_test_standards.md` Pattern 1 (`fn test_<behaviour>_<expected_outcome>()`):

- `heal_stale_resets_generating_status_when_no_active_slot`
- `heal_stale_leaves_generating_status_when_slot_active`
- `heal_stale_is_noop_when_status_is_idle`

Rename each to add the `test_` prefix (e.g. `test_heal_stale_resets_generating_status_when_no_active_slot`). These were pre-existing but the file is already being modified by this effort, so fix them while here. Run `cargo test --lib` for the generation module.

## Resolution

Renamed the three functions in `src/application/generation/gate_tests.rs`:
- `heal_stale_resets_generating_status_when_no_active_slot` → `test_heal_stale_resets_generating_status_when_no_active_slot`
- `heal_stale_leaves_generating_status_when_slot_active` → `test_heal_stale_leaves_generating_status_when_slot_active`
- `heal_stale_is_noop_when_status_is_idle` → `test_heal_stale_is_noop_when_status_is_idle`

`cargo test --lib generation` passes (18 tests).
