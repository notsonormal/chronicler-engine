# P3: `SwipeState` and the game rules

Type: task
Status: resolved
Blocked by: 03

## Question

Build §6 of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) on branch `wf/rebuild-p3`, from the current `main`. Read §1, §2 and §6 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §6 pass. The owner has approved the last commit, and the branch has merged to `main`.

## Comments

**2026-10-17, implementer (P3 built, uncommitted).** Branch `wf/rebuild-p3`, base `487ea179` (P2 merged).

Facts measured.

| Item | Result |
|---|---|
| `src/domain/model/state/game_state.rs` | 553 → 326 lines (317 removed, 90 added). The story methods and four rule wrappers stay |
| `src/domain/model/utils/game_rules.rs` (new) | 385 lines: `apply_game_rules`, `GameRules`, `GameRulesOutcome`, `TriggerMatch`, the rule helpers, `walk`, `evaluate_triggers`, `assert_state_consistency` |
| `src/domain/model/state/swipe_state.rs` (new) | 44 lines: `SwipeState { movement, scene, npc_encounter_log }` and `init_scenario_npcs` |
| `game_state_tests.rs` | 1840 → 951 lines |
| `domain/model/utils/game_rules_tests.rs` (new) | 1091 lines, 39 tests (34 plain, 5 property tests) |
| Unit tests (lib binary) | 69 in `game_state_tests.rs` → 73 across the two files (+3 rule tests, +1 property test; no assertion dropped) |
| Snapshot format | unchanged (`GameStateSnapshot` keeps its flat fields; `from_game_state` reads `state.swipe.*`); `game_state_snapshot_tests.rs`, `mappers/state_snapshot_tests.rs` and `tests/storage` pass |
| `arch-lint.toml` | the rule-12 block: `model` → `storage`, with the reason in the comment above it |

Full gate: `python build.py` green (`logs/build_20261017_191116.log`); clippy green after the last edit (`logs/build_20261017_191614.log`); `guardrails` 187 passed; `integration` 1629 passed, 2 skipped; `browser` 29 passed.

Deviations from §6, all inside its Done-when.

1. **The outcome shape.** `GameRulesOutcome { room_report, unknown_location, trigger_match, trigger_committed }`. The plan's placeholder triple (movement result, unknown location, matched Trigger) has no room for the two no-op answers the wrappers need: a movement with no destination, and a Trigger narration with an empty text (`trigger_committed`).
2. **One entry, four requests.** `apply_game_rules(state, GameRules, map, npcs)` dispatches `Movement`, `Analysis`, `NpcEvents` and `TriggerNarration`. Each kept rule wrapper calls it once and writes the story side effects from the outcome.
3. **Methods not kept.** `attempt_movement`, `update_npc_encounters_on_room_change` and `log_movement_completion` are folded into the `Movement` rule (their only caller was `handle_movement`). `attempt_semantic_walk`, `evaluate_triggers` and `assert_state_consistency` are no longer `GameState` methods: the rules module owns `walk`, `evaluate_triggers` and `assert_state_consistency`, and the tests that called the old methods moved with them. No production caller was dropped.
4. **`init_scenario_npcs`** stays a `GameState` method that delegates to `SwipeState::init_scenario_npcs`, next to the type's own data rather than in the rules, because it seeds the start state and reads no analysis. (The core step's `SwipeState::starting` takes its place.)
5. **The tests that moved.** A test moved when its subject moved to the rules (movement, Character events, Triggers, the walk, `evaluate_triggers`, the consistency assertion); a test whose subject is the story translation of a surviving wrapper (`[System] Entered unknown location`, the room report on the Narration, `last_trigger`/`pending_event`, the event header) stays in `game_state_tests.rs`, unchanged or with the mechanical `state.swipe.*` path.
6. **The docs item.** `game_flow.md` names no `GameState` rule method, so §6's inference was empty; the slice adds a `## Game Rules` section that names `apply_game_rules` and the module, and the two side effects the story writes.
7. **No new scenario.** Every player-visible clause §6 names already has one: movement 1.14, the Characters present and the encounter record 1.6, one-shot and repeat Triggers 1.7 and 1.8, the contradiction 2.1. `validate_feature_spec.py` is green.

**2026-10-17, implementer (review fix pass, uncommitted).** The review findings of the P3 fix pass are applied. The facts below supersede the names and the API of the comment above.

Facts measured after the fix pass.

| Item | Result |
|---|---|
| `src/domain/model/utils/game_rules.rs` | 385 → 354 lines. `apply_game_rules(state, GameRuleRequest, map, npcs) -> Result<GameRulesOutcome>`; the only public items are `apply_game_rules`, `GameRuleRequest`, `GameRulesOutcome` and `TriggerMatch`. `walk`, `evaluate_triggers`, `movement`, `attempt_movement`, `apply_npc_events`, `trigger_narration` and `room_of` are private; `assert_state_consistency` is `pub(crate)` for the Trigger wrapper |
| `src/domain/model/state/game_state.rs` | 326 → 314 lines. 3 rule wrappers: `apply_npc_events`, `commit_trigger_narration`, `execute_freeaction_impl` |
| `game_rules_tests.rs` | 1062 → 876 lines, 39 → 34 tests (27 plain, 7 property tests) |
| `game_state_tests.rs` | 951 → 979 lines, 34 → 33 tests (31 plain, 2 property tests) |
| lib binary | 1631 → 1625 tests run (−6) |

The renames: `GameRules` → `GameRuleRequest`; `GameRulesOutcome::room_report` → `arrived_room_name`; `GameRulesOutcome::trigger_committed` → `trigger_narration_landed`. `docs/diataxis/reference/game_flow.md` names none of the three, so it does not change.

The review items, per number.

1. **Order restored.** On `main` the story writes of a Trigger narration land before `assert_state_consistency`, so `pipeline_run.rs` hands `set_error` a state that holds the event Narration. The `TriggerNarration` request no longer runs the check: `commit_trigger_narration` writes `last_trigger`, `pending_event` and the Narration, then runs `assert_state_consistency(&self.swipe, map, npcs)`. A new test pins it: `test_a_failed_trigger_consistency_check_keeps_the_event_narration` (a Trigger whose Character is absent from the cast makes the check fail; the history must hold the event Narration). The other wrappers' error paths were checked: `apply_npc_events` writes no story before its check, and `execute_freeaction_impl`'s `Err` arm in `core.rs:318` passes `None` to `finalize_phase_error` and drops the state, so its write order is not observable.
2. **`GameState::handle_movement` and the `Movement` request are gone.** Its story tests now run through `execute_freeaction_impl` with an `Analysis` request that moves (`make_quantifier_result_moving_to`): the room report on the arriving Narration, the redo Swipe, the queued `ArrivingLocationWrite`, and the room the map does not know.
3. **Tests go through the entry only.** `walk`, `evaluate_triggers` are private and `assert_state_consistency` is `pub(crate)`; the `test_walk_*`, `evaluate_triggers` and consistency tests were rewritten against `apply_game_rules` (outcome fields and `SwipeState`). `walk` returns `Result<()>`: nothing read its `You go to: …` line. `test_mutation_order_violation_demo` is deleted; `test_state_mutation_order` covers the order.
4. **The two tautological properties are deleted** (`prop_npcs_in_area_are_always_known`, `prop_npc_encounter_log_references_valid_npcs`).
5. **The property set** (7, from generated inputs: the starting `SwipeState` with the Characters present and their times-met counts, the cast, and the Triggers): the step keeps the state consistent (its `Err` is the check) and the scene is what the analysis named; the Character events keep it consistent; the step reads times-met before the events advance it (every case asserts); a Trigger never breaks the state (one-shot vs repeat, fired or not, room-limited or not, three requirement shapes); a one-shot Trigger fires at most once; a room-limited Trigger fires only in its room; Characters that leave are removed. The redundant re-assert after `.unwrap()` is gone, and the case count stays at proptest's default: `test-pattern "game_rules" --test-target lib` runs in 0.23 s.
6. **Renames applied**, docs unaffected.
7. **One room lookup** (`room_of`: the map, then the added rooms) serves `walk`, `room_name` and `assert_room_exists`. The duplicate check was removed from `movement` rather than from `analysis`: main's `execute_freeaction_impl` asserted after `handle_movement`, which asserted only when a destination existed, so `analysis` must keep the single call to check both cases.
8. **"engine commit"** → "game rules" in `narration_generation.rs:31` and `test_support/fixtures.rs:751` (comments). `core.rs:322` is left as it is and reported: `label: "engine commit"` is player-visible, because `PipelineRun::save_failure` renders it as `Failed to save engine commit: …` in the failure line. No spec or test asserts it.
9. **The two reasons are restored**: `write_room_report` says why the room name is written back onto the Narration, and `app_with_an_event_that_meets_gabriella` says that its `u64` is the event Message's id.

Checks: `clippy` green (`logs/build_20261017_193947.log`), `guardrails` 187 passed (`logs/build_20261017_194023.log`), `test-pattern "game_rules" --test-target lib` 34 passed (`logs/build_20261017_193907.log`), `test-pattern "game_state" --test-target lib` 39 passed (`logs/build_20261017_193959.log`), full `python build.py` green (`logs/build_20261017_194044.log`: integration 1623 passed + 2 skipped, browser 29 passed).

**2026-10-17, lead.** The failure-line label `"engine commit"` in `action_pipeline/core.rs` is player-visible ("Failed to save engine commit: …") and no spec asserts it. It stays: the core step rewrites the failure lines (target design §4.7). Retro: the lead's brief gave `test-pattern "a|b"`, but `test-pattern` matches a substring, not a regex; the `AGENTS.md` Development Loop row now says so.

## Answer

P3 is built and merged to `main`. `SwipeState` (`domain/model/state/swipe_state.rs`) holds the movement, the scene and the encounter log; `apply_game_rules(&mut SwipeState, GameRuleRequest, ..) -> Result<GameRulesOutcome>` in `domain/model/utils/game_rules.rs` holds the game rules. `GameState` keeps thin wrappers that write the story from the outcome. Rule 12 is an arch-lint row (`model` → `storage` denied). Full gate green: `logs/build_20261017_194518.log`.
