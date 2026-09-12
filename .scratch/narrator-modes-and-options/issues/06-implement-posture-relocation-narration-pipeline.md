# Task: Posture relocation — narration + pipeline (read posture + presets from the game)

Type: task
Status: resolved
Blocked by: (none)

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

## Answer

Resolved 2026-09-07 — shipped; build fully green (1082 unit + 1485 integration nextest; full `python build.py` gate).

**Premise repair (the headline).** The ticket's premise — "the game row must carry the fields first" — had been broken by migration **v21** (commit d03c337, guided-generations pre-merge), which dropped `games.narrative_perspective` / `narrative_tense` as write-only. They were write-only only because *this* ticket hadn't landed its reads yet; guided-generations ticket 05's own cross-map note said "no removal — vindicated by narrator-modes 06". Ticket 01's resolved decision (posture is per-game, inherited from world; mode-switch nudges *that game's* perspective) and ticket 07's in-game-override scope both require the columns. So v21 is reversed as cleanup-overshoot: **migration v23** re-adds the columns and re-backfills from the world's posture (COALESCE + orphan warn, mirroring v19). No user data was ever lost — nothing wrote per-game posture between v19 and v23 other than inheritance itself.

**Threading decision.** `PromptContext` gains `narrative_perspective` / `narrative_tense` as required constructor params (after `world`) — posture is not optional, so no builder default; forgetting it is a compile error, which is the silent-wrong-voice failure mode guided-generations 05 grilled against. `PromptAssembler::assemble` remains the sole stamper, now from the context instead of the world. `PipelineRun::resolve_posture(&world)` is the game-first resolver (game this run started for → world fallback + warn-log, mirroring `resolve_active_preset_id`'s shape); narration generation and the trigger-continuation path both use it. Arrival passes its already-loaded `game` fields directly — the cross-map note's "world fallback in arrival" proved unnecessary: `require_game` hard-fails on a missing row and the columns are NOT NULL.

**Preset-id half (scope items 3–5) was already shipped** by ticket 05's storage helpers: `narration_generation.resolve_preset_choice` reads `active_system_preset_id` / `active_impersonate_preset_id`, the quantifier agent reads `active_quantifier_preset_id`, all game-first with registry-Novel fallback. Items 3–5 reduced to verification; no changes needed there.

**Tests.** `test_assemble_injects_narrative_voice_from_resolved_posture` (renamed) now proves the resolved posture wins over both the world's shipped values and conflicting caller template_vars. New `resolve_posture` game-first / world-fallback pair in `narration_generation_tests.rs`. Migration tests: v21-era terminal assertions rewritten for the composed chain (v19 add → v21 drop → v23 restore), plus `test_v23_backfills_game_posture_from_world`.

**Map maintenance.** Ticket 09's "this ticket ships v21" migration reference is stale (v21–v23 all shipped by other work); its body updated to take the next free version. Ticket 07 is now unblocked (06 and 14 both resolved).

## Comments

- 2026-08-30 — cross-map note from [guided-generations-architecture 05 — Single injection point for narrative-voice setting](../../guided-generations-architecture/issues/05-single-narrative-voice-injection.md):
  voice-application ownership is settled there — the prompt-construction
  module (`PromptAssembler::assemble`) is the sole owner of applying posture
  to `TemplateVars`; `TemplateVars::set_narrative_voice` is being deleted and
  replaced by a module-private helper in `assembler.rs`. When this ticket
  threads posture from the game, thread the **resolved posture**
  (perspective, tense) *to that owner*: callers resolve (game, with world
  fallback in arrival) and supply it; `arrival_service` itself never stamps.
  Scope items 1–2 then reduce to changing the *source* the owner stamps from
  (world → game), not adding a second stamper. If this ticket lands before
  05's pre-merge execution, apply the same single-owner shape here and 05's
  cleanup will consume it.
- 2026-08-30 — sequencing settled by [guided-generations-architecture 07 —
  Pre-merge handoff: landing shape and merge
  strategy](../../guided-generations-architecture/issues/07-pre-merge-handoff-shape.md):
  the guided-generations pre-merge execution (including that map's ticket 05
  voice-ownership cleanup) lands FIRST, as one commit, and
  `guided-generations` merges before this effort resumes. By the time this
  ticket runs, the single-owner shape is already in place —
  `TemplateVars::set_narrative_voice` deleted, a module-private helper in
  `assembler.rs` owning the stamp — so scope items 1–2 reduce exactly as
  the comment above describes: change the *source* the owner stamps from
  (world → game), never add a second stamper. Note also: the same pre-merge
  commit removes Narrator Action entirely (guided-generations-architecture
  ticket 06), so this map's destination point 3 and Out-of-scope references
  to "narrator-action" are stale — reconcile them when the effort resumes.
