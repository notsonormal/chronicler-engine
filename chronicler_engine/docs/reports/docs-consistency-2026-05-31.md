# Documentation Consistency Report

**Date:** 2026-05-31  
**Branch:** feat/forensics  
**Status:** ✅ PASS (with fixes applied)

## Summary

Reviewed documentation consistency with uncommitted code changes in the `feat/forensics` branch. Found and fixed **4 documentation inconsistencies** related to the action pipeline API refactoring and tracing improvements.

---

## Documentation Inconsistencies Fixed

### 1. ARCHITECTURE: Application Tier API
- **FILE:** `chronicler_engine/docs/architecture/system.md` (line 45)
- **Type:** Ghost Features / Outdated Patterns
- **Issue:** Documentation claimed callers invoke `execute_action_impl()`, `retry_last_response_impl()`, `retrigger_event_impl()` directly from the `action_pipeline` module
- **Current:** `DefaultGameService` exposes public wrapper methods `execute_action(ctx, input, player_name)` and `retry_last_response(ctx)`. Internal `*_impl` functions are private to the action pipeline module.
- **Expected:** Documentation should describe the public API boundary correctly
- **Fix Applied:** ✅ Updated to clarify that `DefaultGameService` methods are the public API and `*_impl` functions are internal implementation details

### 2. REFERENCE: Testing Documentation API Examples
- **FILE:** `chronicler_engine/docs/reference/testing.md` (lines 65-77)
- **Type:** Ghost Features / Missing Concepts
- **Issue:** Test examples showed direct calls to `execute_action_impl()` without clarifying the public API boundary
- **Current:** Tests should call `DefaultGameService::execute_action()` wrapper method; `execute_action_impl()` is internal
- **Expected:** Examples should use the public API and explain when internal functions are appropriate
- **Fix Applied:** ✅ Updated dependency injection example to use `service.execute_action()` and added explicit note about not calling `execute_action_impl()` directly except in narrow mock scenarios

### 3. REFERENCE: Pipeline-Only Mocking Examples
- **FILE:** `chronicler_engine/docs/reference/testing.md` (lines 76-98)
- **Type:** Specificity / Missing Concepts
- **Issue:** Pipeline mocking example showed trait implementation but didn't show how to use it with the internal `execute_action_impl()` function
- **Current:** Narrow mocks implementing `ActionPipelineBackend` can be used directly with `execute_action_impl()` for unit testing the pipeline
- **Expected:** Example should demonstrate the complete usage pattern
- **Fix Applied:** ✅ Added import for `execute_action_impl`, added `GameServiceContext`, and showed complete usage example

### 4. SYSTEM: LLM Processing Tracing Documentation
- **FILE:** `chronicler_engine/docs/system/llm_processing.md` (lines 144-147)
- **Type:** Wrong Signatures / Outdated Patterns
- **Issue:** Listed `execute_action_impl`, `retry_last_response_impl`, `retrigger_event_impl` as instrumented functions without context about the new phase-based pipeline structure
- **Current:** Pipeline uses `ActionPipeline::run_from_input()`, `ActionPipeline::run_trigger_continuation()`, and `phase_*` methods with debug-level tracing (not info-level)
- **Expected:** Documentation should reflect the phase-based pipeline and correct tracing level
- **Fix Applied:** ✅ Updated to describe the phase-based methods and clarified that tracing is at debug level to reduce noise

---

## Documentation That Remains Accurate

### ADR-014: Action Pipeline Architecture
- ✅ Correctly describes the `ActionPipelineBackend` trait and its benefits
- ✅ Minor enhancement applied: added note about public API methods in Consequences section

### System Documentation (triggers.md, game_flow.md)
- ✅ Trigger evaluation flow documented accurately
- ✅ Early-return guard pattern implementation detail doesn't change the documented behavior
- ✅ Mutation order invariant still correctly documented

### Architecture Documentation (system.md)
- ✅ Module structure accurately reflects the codebase
- ✅ Application tier responsibilities correctly described

---

## Code Changes Verified

### Action Pipeline API Changes
- `DefaultGameService::execute_action()` — public wrapper ✅
- `DefaultGameService::retry_last_response()` — public wrapper ✅
- `execute_action_impl()` — internal implementation detail ✅
- `retry_last_response_impl()` — internal implementation detail ✅

### Tracing Improvements
- Changed from `tracing::info!` with `[DEBUG]` prefix to `tracing::debug!` without prefix ✅
- Reduces log noise while preserving diagnostic capability ✅
- Affected files: `pipeline.rs`, `logging.rs` ✅

### EnvFilter Error Handling
- Changed from `println!` to `eprintln!` for error output ✅
- Better practice for error messages ✅

### Trigger Evaluation Pattern
- Refactored from nested if-let to early-return guards ✅
- Improved readability and debug tracing integration ✅
- Behavior unchanged (documented flow still accurate) ✅

---

## Historical Documents (No Action Needed)

The following documents describe past states and are correctly framed as historical:

1. **CHANGELOG.md** — Documents the evolution of the codebase, including the refactoring that moved `#[instrument]` to `*_impl` functions. This is historical context, not current API documentation.

2. **docs/plans/archived/remove-identity-wrapper-functions-2026-05-31.md** — Archived plan describing the identity wrapper removal. Correctly archived and labeled as historical.

3. **docs/plans/observability-and-forensics-plan.md** — Forward-looking plan document mentioning `tracing::info!`/`warn!` as general instrumentation guidance. Not a claim about current implementation specifics.

---

## Conclusion

All **active documentation** (architecture specs, system docs, reference docs, ADRs) is now **consistent** with the uncommitted code changes. Documentation correctly describes:

1. The public API surface (`DefaultGameService::execute_action()` and `retry_last_response()`)
2. The internal implementation boundary (`execute_action_impl()` and `retry_last_response_impl()` as private)
3. The phase-based pipeline structure with debug-level tracing
4. Test patterns using both the public API and narrow mocks

**No blocking inconsistencies remain.** Documentation accurately reflects the current state of the codebase.

---

**Documents Updated:**
- `chronicler_engine/docs/architecture/system.md`
- `chronicler_engine/docs/reference/testing.md`
- `chronicler_engine/docs/system/llm_processing.md`
- `chronicler_engine/docs/adr/adr-014-action-pipeline.md`

**Verification Method:**
- Compared uncommitted git diff against all active documentation
- Validated API signatures and behavioral descriptions
- Ensured test examples match current patterns
- No build or test execution required (documentation-only changes)
