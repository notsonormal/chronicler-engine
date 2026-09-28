# Grill: Replace the per-mode preset registry with per-preset mode-allow flags + ship the IF system preset seed

Type: grilling
Status: resolved
Blocked by: (none)

## Question

Replace the `NarratorMode`-keyed preset-registry mechanism with per-preset mode-allow flags, and ship the missing IF system preset seed file. This reopens the preset-selection mechanic settled in ticket 01 (decision 8, the symmetric `ModePresetRegistry`) and absorbs the file-creation half of ticket 08. The current `settings_defaults.rs` bundle/registry construction is the hack being removed.

### What's wrong today

- **The registry is a hack.** `src/domain/model/utils/settings_defaults.rs` builds two `ModePresetBundle`s (`default_novel_preset_bundle` / `default_interactive_fiction_preset_bundle`) whose only real divergence is the system preset id (`system_default` vs `system_if_default`); quantifier and impersonate are duplicated across both bundles. `AppSettings.mode_preset_registry` (`src/domain/model/settings.rs:74`) and `ModePresetRegistry::bundle_for` exist only to carry this duplication. A new mode would force a third bundle copy of the same quantifier/impersonate ids.
- **The IF system preset does not exist.** `default_if_system_prompt_preset_id()` returns `"system_if_default"`, and the IF bundle points at it, but no `data/prompt_presets/system/if_default.json` is seeded. Any game whose mode resolves to the IF bundle will fail to load its system preset. Ticket 08 was meant to ship this file; this ticket supersedes that.

### Proposed direction (to grill, then lock)

(a) **Per-preset mode-allow flags, not a mode-keyed registry.** Add `allowed_for_novel` / `allowed_for_if` (names TBD in grilling) fields to the preset files and the `PromptPreset` model (`src/domain/model/prompt_preset.rs:53`). A preset is selectable for a game's mode when its flag for that mode is set. One quantifier preset and one impersonate preset then serve both modes by setting both flags — no second bundle, no duplicated ids. Grill what this means for: `AppSettings.mode_preset_registry` (likely removed), the per-game `active_*_preset_id` columns (likely stay — a game still selects which preset), `bundle_for` and its callers in `GameCatalogue` and the CLI boot (`src/application/games/catalogue.rs`, `src/bootstrap/init_game.rs`), and the migration v19 `mode_preset_registry` column (just landed in ticket 05).

(b) **Ship the IF system preset seed.** Author `data/prompt_presets/system/if_default.json` from the approved draft at `.scratch/narrator-modes-and-options/research/02-if-preset-draft.json` (content decided in ticket 02). Confirm the loader seeds it (`src/bootstrap/run.rs` `process_preset_file`, keyed on JSON `id` = `system_if_default`) and that it is no longer a dangling reference.

(c) **Block all open map tickets.** This ticket blocks 06–12: the registry/allow-flags mechanic and the IF seed are prerequisites for the narration pipeline (06), the UI (07), the IF seed (08, superseded here), the options work (09–11), and the spec/tests (12).

### Notes for the session

- This reverses ticket 01 decision 8 (the symmetric 2×3 registry). Read that decision in `issues/01-grill-narrator-mode-shape.md` before grilling, and record the reversal explicitly in the resolution.
- Read before grilling: `src/domain/model/prompt_preset.rs` (`PromptPreset`, `PresetType`), `src/domain/model/settings.rs:64-99` (`ModePresetBundle`/`ModePresetRegistry`/`bundle_for`), `src/domain/model/utils/settings_defaults.rs` (the bundle/registry fns), `src/application/games/catalogue.rs` (`create_game`/`reset` — the `bundle_for` callers), `src/bootstrap/init_game.rs` (`resolve_game_id`), `src/adapters/driven/storage/utils/plumbing.rs` (v19 `mode_preset_registry` column), `data/prompt_presets/system/default.json` (sibling shape), `.scratch/narrator-modes-and-options/research/02-if-preset-draft.json` (approved IF content).
- Open questions to resolve in grilling, not pre-decide: exact field names and shape (two bools vs one enum vs a list); whether `ModePresetRegistry` is removed outright or retained as a thin default-id lookup; whether the v19 migration needs a follow-up (v20) to drop/reshape the `mode_preset_registry` column, or whether the column is repurposed; how the preset panel filters/selects by mode (ticket 07's UI surface); whether `is_default` semantics interact with the new flags.
- Per the map's Notes override: decide, then implement, then build-green. The resolution is the decision; implementation may follow in the same session if the decision is clean.
- Skills: `/grilling`, `/domain-modeling`.

## Comments

### Relationship to existing tickets

- Supersedes the file-creation half of **08** (the IF seed). 08 remains on the map only as blocked-by-13; if 13 ships the seed, 08 should be closed as superseded with a pointer here.
- Reopens **01** decision 8. The resolution must state the reversal.
- Does **not** reopen ticket 05's posture relocation (posture on world/game stays); only the preset-selection mechanic changes.

## Answer

**Grilling complete — 5 decisions across 6 rounds. The frontier is empty. The user chose grilling-only for this session; implementation is spun to ticket 14.**

### The decision, in one line

Presets carry per-preset `allowed_modes` flags gating **selection surfaces only**; the per-mode registry **survives** as user-configurable settings state, reshaped into a mode-tagged JSON list; activation is mode-targeted and flags-validated; the IF system seed ships with the implementation (ticket 14).

### Settled decisions

1. **Flag shape (Q1=B).** `PromptPreset.allowed_modes: Vec<NarratorMode>` — one list, not per-mode bools. Missing field = allowed for all modes (back-compat). Editor requires ≥1; panel-created presets default to both. Seed values: `system_default` → `["novel"]`, `system_if_default` → `["interactive_fiction"]`, `quantifier_default` / `impersonate_default` → both.

2. **The registry survives (Q2 — user override of the ticket's premise).** The ticket proposed replacing the registry with flags. Overruled: configuring the default preset in settings must remain possible (the pre-05 capability; ticket 01 decision 7 preserved it deliberately). Flags and the registry answer different questions — flags = what is *selectable* per mode; the registry = what is the *default* per mode (new games, resets, mode-switch retargets, no-game fallback). **Ticket 01 decision 8 is refined, not reversed**: the registry stays, reshaped and complemented by flags.

3. **Registry shape (Q4 — user picked the list).** `ModePresetBundle` gains `mode: NarratorMode`; `ModePresetRegistry` becomes a Vec-backed newtype; `bundle_for(mode)` = find-by-mode, falling back to constructed defaults for a missing entry. `BTreeMap<NarratorMode, _>` was rejected as obscure — keying buys nothing at two entries.

4. **Registry storage (Q4b=A).** One JSON column on the settings singleton — the status-quo location, list-shaped content. Rejected: M×T flat columns (multiplicative DDL across modes × preset types) and a normalized `mode_preset_defaults` child table with FK. The FK option was seriously weighed — it would structurally prevent the dangling-reference class — but costs a second table, a hydration boundary into `AppSettings`, a boot-ordering wrinkle (migrate runs before seeding), and would be the first FK inside the settings aggregate. The dangling-reference class is handled the engine's way instead: the delete handler gains a registry reference check.

5. **Seeding stays insert-only (user correction).** "Seeding is seeding": the seeder reads `allowed_modes` from seed JSON and applies it **at insert only**; existing rows are never touched at boot — flags are user-owned from birth. (An earlier boot-time flag-refresh proposal was withdrawn.) The v20 migration owns the one-time backfill: the column is born all-allowed on every existing row, then a single UPDATE tightens `system_default` → `["novel"]` — no user value can pre-exist a column at birth. `system_if_default` cannot pre-exist anywhere (its seed ships in 14) and arrives IF-only from JSON.

6. **Gate scope (Q3).** Flags filter selection surfaces only: the per-game preset picker (07), the mode-switch retarget (07), activation validation (14). The narrate path never re-validates a game's stored ids — curation, not a hard mode-gate (consistent with the map's no-mode-gating preference).

7. **Activation is mode-targeted (Q5 — user chose per-mode over all-allowed-bundles).** Activating a preset sets it as the default for ONE chosen mode, refused when that mode ∉ `allowed_modes`. Per-mode divergence becomes a deliberate, visible act (badged), which also retires the drift bug in the current handler (`prompt_presets.rs:285-295` writes only the Novel bundle). All-allowed activation was rejected as too restrictive: a both-modes preset would be forced to default for both. Endpoint semantic = ticket 14 (the existing single button defaults to Novel until then); per-mode buttons/badges/editor checkboxes/picker filtering = ticket 07.

8. **Mode→default consumers unchanged.** New-game inheritance, reset, mode-switch retarget, and the no-game fallback all read the registry as before — only the shape changed.

### Facts discovered during grilling (current tree)

- The registry serves four consumers, not one: new-game inheritance (`catalogue.rs:51`, `init_game.rs:42`), the panel's global-active state (`prompt_presets.rs:285-295` write; `:44-56`, `:94-104` reads), the no-game fallback (`games.rs:158-196`), and the pending mode-switch retarget (07).
- Pre-existing gap at HEAD and in the working tree: `delete_preset_handler` never checked references — deleting a preset that is a mode default already produces a dangling id today. Decision 4's delete check closes it for the registry.
- `is_default` today means only "seeded + protected from edit/delete"; the seeder keys off JSON `id`, manually maps fields (new seed fields are dropped unless the loader learns them), and refreshes an existing seed row only when it has no content (`run.rs:296-304`).

### Bookkeeping

- **Implementation = [ticket 14](issues/14-implement-preset-mode-flags-registry-and-if-seed.md)**: model flags, registry reshape, v20 migration, seeder, seed files (incl. the IF seed — absorbing 08's file half), handler semantics, tests, build-green. Migration v20 belongs to 14; **ticket 09 updated to v21**.
- **08 closed as superseded** (pointer → 14). **07** scope 2 restated (per-mode activation UI, editor checkboxes, picker filtering). **12** registry phrasing updated. **06/09/10/11/12** blockers re-pointed 13 → 14.
