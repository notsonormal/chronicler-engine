# Invariants the World theme relies on but does not enforce

Scope: World, Scenario, Room, Character, Trigger, Persona, settings and prompt preset data.
Read-only investigation. No build was run. Terms follow `CONTEXT.md`.

---

## world-01 — The Scenario a Game starts in is `scenarios[0]`, not the one `default_scenario_id` names

**Statement.** The starting Scenario of a World is its first Scenario in list order, and
`default_scenario_id` must agree with that order (or not exist).

**Relied on at**
- `src/domain/model/world.rs:69-76` — `WorldCard::default_scenario()` returns `self.scenarios.first()`;
  `starting_room_id()` reads only that Scenario.
- `src/application/generation/gate.rs:764-767` and `src/domain/model/utils/opening.rs:16-19` — a new
  Game's opening Message and Swipe state come from that one Scenario. A Game in the wrong Scenario
  starts in the wrong Room with the wrong opening text, and no error is raised.

**Write paths that can violate it**
- World HTTP create: `src/adapters/driving/http/worlds/handlers/worlds.rs:76-80` — the posted
  `scenarios_json` is accepted, and `default_scenario_id` is hard-set to `None`.
- World HTTP update: `worlds.rs:191-193` — the stored id is only *filtered* to an existing Scenario;
  it is never used to reorder or select. So a stored id that names a non-first Scenario is kept and
  silently ignored.
- Seed: `src/domain/model/world.rs:88` copies the manifest field; `bootstrap/load.rs:115-124` writes
  it on first seed only.
- Any DB edit of `worlds.scenarios` / `worlds.default_scenario_id`.

**Enforcement.** PARTIAL. `scripts/validate_data.py:126-133` validates the Room of the
`default_scenario_id` Scenario and falls back to `scenarios[0]`. So the build checks one Scenario
while the server runs the other whenever the two disagree. No runtime reader consults
`default_scenario_id` at all (grep: only writes in `storage/worlds.rs:230,321` and the
`filter` in the HTTP handler).

**Consequence.** A Game silently starts in the wrong Room and shows the wrong opening Narration.
The player sees a story that does not match the World the author configured.

**Likelihood.** A World author who sets `default_scenario_id` to a non-first Scenario, through the
World form or a `world.json`. The shipped Worlds have no `default_scenario_id`, so today's data
hides it.

**Epistemic status.** VERIFIED (both readers, all writes).

**Suggested enforcement.** Make the default Scenario explicit: a constructor on
`WorldCard`/`WorldManifest` that resolves `default_scenario_id` and refuses an id that names no
Scenario, and have `default_scenario()` use it.

---

## world-02 — A Game's current Room resolves in its World's map (or in its added Rooms)

**Statement.** `state.movement.current_room_id` names a Room of the World's map, or one of the
Game's added Rooms, for every Swipe state of every Game.

**Relied on at**
- `src/application/games/view_query.rs:63-65` — the Room header read fails with `RoomNotFound`, so
  the dashboard's Room name and image for that Game break.
- `src/application/pipeline/job.rs:225-231` — `room_check` refuses the Job with `invalid_room`, so
  every Action, Retry and Autoplay Cycle of that Game fails.
- `src/domain/model/utils/game_rules.rs:245-255` — the same check exists only under the
  `diagnostics` feature.

**Write paths that can violate it**
- Game create: `src/application/games/catalogue.rs:47-73` never reads the map, and
  `src/domain/model/state/swipe_state.rs:34-45` copies the starting room string without a lookup.
  `starting_room_id()` falls back to `"start"` when the World has no Scenario
  (`world.rs:73-77`).
- World HTTP create/update: `worlds.rs:120-160` and `worlds.rs:163-217` accept any map JSON; no Room
  of the old map is consulted. The Browser form offers the map as a free JSON textarea
  (`worlds/templates/worlds.rs:111`).
- World storage write: `src/adapters/driven/storage/worlds.rs:297-334` replaces `maps.map_data`
  wholesale, so every Game of that World can lose its Room.

**Enforcement.** PARTIAL. `scripts/validate_data.py:132-133` checks the seed file's starting Room at
build time only. The HTTP create/update paths and Game create check nothing.

**Consequence.** A newly created Game is unplayable from its first Action: the header answers an
error and the Job refuses to start. The Game still exists, so it stays broken.

**Likelihood.** A normal player. Create a World with `scenarios_json` `[]` (the form's own
placeholder is `Example: []`) or a `starting_room_id` typo, then create a Game. Editing the map
JSON of a World that already has Games breaks those Games the same way.

**Epistemic status.** VERIFIED.

**Suggested enforcement.** One validator for a `(WorldCard, MapDef)` pair, called by
`WorldCatalogue::create_world`, `update_world` and before `GameWriter::create`; it returns
`EngineError::Validation` naming the offending Scenario and Room.

---

## world-03 — A Scenario's `characters` ids name Characters of the same World

**Statement.** Every id in `StartingScenario::characters_present` names a Character row of the
World.

**Relied on at**
- `src/domain/model/state/swipe_state.rs:64-87` — the seed of the opening state. An unknown id is
  dropped by `if let Some(character) = characters.get(character_id).cloned()`, so no encounter and no
  `characters_present` entry is created.
- The prompt then says "No Characters are present in this Room"
  (`src/application/prompting/assembler.rs:376-377`), and the Quantifier's answer for that Character
  is dropped at `game_rules.rs:67-71`.

**Write paths that can violate it**
- `worlds.rs:54-56` and `worlds.rs:188-190` — `scenarios_json` is deserialized and stored verbatim.
- `WorldManifest` seeding (`bootstrap/load.rs:105-124`), and any DB edit of `worlds.scenarios`.

**Enforcement.** BUILD-TIME only: `scripts/validate_data.py:43-63,150-156` checks a matching file in
`data/characters/<group>/`. Nothing checks at runtime, and nothing checks the HTTP write paths.

**Consequence.** Silent wrong opening state: the authored Character is missing from the first
scene, from the prompt and from the encounter log. The story contradicts the World data.

**Likelihood.** A World author typing a Character id wrong in the Scenarios JSON, or renaming a
Character file (the old row survives, see *Checked and enforced*: seeding is insert-only).

**Epistemic status.** VERIFIED.

**Suggested enforcement.** The same `(WorldCard, MapDef)` validator as world-02, extended with the
World's Character id set, run at World create/update and at Game create.

---

## world-04 — "The first matching Trigger fires" depends on an unordered map

**Statement.** When several Triggers match one Swipe state, the engine fires a defined first one —
the documented rule — so the same state always produces the same event Narration.

**Relied on at**
- `src/domain/model/utils/game_rules.rs:87-91` — `fired_trigger` takes
  `evaluate_triggers(...).into_iter().next()`.
- `game_rules.rs:173` — `evaluate_triggers` builds that vector by iterating
  `characters.values()`, where `characters` is a `HashMap<String, CharacterCard>`
  (`src/adapters/driven/storage/worlds.rs:27-32`), rebuilt per `world_bundle_for` call
  (`worlds.rs:129-143`). Rust's default hasher reseeds per map, so the order differs between two
  reads in the same process.
- `docs/diataxis/reference/game_flow.md:137` states the rule: "The first matching Trigger fires."

**Write paths that can violate it.** Not a data write: any World with two Characters whose Triggers
can both match the same state. No write path can express a priority.

**Enforcement.** DOC-ONLY. No ordering is imposed anywhere. Prompt rendering sorts cards
deliberately (`character.rs:68-79`, `CharacterCard::in_id_order`) — the game rules do not.

**Consequence.** The same input can fire a different Trigger on a Retry or in two consecutive Jobs:
a different event Narration, a different `event_record`, a different Character marked as fired. The
story becomes irreproducible.

**Likelihood.** An authored World where two present Characters carry satisfiable Triggers. No code
bug and no manual edit needed.

**Epistemic status.** VERIFIED.

**Suggested enforcement.** Order the candidates in `evaluate_triggers` by a stable key
(Character id, then Trigger index) and state that order in `game_flow.md`; or store an explicit
`priority` on `Trigger`.

---

## world-05 — A Trigger's `room_id` names a Room of the map

**Statement.** `Trigger::room_id`, when set, names a Room of the World's map, so the Trigger can
ever fire.

**Relied on at**
- `src/domain/model/utils/game_rules.rs:175-190` — a mismatch is a `tracing::debug!` plus
  `continue`; the Trigger is silently skipped for the life of the Game.

**Write paths that can violate it**
- Character data: `data/characters/**/*.json` → `storage/characters.rs:46-88` (opaque
  `triggers` JSON).
- No in-app Character write path exists.

**Enforcement.** BUILD-TIME only: `scripts/validate_data.py:164-171` checks `room_id` against the
map's Room ids for seed files. Nothing checks at runtime, and the read path tolerates the
violation instead of reporting it.

**Consequence.** A one-shot or repeat Trigger never fires. The authored event Narration never
appears, with only a DEBUG line as evidence.

**Likelihood.** A World author typo in a Character file. Not player-triggerable.

**Epistemic status.** VERIFIED.

**Suggested enforcement.** Keep the build check, and make the runtime comparison use
`state.room_by_id(map, room_id)` with a single warning per Game when the id is unknown to the map.

---

## world-06 — An encounter's `trigger_fired` index names the same Trigger for the life of a Game

**Statement.** `Encounter::trigger_fired` keys a fired one-shot Trigger by its position in
`CharacterCard::triggers`, so index `i` must keep naming the same Trigger.

**Relied on at**
- `src/domain/model/trigger.rs:45` (`HashMap<usize, bool>`), `:62-65` (`mark_trigger_fired`),
  `:79-85` (`is_trigger_fired`).
- `src/domain/model/utils/game_rules.rs:202-210` — the skip that makes a one-shot Trigger fire once.
- `src/domain/model/generation.rs:117-124` — an event Swipe marks `record.trigger.trigger_idx` fired.
  The same `EventRecord` also stores `trigger_name` and `trigger_repeat`
  (`src/domain/model/state/event_record.rs:13-14`), so the name alone already carries what the index does.

**Write paths that can violate it**
- Any reorder, insert or removal in a Character's `triggers` array. Today the only writer is the
  insert-only seed (`storage/characters.rs:46-88`, `INSERT OR IGNORE`), so a file edit does not
  apply; a DB edit, a future Character edit path, or a delete-then-reseed does.

**Enforcement.** NONE. No trigger id exists, nothing compares a stored index with the current list
length, and no test or script covers the mismatch.

**Consequence.** A one-shot Trigger fires twice, or never fires again; an unrelated Trigger can be
suppressed. Silent wrong story, persisted in `message_swipes.state`.

**Likelihood.** Only a DB edit or a code change that starts updating Character rows.

**Epistemic status.** VERIFIED for the reliance; INFERRED for whether any path today reorders a
stored list (I found no in-app writer).

**Suggested enforcement.** Give `Trigger` a stable `id` and key `trigger_fired` by it; migrate the
stored maps.

---

## world-07 — A Relationship's `with` names a Character of the same World

**Statement.** `Relationship::with` names another Character of the World, so the relationship can
reach the prompt.

**Relied on at**
- `src/application/prompting/builders/sections.rs:104-125` — the relationship is rendered only when
  `in_area_ids.contains(r.with)`, and the partner's name falls back to the raw id:
  `.unwrap_or(&rel.with)`.
- `src/application/prompting/assembler.rs:397` is the only caller.

**Write paths that can violate it**
- Character files → `storage/characters.rs:46-88` (opaque `relationships` JSON).
- No in-app Character write path; `data/schemas/character.schema.json:52-60` declares
  `with` as a plain string with no reference check.

**Enforcement.** NONE. `scripts/validate_data.py` checks Scenario Characters and Trigger Rooms but
never `relationships[*].with`.

**Consequence.** Silent prompt degradation: the relationship is silently dropped whenever the
partner is present, or shown with a raw id instead of a name. No error anywhere.

**Likelihood.** A World author typo in a Character file.

**Epistemic status.** VERIFIED.

**Suggested enforcement.** Add the `with` reference to `validate_data.py` and warn once at
`WorldBundle` construction when a relationship names no Character of the World.

---

## world-08 — A preset id in a settings slot names an existing preset of that slot's type

**Statement.** Every active preset id in `AppSettings` (`mode_preset_registry` slots and
`active_options_prompt_preset_id`) names a `prompt_presets` row whose `preset_type` matches the slot.

**Relied on at**
- `src/application/prompt_preset_service.rs:66-71` — `chosen_preset` resolves
  `game.active_preset_id(preset_type)` and returns `EngineError::PresetNotFound`; the Job reads it
  through `Self::read(Record::Preset, …)` (`src/application/pipeline/job.rs:240-246`), so every
  Narration, Impersonate and Options call of that mode fails.
- `src/domain/model/settings.rs:580-588` — `active_preset_id` returns the stored string with no
  existence check, and no type check follows: a Quantifier preset id placed in the System slot is
  loaded as the narration system prompt.

**Write paths that can violate it**
- `data/settings.json` seed: `src/bootstrap/load.rs:48-89` checks only the Connection slots
  (`settings.check_connection_slots()?`, `load.rs:69`). There is no `check_preset_slots`.
- `bootstrap/run.rs:155-163` skips a missing `data/prompt_presets/<type>` directory with an INFO log,
  so a deployment without that directory boots with settings naming ids that no row holds.
- `activate_preset_handler` (`prompt_presets.rs:269-299`) is safe: it requires the preset to exist.

**Enforcement.** PARTIAL. The sibling invariant (a Connection slot names an existing Connection) is
checked at boot, at `set_role`, and at removal; the preset counterpart is checked nowhere at boot.

**Consequence.** The Game stays idle but every generation fails with a preset error. A type mismatch
is worse: a wrong prompt is silently used, so the model gets Quantifier instructions as its system
prompt.

**Likelihood.** A fresh DB whose `data/prompt_presets` directory is missing or renamed, or a
hand-edited `data/settings.json`. A normal player cannot reach it (the preset delete path is
guarded).

**Epistemic status.** VERIFIED.

**Suggested enforcement.** `AppSettings::check_preset_slots(&storage)` beside
`check_connection_slots`, called from `seed_settings`, refusing a missing id or a `preset_type`
mismatch.

---

## world-09 — A prompt preset seed's `preset_type` matches the directory it sits in

**Statement.** The `preset_type` a seed file declares is the type it is stored with.

**Relied on at**
- `src/bootstrap/run.rs:197-263` — `process_preset_file` takes `preset_type` from the loop's
  directory (`run.rs:149-156`) and ignores the file's own field. The seed files declare it
  (`data/prompt_presets/system/default.json:10`, `"preset_type": "System"`).
- The stored type then decides every slot the preset can be activated into
  (`prompt_preset.rs:36-38`, `settings.rs:592-602`).

**Write paths that can violate it**
- A seed file placed in the wrong directory, or whose `preset_type` field is edited without moving
  the file. `data/prompt_presets/*/*.json` is authored data.

**Enforcement.** NONE. The field is neither validated nor compared; no script checks preset seeds at
all (`scripts/validate_data.py` covers worlds, maps, settings, characters and personas).

**Consequence.** A Quantifier prompt stored as a System preset is used as the narration system
prompt for every Game in that mode, and the operator sees nothing.

**Likelihood.** A preset author moving or copying a file between directories.

**Epistemic status.** VERIFIED.

**Suggested enforcement.** Refuse a seed whose `preset_type` differs from its directory, in
`process_preset_file`, and add the preset seed files to `validate_data.py`.

---

## world-10 — Exactly one `maps` row per World

**Statement.** A World has one map row, so the map read returns the map that was last written.

**Relied on at**
- `src/adapters/driven/storage/worlds.rs:100-106` and `:355-361` — `query_row` on
  `WHERE world_id = ?` returns the first row and ignores any other. Two rows make the read pick an
  arbitrary map forever.

**Write paths that can violate it**
- `write_map_row` (`worlds.rs:273-296`) does UPDATE then INSERT, with no `UNIQUE(world_id)`
  behind it (`plumbing.rs:179-187` defines `maps` with a plain index
  `idx_maps_world`, not a unique one).
- `update_world` calls `write_map_row` outside a transaction (`worlds.rs:332`), while `write_world`
  wraps it in one.

**Enforcement.** NONE at the schema level. The backend mutex serializes today's writers, so only a
manual edit, a crash between the two statements of a future path, or an older version can create the
second row.

**Consequence.** A World keeps reading a stale map while the newer map row is written and never
read: Rooms disappear from every Game of that World.

**Likelihood.** No in-app path today; manual DB edit or a future transaction-less writer.

**Epistemic status.** INFERRED (I did not audit historical schema versions for a path that inserted
a second row).

**Suggested enforcement.** Add `UNIQUE(world_id)` to `maps` in a migration, and use
`INSERT … ON CONFLICT(world_id) DO UPDATE`.

---

## Checked and enforced

- **World key is unique** — `worlds.key UNIQUE` (`plumbing.rs:165`), and the user-facing create uses
  the `Insert` mode that maps the violation to `WorldAlreadyExists`
  (`storage/worlds.rs:196-215,240-247`).
- **Character id is unique per World** — `UNIQUE(key, world_id)` (`plumbing.rs:225`),
  plus the insert-only seed.
- **Persona key is unique** — `personas.key UNIQUE` (`plumbing.rs:192`).
- **A World with Games cannot be deleted** — `storage/worlds.rs:372-392` refuses `WorldHasGames`.
- **A Game's World and Persona rows exist** — protected by the line above, by the absence of a
  Persona delete path, and by `require_world`/`require_persona` failing loudly
  (`storage/worlds.rs:121-124`, `storage/personas.rs:48-53`).
- **A Connection slot names an existing Connection** — boot check
  (`load.rs:69`, `settings.rs:638-643`), `set_role` (`http/settings/handlers/settings.rs:403-415`),
  and the `InUse` refusal in `remove_connection` (`settings.rs:647-680`).
- **At least one Connection exists** — `LastConnection` refusal, and the boot slot check fails loudly
  on a seeded empty list.
- **The settings row is a singleton** — `CHECK (id = 1)` (`plumbing.rs:232`) and every statement
  binds `id = 1`.
- **A named Connection keeps a unique name** — trim-and-compare in `add_connection` and
  `update_connection` (`settings_service.rs:41-78`). Note: a `data/settings.json` seed bypasses this
  check, and it is display-only.
- **A preset name is unique per type** — trim-and-compare inside `Storage::save_preset`
  (`presets.rs:70-116`), and both the UI and the seed path (`run.rs:268`) go through it.
- **A preset reached from a mode/Game slot cannot be deleted** — `delete_preset`
  (`prompt_preset_service.rs:110-141`) refuses defaults, mode references, the Options slot and any
  Game's four preset columns (`storage/games.rs:170-191`).
- **A Game's new preset selection exists and allows the mode** — `set_preset_selection`
  (`games/catalogue.rs:205-227`), with the deliberate unchanged-id exemption.
- **`option_count` is absent or a positive count on an Options preset only** —
  `PromptPreset::validate` (`prompt_preset.rs:103-131`), called on every save path including seeds.
- **A refused custom parameter key never reaches a preset** — same `validate`, same paths.
- **`TextCheckMode::Disabled` never carries auto-check** — normalised in the domain
  (`settings.rs:315-335`) and used by the write path.
- **A Trigger cannot reference another Trigger** — `Trigger` has no such field
  (`trigger.rs:29-35`, `data/schemas/character.schema.json`), so there is no dangling reference to
  enforce.
- **Room `exits` are not read by the engine at all** — grep over `src/` finds `Room::exits` only in
  `map.rs` and tests; prompts use `navigation_description`. A bad exit target is currently harmless,
  so there is no invariant to enforce (this also means the data is unvalidated dead weight).
- **Scenario `id` uniqueness** — not relied on: nothing looks a Scenario up by id (`default_scenario`
  uses list order), so duplicates change no behaviour.
- **A Game's `world_key` names a World row** — guarded for the delete path by `WorldHasGames`, and
  `require_world` fails loudly otherwise. I did not report it as a finding: `games.world_key` has no
  foreign key (`plumbing.rs:68`), so `Storage::update_world` (`worlds.rs:297-334`) could bind a
  different `key` and orphan a Game's reference, but no HTTP path passes a changed key today. Only
  the v19/v23 migrations warn about such orphans.
