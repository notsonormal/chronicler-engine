# Findings: Hex compared to Chronicler Engine

Status: reference

This note is the first look at Hex, made on 2026-11-01, and it is an asset of the [map](../map.md). The first look was shallow. It read part of `AGENTS.md`, a few config files and `git log` statistics. It did not read `docs/`, the source and test code, the history of `AGENTS.md`, or GitHub. The practices below are candidates, not a complete list.

## Sources

| Repo | Commit read | Local clone |
|---|---|---|
| [anomalyco/hex](https://github.com/anomalyco/hex) (the Rust rewrite) | `7609574ca92c`, 2026-10-31 | `tmp/hex-compare/anomalyco-hex/` |
| [kitlangton/Hex](https://github.com/kitlangton/Hex) (the archived original) | `3e300721b878`, 2026-10-14 | `tmp/hex-compare/kitlangton-hex/` |

`tmp/` is git-ignored. If the clones are gone, clone the repos again at these commits.

## Corrections to the starting assumptions

- `kitlangton/Hex` is a Swift app (SwiftUI and The Composable Architecture), not Rust. Its agent files are a short `CLAUDE.md` and two `.cursor/rules/` files.
- The Rust rewrite, `anomalyco/hex`, keeps its agent rules in the repo. `AGENTS.md` has 1486 lines (about 100 KB). The workflow rules are not in it: it points to `~/code/control/AGENTS.md`, which is private and global.
- `skills/` in `anomalyco/hex` holds one skill for Hex users who edit `hex.config.ts`. It is not a developer skill.

## Numbers

Measured on 2026-11-01.

| | anomalyco/hex | Chronicler Engine |
|---|---|---|
| Rust lines | 263k (`src/` and `crates/`) | 42.6k in `src/`, 52.4k in `tests/` |
| Tests | 2216, mostly inline `#[cfg(test)]` (241 files) | 1912, inline test modules not allowed |
| Tests per 1k source lines | about 8 | about 45 |
| Commits | 1146 in 15 weeks; 852 in weeks 43 and 44 | 868 in 33 weeks |
| Median changed lines per commit, last 300 | 151 (p90: 1515) | 382 (p90: 3049) |
| Agent docs in the repo | `AGENTS.md` only: about 455 lines of module map, 660 lines of invariants | `AGENTS.md`, `CODING_STANDARDS.md`, `CONTEXT.md`, 38 skills |
| Check loop | `scripts/check`, 40 lines | `build.py`, 1752 lines, and 27 scripts |
| Who merges | The agent merges its own PR when the checks pass | Nothing is committed without owner approval |

In weeks 43 and 44, Kit Langton made 422 commits and the author names `Dax Raad` and `Dax` made 377 (inferred: one person).

## Hex practices

Each entry gives the source file in `anomalyco/hex` and the status in Chronicler.

1. **A deterministic risk gate on agent diffs.** `.github/agent-policy/check.ts` (118 lines, no AI; corrected 2026-11-01, [candidates](./candidates.md) K-02) sorts a diff into `clean`, `flagged` or `dax`.
   - Path rules: `Cargo.toml`, `Cargo.lock`, `build.rs`, `AGENTS.md`, build and install scripts, updater code.
   - Line rules: new URLs, process spawns, network APIs, `unsafe`, encoded data, invisible characters, secret env reads.
   - Size rule: more than 600 changed lines or 25 files.
   - Effect: a `flagged` diff needs the strict reviewer, and a `dax` diff needs the owner. The workflow `.github/workflows/agent-policy.yml` runs the check from the base branch, so a patch cannot change its own policy.
   - Chronicler: guardrail tests check structure, and the permission system limits the agent. No step sorts a diff by risk.
2. **Invariants in prose with exact values.** The `## Invariants` section of `AGENTS.md` states each behavior with its numbers, for example "Captures shorter than 300 ms discard". The `## Architecture` section gives each module one bullet that says what it owns.
   - Chronicler: these facts are spread over `CONTEXT.md`, `docs/diataxis/`, the feature specs and the tests.
   - Cost: the Hex file is about 25k tokens, and it loads into every session.
3. **A feature map that states its proof.** `docs/features/` is organized by user capability. Each behavior has a stable ID (for example `setup.download-button`). Each entry tells three things apart: the behavior is built, a check exists, or someone saw it work. One rule says: "A test's existence is not a passing run."
   - Chronicler: `scripts/validate_feature_spec.py` links scenarios to tests. I don't know whether any record shows what someone saw work.
4. **Small units of work that land automatically.** An issue pipeline turns one issue into one PR. The commits carry the trailer `Co-authored-by: Hex issue pipeline`. 59 such commits are dated from 2026-10-29 to 2026-10-31. `.hivemind.toml` defines a land queue that batches up to 4 PRs, runs `verify`, and installs the built app. Eight commits come from `opencode-agent[bot]`.
   - Inferred: the small diffs are what make the automatic merge safe.
5. **Golden-image UI tests.** `src/rune_ui/tests.rs` renders each UI state in light and dark and compares it to a PNG in `tests/golden/`. `HEX_UPDATE_GOLDENS=1` rewrites the images. The rule in `AGENTS.md`: a UI change regenerates the images in the same PR, and a person looks at each changed image.
   - Chronicler: the Playwright tests check behavior. I found no screenshot comparison.
6. **Clippy `disallowed-methods`.** `clippy.toml` bans `std::env::var` ("read env in config.rs at startup") and `std::time::SystemTime::now` ("use config::now() or an injected clock"). The block header `abattoir-lints` suggests a lint set shared across their repos (inferred).
   - Correction (research 05): Hex does not enforce this ban. `Cargo.toml` sets `disallowed_methods = "allow"` with the comment "shared: deny; 85 errors left (t1003)". The rule is written, but it is off.
   - Chronicler: `clippy.toml` sets thresholds only. See [Env and clock reads](#env-and-clock-reads) for the call sites.
7. **"No seam until there are two real adapters or a test needs one."** This rule appears in `AGENTS.md` (`## Future Direction`) and in `CONTRIBUTING.md`.
   - Chronicler: hexagonal ports, and the `antipattern-checker` skill. The port layer was checked later: `LlmProvider` and `TextChecker` pass the rule. `AfterNarrationAgent` and `EndOfJobAgent` each have one impl and no test impl, so they do not pass it (corrected 2026-11-01, [candidates](./candidates.md) K-31).
8. **Labs.** New and user-requested features ship off by default behind a toggle.
   - Chronicler: no feature flags.

Other facts:

- Hex tests run through nextest with a slow timeout that kills a hung test and names it (`.config/nextest.toml`). Chronicler also uses nextest.
- Hex modules are large: `src/suppression.rs` has 6120 lines. Recent commits fold flat modules into capability folders ("Fold dictation_* into dictation/").
- `ROADMAP.md` is the authoritative work list. A `## Current Work` section in `AGENTS.md` keeps live constraints visible, for example a release freeze.

## The main difference (inferred)

The two projects put their effort in different places:

- **Hex** uses a small amount of process scaffolding. It invests in one dense spec, product-level checks (golden images, CI on three OSes, checklists on real devices) and an automated merge path.
- **Chronicler** invests in process scaffolding: skills, `build.py`, guardrails, and a high test density.

## What is public and what is private

The pipeline is partly public. Each pipeline PR on GitHub has a body that describes the run. For example, the body of [PR #714](https://github.com/anomalyco/hex/pull/714) says:

- "The fixer and reviewer saw only the validated issue form, never the issue text." Inferred: this keeps prompt injection in the issue text away from the agents.
- "**Gates**: build passed in the apollo sandbox; policy `clean`; `console-anthropic-cyber/claude-opus-5-5` approved."

On 2026-11-01, GitHub had 497 PRs and 219 issues for the repo, and 17 PRs had the label `agent-strict-review`.

The pipeline prompts, `~/code/control/AGENTS.md` and the opencode configuration are not in the repo. Public posts or talks by Kit Langton or Dax Raad may describe them.

## Env and clock reads

Call sites in Chronicler on 2026-11-01:

- `std::env::var`: 3 in `src/` (`src/bootstrap/logging.rs`, and two in `src/domain/model/settings.rs`). 6 in `tests/` (API keys, `HEADED`, `SLOW_MO`, `CARGO_TARGET_DIR`). `Settings::resolve_api_key_with` and `check_api_key_available_with` already take the lookup as a parameter.
- Clock reads: 17 lines in 15 production files (corrected 2026-11-01, [candidates](./candidates.md) K-32) call `Utc::now`, `Local::now` or `SystemTime::now`. They are in the domain (`message.rs`, `utils/game_name.rs`, `map.rs`), the application layer and the storage adapters. No injected clock exists.
- Three tests need a substitute clock (corrected 2026-11-01, [candidates](./candidates.md) K-32). `generate_game_name` reads the clock internally. `src/domain/model/game_tests.rs:15` and `:23` read the clock separately, and `src/application/games/catalogue_tests.rs:162` computes the date apart from `create_game`. Each test fails if the UTC date changes between the two reads. By Hex's own rule (practice 7), this test justifies a clock seam.
