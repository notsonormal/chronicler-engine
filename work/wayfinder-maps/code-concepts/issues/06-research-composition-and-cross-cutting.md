# Research: the concepts in the composition root and the cross-cutting code

Type: research
Status: resolved

## Question

Which concepts does the code in `src/bootstrap/`, `src/utils/`, `src/error.rs` and the error, logging and configuration shapes that run through every layer follow, and where do opposing or overlapping concepts meet in it?

Write the asset in the shape that the map's Notes give (Concepts, Meeting points, Shapes with no name, Questions). Do not rule on a concept or label a meeting point; write it as a question.

Dig the concepts out of the code. You get no list of concepts: do not open `assets/research-01-lens.md` or the other research assets. Read the code, and find the shapes that it repeats and the choices that it makes. Give each one the name that software-engineering books and well-known essays give it (a principle, an architecture style, a design style, a pattern or an idiom), and its opposite. When no such name fits, put the shape under Shapes with no name.

For the cross-cutting shapes (errors, logging, configuration, settings reads, lock handling), sample every layer, not only these folders. Record how the layers differ.

## Done when

Every module in the scope is read, or is named in the asset as sampled with the reason. Every row cites `path:line` evidence and has a known, inferred or guessed label.

## Answer

Asset: [research-06-composition-and-cross-cutting.md](../assets/research-06-composition-and-cross-cutting.md). The researcher read all 11 non-test files in the scope (`src/main.rs`, `src/lib.rs`, `src/error.rs`, `src/bootstrap/`, `src/utils/`; 1,220 lines). The researcher read the test bodies that touch a claim (`src/bootstrap/run_tests.rs`, `load_tests.rs`, `wiring_tests.rs`) and the names of the other unit tests of the scope. For the cross-cutting shapes the researcher sampled every layer by search, then read the hit sites: lock handling (about 40 `.lock()` sites), logging (macro counts per layer and level), error types and their conversions, settings and environment reads, ambient clock and ids, guardrails (`arch-lint.toml`, `clippy.toml`, `layers.rs`) and test seams (every `cfg(feature = "testing")` site). The researcher did not open the other research assets. The researcher checked the 354 citations with a script and corrected 19 of them (shifted ranges, one range past the end of the file, three bare file names, and one allow-list cited at the wrong lines).

- **60 concepts.** The five with the strongest evidence (known): Composition Root with pure manual dependency injection (`bootstrap/wiring.rs:98-177`), with a guardrail that limits where `WiredApp` may be named; one crate-wide typed error enum (`EngineError`, 20 variants) with translation at each layer boundary (`error.rs:112-205`, `application/errors.rs:6-70`, `http/error.rs:13-43`); lint-enforced "no panics in production" with named escape hatches (`lib.rs:4-23`, `arch-lint: allow(...) reason=`); seed-if-absent, so the database outranks the JSON files after the first run, with settings as one stored singleton row edited by a closure (`storage/settings.rs:106-125`); late binding of the model connection per call, through a resolver closure (`bootstrap/wiring.rs:44-72`, `application/llm_recorder.rs:11-17`).
- **24 meeting points, each written as a question.** Examples: which settings are read at each use and which once at boot (connections per call, Agent list once, MP-1); a bad seed file skips for a world manifest and an NPC file but returns `Err` for a map, a persona and a preset file, and the doc says "skipped" (MP-2); a bad Agent config gives a silent default runner while a bad connection id stops the boot (MP-5); the boot builds a second `GameWriter`, `ActiveGame` and `GameCatalogue` before the wired graph exists (MP-6); `WiredApp` and `AppState` as a bag of services against per-need injection (MP-8); seven error types cross one failed request (MP-9); `EngineError::Config` serves as a catch-all (MP-10); the pure model logs and reads the environment (MP-14); the default context size has two homes (MP-18).
- **11 shapes with no name, 10 questions.** One question (Q2): the log line names `chronicler_<date>.log`, but the appender adds its own date to the file name, so the printed name probably differs from the real file (inferred).
- **Hazards to check (inferred, not tested):**
  - `bootstrap/wiring.rs:161-166` and `:199-201`: `AgentRunner::from_configs` returns `Err` for an unknown Agent type, and `.unwrap_or_default()` drops it with no log. A Job then runs with no Quantifier result (`application/agents/runner.rs:69-74`).
  - `bootstrap/run.rs:179` and `:200-203`: one malformed preset file ends `ensure_presets`, so later preset files do not seed. The caller only warns (`run.rs:54-60`). `startup.md` says such a file is skipped. The same shape holds for a map file (`load.rs:123`) and a persona file (`load.rs:179`), which also stop the rest of `seed_game_data`.
  - `bootstrap/logging.rs:46`: when the log directory fails, the fallback logger sets `debug` and ignores `RUST_LOG`.
  - `application/llm_recorder.rs:83`: a failed save of the success record fails a call whose text exists. A failed save of the failure record only warns (`:70-73`).
  - `bootstrap/wiring.rs:198`: the test graph builds the Options recorder from stored settings, while the narrator and Quantifier use the fixed mock.
  - `Cargo.toml:28`: `parking_lot` is a dependency and no file in `src/` or `tests/` uses it, while every production `lock()` ignores poison on `std::sync::Mutex`.
  - `tests/infrastructure/guardrails/layers.rs:86` and `arch-lint.toml:162-166` name `app_err_to_response()` and `ApplicationService`, and neither exists in `src/`.
  - `bootstrap/run.rs:51`: `to_str().unwrap_or("chronicler.db")` puts the database in a relative path when the exe path is not valid UTF-8.
- **Check by the coordinator:** open. Suggested spot check of five citations: `bootstrap/wiring.rs:161-166`, `bootstrap/run.rs:179`, `bootstrap/logging.rs:46`, `application/llm_recorder.rs:83`, `Cargo.toml:28`. The researcher checked all citations by script, not by a second reader.
- **Check by the coordinator:** spot check of three hazards against the code. All three match (known). `src/bootstrap/wiring.rs:161-166` ends `AgentRunner::from_configs(...)` with `.unwrap_or_default()`. `src/bootstrap/run.rs:179` applies `?` to each preset file in the loop, and `run.rs:54-56` only logs the error, so one bad preset file stops the files after it; `docs/diataxis/reference/startup.md:35` says such a file "is skipped without halting bootstrap". `src/bootstrap/logging.rs:45-47` builds the fallback logger with `EnvFilter::new("debug")`, so it ignores `RUST_LOG`.
