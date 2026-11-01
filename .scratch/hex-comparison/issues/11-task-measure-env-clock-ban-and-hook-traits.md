# Measure an env and clock ban, and the two hook traits

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

What does a ban on direct env reads and clock reads cost in Chronicler? What does each option for the two agent hook traits cost?

## Context

- **Covers** C-30 (no seam until two real adapters or a test needs one) and C-31 (env and clock reads behind one door, with a lint ledger). See `assets/candidates.md`, theme E, and checks K-31, K-32 and K-33.
- **Facts (known).**
  - There are 3 `env::var` sites in `src/`: `src/bootstrap/logging.rs:14`, and two in `src/domain/model/settings.rs`: the wrappers that pass `std::env::var` to `resolve_api_key_with` and `check_api_key_available_with`. There are 6 more in `tests/` and `src/test_support/`. `Settings::resolve_api_key_with` already takes the lookup as a parameter.
  - There are 17 clock-read lines in 15 `src/` files.
  - Three tests read the date apart from the code under test: `src/domain/model/game_tests.rs:15` and `:23`, and `src/application/games/catalogue_tests.rs:162`. Each test fails if the UTC date changes between the two reads. This is a real defect.
  - `AfterNarrationAgent` and `EndOfJobAgent` (`src/application/agents/trait_def.rs`) each have one impl (`QuantifierAgent`, `OptionsGeneratorAgent`) and no test impl. `src/application/agents/runner.rs:16-17` holds them as `Option<Box<dyn …>>`. The Chronicler seam rule is at `.agents/skills/codebase-design/SKILL.md:65`.
  - Hex writes a `clippy.toml` ban on `std::env::var` and `SystemTime::now`, but its `Cargo.toml` turns the lint off with "85 errors left (t1003)". Hex keeps a ledger of 7 lints that are off, each with its count of remaining errors (`assets/research-05-code-and-test-style.md`).
- **Do.**
  1. In a scratch git worktree under `tmp/`, not in this checkout, add `disallowed-methods` for `std::env::var`, `chrono::Utc::now`, `chrono::Local::now` and `std::time::SystemTime::now` to `clippy.toml`. Run clippy, then count and list the errors. Remove the worktree when you finish. `ENVIRONMENT.md` explains the machine-wide build slot.
  2. In the asset only, sketch an injected clock that makes the three tests deterministic. Say where the clock enters and how many call sites change.
  3. For the two hook traits, list the options with the files that each one touches: keep the traits, replace them with a concrete type or an enum, or add a test impl.
  4. Recommend one of the map's three options, or a better one: ban env reads only and decide the clock apart; ban both and mark each existing site; ban both and inject a clock. Recommend one hook-trait option too. If a recommendation is a change of one session or less, name the follow-on task.
- **Not in scope.** Change no code in this checkout, and fix none of the three tests. A fix is a follow-on task that ticket 16 can create. Use `/codebase-design` for the seam terms.
- **What already exists.** `build.py` seeds the cold target dir of a worktree from a warm sibling (`scripts/target_seed.py`), so the first clippy run in the worktree is not fully cold. `resolve_api_key_with` and `check_api_key_available_with` already show the `_with` seam form for env reads.
- **Failure modes.**
  - Clippy may fail to resolve a re-exported path such as `chrono::Utc::now`. It then warns that the path does not resolve, and the count is too low. Look for that warning, and use the defining path (`chrono::offset::Utc::now`) if you need to.
  - The full gate runs `scripts/remove_worktrees.py`. That script refuses to remove a dirty tree, a tree younger than 24 h, or a tree with a live process inside. So the worktree survives a gate run in parallel while it holds the edit.
- **Output.** Write `assets/task-11-env-clock-ban.md`. The `## Answer` gives the gist, the recommendation and at most 2 questions for [Confirm the recommendations of tickets 10 to 15 and 20](./16-grilling-confirm-the-recommendations.md).
- **When done.** Add `## Answer` and set `Status: resolved`.

## Answer

Asset: [task-11-env-clock-ban.md](../assets/task-11-env-clock-ban.md).

**Gist (known, measured in a scratch worktree, now removed).**

- The four-path ban gives **79 clippy errors**: 20 in production code (17 files), 3 in `src/test_support/fixtures.rs`, 43 in unit tests, 13 in `tests/`. Clippy resolved all four paths, so no path-resolution workaround was needed.
- With test code exempt, the ban shows exactly the 20 production sites. 19 `#[expect(clippy::disallowed_methods, reason = …)]` marks on the enclosing `fn`s (17 files) cleared them on Rust 1.88.
- The ban does not catch the 3 defective tests, because they read the clock in test code. Their cause is `generate_game_name` reading the date inside.
- A clock injected everywhere is large: `Message::new` has 14 callers in 9 files, and `Storage::new_*` has 183 calls in 35 files.
- The two hook traits each have one impl, no test impl and a closed hook set (a string match in `runner.rs`). Tests substitute at `LlmProvider`, not at the hook.

**Recommendation (inferred).**

- **Ban:** option B, ban both and mark each site, in two steps.
  - T1 (follow-on task, one session or less): pass `today: NaiveDate` to `generate_game_name` and a `Clock` port to `GameCatalogue`. This fixes the 3 tests (6 existing files, 3 new files).
  - T2 (follow-on task, one session or less): add the 4 paths to `clippy.toml` and 18 `#[expect]` marks. Test code is exempt.
- **Hook traits:** replace them with concrete types (11 files: 7 source, 3 docs, 1 comment). Follow-on task T3, one session or less. It also answers Q52 of `code-concepts`.

**Questions for ticket 16.**

1. Does the owner want the ban (T2) at all? With test code exempt, it would not have caught the 3 tests. Its value is that a new production clock read needs a visible mark. T1 stands alone and fixes a real defect.
2. Are the two hook traits meant to stay open for a third Agent? If no, replace them (T3). If yes, keep them and write the reason in `agent_system_design.md`.
