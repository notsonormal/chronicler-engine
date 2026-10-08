# Final re-review of the dashboard

Type: task (AFK)
Status: open
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70

## Question

After every other ticket closes, does a fresh review, using the same method as the [first review](../review-2026-09-29.md), find any new P1 or P2?

## Context

- This is the map's finish line.
- Every ticket added to the map later must also be added to this `Blocked by:` line.
- Also check the fog item "browser suite cost": compare the browser binary's run time in `logs/build_*.log` against the 2026-09-29 baseline (browser 21 passed), and decide whether it needs action.
- The new tickets 51–54 add banner, poll-failure, status-clamp and toast-retirement coverage; the browser suite grew again. Fold that into the cost check above.

## Post-implementation review rulings

The after-plan review of the branch found no new P1/P2. Two P3s were fixed in the same pass: the Settings panel now renders its refusal through the shared disclosure (ticket 63's shape), and scenario 31.7 plus its browser test were deleted (the test stages the `#action-area` swap it asserts, and no shipped code performs that swap). The remaining P3s are ruled out of scope for this map:

- **Two inert route-boundary guardrails.** `check_handler_return_type` matches `src/server/` paths and `check_server_layer_boundaries` matches `server/`, but the gate strips the `src/` prefix (`tests/infrastructure/guardrails/mod.rs:91-93`) and the layer is now `adapters/driving/http/`, so neither rule can fire. Ruled out here: re-pointing them redraws what the gate watches and may fail it on unfixed handlers, which is larger than this map. Owner: the guardrail suite. The same-class `check_test_layer_boundaries` was deleted by this map, because it could never fire and no rule intent was lost.
- **Panel wipe on a failed preset add or activate.** Ticket 64's rule covers its four paths; `duplicate_preset_handler` (`prompt_presets/handlers/prompt_presets.rs:338,343`) and `activate_preset_handler` still answer a 200 fragment that replaces the whole panel. Owner: `.scratch/architecture-deepening/issues/15-single-fragment-error-policy.md` (open), which decides the single fragment error policy.
- **`isPollRequest` reports a server-answered 500 as "unreachable".** Scenario 16.18 deliberately pins the banner for a failed poll, so the behaviour is intended. Only the heuristic (`assets/index.html:562-566`) and the wording are open, and a wrong sentence is not a defect this map must close.
- **Panel padding ships as 48px, not the declared 24px.** Every panel nests a same-class fragment root inside a same-class shell div, so the padding applies twice. Pre-existing on `main` and not a regression from this map. Ruled out: the fix reshapes the CSS of four panels with no test coverage. Tickets 19 and 67 keep the 24px convention as the intent; this line records the deviation.

## Done when

- The review finds no new P1/P2. Any new P3 is fixed or ruled out of scope.
- `python build.py` passes.
