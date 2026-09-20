//! Browser test binary root (Playwright-driven): per-surface behaviour modules mirroring the `docs/specs/browser_<feature>.md` specs (`dashboard`, `games`, `options`, `prompt_presets`, `worlds`) plus `stub/` (stub-browser tests against a fake engine, incl. `invariants` — CSS/layout rendering invariants, named exemption, no spec, test code is the definition).

#[path = "../test_utils/mod.rs"]
mod test_utils;
pub use test_utils::*;

mod dashboard;
mod games;
mod options;
mod prompt_presets;
mod stub;
mod worlds;
