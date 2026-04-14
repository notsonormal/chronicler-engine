# Changelog

## 2025-04-14

### Added
- **Askama template migration (pilot)** - Migrated header template from manual `format!` strings to Askama
  - New `src/server/templates.rs` with compile-time validated `HeaderTemplate`
  - New `tests/template_tests.rs` with fast unit tests (<1ms vs ~5s for integration tests)
  - Compile-time validation: missing field = compiler error
  - Added `askama = "0.12"` to dependencies

### Changed
- `src/server/fragments.rs` now uses `HeaderTemplate` instead of manual string formatting

### Added (Full Migration)
- **Full Askama migration** - All 4 templates now use Askama (complete)
  - `StoryLogTemplate` - Renders narration history with auto-escaped text
  - `VisualSidebarTemplate` - Renders room image + NPC portraits
  - `ActionAreaTemplate` - Renders command form with state-aware disabled
  - 12 unit tests in `src/server/templates.rs` (all pass)
  - Rust 2024 compatible (avoided reserved words in CSS)

## 2025-04-12

### Added
- Multi-world support with CLI arguments (`--world`, `--port`, `--list-worlds`)
- Data organized under `data/worlds/<world_id>/`
- Test world at `data/worlds/test/` for UI tests
- UI tests spawn self-managed server on port 3001
- Auto-kill existing process when port is in use
- Static file serving for `/data/images/` and `/assets/`
- Image endpoint route `/data/images/:file` for serving character images
- UI tests for image loading and NPC image visibility
- `run_background.ps1` script for manual testing

### Changed
- Migrated from Ratatui TUI to HTMX web dashboard
- Server added with Axum + WebSocket for real-time updates
- Added `crate::server::*` module, removed `crate::ui::*`
- Fallback service now serves from `assets` directory
- Use `unpkg.com` CDN for HTMX and WS extension (jsdelivr issues on Windows)

### Fixed
- Static image 404s by adding explicit routes and services for `/data/images/`
- Server not binding on Windows (use `Start-Process -WindowStyle Hidden`)
- WS extension not loading (CDN issue - switched to older version 2.0.3)
