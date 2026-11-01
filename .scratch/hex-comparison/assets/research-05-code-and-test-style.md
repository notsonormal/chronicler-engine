# Research 05: Hex code and test style

Status: research asset

Read of [anomalyco/hex](https://github.com/anomalyco/hex) at commit
`7609574ca92c` (clone in `tmp/hex-compare/anomalyco-hex/`), against Chronicler
Engine at this repo. Labels: **known** means this read saw it in the source.
**Inferred** means this read draws the conclusion from what it saw.

## Summary

Hex writes plain Rust and gets its strictness from prose, not from tooling.
Six traits serve 190k lines. Errors are `color_eyre` strings. Files reach 6120
lines. Tests sit inline in the file they test, and their names are full
sentences that carry the invariant's exact number. The one automated code
standard is a shared lint block, and Hex records the remaining error count for
each lint it turns off. The `clippy.toml` ban on env and clock reads is written
down but switched off in `Cargo.toml` (85 errors left. Known). Chronicler takes
the other road: 55 guardrail rules, 66 `mod.rs` files, `thiserror` enums, inline
test modules banned, and a 2000-line file cap. Ten of the sixteen practices
below are new candidates that the first look did not name. The biggest gap is
the golden images. The biggest conflict is comment length: Hex writes six-line
why-comments, and Chronicler's guardrail fails at five.

## Practices

| # | Practice | Hex evidence | Chronicler state | Candidate |
|---|---|---|---|---|
| 1 | A test name is one sentence of behaviour, and it carries the invariant's number. | `src/rune_hud/hud_placement.rs:386` `below_hangs_6_points_under_the_menu_bar_or_above_the_dock`. `src/suppression.rs:2841`. 4 names start `test_` in 190k lines. | **partly covered**. `tests/STRATEGY.md:247` says a unit test name describes the behaviour. No rule names the number, and 1424 names start `test_` (`src/domain/model/game_tests.rs:7`). | new |
| 2 | Each invariant's number gets a boundary triad `[n-1, n, n+1]` in one test. | `src/suppression.rs:2841-2842` `for duration_ms in [299, 300, 301]`, against the `300 ms` line in `AGENTS.md:605`. | **partly covered**. Several tests pin a boundary (`tests/infrastructure/guardrails/structure_tests.rs:186`). No rule makes it the shape of an invariant's test. | new |
| 3 | A module doc states why the module exists and who its two partners are. | `src/desktop/opencode_service.rs:1-5`. `src/rune_hud/hud_placement.rs:1-5`. 200 of 256 `src/` files carry `//!`. | **covered, different shape**. `CODING_STANDARDS.md:11-15` requires a `// [DOC: ...]` anchor and a one-line summary. The anchor points at the reason, and the file does not state it. | new |
| 4 | Comments run long, name a measured fact, and use `// ---` section banners. | `src/capture_host.rs:163`, `src/rune_hud/hud_placement.rs:11-25`, `src/rune_ui/tests.rs:1906-1908`. 24 banner sites. | **gap, with a conflict**. `tests/infrastructure/guardrails/style.rs:125` fails at five consecutive comment lines. `style.rs:245` fails `// ===` banners. Both call `panic!` on any violation. | new |
| 5 | A capability folder: `src/foo.rs` holds the module code and declares `src/foo/`. Only 4 `mod.rs` files. | `src/dictation.rs` (1554 lines) plus `src/dictation/`. Commit `38de838a` "Fold dictation_* into dictation/" moves the `dictation_*` files into `dictation/` and adds 10 lines to the parent. | **partly covered**. Chronicler has capability folders, and always with `mod.rs` (66 of them). `tests/infrastructure/guardrails/structure.rs:128` forbids code in `mod.rs`, so the parent-file-with-code shape has no Chronicler form. | capability-folder habit |
| 6 | One crate-wide `eyre::Result` with added string context. No per-domain error enum hierarchy. | `Cargo.toml:48` `color-eyre`. 80 files `use color_eyre`. 9 hand-written error enums, 2 of them with a `Display` impl (`src/meeting/generator.rs:48`). | **covered differently**. `src/error.rs:4` uses `thiserror` for `EngineError`, `ApplicationError` and `LlmFailure`. | new |
| 7 | A ledger of disabled lints, each with its remaining error count and its reason. | `Cargo.toml:164-185`. For example `# shared: deny; 85 errors left (t1003).` above `disallowed_methods = "allow"`. | **gap**. `src/lib.rs:4` denies 9 lints outright. `clippy.toml:1` sets thresholds only. No debt record exists. | new |
| 8 | An env and clock ban in `clippy.toml`, with the lint switched off and a count. | `clippy.toml:6-9` bans `std::env::var` and `std::time::SystemTime::now`. `Cargo.toml:176` allows the lint. 60 `env::var(` sites and 27 `SystemTime::now` sites remain in `src/`. | **gap**. `clippy.toml` has thresholds only. 3 `env::var` sites and 2 `SystemTime::now` sites in `src/`. | 6 |
| 9 | Golden-image UI tests: offscreen render against a checked-in PNG, with a tolerance and an update switch. | `src/rune_ui/tests.rs:1906-2039`. `HEX_UPDATE_GOLDENS=1` rewrites (`:1985`). Tolerance 12 per channel and `width*height/500` pixels (`:2010-2012`). 122 PNGs in `tests/golden/`. Rule in `AGENTS.md:13-17`. | **gap**. The only screenshots are failure diagnostics (`tests/test_utils/browser.rs:470`). The closed `ui-verification-redesign` map never weighed image comparison (known: its issues and assets do not mention it). | 5 |
| 10 | An adapter test uses a real fixture at the boundary, not a mock trait. | `src/desktop/opencode_service.rs:396` `serve()` binds a loopback `TcpListener` and asserts the request line. Temp dirs for the state root (`:456`). | **covered differently**. Chronicler substitutes at the port: `ScriptedModel`, `MockBackend`, `Storage::with_failure`. | 7 |
| 11 | A seam by parameter: the production wrapper calls a `_in` or `_with` variant. | `src/desktop/opencode_service.rs:42` `find()` calls `find_in(&state_dir()?)` at `:63`. `src/suppression.rs:1726` `recover_stale_keys()` calls `recover_stale_keys_with(now, flags, key_down)` at `:1735`. | **partly covered**. `Settings::resolve_api_key_with` (`src/domain/model/settings.rs:385`) takes the lookup. The clock has no such variant. | 7 |
| 12 | A trait appears only when a second adapter needs it, and its doc names that adapter. | `src/capture_host.rs:165-167` "What `CaptureHost` needs from an engine: Scrawl's in the app, and in tests one that records no screen." Rule at `AGENTS.md:1468-1469` and `CONTRIBUTING.md:63`. | **partly covered**. `docs/diataxis/explanation/architecture.md:14` rejects a phantom trait, and counts indirection as a cost. The rule names neither "two adapters" nor "a test needs one". | 7 |
| 13 | A production struct swaps OS function pointers under `if cfg!(test)`. | `src/suppression.rs:1545` `const SESSION: Self = if cfg!(test) { ... } else { ... }`. | **gap**. No equivalent. Chronicler substitutes a whole port, or a `_with` parameter. | new |
| 14 | Test-only methods and `for_test()` constructors on production types. | `src/suppression.rs:1638` `is_locked`, `:1719` `suppress_until_release`, `src/audio/browser_media.rs:74` `for_test()`. 276 `#[cfg(test)]` sites. | **gap, and in tension**. `.agents/skills/antipattern-checker/SKILL.md:16` names "a test-only type threaded through production code" a symptom refactor. | new |
| 15 | One check script stages fmt, then clippy, then nextest, with a distinct exit code per stage and a test-filter argument. | `scripts/check:28-43` exit 10, 11, 12. `.config/nextest.toml` `slow-timeout` kills a hung test and names it. `.config/nextest.toml:13-18` gives the GPU tests their own group. | **partly covered**. `build.py` is the one entry point. Both projects use nextest. Chronicler has a filter argument, `python build.py test-pattern "<pattern>"` with `--test-target`, and no per-stage exit code (corrected 2026-11-01, [candidates](./candidates.md) K-38). | new |
| 16 | Unit tests sit inline in a `#[cfg(test)] mod tests` in the file they test. | 241 files, 1664 `#[test]` in `src/`. `src/suppression.rs:2449` is the test module of a 6120-line file. About 9 tests per 1k lines of `src/`. | **covered as the opposite**. `scripts/check_test_structure.py:9` fails any inline test module. Unit tests live in sibling `*_tests.rs`. About 45 tests per 1k lines. | new |

## Details

### Comment style

Hex doc-comments the reason, then the constraint. `src/rune_hud/hud_placement.rs:11-14`
gives one line to each constant:

```
/// The plate's gap under the menu bar: as close as it sits and still
/// reads as its own thing.
pub const TOP_OFFSET: f64 = 6.0;
```

Some comments carry a measurement. `src/rune_ui/tests.rs:1910-1911` says why the
goldens share one device: "creating Vulkan devices from parallel test threads
crashed Mesa's RADV on Linux". `src/dictation.rs:45-48` says why the both-sides
window is 80 ms, and names Karabiner-Elements as the reference. **Known.**
**Inferred:** most of these survive Chronicler's KEEP test in
`/chronicler-comment-fixer` (a hidden constraint, not carried by a name).

The density is close. Hex `src/suppression.rs` has 328 comment lines in 6120
(5.4%). Chronicler `src/application/generation/gate.rs` has 96 in 968 (9.9%).
The difference is the length of one block, not the number of blocks. **Known.**
**Inferred:** Hex targets one dense spec and lets the code carry less prose.
Chronicler targets a rule per comment.

### Error handling

Hex has one error flow. A function returns `color_eyre::eyre::Result`, adds
context with `WrapErr`, and names a failure with `eyre!`. 80 of 256 `src/` files
import it. The 9 error enums are local to a subsystem, and 7 of them have no
`Display` impl. A caller reads the enum, or formats the string. **Known.**
**Inferred:** the enum decides control flow only inside its own module. Across
modules, the string is the contract.

Chronicler draws a typed error at each layer (`src/error.rs:4`), so an HTTP
adapter maps an error arm to a status. Hex has no such map. Its local API
answers a `ControlError` with a code string (`src/control/cli.rs:867`). **Known.**

### Module size and depth

Hex has no file-length rule. `src/suppression.rs` is 6120 lines and holds one
concern: the macOS event tap and the dictation state machine. It is a deep
module: about 100 functions behind a few public entry points. **Known.**
**Inferred:** the size comes from the number of state transitions, not from
mixed concerns.

Chronicler fails at 2000 non-blank lines (`tests/infrastructure/guardrails/structure.rs:309`)
and at nesting depth 3 (`nesting.rs:9`). Its largest `src/` file is
`src/application/narration/narrator_tests.rs` at 1381 lines. **Known.**
**Inferred:** a direct port of Hex's file sizes is incompatible with the
Chronicler gate.

### How an invariant from `AGENTS.md` shows in a test

`AGENTS.md:605` says "Captures shorter than 300 ms discard".
`src/suppression.rs:2841` is `callback_hold_and_double_tap_boundaries_are_milliseconds`.
It loops `[299, 300, 301]` and asserts the discard flips between 299 and 300
(`:2848-2855`). The test name carries the unit, and the loop carries the boundary.
**Known.** No ID links the invariant to the test. The join is the number and
the words. **Inferred.**

Chronicler has no invariant list in prose. A behavior lives in a spec scenario
with a stable ID, and `scripts/validate_feature_spec.py` checks that a test
cites it. The link is mechanical there and semantic in Hex. **Known.**

### Test doubles and seams

Hex prefers a real object with a narrow door. Three shapes appear:

1. **A parameter.** `find_in(&path)` and `recover_stale_keys_with(now, flags, key_down)`
   take the read that production makes.
2. **A fixture at the boundary.** `serve()` is a real loopback server.
   `state_root()` is a real temp directory.
3. **A trait with a real reason to exist.** `CaptureEngine` has one production
   impl, Scrawl's engine (`src/capture_host.rs:337`), and 3 test impls. Its doc
   names the test substitute as the reason (`:165-167`). `AudioTimeline` has two
   production impls (`:958`, `:3589`). `control::Host` has two
   (`src/linux/app.rs:1171`, `src/windows/rune.rs:396`) plus `FakeHost`
   (`src/control/host.rs:607`).

`dyn` appears 112 times in 256 files, and 6 trait definitions exist outside
`crates/`. **Known.** **Inferred:** the small count is a choice, not an accident,
because `AGENTS.md:1468`, `CONTRIBUTING.md:63` and the `CaptureEngine` doc
(`src/capture_host.rs:165-167`) all state the rule.

### Test-only code in production types

Hex marks a helper `#[cfg(test)]` and keeps it on the production type. 276 sites
exist. `src/suppression.rs:1545` goes further: a production `const` picks test
function pointers at compile time, so unit tests never read the real keyboard.
The comment at `:1542-1544` states this. **Known.** Chronicler's
`antipattern-checker` would flag the first shape and has no name for the second.
**Inferred.**

## What I could not find or did not read

- **The golden workflow's review step.** `AGENTS.md:13-17` says a person looks
  at every changed image. I found no record that proves this happens.
- **Any test policy for the TypeScript SDK** beyond the command list in
  `CONTRIBUTING.md`. I did not read `sdk/`.
- **The rest of the tree.** I did not read `crates/`, `prototypes/`, `design/`,
  `site/`, or the `ios/` and `macos/` trees.
- **Whether the lint ledger holds.** I did not run clippy, so I cannot say that
  85 errors is today's count. That is the number the comment claims.
- **A Hex rule for code review.** `AGENTS.md` covers behavior and invariants. It
  states no review checklist. `CONTRIBUTING.md` says the team writes every
  change, so no external review path exists to describe.

## Sampling

I read these in full: `clippy.toml`, `Cargo.toml` (lints and profiles),
`.config/nextest.toml`, `.cargo/config.toml`, `scripts/check`, `CONTRIBUTING.md`,
and `AGENTS.md` sections `## Private Kit Tool`, `## Invariants` (first 60 lines
of 660), `## Development`, `## Future Direction` and `## Current Work`.

I read these three modules with their tests, as the ticket asked:
`src/rune_hud/hud_placement.rs` (452 lines, all), `src/suppression.rs`
(the first 120 lines, the hotkey machine around 1440-1850, and the test module
from 2449), and `src/desktop/opencode_service.rs` (640 lines, all).

I read these in part: `src/capture_host.rs` (1-200, 940-1000),
`src/control/host.rs` (1-60, 594-700), `src/dictation.rs` (1-60),
`src/rune_ui/tests.rs` (1-60, 1906-2109), `src/transcription/service.rs` (55-90),
`tests/keyboard_layout.rs` (1-40).

Counts come from `grep` and `wc` on the clone at `7609574ca92c`. The Chronicler
counts come from this working tree on 2026-11-01.
