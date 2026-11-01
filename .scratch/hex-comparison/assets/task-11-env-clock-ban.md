# Task 11: the env and clock ban, and the two hook traits

Covers C-30 and C-31 of [candidates.md](./candidates.md). Measured on 2026-11-01 in a scratch git worktree (`tmp/wt-task11-ban`, from `1a0c9162`). The worktree is removed. This checkout is unchanged.

Labels: **known** = measured or read in this session. **inferred** = reasoned, not built.

## 1. The ban, measured

### What I ran (known)

1. Added to `clippy.toml` in the worktree: `disallowed-methods` for `std::env::var`, `chrono::Utc::now`, `chrono::Local::now` and `std::time::SystemTime::now`.
2. Ran `python build.py clippy` (`cargo clippy --all-targets --all-features -- -D warnings`). Build logs: `logs/build_20261101_200142.log` and three later ones, inside the worktree.
3. Cargo stops after the first failing crate. So I made the lib clean in steps and ran again, until every test crate had compiled. The steps are in the table below.

**No path-resolution problem (known).** Clippy matched all four paths. No log has a warning that a path does not resolve. `chrono::Utc::now` and `chrono::Local::now` both fired, so the re-export path works. The defining-path fallback of the ticket was not needed.

### Error count (known)

| Scope | Sites | Method split | Source |
|---|---|---|---|
| Production code in `src/` | **20** in 17 files | 14 `Utc::now`, 1 `Local::now`, 2 `SystemTime::now`, 3 `env::var` | log `200142` |
| `src/test_support/fixtures.rs` (compiled in the lib with the `testing` feature) | 3 | 3 `Utc::now` | log `200142` |
| Unit tests in `src/**/*_tests.rs` | 43 in 10 files | 43 `Utc::now` | log `200142` |
| Integration tests in `tests/` | 13 in 6 files | 6 `Utc::now`, 6 `env::var`, 1 `SystemTime::now` | logs `200322`, `200740` |
| **All** | **79** | | |

The 20 production sites match candidates K-32 (17 clock lines in 15 files) plus the 3 `env::var` sites: 17 + 3 = 20.

Clippy first reported 66 errors for the lib test build (23 in the lib build). Those counts include the same lines twice, because the lib and its unit-test build both compile the lib code. The 66 are unique locations. The 13 in `tests/` are shared files (`tests/test_utils/browser.rs`, `server.rs`) counted once. Five test crates report them (`llm`, `http`, `storage`, `bootstrap`, `browser`).

### The 20 production sites

| Site | Method | What it does | Kind |
|---|---|---|---|
| `src/adapters/driven/storage/{characters,core,games,personas,presets,settings}.rs` (1 each), `transaction.rs:87,:255`, `worlds.rs:172,:306` | `Utc::now` | 10 `updated_at` or `created_at` stamps for SQL rows | infrastructure time |
| `src/domain/model/message.rs:70` | `Utc::now` | `Message::new` stamps a Message | domain time |
| `src/domain/model/utils/game_name.rs:7` | `Utc::now` | the date in a Game name | domain time, **tested** (3 defective tests) |
| `src/application/llm_recorder.rs:112`, `src/application/ports/llm_provider.rs:39` | `Utc::now` | `created_at` of a stored model call | infrastructure time |
| `src/domain/model/map.rs:54`, `src/adapters/driving/http/utils/handler_helpers.rs:40` | `SystemTime::now` | seconds or milliseconds inside an id | id source, not a time stamp |
| `src/bootstrap/logging.rs:32` | `Local::now` | the date in the log file name | bootstrap |
| `src/bootstrap/logging.rs:14` | `env::var` | `CHRONICLER_LOG_CONSOLE` | bootstrap |
| `src/domain/model/settings.rs:393`, `:407` | `env::var` | the two wrappers that pass `std::env::var` to `resolve_api_key_with` and `check_api_key_available_with` | **the existing door** (K-33) |

### The 13 test sites in `tests/` (known)

`tests/test_utils/browser.rs:78,:79` (`HEADED`, `SLOW_MO`), `:471` (`SystemTime::now`); `tests/test_utils/server.rs:200,:558` (`CARGO_TARGET_DIR`); `tests/llm/flow_llm_tests.rs:18,:147` (API key gates); `tests/storage/llm_message_storage.rs:29,:60,:84,:110`, `tests/storage/preset_storage.rs:758`, `tests/http/llm_messages.rs:166` (`Utc::now`).

### What the ban does not cover (known)

- `Instant::now` is not on the map's list. It has 2 sites in `src/` (`connection_test_service.rs:67`, `adapters/driven/llm/transport/utils/client.rs:24`) and 15 in `tests/`. They measure durations, not dates (inferred).
- `Uuid::new_v4` (a random source, used in `generate_storage_id`) is not on the list either.

### Two probes that show how a ban is scoped (known)

| Probe | Result |
|---|---|
| Put `clippy::disallowed_methods` into the existing `cfg_attr(test, allow(...))` block of `src/lib.rs` (lines 25-35), and `#![allow(clippy::disallowed_methods)]` in `src/test_support/mod.rs` | Clippy reports exactly the 20 production sites. The 43 unit-test sites and the 3 fixture sites go away. |
| Mark each of the 20 sites with `#[expect(clippy::disallowed_methods, reason = "…")]` on its enclosing `fn` | 19 attributes in 17 files clear all 20 production errors. One fn holds two sites. No "unfulfilled expectation" appears. Rust is `1.88.0` (`rust-toolchain.toml`). |

The marks must sit on the `fn`, not on the expression. I did not test an attribute on a struct-literal field such as `created_at: chrono::Utc::now()` (inferred: not allowed there).

Test crates in `tests/` are separate crates. They need their own exemption: at most 4 files (the shared `tests/test_utils/mod.rs`, and the roots of `storage`, `llm`, `http`) (inferred, not built).

### The ban does not catch the defect that started this ticket (known)

The three clock-dependent tests call `chrono::Utc::now()` in test code. If the ban exempts test code, it does not fire on them. The defect comes from a production function that reads the date inside (`generate_game_name`), so a test cannot fix the date. A ban on production code makes a new site of that kind need a visible mark (inferred). It does not repair the existing one.

## 2. The three options for the ban

| Option | What changes | Sites touched | Fixes the 3 tests? | Cost |
|---|---|---|---|---|
| **A. Ban env only** | `clippy.toml` lists `std::env::var`. 3 production sites get a mark (two are the `_with` wrappers). Test code is exempt. | 3 (+ 6 in tests, exempt) | No | Under half a session (inferred) |
| **B. Ban both, mark each existing site** | 4 paths in `clippy.toml`; 19 `#[expect]` marks in 17 files; one `cfg_attr(test)` entry; one `allow` in `test_support`; up to 4 test-crate roots | 20 (+ exempt tests) | No | About one session (inferred from the probe: each clippy run in the probe took 1 to 2 min) |
| **C. Ban both and inject a clock everywhere** | A `Clock` port, a `SystemClock` adapter, and every clock reader takes the clock or a timestamp | 20, and their callers (see below) | Yes | More than one session (inferred) |

Why C is large (known counts):

- `Message::new` is a domain function. It has 14 callers in 9 files (production: `opening.rs`, `gate.rs` ×2; the rest test support and tests). The domain cannot see an application port (inferred from the layer layout), so the timestamp must enter as a value.
- The 10 storage stamps need the clock inside `Storage`. `Storage::new_sqlite` and `Storage::new_in_memory` have 183 calls in 35 files. A constructor parameter would touch all of them. A `with_clock` builder with `SystemClock` as the default would not (inferred).
- The recorder (2 sites), the 2 id sources and the log file name each need their own handling.

Hex's ledger is not a good model here (known, from `assets/research-05-code-and-test-style.md` and K-19): Hex turns its `disallowed_methods` lint off, with "85 errors left". A lint that is off blocks nothing. `#[expect]` keeps the ban on from the first day, and it fails the build when a marked site is removed, so the mark list cannot go stale (inferred).

## 3. An injected clock for the three tests (sketch, inferred)

The three tests: `src/domain/model/game_tests.rs:15` and `:23`, and `src/application/games/catalogue_tests.rs:162`. Each reads the UTC date apart from the code under test. Each fails if the date changes between the two reads.

**Where the clock enters.** In two places:

1. **Domain function: a value.** `generate_game_name(world_name, existing_names, today: NaiveDate)`. The function becomes pure. The domain layer does not see a port.
2. **Application service: a port.** `GameCatalogue` holds `clock: Arc<dyn Clock>` and passes `self.clock.now().date_naive()` to `generate_game_name` at its two call sites (`catalogue.rs:55` in `create_game`, and `:265` in the reset).

**New parts.**

- `trait Clock { fn now(&self) -> DateTime<Utc>; }` in `src/application/ports/clock.rs`.
- `SystemClock` in `src/adapters/driven/` (the one marked `Utc::now` site for the Game name).
- `FixedClock` in `src/test_support/`. Two adapters make the seam real by the rule at `.agents/skills/codebase-design/SKILL.md:65`: a test needs the second one.

**Call sites that change.**

| File | Change | Count |
|---|---|---|
| `src/domain/model/utils/game_name.rs` | `generate_game_name` takes `today`, drops `use chrono::Utc` | 1 definition |
| `src/application/games/catalogue.rs` | two callers pass the date; the struct gets a field and `new` a parameter | 2 callers, 1 constructor |
| `src/bootstrap/wiring.rs:103`, `src/bootstrap/run.rs:102` | pass `Arc::new(SystemClock)` | 2 |
| `src/application/games/catalogue_tests.rs:18` | `new_catalogue` passes a `FixedClock` | 1 |
| `src/application/games/catalogue_tests.rs:162` | expects the fixed date, not the clock | 1 test |
| `src/domain/model/game_tests.rs:8,:17,:25` | call with a fixed `NaiveDate`; lines 15 and 23 drop the clock | 3 calls, 2 tests |
| new files | `clock.rs` (port), the `SystemClock` file, `FixedClock`, plus `mod.rs` lines and a DOC anchor for each new file (`CODING_STANDARDS.md:11-15`) | 3 |

That is 6 existing files and 3 new files (inferred).

**A lighter variant.** Do only step 1. This fixes `game_tests.rs:15` and `:23`. Change `catalogue_tests.rs:162` to compare `game.display_name` with `default_display_name(&game.name)`. No port and no constructor change. The test then no longer checks that the date is today's date (inferred). I do not recommend it: the full sketch costs 6 files more and keeps that check.

## 4. The two hook traits

Facts (known): `AfterNarrationAgent` and `EndOfJobAgent` are in `src/application/agents/trait_def.rs` (23 lines). `QuantifierAgent` and `OptionsGeneratorAgent` are their only impls. No test impl exists. `runner.rs:16-17` holds them as `Option<Box<dyn …>>`. `AgentRunner::from_configs` builds each one from a string match on `agent_type` (`runner.rs:38-60`). A third type needs a new arm there, so the hook set is closed. Tests do not substitute at these traits: `QuantifierAgent::with_provider` (`quantifier/agent.rs:43`) substitutes at the `LlmProvider` port. The glossary says an Agent "runs at a Hook" (`CONTEXT.md:145-150`). The docs defend the traits as "the polymorphism is the point" (`docs/diataxis/explanation/rust_idioms.md:8`; `agent_system_design.md:8`). The code-concepts map asks the same question as its open question Q52 (`.scratch/code-concepts/assets/research-10-synthesis.md:471`).

| Option | Files it touches | Seam-rule result |
|---|---|---|
| **1. Keep the traits** | None. | Two hypothetical seams (one impl each, no test impl). The rule at `SKILL.md:65` is broken in two places, on purpose or not. |
| **2. Replace with concrete types** (`Option<QuantifierAgent>`, `Option<OptionsGeneratorAgent>`) | Delete `trait_def.rs`. Edit `agents/mod.rs:10` (re-export), `runner.rs` (imports, fields, two `Box::new`), `quantifier/agent.rs` and `options/agent.rs` (the trait `impl` becomes an inherent `impl` with `name` and `run`), `quantifier/agent_tests.rs:7`, `options/agent_tests.rs:8` (drop the trait import). Docs: `reference/narrative/agent_system.md:8-9`, `explanation/agent_system_design.md:8`, `explanation/rust_idioms.md:8`. Comment: `tests/infrastructure/guardrails/walkers.rs:19`. The structure line in `AGENTS.md:180` is regenerated by the hook. | No seam. The runner's return types (`QuantifierResult`, `Option<Vec<String>>`) still fix the kind of result for each hook. |
| **3. Replace with an enum** | Same files as option 2, plus an enum type with one variant for each hook. | Same as option 2, with one more type that has one variant (inferred: no gain). |
| **4. Keep the traits and add a test impl** | New stub agents in `src/test_support/`, a `testing`-only constructor on `AgentRunner`, and tests that use them. | Two adapters: the seam becomes real by the rule. But no test needs it. `runner_tests.rs:87` already tests a failing Agent through a real Agent with a failing model (inferred). |

Option 2 is 11 files (7 source, 3 docs, 1 comment). Option 4 adds 2 to 3 files and gives a double that no test asked for (inferred).

## 5. Recommendation

**Ban (inferred).** Take option B in two steps, in this order:

1. **Follow-on task T1, fix the date seam.** Pass the date to `generate_game_name` and a `Clock` to `GameCatalogue` (section 3). This repairs the 3 tests. It is one session or less (inferred: 6 files and 3 small new ones).
2. **Follow-on task T2, ban with marks.** Add the 4 paths to `clippy.toml`. Add 18 `#[expect(clippy::disallowed_methods, reason = "…")]` marks on the remaining production sites (19 minus the `generate_game_name` fn, which T1 clears), and the test exemptions of section 1. One session or less (inferred).

Why B and not A or C:

- A guards 3 sites, and 2 of them already use the `_with` door. It leaves the 17 clock sites out, and the 3 tests exist because of a clock site.
- C touches `Message::new` (14 callers) and `Storage` (183 constructor calls) for stamps that no test needs to fix (inferred from the 3-test list; no other test reads the clock apart from the code under test, per K-32).

State the weakness: with test code exempt, the ban would not have caught the 3 tests (section 1). Its value is that a new production clock read needs a visible mark and a reason. That value is moderate (inferred). The owner may judge it not worth a session; T1 stands on its own because it fixes a real defect.

**Hook traits (inferred).** Take option 2, concrete types. Reasons: one impl each, no test impl, a closed hook set (the string match), tests that substitute at `LlmProvider` and not at the hook, and `rust_idioms.md:8` that already says trait objects belong only where the polymorphism is the point. It is one session or less (11 files). Name it **follow-on task T3, replace the two hook traits with concrete types**. T3 answers Q52 of `code-concepts` in the same way, and changes the docs sentence that defends the traits, so the owner should agree first.

If the owner wants the hooks to stay open for a third Agent, take option 1 and write that reason in `agent_system_design.md`. Do not take option 4 only to satisfy the rule (inferred).

## 6. What I did not do or check

- I did not build any of the follow-on changes. The `#[expect]` probe is from a throwaway worktree.
- I did not try `#[expect]` on an expression inside a struct literal. The marks sit on whole `fn`s.
- I did not measure how the ban behaves with `clippy::disallowed_methods` and a function pointer such as `.map(Utc::now)`. No such use exists in the 79 sites (known: all are direct calls).
- The test-crate exemption (at most 4 files) is inferred. Only the error locations are measured.
