# Build plan: the Rust rebuild of the game-state lifecycle

## How to use this plan

This plan builds [target design](../../.scratch/turn-state-model/assets/target-design.md) version 2 in the current Rust crate, in place: six preparation slices (P0 to P5), then one core step.

**The implementer of one slice reads §1, §2 and the section of that slice.** The core step also reads the P5 section, because P5 writes its target tests. The reviewer reads the same sections.

Each item cites the ticket that decides it, so a doubt about an item goes to that ticket, not to a new choice. A gap that needs a decision stops the slice and goes to the owner.

Facts with no label are known: read in a ticket, or in the code at commit `d067420b`. **Inferred** and **guessed** facts are labelled; check them before you rely on them.

Source: [Write the build plan](../../.scratch/platform-choice/issues/14-write-the-build-plan.md), in the map [the Rust rebuild](../../.scratch/platform-choice/map.md).

## 1. The design sources

Read [The rebuild at a glance](../../.scratch/platform-choice/assets/shape-at-a-glance.md) first: the code after the rebuild, what goes, what holds the shape in place, and the path.

The design sources, in the order of their authority:

1. [Target design](../../.scratch/turn-state-model/assets/target-design.md) version 2. Its §9 decision log wins over the sketches.
2. [Sketch 25](../../.scratch/turn-state-model/assets/sketch-25-new-design.md): the code shape of the six areas (§3), the rules (§6.2), the fate tables (§7).
3. [Sketch 24](../../.scratch/turn-state-model/assets/sketch-24-agent-hooks.md): the Agent hooks.
4. The tickets of the map. Where two tickets conflict, the later ruling wins. Ticket 15 amends tickets 08, 09, 11 and 13; ticket 16 amends tickets 11, 12 and 15.

The sketch text that a later decision changed. Build the right-hand column:

| Sketch text | Build this | By |
|---|---|---|
| Sketch 25 rule 3 names `Storage::create` | `Storage::transaction` and `Storage::transaction_new_game`, which take a capability token (core step, §9.5) | ticket 11 R7; ticket 15 Q3; ticket 16 Q4 |
| Sketch 24 §2.2: four hook traits, `save_rewrite`, `PromptContext` | The analysis hook and the Options hook only: `AnalysisAgent`, `OptionsAgent`. The other two hooks and their save unit come with their first Agent, in a later effort | target design D-34 |
| Sketch 25 §3.3: `generate` as one function | Step functions, a `JobPlan` value and one runner. `save_text` returns `SavedText`; `save_state_update` takes it by value | ticket 05 |
| Sketch 25 `LlmRoles`, `application/agents/roles.rs` | The hook traits, `AgentRunner`, and `Narrator` in `application/narration/narrator.rs` | target design D-29; sketch 24 |
| Sketch 25 rules 8 and 10; ticket 05 rule 17 | Removed. The R-59 scenario takes the place of rule 8 | ticket 15 Q3 |
| Ticket 11 R3: a copy-and-swap `Tx` on the in-memory backend | `Tx` on SQLite only. Tier 1 runs on SQLite `:memory:` | ticket 15 Q4 |
| Ticket 08 R2, R4: no unit test on a pure domain function; a count pin | The one rule of §2 | ticket 15 Q2 |
| "World rules" | The term is **game rules**, and the function is `apply_game_rules` in `domain/model/utils/game_rules.rs`. Where tickets 09 and 26 write `apply_world_rules`, read `apply_game_rules` | target design D-28; [ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q3 |

## 2. Rules for every slice

From [Decide the build path](../../.scratch/platform-choice/issues/09-decide-build-path.md) (ticket 09), amended by tickets 15 and 16.

**Order and people.**

- `dashboard-ui-review-phase3` merged to `main` on 2026-10-17 (`ac22d864`), so P0 starts from `main`.
- Each slice starts on its own branch from the current `main`: `wf/rebuild-p0` to `wf/rebuild-p5`, and `wf/rebuild-core` for the core step. The implementer of the slice makes the branch when the slice starts and deletes it after the merge (owner, 2026-10-17).
- One implementer at a time. The slices run in order. Every commit passes the commit gate below, and the owner approves each commit (Q8).
- No other work runs during the build. A player-visible bug found during the build becomes a spec scenario in the slice that runs (Q4, Q11).

**The one rule** (ticket 08 R1 to R3, amended by ticket 15 Q2). A unit test lives in one of three places:

1. a **seam contract**: a `CASE` list in the test file of a box behind a port boundary. The boxes are the storage adapters, the LLM providers and the transport, the Harper text checker, each Agent, and the `Narrator`;
2. a pure function in `domain/model/`, as a plain unit test;
3. a file on the **legacy list** (P0).

A behaviour that a player can see is a **scenario** in `docs/specs/`, with one test at the tier that sees it.

**Tests in a slice** (ticket 08 R5).

- A slice that changes behaviour writes its scenarios in `docs/specs/` before their tests (ticket 26 R7 condition 3).
- A test whose subject the slice deletes goes with it.
- A test whose subject is new code follows the one rule.
- A test in a legacy file that breaks only by a mechanical change (a new parameter, a renamed type) changes in place. Reason (inferred): ticket 15 Q2 removed the count pin, so the allowlist checks files, not test counts. A test whose behaviour breaks is rewritten into its seam contract or scenario under the one rule ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q8, which amends ticket 09).
- A test that the slice does not break stays as it is.

**Guardrails** (Q10). Each rule lands in the slice that creates the code it guards, and its slice table is the source of truth. The 15 rules by slice: P1: 1, 2. P3: 12. P4: 11, 16. P5: 14 (its test half). Core: 3, 4, 5, 6, 7, 9, 13, 14 (its compiler half), 15, 18. Rules 8, 10 and 17 are removed (ticket 15 Q3; ticket 16 Q5).

**Docs** (Q12). Each doc change lands in the slice that makes it true. The generated blocks of `AGENTS.md`, `tests/AGENTS.md`, `docs/AGENTS.md` and `guardrails.md` regenerate through the pre-commit hook.

**The dev loop.** While you work, use `python build.py clippy` and `python build.py test-pattern "<pattern>"` with the test target of P0. The full gate runs inside the commit gate. A narrow loop does not compile the other test binaries, so a change to a shared API shows only in `clippy` or in the full gate (ticket 07).

**The commit gate** (owner, 2026-10-17, during the review of this plan). Before every commit of the build, run `/simple-after-plan` (`.agents/skills/simple-after-plan/SKILL.md`), with the scope of that one commit. The skill is the source of truth for its steps. Two rules adapt it to the build:

- **A step whose input is empty is skipped.** Example: the document review runs only when the commit changes a hand-written `.md` file. Generated blocks do not count.
- **The rows below replace or move a step.** A commit with no row runs the whole skill, except its retro.

| Commit | Change to `/simple-after-plan` |
|---|---|
| P0 to P5, the last commit of the slice | The retro runs here, once per slice |
| C4 (red on purpose, §9.3) | The full gate becomes `python build.py clippy`, `python build.py guardrails`, `python build.py test-pattern "" --test-target lib` and `python build.py test-pattern "" --test-target storage` (inferred: tier 1 and the browser tier are the red binaries). All four are green. Name the red tests in the commit message |
| C7 (docs only) | The comment cleanup and the code review have no code to check, so they are skipped |
| C7, the last commit of the core branch | The retro runs here, once for the core step |

Every other commit runs the full gate (`python build.py`), and all tests must pass. Tests marked `#[ignore]` by P5 do not run, so they do not make the gate red. `python build.py --include-llm` adds `--run-ignored all` and runs them, so it is red between P5 and C5.

**Done.** A slice is **done** when all of these are true:

1. Every Content item of the slice exists in the code.
2. Every listed test has its listed fate: written, rewritten, moved, or deleted.
3. Every listed guardrail rule is in place with its reason (a walker row, an arch-lint row, or the compiler types that the table names), and it is green.
4. Every listed doc states the new truth. A search of `docs/diataxis/` for the names that the slice removed finds nothing.
5. The slice's own **Done when** checks pass.
6. The last commit of the slice has passed the commit gate, and the owner has approved it.

A preparation slice that is done merges to `main`. §9.7 holds the merge of the core step.

## 3. P0: the test rule and the guardrail machinery

**Content.**

1. **`build.py` target selection** (ticket 07). `test-pattern` takes an optional test target: `cargo nextest run --test <TARGET>`, or `--lib`. With no target, the step runs today's all-binaries command. The flag is `--test-target <name>`, with `lib` for `--lib` (a distinct name, because `build.py` already has `--target-dir` and argparse matches option prefixes).
2. **The one rule** (§2) in `tests/STRATEGY.md` "The tiers", for the whole repository.
3. **The seam-case checker** `scripts/validate_seam_cases.py` (ticket 26 R4). It fails on a declared `CASE` with no tagged test, a tag with no declared case, a test with no tag in a declaring file, and a duplicate id. `build.py` runs it in the guardrails family, next to `spec-coverage`. It has no declared case until P4 (inferred: the machinery lands in P0, as ticket 13 §6 does for the walkers).
4. **The legacy list and its allowlist guardrail** (ticket 08 R4, amended by ticket 15 Q2). The list holds today's `src/**/*_tests.rs` files. The guardrail fails when a file holds a unit test and is not on the list, not a seam-contract file, and not under `domain/model/`. A unit test in a seam-contract file calls only that box's entry point (ticket 26 R2 clause 3). The list is a table literal in the guardrail, as the walker rows are (inferred: ticket 08 R4 makes it part of the call-site guardrail). `requires_migration/` keeps its own pin.
5. **The four table-driven syn walkers** (ticket 13 §2): banned name in a scope, call-site allowlist, type check, call count. Each walker reads a table literal in Rust with one row per rule and the row's reason in a comment. The rows come with their code, in later slices.
6. **The dead-path assertion** (ticket 13 §2). Each path-limited row of the new walkers must match at least one file under its root, else the guardrail fails. Today's two dead walkers (`handler return type`, `server layer boundaries`) get the assertion when C4 re-points them (inferred: else P0 is red until C4).
7. **`generate_guardrails_doc.py`** parses the row tables in one strict, documented form. A row with no reason fails `--check`.
8. **`registry_tests.rs` rework** (ticket 13 §2): one table-driven detection test per walker kind, in place of one hand-written test per rule.

Today's 23 guardrail rules stay as `check_*` functions on their own code (ticket 13 §2, Q3).

**Spec scenarios.** None. P0 changes no behaviour.

**Tests.** The walker detection tests and the checker tests (ticket 26 R9). Today's tests stay.

**Guardrail rules.** The machinery only.

**Docs.** `tests/STRATEGY.md`; `docs/diataxis/reference/coding_standards/unit_test_standards.md`, rewritten as the readable surface of the one rule (ticket 26 R11); the Development Loop table of `AGENTS.md` and `build.py --help` name the test target; the "a test for each branch" sentence of `tests/STRATEGY.md` goes; `integration_test_standards.md` and `unit_test_standards.md` lose their examples that call `storage.set_game_id` or `current_game_id` only in P1 (below).

**Done when** (§2, plus): `python build.py test-pattern "<pattern>" --test-target http` runs only the `http` binary. Ticket 07 measured 5.0 to 5.6 s after a one-line edit, against 13.2 s for all binaries; record the real number in the slice ticket. Record the line count of the walkers with their tests (guessed: about 350, sketch 25 §6.2).

## 4. P1: `ActiveGame` and `CurrentGame`

**Content** (sketch 25 §3.2; ticket 11).

- `application/games/active.rs`: `ActiveGame { Mutex<Option<GameId>> }` with `get` and `set`. `ActiveGame::set` keeps the touch of `games.updated_at` that `set_game_id` does today (research 10 §6).
- `adapters/driving/http/utils/current_game.rs`: `CurrentGame(pub GameId)`, a `FromRequestParts` extractor that reads `ActiveGame` once. No active Game gives `400` (R-45).
- The `Storage::game_id` field and `set_game_id`, `current_game_id` and `game_id()` go. Each `Storage` read and write that used them takes a `GameId` parameter. `Storage::new_sqlite(pool, game_id)` becomes `Storage::new_sqlite(pool)`.
- `GameCatalogue` (switch, create, reset, delete) uses `ActiveGame`.
- The game-changed check stays ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q1). The Job takes its `GameId` from the request, and `PipelineRun::check_game_unchanged` compares it with the active Game, read through `GameCatalogue`. The same holds for the other non-handler readers of today's Game id in `application/pipeline/` and `application/games/view_query.rs`: each takes the `GameId` as a parameter, or reads it through `GameCatalogue`. The core step deletes the check (I5).
- `bootstrap/run.rs` and `test_support/test_data_builder.rs` set the pointer through `GameCatalogue`.
- The handlers that call `current_game_id()` today take `CurrentGame`, with three exceptions that take `Option<CurrentGame>` and keep today's answer when no Game is active: the Games list (`current_and_saved_games`), the status poll, and the stop, which answers "failed" (target design §5). Inferred: today's code treats "no Game" as a normal state in these three.

The change touches about 240 call sites in 50 files and is mostly mechanical (ticket 11).

**Spec scenarios.** None new. Scenarios 11.2 (`swipe_new.md`) and 14.6 (`retrigger.md`) cover R-45.

**Tests.**

- Tier 1 covers `ActiveGame` and `CurrentGame` through the Game switch, create and reset scenarios (ticket 26 R9).
- Tests that call the removed methods change in place (§2, the mechanical change).
- The `core_tests.rs` cases whose subject is the `game_id` atomic go with it.

**Guardrail rules.**

| # | Rule | Row |
|---|---|---|
| 1 | Every read and write names its Game | banned-name walker: no `static` item and no `Atomic*` type in `storage/` (a `&'static str` lifetime is not a `static` item) |
| 2 | The active-Game pointer has one reader | call-site walker: only `current_game.rs` and `GameCatalogue` call `ActiveGame::get` and `ActiveGame::set`; building the `ActiveGame` value in `bootstrap/wiring.rs` is not a call it matches (ticket 17 Q1) |

**Docs.** `storage.md` ("A `Storage` instance holds the active game id"), `game_flow.md` (the check that discards a Job on a switch: see ticket 17), `architecture_system.md`, `integration_test_standards.md` and `unit_test_standards.md`, where they name `current_game_id` or `set_game_id`.

**Done when** (§2, plus): a search of `src/` and `tests/` for `set_game_id`, `current_game_id` and `game_id()` on `Storage` finds nothing.

## 5. P2: delete the arrival opening

**Content** (target design D-25; sketch 25 §7.1).

- Delete `application/arrival_service.rs` and `arrival_service_tests.rs`.
- Delete `ArrivalSpawnRequest` and `spawn_arrival_task_if_needed` from `bootstrap/init_game.rs`, and their call in `bootstrap/run.rs`.
- Delete `PromptContext::build_narration_prompt`: its only production caller is the arrival opening (research 04 §2.4).
- Player-visible change: a World with no scenario text starts with no LLM opening. Today the arrival opening runs only for such a World (`spawn_arrival_task_if_needed` returns early when the World has scenario text). The opening Narration from the World's opening text comes in the core step (area 5).

**Spec scenarios.** None: no scenario in `docs/specs/` names the arrival opening.

**Tests.** `arrival_service_tests.rs` goes. No tier-1 or browser test names the arrival opening.

**Guardrail rules.** None.

**Docs.** The generated structure index drops `arrival_service.rs`. Ticket 13 §4 names `startup.md` and `narration_system.md`; on 2026-10-17 neither mentions the arrival opening, so check them and edit only a reference that you find.

**Done when** (§2, plus): a search of `src/`, `tests/` and `docs/diataxis/` for `arrival_service`, `ArrivalService` and `spawn_arrival` finds nothing.

## 6. P3: `SwipeState` and the game rules

**Content** (sketch 25 §3.1, §7.2; ticket 09).

- `domain/model/state/swipe_state.rs`: `SwipeState`, a field of today's `GameState`. It holds today's `movement`, `scene` and encounter log.
- `domain/model/utils/game_rules.rs`: the free function `apply_game_rules(&mut SwipeState, ..) -> GameRulesOutcome` ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q3; `utils` is an allowed folder of the `free fn location` guardrail). About 330 lines move out of `GameState` with no change of logic (inferred, sketch 25 §7.2).
- `apply_game_rules` changes only `SwipeState`. It returns an outcome value: the movement result, an unknown location, the matched Trigger ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q4; the type name is a placeholder). `GameState` keeps thin wrappers until the core step. The wrappers write the story side effects from the outcome: the `System` message of `attempt_movement`; `pending_location`, the location header and `arriving_location_write` of `log_movement_completion`; `last_trigger`, `pending_event` and the Narration of `commit_trigger_narration`. `evaluate_triggers` has no side effect.
- Today's callers reach the moved code through the `SwipeState` field of `GameState`. Today's Snapshot format stays.
- The core step keeps this code and deletes only the `GameState` shell around it (inferred, ticket 09).

**Spec scenarios** (ticket 08 R6; ticket 26 R7). `docs/specs/actions.md` already states the Trigger behaviour in scenarios 1.4, 1.6, 1.7, 1.8 and 2.4, and the movement in 1.14. Ticket 26 R5 names the clauses of the game rules that a player can see: a movement (1.14), the Characters present and the encounter record (1.6), a Trigger (1.4, 1.8), a contradiction (2.1). P3 adds a scenario to `actions.md` only for a clause of `apply_world_rules` that a player can see and no scenario states; the property-test set covers the inner combinations. The behaviour does not change, so the tests are green.

**Tests.**

- The world-rule tests of `game_state_tests.rs` break when their methods move. They move as plain unit tests onto `apply_game_rules` (§2, the one rule, place 2).
- One property-test set over generated inputs for the inner combinations of `apply_game_rules` (ticket 26 R5). `proptest` is already a dependency.
- The other `game_state_tests.rs` tests stay.

**Guardrail rules.**

| # | Rule | Row |
|---|---|---|
| 12 | The game rules are pure | arch-lint: add `model` → `storage` to the deny list. `domain/` imports no `storage` today, so the row is green at once (ticket 13 §1) |

**Docs.** `game_flow.md`, where it describes the world-rule methods of `GameState` (inferred).

**Done when** (§2, plus): `GameState` holds no game-rule logic; each kept method is a wrapper that calls `apply_game_rules` and writes only story fields. `python build.py guardrails` is green (the `free fn location` guardrail accepts `game_rules.rs`).

## 7. P4: the Agent edge on values

**Content** (sketch 24 §2.2, §7; target design D-29 to D-34; ticket 09).

**Order** (owner, 2026-10-17, plan review). The first commit of P4 is the new strict fake with the real Quantifier at tier 1, on today's `AgentRegistry`: the configured Agents take the place of `AgentRegistry::default()`, and the tier-1 files of the strict-fake rework (below) script their calls. Behaviour does not change, so this commit is green. A defect that the empty registry hid becomes a scenario (§2). The Agent edge, the `Narrator` and the value types follow, checked by the stricter suite.

- `domain/model/agent.rs`: `ExecutionPhase` becomes `Hook`, with the analysis hook and the Options hook (D-34). `AgentResult` becomes the output types `AnalysisOutput` and `OfferedSet`. `AgentContext` (`&GameState`) becomes `HookInput` (values). `AgentSettings` carries the preset text and the token budget. `HookInput` and `AgentSettings` carry values only: no `GameId`, no dock record, no failure line. `StatePatch`, `Confidence` and `Role` stay.
- `application/agents/trait_def.rs`: the `AnalysisAgent` and `OptionsAgent` traits. Each returns a value.
- `application/agents/registry.rs` becomes `runner.rs`: `AgentRunner`, built from `AgentConfig` as today. The `PostGeneration` merge loop of `core.rs` moves into `AgentRunner::analysis`.
- The Quantifier implements `AnalysisAgent`, and the Options Agent implements `OptionsAgent`. Their Settings and preset reads move to the caller (D-33). `from_config_with_storage` goes.
- `application/narration/narrator.rs`: `Narrator` with `narrate`. `call_narrator` and the preset lookups move here from `pipeline_run.rs`. `PromptAssembler::with_storage` goes; the token budget comes in as a value.
- `LlmCallRecorder` (`application/llm_recorder.rs`) stays. The Agents and the `Narrator` call the model through it, as the Agents and `ActionPipeline` do today, and it saves the LLM message record through its `SaveLlmMessageFn`. That record is outside the designed records (ticket 11, its storage table), so rule 16 does not cover it. Each seam contract of P4 has a case for the call record (ticket 26 R5).
- `domain/model/`: the value types that `HookInput` needs: `Kind`, `StoryEntry`, `EventRecord` and `WorldView` (sketch 24 §2.2; [ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q6). The glue below fills them from `GameState`. C1 reuses them and adds their storage; C1 may adjust their shape.
- No `AnalysisOutput::Tracker` and no `SwipeState.trackers`. They come with the first tracker Agent, for the reason of D-34 ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q9).
- Glue: today's pipeline builds `HookInput` from `GameState` (guessed: less than 50 lines). The core step deletes it. It is the only throw-away code of the path (ticket 09).
- **The model-port fake** (ticket 12 Q4; ticket 26 R6; ticket 16 Q3). It is a new test-only type in `src/test_support/` (owner, 2026-10-17, plan review; the name `ScriptedModel` is a placeholder), with one script entry per model call. `MockBackend` stays the lenient runtime provider of `LlmBackendType::Mock`, which tier 3 and dev runs use. Its test-only knobs (`with_narrations`, `with_prompt_responses`, `with_delay`, `narration_started`) go when their last tier-1 user moves to the new fake.
  - It can hold one model call until the test releases it.
  - It is strict: a model call that the test did not script fails the test.
  - Tier 1 runs the real Quantifier. The `AgentRegistry::default()` wiring of tier 1 (`src/test_support/context.rs`, `tests/http/support/app_wiring.rs`) gives way to the configured Agents, and each tier-1 turn scripts its analysis answer.

**Spec scenarios.**

| Case | Spec file | Green on P4? |
|---|---|---|
| I1: Action while busy, with the two-fast-clicks assertion (two accepts at the same moment save one Input) | `actions.md` | yes (ticket 12) |
| I1: `/options` while busy | `options.md` | yes (inferred, ticket 12). If it is red, move it to the core step |
| The Quantifier scenarios that a player can see (ticket 08 R6; ticket 26 R7). Existing: 1.4, 1.6, 2.5. P4 adds a scenario only for a Quantifier behaviour that a player can see and no scenario states | `actions.md` | yes |

**Tests.**

- **Seam contracts** (ticket 08 R3, R6): one for the Quantifier, one for the Options Agent, one for the `Narrator`. Each declares its `CASE` list in its test file. The Narrator's cases are the sections of the prompt, the fit to the token budget and the input cleanup (today `assembler`, `builders/sections`, `utils/context`, `sanitize`, `token_budget`).
- The Quantifier and Options `agent_tests.rs` break and become their seam contracts. Their `prompt`, `parser` and `orchestration` tests stay legacy (ticket 08 R6).
- The test files of `assembler`, `builders/sections`, `utils/context`, `sanitize` and `token_budget` move into the Narrator seam contract, one `CASE` per test (ticket 08 R3, R6: the `prompting/` tests go to the Narrator contract in the build).
- `tests/http/races.rs` is new, registered in `tests/http/mod.rs` (ticket 12 Q6). Its module header names its cases. It holds I1.
- Scenario 3.3's test (`test_delayed_llm_completes_without_deadlock_http` in `actions.rs`) holds the fake in place of its 200 ms delay (ticket 26 R6).
- The tests of 12.1 (`swipe_new.rs`) and 15.1 (`retrigger.rs`) hold a real Job through the route with the holdable fake, in place of the direct `try_claim` arrange (ticket 12 Q3, Q9). They stay green through the core step.
- **The strict-fake rework** (ticket 16 Q3): every tier-1 test that runs a Job scripts each model call it makes: the narrator, the analysis, the event narrator and the Options call. Find them with a search of `tests/http/` for `post_action` and for `/swipe/new` and `/retrigger` (at `d067420b`: `actions.rs`, `swipe_new.rs`, `retrigger.rs`, `story_log.rs`, `swipe_switch.rs`, `llm_messages.rs`, `narrator_mode.rs`, `reset.rs`, `options.rs` and `requires_migration/fragment.rs`). Scenario 22.1's "the player has not moved" line gets an assertion (ticket 26 R6).
- `wait_idle` (`tests/http/support/http_requests.rs`) changes only if P4 needs it (ticket 12).

**Guardrail rules.**

| # | Rule | Row |
|---|---|---|
| 11 | Agents and the narrator get values and do not know storage | arch-lint: a scope for `application/agents/**` and `application/narration/**` with `storage` on its deny list |
| 16 | An Agent writes no game record | compiler: each hook trait returns a value; no Agent holds `Storage` or a `JobWriter` |

**Docs.** `integration_test_standards.md` (its `AgentRegistry::default()` call sites) and the tier-1 row of `tests/STRATEGY.md` (the LLM fake). `docs/diataxis/reference/narrative/agent_system.md`, `prompt_system.md` and `docs/diataxis/explanation/agent_system_design.md`, where they describe `ExecutionPhase`, `from_config_with_storage`, `with_storage` or the registry (inferred). `CONTEXT.md` gets the term **Hook** (target design §1).

**Done when** (§2, plus): `validate_seam_cases.py` reports the three seam contracts with every case tagged; no tier-1 test has a story log that holds the `[System] NPC detection uncertain` line (a support assertion in the tier-1 helpers checks it).

## 8. P5: the storage `Tx`

**Content** (ticket 11, amended by tickets 15 Q4 and 16 Q1).

- `adapters/driven/storage/transaction.rs`: `Tx`, on SQLite only. It wraps one rusqlite transaction that rolls back on drop (C4). One `Tx` is one hold of the backend mutex, inside one `with_backend_mut` call (C2). The closure of a `Tx` makes no `Storage` call (C9).
- `TxWrite`: an enum of record writes. Each record write on `Tx` checks the key before it writes. The `Tx` carries the override map from `BackendKind::Test`; in `Direct` the check does nothing (ticket 11 R4; C8). The whole-call override (`with_failure`, `add_failure`, `with_test_failures`, `TestOverride`) stays for every kept method (C7).
- `Storage::transaction` and `Storage::transaction_new_game` (ticket 11 R7), with no token yet. The token comes with `gate.rs` in core commit C2 (ticket 16, gap 4).
- The record writes of `Tx` in P5 are the Game record writes that the core step's create and reset need: insert a Game and delete a Game ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q7). No production code calls them before C4.
- The storage seam contract is the only user of `Tx` in P5 (ticket 16 Q1). Today's Game writes (`create_game`, `delete_game`) keep their current path; they move onto `GameWriter` with create and reset in the core step.
- `Storage::newest_game_of_world(world_key) -> Option<GameId>` (ticket 11 R6; the name is a placeholder), on both backends, through `with_backend_mut`. Order: the newest `messages.timestamp`, else `games.updated_at`. `init_game` calls it through `GameCatalogue`, and creates a missing Game through `GameCatalogue::create_game` (ticket 11 R6; research 04 §2.9). `find_latest_game_for_world`, `list_game_names_for_world`, their raw SQL in `bootstrap/run.rs`, `DbPool::insert_game_from_request` and the second creation path of `init_game` go.
- **Tier 1 moves to SQLite `:memory:`** (ticket 15 Q4; ticket 16 Q1). The `TestAppBuilder` default (`build_service_with_storage`) builds `Storage::new_sqlite` on `DbPool::new(":memory:")`. The tier-1 tests under `tests/http/` that build `Storage::new_in_memory()` themselves move too (inferred: Q4 says "tier 1"). The legacy unit tests in `src/` keep the in-memory backend.
- **The target tests of the core step** land at the end of P5 (ticket 16 Q2). §9.1 lists them.

Constraints (ticket 11): C10 lock order (the guard, then the backend mutex, then the connection mutex); C14 (storage files flat in `storage/`, one `impl Storage` per file); C15 (`Tx` and each read handle hold their inherent impl in their own file).

**Spec scenarios.** The target scenarios of §9.1, written before their tests.

**Tests.**

- The target tests of §9.1. Each test that is red on today's code carries `#[ignore = "red until the core step: <case>"]`; a test that is green today carries none (ticket 16 Q2).
- Tier-1 tests that the SQLite switch breaks are fixed in place.
- The storage seam contract in `tests/storage/`, with `Tx` cases on SQLite: a unit of two record writes whose second write fails through a `TxWrite` injection leaves no row (rule 14); `transaction_new_game`; `newest_game_of_world`.
- The lookup cases of `bootstrap/run_tests.rs` move into the storage suite, on both backends (ticket 11 R6).

**Guardrail rules.**

| # | Rule | Row |
|---|---|---|
| 14 | A save unit lands whole | test: the storage test above. The compiler half (record writes exist only on `Tx`) lands in C6, where the old write paths go ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q7) |

**Docs.** `storage.md`: the `Tx`, `TxWrite`, the two entries, the lookup by World. `integration_test_standards.md` ("`TestAppBuilder` defaults to `Storage::new_in_memory()`") and the tier-1 row of `tests/STRATEGY.md` (its storage column). `startup.md`, where it describes the lookup of the newest Game (inferred).

**Done when** (§2, plus): `validate_feature_spec.py` counts each `#[ignore]`d target test as the cover of its scenario (if it does not, P5 makes it do so); the `TestAppBuilder` default builds SQLite. Record the time of `python build.py integration` before and after the SQLite switch in the slice ticket (inferred risk: each tier-1 test now runs the schema migrations on a new database).

## 9. The core step

**Content.** The six areas of the design, on one branch (ticket 09):

| Area | Content | Source |
|---|---|---|
| 1. The records and their storage | `SwipeState` on the Swipe, in place of the Snapshot; `Message.kind`; `Swipe.state`, `maker`, `room_report`; the failure line on the Game; the dock record; `stories.rs`, `docks.rs`; the reads by `GameId` (`games()`, `stories()`, `docks()`); the record writes on `Tx`; `Tail`, `GenerationOrder`, `SwipeAt`, `DockRecord`, `Step`, `MessageKind`, `StoryEntry`, `TriggerRef`, `EventRecord` | sketch 25 §3.1, §7.2; target design §2 |
| 2. The writer rule and the guard | `application/generation/gate.rs`: `GameWriter` and `JobWriter`, one `Mutex<HashMap<GameId, JobEntry>>`, the job id check inside each save method, `Drop` ends the entry. `save_text` returns `SavedText`; `save_state_update` takes it by value. The guard is a private field of one small type whose only method is `hold(game, \|entries, tx\| ..)`. A helper in `gate.rs` takes the guarded data as `&mut` and leaves the locking to its caller (ticket 15 Q3). `gate.rs` has no child modules, so Rust privacy keeps `hold` inside it (ticket 12, rule 18) | sketch 25 §3.2; ticket 05; ticket 12 |
| 3. The Job and its steps | `application/pipeline/job.rs`: methods and associated functions of `Job`, not free functions ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q3): `Job::start`, `JobPlan { input, first, options }`, one runner, `generate`, the step functions (`narrate_step`, `analyse_step`, the Options step), `run_step` (sets the status name, then runs the step). `OptionsRun` keeps `AfterNarration` and `OnDemand`. The records between steps are local variables of the step functions. `Job::start` and `run_step` carry `#[instrument]` with the Game id, the job id, the kind and the step, in place of the spans of `action.rs` and `retry.rs`; a refused save and each failure line log at `warn` (plan review, 2026-10-17; inferred: keeps today's level of logging) | ticket 05; sketch 24 §2.3; target design §4 |
| 4. The reads of the views | `GameViewQuery`: the story log, the dock, the status rule (it moves from `endpoints.rs`), busy; `role_health` stays; the debug DTO reads `busy_step` | sketch 25 §3.4; target design §6 |
| 5. Create, reset and the opening | One create unit for create and reset: the Game, an empty dock record, the opening Narration with the World's opening text (which can be empty) and the starting state | sketch 25 §3.5; target design §3.2, D-25 |
| 6. The dock and notices | The dock record and its view, with the `System` templates and the CSS | sketch 25 §3.6; ticket 09 Q12 |
| HTTP | `Refusal` and its `200`/`400`/`500` mapping next to `impl IntoResponse for ApplicationError` in `http/error.rs`. `ConcurrentGeneration` (`503`) goes | research 04 §2.8; sketch 25 §7.1 |
| Deletions | Every module that sketch 25 §7.1 marks **replaced** or **deleted**: Snapshot (records, storage, mappers, schema), `GameState`, `NarrativeState`, `MessageService`, `ActionPipeline` with `PipelineRun` and its five entry files, `guard.rs`, `slot.rs`, the P4 glue; in `init_game`, everything but `resolve_game_id` (`load_game_state`, `try_persist_initial_message`); `ProcessActionResult` in `application/errors.rs`; `heal_stale_status` at boot and on switch | sketch 25 §7.1; research 04 §2.9 |

### 9.1 The target tests

These scenarios and tests exist on `main` before the core branch opens, so the core step has a fixed target (ticket 15 Q1). P5 writes them; C5 removes their `#[ignore]` (ticket 16 Q2).

| Case | Scenario | Spec file | Test |
|---|---|---|---|
| I2 | A stop at each stop point, for each kind (Narration, Impersonate, user regen, event), checked against the table of target design §4.4 | `dashboard.md` (inferred: it holds the `/status/` routes; no spec covers `/status/reset-generating` today) | one generated tier-1 test over (kind × stop point), with the §4.4 table as data (ticket 05 item 4), in `races.rs` (inferred: ticket 12 Q6 keeps `races.rs` for the race cases) |
| I3 | Stop, then a second Job; the late save of the first Job is refused | `dashboard.md` (inferred, as I2) | `races.rs` |
| I4 | Reset while a Job is held | `reset.md` | `races.rs` |
| I5 | A Job held on Game A, an Action on Game B, then a switch back to Game A | `games.md` | `races.rs` |
| I6 | Edit, delete and swipe switch while a Job is held answer "Still thinking..." | `story_log.md` (edit, delete), `swipe_switch.md` (swipe switch) | `races.rs` |
| R-59 | The preset changes between the Narration and the event Narration; the event Narration uses the new preset | `actions.md` | tier 1, with the holdable fake |

### 9.2 Other scenarios of the core step

- **A swipe switch gives back the state of that Swipe** (ticket 26 R7: today no scenario states it). In `swipe_switch.md`.
- Each scenario whose behaviour dies is deleted, and each new behaviour that a player can see gets a scenario (ticket 13 §4). The known changes:
  - The opening Narration on create and reset (target design §3.2: R-25a, R-39, R-77, R-78): `games.md`, `reset.md`.
  - The failure lines of target design §4.7 (D-18): `failure_display.md`, and `actions.md` 2.4 ("Trigger narration failed" becomes "The event narration failed."). The lines that use "turn" change; new kinds: empty text, invalid room, contradiction, missing data.
  - The status order: busy first, then the failure line, then idle (target design §2.3): `dashboard.md`.
  - Scenario 22.1 (`swipe_new.md`): a re-impersonate runs the analysis (target design §10 S-2, R-60), against its "the quantifier did not re-run" line.
  - Scenarios 12.1 and 15.1: the "(the gate is not claimed)" wording.
  - The `System` notices go (sketch 25 §3.6), with the "Entered unknown location" line and the low-confidence line.
  - A swipe switch while busy answers `200` "Still thinking...", not `503` (I6): `swipe_switch.md`.
  - The edit route `/history/:id` has no spec today; I6's edit case adds it to `story_log.md` (inferred).

### 9.3 The commits, and the red window

Each commit passes the commit gate of §2; C4 is the one commit with red tier-1 tests, and it is a commit of its own (owner, 2026-10-17). The new code goes in next to today's code first, and one commit switches the routes, so the **red window** of tier 1 is short (ticket 15 Q1; the order is inferred: the new types have no caller until C4).

| Commit | Content | Tier 1 |
|---|---|---|
| C1 | Area 1, added next to today's records: the schema migration adds the new columns and the dock table; the new domain types; `stories.rs`, `docks.rs` and the `Tx` record writes; the storage seam contract cases for the new records (SQLite). Where a shared type changes shape (`Swipe`, `Message`), today's callers get the new field in the same commit. From C1 to C5, `Swipe.state` and `Message.kind` are `Option`, and today's code writes `None`; C6 removes the `Option` ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q5) | green |
| C2 | Area 2: `GameWriter` and `JobWriter` in `gate.rs`, next to today's `GenerationGate`; the types that their signatures need (`JobPlan`, `JobRequest`, `SavedText`, `Refusal`; sketch 25 §3.2); `TxToken` in `storage/transaction.rs` and the token parameter on `Storage::transaction*` (rule 3) | green |
| C3 | Area 3: `job.rs` with the runner, `generate`, the step functions and `run_step` | green |
| C4 | **The switch.** Areas 4, 5 and 6 and the HTTP part: the handlers call `job::start` and `GameWriter` through `CurrentGame`; `GameViewQuery` reads; create and reset through `GameWriter`, called from `GameCatalogue`; the Game switch stays in `GameCatalogue` (`games.get`, then `ActiveGame::set`, with no heal; sketch 25 §3.5); stop through `GameWriter::stop`; the `Refusal` mapping; the dock templates and CSS; `wait_idle` reads the new busy state; the debug DTO; the two dead walkers re-pointed (§9.5) | **red from here** |
| C5 | **The tier-1 rework.** Tier-1 tests arrange through the public write path (ticket 26 R7 condition 2), so the hand-seeded fixtures go: `seed_main_narration_flow`, `seed_event_flow`, `seed_swipe_with_stored_inputs`, `insert_message_with_swipe`. The `TestAppBuilder` methods that seed the old state (`generation_status`, `claim_generation_slot`, `last_trigger`, `log`) change or go. The tier-1 tests that name Snapshot functions change; find them with a search of `tests/http/` for `snapshot` (at `d067420b`: `games_create.rs`, `games_switch.rs`, `swipe_new.rs`, `swipe_switch.rs`, `requires_migration/fragment.rs`). The target tests of §9.1 lose their `#[ignore]` | **green again here** |
| C6 | **The deletions**: the modules of the Deletions row above, with their unit tests; the Snapshot tables leave the schema; the P4 glue; the legacy-list entries of deleted files; the rule-13 row (§9.5); the migration deletes today's saved Games (OR-01) and makes `message_swipes.state` and `messages.kind` `NOT NULL`, and `Swipe.state` and `Message.kind` lose the `Option` ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q5; this deletes the Games in the dev database); rule 9 and the compiler half of rule 14 (§9.5) | green |
| C7 | **The docs** of §9.6 | green |


### 9.4 Tests of the core step

- **Go with their module** (ticket 26 R10; sketch 25 §7.1): the pipeline's unit tests; the tests of `message_service`, `guard`, `slot`, the Snapshot storage and mapper, `message_history`, and the message and swipe storage; `domain/model/state/game_state_snapshot_tests.rs`; `tests/storage/snapshot_storage.rs`.
- **Change with their module** (ticket 26 R9): `application/generation/gate_tests.rs` goes, because tier 1 covers `gate.rs`; `generation_status_tests.rs`, `domain/model/message_tests.rs`, `games/view_query_tests.rs` and `games/catalogue_tests.rs` change where they name a deleted type, under §2's test rules.
- **`game_state_tests.rs`**: the story-method tests go with the `GameState` type. The world-rule tests moved in P3.
- **Storage** (ticket 11, its test table): `snapshots_tests.rs`, `messages_tests.rs`, `swipes_tests.rs` and `mappers/state_snapshot_tests.rs` go. `mappers/message_tests.rs` breaks on the new fields. The override keys in `core_tests.rs` that name `save_snapshot` and `load_latest_snapshot` are rekeyed to a kept method. `games_tests.rs` and `db_tests.rs` change in part. Each broken test is rewritten into the storage seam contract in `tests/storage/`, on SQLite for the new records (ticket 15 Q4).
- **`actions_tests.rs`**: the busy test that arranges with `try_claim` breaks when `GenerationGate` goes. It goes: I1 covers the behaviour at tier 1 (ticket 12).
- **New**: the scenarios of §9.2. The status test checks that each step sets its step name (it replaces rule 17, ticket 15 Q3). A tier-1 test holds the fake at the analysis step and asserts that the story log shows the text and that the status names that step (R-11a; ticket 26 R8, in place of the order check of rule 13).
- **Placement** (ticket 26 R9): tier 1 covers `job.rs` and `gate.rs`, with no unit tests. `SwipeState`, `Tail` and `DockRecord` behaviour is tier 1; as pure `domain/model/` code they may also have plain unit tests.

### 9.5 Guardrail rules of the core step

| # | Rule | Tool | Row or type | Commit |
|---|---|---|---|---|
| 3 | Only `GameWriter` writes game records | compiler + syn | `Storage::transaction` and `Storage::transaction_new_game` take a `TxToken` (ticket 15 Q3). `TxToken` lives in `storage/transaction.rs` with a `pub(crate)` constructor, so `storage/` imports nothing from `application/` and arch-lint does not change. Call-site row: only `generation/gate.rs` and `test_support/` call `TxToken::new`; the storage seam contract gets its token from `test_support/` ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q2, which replaces the second constructor of ticket 16 Q4) | C2 |
| 4 | A running Job writes only through its `JobWriter` | compiler | private fields; only `accept` makes one; each method checks the job id. `SavedText` enforces D-03 (ticket 05) | C2 |
| 5 | Edit, delete and swipe switch only while idle | compiler + test | `GameWriter` methods that check busy under the guard; the test is I6 | C2, C4 |
| 6 | The guard never spans an LLM call | compiler + syn | the guard is a private `MutexGuard` that no function returns; banned-name row: `gate.rs` names no `Narrator` and no hook trait | C2 |
| 7 | Between steps, pass ids | syn | one type-check row on `pipeline/job.rs` with two positions: no struct field and no function parameter of type `Game`, `Message`, `SwipeState`, `Tail`, `DockRecord` or `WorldBundle`. A step takes ids, texts and step results, and loads the records it uses itself. A helper that builds an input from values lives outside `job.rs` (for example `WorldBundle::view`). The field half is ticket 15 Q3; the parameter half is [ticket 20](../../.scratch/rust-rebuild-build/issues/20-shape-of-the-job.md) Q12 | C3 (field half), R8 of the response plan (parameter half) |
| 13 | One generation path | syn | call-site row: only the step functions of `pipeline/job.rs` call `Narrator::narrate` and the `AgentRunner` hook methods | C6 (inferred: today's pipeline calls them until C6 deletes it) |
| 15 | The status has one writer per part | compiler | `JobEntry` fields are private to `gate.rs`; the failure line changes only through save units | C2 |
| 18 | One hold per `gate.rs` function (direct count) | compiler + syn | call-count row: each function of `gate.rs` makes at most one call to `hold`. The row's comment holds the re-look triggers of research 06 §5 item 4: a second lock, an atomic with weak ordering in the write path, a background Agent save that checks "this Swipe is still active" outside `hold` | C2 |
| 9 | A save writes only what changed | compiler | the `Tx` write methods take one small record each, and no `GameState` type is left | C6 ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q7) |
| 14 | A save unit lands whole (compiler half) | compiler | record writes exist only on `Tx`; the test half landed in P5 | C6 ([ticket 17](../../.scratch/platform-choice/issues/17-decide-the-plan-review-gaps.md) Q7) |

Other guardrail changes (ticket 13 §3):

- **Re-point** `handler return type` and `server layer boundaries` to `adapters/driving/http/`, and make `SwipeState` the banned type of the second. This retires the `TODO` above `check_handler_return_type` in `layers.rs` (C4).
- **Retire** `messages swipes separation`, because `messages.rs` and `swipes.rs` become `stories.rs` (C6).

### 9.6 Docs of the core step (C7)

| Doc | Change (ticket 13 §4) |
|---|---|
| `docs/diataxis/reference/game_flow.md` | Rewrite in place; 53 files anchor to it |
| `docs/diataxis/reference/storage.md` | Rewrite in place; 37 files anchor to it |
| `docs/diataxis/reference/architecture_system.md` | Rewrite, with a *who owns what* section: for each part, what it owns and what it may write. It also states the module-shape convention: an inherent impl lives in its type's file, and one file may hold more than one type (ticket 13 §5) |
| `docs/diataxis/explanation/two-state-channels.md` | Rewrite: both status channels change |
| `docs/diataxis/explanation/storage_design.md` | Rewrite: Snapshot persistence goes |
| `CONTEXT.md` | **Snapshot** goes. The terms of target design §1 enter: Swipe state, start state, current state, pre-analysis state, Job entry, save unit, dock record, event record |
| `docs/specs/*` | §9.1, §9.2 |
| `AGENTS.md`, `tests/AGENTS.md` | The hand-written prose changes where the core step makes it wrong |

New modules point their DOC anchor at the rewritten area docs (ticket 13 §4).

### 9.7 Merge of the core step

Each commit passes the commit gate of §2, so C7 has passed the full gate and the retro. The branch merges once, when the core step is **done** (§2), with no `#[ignore]` left from §9.1. A search of `src/` and `tests/` for `GameStateSnapshot`, `save_snapshot`, `load_latest_snapshot` and `snapshot_id` finds them only in the schema migration history of `storage/utils/plumbing.rs`, which keeps its old steps.

## 10. Past the build

The build map holds one `task` ticket per slice, and its Notes override "plan, don't do" (ticket 09 Q9). These efforts come after the build:

- Legacy unit tests move to the one rule when the owner decides, one module at a time (ticket 08 R5).
- A `StorageBackend` trait, only if the owner reopens it (ticket 11 R1; ticket 15 Q5); the rest of four `architecture-deepening` tickets (research 04 §2); background Agents (D-32); the before-narration and rewrite hooks (D-34).
- [Make the two path-limited guardrails check real files](../../.scratch/retro-issue-bucket/issues/50-dead-path-guardrails.md): ticket 13 answers it, and C4 delivers it. The owner changes its status.
