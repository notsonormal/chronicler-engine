# Plan: Remove Legacy Synchronous Dead Code (`evaluate_and_narrate_triggers`)

## Problem
`src/engine/action_processing.rs` contains `evaluate_and_narrate_triggers()`, a synchronous blocking LLM call that is dead code in production. The current pipeline (`execute_freeaction_pipeline` in `actions.rs`) uses the split architecture:
1. `execute_freeaction_impl()` evaluates triggers and builds a `TriggerContinuationRequest` (no LLM call)
2. LLM call happens asynchronously outside the state lock
3. `commit_trigger_narration()` applies the result

The old function is only kept alive by a single test. This creates confusing overhead because two completely different trigger execution architectures exist side-by-side.

## Scope
- **Remove** `evaluate_and_narrate_triggers()` from `action_processing.rs`
- **Update** the doc comment on `build_trigger_prompt_parts` (no longer shared with the removed function)
- **Rewrite** `test_evaluate_and_narrate_triggers_adds_event_header` to exercise the new split architecture (`build_trigger_request` + `commit_trigger_narration`)
- **Update** `docs/architecture/system.md` to remove the reference to `evaluate_and_narrate_triggers`
- **Validate** with `cd chronicler_engine && python build.py`

## Files to Modify

### 1. `chronicler_engine/src/engine/action_processing.rs`
- Delete the `evaluate_and_narrate_triggers` function (lines ~201-274)
- Update doc comment on `build_trigger_prompt_parts` from:
  `/// Shared by `build_trigger_request` and `evaluate_and_narrate_triggers`.`
  to:
  `/// Shared helper for building trigger continuation prompt parts.`

### 2. `chronicler_engine/src/engine/action_processing_tests.rs`
- Remove `evaluate_and_narrate_triggers` from the `use` import (line ~276)
- Rewrite `test_evaluate_and_narrate_triggers_adds_event_header` to:
  1. Set up state with an NPC that has a `TimesMet Eq 0` trigger (same as now)
  2. Call `build_trigger_request` (or `execute_freeaction_impl` with a `FreeActionContext`) to get a `TriggerContinuationRequest`
  3. Call `commit_trigger_narration` with a mock continuation text
  4. Assert the state has the expected narration log with `event_header` set

  The test already asserts the same outcome; we just change the execution path to use the production architecture.

### 3. `chronicler_engine/docs/architecture/system.md`
- In the `action_processing` bullet (line ~28), change:
  ``(`handle_movement`, `apply_npc_events`, `evaluate_and_narrate_triggers`, `commit_trigger_narration`, `execute_freeaction_impl`)``
  to:
  ``(`handle_movement`, `apply_npc_events`, `commit_trigger_narration`, `execute_freeaction_impl`)``

## Success Criteria
- [ ] `evaluate_and_narrate_triggers` no longer exists in the codebase
- [ ] `python build.py` passes (fmt + clippy + tests + coverage)
- [ ] The rewritten test still validates that trigger firing produces a narration log with the correct `event_header`

## Single Approach
This is a straightforward dead-code removal with test migration. There is only one sensible path: delete the old function and rewrite the dependent test to use the current production APIs.
