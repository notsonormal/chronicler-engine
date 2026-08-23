# Grill: NarratorMode setting shape + perspective default interaction

Type: grilling
Status: pending
Blocked by: (none)

## Question

Decide the shape of the new `NarratorMode` setting and how it interacts with the existing `NarrativePerspective` setting (ticket 19). This is the mode-axis mechanics ticket; the preset content it selects is designed in ticket 02.

### Sub-questions

1. **The field.** A `NarratorMode` enum (`Novel` / `InteractiveFiction`) on `AppSettings` (`src/domain/model/settings.rs`), defaulting to `Novel` (current behavior). Confirm the variant names and default. Storage migration bumps past v18.

2. **Bundle selection.** Q1=A (Round 2) chose a per-mode preset *bundle* (system + impersonate + quantifier). Confirm the mechanism: does `NarratorMode` select a *bundle id* (one coherent set per mode), or does it independently retarget each `active_*_prompt_preset_id` (system/impersonate/quantifier) to a mode-specific default? The former is one knob; the latter is three knobs the mode nudges. Grill which keeps the bundle coherent (the original defect from ticket 16 was preset drift).

3. **Perspective default interaction.** Ticket 16 established mode and perspective are orthogonal; their defaults correlate (novel→third, IF→second). Does IF mode (a) force second person and lock perspective, (b) set second as a default the user can still override, or (c) leave perspective entirely independent (the user sets it, mode does not touch it)? Grill the coherence-vs-flexibility tradeoff. Note: the perspective macro `{{narrative_perspective}}` already exists and is injected centrally (`PromptAssembler::assemble`); the interaction is about the *default*, not the mechanism.

4. **Settings panel.** A mode dropdown in the narrative-voice settings panel (`src/adapters/driving/http/settings/handlers/`). Confirm placement alongside the perspective/tense dropdowns, and whether changing mode also nudges the perspective dropdown (if Q3 picks (b)).

## Notes for the session

- Read before grilling: `src/domain/model/settings.rs` (AppSettings — `NarrativePerspective`/`NarrativeTense` at `:28`/`:70` are the pattern; `active_*_prompt_preset_id` at `:211`/`:215`), `src/domain/model/utils/settings_defaults.rs` (default-fn pattern), `src/adapters/driving/http/settings/handlers/settings.rs` + `templates/settings.rs` (the narrative-voice panel), `data/prompt_presets/` (the three preset dirs).
- This ticket decides the *mechanics* only. The IF preset *content* (Agency Rule surgery) is ticket 02 — unblocked, can run in parallel. Do not design preset text here.
- The bundle-coherence concern is the original defect from the parent map's ticket 16: independent preset edits let the three presets drift out of coherence. The mode shape should not reintroduce that.
- Skills: `/grilling`, `/domain-modeling`.
