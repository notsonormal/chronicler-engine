# Grill: NarratorMode setting shape + perspective default interaction

Type: grilling
Status: resolved
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

## Answer

**Grilling complete: 9 questions across 4 rounds. The frontier is empty.**

### The decision, in one line

Posture (narrator mode + perspective + tense) is **per-game, inherited from the world**; presets are **per-game, inherited from a global per-mode registry**. This reversed the ticket's original global-setting framing (Q5).

### Settled decisions

1. **The field (Q1).** A `NarratorMode` enum with variants `Novel` / `InteractiveFiction`, default `Novel`. `#[serde(rename_all = "snake_case")]` → `"novel"` / `"interactive_fiction"`. `as_str()` + `parse_or_default()` (→`Novel` on bad data) + `FromStr`, mirroring `NarrativePerspective` (`src/domain/model/settings.rs:28`). A `default_narrator_mode()` fn-pointer joins `settings_defaults.rs`. snake_case (not lowercase) because `InteractiveFiction` is two words; the existing enums use lowercase only because their variants are single words.

2. **Posture locus (Q5 + Q6) — the reversal.** The posture triple (`narrator_mode`, `narrative_perspective`, `narrative_tense`) moves OFF global `AppSettings` to:
   - **World** (default, on `WorldCard`/`WorldManifest`): the author's shipped posture.
   - **Game** (override, on the `Game` row — NOT `GameState`, which is mutable narrative state): inherited from the world at creation, overridable per-game.
   - `AppSettings` LOSES `narrative_perspective` and `narrative_tense` (added in migration v18) and never gains `narrator_mode`.
   - The narration path reads posture from the game: `assembler.rs:85-87` and `arrival_service.rs:149` (today `set_narrative_voice(settings.X, settings.X)`) change to read the game's posture.
   - Domain instinct: the narrative posture is a property of the world the author shipped, not a global engine preference.

3. **Bundle-selection mechanism (Q2-B, relocated to per-game by Q5).** Switching a game's mode retargets that game's three preset-ids to the mode's default bundle (looked up from the global per-mode registry, decisions 7–8). The three preset-ids stay independently editable per-game afterward. Coherent triple at switch time; per-game customization preserved. No mode-gating of preset choice.

4. **Perspective nudge (Q3-b + Q5-b).** Switching a game's mode to IF nudges that game's `narrative_perspective` to `Second`; switching to Novel nudges to `Third`. The perspective dropdown stays enabled (no mode-gating). The nudge is **conditional**: it fires only if the current value is the OTHER mode's default (Third→Second on IF switch only if currently Third; Second→Third on Novel switch only if currently Second). A deliberately-set perspective is never clobbered. The `{{narrative_perspective}}` macro already flows whatever the game holds; no mechanism change.

5. **Mode-switch trigger + handler (Q4).** Auto-save-on-change (HTMX `hx-trigger="change"` — standard modern pattern). The mode switch is a **distinct action** (not folded into a generic voice-save): it sets the game's mode, retargets the three preset-ids, nudges perspective per decision 4, and re-renders the override UI. Mirrors the existing `set_narrator_handler` "explicit retarget + full re-render" pattern. The endpoint lives on the in-game override surface (implementation ticket 07), NOT on global `/settings`.

6. **Preset-id locus (Q7).** The three `active_*_prompt_preset_id` (system/quantifier/impersonate) are **per-game selections** into the global preset library. Presets live on global + game, **NOT world** — the world ships a posture default, not a rule-set selection. The existing global `/prompt-presets/:id/activate` UI and `PromptPresetService` (library CRUD) stay; the per-game selection is new.

7. **New-game preset inheritance (Q8, refined by Q9).** A new game inherits the **mode-matched default bundle** for its world mode from the global per-mode registry. The global `AppSettings` ids are the **mode-aware reset target** ("reset my game to my mode's seeded defaults"), NOT the verbatim creation source. Coherent games out of the box.

8. **Global per-mode registry structure (Q9-b).** Global `AppSettings` holds **TWO full default bundles** (2×3 preset-ids), keyed by mode: Novel → (`system_default`, `quantifier_default`, `impersonate_default`); InteractiveFiction → (`system_if_default`, `quantifier_*`, `impersonate_*`). **Symmetric** — the IF bundle's quantifier/impersonate ids initially point to the SAME presets as the Novel bundle. This defers the quantifier/impersonate content decision to ticket 02 without a schema change. Cost: two redundant ids if quantifier/impersonate stay identical; payoff: the open question stays open.

### Carry-forwards / corrections from the grilling

- **Q5 reversal.** The original ticket assumed a global `NarratorMode` on `AppSettings` + a settings-panel dropdown. The grilling reversed this to world→game. Implementation tickets 05–07 reflect the relocation, not the original global-setting framing.
- **Preset model correction (mid-grilling re-review).** Presets are authorable behavioral rule-sets (five fields each: `role`/`instructions`/`writing_style`/`output_format` + injected `global_rules`), NOT posture. Posture flows INTO presets via `{{narrative_perspective}}`/`{{narrative_tense}}` macros (injected centrally in `PromptAssembler::assemble`); preset text is posture-agnostic by design. Mode ↔ preset-bundle is a **coherence** relationship (the system preset's Agency Rule must agree with the mode's elaboration posture), not identity. The three preset-ids are independent selectors into a global library. Confirmed by re-reading `data/prompt_presets/*/default.json`, `src/domain/model/prompt_preset.rs`, `src/application/prompting/assembler.rs`, `src/application/agents/quantifier/agent.rs`, `src/application/pipeline/pipeline_run.rs`.

### Deferred to ticket 02 (content, not mechanics)

- The IF system preset content (Agency Rule surgery — the core work of ticket 02).
- Whether the quantifier preset needs a second-person-IF examples variant (its examples are hardcoded third-person past; macro-driving was rejected as ungrammatical per parent ticket 16). The symmetric registry (decision 8) holds either answer.
- Whether the impersonate preset needs a terse IF variant (rules are mode-independent; a terse variant is optional authoring, not a rule requirement).

### Per-preset mode-dependence findings (from the mid-grilling re-review)

| Preset | Rules change by mode? | Verdict |
|---|---|---|
| System | Yes — mode-defining (Agency Rule) | Two versions required (ticket 02) |
| Impersonate | No — rules are mode-independent; macros drive posture | One version; terse IF variant is optional authoring |
| Quantifier | No for mode; yes for perspective (separate axis) — examples hardcoded third-person past | Rules don't need two versions; examples have a perspective problem, deferred to ticket 02 |

### Implementation tickets graduated from this resolution

- **05** — Posture relocation, data layer (domain model + storage migration v19 + per-mode preset registry).
- **06** — Posture relocation, narration + pipeline (read posture + presets from the game). Blocked by 05.
- **07** — Posture relocation, UI (world-editor dropdowns + in-game override + per-game mode-switch action). Blocked by 06.
