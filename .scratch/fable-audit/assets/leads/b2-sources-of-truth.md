# b2 — State and data with more than one source of truth

Lens: one datum, several homes. Each row names where the copy is written, where it
is read, and one sequence of actions that makes the copies differ.
Read-only audit. Code quoted as it stands at the time of reading.
Labels: **known** (I read the code that shows it), **inferred** (the code reads that
way, and I did not run it), **guessed** (no proof).

## Summary

| # | Datum | Homes | Label | Seriousness |
|---|---|---|---|---|
| 1 | The active prompt preset | settings (`mode_preset_registry`, `active_options_prompt_preset_id`) and the Game row | known | high |
| 2 | A Role's health | the pruned `llm_messages` window | known | high |
| 3 | The Room name above each story entry | recomputed from the live World map at read time | known | high |
| 4 | The Trigger that fired for an event Narration | `kind`, the Swipe state, `event_header` | known | high |
| 5 | Who wrote a Swipe | `maker`, `impersonated`, `kind`, `message_type` | known | medium |
| 6 | "The run failed" for Options | `game_docks.notice`, `games.failure`, `llm_messages` | known | high |
| 7 | The text-check mode value | template strings, handler strings, serde name, schema enum | known | high |
| 8 | The status display's machine state and label | Rust `status_element`, the JS that reads `data-status` | known | medium |
| 9 | The provider list | enum, `ALL`, `SELECTABLE`, `wire_id`, schema enum | known | medium |
| 10 | The default preset ids | Rust consts, the preset seeds, a migration literal, `data/settings.json` | known | medium |
| 11 | The default Agents | `AgentConfig::defaults`, `data/settings.json` | known | medium |
| 12 | The wire string of each small enum | `as_str`, `FromStr`/`TryFrom`, the serde rename | known | medium |
| 13 | An Agent's name | `AgentConfig.name`, the literal in `from_config`, `Role::agent_name()` | known | medium |
| 14 | The Autoplay limits | `Budget`, the shell's `<input>` | known | low |
| 15 | The posture fields of a World | the Rust manifest, `world.schema.json` | known | medium |

---

## 1. The active prompt preset has a settings home and a Game home

All four preset slots have two homes: a settings-level slot and a Game row. The
Presets tab marks the first; every model call reads the second.

Written, settings side:

- `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs:292` —
  `settings.set_active_preset(preset.preset_type, mode, id.clone());`
- `src/domain/model/settings.rs:580` —
  `pub fn active_preset_id(&self, preset_type: PresetType, mode: NarratorMode) -> &str {`

Written, Game side, at create and at a mode switch:

- `src/application/games/catalogue.rs:287` — `system_prompt_preset_id: bundle.system_prompt_preset_id,`
- `src/application/games/catalogue.rs:290` —
  `options_prompt_preset_id: settings.active_options_prompt_preset_id.clone(),`

Read, settings side (the "Active" badge):

- `src/adapters/driving/http/builders/presets.rs:194` —
  `let is_active = settings.active_preset_id(preset.preset_type, mode) == preset.id;`
- `src/adapters/driving/http/builders/presets.rs:212` —
  `let is_active = settings.active_preset_id(preset.preset_type, NarratorMode::Novel) == preset.id;`

Read, Game side (every model call):

- `src/application/prompt_preset_service.rs:67` — `let id = game.active_preset_id(preset_type);`

Snapshot on purpose? No. No comment says the Game's preset ids are a snapshot of
settings. The Settings model calls its field "active" and the Presets page prints an
"Active" badge for it.

How they differ. The player opens Prompt Presets and clicks "Set Active (Novel)" on a
new system preset. `set_active_preset` writes the mode bundle. The badge moves to the
new card. The running Game keeps the id it copied at create, so `chosen_preset` still
returns the old instructions, and the Games tab's own preset picker still shows the
old id (`src/adapters/driving/http/games/templates/games.rs:277`). The player sees two
different presets marked active for the same kind, and the narrator ignores the one the
player just picked. The Options slot behaves the same way: `active_options_prompt_preset_id`
is read by the badge and copied into the Game row at create (`catalogue.rs:290`), so it
drifts too.

## 2. A Role's health is derived from a log that retention deletes

Health is "the newest record for this role", read from a table that keeps 50 rows.

- `src/adapters/driven/storage/llm_messages.rs:11` —
  `pub const LLM_MESSAGE_RETENTION_LIMIT: usize = 50;`
- `src/adapters/driven/storage/llm_messages.rs:44` —
  `[LLM_MESSAGE_RETENTION_LIMIT as i64],` in
  `"DELETE FROM llm_messages WHERE id NOT IN (SELECT id FROM llm_messages ORDER BY created_at DESC LIMIT ?1)"`
- `src/application/games/view_query.rs:260` —
  `.filter(|message| message.agent_name == role.agent_name())`
- `src/adapters/driving/http/builders/headers.rs:36` — `/// Role health is engine-wide, so a failure can outlive the game it came from until`
  `:37` — `/// the role's next call succeeds.`
- `src/adapters/driving/http/builders/headers.rs:66` —
  `fn classify(health: Option<&'a RoleHealth>) -> Self {`

Snapshot on purpose? The comment says a failure lasts "until the role's next call
succeeds". The code cannot keep that promise. It lasts until 50 newer records of any
role push the row out.

How they differ. The Quantifier fails once. The failure banner reads
"Quantifier failed — keeping the Room and the Characters present". The player then runs
40 Narrator calls and 12 Options runs. `save_llm_message` prunes; the Quantifier's row
goes. `role_health` finds no row for `quantifier`, so `classify` returns `NoCalls`, and
the Settings cell reads "No calls yet". The failure is gone though no Quantifier call
has succeeded. The player sees the engine repair itself with no change of behaviour.

## 3. The Room name in the story log is recomputed from the live World map

The log's location header is not stored. It is derived at read time from the Swipe
state's room id and the World map as it stands now.

- `src/application/games/view_query.rs:170` —
  `.map(|(_, state)| state.room(&world.map).map(|room| room.name.clone()))`
- `src/domain/model/state/swipe_state.rs:59` — `pub fn room_by_id<'a>(&'a self, map: &'a MapDef, room_id: &str) -> Option<&'a Room> {`
- `src/domain/model/state/swipe_state.rs:60` — `map.get_room_by_id(room_id)`
- `src/domain/model/state/swipe_state.rs:61` — `.or_else(|| self.movement.dynamic_rooms.get(room_id))`
- `src/domain/model/story_window.rs:31` —
  `Some(name) if before != Some(name) => Some(name.clone()),`

Snapshot on purpose? No. The comment at `src/domain/model/story_window.rs:21` says
"the room name of each entry's active Swipe state", so the room id is the stored fact
and the name is a join.

How they differ. The player walks into a new Room that the Quantifier named "gardens".
`walk` adds it to `dynamic_rooms` because the map has no such id. The narration says
"the gardens". The author later adds a Room called "East Gardens" with the id `gardens`
to the map. Every past entry of every Game of that World now shows "East Gardens" above
text that says "the gardens". Two dynamic Rooms with the same name and the dedupe rule
hide the change entirely: `room_headers` compares names, so an entry that moves between
two Rooms of one name shows no header at all.

## 4. The fired Trigger has three stored copies, and two features read different ones

For an event Narration the Trigger record exists in the Message kind, in the Swipe
state of the previous Message, and as an `event_header` string. A Retry reads one; a
Retrigger reads another.

- `src/domain/model/generation.rs:128` — `kind: MessageKind::EventNarration(record.clone()),`
- `src/domain/model/generation.rs:130` — `event_header: Some(record.header_name.clone()),`
- `src/domain/model/utils/game_rules.rs:78` — `state.event_record = fired.clone();`
- `src/domain/model/generation.rs:336` — `MessageKind::EventNarration(record) => Generation::Event(record.clone()),`
- `src/domain/model/tail.rs:58` — `self.current_state()?.event_record.as_ref()`
- `src/domain/model/state/event_record.rs:24` — `pub header_name: String,` (no reader in `src/`)

Snapshot on purpose? No comment links the two homes. `Generation::write` clears the
state's record for the new Swipe (`src/domain/model/generation.rs:140`
`event_record: None,`) and writes the record into `kind` instead, so the copies sit on
different rows.

How they differ. The state copy and the kind copy are written by different steps: the
state copy by the Game rules of the analysed Narration, the kind copy by the event
generation. If the event generation fails after the state was saved, the story holds a
Narration whose state names a fired Trigger, and no event Message. `retrigger_allowed`
(`src/domain/model/tail.rs:61`) reads the state copy and offers the Retrigger button;
`redo` (`:336`) reads the kind copy for the same Trigger when the last Message is the
event Narration. Deleting the event Narration then Retriggering repeats the event, as
story-06 already reports; the two homes are why the check cannot see it.

## 5. `maker` is written on every Swipe and read by nothing, and it is wrong for Impersonate

- `src/domain/model/message.rs:16` — `pub enum Maker {`
- `src/domain/model/message.rs:21` — `/// The player.` (variant `Player`)
- `src/domain/model/generation.rs:106` — `Generation::Impersonate { direction } => NewSwipe {`
- `src/domain/model/generation.rs:110` — `..Swipe::new(text, Maker::Player, Self::without_context(start))`
- `src/adapters/driven/storage/transaction.rs:198` — `serde_json::to_string(&swipe.maker)?,`
- `src/adapters/driven/storage/mappers/message.rs:40` — `maker: serde_json::from_str(&db_swipe.maker_json)?,`

Read sites in production code: none. A search for `.maker` and `Maker::` over `src/`
finds the definition, the four write sites in `generation.rs` and `opening.rs`, and the
storage round trip. No branch reads it.

Snapshot on purpose? No comment says so. `src/domain/model/generation.rs:90` calls
`write` "The one home of the Message kind, type, maker, stored inputs and event
header", so `maker` is meant to be authoritative.

How they differ. The player sends `/impersonate`. The narrator writes the Persona's
text, and the Swipe stores `message_type: Input`, `kind: ImpersonateInput`,
`impersonated: true` and `maker: Player`. `maker` says the player typed it. Four fields
now answer "who speaks" and one of them is wrong. Nothing reads it today, so the
disagreement stays hidden until a reader is added.

## 6. An Options failure has its own home, and the status rule cannot see it

- `src/application/generation/gate.rs:828` — `pub fn save_failure(&self, failure: GenerationFailure) -> Result<(), JobFailure> {`
- `src/application/generation/gate.rs:845` — `tx.put_options_failure(&OptionsFailure { reason, raw })`
- `src/application/pipeline/job.rs:359` — `Ok(None) => OptionsResult::Unavailable {`
- `src/application/games/view_query.rs:206` — `.and_then(|game| game.failure)`
- `src/adapters/driven/storage/transaction.rs:307` — `pub fn put_options_failure(&mut self, failure: &OptionsFailure) -> Result<(), EngineError> {`

Snapshot on purpose? Partly. `src/application/generation/gate.rs:836` says "A failed
run keeps the offered set", which is why the dock owns the notice. No comment says why
an Options failure must stay out of `games.failure`.

How they differ. The Options call fails at the end of a Job. The dock stores the same
kind of line in `game_docks.notice`; `games.failure` is not written. `status` reads
`game.failure` only, so the status display reads "Ready" while the dock shows "Options
generation failed" and the disclosure holds the raw error. The same concept, "the last
run failed", has two homes, two readers and two answers on one screen. A Job-level
failure of any other step takes the other home and does show in the status display.

## 7. The text-check mode value is spelled four ways, and one way silently means Disabled

- `src/domain/model/settings.rs:15` — `pub enum TextCheckMode {` (no `as_str`, no `FromStr`)
- `src/adapters/driving/http/settings/templates/settings.rs:352` — `TextCheckMode::SpellGrammar => "spell_grammar",`
- `src/adapters/driving/http/settings/templates/settings.rs:324` — `<option value="spell_grammar" {% if text_check_mode == "spell_grammar" %}selected{% endif %}>Spell + Grammar</option>`
- `src/adapters/driving/http/settings/handlers/settings.rs:189` — `"spell_grammar" => TextCheckMode::SpellGrammar,`
- `src/adapters/driving/http/settings/handlers/settings.rs:190` — `_ => TextCheckMode::Disabled,`
- `data/schemas/settings.schema.json:100` — `"enum": ["Disabled", "Spell", "Grammar", "SpellGrammar"],`

Snapshot on purpose? No. Two hand-written lists must agree, and the parse direction has
a catch-all.

How they differ. Rename the option value in the template to `spell-grammar`, or add a
fifth mode to the enum without the template. The form still submits, the handler falls
to `_ => TextCheckMode::Disabled`, and the settings save succeeds. The player picks
"Spell + Grammar", the save answers 200 with no text, and the Text Check card comes back
with "Disabled" and the auto-check box greyed out. Nothing reports a bad value
(`src/adapters/driving/http/settings/handlers/settings.rs:186`
`let mode = match form.check_mode.as_str() {`). The schema's own enum lists the serde
names, which no form ever sends, so a schema check cannot catch the drift either.

## 8. The status display's machine states and its idle label are written in Rust and in JS

- `src/adapters/driving/http/layout/handlers/endpoints.rs:66` — `status_element("idle", "ready", "Ready")`
- `src/adapters/driving/http/layout/handlers/endpoints.rs:88` — `Busy::Job(step) => status_element("thinking", "thinking", step_label(step)),`
- `src/adapters/driving/http/layout/handlers/endpoints.rs:95` — `status_element("autoplay", "thinking autoplay", &label)`
- `assets/dashboard-action-area.js:41` — `statusDisplay.querySelector('[data-status="' + state + '"]')`
- `assets/dashboard-action-area.js:46` — `return statusShows("thinking");`
- `assets/dashboard-action-area.js:110` — `let announcedStatusLabel = "Ready";`

Snapshot on purpose? The JS has no comment for the three state names. `:108` explains
why the label starts as "Ready", which makes it a copy of the server's idle label by
construction.

How they differ. Add a step or rename one state, for example `data-status="quantifying"`.
`deriveActionState` still asks for "thinking" and "autoplay", so it reads the new state
as neither: the action area treats a running Job as idle, the command input stays
enabled, and the player's next Action is refused with "Still thinking...".
The label copy drifts the other way: if the server's idle label becomes "Idle", the
first load announces "Ready" as a change the player did not cause.

## 9. The provider list is written in five places

- `src/domain/model/llm_backend.rs:34` — `pub const ALL: [Self; 4] = [Self::OpenRouter, Self::Mock, Self::Ollama, Self::OpenCodeGo];`
- `src/domain/model/llm_backend.rs:38` — `pub const SELECTABLE: [Self; 3] = [Self::OpenRouter, Self::OpenCodeGo, Self::Ollama];`
- `src/domain/model/llm_backend.rs:44` — `Self::OpenRouter => "openrouter",`
- `src/domain/model/llm_backend.rs:54` — `Self::OpenRouter => "OpenRouter",`
- `data/schemas/settings.schema.json:23` — `"enum": ["OpenRouter", "Mock", "Ollama", "OpenCodeGo"],`

Snapshot on purpose? No. `:36` explains why `Mock` is not selectable, which is a
deliberate difference in one list. The schema enum is a hand copy of the serde names.

How they differ. Add a provider, for example a local `LlamaCpp`. The match arms in
`wire_id` and `display_label` fail to compile, which is the guard. The schema enum does
not: `data/settings.json` with `"provider": "LlamaCpp"` fails `validate_data.py`, so the
build stops for a Connection the engine can read. The reverse order is worse: a schema
entry with no Rust variant passes validation, and the settings read then fails in the
serde deserialize of `LlmBackendType`, so no settings row loads.

## 10. The default preset ids are written in Rust, in the seeds, in a migration and in settings.json

- `src/domain/model/utils/settings_defaults.rs:7` — `pub const DEFAULT_SYSTEM_PROMPT_PRESET_ID: &str = "system_default";`
- `src/domain/model/utils/settings_defaults.rs:9` — `pub const DEFAULT_QUANTIFIER_PROMPT_PRESET_ID: &str = "quantifier_default";`
- `data/prompt_presets/system/default.json:2` — `"id": "system_default",`
- `src/adapters/driven/storage/utils/plumbing.rs:392` —
  `"UPDATE settings SET mode_preset_registry = '{\"novel\":{\"system_prompt_preset_id\":\"system_default\",...`
- `data/settings.json:89` — `"system_prompt_preset_id": "system_default",`
- read: `src/application/prompt_preset_service.rs:69` — `.get_preset(id)?` then
  `.ok_or_else(|| EngineError::PresetNotFound(id.to_string()))`

Snapshot on purpose? The migration literal is a historical record, which is fair. The
seed id and the Rust constant are two live homes with no check between them:
`src/bootstrap/run.rs:237` reads the id from the seed and never compares it to the
constant.

How they differ. Rename the seed id to `system_default_v2` and keep the constant. A
fresh database seeds `system_default_v2`, and `mode_preset_registry` still names
`system_default`. Every new Game copies `system_default` into its row, and the first
generation fails with `PresetNotFound`, which the Job shows as a missing-data failure.
The dashboard marks no preset active for the Novel system slot at the same time.

## 11. The default Agents are written in Rust and in `data/settings.json`

- `src/domain/model/agent.rs:115` — `pub fn defaults() -> Vec<Self> {`
- `src/domain/model/agent.rs:118` — `name: "quantifier".to_string(),`
- `data/settings.json:74` — `"agents": [`
- `data/settings.json:76` — `"name": "quantifier",`

Snapshot on purpose? Yes for the seed: the data file fills the row once. `AgentConfig::defaults`
is the serde default, so it fills the row when the key is absent.

How they differ. Add a third Agent to `data/settings.json` only. A fresh database seeded
from the file runs it. A database whose `agents` key is absent, or a deleted settings row
(`src/adapters/driven/storage/settings.rs:33` `Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings::default()),`),
falls back to the two Rust defaults. The same build then runs one or two Agents
depending on how the row was made.

## 12. Each small enum's wire string is written twice or three times

- `src/domain/model/prompt_preset.rs:24` — `pub fn as_str(&self) -> &'static str {`
- `src/domain/model/prompt_preset.rs:46` — `"system" => Ok(PresetType::System),`
- `src/domain/model/settings.rs:38` — `pub fn as_str(&self) -> &'static str {`
- `src/domain/model/settings.rs:77` — `"novel" => Ok(Self::Novel),`
- `src/domain/model/settings.rs:28` — `#[serde(rename_all = "snake_case")]`
- `src/domain/model/settings.rs:235` — `Self::Second => "second",`
- `src/domain/model/settings.rs:253` — `"second" => Ok(Self::Second),`

Snapshot on purpose? No. `as_str` is an exhaustive match, so a new variant fails to
compile there. `FromStr` and `TryFrom<&str>` are hand-written matches with a catch-all
`_ => Err(...)`, so a new variant compiles and is not accepted. The serde rename is a
third spelling, derived from the variant name.

How they differ. Add `NarratorMode::SecondPersonCyoa`. `as_str` fails to compile until
it gets an arm; `from_str` compiles as it is and returns `Unknown narrator mode`. The
world form's posture select then sends the new value, `parse_or_default`
(`src/domain/model/settings.rs:54`) logs a warning, and the World keeps Novel. The
player sees the select snap back with no message. No test ties `as_str` to `from_str`:
`src/domain/model/prompt_preset_tests.rs` checks each `as_str` value one by one and has
no round trip.

## 13. An Agent's name is stored, and the engine uses a literal instead

- `src/application/agents/quantifier/agent.rs:32` — `_config: &AgentConfig,`
- `src/application/agents/quantifier/agent.rs:37` — `name: "quantifier".to_string(),`
- `src/application/agents/runner.rs:41` — `match config.agent_type.as_str() {`
- `src/domain/model/agent.rs:118` — `name: "quantifier".to_string(),`
- `src/application/games/view_query.rs:260` — `.filter(|message| message.agent_name == role.agent_name())`

Snapshot on purpose? No. `_config` is discarded, so `AgentConfig.name` is inert. The
recorded name comes from the literal, which must equal `Role::agent_name()` for
`role_health` to find the record.

How they differ. The author edits the agent name in `data/settings.json` to "scorer", or
the Settings UI shows it. The stored name is "scorer"; every LLM record still says
"quantifier"; role health still works. Rename the literal instead and `role_health`
finds nothing, so the Quantifier cell reads "No calls yet" while its records exist under
"scorer".

## 14. The Autoplay limits are written in `Budget` and in the shell

- `src/application/autoplay/budget.rs:13` — `pub const DEFAULT: Budget = Budget(10);`
- `src/application/autoplay/budget.rs:14` — `pub const MAX: u32 = 100;`
- `src/application/autoplay/budget.rs:24` — `if cycles == 0 || cycles > Self::MAX {`
- `assets/index.html:154` — `value="10"`
- `assets/index.html:155` — `min="1"`
- `assets/index.html:156` — `max="100"`

Snapshot on purpose? No comment links them. `http-07` in the earlier report already
covers the empty field, which `Budget::parse` reads as 10.

How they differ. Raise `Budget::MAX` to 200. The server accepts 150; the shell refuses
it before the request with a browser tooltip, so the player cannot use the new limit and
the engine answers a `400` for a value it would accept. Lower `Budget::MAX` to 50 and the
form offers 51 to 100, which the server then refuses with "An autoplay run needs a budget
of 1 to 50 cycles".

## 15. A World's posture fields are in the Rust manifest and not in the schema

- `src/domain/model/world.rs:52` — `#[serde(default = "world_defaults::default_world_narrator_mode")]`
- `src/domain/model/world.rs:53` — `pub narrator_mode: NarratorMode,`
- `src/domain/model/world.rs:57` — `pub narrative_tense: NarrativeTense,`
- `data/schemas/world.schema.json:6` — `"properties": {`, whose keys are only
  `characters_dir, default_room_image, default_scenario_id, description, global_rules, id, map_file, name, scenarios`
- `data/schemas/world.schema.json` sets no `additionalProperties`, so an unknown key passes.

Snapshot on purpose? The schema documents the file the seed loader reads. No comment
says the posture keys are outside it.

How they differ. The author writes `"narrator_mode": "InteractiveFiction"` in
`world.json`. `validate_data.py` passes it, because the schema does not name the key.
The Rust `Deserialize` for `NarratorMode` renames to `snake_case`
(`src/domain/model/settings.rs:28`), so the parse fails and the World does not load. The
build is green and the World is unusable. The same hole hides a typo in a key that the
engine does read, such as `"global_rule"`.

---

## Also noted, weaker

- `"No active game"` is written twice: `src/adapters/driving/http/screen/handlers/parts.rs:52`
  `const NO_ACTIVE_GAME: &str = "No active game";` and
  `src/adapters/driving/http/games/templates/games.rs:69`
  `<div class="game-item"><span class="game-name">No active game</span></div>`. The
  header and the Games panel show different sentences if one is edited.
- `"Unknown"` as a Game display name is written twice:
  `src/application/games/view_query.rs:232` `None => Ok("Unknown".to_string()),` and
  `src/adapters/driving/http/app_state.rs:91` `"Unknown".to_string()`.
- The `data-saved` "Saved" line convention is applied to one of two status elements:
  `src/adapters/driving/http/settings/templates/settings.rs:312` carries
  `data-keep="data-saved"`, `src/adapters/driving/http/worlds/templates/worlds.rs:132`
  does not, and both are styled by `assets/styles.css:1003` `.saved-flag[data-saved]::before`.
  The Settings line survives a later load of its part; the World line does not.
- Eleven class names are rendered by Rust and match no rule in `assets/*.css` and no
  selector in `assets/*.js`: `btn-original`, `edit-form`, `games-page-next`,
  `games-page-previous`, `llm-message-open`, `new-game-form-dialog`, `new-game-section`,
  `options-regen-form`, `preview-cancel`, `subtab-marker-slot`, `text-check-form`,
  `view-form`. Harmless today; nothing warns when a class the JS selects is renamed.

## Dropped, and why

Twelve candidates were checked against the code and dropped, because a check already
holds them or because the copies cannot disagree:

1. Screen part names (`SCREEN_PARTS` against `assets/index.html`) — guarded by
   `tests/infrastructure/guardrails/screen.rs:147 check_shell_parts_match_the_server`
   and `:240 check_part_table_matches_shell`.
2. `data-part` names read by the JS — the same guardrail, plus the server answers `400`
   for an unknown name.
3. The Harper dictionary cache — keyed by the word list it was built from
   (`src/adapters/driven/text_check/harper_text_checker.rs:32`).
4. The slash command palette — already reported (http-03).
5. `SAVED_GAMES_PAGE_SIZE`, `STORY_WINDOW_SIZE`, `AGENT_HISTORY_WINDOW`,
   `LLM_MESSAGE_RETENTION_LIMIT` — one definition each, read from that definition.
6. The `worlds` column list — one `WORLD_COLUMNS` list that every statement derives from
   (`src/adapters/driven/storage/worlds.rs:399`).
7. The busy registry against the database — the registry is the only home, and
   `raise_busy_changes` derives the change from a before/after diff
   (`src/application/generation/gate.rs:200`).
8. The `games_page` the browser keeps in `sessionStorage` against the server's clamp —
   the clamp is idempotent and the player sees the same page.
9. `set_event_header` (`src/domain/model/message.rs:160`) — no caller, so no second
   writer; folded into row 4.
10. `AutoplayProgress::cycle()` — one derivation, `done + 1`.
11. `is_default` against the `DEFAULT_*_PRESET_ID` constants — a delete of a default is
    refused by the `mode_preset_registry.references(id)` check
    (`src/application/prompt_preset_service.rs:120`), so the two agree in effect.
12. `retrigger_allowed` against `show_retrigger` — one value, passed through
    (`src/adapters/driving/http/templates.rs:37`).

## Already known

Not repeated here: the 53 findings in `.scratch/invariant-audit/assets/REPORT.md`, and the
brief's own list — Character cards inside each Swipe state, World settings copied into a
Game at create, data files never re-read, the default host in `cli.rs` and `build.py`,
typed Input and the Quantifier, the Connection form's `ALL`/`SELECTABLE` parse, a
`finish_reason=length` Narration, HTTP status text as "could not be reached", and the
Quantifier mapping only map Rooms. `message_type` against `kind` is story-04 there, and
so is `active_swipe_index` (storage-03).
