# Task: Posture relocation — narration + pipeline (read posture + presets from the game)

Type: task
Status: pending
Blocked by: 05, 14

## Question

Implement the narration-path half of the posture relocation: the narrator and agents read posture + preset-ids from the game, not from global `AppSettings`. This is implementation per the map's Notes override.

### Scope

1. **Assembler reads posture from the game.** `PromptAssembler::assemble` (`src/application/prompting/assembler.rs:85-87`) today does `set_narrative_voice(settings.narrative_perspective, settings.narrative_tense)`. Change it to read posture from the game context. The assembler keeps reading `AppSettings` ONLY for the token budget (`resolve_budget`); the per-mode registry lookup is NOT needed here (the pipeline resolves the preset-id, not the assembler).

2. **Arrival service reads posture from the game.** `src/application/arrival_service.rs:149` — same change: posture from the game, not `settings`.

3. **Pipeline resolves the system preset from the game's preset-id.** `src/application/pipeline/pipeline_run.rs:495` today reads `settings.active_system_prompt_preset_id`. Change to read the game's `active_system_prompt_preset_id`.

4. **Quantifier agent reads the quantifier preset from the game's preset-id.** `src/application/agents/quantifier/agent.rs` today reads `settings.active_quantifier_prompt_preset_id`. Change to read the game's id.

5. **Impersonate preset selection reads from the game.** `src/application/pipeline/pipeline_run.rs:528` today reads `settings.active_impersonate_prompt_preset_id`. Change to the game's id.

## Notes for the session

- Read before implementing: `src/application/prompting/assembler.rs`, `src/application/arrival_service.rs`, `src/application/pipeline/pipeline_run.rs` (`phase_narrate`, `call_narrator`, `load_system_preset_and_response_length`, the impersonate preset path), `src/application/agents/quantifier/agent.rs`. Read the `*_tests.rs` beside each.
- The game's posture + preset-ids must thread through to these call sites. Decide the threading (pass the game handle, or a resolved posture struct) during implementation; keep the seam clean.
- Build-green: novel mode must stay green; IF mode end-to-end still pending ticket 02.
- Blocked by 05 (the game row must carry the fields first).
- Skills: `/domain-modeling`.
