# Plan: Multi-World Support

## Objective

Enable loading different game worlds via CLI, with test world for UI tests.

## Background

- Server hardcoded to load Redmist Estate only
- Test data existed but never used
- UI tests ran against live server

## Implementation Steps

### 1. Data Reorganization
- Create `data/worlds/<world_id>/` structure
- Migrate Redmist Estate to new structure
- Create test world

### 2. CLI Parsing
- Add `--world <id>` argument (default: redmist_estate)
- Add `--list-worlds` argument
- Add `--port <port>` argument

### 3. Loading Logic
- Implement `load_world()` in main.rs
- Add `WorldManifest` struct to model/world.rs
- Support backward compatibility with old paths

### 4. UI Tests
- Spawn test server on port 3001
- Run tests against test world

## Files Changed

- `src/main.rs` - CLI args + load_world()
- `src/model/world.rs` - WorldManifest
- `src/server/mod.rs` - configurable port
- `tests/ui_tests.rs` - self-managed server

## New Data

- `data/worlds/redmist_estate/` (migrated)
- `data/worlds/test/` (new)

## Verification

- All 36 tests pass
- `cargo fmt` / `cargo clippy` pass