# Impersonate preset and context-layer filtering

Type: task
Status: pending

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
