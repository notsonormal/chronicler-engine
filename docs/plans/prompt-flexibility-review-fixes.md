# Plan: Fix the review findings of the `prompt-flexibility` branch

**Date:** 2026-11-01
**Status:** Open. Plan review done, and the owner answered its two questions (2026-11-01).
**Goal:** Each finding of the after-plan review of `prompt-flexibility` is fixed, or kept with a written reason. A check proves each fix.

## Origin

The after-plan review ran on `92949b8e..6cf23131` (2026-11-01). Four reviews answered: `thermo`, `code-review`, `antipattern` and `test-police`. Their findings files are `tmp/review/pf-findings-{thermo,code-review,antipattern,test-police}.md`. The full gate passed on `6cf23131` (`logs/build_20261101_151830.log`). The owner asked for a plan that fixes all findings and checks each fix.

## Branch

Do the work on `prompt-flexibility`, before it merges into `main`. The map `.scratch/prompt-flexibility/` closes at that merge. The owner approves each commit. Make one commit for each phase.

## Decisions (agent, open to change)

- **D1. The slot key stays `PresetType`.** Ticket 21 chose `connection_for(PresetType)`. A `ConnectionSlot` enum would map one to one to `PresetType`. One slot table in `AppSettings` removes the copies.
- **D2. The movement rule: fix the doc, not the code.** `QuantifierReading::into_result` reads only `destination`. A code change alters gameplay, so it needs its own ticket.
- **D3. The history cut keeps two size units.** One unit moves the cut points. That needs a new trial. Phase D changes the structure only.
- **D4. The dangling-slot Job failure stays.** The owner chose it on 2026-11-01 (`856e36a5`). A test pins it (step A3).
- **D5. The Impersonate call keeps the agent name `narrator`.** `GameViewQuery::role_health` counts the rows by agent name. A new name would move the Impersonate failures out of the Narrator health. The map's watch item "A Role for Impersonate" owns that change. So `trial.py` keeps its call-kind heuristic, with a comment that names this reason.
- **D6. `AgentSettings.option_count` stays.** Both reviews accept it at this size. A second Options-only setting is the trigger for an own settings type.
- **D7. An unknown 5h reset time is "no data" in `delegate`.** Today `pickModel` treats a missing `fiveReset` as an ended window and allows the spare model. A missing `sevenReset` already gives the base model. After the change, both give the base model. `rate-limit-wait` already treats an unknown reset as "retry in 5 minutes", so `decide` does not change.
- **D8. The real wiring of the slots is tested through the production graph.** Tier-1 apps give one `ScriptedModel` recorder to every agent, so they do not see `bootstrap/wiring.rs`. `app_with_production_graph` (`tests/http/support/app_wiring.rs`) boots `build_app_graph` with Mock Connections. Each stored LLM message records `model_name`. So a tier-1 test can show which Connection a call used. This replaces the test-only accessor `Narrator::impersonate_recorder()`.
- **D9. The Narrator case `narrator-history-cut-holds-between-blocks` stays.** Each box declares its own cases (`tests/STRATEGY.md`, "The one rule"). The Narrator case shows that the Narrator uses the cut. `test_one_more_message_after_a_cut_keeps_the_same_start` shows the rule of the cut. Ticket 20 declared both.
- **D10. The two `map_err` arms in `presets.rs` stay.** `serde_json::to_string` cannot fail on these values, so no test reaches the arms. The guardrails forbid `unwrap` and `expect` in production (`clippy::unwrap_used`, arch-lint `no-unwrap-expect`). So `map_err` is the allowed form, and it matches the `allowed_modes` arm above it.

- **D11. `scripts/trial.py` stays one file (owner, 2026-11-01).** No rule asks for a split: the `file length` guardrail allows 2000 non-blank lines. F2 and F3 fix the coupling that the review named. The `layout-trials` map runs `scripts/trial.py run` next, so less churn is safer.
- **D12. The preset seed keeps its policies (owner, 2026-11-01).** A bad `allowed_modes` warns and uses the default, because the user owns `allowed_modes` after the first write. A bad `parameters` or `option_count` skips the file.

## Checks

Each step has a **Verify** line. A test that a step adds must fail on the code before the fix. When the fix is already in the code, run a mutation check: break the code on purpose, see the test fail, then restore the code. Each phase ends with `python build.py clippy` and the test patterns of its steps. The plan ends with the full `python build.py`.

Each test under `tests/http/` needs a `// [docs/specs/<spec>.md] SCENARIO: X.Y` tag, and the scenario must exist in the spec (`scripts/validate_feature_spec.py`). Each step that adds such a test names its spec. Write the scenario first.

`build.py` does not run the TypeScript tests of `.pi/extensions/`. Run them with `node --test <file>` in phases A and H.

## Phase A: Tests that pin the behaviour of today

These tests pass on the current code. They protect the refactors of phases B, C and D. Each one gets a mutation check.

| Step | Finding | Change | Verify |
|---|---|---|---|
| A1 | No test covers how the Job builds the call settings (`code-review` S1, P1) | Add one tier-1 scenario to `docs/specs/prompt_presets.md` and a test in `tests/http/`. Seed a system preset and a Quantifier preset, each with `max_tokens` 900 and `temperature` 0.3. Seed the Narration Connection with `max_tokens_ceiling` 700. Point `options_connection_id` at a second Connection with `max_tokens_ceiling` 500. Play one Action, then `/options`. Assert `model.calls()`: the Narration and Quantifier calls carry `max_tokens` 700, `temperature` 0.3 and the `GameId`. The Options call carries `max_tokens` 500 | The test passes. Mutation 1: `max_tokens_for` returns the preset value. Mutation 2: `CallSettings::of` sets `game: None`. Mutation 3: `hook_settings` reads `PresetType::System`. The test fails each time |
| A2 | No test drives the Options Connection slot through the wiring (`test-police` P1) | Add two scenarios to `docs/specs/settings.md`, next to 20.53, and their tests. Use `app_with_production_graph` (D8). Seed two Mock Connections, the second with the model `slot-model`. Test 1: the Options slot names the second Connection. Play one Action, then `/options`. Test 2: the Impersonate slot names it. Post `/impersonate hello`. Each test reads `GameViewQuery::list_latest_llm_messages` and asserts the `model_name` of the call | Both tests pass. Mutation: the Options and the Impersonate recorders resolve `PresetType::System`. Each test fails |
| A3 | The dangling-slot branch at `job.rs:471` has no test (`test-police` P2-1) | Add one scenario to `docs/specs/options.md` and its test. After boot, save settings through `Storage` whose Options slot names no stored Connection. Post `/options`. Assert that the Job ends with the Settings read failure, read through `status_error_message` as the missing-record tests do | The test passes. Mutation: replace the `?` with a fallback to the Narration Connection. The test fails |
| A4 | No test pins the session id across restarts (`code-review`, spec P2) | In `chat-completions-openrouter-game-session`, assert that `GameId(7)` gives `7d30fc87-b5d6-50d4-a2a9-4e5d581dfeaf`. This value was computed again from `GAME_SESSION_NAMESPACE` on 2026-11-01 | Mutation: change one byte of `GAME_SESSION_NAMESPACE`. The test fails |
| A5 | Three new branches have no test (`test-police` P2-2, P2-3, P2-4) | (a) In `test_ensure_presets_skips_a_refused_seed_file_and_seeds_the_others`, add the seed file `{"id": "bad_count_type", "name": "Bad", "option_count": "two"}`. (b) In `tests/storage/preset_storage.rs`, write a row with `option_count = -1` through the raw-insert pattern of that file. Assert an `EngineError::Parse` that names the count. (c) Add the Narrator seam case `narrator-history-system-sender`: a `MessageType::System` row renders as `System: <text>` | Each test passes. Mutation: remove the guard of each branch. Each test fails. `python build.py guardrails` passes the `seam-cases` check |
| A6 | `decide()` and `stateFromHeaders` have no test (`code-review` S7) | Add `.pi/extensions/rate-limit-wait/state.test.ts`. Add one case for each of the five branches of `decide`, and one header-parse case | `node --test .pi/extensions/rate-limit-wait/state.test.ts` passes. Mutation: swap two branches of `decide`. A case fails |

## Phase B: The Connection slot rule in one place

Finding: `thermo` P1-1, `antipattern` 1, 3 and 10, `code-review` S5.

| Step | Change | Verify |
|---|---|---|
| B1 | In `src/domain/model/settings.rs`, add one slot table. Each row holds the `PresetType`, the field name, the id or `None` for an empty optional slot, and the `Role` that a refused delete names (`System` gives `Role::Narrator`, `Quantifier` gives `Role::Quantifier`). Derive `connection_for`, `check_connection_slots` and the clear of the optional slots from it | The `settings_tests.rs` cases pass with no assertion change |
| B2 | Add `AppSettings::remove_connection(&mut self, id) -> Result<(), RemoveConnectionError>`. It checks "exists", "not the last" and "no required slot names it" first. Only then does it clear the optional slots and remove the Connection. The error names the Roles in use. `delete_connection_handler` becomes one `update_settings` call, and it keeps only the words (`role_in_use_message`). Delete `roles_using` | New `settings_tests.rs` case: a refused remove leaves the whole `AppSettings` unchanged. Mutation: clear the optional slots before the check. The case fails. Scenario 20.53 and `test_delete_connection_used_by_a_role_is_refused` pass |
| B3 | In `bootstrap/wiring.rs`, change `recorder_with_storage` to take a `PresetType`. Use it in `build_app_graph` and in `build_app_graph_for_tests`. Delete `options_recorder`, `impersonate_recorder`, `AppSettings::narration_connection`, `AppSettings::quantifier_connection`, `get_narration_connection` and `get_quantifier_connection`. Move their test callers to `connection_for`: `http/mod_tests.rs` (1), `settings_tests.rs` (4) and `load_tests.rs` (2) | `rg "get_(narration\|quantifier)_connection\|fn (options\|impersonate)_recorder\|\.(narration\|quantifier)_connection\(\)" src tests` finds nothing. A2 passes |
| B4 | In `Narrator`, choose the recorder with one private `recorder_for(PresetType)`. Delete the accessor `impersonate_recorder()`. Delete the two `wiring_tests.rs` cases that call it. A2 test 2 replaces them (D8). Keep `a_dangling_impersonate_slot_fails_the_boot_and_names_the_id` | `rg "impersonate_recorder\(\)" src` finds nothing. A2 passes |
| B5 | In `scripts/validate_data.py`, keep the four slot keys in one constant. In `scripts/trial.py`, add one `slot_connection(settings, setup, slot)` that holds the "empty slot uses Narration" rule. Both get a comment that names `AppSettings::connection_for` | The Python tests pass. `rg "stored_slot\(" scripts` finds only the call in `slot_connection` |

## Phase C: One response length

Finding: `thermo` P1-2 and P1-5, `antipattern` 2, 4, 6 and 7.

| Step | Change | Verify |
|---|---|---|
| C1 | `TokenBudget.max_tokens` becomes `u32`. `TokenBudget::fallback()` uses `DEFAULT_MAX_TOKENS`. The test sites are the 8 `TokenBudget {` literals | The Narrator seam cases pass |
| C2 | `input_limit`, `history_space` and `fit_messages_to_context` become methods of `TokenBudget`: `input_limit()`, `history_space(system, fixed_user)` and `response_tokens(system, user)`. The last name replaces `fit_messages_to_context`, which no longer fits the messages. `context.rs` then holds nothing, so delete it | `rg "fit_messages_to_context\|utils::context" src` finds nothing. The `narrator-history-*` cases and `history_cut_tests.rs` pass |
| C3 | `CallSettings.max_tokens` becomes `u32`. A manual `impl Default for CallSettings` uses `DEFAULT_MAX_TOKENS`, so the 62 `CallSettings::default()` sites still compile. The 9 `CallSettings {` literals drop their `Some` | `rg "unwrap_or\(DEFAULT_MAX_TOKENS\)" src` finds only `max_tokens_for` (after C6) |
| C4 | Add one Job helper `call_settings(&settings, preset_type, &preset) -> Result<(CallSettings, u32), JobFailure>`. The `u32` is the context window. `budget` and `hook_settings` both use it. The helper takes the `PresetType` of the call, not `preset.preset_type`. A Game stores one preset id for each type, and no check ties that id to a preset of the same type | A1 and A3 pass |
| C5 | `Narrator::narrate` takes the `CallSettings` and the context window, not a `TokenBudget` and a `GameId`. It sends `CallSettings { max_tokens: assembled.max_tokens, ..call }`. The callers are `job.rs` and about 8 sites in `narrator_tests.rs` | The Narrator seam cases and A1 pass |
| C6 | `ChatCompletionRequest` holds `call: &CallSettings` and `session_id`, not four copied fields. Delete `resolved_max_tokens`. Its callers are `build_request_payload` and `client.rs`. Delete the `OnceLock` helper `no_custom_parameters` of `request_tests.rs` | The `transport-*` and `chat-completions-*` seam cases pass |
| C7 | Split `REFUSED_CUSTOM_KEYS` into `ENGINE_KEYS` (`model`, `messages`, `stream`, `max_tokens`) and `TYPED_KEYS` (`temperature`, `top_p`). `refused_custom_key` refuses both. `transport_without_samplers_sends_only_the_engine_keys` asserts against `ENGINE_KEYS`, not a literal. The seam case `transport-engine-keys-win-over-custom` already shows that the engine value wins, so no new transport case | A new `prompt_preset_tests.rs` case refuses `temperature` in `custom`. It fails before the change. No preset in `data/` has a `custom` object (checked 2026-11-01) |

## Phase D: The structure of `render_and_fit`

Finding: `thermo` P1-3, `antipattern` 8. D3 keeps the behaviour.

| Step | Change | Verify |
|---|---|---|
| D1 | `render_and_fit` takes the whole `TokenBudget`. Its one caller is `PromptAssembler::assemble` | Compiles |
| D2 | Replace `join_user_layers(.., with_tail: bool)` with `fixed_user_layers()` and `tail_layers()`, and one join | The Narrator seam cases pass with no assertion change |
| D3 | Extract `fn cut_history(&self, entries, budget, system_tokens) -> HistoryCut`. It holds the token cut and the character guard. `render_and_fit` then reads: render the system part, cut, render the user part, fit | The `narrator-history-*` cases pass. `git diff` of `narrator_tests.rs` in this phase shows no assertion change |
| D4 | `render_guide_layer` uses `render_non_blank_layer` for its blank check. It keeps its fixed sentence in front of the guide text | The guide seam cases pass |

## Phase E: The other code fixes

| Step | Finding | Change | Verify |
|---|---|---|---|
| E1 | `agent_system.md` states a false movement rule (`code-review`, spec P1) | Write the rule that the code has (D2): `destination` names the Room the player moves to. The parser keeps `type`, and the game rules do not read it | `python build.py validate-docs` and `vale` pass. `test_parse_response_json_with_movement_null_type` agrees with the new sentence |
| E2 | The Options user prompt renders `{{option_count}}` as 3 (`code-review` S3) | Build the `TemplateVars` once in the Options prompt builder, with the count applied. Use them in both halves. Do the same in the Quantifier builder, which has no count | New Options seam case: a card with `{{option_count}}` and a preset count of 5 renders 5. It fails before the change |
| E3 | Two code paths keep the Characters present (`thermo`, `antipattern` 5) | `static_character_result` takes its ids from `QuantifierResult::keeping_present(world.present).present`. It keeps its movement and its confidence | `rg "world.present.iter\(\).map" src/application/agents/quantifier` finds nothing. Scenario 2.5 and the Quantifier failure cases pass |
| E4 | False comment in `response.rs`, above the `finish_reason` read (`code-review` S4) | Write: "Every other reason is an empty answer, and the reason goes into the failure text." | Read the diff |
| E5 | `HistoryBlock.arrival` has no reader (`thermo` P2) | Delete `arrival`. Keep `dropped`, because ticket 16 needs the cut to know what it dropped | `history_cut_tests.rs` passes after its `arrival` assertions go |

## Phase F: The trial script

Finding: `thermo` P1-4, `antipattern` 13, `code-review` S6 and S8.

| Step | Change | Verify |
|---|---|---|
| F2 | Extract `export_pack(folder)` and `compare_packs(packs, out)`. `cmd_export` and `cmd_compare` call them, and the play loop calls them, not the `cmd_*` functions | `rg "argparse.Namespace\(" scripts/trial.py` finds nothing |
| F3 | Keep the call kinds in one table, in place of `NON_STORY_AGENTS` and the three inline tuples. Rename `history_cut_lines` to `failure_and_cut_lines`. Keep the call-kind heuristic (D5), with a comment that names its reason | `rg "\"quantifier\", \"options\"" scripts/trial.py` finds one place |
| F4 | Fix the typo on line 3 ("four prompt presets"). Add the docstring of `connection_fields`. Name `trigger` in the docstring of `is_story_call`. Close the connection that `connect` opens in the play loop with `contextlib.closing` | The Python tests pass |
| F5 | `engine_tables()` in `test_trial.py` raises when an `ALTER TABLE` literal matches no pattern | New test case: a fake migration text with an unmatched `ALTER TABLE` raises |

## Phase G: The test tree

| Step | Finding | Change | Verify |
|---|---|---|---|
| G1 | Tests that test the same thing twice (`test-police` P2-6, `code-review` S2) | In `generation_parameters_tests.rs`, delete the refused-key pair (`refused_custom_key_names_each_key_the_engine_owns`, `refused_custom_key_is_none_for_other_keys`), because `PromptPreset::validate` cases cover it. Delete the three derive-only tests (`parameters_default_to_nothing_set`, `parameters_roundtrip_through_json`, `call_settings_default_sends_nothing`). In `settings_tests.rs`, delete `connection_for_errors_on_a_dangling_optional_slot`, and keep `check_connection_slots_names_the_dangling_slot` | `python build.py integration` passes. The line coverage of `generation_parameters.rs` and `settings.rs` does not drop |
| G2 | The two Agent-state tests in `runner_tests.rs` are on the legacy list (`thermo` P2) | Add scenario 2.x to `docs/specs/actions.md`, next to 2.5: a Game with no Quantifier Agent completes the Action with the prior Character set. Its test uses `configured_runner_without(recorder, "quantifier")`. Then delete `test_after_narration_keeps_the_state_when_no_agent_is_registered`. Keep `test_after_narration_keeps_the_state_when_the_agent_fails`: its `Err` arm needs a call with no text, and a Job always gives the text. `runner_tests.rs` keeps four other tests, so it stays on the allowlist | The new test passes. Mutation: the `None` arm of `after_narration` returns an empty `present`. The test fails |
| G3 | Copied test helpers (`thermo`, `antipattern` 11 and 12) | Move one `prompt_block(prompt, open, close)` into `src/test_support/`. Use it in place of `between` (Options and Quantifier `agent_tests.rs`) and `layer` (`narrator_tests.rs`). Add `get_with_headers` to `tests/http/support/http_requests.rs`. Use it in place of `get_with_game_header` (`screen.rs`) and the inline request of `answer_words.rs` | The duplicate-code report of the full gate no longer lists `between`. The tests pass |
| G4 | About 20 fixtures list every new preset field (`thermo` P2) | Use `..PromptPreset::default()` in the fixtures that build a whole `PromptPreset` and do not test `option_count` or `parameters` | `rg -c "option_count: None" src tests` drops to the sites that test the value, plus `PromptPreset::default` |
| G5 | The doc comment of the stub list overstates its check (`test-police` P2-5) | Correct the doc comment of `GAME_BOUND_ROUTES` (`tests/test_utils/stub_request_layers.rs`): a test compares the list with the `POST` routes of `game_bound_routes`. The stub server serves no `/debug/is_busy`, so the stub check stays on `POST` | Read the diff |
| G6 | Missing blank lines before `#[test]` in `run_tests.rs` (`test-police` nit) | Add the two blank lines | Read the diff |

## Phase H: Tooling and maps

| Step | Finding | Change | Verify |
|---|---|---|---|
| H1 | Two extensions read the rate-limit state with their own rules (`thermo`, `antipattern` 14) | Move one `pct` and one `windowEnded(reset, nowS)` into `rate-limit-wait/state.ts`. `windowEnded` gives `false` for an unknown reset (D7). `delegate/lib.ts` and `rate-limit-wait/index.ts` import them | A6 and `node --test .pi/extensions/delegate/lib.test.ts` pass. New `lib.test.ts` case: a state with no `fiveReset` gives the base model. It fails before the change |
| H2 | The live maps name the deleted skill `/tdd` | In `.scratch/layout-trials/map.md` and `.scratch/dashboard-design/map.md`, write "test first" without `/tdd`. The `prompt-flexibility` map is a record of past work, so it stays | `rg --hidden "/tdd" .scratch/layout-trials .scratch/dashboard-design` finds nothing |

## What already exists

- `app_with_production_graph` (`tests/http/support/app_wiring.rs`): boots the real wiring with Mock Connections. A2 uses it.
- `ScriptedModel::calls()` and `ModelCall.settings`: the tier-1 read of the settings a call carried. A1 uses it, as `tests/http/retrigger.rs` does.
- `configured_runner_without` (`src/test_support/context.rs`): a runner with one Agent type left out. G2 uses it.
- `GameViewQuery::list_latest_llm_messages` and `LlmMessage.model_name`: the read seam of A2.
- The raw-insert pattern of `tests/storage/preset_storage.rs`: A5(b) uses it.
- The seam case `transport-engine-keys-win-over-custom`: C7 keeps it.
- Scenario 2.5 of `docs/specs/actions.md`: the failed-model path of the Quantifier. G2 adds its missing-Agent twin.
- `render_non_blank_layer`: D4 reuses it.

## Not in scope

- A change to the movement rule of the game rules (D2).
- One size unit for the history cut (D3).
- A Role for Impersonate, or a new agent name for its calls (D5).
- A settings type for the Options hook (D6).
- A UI for the Connection slots or the preset parameters. The map's watch item owns it.
- A `build.py` step for the TypeScript tests of `.pi/extensions/`.
- The card-order and card-placeholder cases in three test files. Each box declares its own case. G3 removes only the copied helper.
- A split of `scripts/trial.py` (D11).
- One policy for a bad field in a preset seed (D12).

## Failure modes

| New code path | Failure | Handling |
|---|---|---|
| `AppSettings::remove_connection` | A refused remove changes the settings | B2 checks before it changes anything. A domain case pins it. `update_settings` also rolls back on an error |
| The slot table (B1) | A row names the wrong field | The `settings_tests.rs` cases, A1 and A2 cover each slot |
| `recorder_with_storage(storage, PresetType)` (B3) | The wrong recorder reaches the Narrator or the Agent runner | A2 reads the `model_name` of the Options call and of the Impersonate call |
| The Job helper `call_settings` (C4) | A dangling slot | The Job ends with the Settings read failure (D4). A3 pins it |
| `TokenBudget` methods (C2) | A changed sum moves the history start | The `narrator-history-*` cases and `history_cut_tests.rs` pin the cut points |
| `refused_custom_key` with `TYPED_KEYS` (C7) | A stored preset with `temperature` in `custom` | The seed skips such a file with a warning. `save_preset` refuses it. No preset in `data/` has `custom` |
| `windowEnded` (H1) | An unknown reset gives the spare model | D7 makes it the base model. A `lib.test.ts` case pins it |

## Done when

- Each **Verify** line holds.
- `python build.py` passes.
- `node --test` passes for both extension test files.
- The four findings files have no open finding that this plan does not fix or keep.
