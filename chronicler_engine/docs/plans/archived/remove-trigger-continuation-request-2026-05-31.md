# Remove Thin Abstraction TriggerContinuationRequest

**Status:** ✅ COMPLETED  
**Date:** 2026-05-31  
**Original Location:** `local://remove-trigger-continuation-request.md`

## Problem
`TriggerContinuationRequest` is an identity wrapper around `StoredTriggerContext` with zero semantic value. It adds cognitive overhead (new type name, `.stored` field accessor) without providing abstraction, validation, or behavior.

## Current State (Before Removal)
- **Definition:** `chronicler_engine/src/engine/action_processing.rs:30-32`
- **No impls:** Zero methods, zero trait implementations beyond default struct behavior
- **No derives:** Missing even `Debug`, `Clone`
- **Usage pattern:** All 10 call sites access `request.stored.*` directly

## Solution
Remove `TriggerContinuationRequest` struct entirely. Update all callers to use `StoredTriggerContext` directly.

## Files Modified

1. **chronicler_engine/src/engine/action_processing.rs**
   - ✅ Removed `TriggerContinuationRequest` struct definition (lines 29-32)
   - ✅ Updated `commit_trigger_narration()` signature to accept `&StoredTriggerContext`
   - ✅ Updated all field accesses from `request.stored.*` to `trigger.*` (4 occurrences)

2. **chronicler_engine/src/application/action_pipeline/pipeline.rs**
   - ✅ Removed `TriggerContinuationRequest` from imports (line 7)
   - ✅ Updated `phase_trigger_continuation()` parameter to `trigger: &StoredTriggerContext` (line 297)
   - ✅ Updated field accesses in `phase_trigger_continuation()` (3 occurrences: `system_prompt`, `user_prompt`, `max_tokens`)
   - ✅ Updated `build_trigger_request()` return type to `Option<StoredTriggerContext>` (line 608)
   - ✅ Removed wrapper construction, return `StoredTriggerContext` directly (lines 642-653)
   - ✅ Updated call site to pass `trigger` directly instead of wrapper (lines 462-465)
   - ✅ Updated call site in `phase_trigger_continuation()` invocation (line 147)

3. **chronicler_engine/src/engine/action_processing_tests.rs**
   - ✅ Removed `TriggerContinuationRequest` from imports (line 2)
   - ✅ Updated 6 test call sites (lines 286, 320, 344, 368, 382, 399)
   - ✅ No logic changes — only removed `{ stored: ... }` wrapper

## Verification Results
✅ `cargo check` - Clean compilation  
✅ `cargo test` - 947 tests passed (19 suites)  
✅ `python build.py` - All 9 steps passed  
✅ No references to `TriggerContinuationRequest` remain in codebase  

## Trade-offs Considered

### Arguments for keeping wrapper
- **Semantic naming:** Makes it clear this is a "request" vs. raw "context"
- **Future extensibility:** Could add fields later without changing function signatures

### Why removal wins
- **Zero current value:** No methods, validation, or encapsulation
- **Cognitive overhead:** Developers must learn `.stored` accessor pattern
- **Violates "don't pay for what you don't use":** Every caller pays the accessor cost
- **Naming without benefit:** Function parameter name `trigger: &StoredTriggerContext` is equally clear
- **Extensibility fallacy:** If future needs require a request type, it can be added then with actual value

## Changes Summary
- **Remove:** 1 struct definition (4 lines)
- **Modify:** 3 files (action_processing.rs, pipeline.rs, action_processing_tests.rs)
- **Call sites:** 10 total (4 production + 6 tests)
- **Behavioral impact:** Zero — pure renaming/refactoring change

## Notes
- `StoredTriggerContext` already has all necessary derives: `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`
- `NarrativeState.last_trigger` was already `Option<StoredTriggerContext>` — no wrapper used there
- Function parameter names are sufficiently descriptive without the wrapper type
