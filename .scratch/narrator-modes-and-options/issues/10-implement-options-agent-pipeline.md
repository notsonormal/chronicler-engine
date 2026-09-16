# Task: Options agent + pipeline — OptionsAgent, Action::Options, output parsing, current-set state, always-on hook

Type: task
Status: resolved
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

## Answer

Shipped. The options generation mechanism is in place and build-green; ticket 11 (UI dock) is unblocked.

### What landed

1. **`OptionsAgent`** (`src/application/agents/options/` — agent.rs, prompt.rs, types.rs, utils/{orchestration,parser}.rs, mirroring the quantifier's layout): `name() = "options"`, `backend_selector() = UseNamed("options")`, registered in `AgentRegistry::from_configs_with_storage` behind the `enabled` config flag. `AgentConfig::defaults()` gains an options entry (enabled), so settings.toml files without an `[agents]` section get it; settings WITH an `[agents]` section lacking `options` do not (quantifier precedent — recorded below as the no-agent behavior).
2. **Result shape (the scope-2 DECIDE): a new `AgentResult::Options(Vec<String>)` variant.** The side-channel alternative was impossible without changing `AgentContext`'s `&GameState` to `&mut`. The generic PostGeneration merge loop ignores the variant (unreachable there — see the phase decision).
3. **Dispatch (deviation from this ticket's `PostGeneration` text, grilled as Q1=A): a third `ExecutionPhase::OptionsGeneration` variant.** Registering the agent as PostGeneration would have fired it inside `run_post_generation_agents` on every turn — impersonate turns, trigger reconciles, and with the toggle off — directly contradicting ticket 04's decisions 4/5. The pipeline dispatches `agents_for_phase(OptionsGeneration)` explicitly at exactly two gated sites: the turn-end rewrite in `run_from_input` (and the event-retry tail), and the `/options` entry path. Recorded here as the resolution of that deviation.
4. **Prompt + parsing**: options preset resolved game-first via a new `Storage::active_options_preset_id` accessor (fallback `AppSettings.active_options_prompt_preset_id` — options sit outside the per-mode registry); preset text rendered through `TemplateVars` — now carrying `{{option_count}}` (domain const `DEFAULT_OPTION_COUNT = 3`); scene context = CurrentRoom + NpcsInArea + RecentHistory (last 4, quantifier-style). Parser (`utils/parser.rs`, regex crate) accepts BOTH seed shapes in priority order — `<suggestion>` tags → numbered list → `Suggestion N:` — first matching shape wins, trimmed to count; zero parse after 2 attempts (quantifier's retry pattern) surfaces an error to the caller, never a failed turn.
5. **Backend routing**: wiring resolves `settings.find_connection("options")` to a dedicated recorder (Roadway's cheap-model seam — a connection with id `options`); fallback is the narrator recorder (this ticket's fallback rule). No schema change, no settings UI (a later concern if wanted).
6. **`Action::Options`** (grilled Q4=A): `/options` with no argument parses to `Action::Options`; `/options anything` falls through to `FreeAction` like any unknown slash command. `is_engine_command()` added (guide/impersonate/options) and used by `action_check_handler` so engine commands bypass the player-input text check; `is_steering()` semantics unchanged. Entry path `process_options` claims the generation gate (mirrors `retry()`), validates scene history (new additive `EngineError::Validation(String)` — EngineError had no validation variant and `ApplicationError::validation` was unreachable through `process_action`'s error type), and runs `execute_options_refresh` with `GenerationPhase::Options` ("Generating options...").
7. **Offered-set state**: `NarrativeState.current_options: Vec<String>` + `NarrativeSnapshot.current_options` (both `#[serde(default)]` — old snapshots deserialize empty, unit-tested) + `DebugStateView.current_options`. Persisted via snapshots; survives reload mid-turn (ticket 04 decision 8).
8. **Lifecycle (grilled Q3=A)**: every non-impersonate narration turn rewrites the set at turn end — clears, then regenerates when the game's `options_always_on` toggle (game-first, world fallback via `PipelineRun::resolve_options_always_on`) is on; a failed regeneration leaves the set empty plus a system message. Impersonate turns and error/cancelled runs touch nothing. On-demand `/options` replaces on success and KEEPS the prior set on failure (scene unchanged). Event-only retry follows the same rewrite rule. No-agent-registered: on-demand surfaces "[System] Options agent is not available" and keeps the set; always-on surfaces the same message with the set cleared.

### Known gaps (recorded, deliberately deferred)

- **Arrival narration never fires always-on options** (grilled Q2=A): the boot-time `ArrivalTaskContext` has no registry handle or gate claim; plumbing it in is scope creep. The player can press `/options` on the first scene. Candidate for a later small ticket.
- Swipe switches and retry rewinds restore `current_options` from the point-in-time snapshot (consistent with the swipe model's rewind of `npcs_in_area`); regeneration costs one `/options`.
- Options-preset deletion is still not reference-checked against the options preset ids (ticket 09's carried edge, mirrored).
- `EngineError::Validation` renders through `dispatch_action` as a 500 rather than 400 (retry's `ApplicationError::validation` gets 400 via its own endpoint); cosmetic, noted for the HTTP error-mapping pass.

### Build

`python build.py` fully green: fmt, clippy, test-structure, docstrings, docs validation, architecture (all OK); guardrails 129/129; tests 1571 passed, 0 failed, 2 skipped. Guardrail-driven restructure during the gate: free fns relocated to `options/utils/{orchestration,parser}.rs` (free-fn-location rule) and the pipeline options methods consolidated into `action_pipeline/options.rs` (test-file-location rule); `TRIVIAL_ENUM` variant docs removed; the LLM retry loop flattened to depth ≤ 3.

### Unblocks

11 (options UI) — now the frontier. 12 waits on 11.
