# Architecture review of the rebuilt code — Opus run (ticket 14)

- **Tree:** `wf/rebuild-core`, HEAD `d7cf3eb0`. HEAD is two ticket-only commits after `2885df2c` (`cc20bdda`, `d7cf3eb0`). `git diff 2885df2c HEAD -- src tests` is empty, so the code is the code of `2885df2c`.
- **Dirty files under `src/` and `tests/`:** 0.
- **Method:** steps 1 and 2 of `/improve-codebase-architecture`. I explored myself (no subagent). I did not read the whole-rebuild review assets, the other architecture-review runs, or tickets 13 and 17. `docs/adr/` does not exist. `arch-lint.toml` and `tests/STRATEGY.md` hold the recorded decisions, so I checked each candidate against them.
- **HTML report:** `/tmp/architecture-review-arch-opus-20261019T232823Z.html`.
- **Epistemic labels:** *known* = I read the code; *inferred* = it follows from the code, but no test or run confirms it.

| Strength | Count |
|---|---|
| Strong | 3 |
| Worth exploring | 4 |
| Speculative | 1 |

---

## 1. Deepen the Game rules: one call per generation — **Strong**

Dependency category: in-process (pure `domain/model/`).

**Files**
- `src/domain/model/utils/game_rules.rs` (`apply_game_rules`, `GameRuleRequest`, `TriggerMatch`, `room_of`, `room_name`)
- `src/application/pipeline/job.rs` (`apply_rules`, `room_of`, `room_name`, `is_the_opening`, `resolve_room`)
- `src/application/generation/gate.rs` (`event_record`, `trigger_direction`, `JobWriter::opening_room_report`)
- `src/domain/model/state/trigger_context.rs` (`From<&TriggerMatch> for EventRecord`)
- `src/domain/model/state/swipe_state.rs` (`pre_analysis`)
- `src/domain/model/utils/opening.rs` (opening room report)
- `src/application/games/view_query.rs` (`get_current_room_view` room lookup)

**Problem**
- The Game rules module is shallow at its interface. Its interface is a request enum with three variants. The Job must know which variants to call, in which order, and what to undo after the call. For an event Narration, `Job::apply_rules` takes the movement out of the analysis, calls the rules twice, ignores the Trigger the rules report, and clears `trigger_context` by hand (known).
- The room report rule is in three places: `opening.rs` (create), `JobWriter::save_text` (a Retry of the opening, `tail.count == 1`), and `Job::apply_rules` (`is_the_opening`). The "room of a Swipe state" lookup (map room, then added room) is in four places: `game_rules::room_of`, `Job::room_of`, `JobWriter::opening_room_report`, `GameViewQuery::get_current_room_view` (known).
- The event record is built in two places with two rules. A first fire renders the Trigger direction through `TemplateVars` (`game_rules.rs:119`). A Retrigger copies the raw `narration_prompt` (`gate.rs:587`). The assembler does not render the user message. So a `{{user}}` in a Trigger direction reaches the model raw on a Retrigger only (inferred). No test pins a placeholder in a Trigger direction (known).
- `GameRuleRequest::NpcEvents` has no production caller; only `game_rules_tests.rs` calls it (known).
- `tests/STRATEGY.md` gives the Job no unit tests. So the event rules and the room report rules that sit in the Job have tier-1 HTTP coverage only.

**Solution**
The Game rules take one generation: its kind, the start state, the analysis, the text, the event record and the World. They return the analysed Swipe state, the room report and the event record that fired, with the direction already rendered. The Job calls them once. The Retrigger builds its event record through the same Game rules function. One `SwipeState` room lookup serves every caller.

**Benefits**
- Locality: the event rules, the room report and the event record live in one module. The Retrigger divergence cannot recur.
- Leverage: the Job, the writer and the view query call one interface instead of three variants and four lookups.
- Tests: the rules are a pure `domain/model/` function, so rule 2 of `tests/STRATEGY.md` gives them plain unit tests. Event, opening and Retrigger cases move from tier-1-only to the unit tier. The `NpcEvents` test surface goes.

---

## 2. Delete the in-memory Storage adapter; SQLite `:memory:` is the one stand-in — **Strong**

Dependency category: local-substitutable.

**Files**
- `src/adapters/driven/storage/core.rs` (`Backend::InMemory`, `Storage::new_in_memory`, the `add_failure` placeholder)
- `src/adapters/driven/storage/in_memory_data.rs`
- `src/adapters/driven/storage/utils/records.rs` (`no_in_memory`)
- Every storage file with a `Backend::InMemory` arm: `games.rs`, `worlds.rs`, `presets.rs`, `personas.rs`, `characters.rs`, `settings.rs`, `llm_messages.rs`, `stories.rs`, `docks.rs`, `transaction.rs`
- `src/test_support/fixtures.rs`, about 30 `*_tests.rs` files that call `Storage::new_in_memory`
- `tests/STRATEGY.md` (the unit-tier row names "in-memory `Storage`")

**Problem**
- The seam between the SQLite adapter and the in-memory adapter is not at an interface. It is a `match backend` inside each storage method: 40 `Backend::InMemory` arms in non-test code (known).
- The in-memory adapter is partial. The story reads, the dock read and the transaction refuse on it with `no_in_memory` (10 call sites). So a unit test on the in-memory adapter cannot hold a Message, a Swipe state or a dock record (known).
- A second stand-in already exists and is complete: SQLite `:memory:`. Tier 1 (`tests/http/`), the driven-adapter tier (`tests/storage/`) and `test_support::fixtures` use it (known).
- Deletion test: delete the in-memory adapter and the complexity vanishes. Nothing reappears at the callers, because `:memory:` gives the same interface.

**Solution**
One SQLite adapter. Unit-tier tests that need storage open `DbPool::new(":memory:")`. Each storage method loses its second arm, and `no_in_memory` goes.

**Benefits**
- Locality: one behaviour per storage method. A SQL fix is a fix for every tier.
- Leverage: the storage implementation shrinks; the interface does not change.
- Tests: unit-tier tests read the same SQL that production reads. A unit test can reach the story records.
- Note: `tests/STRATEGY.md` names the in-memory `Storage` as the unit-tier fake. That is a doc decision, not an ADR. The change updates that row.

---

## 3. Deepen the narrator: the narration prompt rules move behind its interface — **Strong**

Dependency category: in-process values in; the LLM port (mock: `ScriptedModel`) behind.

**Files**
- `src/application/pipeline/job.rs` (`narrate_step`, `user_message`, `continuation_user_message`, `previous_input`, `player_text`, `place_before`, `preset`, `budget`, `window_before`)
- `src/application/narration/narrator.rs` (`Narrator::narrate`, `Narrator::assemble`)
- `src/application/prompting/` (`PromptContext`, `PromptAssembler`)
- `src/application/narration/narrator_tests.rs` (the seam contract)

**Problem**
- The narrator is shallow. Its interface takes five prepared arguments (role, `PromptContext`, preset, response length, budget). Its implementation adds one empty-text check to the assembler and the recorder (known).
- The rules of the narration prompt sit in the Job: which preset (System or Impersonate), the user message per kind (Impersonate direction, the Input of a user regen, the continuation prompt of an event, the previous Input of a Continue), and where the history window ends (before the redone Message on a Retry). `tests/STRATEGY.md` gives the narrator a seam contract (`narrator_tests.rs`, 1100 lines), but that contract cannot reach these rules. Only tier-1 HTTP scenarios reach them (known).
- One generation reads the settings four times (`preset`, `budget`, `preset_text` twice), the Game row twice (`narrate_step`, `options_enabled`), and the Tail up to four times (known). A Game config write does not go through the writer, so a mode or preset switch during a Job can give one Job two different Game rows (inferred).

**Solution**
The narrator takes values only, as the `agent-edge` rule of `arch-lint.toml` requires: the generation order, its start state, and one "generation inputs" value that the Job reads once (Game posture and presets, settings, World bundle, history window). The narrator builds the user message and the `PromptContext` itself.

**Benefits**
- Locality: the prompt rules of a generation live in one module.
- Leverage: the Job shrinks to the sequence of steps and the save units.
- Tests: the user-message rules become `CASE`s of the narrator seam contract. One read of the inputs per generation gives one consistent Game row per Job.
- Constraint kept: `arch-lint.toml` `agent-edge` (the narrator does not import storage).

---

## 4. Move the Job plan rules from the writer to the Tail — **Worth exploring**

Dependency category: in-process.

**Files**
- `src/application/generation/gate.rs` (`plan_for`, `generation_plan`, `require_start`, `require_job_data`, `event_record`, `trigger_direction`, `story_has_text`, `opening_message`)
- `src/domain/model/tail.rs` (`redo`, `retrigger`, `retrigger_allowed`, `can_delete`)

**Problem**
- `gate.rs` (910 lines) holds two concerns: the writer rule (job entries, the id check, the save units) and the meaning of a request (the translation from `JobRequest` to `JobPlan`). `Tail` already owns part of the second concern (`redo`, `retrigger`); `GameWriter::plan_for` owns the rest (known).
- The writer rule module is the one place that must stay small and obvious, because its lock order is a stated invariant. Every plan rule there adds code inside the hold (inferred).
- The plan rules have no unit tests: `gate.rs` is neither a box nor a `domain/model/` function (known).

**Solution**
`Tail` answers "what does this request plan" as a pure function that returns the plan or the refusal reason. The writer keeps the reads that need the guard (Tail, Game row, Persona, World direction) and the accept unit.

**Benefits**
- Locality: the request rules sit beside `redo` and `retrigger`.
- Leverage: the writer interface does not change; its implementation shrinks.
- Tests: the plan rules get plain domain unit tests.

---

## 5. One preset module: the preset of a Game for a hook, and the preset edit rules — **Worth exploring**

Dependency category: local-substitutable.

**Files**
- `src/adapters/driven/storage/games.rs` (`active_quantifier_preset_id`, `active_options_preset_id`, `preset_id_or`)
- `src/application/pipeline/job.rs` (`preset`, `preset_text`, `analysis_settings`, `options_settings`)
- `src/domain/model/game.rs` (`active_preset_id`), `src/domain/model/settings.rs` (`active_preset_id`)
- `src/application/prompt_preset_service.rs`
- `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs` (`delete_preset_handler`, `update_preset_handler`, `activate_preset_handler`)
- `src/application/games/catalogue.rs` (`switch_mode`, `set_preset_selection`)

**Problem**
- Three resolution policies exist. A missing narration preset is a failure line. A missing Quantifier or Options preset is a silent "no preset". The storage method falls back to the mode registry when the Game row is gone, which cannot happen inside a Job, because the accept read the row (known).
- The preset rules sit in the HTTP driving adapter: "a default preset cannot be edited or deleted", "a mode default cannot be deleted", "the default Options preset cannot be deleted", "both-false checkboxes keep the stored modes" (known). `PromptPresetService` passes through to storage.
- A delete checks the registry defaults but not the preset slots of the Games, so a Game can point at a deleted preset (inferred).

**Solution**
One preset module answers "the preset text of this Game for this hook" with one policy, and it owns the edit, delete and activate rules. The handlers map its refusals to responses.

**Benefits**
- Locality: one policy for a missing preset.
- Leverage: the Job and the handlers call one interface.
- Tests: the preset rules move from handler tests to the interface of one module.

---

## 6. One analysis output type from the analysis hook to the Game rules — **Worth exploring**

Dependency category: in-process.

**Files**
- `src/domain/model/agent.rs` (`StatePatch`, `Confidence`, `AnalysisOutput`)
- `src/domain/model/quantifier.rs` (`QuantifierResult`, `QuantifierConfidence`, `MovementParseResult`)
- `src/application/agents/runner.rs` (`AgentRunner::analysis`)
- `src/application/agents/quantifier/agent.rs`, `quantifier/utils/orchestration.rs`
- `src/application/pipeline/job.rs` (`apply_rules`)

**Problem**
- The Quantifier makes a `QuantifierResult`. The Quantifier Agent turns it into a `StatePatch`. The runner merges the patches and turns them back into a `QuantifierResult`. The Game rules take the `QuantifierResult` (known).
- `Confidence` and `QuantifierConfidence` are the same three values, with two `From` impls (known).
- The analysis hook has one adapter in production (`QuantifierAgent`), and tests fake the LLM port, not the hook. By "one adapter = hypothetical seam", the hook seam is not yet real. `CONTEXT.md` plans more Agents per hook, so the seam stays; only the round trip goes.

**Solution**
The output type of the analysis hook (`StatePatch`) is the input type of the Game rules. `QuantifierResult` stays private to the Quantifier. One confidence enum.

**Benefits**
- Locality: one type carries the analysis.
- Leverage: candidate 1 gets a smaller interface.
- Tests: no conversion code to test.

---

## 7. The story window: one read for the story log, with the Retrigger rule from the Tail — **Worth exploring**

Dependency category: local-substitutable.

**Files**
- `src/adapters/driving/http/app_state.rs` (`render_story_log`, `MAX_LOG_DISPLAY`)
- `src/adapters/driving/http/templates.rs` (`NarrativeLogTemplate::new`, `show_retrigger`)
- `src/application/games/view_query.rs` (`get_story_log_entries`, `get_debug_state`)
- `src/domain/model/tail.rs` (`retrigger_allowed`)
- `src/adapters/driven/storage/stories.rs` (`entries`, `STORY_LIMIT`), `src/domain/model/agent.rs` (`STORY_WINDOW`)

**Problem**
- `render_story_log` reads the whole story, oldest first, and keeps `.take(50)`. That keeps the 50 oldest Messages, so a story past 50 Messages hides its newest ones, and the Retrigger button lands on the 50th entry (inferred; known from the read order of `StoryReads::entries`).
- The Retrigger rule is in two places with two answers. `Tail::retrigger_allowed` allows a Narration or an Impersonate Input. `NarrativeLogTemplate` shows the button on a Narration only (known).
- The window sizes are chosen at each caller: `STORY_LIMIT`, `STORY_WINDOW`, `5`, `50` (known).

**Solution**
One story-window read returns the newest N entries and the per-entry flags that come from the Tail. The template renders the flags; it does not decide them.

**Benefits**
- Locality: the Retrigger rule has one home.
- Leverage: the story log, the debug state and the Agents ask one interface for a window.
- Tests: one tier-1 scenario pins the window end.

---

## 8. One composition root; the LLM port is the only test input — **Speculative**

Dependency category: ports & adapters (the LLM port has real adapters on both sides).

**Files**
- `src/bootstrap/wiring.rs` (`build_app_graph`, `build_app_graph_for_tests`, `recorder_with_storage`, `options_recorder`)
- `src/test_support/context.rs` (`make_test_job_deps`, `configured_runner`), `src/test_support/test_app_builder.rs`
- `src/adapters/driving/http/app_state.rs` (`job_deps`)

**Problem**
- Tests build a second graph. In `build_app_graph_for_tests`, one recorder serves the narrator and the Quantifier, and a `JobDeps` override replaces the narrator and the Agents. Production builds one resolver per role from the settings (known).
- So tier-1 tests do not run the per-role connection resolution of production (inferred).
- `JobDeps` is a bag that a handler must pair with `GameWriter` to start a Job (`Job::start(&state.game_writer, state.job_deps.clone(), ...)`) (known).

**Solution**
One `build_app_graph` takes a provider factory. Production passes the settings-driven factory; tests pass a `ScriptedModel` factory. The Job start takes only the Game and the request.

**Benefits**
- Locality: one wiring for both.
- Leverage: test builders shrink.
- Tests: tier 1 runs the production wiring.

---

## Top recommendation

**Candidate 1 — deepen the Game rules into one call per generation.** It is a pure in-process change, so it gets plain unit tests by rule 2 of `tests/STRATEGY.md`. It removes the largest spread of rules (event semantics, room report, event record) from the Job and the writer. It closes an untested divergence (the unrendered Retrigger direction). It also gives candidates 3, 4 and 6 a smaller interface to build on.
