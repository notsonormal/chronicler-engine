# Impersonate preset and context-layer filtering

Type: task
Status: resolved
Assignee: wayfinder-session

## Question

Add `PresetType::Impersonate` and the impersonate prompt structure that replaces the narrator voice apparatus.

Per the design synthesis (`../research/04-design-synthesis.md`, Q7 + Q7b + Q6 + Q8):

1. Add `PresetType::Impersonate` to `src/domain/model/prompt_preset.rs` (today: `System`/`Quantifier`). When `impersonate=true`, the assembler selects the impersonate preset instead of the system preset (Marinara `impersonatePromptPresetId`, Q7=A).
2. The impersonate preset's voice apparatus replaces the narrator voice. Verified: the default system preset is entirely narrator-framed and contradicts impersonate — Role: *"You are an interactive fiction author… acting as the narrator… except the protagonist, who is played by the user"*; WritingStyle: *"Third-person limited… focused on the player character"*; OutputFormat: *"narrate what happens now"* (`data/prompt_presets/system/default.json`). These must be **replaced**, not toggled. Marinara suppresses all non-marker sections (`assembler.ts:367,505`).
3. Context-layer filtering: drop the voice apparatus (Role/Instructions/WritingStyle/OutputFormat) and the `<PlayerCharacter>` reference-card layer; keep the context layers (`<GameState>`, `<KnownNpcs>`/`<NpcsInRoom>`, `<WorldLore>`, `<ConversationHistory>`).
4. Persona injection: drop `<PlayerCharacter>`; inject persona data into the impersonate instruction via `{{user}}`/`{{persona_description}}` macros (Marinara `DEFAULT_IMPERSONATE_PROMPT`, Q7b=A). Single coherent voice instruction.
5. Output: generated text saved as a player-voiced `MessageEntry` (Q6=A). The replay blob (ticket 06) holds impersonate=true + direction + preset for retry.
6. Mutual exclusivity with guide (Q8=A): impersonate and guide cannot fire on the same turn.

Blocked by: 06 (replay blob), 07 (slash parser for `/impersonate` entry).

## Resolution

Implemented `PresetType::Impersonate` and the impersonate prompt structure, wired through the prompt assembler and pipeline. `python build.py` green.

**Preset type + settings + seed data.** Added `PresetType::Impersonate` (as_str `"impersonate"`, `TryFrom`, serde) to `src/domain/model/prompt_preset.rs`. Added `active_impersonate_prompt_preset_id` to `AppSettings` (`src/domain/model/settings.rs`) with default fn `default_active_impersonate_prompt_preset_id` → `"impersonate_default"`. Storage migration v16 adds the `settings.active_impersonate_prompt_preset_id` column (`src/adapters/driven/storage/utils/plumbing.rs`); the settings DB model + storage round-trip the field (`models/settings.rs`, `settings.rs`). New seed preset `data/prompt_presets/impersonate/default.json` — voice apparatus that replaces the narrator voice, using `{{user}}`/`{{persona_description}}`/`{{persona_personality}}`/`{{persona_background}}` macros. `ensure_presets` now seeds the impersonate directory (`bootstrap/run.rs`).

**Persona macros.** Extended `TemplateVars` (`src/domain/model/template.rs`) with `persona_description`/`persona_personality`/`persona_background` fields + `from_persona(PersonaCard)` constructor; `TemplateVars::new` defaults them empty (so existing `{{user}}`-only call sites are unaffected). `render_template` substitutes all four macros. `PromptContext::new` now builds vars via `from_persona`, so the impersonate preset's macros resolve.

**Context-layer filtering + prompt assembler.** Added `impersonate: bool` to `PromptContext` + `with_impersonate` builder (mutually exclusive with `with_guide` — impersonate clears the guide). Threaded into `LayerRenderer`; when impersonate, `render_persona_layer` returns empty (drops `<PlayerCharacter>`), keeping `<GameState>`/`<KnownNpcs>`/`<NpcsInRoom>`/`<WorldLore>`/`<ConversationHistory>`. The voice apparatus (Role/Instructions/WritingStyle/OutputFormat) comes from the impersonate preset passed by the pipeline — no `PresetField` filtering needed since the preset itself is narrator-free.

**Pipeline wiring.** `PipelineInputs` gained `impersonate`/`impersonate_direction`/`impersonate_preset_id` (`pipeline_run.rs`). `run_from_input` derives them from `pending_replay` (impersonate suppresses guide — mutual exclusivity enforced at the input-derivation point). `phase_narrate` resolves impersonate (new `resolve_impersonate`: inputs first, else `retry_target.replay()` when `r.impersonate`), loads the impersonate preset via `load_impersonate_preset_and_response_length` (preset id from the blob, fallback `active_impersonate_prompt_preset_id`), sets `context.with_impersonate(true)`, feeds the direction as `user_message`, and saves the output as `MessageType::Dialogue` with `sender = persona.sheet.name` (player-voiced).

**Seam method.** `impersonate()` (`action.rs`) reads `active_impersonate_prompt_preset_id` from settings, builds a `GenerationReplay { impersonate: true, impersonate_direction, impersonate_preset_id }`, and runs the continue path (empty input). Generalized the guide seam into `process_action_with_replay`/`execute_action_with_replay` (take `Option<GenerationReplay>`); `process_action_with_guide`/`guide_narration` are now thin wrappers that build a guide-only blob. Retry re-applies impersonation from the swipe replay blob via `resolve_impersonate`.

**HTTP UI.** Extended the preset-panel template + handlers (`prompt_presets/templates/prompt_presets.rs`, `handlers/prompt_presets.rs`) for impersonate presets: list, add-form, activate, edit/delete, active-badge. The 4 `match preset.preset_type` arms now cover `Impersonate`.

**Tests.** PresetType impersonate as_str/TryFrom/serde round-trip; `TemplateVars::from_persona` macro substitution; assembler drops `<PlayerCharacter>` and omits `<Guide>` under impersonate while keeping context layers; assembler injects persona macros into the impersonate preset's system prompt; `push_message` stages the impersonate replay blob on a player-voiced `Dialogue` swipe; settings round-trip asserts the new field. Test seed helper `seed_default_impersonate_preset` added to `test_support/context.rs`.

**Review decisions (post code review).**
- **Cache architecture: Option A (accepted).** The impersonate preset swaps the storyteller preset on the storyteller connection. No separate impersonate connection. The swap breaks the storyteller prefix cache at token 0 on impersonate Messages. This cost is accepted because provider prefix caching is not reliable enough to design around in this framework. A separate impersonate connection (Option C, mirroring the Quantifier pattern) is the principled fix if impersonate becomes frequent; deferred. Moving the storyteller preset after the Message history was rejected: static text after dynamic text never caches, so it would break the storyteller cache on every normal Message.
- **Mutual exclusivity.** The review flagged `with_guide` not clearing `impersonate` as a one-way footgun. Verified false: the Action Pipeline builds the prompt in one `if`/`else` branch per Message, so only one of guide/impersonate is ever set. No change needed. The `self.guide = None` line in `with_impersonate` is redundant defensive code.
- **Extra persona macros kept.** `{{persona_personality}}` and `{{persona_background}}` are beyond Marinara's default template (which uses `{{user}}`/`{{persona_description}}` only) but are kept as a defensible extension to enrich the impersonation voice.

**Open deferrals.** Narrator generate-then-add path (ticket 10) and the auto-suggestion UI (ticket 12) are separate tickets; this one owns impersonate only. Integration-test/spec authoring (ticket 14) remains open. The impersonate default preset text is a first draft — it should be reviewed against real LLM output (an LLM-test concern, ticket 14's scope).
