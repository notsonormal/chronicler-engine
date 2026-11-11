# Plan check: `docs/plans/rust-rebuild-build-plan.md`

Reviewer: reviewer:plan-check (read-only). Code at `d067420b` (HEAD, branch `dashboard-ui-review-phase3`, 8 ahead of `main`).
Labels: **known** = read in a source or in the code; **inferred** = follows from what was read, not compiled or run.
Severity: **blocker** = the slice cannot be done as written without a new decision, or its gate goes red; **should-fix** = wrong, missing or undecided, but the work can still go ahead; **nit**.

## Blockers

### B1. P1 does not say how code without a request gets its `GameId`, or whether "a switch discards the Job" stays
- **Plan:** §4 P1: "Each `Storage` read and write that used them takes a `GameId` parameter." and "The handlers that call `current_game_id()` today take `CurrentGame`." Rule 2 row: "only `current_game.rs` and `GameCatalogue` call `ActiveGame`". Scenarios: "None new".
- **Evidence (known):** 13 non-handler sites read the process-wide id inside services and the background task: `application/pipeline/action_pipeline/core.rs:146,168,222,430,514`, `retry.rs:71,187,339`, `options.rs:106`, `pipeline_run.rs:42` (the α-check `check_game_unchanged`), `message_service.rs:129`, `games/view_query.rs:41,74,175`, plus `bootstrap/run.rs:107` and `test_support/test_data_builder.rs:191` (`set_game_id`). `docs/diataxis/reference/game_flow.md:40` describes the α-check: today a switch during a Job discards its text (sketch 25 §3.2). Ticket 12 marks I5 "no (today a switch discards the Job)", a core-step target.
- **Gap (inferred):** P1 must choose one of these. (a) The pipeline takes the `GameId` from the handler, and the α-check re-reads the pointer. Under rule 2 it can only read it through `GameCatalogue`, so that is an indirect second reader. (b) The α-check goes. That changes behaviour (I5 turns green early) in a slice that claims no new scenario. P1 also does not say how bootstrap and `test_support` set the pointer (through `GameCatalogue`?), or whether building `ActiveGame` in `wiring.rs` counts as a "call" for the rule-2 row. No source decides any of this.
- **Fix:** Add a P1 item, or send it to the owner. It should say: the request captures the `GameId` and passes it down; the α-check keeps today's behaviour by reading through `GameCatalogue` (or some other choice); bootstrap and `test_support` set the pointer through `GameCatalogue`; the rule-2 row matches method calls (`get`, `set`), not construction.

### B2. The rule-3 capability token cannot be built only in `gate.rs` without breaking arch-lint's `storage → application` deny
- **Plan:** §9.5 rule 3: "`Storage::transaction` and `Storage::transaction_new_game` take a token that only `gate.rs` can build (ticket 15 Q3)". C2.
- **Sources:** ticket 11 "arch-lint: **No change**"; research 10 C11: "Storage does not import the application layer … arch-lint 0.4.3 cannot add an exception; the rule itself must change". `arch-lint.toml:164-169` (`from = "storage"`, `to = ["application"]`). Sketch 25 §6.3: "`pub(in path)` can only name an ancestor module".
- **Gap (inferred, from how Rust visibility works; not compiled):** For the compiler to stop every module except `gate.rs` from building the token, the token type must be declared in `application/generation/gate.rs` (private constructor). `Storage::transaction` must then name that type, so `storage/` imports `application/`, which arch-lint denies. A token declared in `storage/` or `domain/` can only restrict its constructor to `pub(crate)`. That is option C of ticket 16 gap 4, which the owner rejected. No ticket says where the token type lives.
- **Fix:** Send this to the owner as a decision. The two choices: change the `storage → application` deny row (and say so in ticket 11's "no arch-lint change"), or accept a crate-visible constructor plus a syn call-site row. Then state the choice in §9.5 rule 3.

### B3. The kept guardrail `free fn location` rejects `apply_world_rules` (P3) and `job::start` (C3)
- **Plan:** P3: "`domain/model/world_rules.rs`: `apply_world_rules(&mut SwipeState, ..)`". Area 3: "`application/pipeline/job.rs`: `start`, …" (sketch 25 §3.3 `pub fn start(..)`).
- **Evidence (known):** `tests/infrastructure/guardrails/free_fn.rs:1-28`. Top-level free fns may live only in `mappers`, `utils`, `builders`, `test_support`, `bootstrap` or `handlers`. The exempt list is grandfathered: "Do not any add any others to this list." Ticket 13 §3 keeps this rule ("Keep (20) … free fn location").
- **Effect:** If P3 is built as written, the guardrails step is red, so §2 "done" items 3 and 6 fail. The same happens at C3 for `start`, and for any free step function.
- **Fix:** Decide the form and write it in the plan. Options: `SwipeState::apply_world_rules` (a method), `domain/model/utils/world_rules.rs`, or `Job::start`. Send it to the owner if it should not be the implementer's choice.

## Should-fix

### S1. "Today no spec covers Triggers" is false, and the Trigger and Quantifier scenarios to write are not listed
- **Plan:** §6 P3: "Write the Trigger scenarios that a player can see … Today no spec covers Triggers." §7 P4: "The Quantifier scenarios that a player can see … `actions.md`".
- **Evidence (known):** `docs/specs/actions.md` already has Trigger scenarios 1.4, 1.6, 1.7, 1.8 and 2.4, and Quantifier scenarios 1.4, 1.6, 2.5 and 1.x (movement). Ticket 26 R5 itself cites "a Trigger (1.4, 1.8)". The "no spec" claim comes from ticket 08's Context ("seed §3"), which predates these scenarios.
- **Fix:** Replace the sentence with a list: the existing scenarios, and the gaps that P3 and P4 must add (one line each). Otherwise the implementer decides what to write and may write duplicates.

### S2. The P4 strict-fake rework lists 6 files; 10 tier-1 files run a turn
- **Plan:** P4 Tests: "The tests that post `/action`, `/swipe/new` or `/retrigger` are in `actions.rs`, `swipe_new.rs`, `retrigger.rs`, `story_log.rs`, `swipe_switch.rs` and `requires_migration/fragment.rs`."
- **Evidence (known):** `post_action` is also called in `tests/http/llm_messages.rs:25`, `narrator_mode.rs:58`, `reset.rs:13` and `options.rs:111,115` (`/options` through `post_action`). Under the strict fake, each Options and event-narrator call must also be scripted, not only the analysis answer (ticket 26 R6: "an unscripted call fails the test").
- **Fix:** List all 10 files, or say "every file that calls `post_action`, `post_empty("/swipe/new" | "/retrigger")` (grep)". Also say that every model call is scripted, not only the analysis call.

### S3. P3 "no change of logic" does not fit the world-rule methods, which also write the story
- **Plan:** P3: "`apply_world_rules(&mut SwipeState, ..)`. About 330 lines move out of `GameState` with no change of logic."
- **Evidence (known):** `game_state.rs:223-240`: `attempt_movement` calls `self.add_message("[System] Entered unknown location…", MessageType::System)`. `log_movement_completion` (`:257ff`) writes `narrative.pending_location`, `arriving_location_write` and the arriving message's header. `evaluate_triggers` and `commit_trigger_narration` (`:339ff`) write `narrative.last_trigger`, `pending_event` and add a Narration. None of these fields is in `SwipeState` (movement, scene, encounter log).
- **Fix:** Name the methods that move, and say how each story side effect stays outside `apply_world_rules` in P3: returned in an outcome value, or left in a `GameState` wrapper. This is a decision; the plan currently leaves it to the implementer.

### S4. The boot path change is not placed. P5 deletes `list_game_names_for_world` while its only caller stays
- **Plan:** P5: "`find_latest_game_for_world`, `list_game_names_for_world` and their raw SQL in `bootstrap/run.rs` go." The core-step areas never mention `init_game`.
- **Evidence (known):** `bootstrap/init_game.rs:32` calls `list_game_names_for_world` for the second creation path (`:55` `insert_game_from_request`). Ticket 11 R6: it "goes by itself, because `init_game` calls `GameCatalogue::create_game`". Research 04 §2.9 and sketch 25 §3.5: `init_game` "keeps `resolve_game_id` only"; `bundle_for`, `NewGame` and `insert_game_from_request` go. Sketch 25 §7.1 also changes `application/errors.rs` (`ProcessActionResult` → `Refusal`) and drops `heal_stale_status` at boot and on switch.
- **Fix:** In P5 (or the core step), add: "`init_game` creates a missing Game through `GameCatalogue::create_game`; `insert_game_from_request` and the second creation path go." In area 5 or the Deletions row, add `init_game`'s `load_game_state` and `try_persist_initial_message`, `ProcessActionResult`, and `heal_stale_status`.

### S5. C4 "switch through `GameWriter`" contradicts sketch 25 and the target design
- **Plan:** §9.3 C4: "create, reset and switch through `GameWriter`".
- **Sources:** Sketch 25 §3.5: `switch_game` = "`games.get`, then `ActiveGame::set`; no heal" in `GameCatalogue`. Target design §3.4: "Switch | always | the active-Game pointer only". If `GameWriter` sets the pointer, the rule-2 row ("only `current_game.rs` and `GameCatalogue`") goes red.
- **Fix:** "create and reset through `GameWriter` (from `GameCatalogue`); switch stays in `GameCatalogue` with no heal".

### S6. The core commit order has forward dependencies (C2 needs C3 types)
- **Plan:** C2 = `GameWriter`/`JobWriter`; C3 = "`job.rs` with `JobPlan` … `SavedText`; `Refusal`".
- **Sources:** Sketch 25 §3.2: `GameWriter::accept(..) -> Result<(JobWriter, JobPlan), Refusal>`. `switch_swipe`, `edit` and `delete_last` return `Refusal`. Ticket 05: `JobPlan` is "built by `GameWriter::accept`". Plan area 2 already puts `SavedText` in `gate.rs`.
- **Fix (inferred):** Move `Refusal`, `JobPlan`, `JobRequest` and `SavedText` into C2, or merge C2 and C3. Then C2 compiles on its own.

### S7. C1 adds `Swipe.state` and `Message.kind` while today's code still writes, and the plan omits OR-01
- **Plan:** C1: "Where a shared type changes shape (`Swipe`, `Message`), today's callers get the new field in the same commit".
- **Sources:** Ticket 09 Q1: no throw-away `GameState` ↔ `SwipeState` adapter. Sketch 25 §3.1: "Saved Games may be discarded (OR-01), so no migration."
- **Gap (inferred):** From C1 to C4, today's pipeline creates Swipes and Messages. C1 does not say what `state` and `kind` it gives them: an `Option` or default, or a value derived from `GameState`, which is the refused adapter. C1 also does not say what the schema migration does with existing rows (nullable or default columns? drop saved Games under OR-01?).
- **Fix:** State the field form for C1 to C5, and the C1 and C6 handling of existing saved Games, citing OR-01.

### S8. The core step's spec changes are under-listed
- **Plan:** §9.2 lists only the swipe-switch-state scenario and the opening Narration, plus a general rule (ticket 13 §4).
- **Missing, decided in the sources (known):**
  - Target design §4.7 / D-18 failure lines. `actions.md` 2.4 asserts "Trigger narration failed"; the target is "The event narration failed." Today's `utils/error.rs:44,52,53,55` "turn" lines change, and new kinds arrive (empty text, invalid room, contradiction, missing data).
  - The status order: busy first (target §2.3; research 04 §2.5).
  - Scenario 22.1's "(the quantifier did not re-run)" contradicts target §10 S-2 / R-60: a re-impersonate runs the analysis.
  - 12.1 and 15.1 say "(the gate is not claimed)".
  - `MessageType::System` notices go (sketch 25 §3.6). This includes the "Entered unknown location" line and the low-confidence line.
  - Swipe switch while busy changes from `503` to `200` "Still thinking..." (I6).
- **Fix:** List each scenario that changes or dies, with its spec file. "Nothing left to decide" applies here too.

### S9. P1 makes every handler that read the id return `400` on no Game; some must not
- **Plan:** P1: "`CurrentGame` … No active Game gives `400` (R-45)" and "The handlers that call `current_game_id()` today take `CurrentGame`."
- **Evidence (known):** `games/handlers/games.rs:73` (`current_and_saved_games`) treats "no active Game" as `Option`: it lists the saved Games. `layout/handlers/endpoints.rs:69` is the status poll. `:88` is the stop; target design §4.1 says "No Game: `400` (R-45), or 'failed' for a stop".
- **Fix:** Name which handlers take `CurrentGame` and which take `Option<CurrentGame>`. Keep "failed" for the stop.

### S10. The doc lists miss docs that become wrong, and one inferred target does not exist
- **Known:**
  - P0: "`CODING_STANDARDS.md` where it states 'a test for each branch' (inferred)". `CODING_STANDARDS.md` has no such sentence; it is `tests/STRATEGY.md:32`. Drop the item.
  - P1: `docs/diataxis/reference/coding_standards/integration_test_standards.md:46` (`current_game_id`) and `unit_test_standards.md:38,45,223` (`storage.set_game_id(1)`). Unless P0's rewrite removes these, §2 "done" item 4 (search of `docs/diataxis/`) fails in P1. Also `game_flow.md:40` (the α-check) and `storage.md:18` ("A `Storage` instance holds the active game id"). The second one is listed, but the plan labels it neither known nor inferred.
  - P4: `integration_test_standards.md:219-220` ("call sites commonly pass `AgentRegistry::default()`"). The `tests/STRATEGY.md` tier-1 row ("LLM (`MockBackend` via pipeline override)").
  - P5: `integration_test_standards.md:16` ("`TestAppBuilder` defaults to `Storage::new_in_memory()`"), and the `tests/STRATEGY.md` tier-1 row ("SQLite or in-memory storage").
- **Fix:** Add these to the slices' Docs lines.

### S11. P4 needs types that only arrive in core C1
- **Plan:** P4: "`AgentContext` (`&GameState`) becomes `HookInput` (values)"; `Narrator` with `narrate`.
- **Sources:** Sketch 24 §2.2: `HookInput { kind: Kind, start: &SwipeState, story: &[StoryEntry], world: &WorldView, event: Option<&EventRecord>, .. }`. Sketch 25 §3.3: `NarratorRequest` uses the same types. `Kind`, `StoryEntry` and `EventRecord` are area-1 types (plan §9 area 1, C1).
- **Fix (inferred):** Say whether P4 adds these value types early (they are then not throw-away) or uses interim fields, and give the P4 field lists of `HookInput` and `NarratorRequest`.

### S12. The P5 `Tx` has no named record writes, and the rule-14 test needs at least two
- **Plan:** P5: "`TxWrite`: an enum of record writes"; "a transaction that fails half-way through a `TxWrite` injection leaves no row (rule 14)". Rules 9 and 14 land in P5. §2 "done" item 3 says "green".
- **Sources:** Ticket 16 Q1: today's Game writes stay off `Tx` in P5. The new records (stories, docks) arrive in C1.
- **Gap (inferred):** P5 does not say which record writes `Tx` has (a Game insert for `transaction_new_game`? what else?). A "half-way" failure needs two writes in one unit. The compiler half of rule 14 ("record writes exist only on `Tx`") and rule 9's "no `GameState` type" only hold from C6, so they cannot be "green" in P5.
- **Fix:** List P5's `Tx` record writes. Say that P5 delivers the test half of rule 14, and that the compiler half holds from C6.

### S13. The P0 dead-path assertion may make today's two dead walkers red until C4
- **Plan:** P0 item 6: "Each path-limited row **and walker** must match at least one file under its root, else the guardrail fails." The re-point lands in C4.
- **Evidence (known):** `layers.rs:94` (`starts_with("src/server/")`) and `:203` (`starts_with("server/")`) match no file today (sketch 25 §6.3).
- **Fix:** Say that in P0 the assertion covers only the new table rows, or move the re-point into P0 (ticket 13 §3: "in the slice that rewrites those handlers"; P1 also rewrites them).

### S14. The C4 gate cannot be run as written
- **Plan:** C4 row: "`python build.py clippy` plus `python build.py test-pattern` on the tests that C4 does not break".
- **Gap (inferred):** "The tests C4 does not break" is not a pattern. The row names neither the guardrails binaries (C4 re-points two walkers and must keep rows green, §2 "done" item 3) nor `fmt`.
- **Fix:** Give the commands, for example `clippy`, `guardrails`, `test-pattern "" --target storage`, `--lib`, and `fmt`. Name what is skipped.

### S15. "Use one name" leaves the `apply_world_rules` / `apply_game_rules` choice to the implementer
- **Plan:** §1 table: "The function keeps the name `apply_world_rules` (tickets 09 and 26); sketch 24 writes `apply_game_rules`. Use one name and state it in `CONTEXT.md`".
- **Evidence (known):** `CONTEXT.md:51-53` already defines **Game rules** with "_Avoid_: World rules". The plan's own authority order puts target design D-28 above the tickets.
- **Fix:** Pick the name, either by the authority order (`apply_game_rules`, `game_rules.rs`) or by asking the owner. Put it in P3.

### S16. §2's "mechanical change in place" rule goes against ticket 09's literal rule
- **Plan:** §2: "A test in a legacy file that breaks only by a mechanical change … changes in place. Reason (inferred): ticket 15 Q2 removed the count pin".
- **Source:** Ticket 09, "Tests in each slice": "A test that the slice breaks is rewritten into its seam contract or its scenario under the one rule". Ticket 15 Q2 removed the pin but did not amend this sentence.
- **Assessment:** I agree the inference is sensible (P1 alone breaks about 250 call sites). It is still a new rule, and it governs P1's whole test cost.
- **Fix:** Get owner confirmation. Record it as a ruling, not an inference.

### S17. The test-fate lists are incomplete or stale
- **Known:**
  - §9.4 does not name `application/generation/gate_tests.rs` (gate.rs is tier 1 only), `domain/model/state/game_state_snapshot_tests.rs`, `generation_status_tests.rs`, `domain/model/message_tests.rs`, `games/view_query_tests.rs`, `games/catalogue_tests.rs`, or `tests/storage/snapshot_storage.rs` (22 Snapshot references). P4 does not name `domain/model/agent_tests.rs` (`ExecutionPhase`).
  - C5's "`tests/http/{actions, options, retrigger, swipe_new, games_switch}.rs` that name Snapshot functions" is copied from sketch 25 (commit `9cbf3112`). At `d067420b`, the tier-1 files that mention Snapshots are `games_create.rs`, `games_switch.rs`, `swipe_new.rs`, `swipe_switch.rs` and `requires_migration/fragment.rs`. Of the four seed fixtures, only `swipe_new.rs` uses one.
- **Fix:** Give each file its fate, or replace the lists with the grep that finds them.

### S18. P4: the fate of the Narrator's other test files is unclear
- **Plan:** P4: the Narrator's cases cover "`assembler`, `builders/sections`, `utils/context`, `sanitize`, `token_budget`", yet only "`assembler_tests.rs` breaks and becomes the Narrator seam contract".
- **Gap (inferred):** Ticket 26 R4 needs one tagged test per declared case in the box's test file. So either the other four test files move into the contract, or the cases the Narrator declares are limited to what `assembler_tests` covers. Ticket 08 R6 counts "about 1,260" `prompting/` lines as going to the Narrator contract.
- **Fix:** Say which of the four files move and which stay legacy.

### S19. Two "Done when" checks cannot be run as written
- **Known:** P4: "a search of `src/` finds no `[System] NPC detection uncertain` line reached from tier 1". The string exists at `pipeline_run.rs:174`, and a search cannot see what tier 1 reaches. P3: "`GameState` holds no world-rule method; each one is a call into `apply_world_rules`" contradicts itself.
- **Fix:** For P4: "no tier-1 test's story log contains that line" (or a support assertion). For P3: "`GameState` methods only delegate", or "callers call `apply_world_rules` directly".

## Nits

- **N1.** Some placements are unlabelled. I2 is placed "in `races.rs`", but ticket 12 lists I2 as "the generated stop test" and keeps `races.rs` for I1 and I3 to I6 (Q6). R-59 is placed in `actions.md` "with the holdable fake", and I4 to I6 are given spec files. These follow from ticket 12 Q6's "spec file of their route" rule; mark them inferred. Note: the edit route `/history/:id` has no spec today. `story_log.md` lists only `/history/delete`.
- **N2.** P5: "The token comes with `gate.rs` in the core step (inferred: ticket 13 …)". Ticket 16's Answer decides this (gap 4 → "core C2"). Cite it as known.
- **N3.** Research 04 §2.4: `build_narration_prompt`'s only production caller is arrival, so it "goes with it" (P2). The plan removes it in P4.
- **N4.** `shape-at-a-glance.md` §1 still says "Four hook traits", against D-34. Add a row to the §1 table.
- **N5.** P0 says "the flag form is the implementer's choice", yet "Done when" uses `--target http`. Fix the flag. `build.py` already has `--target-dir`, and argparse matches prefixes.
- **N6.** The rule-1 row bans "`static`" in `storage/`. `core.rs:34,56,69` and `test_support.rs:22,26,33` use `&'static str`, and syn's visitor visits the lifetime's ident. Say "`static` items" (inferred risk).
- **N7.** `python build.py --include-llm` adds `--run-ignored all` (`build.py:286,305`), so between P5 and C5 it runs the red target tests. Mention it next to "Tests marked `#[ignore]` … do not run".
- **N8.** Two omitted items. Ticket 12 rule 18: "`gate.rs` has no child modules", which is what makes the compiler half hold. Ticket 13 §5: the one-type-per-file convention "is stated in the reference doc".
- **N9.** `layers.rs` has two TODOs (`:5`, `:91`). Ticket 13 §6 means `:91`.
- **N10.** §10 lists "A `StorageBackend` trait" as an effort after the build. Ticket 15 Q5 rejected it, and ticket 11 R1 calls it out of scope. Reword it as "if the owner reopens it".
- **N11.** §9.4 says "`actions_tests.rs` busy test … goes, or it moves". I1 covers that behaviour at tier 1, so under the one rule it goes. Resolve the "or".
- **N12.** §2 "done" item 3 says "in its walker table". Rules 11 and 12 are arch-lint rows, and 4, 9, 15 and 16 are compiler rules.
- **N13.** The P0 allowlist guardrail does not state ticket 26 R2 clause 3 ("calls only that box's entry point"), or where the legacy list lives and which tool checks it (ticket 08 R4/R7: "the call-site guardrail").
- **N14.** §9.2 cites "(R-76)" for the opening Narration. Target §3.2 cites R-25a, R-39, R-77 and R-78.
- **N15.** Sketch 24's `AnalysisOutput::Tracker` and `SwipeState.trackers`: in or out under D-34? Say which.
- **N16.** C3 lists `SavedText`, but area 2 and C2 already have `save_text` return it (see S6).

## Checked and found correct

- **Plan header and §1.** HEAD is `d067420b`. `dashboard-ui-review-phase3` is checked out and 8 commits ahead. The authority order, and the amendments by tickets 15 and 16, match those tickets. The §1 supersession rows match: rule 3 names `transaction` and `transaction_new_game` (ticket 11 R7, 15 Q3, 16 Q4); D-34 limits the hooks to two; ticket 05 gives step functions, `JobPlan` and `SavedText`; D-29 and sketch 24 put the narrator in `narration/`; rules 8, 10 and 17 are removed and R-59 replaces rule 8 (15 Q3); the `Tx` is SQLite only (15 Q4); the one rule replaces ticket 08 R2 and R4 (15 Q2).
- **§2.**
  - The one rule's three places and its list of seam boxes match ticket 08 R2 and 15 Q2.
  - The rule index (P1: 1, 2; P3: 12; P4: 11, 16; P5: 9, 14; core: the remaining 8) adds up to 15 and matches ticket 13 §1 plus 15 Q3 and 16 Q5.
  - The commit-gate rows match ticket 14's owner comment and ticket 09's last amendment: retro once per slice and once at the core end; C4 is red on its own with `clippy` plus the tests it does not break; C7 skips the comment cleanup and the code review; an empty input skips a step. They are consistent with the 6 steps of `simple-after-plan`.
  - The dev-loop trade-off matches ticket 07.
- **P0.** Ticket 07's numbers (5.0 to 5.6 s against 13.16 s). The test target `http` exists in `Cargo.toml`. The checker spec matches ticket 26 R4. The walker kinds, the dead-path rule, the generator form, the `registry_tests.rs` rework and "today's 23 rules stay" match ticket 13 §2. `requires_migration` has its own pin (`validate_feature_spec.py:65-78`). The `spec-coverage` step exists in `build.py`.
- **P1.**
  - `Storage.game_id: AtomicU64`, `set_game_id` (with the `updated_at` touch at `core.rs:156-179`), `current_game_id` and `game_id()` exist. `new_sqlite(pool, game_id)` exists.
  - The count is 257 matches in 53 files, against "about 240 in 50".
  - Scenarios 11.2 and 14.6 are the R-45 `400` scenarios.
  - `core_tests.rs:5-27` holds the game-id tests.
- **P2.** `arrival_service.rs` and its tests exist. `ArrivalSpawnRequest` and `spawn_arrival_task_if_needed` are in `init_game.rs`, with the early return at `:137-140`, and are called at `run.rs:140`. No spec or test names the arrival opening. `startup.md` and `narration_system.md` do not mention it.
- **P3.** `proptest = "1.6"` is a dev-dependency. `game_state.rs` is 553 lines and `game_state_tests.rs` is 1,840. `domain/` imports no storage, so the rule-12 row is green at once. arch-lint already has the `model` scope.
- **P4.**
  - `ExecutionPhase`, `AgentResult`, `AgentContext`, `StatePatch`, `Confidence`, `Role` and `AgentConfig` are in `domain/model/agent.rs`.
  - `from_config_with_storage` is at `quantifier/agent.rs:34` and `options/agent.rs:33`. `PromptAssembler::with_storage` is at `assembler.rs:72`, `build_narration_prompt` at `:181`, and `call_narrator` at `pipeline_run.rs:95`.
  - `AgentRegistry::default()` is used at `test_support/context.rs:140,154` and `tests/http/support/app_wiring.rs:50`.
  - `try_claim` is at `swipe_new.rs:1265` and `retrigger.rs:307`. `wait_idle` is at `http_requests.rs:110`.
  - The doc paths exist. The I1, 12.1, 15.1, `races.rs` and `wait_idle` notes match ticket 12. The strict and holdable fake matches 26 R6 and 16 Q3.
- **P5.**
  - `BackendKind::{Direct, Test}`, `with_backend_mut`, `with_failure`, `add_failure`, `with_test_failures` and `TestOverride` exist.
  - The `TestAppBuilder` default is `Storage::new_in_memory()` (`test_app_builder.rs:137`). 15 tier-1 files build in-memory storage themselves.
  - `find_latest_game_for_world` and `list_game_names_for_world` are at `run.rs:168,199`. The lookup tests are in `run_tests.rs`.
  - C2, C4, C7, C8, C9, C10, C14 and C15 are cited correctly (research 10 §5). Ticket 16 Q1 and Q2 are reflected.
- **Core.**
  - The areas match sketch 25 §3, ticket 05 and ticket 12 (rule 18 with the direct count, the helper convention from 15 Q3, the re-look triggers).
  - `ConcurrentGeneration` → `503` is at `http/error.rs:15`. `impl IntoResponse for ApplicationError` is at `:11`. No spec covers `/status/reset-generating`, and `dashboard.md` holds the `/status/generating` scenarios.
  - The seed fixtures and the `TestAppBuilder` methods `generation_status`, `claim_generation_slot`, `last_trigger` and `log` exist.
  - `messages swipes separation` targets `storage/messages.rs`. `handler return type` is green once re-pointed (no `(StatusCode, String)` under `http/`).
  - `game_flow.md` has 53 DOC anchors and `storage.md` has 37.
  - The `CONTEXT.md` term list matches target design §1. The rule rows 3, 4, 5, 6, 7, 13, 15 and 18 match tickets 13, 15 and 16. Plumbing uses versioned migrations (`user_version`), so "keeps its old steps" holds.
