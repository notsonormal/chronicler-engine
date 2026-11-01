# Research Hex code and test style

Type: research (AFK)
Status: resolved
Blocked by: —

## Question

Which code and test style does Hex follow, and where does it differ from Chronicler's standards?

## Context

- **Hex size.** 263k lines in 355 `.rs` files. 241 files have inline `#[cfg(test)]` modules, and there are 2216 tests in total. `tests/` has 2 test files, plus `golden/` and `fixtures/`.
- **Sample.** Read three modules of different kinds, with their tests. For example:
  - a pure model: `src/rune_hud/hud_placement.rs`
  - a state machine: `src/suppression.rs` (6120 lines, the largest file)
  - an adapter: `src/desktop/opencode_service.rs`
- **Tooling.**
  - golden images in `src/rune_ui/tests.rs` (`HEX_UPDATE_GOLDENS`)
  - `clippy.toml` (the `abattoir-lints` block) and `[lints] workspace = true` in `Cargo.toml`
  - `.config/nextest.toml` and `scripts/check`
  - Hex has no `rustfmt.toml`.
- **What to find.**
  - comment style and error handling
  - module size and depth
  - how an invariant from `AGENTS.md` shows in a test
  - test doubles and seams
  - how a test names its behavior
  - the test density: about 8 tests per 1k source lines in Hex, and about 45 in Chronicler
- **Chronicler side.**
  - `CODING_STANDARDS.md`, `tests/STRATEGY.md` and `tests/AGENTS.md`
  - `docs/diataxis/reference/coding_standards/`
  - the guardrail tests and `clippy.toml`
  - the `antipattern-checker` and `codebase-design` skills
- **Candidates this touches** ([findings asset](../assets/findings.md)): 5 (golden images), 6 (`disallowed-methods`), 7 (the seam rule), and the capability-folder habit.

## Answer

Hex gains its strictness from prose and not from tooling. Six traits cover 190k
lines, errors are `color_eyre` strings, files reach 6120 lines, tests sit inline
in their own file, and each test name is a sentence that carries the invariant's
exact number. The one automated code standard is a shared lint block that records
the remaining error count of each lint it turns off. Its env and clock ban is
written in `clippy.toml` but switched off in `Cargo.toml`, with 85 errors left.
So candidate 6 rests on a rule Hex does not enforce today. Chronicler fails five
consecutive comment lines and bans inline test modules, so two Hex habits cannot
land without a rule change. Sixteen practices are recorded and ten are new, the
golden images among them.

Asset: [research-05-code-and-test-style.md](../assets/research-05-code-and-test-style.md)
