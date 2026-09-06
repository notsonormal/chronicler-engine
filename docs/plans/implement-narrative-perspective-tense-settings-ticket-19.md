# Implement Narrative Perspective & Tense Settings (ticket 19)

## Summary

Add two `AppSettings` enum fields — `NarrativePerspective` (Second/Third, default Third) and `NarrativeTense` (Past/Present, default Past) — plus `{{narrative_perspective}}`/`{{narrative_tense}}` macros that the system and impersonate prompt presets consume, so the setting *enforces* perspective/tense coherence across presets. Inject the voice **centrally in `PromptAssembler::assemble`** (which already holds settings) — covering all four pipeline/retry sites for free; arrival reads `Storage::get_settings()` once. Add storage migration v18 for the two new columns, expose two dropdowns via a dedicated `/settings/narrative-voice` form, and cover with unit + HTTP tests. `python build.py` green.

Self-contained, dependency-free implementation ticket (unblocked; graduated from ticket 16's grilling).

## Key Changes

- **`src/domain/model/settings.rs`** — `NarrativePerspective`/`NarrativeTense` enums (`as_str`, `FromStr`); two new `AppSettings` fields with serde default-fn pointers; `Default` updated.
- **`src/domain/model/utils/settings_defaults.rs`** — `default_narrative_perspective`/`default_narrative_tense` fns (required: `#[serde(default)]` on a unit enum picks the first variant).
- **`src/domain/model/template.rs`** + **`src/domain/model/utils/template.rs`** — `TemplateVars` gains `narrative_perspective`/`narrative_tense` (string form, default `"third"`/`"past"`); `render_template` substitutes both.
- **`src/application/prompting/assembler.rs`** — `PromptAssembler::assemble` clones `context.template_vars`, overwrites the two voice fields from `self.settings` when present, and passes the patched vars down. This is the single injection point for all four pipeline/retry sites.
- **`src/application/arrival_service.rs`** — arrival's assembler is settings-less, so `run()` reads `Storage::get_settings()` once and sets the two `template_vars` fields on the context before `build_narration_prompt`. No `ArrivalTaskContext` constructor change.
- **`src/adapters/driven/storage/utils/plumbing.rs`** — migration v18 adds two `settings` columns.
- **`src/adapters/driven/storage/models/settings.rs`** + **`src/adapters/driven/storage/settings.rs`** — `DbSettings` gains 2 fields; SELECT/INSERT/`to_settings` map the new columns (string ↔ enum via `as_str`/`FromStr`).
- **`data/prompt_presets/system/default.json`**, **`data/prompt_presets/impersonate/default.json`** — `writing_style`/`instructions` rewritten to use the macros.
- **`data/prompt_presets/quantifier/default.json`** — examples rewritten to third-person past (NO macros; hardcoded to match the system default).
- **`src/adapters/driving/http/settings/templates/settings.rs`** + **`settings/handlers/settings.rs`** + **`builders/router.rs`** — two dropdowns, `NarrativeVoiceForm`, `save_narrative_voice_handler`, route `/settings/narrative-voice`.
- Tests: new `src/domain/model/settings_tests.rs`; extend `template_tests.rs`; extend `tests/http/settings.rs`; add a central-injection test in `assembler_tests.rs`.
- Docs (`prompt_system.md` macro list, `storage.md` migration v18) deferred to the `chronicler-after-plan-workflow` skill after implementation.

## Implementation

### Phase 1: Domain model (3 SP)

- [ ] #### Task 1.1: Add the two enums + conversions (1 SP)
  - [ ] ##### SubTask 1.1.1: Define `NarrativePerspective { Second, Third }` and `NarrativeTense { Past, Present }` in `src/domain/model/settings.rs` — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`, `#[serde(rename_all = "lowercase")]` (wire format `second`/`third`/`past`/`present`). Add `///` doc on each enum and variant (guardrail: every variant needs a doc or `/// [TRIVIAL_ENUM]`).
  - [ ] ##### SubTask 1.1.2: Add `as_str(&self) -> &'static str` and `impl std::str::FromStr` (returns `Result`) on both enums — used by the DB column mapping and the HTTP form parse.

- [ ] #### Task 1.2: Add `AppSettings` fields + defaults (1 SP)
  - [ ] ##### SubTask 1.2.1: Add `narrative_perspective: NarrativePerspective` and `narrative_tense: NarrativeTense` to `AppSettings` with `#[serde(default = "settings_defaults::default_narrative_perspective")]` / `default_narrative_tense`.
  - [ ] ##### SubTask 1.2.2: Add `default_narrative_perspective` (returns `Third`) and `default_narrative_tense` (returns `Past`) to `src/domain/model/utils/settings_defaults.rs`. Update `AppSettings::default()` to use them.

- [ ] #### Task 1.3: Extend `TemplateVars` + `render_template` (1 SP)
  - [ ] ##### SubTask 1.3.1: Add `narrative_perspective: String` and `narrative_tense: String` to `TemplateVars`; default `"third"`/`"past"` in both `new` and `from_persona` (so non-assembler call sites — quantifier, game_state, message_service — render sensibly and no-op for non-macro text).
  - [ ] ##### SubTask 1.3.2: Add `.replace("{{narrative_perspective}}", &vars.narrative_perspective).replace("{{narrative_tense}}", &vars.narrative_tense)` to `render_template` in `src/domain/model/utils/template.rs`.

### Phase 2: Storage (3 SP)

- [ ] #### Task 2.1: Migration v18 + DbSettings + settings.rs (3 SP)
  - [ ] ##### SubTask 2.1.1: In `src/adapters/driven/storage/utils/plumbing.rs` add a `version < 18` block after the v17 sender-drop block: `ALTER TABLE settings ADD COLUMN narrative_perspective TEXT NOT NULL DEFAULT 'third'` and `... narrative_tense TEXT NOT NULL DEFAULT 'past'`, each guarded by `column_exists`. `pragma_update(user_version, 18)`. (Settings storage is column-based — verified: `save_settings` INSERTs explicit columns, `DbSettings::from_row` reads by index — so a schema migration is required, contrary to ticket 19's hopeful note.)
  - [ ] ##### SubTask 2.1.2: Extend `DbSettings` (`src/adapters/driven/storage/models/settings.rs`) with `narrative_perspective: String` + `narrative_tense: String`. Update `from_row` to read indices 12 and 13 (appended last, after `active_impersonate_prompt_preset_id` at index 11). Update `to_settings` to parse via `NarrativePerspective::from_str` / `NarrativeTense::from_str`, falling back to the default with `tracing::warn!` on an unknown value (resilient load of corrupt rows).
  - [ ] ##### SubTask 2.1.3: Update `src/adapters/driven/storage/settings.rs` — add the two columns to the `get_settings` SELECT (positions 12, 13) and to the `save_settings` INSERT column list + `rusqlite::params!` (pass `settings.narrative_perspective.as_str()` / `narrative_tense.as_str()`). In-memory backend rides along by clone.

### Phase 3: Prompt injection (2 SP)

- [ ] #### Task 3.1: Central injection in `PromptAssembler::assemble` (1 SP)
  - [ ] ##### SubTask 3.1.1: In `src/application/prompting/assembler.rs` `assemble`: `let mut template_vars = context.template_vars.clone();` then, `if let Some(settings) = &self.settings { let g = settings.read().unwrap_or_else(|e| e.into_inner()); template_vars.narrative_perspective = g.narrative_perspective.as_str().to_string(); template_vars.narrative_tense = g.narrative_tense.as_str().to_string(); }`. Pass `&template_vars` to `build_system_prompt`, `build_post_history_prompt`, and the `LayerRenderer` (replacing `&context.template_vars`). Settings are the source of truth, so the injected value takes precedence over the context's default — correct. This single change covers `phase_narrate`, trigger-continuation, and both retry sites, all of which call `self.pipeline.prompt_assembler.assemble(..)` (verified). No edits to `pipeline_run.rs` or `retry.rs` production code.

- [ ] #### Task 3.2: Arrival reads settings + sets the context (1 SP)
  - [ ] ##### SubTask 3.2.1: In `src/application/arrival_service.rs` `run()`, after loading state and before `PromptContext::new`: `let voice = self.storage.get_settings()?;` (propagates failure via `?`). Make the `prompt_context` binding `mut`, then set `prompt_context.template_vars.narrative_perspective = voice.narrative_perspective.as_str().to_string();` and likewise for tense. Arrival's assembler is settings-less (`build_narration_prompt` constructs `PromptAssembler::new` without `.with_settings`), so central injection does not fire here — the context-set values stand. No `ArrivalTaskContext` constructor change; no `with_narrative_voice` builder needed.

### Phase 4: Preset rewrites (2 SP)

- [ ] #### Task 4.1: System + impersonate presets use the macros (1 SP)
  - [ ] ##### SubTask 4.1.1: `data/prompt_presets/system/default.json` `writing_style`: `"Third-person limited perspective, focused on the player character.\nPast tense narrative prose."` → `"{{narrative_perspective}}-person limited perspective, focused on the player character.\n{{narrative_tense}} tense narrative prose."`
  - [ ] ##### SubTask 4.1.2: `data/prompt_presets/impersonate/default.json` `writing_style` → `"{{narrative_perspective}}-person perspective as {{user}}. {{narrative_tense}} tense. Keep speech and action grounded in the immediate physical scene."`; the `instructions` line `First person ("I") for speech and internal thought. Describe {{user}}'s own physical actions in the tense the conversation already uses.` → `"{{narrative_perspective}} person for speech and internal thought. Describe {{user}}'s own physical actions in {{narrative_tense}} tense."` (keep surrounding impersonation rules verbatim; impersonate now follows the narrator's perspective/tense instead of hardcoding first/match).

- [ ] #### Task 4.2: Quantifier examples → third-person past, no macros (1 SP)
  - [ ] ##### SubTask 4.2.1: Rewrite the 5 example narrations in `data/prompt_presets/quantifier/default.json` `instructions` to third-person past, e.g. `"You walk through the door into the kitchen."` → `"She walked through the door into the kitchen."`; `"You examine the ancient vase carefully."` → `"He examined the ancient vase carefully."`; convert the second-person player references in the other three ("your path" → "her path"; "between you and the gate" → "between her and the gate"). Do NOT add macros — macro-driving the teaching text would yield ungrammatical `"third walks..."`. Documented deviation from ticket 16's blanket "all three presets" (flag in the resolution): the macro's coherence purpose is met by the examples matching the default voice; the quantifier keys off location, not pronoun (verified, ticket 16).

### Phase 5: HTTP settings UI (3 SP)

- [ ] #### Task 5.1: SettingsTemplate dropdowns (1 SP)
  - [ ] ##### SubTask 5.1.1: Extend `SettingsTemplate` in `src/adapters/driving/http/settings/templates/settings.rs` with `narrative_perspective: String` + `narrative_tense: String`. Add a new card section (after Text Check) with two `<select>` dropdowns — Perspective (Second/Third), Tense (Past/Present) — each with `selected` on the current value, posting to `/settings/narrative-voice`, `hx-target="#settings-status"`.
  - [ ] ##### SubTask 5.1.2: `from_settings` populates the two fields via `settings.narrative_perspective.as_str()` / `narrative_tense.as_str()`.

- [ ] #### Task 5.2: Form + handler + route (2 SP)
  - [ ] ##### SubTask 5.2.1: Add `NarrativeVoiceForm { narrative_perspective: String, narrative_tense: String }` and `save_narrative_voice_handler` in `src/adapters/driving/http/settings/handlers/settings.rs`, mirroring `save_text_check_handler`: lock settings, parse via `from_str` (fallback to default on unknown), set the fields, `save_settings`, return `Html("Narrative voice saved!")` or the error span.
  - [ ] ##### SubTask 5.2.2: Register the route `/settings/narrative-voice` (POST) in `src/adapters/driving/http/builders/router.rs` beside `/settings/text-check`.

### Phase 6: Tests (3 SP)

- [ ] #### Task 6.1: `src/domain/model/settings_tests.rs` (1 SP)
  - [ ] ##### SubTask 6.1.1: Create the file and register `#[cfg(test)] mod settings_tests;` in `src/domain/model/mod.rs`.
  - [ ] ##### SubTask 6.1.2: Tests — `AppSettings::default()` is Third/Past; round-trip serialize/deserialize preserves Second/Present; old JSON (no `narrative_*` keys) deserializes to Third/Past (serde defaults); `as_str`/`FromStr` round-trip for all four values; unknown string falls back per the `FromStr` decision.

- [ ] #### Task 6.2: `template_tests.rs` macro + preset-coherence tests (1 SP)
  - [ ] ##### SubTask 6.2.1: `render_template` substitutes `{{narrative_perspective}}`/`{{narrative_tense}}`; unknown macros left as-is; `TemplateVars::new`/`from_persona` default to `"third"`/`"past"`.
  - [ ] ##### SubTask 6.2.2: Load the real preset files via `include_str!("../../../data/prompt_presets/system/default.json")` (and impersonate), parse to `PromptPreset`, and for each of the 4 value combos (Second/Past, Third/Past, Second/Present, Third/Present) render `writing_style` (via `preset.assemble_text` or `render_field_parts` with a `TemplateVars` carrying that combo) and assert coherent English (e.g. `"third-person limited perspective..."`, `"second-person perspective as Julian. past tense. ..."`). Loading the real JSON pins the actual macro text and catches JSON typos.

- [ ] #### Task 6.3: Central-injection + HTTP tests (1 SP)
  - [ ] ##### SubTask 6.3.1: In `src/application/prompting/assembler_tests.rs` add a test: construct `PromptAssembler::new(..).with_settings(Second/Present)`, call `assemble` with a system preset whose `writing_style` uses the macros, assert the assembled system prompt contains `"second-person"` and `"present tense"`. Add a second test: an assembler WITHOUT `.with_settings` produces the default `"third"`/`"past"` rendering (regression guard for the settings-less arrival/test path).
  - [ ] ##### SubTask 6.3.2: Extend `tests/http/settings.rs` — panel render asserts the two dropdowns and their options exist; new scenarios for `POST /settings/narrative-voice` (sets Second+Present, sets Third+Past, unknown value falls back, save-failure reports the error span). Use `SettingsTestGuard` as the existing settings tests do. Tag against `docs/specs/settings.md` (add scenarios 20.x there only if the spec gates it — check whether the spec requires the new scenarios before asserting spec tags).

### Phase 7: Validation (1 SP)

- [ ] #### Task 7.1: Build green (1 SP)
  - [ ] ##### SubTask 7.1.1: `cargo fmt && cargo clippy --all-targets -- -D warnings`, `cargo test --lib`, then `python build.py` (full gate: fmt + clippy + guardrails + tests). The preset text changes alter rendered prompt content (string substitution, not parsing) — per AGENTS.md LLM policy, run `python build.py --llm-only` as well if any LLM test touches the preset rewrite; expected not to, but verify.

## Test Plan

- **Unit (`cargo test --lib`):** new `settings_tests.rs` (enum/serde/defaults), extended `template_tests.rs` (macro substitution + 4-combo preset coherence loading real JSON), new `assembler_tests` central-injection test (the one new code path) + settings-less default regression guard. Existing `assembler_tests`/`sections_tests` `PromptContext::new` sites use settings-less assemblers or default settings → render third/past → unchanged (regression guard).
- **HTTP (`tests/http/settings.rs`):** dropdown render + `POST /settings/narrative-voice` success/fallback/failure.
- **Storage (`tests/storage/` + `bootstrap/run_tests.rs`):** migration v18 applies cleanly on a v17 DB; `get_settings`/`save_settings` round-trip the two new columns; an old v17 row loads with Third/Past defaults. (`bootstrap/run_tests.rs` presets are built inline via `serde_json::json!`, not read from `data/`, so the preset-file rewrite does not break them — verified.)
- **Guardrails:** enum-variant doc, inherent-impl locality (enums in `settings.rs`, injection in `assembler.rs`), free-fn location (default fns in `utils/settings_defaults.rs`), import ordering.
- **Final gate:** `python build.py` green; `python build.py --llm-only` if preset changes touch LLM tests.

## Per Task/Sub Task Validation Steps

- After Phase 1: `cargo test --lib domain::model::settings_tests domain::model::template_tests` passes; `cargo clippy` clean.
- After Phase 2: `cargo test --lib` + `cargo nextest run --test architecture` (migration in `utils/plumbing.rs` — check no arch-lint violation); rely on `tests/storage/` round-trip for the migration.
- After Phase 3: `cargo test --lib application::prompting` (central injection + arrival compile; existing assembler tests pass on defaults).
- After Phase 4: `cargo test --lib domain::model::template_tests` (preset-coherence tests loading real JSON) — no `bootstrap/run_tests.rs` assertion pins old `writing_style` (verified).
- After Phase 5: `cargo nextest run --test guardrails` (route/handler locality) + `tests/http/settings.rs` green.
- After Phase 6: all new tests green.
- After Phase 7: `python build.py` (and `--llm-only` if triggered) green.

## Assumptions

- **Voice injection centralized in `PromptAssembler::assemble` (user-confirmed).** The assembler already holds `settings` and reads it for `resolve_budget`; extending it to patch `template_vars` voice is consistent with its role as the settings→prompt seam. Covers all four pipeline/retry sites (which all call `self.pipeline.prompt_assembler.assemble` — verified) with one ~5-line change; no edits to `pipeline_run.rs`/`retry.rs` production code; no `with_narrative_voice` builder. Settings are the source of truth, so the injected value overrides the context's default — correct.
- **Arrival reads `Storage::get_settings()` once** and sets the two `template_vars` fields directly (arrival's assembler is settings-less). No `ArrivalTaskContext` constructor churn; one extra DB read on the arrival path only.
- **Preset migration = seed files only (user-confirmed).** Rewriting `data/prompt_presets/*/default.json` updates only fresh DBs. `ensure_presets` (`bootstrap/run.rs`) skips any preset id that already exists with content, so existing DBs keep the old hardcoded-perspective text and the new setting is cosmetic for them until they re-seed/edit. No destructive preset-content migration; user-owned presets respected.
- **Settings form = separate `/settings/narrative-voice` form (user-confirmed)**, mirroring `save_text_check_handler`/`/settings/text-check`, keeping connection selection and narrative voice as independent concerns.
- **Storage = scalar TEXT columns + `as_str`/`FromStr`** (matches the top-level `AppSettings` scalar precedent `active_*_prompt_preset_id`; the `TextCheckMode`-in-JSON-blob precedent does not apply to top-level scalars). The `as_str`/`FromStr` converters are the necessary column↔enum bridge; a JSON-blob alternative would still need a migration plus a new struct, so it is strictly larger.
- **Macro values are lowercase** (`third`/`past`/`second`/`present`), per `#[serde(rename_all = "lowercase")]`. The system preset's `writing_style` renders `"third-person limited perspective..."` (lowercase at line start, a minor stylistic shift from `"Third-person..."`); accepted per ticket 19's explicit template strings. No test pins the old capital-T string (verified).
- **Quantifier preset gets NO macros** (deviation from ticket 16's blanket "all three presets", per ticket 19's explicit override) — examples hardcoded third-person past to avoid ungrammatical `"third walks..."` teaching text. Flagged in the resolution.
- **Docs deferred to `chronicler-after-plan-workflow`** (per AGENTS.md) — `prompt_system.md` macro list and `storage.md` migration v18 updated after implementation, not as code tasks here.
- **No `--llm-only` expected** (macros are string substitution, not prompt-logic/parsing changes), but run it if the build's LLM tests touch the rewritten preset text.
