# P4: the Agent edge on values

Type: task
Status: resolved
Blocked by: 04

## Question

Build §7 of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) on branch `wf/rebuild-p4`, from the current `main`. Read §1, §2 and §7 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §7 pass. The owner has approved the last commit, and the branch has merged to `main`.

## Comments

**2026-10-17, implementer (P4 commit 1 built, uncommitted).** Branch `wf/rebuild-p4`, base `4ab1380b` (P3 merged). The first commit of §7's Order: the strict fake, the real Quantifier/Options at tier 1, and the tier-1 files of the strict-fake rework. The Agent edge (Hook, `AnalysisAgent`/`OptionsAgent`, `AgentRunner`, `HookInput`, `Narrator`, the seam contracts, rules 11 and 16) is **not** in it; today's `AgentRegistry` stands.

Facts measured.

| Item | Result |
|---|---|
| `src/test_support/scripted_model.rs` (new) | 326 lines. `ScriptedModel` is an `LlmProvider`: `.narration/.event_narration/.analysis/.options/.reply(role, text)/.empty(role)/.failed(role)` queue one entry per call, keyed by `ModelRole`; `model()`/`name()` report `scripted`/`Scripted`; an unscripted call fails the test. `hold(role) -> ModelHold` keeps the next call of that role until `release()`, and releases on drop so a failing test cannot strand it. `calls()`/`call_count(role)` read the call log. `analysis_report(&[ids], Some(room))` builds a parsed room report. |
| `src/test_support/context.rs` | `configured_registry(quantifier_recorder, options_recorder)` and `configured_registry_without_options(recorder)` build the production default Agents (`AgentConfig::defaults()`, `storage: None`, as `with_provider` Agents have them today). The unit-only `make_test_pipeline_app*` keep their empty `AgentRegistry`, so no legacy test changes (see Deviation 2). |
| `src/test_support/test_app_builder.rs` | new `.model(Arc<ScriptedModel>)`: it builds the pipeline override on `make_test_recorder_with_storage` + `configured_registry`. Used by the fixtures that seed through the builder (story_log, reset, swipe_new 22.x, retrigger 15.1, requires_migration). |
| `tests/http/support/app_wiring.rs` | `app_with_model`, `app_with_model_and_settings`, `app_with_model_and_storage(storage, model, settings, agents)` replace `app_with_narrator*`; `app_with_production_graph` unchanged. |
| Tier-1 tests scripted | 87 of 122 tests in the files below run on the scripted model; the rest run no Job. Per file (scripted/total): actions 22/22, llm_messages 2/2, narrator_mode 5/5, options 15/15, reset 2/2, retrigger 4/9, story_log 4/6, swipe_new 23/30, swipe_switch 2/2, races 3/3, requires_migration/fragment 5/26. |
| Tier-1 use of the `MockBackend` knobs | none left: no tier-1 file names `with_narrations`, `with_prompt_responses`, `with_delay`, `narration_started` or `with_trigger_delay`. The knobs stay (Deviation 1). `tests/http/settings.rs` keeps `MockBackend::new().with_fail()` for the connection-test route, which runs no Job. |
| `tests/http/races.rs` (new) | 179 lines, registered in `tests/http/mod.rs`; module header names I1 Action (actions.md 3.4) and I1 /options (options.md 24.21). Three tests, all green on today's code: Action while a Job is held, the two-fast-clicks case, `/options` while a Job is held. So neither I1 case moves to the core step. |
| New scenarios | `docs/specs/actions.md` 3.4 (Action while a generation is in flight, with the two-fast-clicks assertion); `docs/specs/options.md` 24.21 (`/options` while a generation is in flight). Written before their tests. |
| 12.1, 15.1 | rewritten to hold a real Job through the route with the holdable fake, in place of the direct `try_claim` arrange. 12.1 holds an Action's narration call, so `/swipe/new` passes the input-last validation and meets the busy slot; 15.1 holds an **Options** Job, because the retrigger route validates that the last stored Message is a Narration, and an Action in flight has only its Input saved (a narration hold answers 400 — measured). |
| 3.3 | holds the fake (narration) in place of `with_delay(200)`: the test asserts the call reached the model, releases, then waits idle. |
| 22.1 | gets the "the player has not moved" assertion (the room is unchanged) plus `call_count(Analysis) == 1`, pinned because the re-impersonate redo runs the analysis today. |
| The Done-when check | `wait_idle` (the tier-1 helper every Job test calls) now asserts the story log holds no `NPC detection uncertain` notice. `story_log_holds_uncertain_npc_notice` reads it. The `wait_idle_allowing_fallback` variant I first added was removed: no tier-1 test needs it (see Finding 2). |

Findings.

1. **22.1's parenthetical is wrong today.** `swipe_new.md` 22.1 says "the player has not moved (the quantifier did not re-run)". A re-impersonate redo goes `RetryMode::ReImpersonate -> retry_main_narration -> run_from_input`, so the post-generation analysis **does** run (measured: one analysis call). The empty registry hid it — no tier-1 app had a Quantifier at all before this commit. Target design R-60 makes the run the intended behaviour, and §9.2 already has the core step reword the scenario, so this is not a new defect: the test asserts the true behaviour and the parenthetical is left for C7. Not a silent fix: the scenario text is untouched here.
2. **The uncertain-NPC notice needs an empty static set too.** `pipeline_run.rs` adds the notice only when the analysis result has no NPCs **and** is not High; a failed or unparseable analysis falls back to `static_npc_result`, which returns the room's current NPCs, so the notice does not appear in 2.5's two tests. They keep the plain `wait_idle` and now also pin `call_count(Analysis) == 2` (both attempts). Relevant to §9.2's "the low-confidence line goes".
3. No other hidden defect surfaced: the full http binary is green with the real Quantifier and Options agent on every Job test.

Deviations from §7, and what they need.

1. **The `MockBackend` knobs stay.** Their last tier-1 user is gone, but 67 uses remain in 10 legacy `src/**/*_tests.rs` files (`narration_generation_tests`, `action_pipeline/{action,core,options,retrigger,retry}_tests`, `agents/options/agent_tests`, `agents/quantifier/utils/orchestration_tests`, `llm_recorder_tests`, `providers/mock_tests`). Deleting the knobs would migrate those legacy tests, which §2 and §10 leave to the owner, one module at a time. **Owner decision needed**: either keep them (this commit) or fold the legacy migration into this slice.
2. **The plan's `src/test_support/context.rs` wiring item is narrower than it reads.** The only `AgentRegistry::default()` calls in that file are in `make_test_pipeline_app` / `make_test_pipeline_app_with_storage`, which no tier-1 file uses (they are shared by the legacy unit tests). They keep the empty registry; the tier-1 entry points (`make_test_pipeline_with_backends` callers, `app_with_model*`, `TestAppBuilder::model`) take the configured Agents. `make_test_pipeline_with_mock_quantifier` keeps its quantifier-only registry, as today.
3. **The registry's Agents read no storage.** `configured_registry*` pass `storage: None`, matching `QuantifierAgent::with_provider` / `OptionsAgent::with_provider`, so no preset override reaches the scripted prompt. The whole-storage wiring arrives with D-33 (the caller owns the Settings/preset reads) in the Agent-edge commit.
4. `tests/http/requires_migration/fragment.rs`'s five Job tests are scripted through a local `one_turn_app()`; the quarantine stays untagged, so its pinned count is unchanged.

Validation. `python build.py` green at `logs/build_20261017_202724.log` (tree `4ab1380b97+b633458b`): fmt, data, clippy, test structure, spec-coverage, seam-cases, Python, http_routes/guardrails freshness, docs, guardrails 187 passed, integration 1626 passed + 2 skipped (+3 = the three `races.rs` tests), browser 29 passed. Focused runs: `logs/build_20261017_200801.log` (the first full http pass, 266/270), `logs/build_20261017_201038.log` (http green), `logs/build_20261017_201818.log` (guardrails green).

What is left in P4: the Agent edge (`Hook`, `AnalysisAgent`/`OptionsAgent`, `AgentRunner::analysis`, `HookInput`/`AgentSettings`, the value types `Kind`/`StoryEntry`/`EventRecord`/`WorldView` and their glue), `application/narration/narrator.rs`, the three seam contracts (Quantifier, Options, Narrator) with their `CASE` lists, and guardrail rules 11 and 16 — the plan's §7 content after the Order paragraph.

**P4 commit 1, lead decisions (2026-10-17).**
- Scenario 22.1: its line "the player has not moved (the quantifier did not re-run)" was false on `main` (the redo runs the analysis once, measured). The behaviour already matches target design R-60, so the spec loses the false parenthetical now: the line is "the player has not moved". The analysis call is not a domain outcome, so the spec does not state it. Plan §9.2's 22.1 item is done. Scenario 22.2 (user regen) runs no analysis and keeps its line.
- `MockBackend` test knobs (`with_narrations`, `with_prompt_responses`, `with_delay`, `narration_started`, `with_trigger_delay`) stay. No tier-1 user is left. Their 61 uses are in legacy unit tests; 45 of them are in pipeline tests that C6 deletes. The Options and Quantifier `agent_tests.rs` move to `ScriptedModel` when they become seam contracts in P4 commit 2. C6 deletes the knobs when their last user goes.

**2026-10-17, implementer (P4 commit 1, fix pass, uncommitted).** Every accepted finding is applied; no commit was made.

1. **Strictness fails the test directly.** An unscripted call is recorded (`stray: Vec<ModelRole>`, with the roles seen so far) and answered with `EngineError::Config` naming it (`STRAY_CALL_MARKER = "did not script"`). Three end points assert it: `ScriptedModel::assert_no_unscripted_calls()` (called by the fake's drop), the hold's drop (`ModelHold::drop`, which holds an `Arc` of the fake's record now), and `wait_settled`, which reads the recorded failed attempt through `GameViewQuery::list_latest_llm_messages` — the fake's own drop runs on the blocking thread a Job holds, and a forensics read is the only handle-free way to see it. Measured, twice: with the analysis dropped from a script, `wait_settled` fails with `a model call reached the fake without a scripted answer: ["LLM Error: Configuration error: ScriptedModel: the test did not script a analysis call..."]` (`logs/build_20261017_205357.log`), and with `wait_idle` in place of `wait_settled` the hold's drop fails with `ScriptedModel: the test did not script 2 call(s): [Analysis, Analysis]` (`logs/build_20261017_210422.log`). `panic!`/`assert!` only; no `unreachable!` left. The three tests that never waited — `requires_migration/fragment.rs`'s `test_action_handler_accepts_command`, `test_action_handler_empty_command`, `test_action_async_inventory` — now settle the Job (`one_turn_app()` returns `(Router, AppState)` and each test ends with `wait_settled`), and `test_action_confirm_empty_command` dropped its hand-rolled 500 ms poll for the same call. Measured with a broken script: `test_action_async_inventory` fails on the stray (`logs/build_20261017_205439.log`).
2. **Hold.** Code and doc agree: it holds the **next** call of its role only (the fake clears the armed hold when it fires, so later calls pass through); `hold()` and every builder method are `#[must_use]`; the sync is one `Mutex<HoldSync> + Condvar`, notified under the lock, and `wait_started` waits on the condvar with `wait_timeout_while` (no polling). It is a blocking method now, documented as such, because the held call runs on the blocking pool and never on a runtime worker; the five call sites lost their `.await`.
3. **`turn()`, one offered set, one app helper.** `ScriptedModel::turn(narration)` (a narration plus an empty analysis report) replaced the seven private `turn()` / `one_action` copies. One `OFFERED_SET` lives in `tests/http/support/http_fixtures.rs`; `actions.rs`, `retrigger.rs` and `swipe_switch.rs` import it (options.rs keeps `SET_A`/`SET_B`, which are two sets to tell apart, not one). The four per-file app wrappers are gone: `tests/http/support/app_wiring.rs` has `app_with_script`, `app_with_script_and_agents` and the existing `app_with_model*` / `app_with_model_and_storage`.
4. **`configured_registry(recorder)`** takes one parameter; `configured_registry_without_options` became `configured_registry_without(recorder, agent_type)`.
5. **No tier-1 test reads the fake's call log.** The call-count assertions went to the forensics read seam: `llm_call_count(&state, "quantifier")` in `actions.rs` 2.5 (2 = the retry) and `llm_call_count(&state, "narrator"/"options")` in `races.rs` (1 and 0). `swipe_new.rs` 22.1's analysis-count assertion is deleted: the lead's decision records that the analysis call is not a domain outcome, so the scenario does not state it and the room assertion carries the line. 12.1 and 15.1 now assert their own last spec line through the story log after the release: 12.1 one Input and one Narration (no retry started), 15.1 no event Narration (no retrigger started).
6. **Two fast clicks.** `test_two_actions_at_the_same_moment_save_one_input_http` is `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`: two spawned tasks released by a `tokio::sync::Barrier`, both POST `command="look"`, and the assertions are order-independent (exactly one body carries "Still thinking...", exactly one Input "look", one narration call recorded). Scenario 3.4 was self-contradictory, so it split: 3.4 is "An Action is refused while the Game is busy" and 3.5 is "Two clients POST the same Action at the same moment and one Action is accepted" (both in `actions.md`); `options.md` 24.21 is "/options is refused while the Game is busy". The scenarios use the CONTEXT.md terms Job, busy Game, Action, Narration; the tests carry the new tags 3.4 / 3.5.
7. **One model, configured Agents, in `swipe_new.rs` and `retrigger.rs`.** All 19 `make_test_pipeline_with_mock_quantifier` sites are gone from tier 1 (the helper stays for the legacy unit tests); those turns now script one `ScriptedModel` handed to `TestAppBuilder::model` or `app_with_script_and_storage`, so they run the real Quantifier **and** the real Options agent. Three of them need an interleaved script rather than `.turn()` twice, because the two turns report different analyses (9.2, 9.5, 9.8) — that cost three red tests and is now explicit in their comments.
8. **`wait_idle` is a pure wait** (doc says so). The plan's check lives in `wait_settled`, which calls the separately named `assert_no_uncertain_npc_notice` after the wait; the notice text now has one home, `pipeline_run::UNCERTAIN_NPC_NOTICE`, which `pipeline_run.rs` writes and the assertion reads. All tier-1 Job tests call `wait_settled` (`wait_idle` keeps its callers through it).
9. **The specs now state today's behaviour.** `swipe_new.md`: 10.2 is "Event retry does not move the player" with "the player is still in "room2" (an event retry applies no movement)"; invariant I.6 was rewritten — it claimed that "re-impersonate and user-regen retries do not re-run the quantifier", but a re-impersonate redo runs the same turn as a main retry (and the event retry does call the analysis for its own reconciliation, applying only the NPC set). I.6 now reads as player-visible movement: a main retry and a re-impersonate retry apply the movement their generation reports, an event retry applies none, and a user-regen retry narrates the replacement message only. 22.2's line stays (a user-regen redo runs no analysis — code-verified) and 22.1 keeps the lead's "the player has not moved". `retrigger.md` 13.2 is "Retrigger does not move the player" with "(a retrigger applies no movement)". The two tests whose names repeated the false parenthetical were renamed: `test_event_retry_does_not_move_the_player_http`, `test_retrigger_does_not_move_the_player_http`. No behaviour changed.
10. **Docs.** Cross-cutting 7 now names which helper wires which registry (`app_with_script`, `TestAppBuilder::model` and `app_with_model` use `configured_registry`; a test that leaves an Agent out passes its own `configured_registry_without(recorder, "options")` to `app_with_script_and_agents`), states the three stray-call end points, and dropped the `make_test_pipeline_with_backends` paragraph while keeping the `MockBackend` sentence. Pattern 1 gained `.model(...)` and the scripted-pipeline paragraph; its `retrigger.rs` exemplar line no longer says "mock quantifier". No plan ids in code, module headers or docs.

Not done, with the reason. `assert_no_unscripted_calls()` is not called by a test body anywhere: the fake's drop and the hold's drop call it, and `wait_settled` uses the forensics read, which is the only check that works without a handle. The `MockBackend` knobs still stand (the lead's decision: their 61 uses are legacy, 45 of them in pipeline tests C6 deletes). Also note: the hold must be released for a stray to surface through the hold's drop, so the drop releases first and asserts second.

Validation. Full gate green at `logs/build_20261017_211007.log` (tree `4ab1380b97+ccc9ffd7`): clippy, data, structure, spec-coverage, seam-cases, Python 393, http_routes/guardrails freshness, docs, guardrails 187, integration 1626 + 2 skipped, browser 29. `--test-target http` three times: `logs/build_20261017_210950.log` 4.16 s, `logs/build_20261017_210955.log` 4.39 s, `logs/build_20261017_211000.log` 4.18 s, 274 passed each, no races.rs flake. Focused stray and hold-drop probes: `logs/build_20261017_205357.log`, `logs/build_20261017_210402.log`, `logs/build_20261017_210422.log`, `logs/build_20261017_205439.log` (each reverted after measuring).

**2026-10-17, implementer (P4 commit 2, uncommitted).** The rest of plan §7: the Agent edge on values, the `Narrator`, the three seam contracts, rule 11, and the docs. No commit was made.

Content items, each to its result.

| Plan item | Result |
|---|---|
| `ExecutionPhase` → `Hook` | `Hook { Analysis, Options }` in `domain/model/agent.rs`, `#[serde(rename_all = "snake_case")]`, `Analysis` default. The before-narration and rewrite hooks come with their first Agent. |
| `AgentResult` → the output types | `AnalysisOutput(pub StatePatch)` and `OfferedSet(pub Vec<String>)`. `AgentResult`, `PromptDirective` and `NoOp` go. |
| `AgentContext` → `HookInput` (values) | `HookInput { kind, text, player_text, story, world, event }`, with `Kind { Narration, Impersonate, UserRegen, Event }`, `StoryEntry { message_type, text }` (+ `From<&MessageEntry>`), `EventRecord` (+ `From<&StoredTriggerContext>`), `WorldView { room, map, persona, characters, present }` and `STORY_WINDOW = 4`. No `GameId`, no storage, no dock record, no failure line; `AgentContext.game_id` (the P1 throw-away glue) is gone. `StatePatch`, `Confidence` and `Role` stay. |
| `AgentSettings` | `{ preset: Option<String>, max_tokens: Option<u32> }`, the values one Agent reads from its own config. The caller loads both (D-33) and the Agents pass `max_tokens` into their model calls, so the field is live. |
| `AnalysisAgent` and `OptionsAgent` | In `trait_def.rs`: `name()` and `run(&HookInput, &AgentSettings) -> Result<Output, EngineError>`. `Agent`, `phase()` and `backend_selector()` go (nothing read the selector: the wiring binds each Agent's recorder by name). |
| `registry.rs` → `runner.rs` | `AgentRunner { analysis, options }`: `from_configs(configs, analysis_recorder, options_recorder)` (the production wiring, empty config keeps the defaults), `with_analysis_agent`, `with_options_agent`, `analysis(&input, &settings) -> QuantifierResult` (the old `PostGeneration` merge loop, failures logged and dropped), `options(&input, &settings) -> Result<Vec<String>>` (`Ok(empty)` = no Options Agent), `is_empty`. |
| The Quantifier and the Options Agent | Both implement their hook trait and hold only a recorder: `from_config(config, recorder)`, `with_provider` for tests. `from_config_with_storage` and their Storage-backed Settings/preset reads are gone (D-33). |
| `Narrator` with `narrate` | `application/narration/narrator.rs`: `Narrator { assembler, recorder }` with `narrate(context, preset, response_length, budget) -> Result<Narrated, GenerationFailure>` (assemble, one call, empty-answer failure) and `assemble(..)` for the trigger context. `call_narrator` is gone from `pipeline_run.rs`; `Narrated { text, backend_name, model_name }` replaces the triple. |
| `PromptAssembler::with_storage` goes; the budget is a value | `with_storage`, `with_max_tokens`, `resolve_budget` and the `max_tokens` field go; `assemble(.., budget: TokenBudget)` takes `TokenBudget { max_context_tokens, max_tokens }` (`prompting/token_budget.rs`). `PipelineRun::narration_budget()` reads the connection per call, as the old `resolve_budget` did. |
| The glue | `PipelineRun::run_analysis_hook(state, map, persona, npcs, text, player_text, kind)` builds the story window, the event record and the `WorldView` (about 25 lines) and calls the runner; `phase_post_generation` and `reconcile_post_trigger_npcs` both use it (kind `Narration` and `Event`). The Options slice builds its own input in `run_options_generation`. |
| No `Tracker`, no `SwipeState.trackers` | Neither added (ticket 17 Q9). |

Seam contracts. `validate_seam_cases.py` now prints its result and the contracts with their case counts:

```
Seam cases OK: 3 contract(s), 32 case(s).
  src/application/agents/options/agent_tests.rs: 8 case(s)
  src/application/agents/quantifier/agent_tests.rs: 11 case(s)
  src/application/narration/narrator_tests.rs: 13 case(s)
```

- **Quantifier** (11): npc ids, unknown id dropped, movement, low-confidence fallback, retry, transport failure, no text, no room, preset, call record, failed-call record.
- **Options** (8): offered set, retry, transport failure, no room, preset, fallback prompt, call record, failed-call record.
- **Narrator** (13): every prompt layer, history fit to the budget, response cap, posture macros, guide layer, impersonate layer, global rules, two cleanup cases, empty answer, transport failure, call record, failed-call record.

All three call only their box's entry point (`AnalysisAgent::run`, `OptionsAgent::run`, `Narrator::narrate`) and build the box as production does (`from_config`/`from_configs`, `Narrator::new(assembler, recorder)`); the two Agents' files left the legacy list and the three are on `SEAM_CONTRACT_FILES`. They use `ScriptedModel`, which is why the old `MockBackend` knobs lost their last tier-1 user; the prompt/parser/orchestration tests stay legacy per the plan. `runner_tests.rs` (the renamed `registry_tests.rs`) stays legacy: the runner is not a box behind a port.

Guardrails. **Rule 11**: `arch-lint.toml` gained the `agent-edge` scope (`src/application/agents/**`, `src/application/narration/**`) and a `deny-scope-dep` on `storage`, with the reason in the file's comment. **Rule 16**: each hook trait returns a value (`AnalysisOutput`, `OfferedSet`), and no Agent holds `Storage` or a `JobWriter` — the Agents hold one `LlmCallRecorder` and the arch-lint scope now holds the door shut. Both are green. Rule 16's compiler half has no walker row by design.

Docs. `agent_system.md` rewritten for the hooks, the values and `AgentRunner`; `agent_system_design.md` rewritten for the runner, the hook output types and the two hooks that wait for their first Agent; `prompt_system.md` names none of the changed items, so it did not change; `CONTEXT.md` gained **Hook**; `tests/STRATEGY.md` "The one rule" names the `Narrator` among the boxes; `scripts/validate_seam_cases.py` says the same in its docstring; `tests/AGENTS.md`'s seam recipe now names `AgentRunner::with_options_agent`; `integration_test_standards.md` names `configured_runner` / `configured_runner_without`. `data/settings.json` and `data/schemas/settings.schema.json` carry `hook` instead of `phase` (`["analysis", "options"]`).

Behaviour. Unchanged: no scenario text was touched, and the tier-1 suite plus the lib unit tests are green. `log` lines: `logs/build_20261017_214317.log` (full gate, tree `90128752ed+1641a281`, integration 1642 + 2 skipped, guardrails 187, browser 29), `logs/build_20261017_214230.log` (`--test-target lib`, 1265 passed), `logs/build_20261017_214251.log` (`--test-target http`, 274 passed), `logs/build_20261017_214213.log` (clippy).

Deviations, and what is left.

1. **The storage-backed preset and budget reads stay in the pipeline.** The plan says the preset lookups move to the narrator; rule 11's scope denies `application/narration/**` storage, and the preset id, its text and the connection budget all come from storage. So the narrator takes them as values (`&PromptPreset`, `&str response_length`, `TokenBudget`) and `PipelineRun::{analysis_settings, options_settings, narration_budget}` keeps the reads. The value boundary is the same one D-33 draws for the Agents.
2. **`AgentSettings` is one value per hook call**, not the sketch's `HookSettings` keyed by Agent name: each hook has one Agent today, and a hook with several Agents needs the per-name map when its second Agent arrives.
3. **`PromptAssembler::with_max_tokens` was removed, not kept.** The budget is read per call from the connection today, so it comes in as an argument; a boot-time value would freeze a dashboard edit.
4. **The 58 prompting unit tests did not move into the Narrator contract yet.** The contract covers the clauses the plan names (the prompt sections, the budget fit, the input cleanup, the call record), but `assembler_tests.rs` (20), `builders/sections_tests.rs` (7), `utils/context_tests.rs` (10), `sanitize_tests.rs` (10) and `token_budget_tests.rs` (11) are still legacy files. Their subjects are inside the Narrator box, so the one rule wants one `CASE` per test through `narrate`, and the rewrite is the next bounded piece of work; `token_budget_tests.rs` needs an owner decision first, because `estimate_tokens`/`truncate_to_budget` are pure helpers whose arithmetic no `narrate` call can observe (they are neither in `domain/model/` nor a box, so place 2 does not cover them).
5. `MAX_RESPONSE_TOKENS` and the other budget constants stay where they were; `TokenBudget::fallback()` mirrors the old no-storage fallback (`MAX_CONTEXT_TOKENS`, no response cap). Rule 11 has no walker row (it is an arch-lint scope, as the plan's table says).

**2026-10-17, implementer (P4 commit 2, the prompting tests move, uncommitted).** The prompting unit tests moved into the Narrator seam contract, per the lead's decision. `validate_seam_cases.py` now reports:

```
Seam cases OK: 3 contract(s), 55 case(s).
  src/application/agents/options/agent_tests.rs: 8 case(s)
  src/application/agents/quantifier/agent_tests.rs: 11 case(s)
  src/application/narration/narrator_tests.rs: 36 case(s)
```

The Narrator contract grew from 13 to 36 cases; every case drives `Narrator::narrate` only, and a prompt-only case reads the one `ScriptedModel` call the box made.

**Count per source file (moved as a case / merged into another case / stayed legacy).** The sum is 58.

| Source file | Tests | Moved as its own case | Merged | Stayed legacy |
|---|---|---|---|---|
| `prompting/assembler_tests.rs` | 20 | 8 | 12 | 0 |
| `prompting/builders/sections_tests.rs` | 7 | 4 | 3 | 0 |
| `prompting/utils/context_tests.rs` | 10 | 1 | 5 | 4 |
| `prompting/sanitize_tests.rs` | 10 | 8 | 2 | 0 |
| `prompting/token_budget_tests.rs` | 11 | 0 | 0 | 11 |
| **total** | **58** | **21** | **22** | **15** |

`assembler_tests.rs`, `builders/sections_tests.rs` and `sanitize_tests.rs` are empty after the move, so the files go with their legacy-list entries; `utils/context_tests.rs` and `token_budget_tests.rs` keep theirs. The lib binary lost 14 tests net (58 out, 23 new cases in, 15 kept).

**The merges** (two old tests pinning one clause through the entry, one case carrying both):

- `narrator-prompt-sections` ← assembler `test_assemble_includes_all_layers`, context `test_context_fitting_no_trim_needed`, context `test_context_fitting_user_already_fits`.
- `narrator-preset-fields-absent` ← assembler `test_assemble_empty_preset_sections`, sections `render_field_parts_omits_all_fields_when_preset_has_none`.
- `narrator-preset-fields-present` ← sections `render_field_parts_emits_only_present_fields`, sections `render_field_parts_appends_response_length_to_output_format`.
- `narrator-history-fit` ← assembler `test_assemble_budget_trimming`, assembler `test_budget_value_fits_the_history_and_the_response_cap`, context `test_context_fitting_trims_oldest_history`.
- `narrator-response-cap` ← assembler `test_assemble_respects_max_tokens`, assembler `test_budget_value_fits_the_history_and_the_response_cap`, context `test_context_fitting_caps_max_tokens`.
- `narrator-posture-macros` ← assembler `test_assemble_injects_narrative_voice_from_resolved_posture`, assembler `test_assemble_without_settings_uses_default_voice` (and the earlier case).
- `narrator-guide-layer` ← assembler `test_assemble_guide_layer_renders_last_with_wrapper`.
- `narrator-guide-absent` ← assembler `test_assemble_no_guide_omits_guide_layer`, assembler `test_assemble_blank_guide_omits_guide_layer`.
- `narrator-impersonate-layer` ← assembler `test_assemble_impersonate_drops_player_character_layer` (and the earlier guide clause).
- `narrator-empty-answer` ← sanitize `test_sanitize_empty_string`, sanitize `test_sanitize_whitespace_only` (both answers sanitize to empty, which the box fails).
- `narrator-context-overflow` ← context `test_context_fitting_system_overflow`, context `test_context_fitting_post_trim_overflow`.

**Owner items: the 15 tests that stayed legacy, each a pure or internal helper outside `domain/model/`, not observable through the Narrator entry.** They are unchanged, and their files stay on the list.

- `prompting/token_budget_tests.rs` (11): `test_token_budgets` (the constants' values), `test_estimate_tokens_empty`, `test_estimate_tokens_single_char`, `test_estimate_tokens_exact_four`, `test_estimate_tokens_five_chars`, `test_estimate_tokens_many_chars`, `test_truncate_to_budget_no_truncate_needed`, `test_truncate_to_budget_exact_fit`, `test_truncate_to_budget_truncate`, `test_truncate_to_budget_preserves_recent`, `test_truncate_to_budget_zero_tokens` — the arithmetic of `estimate_tokens` and `truncate_to_budget`.
- `prompting/utils/context_tests.rs` (4): `test_make_prompt_context` (the `PromptContext::new` value builder), `test_trim_history_to_budget_no_history_tag`, `test_trim_history_to_budget_empty_content`, `test_trim_history_to_budget_no_close_tag` (the `trim_history_to_budget` helper's own edge cases).

Behaviour did not change: no source test was deleted without a case, no scenario was touched, and the full gate is green.

Validation. `python scripts/validate_seam_cases.py` → `Seam cases OK: 3 contract(s), 55 case(s).` (above). Clippy `logs/build_20261017_215212.log`; guardrails `logs/build_20261017_215251.log` (187 passed); `--test-target lib` `logs/build_20261017_215225.log` (1251 passed); full `python build.py` `logs/build_20261017_215315.log` (tree `90128752ed+22858548`): clippy, data, structure, spec-coverage, seam-cases, Python 393, docs, guardrails 187, integration 1628 passed + 2 skipped, browser 29.

**2026-10-17, implementer (P4 commit 2, fix pass, uncommitted).** The spec-review and standards-review findings are applied. No commit was made.

Lead decisions recorded here.

- **The preset and budget reads stay in the pipeline.** Rule 11's `agent-edge` scope denies `storage` to `application/agents/**` and `application/narration/**`, and the plan's line "the preset lookups move here" cannot hold against that: the preset id, its text and the connection budget all come from storage. The narrator keeps the value boundary (`&PromptPreset`, `&str response_length`, `TokenBudget`) and `PipelineRun::{analysis_settings, options_settings, narration_budget}` keeps the reads.
- **Rule 11 bans the storage adapter scope only.** The row's deny list holds the `storage` scope, so a wrapper such as `MessageService` would not trip it. Noted; no change made, because the Agents and the narrator hold neither.
- **The `OptionsAgent` struct is renamed.** The plan names the traits `AnalysisAgent` and `OptionsAgent`, so the struct became `application::agents::options::agent::OptionsGeneratorAgent` (CONTEXT.md's term is **Options**; the Quantifier's struct is `QuantifierAgent`). The `OptionsAgentTrait` aliases are gone, and `agent_system.md` and `tests/AGENTS.md` name the new type.
- **`Kind` is not a target-design §1 term** (its tables use the kinds but §1 has no bold entry), so the type carries a one-line doc instead of a glossary entry: the kind of one generation, and a typed Input is no generation.

Fixes, each to its result.

- **S1 — the per-call budget read is a tier-1 test now.** Per the lead's decision, the legacy-file test and its helpers went from `application/pipeline/narration_generation_tests.rs` (a legacy file only shrinks, and the test used `MockBackend` knobs that C6 deletes), and `tests/http/settings.rs` gained `test_connection_context_window_applies_from_the_next_narration`, tagged `SCENARIO: 20.25`: a `ScriptedModel` app on a seeded 150-entry story log, a first Action that settles, an edit of the Connection through `POST /connections/{id}/edit`, a second Action that settles, and the two recorded narrator prompts read back through `GameViewQuery::list_latest_llm_messages`. It asserts the small window does not carry the whole log (`tight < 150`) and that the second prompt holds more entries than the first. **The route used is the edit route, but the field it changes is the provider, not the window:** `ConnectionForm` and `ConnectionFormTemplate` carry only name, provider, model, API key, base URL and single-user-message, `add_connection_handler` hard-codes `max_tokens: None, max_context_tokens: None`, and `edit_connection_handler` never touches either field. So the window moves only through the per-provider default (`resolve_max_context_tokens`: Mock 4096, Ollama 8192, OpenRouter/DeepSeek 32768) and the test edits `conn_provider=mock` to `conn_provider=ollama`. Owner item: scenario 20.25 speaks of editing "that Connection to a large context window", which today has no form field; the test drives the nearest client path instead. Adding a window field to the form and the handlers would let the test edit the window itself, and until then the scenario's wording and the form disagree.
- **S3 — the six merged cases left `utils/context_tests.rs`.** They were running twice (once there, once as Narrator cases). The corrected per-file table is below: **43 tests left the prompting files** (21 became their own Narrator case, 22 were merged into one), and **15 stay legacy** (11 `token_budget_tests.rs` + 4 `utils/context_tests.rs`). 43 + 15 = 58. The earlier comment's "37 left / 21 remain" was wrong.
- **S4 — one fallback only.** The `max_context_tokens == 0` branch is gone, and the now-vestigial window field, accessor and `PromptAssembler::new(max_context_tokens)` argument went with it: the builder is `PromptAssembler` (a unit struct) and every call carries its `TokenBudget`. A saved `Some(0)` therefore behaves as on `main` (a window of 0, which the fit refuses).
- **S5 — the Options failure is logged again.** `AgentRunner::options` logs `warn!` for a failed Agent and returns its error.
- **T1 — the runner's tests keep only the config wiring.** `test_runner_merges_its_analysis_agents` (the runner is not a box) and the dead bindings went; `test_agent_config_serde_roundtrip` moved to `domain/model/agent_tests.rs`. The merge rule's tests (union of ids, first destination wins, minimum confidence) were already there — twelve of them, from P3 — so no test was duplicated in. `runner_tests.rs` keeps the three config-wiring tests and stays on the legacy list.
- **T2 — one clause per case.** `narrator-prompt-sections` split into `narrator-prompt-layers` (every layer renders) and `narrator-turn-untrimmed` (a fitting turn passes through); `narrator-history-fit` split into `narrator-history-small-budget` and `narrator-history-large-budget`. The CASE form is unchanged.
- **T3/T4 — names and the two reason comments.** The struct rename above; the `core.rs` comment "An impersonate turn is the player acting, so its offered set is not rewritten." is back; `StatePatch::merge` carries its policy in a doc comment (union, first destination wins because a second is a contradiction between Agents, least confident wins).
- **T5 — the hooks get the real kind.** The analysis hook receives `Kind::Impersonate` on an impersonate turn and `Kind::Narration` otherwise (`phase_post_generation` gained the kind; the retry paths recover impersonation before that point), and `Kind::Event` for an event's reconciliation. The Options hook has no kind of its own: `PipelineRun::scene_kind` reports the kind of the generation whose scene it reads (an event Narration when the last Message is one, else a Narration) for both the on-demand and the turn-end call. Correction: **`AgentSettings.max_tokens` is always `None` in production**, as before this slice — the field exists so a hook's budget reaches its Agent, and the Agents pass it into their model calls.
- **T6/T11 — one place builds a hook input.** `PipelineRun::with_hook_input` owns the story window, the event record and the `WorldView` and hands the input to a closure; `run_analysis_hook` and `run_options_generation` both go through it. `room_of` is an associated function and `scene_kind` is one too.
- **T7 — "no Agent" is its own answer.** `AgentRunner::options` returns `Result<Option<Vec<String>>, EngineError>`: `Ok(None)` is no Options Agent (the caller reports it unavailable) and a failure is an error. The two call sites distinguish them.
- **T8 — `Confidence` is ordered for `.min()`.** `Low < Medium < High`, so `self.confidence.min(other.confidence)` keeps the least sure analysis. No serde on the type, so nothing serialized moved.
- **T9 — the analysis reads its text from the input** (`text` argument dropped; the input without text is the error) and `process_quantifier_result` filters the ids by the Character map instead of cloning a Vec to search it.
- **T12 — dead code.** The throw-away `fat_room` fixture and the unused recorder in `recording_narrator` went, and `ModelCall` lost `agent_name` (never read; the call-record cases read the stored forensics row).

The corrected per-file table (the sum is 58).

| Source file | Tests | Moved as its own case | Merged | Stayed legacy |
|---|---|---|---|---|
| `prompting/assembler_tests.rs` | 20 | 8 | 12 | 0 |
| `prompting/builders/sections_tests.rs` | 7 | 4 | 3 | 0 |
| `prompting/utils/context_tests.rs` | 10 | 1 | 5 | 4 |
| `prompting/sanitize_tests.rs` | 10 | 8 | 2 | 0 |
| `prompting/token_budget_tests.rs` | 11 | 0 | 0 | 11 |
| **total** | **58** | **21** | **22** | **15** |

Validation. `python scripts/validate_seam_cases.py`:

```
Seam cases OK: 3 contract(s), 57 case(s).
  src/application/agents/options/agent_tests.rs: 8 case(s)
  src/application/agents/quantifier/agent_tests.rs: 11 case(s)
  src/application/narration/narrator_tests.rs: 38 case(s)
```

Clippy `logs/build_20261017_222755.log`; spec-coverage `logs/build_20261017_222743.log`; guardrails `logs/build_20261017_222758.log` (187 passed); `--test-target lib` `logs/build_20261017_222813.log` (1246 passed); `--test-target http` `logs/build_20261017_222836.log` (275 passed, the new scenario among them); full `python build.py` `logs/build_20261017_222844.log` (tree `90128752ed+229d6554`): clippy, data, structure, spec-coverage, seam-cases, Python 393, docs, guardrails 187, integration 1624 passed + 2 skipped, browser 29.

**2026-10-17, lead (retro).** The lead wrote Scenario 20.25 for a context-window field that the Connection form does not have; the test changes the window through the provider, and the scenario now says so. This repeats ticket 02's lesson (check that a named thing exists before building against it), so it applies to spec lines too. The two code-review axes and the doc review each found real defects in both P4 commits; no recurring process gap beyond this one. Owner items from P4: (1) the 15 prompting helper tests on the legacy list (`token_budget_tests.rs` 11, `utils/context_tests.rs` 4): move the helpers to `domain/model/` or delete the tests; (2) the Connection form has no context-window or `max_tokens` field; (3) the seam-case form, decided as option B in [ticket 08](08-seam-case-tag-only-form.md).

## Answer

P4 is built and merged to `main` in two commits. Commit 1: the strict `ScriptedModel` fake and the real Agents at tier 1, with `races.rs` (I1). Commit 2: the Agent edge on values: `Hook`, `AnalysisAgent`/`OptionsAgent`, `AgentRunner`, `HookInput` with its value types, `Narrator::narrate`, three seam contracts (57 cases), rule 11 (arch-lint) and rule 16 (compiler). The preset and budget reads stay in the pipeline (rule 11). Full gate green: `logs/build_20261017_222844.log`.
