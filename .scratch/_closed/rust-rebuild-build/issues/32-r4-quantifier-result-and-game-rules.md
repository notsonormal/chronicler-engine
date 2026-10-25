# R4: the Quantifier result and the Game rules

Type: task (AFK)
Status: resolved
Blocked by: 31, 28

## Question

Build §6 of [the build plan of the response](../../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §6 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §6 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built on `wf/rebuild-response` in one commit, after a green full gate (`logs/build_20261021_015920.log`: 189 guardrail, 1339 integration, 29 browser tests pass).

- `QuantifierResult { present, destination }` is the result of the after-narration hook. `StatePatch`, `AnalysisOutput`, `merge`, the second confidence enum, `SceneState.quantifier_confidence` and `DebugStateView.quantifier_confidence` are gone. The confidence and the parse results are private to the Quantifier.
- Hooks by position: the traits `AfterNarrationAgent` and `EndOfJobAgent`, and `AgentRunner::after_narration` and `AgentRunner::end_of_job`. The `Hook` enum is deleted: after `AgentConfig.hook` went (finding 13), nothing read it. The runner holds one Agent per hook. A second config of the same hook replaces the first and logs a warning (lead; `data/settings.json` has one Agent per type).
- `game_rules::apply(state, &QuantifierResult, &GameWorld, Kind) -> Result<RulesOutcome>` has one production caller (`job.rs`). An event has no movement, no Trigger check and no trigger context. The fired record's direction is rendered once. `WorldBundle::game_world()` in storage builds `GameWorld`.
- The trigger context holds `EventRecord`; migration v30 deletes the saved Games. `Tail::retrigger` copies the record from the trigger context, so no World read is left (the plan placed the copy in `gate.rs`; R5 moves `Tail::retrigger` into `JobPlan::for_request`).
- Scenario 13.3 in `retrigger.md`: a Retrigger sends the rendered direction to the model.
- Left for R5: `SwipeState::pre_analysis`, `ScriptedModel::analysis` and the term "pre-analysis state".
