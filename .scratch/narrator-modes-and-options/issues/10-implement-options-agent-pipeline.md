# Task: Options agent + pipeline — OptionsAgent, Action::Options, output parsing, current-set state, always-on hook

Type: task
Status: pending
Blocked by: 09, 14

## Question

Implement the generation mechanism for options-autogeneration as decided in ticket 04 (decision 5): an `OptionsAgent` mirroring the quantifier, serving BOTH triggers — always-on (post-narration hook) and on-demand (`/options` slash command). This is implementation per the map's Notes override.

### Scope

1. **`OptionsAgent`** in `src/application/agents/options/` (mirror `src/application/agents/quantifier/`): implements the `Agent` trait (`src/application/agents/trait_def.rs`) with `phase() = ExecutionPhase::PostGeneration` and `backend_selector() = BackendSelector::UseNamed("options")` — the named-backend seam that allows routing options to a cheap model (ticket 03's Roadway two-model pattern). Fall back to the narrator backend when no "options" backend is configured. Register `"options"` in `AgentRegistry::from_configs_with_storage` (`src/application/agents/registry.rs`) with an `enabled` config flag.

2. **Result shape.** The quantifier returns `AgentResult::StatePatch`; options produce a list of strings. Extend `AgentResult` with a new variant (or write the set to game state as a side-channel) — DECIDE during implementation, record the choice in the resolution comment. The offered set lands on game state as the CURRENT options set (e.g. `state.narrative.current_options: Vec<String>`), replacing any previous set wholesale (ticket 04 decision 8: current-state artifact, no history).

3. **Prompt assembly.** Load the game's active options preset (ticket 09), substitute `{{option_count}}` (default 3), build scene context from the same `PromptContext` inputs the quantifier sees (current room, NPCs in area, recent history). Options are "what the player could do next" — possible player actions, not story branches (ticket 04 decision 3).

4. **Output parsing.** Parse the LLM output into N option strings, accepting BOTH seed shapes: `<suggestion>...</suggestion>` tags AND numbered lists (`1. ...`). Ticket 03's CYOA parser regex is the reference. Reject/trim to the requested count; on unparseable output, surface a system message rather than failing the turn (mirror the quantifier's uncertain-detection fallback).

5. **`Action::Options`.** Add the variant to `src/domain/model/action.rs` and parse `/options` in `Action::parse` (no argument). Add a pipeline entry path (`src/application/pipeline/action_pipeline/action.rs`, mirroring `impersonate()`) that runs the OptionsAgent against the current scene WITHOUT narrating, and commits the new set to state.

6. **Always-on hook.** After a narration-producing turn commits (in or after `phase_post_generation`, `src/application/pipeline/pipeline_run.rs`), if `game.options_always_on` is set, run the OptionsAgent. Do NOT fire after impersonate turns (ticket 04 decision 4 — an impersonate already IS the player acting).

## Notes for the session

- Read before implementing: ticket 04's Answer; `src/application/agents/quantifier/agent.rs` + `prompt.rs` (the template to mirror); `src/application/agents/registry.rs`; `src/domain/model/agent.rs` (`AgentResult`, `BackendSelector`, `ExecutionPhase`); `src/domain/model/action.rs`; `src/application/pipeline/pipeline_run.rs` (`phase_post_generation`); `src/application/pipeline/action_pipeline/action.rs` (the `impersonate()` entry-path pattern); `research/03-option-generation-prior-art.md` §1b (parsing regexes, prompt shapes).
- Unit tests beside the new modules (parser especially — both tag and numbered-list shapes, wrong-count, unparseable); register them per `scripts/check_test_structure.py`.
- Build-green check: `python build.py` (fast suite). The UI (ticket 11) does not exist yet — verify via unit/integration tests, not the browser.
- Skills: `/domain-modeling`.
