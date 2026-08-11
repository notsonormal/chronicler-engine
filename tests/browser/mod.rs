//! Browser test binary root (Playwright-driven): `behaviour` (client-side JS interaction, tagged against `docs/specs/browser.md`) + `invariants` (CSS/layout rendering invariants, named exemption — no spec, test code is the definition).

#[path = "../test_utils/mod.rs"]
mod test_utils;
pub use test_utils::*;

mod behaviour;
mod invariants;
