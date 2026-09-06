# Implement: narrative perspective and tense settings

Type: task
Status: closed
Blocked by: (none)
Assignee: pi (wayfinder session 2026-08-23)

## Question

Implement the `NarrativePerspective` and `NarrativeTense` settings decided in ticket 16, plus the macro plumbing and three-preset rewrite. Fully specified by ticket 16's answer.

## Resolution

Implemented and build-green (`python build.py` and `python build.py --llm-only` both pass).

### What changed

- Added `NarrativePerspective { Second, Third }` and `NarrativeTense { Past, Present }` enums in `src/domain/model/settings.rs` with `as_str()` and `FromStr` conversions.
- Added `narrative_perspective`/`narrative_tense` fields to `AppSettings` (default `Third`/`Past`) via `settings_defaults` serde default fns.
- Extended `TemplateVars` with `narrative_perspective`/`narrative_tense` macro strings and updated `render_template` to substitute `{{narrative_perspective}}` / `{{narrative_tense}}`.
- Centralized prompt injection in `PromptAssembler::assemble`: it clones `context.template_vars` and overwrites the two voice fields from `self.settings`. This covers all four pipeline/retry sites without touching `pipeline_run.rs` or `retry.rs`.
- Updated `arrival_service.rs` to read `Storage::get_settings()` once and set the two voice fields on the context before `build_narration_prompt` (arrival's local assembler is settings-less).
- Added storage migration v18 (`settings.narrative_perspective`, `settings.narrative_tense` TEXT columns) and wired `DbSettings`/`settings.rs` round-trip.
- Rewrote `data/prompt_presets/system/default.json` and `data/prompt_presets/impersonate/default.json` to use the macros.
- Rewrote `data/prompt_presets/quantifier/default.json` examples to third-person past, **without macros** — macro-driving the teaching examples would produce ungrammatical `"third walks..."`. This is the deliberate deviation from ticket 16's blanket "all three presets"; the quantifier keys off location, not pronoun, so the examples match the default narrator voice by hardcoding.
- Added a dedicated `/settings/narrative-voice` form with two dropdowns (Perspective, Tense), mirroring the text-check pattern.
- Added unit tests (`settings_tests.rs`, extended `template_tests.rs`, central-injection tests in `assembler_tests.rs`) and HTTP E2E tests (`tests/http/settings.rs`). Updated `docs/specs/settings.md` with scenarios 20.8–20.10 and regenerated `docs/diataxis/reference/frontend/http_routes.md` (now 53 routes).

### Key design call

Voice injection is centralized in `PromptAssembler::assemble` rather than threaded through each `PromptContext::new` call site. The assembler already owns `settings` and reads it for budget resolution; extending it to patch `template_vars` keeps the injection in one place and avoids edits to `pipeline_run.rs`/`retry.rs`.

### Deferred

Diátaxis reference doc updates (`prompt_system.md` macro list, `storage.md` migration v18) were deferred to the `chronicler-after-plan-workflow` skill per the plan.

## Specification (from ticket 16)

### 1. Two new `AppSettings` enum fields

Add to `src/domain/model/settings.rs` `AppSettings`:

- `narrative_perspective: NarrativePerspective` — default `Third`.
- `narrative_tense: NarrativeTense` — default `Past`.

New enums in the same module (or a `narrative` submodule if cleaner):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NarrativePerspective { Second, Third }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NarrativeTense { Past, Present }
```

Use `#[serde(default = "...")]` with default-fn pointers in `src/domain/model/utils/settings_defaults.rs` (the existing pattern for serde defaults — `default_active_system_prompt_preset_id` etc.), since `#[serde(default)]` on a unit-variant enum defaults to the *first* variant (would make `Second`/`Past` the default, wrong). Default fns return `Third`/`Past`.

`AppSettings::default()` updated to use the default fns. Add `#[serde(default = "settings_defaults::default_narrative_perspective")]` / `default_narrative_tense` to the struct fields.

### 2. Two new macros on `TemplateVars`

Extend `src/domain/model/template.rs` `TemplateVars` with:

- `narrative_perspective: String` — substituted as `{{narrative_perspective}}`, value `"second"` or `"third"`.
- `narrative_tense: String` — substituted as `{{narrative_tense}}`, value `"past"` or `"present"`.

Update `render_template` (in `src/domain/model/utils/template.rs`) to substitute both new macros alongside the existing `{{user}}`/`{{persona_*}}`.

Update `TemplateVars::new` (empty defaults) and `TemplateVars::from_persona` (or add a new constructor / extend the call site) to populate the two new fields from `AppSettings`. The call site that builds `TemplateVars` from `PromptContext` (`src/application/prompting/...`) must read `narrative_perspective`/`narrative_tense` off `AppSettings` and inject the string form. Check how `TemplateVars` currently reaches the prompt builder and thread the settings through the same path.

### 3. Three preset JSONs rewritten to use the macros

All three presets get the macros so the setting *enforces* coherence (the original defect: three presets drifting to three perspectives with nothing checking agreement).

**`data/prompt_presets/system/default.json`** — replace the `writing_style` value:
- From: `"Third-person limited perspective, focused on the player character.\nPast tense narrative prose."`
- To: `"{{narrative_perspective}}-person limited perspective, focused on the player character.\n{{narrative_tense}} tense narrative prose."`

**`data/prompt_presets/impersonate/default.json`** — two edits:
- `writing_style`: from `"First-person perspective as {{user}}. Match the tense already in use. Keep speech and action grounded in the immediate physical scene."` to `"{{narrative_perspective}}-person perspective as {{user}}. {{narrative_tense}} tense. Keep speech and action grounded in the immediate physical scene."`
- `instructions`: from `"First person (\"I\") for speech and internal thought. Describe {{user}}'s own physical actions in the tense the conversation already uses."` to `"{{narrative_perspective}} person for speech and internal thought. Describe {{user}}'s own physical actions in {{narrative_tense}} tense."` (Keep the rest of the instructions — the "write only from {{user}}'s perspective" posture is unchanged. The impersonate preset *follows* the narrator's perspective/tense now, rather than hardcoding first/past-match.)

**`data/prompt_presets/quantifier/default.json`** — rewrite the `examples` to third person (the Q3 fix), macro-driven:
- From second-person: `"You walk through the door into the kitchen."` / `"You examine the ancient vase carefully."`
- To macro-driven, e.g.: `"{{narrative_perspective}} walks through the door into the kitchen."` — but check this renders sensibly. The examples are *teaching* text for the LLM; if `"second walks..."` / `"third walks..."` reads as nonsense, hardcode third-person narration in the examples (`"She walked through the door into the kitchen."`) and leave perspective out of the quantifier preset. Decision: the quantifier's movement logic keys off *location*, not pronoun (verified, ticket 16), so the examples only need to be consistent with the narrator's voice, not macro-driven. **Hardcode the examples to third person** (`"She walked through the door into the kitchen."`, `"He examined the ancient vase carefully."`), matching the system default. Do NOT add macros to the quantifier preset — it would produce ungrammatical teaching text. The quantifier follows the narrator's voice by example, not by setting.

(Note: this means the quantifier preset does NOT get the macros, contradicting the blanket "all three presets" in ticket 16 decision 6. The macro's purpose — enforce coherence — is served by the examples matching the system default's third-person; macro-driving the quantifier examples would produce `"{{narrative_perspective}} walks..."` → `"third walks..."` which is ungrammatical and a regression. The system preset's macro is the source of truth; the quantifier examples are teaching text that matches the default voice. This is the correct call — flag it in the PR/answer.)

### 4. Settings-panel dropdowns

Add two dropdowns to the HTTP settings UI (wherever the existing `active_*_prompt_preset_id` dropdowns live — check `src/adapters/driving/http/` templates and `view_models.rs`):
- Narrative perspective: Second / Third.
- Narrative tense: Past / Present.

Wire POST handlers to persist the two new fields. Follow the existing settings-edit handler pattern.

### 5. Storage migration

New storage migration (increment the version — check the current version, ticket 06 added v15 and ticket 09 added v16, so this is v17). The two new `AppSettings` fields are stored in the settings JSON blob (settings storage already serializes `AppSettings`); confirm whether the settings blob is stored as a single JSON column (no schema change needed, just a serde default for old rows) or requires a column add. Check `src/adapters/driven/storage/settings.rs`. If the settings blob is a single JSON column with `#[serde(default)]` on the new fields, **no schema migration is needed** — old settings JSON deserializes with the serde defaults (`Third`/`Past`). Verify this; only add a migration version bump if the settings storage is column-based.

### 6. Unit tests

- `settings.rs`: round-trip serialize/deserialize `AppSettings` with new fields; default is `Third`/`Past`; old settings JSON (without the fields) deserializes to defaults.
- `template.rs` / `render_template`: `{{narrative_perspective}}` and `{{narrative_tense}}` substitute correctly; presets with the macros render to the expected strings for Second/Past, Third/Past, Second/Present, Third/Present.
- At least one test per preset confirming the macro-substituted `writing_style` reads as coherent English for each of the four value combinations.

### 7. Out of scope for this ticket

- First-person narration or first-person impersonate (excluded by ticket 16 — unavailable to narrator, unwanted for impersonate).
- Auto/detect mode (rejected by ticket 16 — reintroduces the vague "match" instruction).
- Two-narrator-modes (ticket 18 — orthogonal axis).
- `GenerationReplay`→`SteeringRecord` rename (deferred, map Not-yet-specified).

## Validation

`python build.py` must be green. No LLM prompt/parsing behavior change that requires `--llm-only` is expected (macros are string substitution, not prompt-logic changes), but if the build's LLM tests are touched by the preset rewrite, run `python build.py --llm-only` per the AGENTS.md LLM test policy.

## Notes

- Re-verify every file path against the current tree before editing (map standing preference; the old plan's paths are stale).
- The quantifier-preset macro decision (hardcode third, don't macro-drive) is a deviation from ticket 16's blanket "all three presets" — flag it in the resolution, do not silently diverge.
- Read `src/adapters/driven/storage/settings.rs` and the settings storage tests before deciding whether a schema migration is needed.

## Resolution

Implemented and build-green.

Validation results:
- `python build.py` green (all 12 steps passed; 1444 tests passed, 2 LLM tests skipped).
- `python build.py --llm-only` green (2/2 LLM tests passed).

## What changed

- `src/domain/model/settings.rs` — added `NarrativePerspective` (Second/Third) and `NarrativeTense` (Past/Present) enums with `as_str()`/`FromStr`; added the two fields to `AppSettings` with serde defaults; updated `AppSettings::default()`.
- `src/domain/model/utils/settings_defaults.rs` — added default-fn pointers returning Third/Past.
- `src/domain/model/template.rs` + `src/domain/model/utils/template.rs` — `TemplateVars` gained `narrative_perspective`/`narrative_tense` (default `"third"`/`"past"`); `render_template` substitutes the two macros.
- `src/application/prompting/assembler.rs` — `PromptAssembler::assemble` now injects the voice centrally from `self.settings`, covering all four pipeline/retry sites with one change.
- `src/application/arrival_service.rs` — arrival reads `Storage::get_settings()` once and sets the two voice fields on the context (arrival's local assembler has no settings handle).
- `src/adapters/driven/storage/utils/plumbing.rs` — migration v18 adds the two settings columns.
- `src/adapters/driven/storage/models/settings.rs` + `src/adapters/driven/storage/settings.rs` — `DbSettings` maps the two columns; load parses via `FromStr` with a fallback + warning on unknown values.
- `data/prompt_presets/system/default.json` — `writing_style` uses `{{narrative_perspective}}`/`{{narrative_tense}}`.
- `data/prompt_presets/impersonate/default.json` — `writing_style` and `instructions` use the macros; impersonate now follows the narrator's configured voice instead of hardcoding first person.
- `data/prompt_presets/quantifier/default.json` — examples rewritten to third-person past, NO macros (macro-driving would produce ungrammatical `"third walks..."` teaching text).
- `src/adapters/driving/http/settings/templates/settings.rs` + `handlers/settings.rs` + `builders/router.rs` — added a dedicated `/settings/narrative-voice` form with Perspective and Tense dropdowns.
- Tests added/extended: `src/domain/model/settings_tests.rs`; `src/domain/model/template_tests.rs` (macro + real-preset-coherence tests); `src/application/prompting/assembler_tests.rs` (central-injection + settings-less default); `tests/http/settings.rs` (panel + POST + fallback + failure); `src/adapters/driven/storage/settings_tests.rs` (round-trip).
- `docs/specs/settings.md` — added scenarios 20.8–20.10 for the new endpoint.
- `docs/diataxis/reference/frontend/http_routes.md` + `scripts/tests/test_extract_http_routes.py` — regenerated/updated for the new route (53 routes).

## Design deviations from the original ticket

- **Voice injection point:** Centralized in `PromptAssembler::assemble` rather than threading a builder through every `PromptContext::new` call site. Decision made during plan review; avoids duplicated settings reads and keeps the change localized.
- **Storage:** Verified column-based settings storage, so v18 migration was required.
- **Existing DBs:** Preset JSON rewrites only affect fresh DBs; existing DBs keep old preset text until the user edits/re-seeds (user-confirmed "seed files only" option).
