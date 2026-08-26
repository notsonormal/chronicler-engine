# Map: Narrator Modes & Options

Labels: wayfinder:map

## Destination

An implementation map. Deliver an additive IF/CYOA narrator mode alongside the existing novel mode, plus options-autogeneration as a first-class feature. Done when `python build.py` is green:

1. **IF/CYOA narrator mode** — a `NarratorMode` setting (Novel / InteractiveFiction) with a per-mode preset bundle (system + impersonate + quantifier). Novel mode is untouched (additive). IF mode lets the narrator elaborate the player's terse input — the posture the novel preset's Agency Rule blocks today.
2. **Options-autogeneration** — an independent feature available in BOTH modes (not mode-gated): the engine generates pickable options the player can choose from. The trigger model (on-demand button / always / both) is a design decision, not pre-decided.
3. **Steering features stay available** — guide, narrator-action, and impersonate are NOT mode-gated and NOT pre-adapted; existing presets serve both modes unless a design ticket discovers a need.

This map carries implementation into itself (Notes override): decide, then implement, then build-green.

## Notes

- **Domain — grounded against current code (2026-08-23):**
  - Settings: `src/domain/model/settings.rs` — `AppSettings` carries `NarrativePerspective` (Second/Third, default Third) and `NarrativeTense` (Past/Present, default Past) (ticket 19), plus `active_system_prompt_preset_id` / `active_impersonate_prompt_preset_id` (`:211`/`:215`). A new `NarratorMode` field sits alongside these; a storage migration bumps past v18.
  - Presets: `data/prompt_presets/{system,impersonate,quantifier}/default.json`. The system preset's `instructions` carry the **Agency Rule** ("Never write, assume, or infer the player's actions, thoughts, or feelings. The player's speech lines must be in indirect speech.") — the novel posture. IF mode needs a sibling system preset that allows elaboration. Impersonate and system presets use `{{narrative_perspective}}`/`{{narrative_tense}}` macros; quantifier examples are hardcoded third-person past (per ticket 16 — macro-driving teaching text was rejected as ungrammatical).
  - Template macros: `src/domain/model/template.rs` (`TemplateVars`), centralized injection in `src/application/prompting/assembler.rs` (`PromptAssembler::assemble`).
  - Settings panel: `src/adapters/driving/http/settings/handlers/` (`settings.rs`, `mod.rs`) + `templates/`.
  - Pipeline: `src/application/pipeline/pipeline_run.rs` (`phase_narrate`, `call_narrator`); `GenerationPhase { Narrating, Quantifying }` in `src/domain/model/state/generation_status.rs:31`. Options-autogeneration likely needs a new phase or pipeline branch.
  - Steering features (all in and build-green — tickets 05–13/17/19): guide = final `<Guide>` layer (`PromptLayer::Guide`); narrator-action = persisted `MessageType::Narrator` entry + continue; impersonate = `PresetType::Impersonate`, output `MessageType::Input`, drops `<PlayerCharacter>` layer. Reference: `docs/diataxis/reference/narrative/ai_steering.md`.
- **Skills every session should consult:** `/grilling`, `/domain-modeling`, `/prototype`, `/research`.
- **Standing preferences:**
  - This map carries implementation into itself — implementation tickets follow the design tickets; do not stop at decisions.
  - Re-verify every file path against the current tree before editing.
  - Do NOT mode-gate steering features (Round 2 Q3=C): no feature is disabled by mode. Adaptation is preset-tuning at most, and only if a design ticket discovers a need.
  - Options-autogeneration is available in BOTH modes (Round 2 Q2=B), not an IF-mode property.
  - Novel mode is untouched (Round 1 Q3=A): the IF preset is a sibling, not a rewrite of the novel preset.
- **Spun off from:** the steering-and-guided-generation map, ticket 18 (`.scratch/steering-and-guided-generation/issues/18-grill-two-narrator-modes.md`). Seed facts in that ticket's resolution comment.

## Decisions so far

<!-- the index — one line per closed ticket: enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [03 Research: option/choice-generation prior art](issues/03-research-option-generation-prior-art.md) — three systems generate pickable choices: ST-CYOA & ST-Roadway (separate LLM call, selection-as-input via impersonate; CYOA=on-demand, Roadway=on-demand+auto-on-message with a cheap-profile two-model pattern and domain-diversity prompt) and AI Dungeon Classic (same-call, later dropped); ChoiceScript/Twine authored; swipes are retry; open gaps: option-set retry (swipe-style), cross-session persistence, options×steering.

## Not yet specified

<!-- fog toward the destination — graduates as the frontier advances -->

- **Implementation: NarratorMode setting + storage migration + settings panel** — graduates from the mode-shape grilling (ticket 01). The field, the storage migration (bump past v18), the settings-panel dropdown, and wiring the preset-bundle selection by mode. Not specifiable until the shape settles.
- **Implementation: IF system preset seed + quantifier perspective** — graduates from the IF-preset design (ticket 02). Authoring the IF system preset, and resolving whether the quantifier preset needs an IF/second-person variant (its examples are hardcoded third-person past per ticket 16; macro-driving them was rejected as ungrammatical — an IF variant may need its own hardcoding or a different resolution). Not specifiable until the preset design settles.
- **Implementation: options-autogeneration pipeline + UI** — graduates from the options design (ticket 04): the generation mechanism (new phase? new agent?), the HTMX rendering of pickable options, the selection flow (does picking an option submit it as input?), and retry of an option set. Not specifiable until the design settles.
- **Options × steering interaction** — can a player use guide or narrator-action alongside options? Can options be generated for an impersonate? Likely mode-agnostic and falls out of the options design, but may surface sub-questions. Fog until options design settles.
- **Specs + integration tests** — a `docs/specs/` spec and `tests/http/` integration tests for IF mode and options, mirroring the steering spec/test pattern. Graduate after the designs settle.
- **Documentation** — CONTEXT.md terms (Narrator Mode, Options), a diataxis reference doc for IF mode + options, DOC anchors on new code. Last, after implementation.

## Out of scope

<!-- work ruled beyond the destination; closed, never graduates -->

- **Mode-gating steering features** — guide, narrator-action, and impersonate are available in both modes (Round 2 Q3=C). Disabling any by mode is ruled out.
- **Redesigning the novel mode** — novel is untouched (Round 1 Q3=A). The IF posture is a sibling preset, not a rewrite of the novel preset's Agency Rule.
- **NPC impersonation, guide+impersonate composition** — already out of scope on the parent steering map; remain out of scope here.
