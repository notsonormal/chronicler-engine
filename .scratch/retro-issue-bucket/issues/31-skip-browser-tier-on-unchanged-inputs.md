# Skip the browser tier when its inputs have not changed since a green run

Type: grilling
Status: open
Blocked by: —

## Question

Should `python build.py` skip the browser tier when every file the browser tests depend on is the same as in the newest green browser run? If yes, which inputs make up the digest, and how does the epilogue and the meaning of "green" in `AGENTS.md` change?

## Context

- **Source.** `/retro` of session `01a12280` (2026-10-09, implementer `impl-rest`, 21:09–21:39).
- **Cost.** The browser tier is about 150 s of a 210 s warm full gate. In the session it ran 3 times, about 7.7 min of 30 min:
  - `python build.py browser` after the `dashboard.js` split: 162 s (`logs/build_20261009_212307.log`).
  - Full gate 1: 145 s of 208 s (`logs/build_20261009_213047.log`).
  - Full gate 2: 153 s of 223 s (`logs/build_20261009_213438.log`).
- **The skip would have saved nothing in that session.** `catalogue.rs` and test files changed between the browser run and gate 1. `cargo fmt` changed 3 Rust files between gate 1 and gate 2. Gate 2 ran only because the task prompt said `--no-fmt`. That cause is fixed: the `--no-fmt` instruction was removed from `AGENTS.md` (`## Concurrent Builds`).
- **What exists.** `build.py` already journals a tree digest per run in `logs/build_history.txt` (ticket 30). The digest covers the whole tree, not the browser inputs.
- **Concurrency limit.** `.config/nextest.toml` runs 3 browser tests at once. Four gained nothing and hit the 8 GiB memory limit, so more parallel tests is not an easy alternative.

## Risks to decide

- **"Green" changes meaning.** `AGENTS.md` defines green as the full gate on the reported tree. A skipped tier needs a new definition, or the skip must be opt-in.
- **A missed input gives a false green.** Candidate inputs: `src/`, `assets/`, `data/`, `tests/browser/`, `tests/test_utils/`, `Cargo.toml`, `Cargo.lock`, `.config/nextest.toml`, the Rust toolchain, the Playwright/Chromium version. Unknown: whether that list is complete.
- **Option to compare.** Drop the idea, and keep the browser tier unconditional in the full gate.
