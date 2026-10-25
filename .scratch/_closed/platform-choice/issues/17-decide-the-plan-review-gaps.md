# Decide the nine gaps that the review of the build plan found

Type: grilling
Status: resolved
Blocked by: —

## Question

An independent review of [`docs/plans/rust-rebuild-build-plan.md`](../../../../docs/plans/rust-rebuild-build-plan.md) found 38 items ([report](../assets/plan-check-report.md)). The coordinator checked each one. Most of them were text fixes that the sources already decide, and they are applied in the plan. Nine of them need a decision that no source makes. Three of these nine block a slice as written. For each of the nine, which option does the build take?

1. **The game-changed check in P1** (report B1). Today a running Job reads the active-Game pointer through `Storage::current_game_id()` and discards its text when the player switched Games (`PipelineRun::check_game_unchanged`). P1 removes `Storage::game_id`, and rule 2 allows only `current_game.rs` and `GameCatalogue` to read `ActiveGame`. Bootstrap and `test_support` also set the pointer today.
2. **Where the rule-3 token lives** (report B2). Ticket 16 Q4 assumed a token that only `gate.rs` can build. In Rust, the compiler can restrict a constructor to `gate.rs` only if the type is declared in `gate.rs`. Then `Storage::transaction` names an `application` type, and arch-lint denies `storage → application` (`arch-lint.toml`, "Storage adapter must not depend on the application layer"). arch-lint 0.4.3 cannot add an exception (research 10 C11).
3. **The form and name of the game rules, and of `job::start`** (report B3, S15). The kept guardrail `free fn location` (`tests/infrastructure/guardrails/free_fn.rs`) allows top-level free functions only in `mappers`, `utils`, `builders`, `test_support`, `bootstrap` and `handlers`, and its exempt list is closed. So `domain/model/world_rules.rs::apply_world_rules` (P3) and `pipeline/job.rs::start` (core) go red. The name is also open: tickets 09 and 26 write `apply_world_rules`; target design D-28 and `CONTEXT.md` ("_Avoid_: World rules") give **game rules**.
4. **The story side effects of the world-rule methods in P3** (report S3). The methods that move also write the story: `attempt_movement` adds a `System` message; `log_movement_completion` writes `pending_location` and the arriving header; `evaluate_triggers` and `commit_trigger_narration` write `last_trigger` and `pending_event` and add a Narration. None of these fields is in `SwipeState`.
5. **`Swipe.state` and `Message.kind` from C1 to C5, and the saved Games** (report S7). C1 adds the fields while today's code still makes Swipes and Messages until C4. Sketch 25 §3.1: "Saved Games may be discarded (OR-01), so no migration." No source says what value the old code writes, or what the migration does with today's rows.
6. **The value types that P4 needs** (report S11). `HookInput` (sketch 24 §2.2) holds `Kind`, `&SwipeState`, `&[StoryEntry]`, `WorldView` and `EventRecord`. The plan puts `StoryEntry`, `EventRecord` and `Kind` in C1.
7. **The record writes of the P5 `Tx`** (report S12). Ticket 16 Q1 keeps today's Game writes off `Tx` in P5. Then `Tx` has no named record write, and the rule-14 test ("fails half-way, leaves no row") needs at least two writes in one unit. The compiler half of rule 14 and rule 9 ("no `GameState` type") hold only from C6.
8. **The mechanical-change rule for tests** (report S16). The plan's §2 says that a legacy test that breaks only by a mechanical change (a new parameter, a renamed type) changes in place. Ticket 09 says literally: "A test that the slice breaks is rewritten into its seam contract or its scenario under the one rule". P1 alone touches about 257 call sites in 53 files.
9. **Trackers** (report N15). Sketch 24 gives `AnalysisOutput::Tracker` and `SwipeState.trackers`. Target design D-34 builds only the hooks that have an Agent; no tracker Agent exists today.

## Options and recommendations

The recommendations are the coordinator's (inferred). The owner decides.

| # | A (recommended) | Other options |
|---|---|---|
| 1 | P1 keeps today's behaviour. The Job takes its `GameId` from the request, and the game-changed check compares it with `GameCatalogue::active_game_id()` (a read through `GameCatalogue`, which rule 2 allows). The core step deletes the check, and I5 turns green there. Bootstrap and `test_support` set the pointer through `GameCatalogue`. The rule-2 row matches the calls `ActiveGame::get` and `ActiveGame::set`, not the construction in `wiring.rs` | B: P1 drops the check, so a Job goes on for its own Game from P1 (target design §5). That changes behaviour in a slice with no scenario, and part of I5 goes green early |
| 2 | The token type lives in `storage/` (for example `TxToken` in `transaction.rs`) with a `pub(crate)` constructor. `Storage::transaction*` takes it, so every transaction must carry one (compiler). One call-site row allows `TxToken::new` only in `generation/gate.rs` and `test_support/`. This row replaces ticket 16 Q4's row, so the count of syn rows stays 7 | B: change the `storage → application` deny row of arch-lint (the whole rule, no exception). Storage then may import any application module. C: drop the token and keep rule 3 as a syn call-site row on `transaction*` (ticket 13's form) |
| 3 | `domain/model/utils/game_rules.rs` with the free function `apply_game_rules`: `utils` is an allowed folder, `domain/model/utils/` exists, and the name follows D-28 and `CONTEXT.md`. In `job.rs`, `start`, `generate`, the step functions and `run_step` are methods or associated functions of `Job` | B: a method `SwipeState::apply_game_rules`. The inherent-impl locality rule then puts about 330 lines into `swipe_state.rs`. C: a unit struct `GameRules` with `GameRules::apply` in `domain/model/game_rules.rs` |
| 4 | `apply_game_rules` returns an outcome value (the movement result, an unknown location, a fired Trigger). `GameState` keeps thin wrappers until the core step, and they write the story side effects from that value. The observable behaviour does not change; the shape of the code does | B: P3 moves only the parts that touch no story field, and the mixed methods stay in `GameState` until the core step. Less moves in P3; more moves in the core step |
| 5 | From C1 to C5, `Swipe.state` and `Message.kind` are `Option`, and today's code writes `None`. In C6, the migration discards today's saved Games (OR-01) and makes the columns `NOT NULL`, and the fields lose the `Option`. **This deletes the Games in your dev database at C6** | B: C1 discards the saved Games at once, and the old code fills the fields from `GameState` (the adapter that ticket 09 refused). C: keep today's Games with a data migration (sketch 25 refused it) |
| 6 | P4 adds the value types that `HookInput` needs (`Kind`, `StoryEntry`, `EventRecord`, `WorldView`) in `domain/model/`, and the P4 glue fills them from `GameState`. C1 reuses them, so they are not throw-away | B: P4 uses today's types (`MessageEntry`, `StoredTriggerContext`) in `HookInput`, and C1 changes `HookInput`. More throw-away code |
| 7 | The P5 `Tx` gets the Game record writes that the core step's create and reset need (insert a Game, delete a Game). The rule-14 test runs a unit of two writes and fails the second. P5 delivers the test half of rule 14; rule 9 and the compiler half of rule 14 move to C6, where the old writes go | B: `Tx` moves to C1 in full. P5 keeps the lookup by World, the tier-1 switch and the target tests. This goes against ticket 09's reason for P5 ("gives the core step a tested `Tx`") |
| 8 | Confirm the rule: a legacy test that breaks only by a mechanical change changes in place. It is recorded as your ruling and amends ticket 09 | B: ticket 09 literally: every broken test is rewritten under the one rule. P1 then rewrites the tests of 53 files |
| 9 | No `Tracker` output and no `SwipeState.trackers` now. They come with the first tracker Agent, for the same reason as D-34 | B: build both now, with no Agent that uses them |

## Context

- [Report](../assets/plan-check-report.md) (copied from `tmp/plan-check-report.md`): 3 blockers, 19 should-fix, 16 nits; the coordinator verified B1 to B3 in the code.
- [Decide the four gaps that the build plan found](16-decide-the-build-plan-gaps.md) Q4 (amended by question 2 here).
- Target design D-28, D-34, §5; sketch 25 §3.1, §3.5; sketch 24 §2.2; research 10 C11; ticket 09; ticket 13 §3.

## Done when

The owner has chosen an option for each of the nine. The plan is changed in place, and each **Open: ticket 17, Qn** mark is replaced by the ruling.

## Grilling notes

**2026-10-17, owner.**

- Q1 (game-changed check in P1) = **A**. P1 keeps today's behaviour. The Job takes its `GameId` from the request, and `check_game_unchanged` compares it with the active Game read through `GameCatalogue` (rule 2 allows this read). The core step deletes the check, and I5 turns green there. Bootstrap and `test_support` set the pointer through `GameCatalogue`. The rule-2 row matches the calls `ActiveGame::get` and `ActiveGame::set`, not the construction of `ActiveGame` in `wiring.rs`.
- Q2 (where the rule-3 token lives) = **A**. The token type lives in `storage/` (for example `TxToken` in `transaction.rs`) with a `pub(crate)` constructor. `Storage::transaction*` takes it, so the compiler makes every transaction carry one. One call-site syn row allows `TxToken::new` only in `generation/gate.rs` and `test_support/`. This row replaces the ticket 16 Q4 row (the hidden second constructor in `gate.rs`), so the syn rows stay seven. arch-lint does not change.
- Q3 (form and name of the game rules and of `job::start`) = **A**. The game rules are the free function `apply_game_rules` in `domain/model/utils/game_rules.rs`. The name follows target design D-28 and `CONTEXT.md` ("_Avoid_: World rules"), so the plan and every ticket text that says `apply_world_rules` read as `apply_game_rules`. In `application/pipeline/job.rs`, `start`, `generate`, the step functions and `run_step` are methods or associated functions of `Job`. The `free fn location` guardrail does not change.
- Q4 (story side effects in P3) = **A**. `apply_game_rules` changes only `SwipeState` and returns an outcome value (the movement result, an unknown location, the matched Trigger). `GameState` keeps thin wrappers until the core step; they write the story side effects (`attempt_movement`'s `System` message; `log_movement_completion`'s `pending_location`, location header and `arriving_location_write`; `commit_trigger_narration`'s `last_trigger`, `pending_event` and Narration) from that outcome. Correction to the report (known): `evaluate_triggers` takes `&self` and has no side effect.
- Q5 (`Swipe.state`, `Message.kind` and the saved Games) = **A**. From C1 to C5 both fields are `Option`, and today's code writes `None`. In C6 the migration deletes today's saved Games (OR-01) and makes the columns `NOT NULL`, and the fields lose the `Option`. The owner accepts that the dev database loses its Games at C6.
- Q6 (value types of `HookInput`) = **A**. P4 adds `Kind`, `StoryEntry`, `EventRecord` and `WorldView` in `domain/model/`, and the P4 glue fills them from `GameState`. C1 reuses them and adds only their storage (C1 may adjust their shape). P4's seam contracts test the final input shape.
- Q7 (record writes of the P5 `Tx`) = **A**. `Tx` gets the Game record writes that the core step's create and reset need (insert a Game, delete a Game), with no production caller until C4 (ticket 16 Q1 holds). The rule-14 test runs a unit of two writes and fails the second. P5 delivers the test half of rule 14; rule 9 and the compiler half of rule 14 move to C6, where the old write paths go.
- Q8 (mechanical-change rule for tests) = **A**. A legacy test that breaks only by a mechanical change (a new parameter, a renamed type) changes in place and stays on the legacy list. A test whose behaviour breaks is rewritten into its seam contract or scenario under the one rule. This amends ticket 09's "a test that the slice breaks is rewritten".
- Q9 (trackers) = **A**. No `AnalysisOutput::Tracker` and no `SwipeState.trackers` now. Both come with the first tracker Agent, for the same reason as D-34.

## Answer

**2026-10-17, owner: option A for all nine.**

| # | Ruling | Lands in |
|---|---|---|
| 1 | The game-changed check stays until the core step. The Job takes its `GameId` from the request and compares it with the active Game read through `GameCatalogue`. Bootstrap and `test_support` set the pointer through `GameCatalogue`. The rule-2 row matches `ActiveGame::get` and `ActiveGame::set` calls only | P1; the check goes in the core step (I5) |
| 2 | `TxToken` lives in `storage/transaction.rs` with a `pub(crate)` constructor; `Storage::transaction*` takes it. One call-site row allows `TxToken::new` only in `generation/gate.rs` and `test_support/`. Replaces the hidden second constructor of ticket 16 Q4; seven syn rows; arch-lint does not change | C2 |
| 3 | `apply_game_rules`, a free function in `domain/model/utils/game_rules.rs` (name per D-28 and `CONTEXT.md`). `job.rs` uses methods and associated functions of `Job`. The `free fn location` guardrail does not change | P3; C3 |
| 4 | `apply_game_rules` changes only `SwipeState` and returns an outcome value. `GameState` keeps thin wrappers that write the story side effects until the core step | P3 |
| 5 | `Swipe.state` and `Message.kind` are `Option` from C1 to C5; C6's migration deletes the saved Games (OR-01), makes the columns `NOT NULL` and removes the `Option`. The dev database loses its Games at C6 | C1; C6 |
| 6 | P4 builds `Kind`, `StoryEntry`, `EventRecord` and `WorldView` in `domain/model/`; C1 reuses them | P4 |
| 7 | The P5 `Tx` gets the Game record writes (insert, delete) with no production caller before C4. P5 holds the test half of rule 14; rule 9 and the compiler half of rule 14 move to C6 | P5; C6 |
| 8 | A legacy test that breaks only mechanically changes in place; a test whose behaviour breaks is rewritten under the one rule. Amends ticket 09 | every slice |
| 9 | No `AnalysisOutput::Tracker` and no `SwipeState.trackers` until the first tracker Agent | — |

The plan [`docs/plans/rust-rebuild-build-plan.md`](../../../../docs/plans/rust-rebuild-build-plan.md) carries each ruling in place, with no **Open: ticket 17** mark left.
