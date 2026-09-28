# Implement: per-preset mode-allow flags + registry list reshape + IF system preset seed

Type: task
Status: resolved
Blocked by: (none)

## Question

Implement the preset-selection redesign locked in ticket 13's Answer: per-preset `allowed_modes` flags, the registry reshaped to a mode-tagged JSON list, migration v20, mode-targeted activation + delete reference check, and the IF system preset seed (absorbing ticket 08's file half). This is implementation per the map's Notes override (decide, then implement, then build-green).

### Scope

1. **Domain model.** `PromptPreset.allowed_modes: Vec<NarratorMode>` (`src/domain/model/prompt_preset.rs`) — serde default = all modes; an `allows(&NarratorMode) -> bool` helper; presets created via the panel default to both modes. `ModePresetBundle` gains `mode: NarratorMode`; `ModePresetRegistry` becomes a Vec-backed newtype; `bundle_for` finds by mode and falls back to constructed defaults for a missing entry; update the constructors in `src/domain/model/utils/settings_defaults.rs`.

2. **Storage + migration v20.** The `presets` table gains an `allowed_modes` TEXT column (JSON array). Migration v20 (`src/adapters/driven/storage/utils/plumbing.rs`): (a) add the column, backfill every existing row all-allowed; (b) one-time tighten `system_default` → `["novel"]` (at column birth no user value can exist); (c) reshape `settings.mode_preset_registry` from the v19 object-keyed JSON to the mode-tagged list, preserving user-customized bundle values. Update the storage models/mappers (`models/settings.rs`, `presets.rs`, the in-memory backend) for the new shapes.

3. **Seeder.** `process_preset_file` (`src/bootstrap/run.rs`) reads `allowed_modes` from seed JSON at insert only; existing rows are never touched at boot — flags are user-owned from birth (ticket 13 decision 5).

4. **Seed files.** Ship `data/prompt_presets/system/if_default.json` from the approved draft `.scratch/narrator-modes-and-options/research/02-if-preset-draft.json` with `"allowed_modes": ["interactive_fiction"]`. Add `allowed_modes` to the three existing seeds: `system/default.json` → `["novel"]`, `quantifier/default.json` and `impersonate/default.json` → `["novel", "interactive_fiction"]`.

5. **Handlers.** Activate becomes mode-targeted (`src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs`): `(preset_id, mode)` writes that mode's bundle slot, refused when mode ∉ `allowed_modes`; the panel's existing single activate button defaults to Novel until ticket 07's per-mode UI. The delete handler refuses presets referenced as any mode's default.

6. **Tests.** Update `settings_tests.rs`, storage `settings_tests.rs`, `prompt_presets_tests.rs`, and `run_tests.rs` for the list shape + flags; add unit tests for `allows`, the `bundle_for` fallback, the v20 reshape/backfill, and seeder flag-reading. Register per `scripts/check_test_structure.py`.

7. **Build-green.** `python build.py`. Novel mode is behaviorally unchanged; the panel looks the same (ticket 07 adds the new UI).

### Notes for the session

- The design rationale lives in ticket 13's Answer — read it first; do not re-derive decisions there.
- Migration v20 belongs to THIS ticket; ticket 09 (options) takes v21 — its text is already updated.
- The panel UI (per-mode activate buttons, per-mode default badges, editor `allowed_modes` checkboxes, per-game picker filtering) is ticket 07's surface — do not build it here.
- Read before implementing: ticket 13's Answer; `src/domain/model/prompt_preset.rs`; `src/domain/model/settings.rs` (`ModePresetRegistry`); `src/domain/model/utils/settings_defaults.rs`; `src/adapters/driven/storage/utils/plumbing.rs` (the v19 block to mirror); `src/adapters/driven/storage/{settings.rs,presets.rs}` + `models/settings.rs`; `src/bootstrap/run.rs` (`process_preset_file`); `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs` (activate/delete).
- Re-verify every file path against the current tree before editing (map standing preference).
- Skills: `/domain-modeling`.

## Answer

Implemented per ticket 13's Answer, plan-reviewed (improve-ai-plan) before coding. `python build.py` fully green (fmt, clippy, guardrails, full test suite; log `logs/build_20260829_195021.log`).

### What shipped

1. **Domain model** — `PromptPreset.allowed_modes: Vec<NarratorMode>` with `#[serde(default = "settings_defaults::default_allowed_modes")]` (missing = all-allowed) and `allows(mode) -> bool`. `ModePresetBundle` gains `mode: NarratorMode`; `ModePresetRegistry` is a `Vec`-backed newtype with `bundle_for(mode) -> ModePresetBundle` (find-or-constructed-fallback, owned return), `set_bundle` (replace-or-push), and `references(id)` (any-mode-default check). `settings_defaults::default_bundle_for_mode` replaces the two per-mode bundle fns; consumers updated (`catalogue.rs` ×2, `init_game.rs`, `games.rs` fallback).
2. **Migration v20** (`plumbing.rs`) — guarded `ALTER TABLE prompt_presets ADD COLUMN allowed_modes` (born all-allowed), one-time tighten of `system_default` → `["novel"]`, and a single self-referential UPDATE reshaping `settings.mode_preset_registry` from the v19 object to the mode-tagged list (custom values preserved; `json_type='object'` guard keeps partial application re-run safe). `user_version = 20`.
3. **Seeder** — `process_preset_file` reads `allowed_modes` at insert only; missing/null → both modes; present-but-invalid → warn + both (a hard error would skip the preset — `ensure_presets` failures only warn at boot). The empty-content refresh branch preserves the existing row's flags.
4. **Seeds** — `data/prompt_presets/system/if_default.json` ships verbatim from the ticket 02 draft + `"allowed_modes": ["interactive_fiction"]`; `system/default.json` → `["novel"]`; quantifier + impersonate → both. Each existing seed gained exactly one line (byte-faithful, no reformat).
5. **Handlers** — activate takes an optional `?mode=` query param (absent/invalid → Novel), refuses with an error span before any save or in-memory commit when the mode ∉ `allowed_modes`, then writes that mode's bundle via `set_bundle`; route unchanged. Delete refuses presets referenced as any mode's default (`references`). Update carries existing flags (form has none until ticket 07). Panel/card/edit reads use `bundle_for(Novel)`; templates and routes untouched.
6. **Tests** — 17 unit + 2 integration tests added/updated: `allows`, serde default, registry list serde round-trip, `bundle_for` fallback (empty + partial), `set_bundle` replace/push, v20 upgrade test (simulated v19 DB: reshape preserves custom ids, backfill, tighten, `user_version=20`), SQLite `allowed_modes` round-trip, corrupt DB JSON → `EngineError::Parse`, seeder flag application/fallback/refresh-preservation, activate Novel-slot write, flags refusal (settings untouched), `?mode=`-targeted write, delete reference refusal. All in existing mirrors — no new files, no registration needed.

### Deviations from the plan (all small, review-justified)

- `PromptPreset` got a manual `Default` impl (was derived) so `..Default::default()` yields all-modes instead of an empty list — keeps the derive-Default and serde-default semantics from disagreeing (the review's "empty = selectable nowhere" trap).
- `ModePresetRegistry::references(id)` added as a registry method rather than inline handler logic.
- `ActivateQuery` re-exported from `handlers/mod.rs` for test access.

### Facts for later tickets

- Boot verification (scratch DB, port 3040): first boot logs `Seeded system prompt preset: system_if_default`; second boot logs no `Seeded`/`Updated` prompt-preset lines — insert-only confirmed end-to-end.
- The engine logs to `logs/chronicler_<date>.log`, not stdout — relevant to any future boot-verification steps.
- Ticket 07's per-mode UI appends `?mode=novel|interactive_fiction` to `POST /prompt-presets/:id/activate`; the editor's `allowed_modes` checkboxes should write via `PresetForm` + a new form field (update path currently preserves flags precisely because the form lacks the field).
