# Quarantine tests/http files without specs

Type: task (AFK)
Status: resolved
Blocked by: 02

## Question

Move every `tests/http/` test file that does not have a corresponding `docs/specs/<spec>.md` file into `tests/http/requires_migration/`, preserving module structure and keeping the suite green.

## Work

- Identify test files in `tests/http/` with matching spec files under `docs/specs/`.
- Move files without matching specs to `tests/http/requires_migration/`.
- Update `tests/http/mod.rs` to declare only spec-backed modules plus `mod requires_migration;`.
- Create `tests/http/requires_migration/mod.rs` to declare the moved modules.
- Update any `super::test_helpers` imports in moved files to `crate::test_helpers`.
- Run `cargo test --test http` to verify.

## Output

- `tests/http/` contains only files with corresponding specs: `actions.rs`, `games_create.rs`, `games_delete.rs`, `games_switch.rs`, `prompt_presets.rs`, `reset.rs`, `retrigger.rs`, `settings.rs`, `story_log.rs`, `swipe_new.rs`, plus shared `test_helpers.rs`.
- `tests/http/requires_migration/` contains: `connections.rs`, `core.rs`, `debug.rs`, `fragment.rs`, `games_fragment_handlers.rs`, `index_handler.rs`, `server_impl_wiring.rs`, `text_check.rs`, `worlds_fragment_handlers.rs`.
- `cargo test --test http` passes: 168 tests.
