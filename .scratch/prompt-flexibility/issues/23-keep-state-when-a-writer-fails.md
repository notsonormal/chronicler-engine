# Keep the state when a state writer fails

Type: task (AFK)
Status: open
Blocked by: —

## Question

Make the `Err` path of `AgentRunner::after_narration` keep the previous state instead of clearing the Characters present. Rewrite its doc comment, and change the banner text. Source: [Decide what the state does when a state writer fails](15-failed-state-writer.md), rules 1 and 3 to 6.

Done when a failed after-narration Agent leaves the Room and the Characters present, the banner reads "Quantifier failed — keeping the Room and the Characters present", and `python build.py` passes.

## Decisions

From [Decide what the state does when a state writer fails](15-failed-state-writer.md): keep the previous state, and continue the Job. The 2 attempts stay. The banner text changes. Every future state writer uses the same rule. The work is test first.

Choices made when the work was ticketed ([Write the build plan for the engine changes](09-build-plan.md)), corrected by the plan review of 2026-10-31:

- **One domain function holds the keep rule:** `QuantifierResult::keeping_present(present: &[CharacterCard])` in `src/domain/model/quantifier.rs`. It gives the ids of the Characters present, in order, and no destination. The name says what it keeps. The `Err` arm returns it, and keeps its `tracing::warn!`.
- **The test reaches the `Err` arm** with a real `QuantifierAgent` and a `HookInput` with no text. A Job never builds that input, so a comment in the test says why it is used. No model is needed: the Agent fails before any call.
- **No spec scenario.** A Job cannot reach the `Err` path (inferred), so no HTTP test can see it. The legacy unit test in `runner_tests.rs` is the only place for it.
- **The banner on the `Err` path** (owner, 2026-10-31, O1 (a) in [Settle the open questions of the engine tickets](17-settle-engine-questions.md)): keep the state and log the fault, with no banner. The banner reads only LLM messages, and the `Err` path makes no model call. A record of a call that never happened would make the debug record false.
- **No Quantifier registered** (owner, 2026-10-31, O2): the same keep rule. A missing writer leaves the state as it was. Effect: a Game whose `agents` setting disables the Quantifier keeps its Characters present for ever. No test covers a disabled Quantifier today.

## Facts

Known on 2026-10-31 (inferred where marked).

- `AgentRunner::after_narration` (`src/application/agents/runner.rs`) returns `QuantifierResult::default()` in two cases: no after-narration Agent is registered, and the Agent returns `Err`. The doc comment states this ("the Job continues with no Character present and no destination"). It becomes wrong with this change, so it must be rewritten.
- `game_rules::apply` sets the Characters present from the result, so the default result empties them. `apply` keeps only ids that the World holds, and it runs the Triggers on the new state.
- `QuantifierResult` is `{ present: Vec<String>, destination: Option<String> }`, with no methods today. `HookInput.text` is an `Option<&str>`. `HookInput.world.present` is `&[CharacterCard]`.
- A model failure does not take this path. It gives a `Low` reading, and `static_character_result` (`quantifier/utils/orchestration.rs`) keeps the Characters present (unit test `test_determine_characters_backend_error_fallback`).
- The `Err` path comes from two preconditions in `quantifier/utils/orchestration.rs`: `EngineError::Config("the Quantifier needs the saved text")` and `EngineError::RoomNotFound`. Both come before any model call, so the path writes no LLM message. The banner reads only the newest LLM message of each Role (`GameViewQuery`).
- (inferred) A Job cannot reach the `Err` path. The Job always passes the saved text, and `Job::narrate` already fails on a Room that is not on the map.
- `QuantifierAgent::from_config` needs only a recorder. `runner_tests.rs` has `mock_recorder()`.
- The banner text "keeping the Characters present" is in `role_failure_effect` (`src/adapters/driving/http/builders/headers.rs`), in a log line of the model-failure path in `quantifier/utils/orchestration.rs`, and in one assertion: `test_degraded_role_raises_header_banner_and_clears_on_recovery` in `tests/http/failure_display.rs` (scenario 38.1). The spec says only "its failure effect", so it needs no change. The stub browser test asserts only "Quantifier failed".
- `runner_tests.rs` is on the legacy list. It has no test of the `Err` path. It must carry no `CASE` tag, because it is not a seam contract file.
- `src/domain/model/quantifier_tests.rs` exists and is registered.
- `docs/diataxis/reference/game_flow.md` already states that each Swipe holds its own Swipe state, and that a Retry starts from the start state of its Message.

## What already exists

| Need | Reuse |
|---|---|
| The "keep the Characters present" value of a model failure | `static_character_result` (`quantifier/utils/orchestration.rs`). It can build on `keeping_present`, but the goal does not need that |
| Test Characters | `TestCharacter::named` (`src/test_support/fixtures.rs`) |
| A `WorldView` and a story row in a test | `TestMap`, `TestPersona`, `TestCharacter` and `default_story_row` in `test_support::fixtures`, as `quantifier/agent_tests.rs` uses them |
| A recorder with no model | `mock_recorder()` in `runner_tests.rs` |

## Steps

1. **Banner text.** In `test_degraded_role_raises_header_banner_and_clears_on_recovery` (`tests/http/failure_display.rs`), change the expected effect to "keeping the Room and the Characters present". Run `python build.py test-pattern "degraded_role_raises_header_banner" --test-target http` and see it fail. Then change `role_failure_effect`, and the log line in `quantifier/utils/orchestration.rs` so the two agree.
2. **The keep rule.** Add `QuantifierResult::keeping_present`, with tests in `src/domain/model/quantifier_tests.rs`: the ids in order, and no destination. Run `python build.py test-pattern "quantifier" --test-target lib`.
3. **The `Err` path.**
   - In `runner_tests.rs`, add one test with no `CASE` tag. Build a `QuantifierAgent` with `mock_recorder()`, and a `HookInput` with no text from the fixtures. `after_narration` gives the ids of `input.world.present`. Add a comment: the input has no text because only that reaches the `Err` arm, and a Job never builds it.
   - Make the `Err` arm return `QuantifierResult::keeping_present(input.world.present)`.
   - Rewrite the doc comment in one sentence: a failed Agent is logged, and the Job continues with the Room and the Characters present of the previous state.
   - The "no Agent registered" arm returns the same value (O2). Add a second test.
   - Run `python build.py test-pattern "runner" --test-target lib`.
4. **Docs.** `docs/diataxis/reference/narrative/agent_system.md`, "Hooks": a failed after-narration Agent keeps the Room and the Characters present, and the Job continues. Every state writer follows this rule.
5. Run `python build.py`.

## Failure modes

| Code path | Failure | Result | Handled by |
|---|---|---|---|
| The `Err` arm | A wiring fault | The state is kept, a log line names the fault, and no banner shows. The banner can still show an older failure, now in the new words | O1 (a) |
| The kept state | A kept id names a Character that the World no longer holds | `game_rules::apply` drops it | Existing code |
| The kept state | A repeatable Trigger matches the kept state | It can fire again after a failed writer | Accepted. A model failure already does the same |
| The "no Agent" arm (O2) | The `agents` setting disables the Quantifier | The Characters present never change | Accepted (O2). The answer records that no test covers it |

## Not in scope

- A Swipe rule for memory. `game_flow.md` already states it for the Swipe state. The long-term memory effort states it for memory ([Decide whether this map designs memory and state, or only keeps room for them](16-memory-and-state-scope.md), room 4).
- The model-failure path. It already keeps the state, and its 2 attempts stay.
- A banner for the `Err` path (O1).

## Output

Resolved when `python build.py` passes.
