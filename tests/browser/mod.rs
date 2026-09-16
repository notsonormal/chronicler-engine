//! Browser test binary root (Playwright-driven): per-surface behaviour modules mirroring the `docs/specs/browser_<feature>.md` specs (`dashboard`, `games`, `options`, `prompt_presets`, `slash_menu`, `story_log`, `worlds`) + `invariants` (CSS/layout rendering invariants, named exemption — no spec, test code is the definition).

#[path = "../test_utils/mod.rs"]
mod test_utils;
pub use test_utils::*;

mod dashboard;
mod games;
mod invariants;
mod options;
mod prompt_presets;
mod slash_menu;
mod story_log;
mod worlds;
