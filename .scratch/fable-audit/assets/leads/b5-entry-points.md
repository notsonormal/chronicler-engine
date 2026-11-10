# B5 — One business rule, many entry points

Lens: every request kind that reaches a Job or a story write, and every record kind an
author can write, compared across the paths that reach it.

Method: read `CONTEXT.md` for the terms, then `docs/specs/` by file name,
`src/adapters/driving/http/builders/router.rs` for the route set, then the handlers,
the gate, the Job, the storage writers, `data/schemas/`, `scripts/validate_data.py` and
`build.py`'s data step. No build, no server, no cargo. Read-only.

Labels: **known** = I read the code that shows the cell. **inferred** = the code shows it
but through one indirection I did not execute. **guessed** = I did not confirm it.

`/swipe/new` is the Retry route; the dashboard button carries `title="Retry"`
(`templates.rs:25`). So "a new Swipe", "Retry" and "redoing the last Message" are one
row below, and "previous/next Swipe" is the switch route.

---

# Table 1 — the story paths

Baseline row is the typed Action. A cell marked **·** equals the baseline; a cell marked
**≠** differs and is quoted in the evidence section under the row's number.

| # | Path (route) | Trim / sanitize | Empty text refused | Text check | Room check | Quantifier | Triggers | Options Agent | `X-Chronicler-Game` | Last-Message id | On model failure |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | Typed Action, Enter (`POST /action/check`) | yes, `trim` | no → Continue | yes (if enabled) | yes | yes, after the text save | yes | after narration, if `options_always_on` | yes | no | failure line on the Game, status shows it |
| 2 | Typed Action, Send (`POST /action` from the preview) | · | · | **≠** none | · | · | · | · | · | · | · |
| 3 | Empty Action / Continue (`command=""`) | · | · | · | · | · | · | · | · | · | · |
| 4 | `/guide` | direction is `trim`med | no (`Guide("")` allowed) | **≠** none | · | · | · | · | · | · | · |
| 5 | `/impersonate` | direction is `trim`med | no (`Impersonate(None)`) | **≠** none | · | **≠** yes, on the Impersonate Input | · | · | · | · | · |
| 6 | `/options` | n/a | n/a | **≠** none | **≠** none | **≠** none | **≠** none | **≠** yes, ungated | · | · | an Options failure in the dock, **≠** not a Game failure line |
| 7 | Retry (`POST /swipe/new`) | no text enters | n/a | **≠** none | · | **≠** except for a typed Input | · | · | · | **≠** yes | · |
| 8 | Retrigger (`POST /retrigger`) | n/a | n/a | **≠** none | · | · | **≠** none (event) | · | · | **≠** yes | **≠** "The event narration failed." |
| 9 | Previous/next Swipe (`POST /message/:id/swipe/:i`) | n/a | n/a | **≠** none | **≠** none | **≠** none | **≠** none | **≠** none | · | **≠** yes | no model call |
| 10 | Edit of a Message (`POST /history/:id`) | **≠** none | **≠** no, empty stored | **≠** none | **≠** none | **≠** none | **≠** none | **≠** none | · | **≠** no check at all | no model call |
| 11 | Delete of the last Message (`POST /history/delete`) | n/a | n/a | **≠** none | **≠** none | **≠** none | **≠** none | **≠** none | · | **≠** yes | no model call |
| 12 | An Autoplay Cycle (`POST /autoplay`, then the loop) | n/a | n/a (Continue is `Action("")`) | **≠** none | · | · | · | · | · | **≠** no (the loop decides) | **≠** the whole run ends |
| 13 | Opening Narration of a new Game (`POST /games`) | n/a | n/a | **≠** none | **≠** none | **≠** none | **≠** none | **≠** none | **≠** not a game-bound route | n/a | n/a, no model call |

## Table 1 evidence — the baseline row, then every cell that differs

**Row 1 — the baseline, cell by cell.** Trim: `actions.rs:46` `let command =
form.command.trim().to_string();`. Empty text: `generation.rs:262`. Text check:
`actions.rs:73`. Room check: `job.rs:185` `self.run_step(step.clone(), Some(kind), || self.room_check(order))?;`.
Quantifier then Game rules after the text save: `job.rs:201-206`. Triggers: `job.rs:205`
`self.update_state(order, saved, &quantified)` reaching `game_rules.rs:73-77`. Options:
`job.rs:337`. Game header: `router.rs:62` `route_layer(middleware::from_fn_with_state(app_state, refuse_stale_game))`
and `request_layers.rs:44` `if let Some(named) = request.headers().get(REQUEST_GAME_HEADER) {`.
No last-Message id: `generation.rs:261-263` is the whole `JobRequest::Action` arm. Failure
line: `job.rs:388-390` `Some(JobFailure::Line(line)) => { … self.writer.save_failure(line)`
into `gate.rs:828`, with the words of `generation_status.rs:24-45`, for example
`generation_status.rs:27` `Self::Unreachable => "The language model could not be reached.".to_string(),`.
[known]

**Row 2 — Send has no text check.** The preview renders two forms, and the Send button
posts to a route that dispatches directly:
`templates.rs:98` `<form method="post" hx-post="/action" hx-swap="none" hx-on::before-request="onPreviewBeforeRequest()" …>` (the
"Send Original" form; "Send with edits" at `templates.rs:102` uses the same route).
Only `/action/check` reads the settings and the checker
(`actions.rs:69` `if !settings.text_check.effective_enable_auto_check() {`,
`actions.rs:73` `state.text_check_service().check_player_input(`). [known]

**Row 3 — the empty Action writes no Input Message.** `generation.rs:262`
`let input = (!text.is_empty()).then(|| text.clone());` and `job.rs:187`
`if let Some(text) = input {`. A non-empty Action writes an Input Message and a
Narration; a Continue writes only the Narration. Stated: `generation.rs:148`
`/// Free text the player typed. Empty text is a Continue.` [known]

**Row 4 — `/guide` never reaches the text check.** `actions.rs:57`
`if Action::parse(&command).is_engine_command() {` returns the dispatch at
`actions.rs:58`. [known]

**Row 5 — `/impersonate` never reaches the text check, but its Input is analysed.**
Same bypass (`actions.rs:57`). The analysis differs from a typed Input because only
`Generation::InputRetry` is exempt: `generation.rs:86-87`
`pub fn is_analysed(&self) -> bool { !matches!(self.generation, Generation::InputRetry) }`.
[known]

**Row 6 — `/options` has no generation, so no room check, no Quantifier and no
Trigger.** `generation.rs:202` `/// The first generation. `None` for `/options`.`, and
`generation.rs:240-244` builds `Self { input: None, first: None, options:
OptionsRun::OnDemand }`. It is also the one Options run the toggle does not gate:
`job.rs:337` `if run == OptionsRun::AfterNarration && !self.options_enabled()? {`.
Its failure is a dock record, not a Game failure line: `gate.rs:844`
`OptionsResult::Unavailable { reason, raw } => Self::in_tx(opener, self.game, |tx| {`
followed by `gate.rs:845` `tx.put_options_failure(&OptionsFailure { reason, raw })`.
[known]

**Row 7 — Retry names the last Message, skips the text check and skips the analysis
for a typed Input.** No check call sits on this path: `chat_window.rs:50`
`Some(message) => start_job(&state, game_id, JobRequest::Retry(message)),`.
`generation.rs:248` `Self::require_last(&tail, *named)?;` and `generation.rs:283`
`if tail.last.id == named {`. The analysis exemption is `generation.rs:86-87`
(quoted above), reached from `job.rs:197` `if !order.is_analysed() {`. [known]

**Row 8 — Retrigger runs the Quantifier but no Trigger, and its failure line is its
own.** No check call: `chat_window.rs:39`
`Some(message) => start_job(&state, game_id, JobRequest::Retrigger(message)),`.
`generation.rs:255` `let record = tail.retrigger_record().ok_or_else(bad_request)?.clone();`
(the last-Message check is `generation.rs:254` `Self::require_last(&tail, *named)?;`).
The Trigger and movement skip: `game_rules.rs:49` `let is_event = kind == Kind::Event;`
and `game_rules.rs:73-77` `let fired = if is_event { None } else { fired_trigger(&state, world) };`.
The failure line: `generation_status.rs:42`
`Self::EventNarrationFailed => "The event narration failed.".to_string(),` [known]

**Row 9 — a Swipe switch is a write with no model call, and it clears the failure line
and the dock.** No check and no generation: `chat_window.rs:67-69`
`state.game_writer.switch_swipe(game_id, message_id, swipe_index)`. `gate.rs:527`
`Self::clear_failure_and_put_dock(tx, &DockRecord::default())`. The last-Message check is
`gate.rs:512-514` `if last.last.id != message { return Err(NotAccepted::LastMessageChanged); }`.
[known]

**Row 10 — an edit checks nothing about which Message it names, and clears nothing.**
`gate.rs:536-542`: `let at = match self.storage.stories().active_swipe(game, message) {`
… `opener.open(game, |tx| tx.set_swipe_text(at, text))`. There is no `require_last` and no
`clear_failure_and_put_dock`, unlike rows 9 and 11. Any Message id of the Game is
editable, and the stored text is whatever the form sent: `history.rs:26`
`match state.game_writer.edit(game_id, id, &form.text) {` (no `trim`, no empty refusal).
[known]

**Row 11 — a delete requires the last Message and more than one Message.** No check
and no generation: `history.rs:40` `state.game_writer.delete_last(game_id, message)`.
`gate.rs:553-555` `if last.last.id != named { return Err(NotAccepted::LastMessageChanged); }`
and `gate.rs:556-558` `if !last.can_delete() { … "The story keeps its last message"`.
It also clears the failure line and the dock: `gate.rs:564`
`Self::clear_failure_and_put_dock(tx, &DockRecord::default())`. [known]

**Row 12 — an Autoplay Cycle takes its request from the loop, not from a form, and a
failed Job ends the run.** The start handler goes straight to the run:
`autoplay.rs:30` `AutoplayRun::start(&state.game_writer, state.job_deps.clone(), game_id, budget)`.
`driver.rs:119`
`Decision::Next(progress, JobRequest::next_in_autoplay(story.tail.as_ref()))`;
`generation.rs:170` `Some(tail) if tail.awaits_narration() => Self::Action(String::new()),`;
`driver.rs:102` `JobEnd::Failed => return RunEnd::JobFailed,`. No text check exists on this
path because no player text exists on it. [known for the absence; the "because" is
inferred]

**Row 13 — the opening Narration is made by the World, with no Job at all.** The create
calls the builder and then writes: `gate.rs:571` `let opening = self.opening_message(new)?;`.
`opening.rs:17-18`
`let text = scenario.map(|scenario| TemplateVars::new(persona_name).render(&scenario.text)).unwrap_or_default();`
and `opening.rs:20` `let swipe = Swipe::new(text, Maker::WorldOpening, state);`.
The create writes that Swipe directly: `gate.rs:603-604` `let message_id =
tx.insert_message(opening)?;` / `tx.insert_swipe(…, &opening.swipes[0])?;`. The create
route is outside `game_bound_routes` (`router.rs:80` `.route("/games",
post(games::handlers::create_game_handler))`, while the header layer sits at
`router.rs:62`). [known]

## Table 1 difference list, and whether the difference is recorded as intended

| # | Difference | Recorded? |
|---|---|---|
| D1 | The text check runs on `/action/check` only; Retry, Retrigger, edit, the Swipe switch, the opening and Autoplay never check | **yes** — `docs/specs/actions.md:401` "The confirm path is the preview's send: it dispatches the checked command." and `actions.rs:48-49` "True for slash commands the engine itself handles. These bypass the / player-input text check" |
| D2 | Engine commands (`/guide`, `/impersonate`, `/options`) skip the check | **yes** — `docs/specs/actions.md:365` "Scenario 1.12: Recognized engine commands bypass the player-input text check" |
| D3 | A text-check fault is swallowed and the request is dispatched anyway; the player sees no line | **no reason recorded** — `actions.rs:64` `tracing::error!("Failed to read settings for text check: {e}");` and `actions.rs:80` `tracing::error!("Text check failed: {e}");` both fall through to `actions.rs:65` / `actions.rs:81` `return dispatch_action(&state, game_id, command);` |
| D4 | A Retry of a typed Input skips the Quantifier; every other Retry runs it | **yes** — `docs/specs/swipe_new.md:321` "And the player has not moved (the quantifier did not re-run)" and `generation.rs:84-85` "The Retry of a typed Input ends after its text save: the Quantifier and the Game rules / read a Narration or an Impersonate Input, not a typed Input again." |
| D5 | The opening Narration runs no Quantifier, no Trigger and no room check, but its **Retry** runs all three | **no reason recorded** — the only comment is about the guide: `generation.rs:329` "The opening re-applies nothing the player asked for." `redo` maps the opening to a plain `Narration` (`generation.rs:330-332`), which `is_analysed` accepts, so `job.rs:201-206` runs `analyse` and `update_state` on it |
| D6 | Edit names any Message; Retry, Retrigger, the Swipe switch and delete refuse anything but the last Message | **no reason recorded** — `docs/specs/story_log.md:196` covers the delete case ("Scenario 8.17: A delete that names a Message that is not the last one is refused") and the spec has no matching edit scenario (`docs/specs/story_log.md:172` "Scenario 8.15: An edit answers nothing and the story log shows the new text" uses the last Message only) |
| D7 | Edit leaves the Game's failure line and the offered Options in place; the Swipe switch and delete clear both | **no reason recorded** — `gate.rs:542` `opener.open(game, \|tx\| tx.set_swipe_text(at, text))` against `gate.rs:527` `Self::clear_failure_and_put_dock(tx, &DockRecord::default())` |
| D8 | Edit stores text verbatim, with no trim and no empty refusal; the Action routes trim | **partly** — the empty case is already known (llm-04). The trim difference has no recorded reason |
| D9 | Options after a Narration is gated by `options_always_on`; `/options` is not | **yes** — `generation.rs:184-193` "Whether the run happens at the end of the Job. `OnDemand` always runs." and `job.rs:335-336` "The toggle decides the run that follows a Narration; `/options` asks for / the run itself." |
| D10 | A model failure ends the whole Autoplay run but only the Job on the other paths | **yes** — `docs/specs/autoplay.md:19` "- A Job of the run that ends with a failure line." and `docs/specs/autoplay.md:63` "Scenario 41.4: A failed narrator call ends the run and its failure line stays" |
| D11 | `/action` refuses a busy Game before anything else; `/action/check` runs the text check first and answers a `200` preview | **no reason recorded**, and it contradicts the gate's own rule: `gate.rs:333` "The busy refusal comes before any other check of the request." |
| D12 | The create path is outside the Game-header check and the room check | **already known** (world-02) — the header difference is not stated anywhere, and I found no spec sentence about it |
| D13 | An Options Agent failure is the only Agent failure that never becomes the Game's failure line: it answers `200` and writes a dock notice | **yes** — `docs/specs/autoplay.md:75` "Scenario 41.5: A failed Options run writes its Options failure and the run goes on"; the Narrator and Quantifier failures go to `gate.rs:828` `pub fn save_failure(failure: GenerationFailure)` instead |

---

# Table 2 — the author paths

Cell legend: `—` means the path cannot write that record at all.

| # | Record / path | Name uniqueness | id / key format | Cross references | JSON schema | Non-empty fields | Numeric ranges |
|---|---|---|---|---|---|---|---|
| 1 | World + map, boot seed `data/worlds/*/world.json` | none (key only, upsert) | none | none at boot | not run at boot | none | none |
| 2 | World + map, dashboard create `POST /worlds` | none | none (empty or `/`-bearing key stored) | none | none (serde only) | none | none |
| 3 | World + map, dashboard edit `POST /worlds/:key` | none | key is `readonly` | `default_scenario_id` filtered | none (serde only) | none | none |
| 4 | World + map, `scripts/validate_data.py` | none | none | starting Room, Scenario Characters, Trigger Rooms, image assets | yes | `required` lists only | `minimum: 1` on token fields |
| 5 | Scenario, boot seed (inside `world.json`) | none | none | Characters checked by the script only | `world.schema.json:51` | none | none |
| 6 | Scenario, dashboard (`scenarios_json`) | none | none | none | none | **serde defaults**, so `text` and `starting_room_id` may be absent | none |
| 7 | Character, boot seed `data/characters/**` | none (duplicate id: first wins) | none | none | not run at boot | none | none |
| 8 | Persona, boot seed `data/personas/*.json` | none | the file stem is the key; the JSON has no key field | none | validated against the **Character** schema | none | none |
| 9 | Prompt preset, boot seed `data/prompt_presets/<type>/*.json` | none | `id` defaults to `"default"` | none | no preset schema exists; the script does not read these files | `name` defaults to `"Default"` | `validate()`, but a refusal only skips the file with a warning |
| 10 | Prompt preset, dashboard create `POST /prompt-presets` | yes | generated | none | none | none | `validate()` refuses, `400` |
| 11 | Prompt preset, dashboard edit `POST /prompt-presets/:id` | yes | n/a | n/a | none | none | `validate()`, `400`; `parameters` cannot change |
| 12 | Prompt preset, duplicate `POST /prompt-presets/:id/duplicate` | yes | generated | n/a | none | none | `validate()`, `400` |
| 13 | Connection, boot seed `data/settings.json` | **none** | **none** | slot ids checked | yes | none | `minimum: 1` |
| 14 | Connection, dashboard add/edit | yes | generated | the delete route checks the slots | none | none | `optional_token_count` |

## Table 2 evidence

**Row 1 vs row 4 — the boot seed reads no schema and checks no reference.** `load.rs:123`
`storage.seed_world(&world_card, &map)?` is reached with whatever serde accepted;
`load.rs:115` `let world_with_map = storage.get_world(&world_key)?;` never failed a
reference. The script is the only place the references are known:
`validate_data.py:132` `if starting_room_id not in valid_room_ids:`,
`validate_data.py:150` `for scenario_id, character_id, character_file in
missing_scenario_characters(`, `validate_data.py:168`
`if room_id and room_id not in valid_room_ids:`. [known]

**Row 1 vs row 2 — a repeated key: the seed replaces, the dashboard refuses.**
`worlds.rs:148-149` `pub fn seed_world(… WorldWriteMode::Upsert)` against
`worlds.rs:157-158` `pub fn create_world(… WorldWriteMode::Insert)`, with the refusal at
`worlds.rs:239-241` `WorldWriteMode::Insert if Self::is_unique_violation(&e) =>
EngineError::WorldAlreadyExists(world_card.key.clone()),`. Stated: `world_catalogue.rs:36-38`
"Refuses a key that already exists — the user-facing create path must never silently
replace an authored world. Bootstrap seeding uses the storage `seed_world` upsert
instead." [known]

**Row 2 — the map and the scenarios are parse-checked only.** `worlds.rs:50`
`serde_json::from_str(&self.map_json).map_err(|e| format!("Invalid map JSON: {e}"))` and
`worlds.rs:54` for the scenarios. Nothing else runs before
`worlds.rs:136` `match state.world_catalogue.create_world(world_card, map) {`. [known]

**Row 3 — the edit re-runs the same parse-only rule and patches fields in place.**
`worlds.rs:182` `let map = match form.parse_map() {` and `worlds.rs:186` `let scenarios =
match form.parse_scenarios() {` are the whole validation of the edit; the card is then
patched (`worlds.rs:196` `world_card.name = form.name;`) and written by
`worlds.rs:212-215` `state.world_catalogue.update_world(world_id, world_card, map)`. The
edit also differs from the create in what it keeps: `worlds.rs:191-193`
`world_card.default_scenario_id = world_card.default_scenario_id.filter(|id|
scenarios.iter().any(|s| &s.id == id));`. [known]

**Rows 4-6 — `validate_data.py` requires fields the engine does not.**
`world.schema.json:51` `"required": ["id", "name", "description", "starting_room_id", "text"]`
against `scenario.rs:13-16`
`#[serde(default = "scenario_defaults::default_starting_room"])` / `pub starting_room_id: String,` /
`#[serde(default)]` / `pub text: String,`. So the validator fails a Scenario that both
write paths accept, and an accepted Scenario can hold an empty opening text. [known]

**Row 1 — a map Room without `exits` passes the script and stops the seed.**
`map.schema.json:42` `"required": ["id", "name", "description"]` (no `exits`) against
`map.rs:42` `pub exits: HashMap<Direction, String>,` (no `serde(default)`). The map read
is `load.rs:122` `let map: MapDef = read_json_file(&map_path)?;`, and that error leaves
`process_world_dir`, so `run.rs:58-60` `if let Err(e) = seed_game_data(&storage,
&data_dir) { tracing::warn!("Failed to seed game data: {e}"); }` logs one warning and
**every World after the failing one is not seeded**. [known]

**Row 7 — a Character file the script passes can be dropped silently.**
`character.schema.json:57` `"required": ["name", "description", "personality", "scenario"]`
(no `id`) against `character.rs:61` `pub id: String,` (no serde default). The drop:
`load.rs:155-157` `Err(e) => { tracing::warn!("Failed to parse Character {}: {e}", path.display()) }`.
A duplicate `id` is worse: `load.rs:136-140` reads `existing_chars` **before** the loop, and
`characters.rs:63` `INSERT OR IGNORE INTO characters` then keeps whichever file the
directory listing reached first. [known]

**Row 8 — a Persona's own key field is discarded, and the file stem becomes the key.**
`load.rs:169-176`: `let key = path.file_stem()…` then `load.rs:179`
`storage.seed_persona(&key, &persona)?;`, while `personas.rs:64-66` bind `key` (`rusqlite::params![` … `key,`).
`PersonaCard::key` exists (`character.rs:52` `pub key: String,`) and no production code
reads it. The script validates personas against the Character schema
(`validate_data.py:87` `errors += validate_group(data_dir.glob("personas/*.json"),
char_schema, "personas")`), so a persona may carry `triggers` and `relationships`, which
`PersonaCard` then drops. [known]

**Row 9 vs rows 10-12 — a seed refusal is a warning, a form refusal is a `400`.**
`run.rs:254-256` `if let Err(refusal) = preset.validate() {
tracing::warn!("Skipping prompt preset seed {}: {refusal}", path.display());` against
`prompt_preset_service.rs:57-59` `preset.validate().map_err(ApplicationError::from_engine_error)?`.
A skipped seed leaves no preset row, so a Game or a mode bundle that names that id then
fails every call with the missing-record line. [known for the skip; the later failure is
inferred from `prompt_preset_service.rs:67-72` `chosen_preset` returning
`EngineError::PresetNotFound`]

**Row 9 — a seed may set the generation parameters; no dashboard path can.**
`run.rs:230` `let Some(parameters) = seed_field(&seed, "parameters", path) else {` and
`run.rs:233` for `option_count`, against `prompt_presets.rs:156` `parameters:
Default::default(),` / `prompt_presets.rs:157` `option_count: None,` in `into_preset`, and
`prompt_preset_service.rs:96-103`, which copies name, role, instructions, writing style,
output format and modes but never `parameters`. `call_plan` reads them:
`job.rs:477` `let max_tokens = connection.max_tokens_for(preset.parameters.max_tokens);`.
[known]

**Row 9 — a seed's id and display name have fallbacks; the forms always supply both.**
`run.rs:237` `let id = seed["id"].as_str().unwrap_or("default").to_string();` and
`run.rs:240` `name: seed["name"].as_str().unwrap_or("Default").to_string(),` and
`run.rs:246-249` `is_default: seed.get("is_default").and_then(|v| v.as_bool()).unwrap_or(true),`.
A seed file that mentions no id becomes the default preset with the id `default`. [known]

**Rows 10-12 — the three dashboard preset writes share one name rule.**
`prompt_presets.rs:181` `if let Err(e) = app_state.prompt_preset_service.save_preset(&preset)`
(add), `prompt_presets.rs:219` `edit_preset(&id, edit)` (edit), `prompt_presets.rs:257`
`save_preset(&copy)` (duplicate), each reaching `presets.rs:114`
`Self::ensure_preset_name_available(backend, preset)?;`. [known]

**Row 13 — `data/settings.json` seeds duplicate connection ids and names unchecked.**
`settings.schema.json:74` `"required": ["id", "name", "provider", "model"]` has no
`uniqueItems` and the script adds no rule (`validate_data.py:96` `valid_connection_ids =
{c["id"] for c in settings_data.get("connections", [])}` builds a set, so a duplicate id
collapses silently and satisfies the slot check). `settings.rs:604-605` `pub fn
find_connection(&self, id: &str) -> Option<&LlmConnection> { self.connections.iter().find(|c| c.id == id)`
then resolves the slot to the first match. The dashboard refuses a repeated name:
`settings_service.rs:36-37` "Refuses a name already used by another connection; two cards
the user cannot tell apart must not exist." [known]

**Row 13 vs row 14 — one provider field, two spellings that no path shares.**
The file and the schema use the serde names: `settings.schema.json:23`
`"enum": ["OpenRouter", "Mock", "Ollama", "OpenCodeGo"],`, and `llm_backend.rs:12-18`
declares those variant names with no `rename_all`. The forms use the wire ids:
`view_models.rs:55` `value: provider.wire_id().to_string(),` with `llm_backend.rs:43-44`
`Self::OpenRouter => "openrouter",`, matched by `llm_backend.rs:28`
`.find(|provider| provider.wire_id() == s)`. So `"openrouter"` is refused by the schema and
by the boot, and `"OpenRouter"` is refused by the form with
`llm_backend.rs:29` `EngineError::Config(format!("Unknown LLM backend '{s}'"))`. [known]

---

## Validations one path runs and the other skips

1. **JSON schema.** `validate_data.py:85-88` is the complete set of groups it validates
   (settings, `worlds/*/world.json`, `worlds/*/map.json`, `personas/*.json`,
   `characters/**/*.json`). Prompt presets are in no group and have no schema file
   (`data/schemas/` holds `world`, `map`, `character`, `settings` only), and the boot
   runs no schema at all.
2. **A Room's `exits`.** Required by Rust (`map.rs:42`), not by `map.schema.json:42`, and
   refused by the World form (`worlds.rs:50`). The script passes the file that will stop
   the seed.
3. **A Character's `id`.** Required by Rust (`character.rs:61`), not by
   `character.schema.json:57`. The script passes the file that the boot drops.
4. **A Scenario's `text` and `starting_room_id`.** Required by
   `world.schema.json:51`, optional in Rust (`scenario.rs:13-16`, defaults `"start"` and
   `""`), so both write paths accept a Scenario the validator refuses.
5. **Cross references.** Only the script checks the starting Room (`:132`), the Scenario
   Characters (`:150`) and the Trigger Rooms (`:168`); no write path does. Image paths are
   checked only by `check_asset` (`:54-60`), so the World form stores a broken
   `default_room_image` and a broken Room `image_path`.
6. **Name uniqueness.** Only the dashboard and the storage services check it, and only for
   Connections (`settings_service.rs:36-37`) and presets (`presets.rs:114`). A World name,
   a Character id and a Persona key have no rule on any path.
7. **Numeric ranges.** `optional_token_count` refuses `0` and non-numbers on the
   Connection form (`settings.rs:66-68`), and `settings.schema.json` carries
   `"minimum": 1` for the same fields. Nothing checks a token field written into a
   preset seed, and `validate()` only refuses an Options preset with `option_count: 0`
   (`prompt_preset.rs:111-114`).
8. **`default_scenario_id`.** The script obeys it (`validate_data.py:126-128` `next((s["starting_room_id"]
   for s in scenarios if s["id"] == default_id), None)`), the engine ignores it
   (`world.rs:69-71` `pub fn default_scenario(&self) -> Option<&StartingScenario> {
   self.scenarios.first() }`), and the World form drops it on create
   (`worlds.rs:79` `default_scenario_id: None,)`). Adjacent to the reported world-01, but a
   third implementation of the same rule with the opposite reading.
9. **Duplicate ids.** No path checks them for Characters (`INSERT OR IGNORE`,
   `characters.rs:63`), Personas (`INSERT OR IGNORE`, `personas.rs:62`) or Connections
   (`find_connection` takes the first). Only the World key has a `UNIQUE` refusal.

---

## Already known, not counted

Character cards stored inside each Swipe state; World settings copied into a Game at
creation; data files never re-read into an existing database; the default host duplicated
in `src/utils/cli.rs` and `build.py`; typed Input gets no Quantifier analysis but
Impersonate Input does (row 5 and D4 cite it); the Connection form parses the provider
with `ALL` but offers `SELECTABLE`; `finish_reason=length` shown as complete; every HTTP
status error shown as "could not be reached"; the Quantifier lists map Rooms only.

## Dropped candidates: 11

Dropped because another path already performs the check, or because the consequence is
nil:

1. World/Character/Persona **name uniqueness** — I looked for a rule on any path; there is
   none, so nothing diverges. (Not a difference between entry points.)
2. **Posture parsing** (`parse_or_default` on the two World writes against `from_str` on
   the posture route) — reported as http-09.
3. **Retrigger button visibility** — `show_retrigger` comes from
   `tail.retrigger_allowed()`, one source for both the view and the handler
   (`templates.rs:37`, `stories.rs:60`). No divergence.
4. **World key in a URL path** — reported as http-08; the empty-key case is the same site.
5. **`default_scenario_id` dropped on create** — folded into item 8 above, because the
   field has no reader.
6. **A World's `key` field ignored on edit** — the input is `readonly` in the form
   (`worlds.rs:89`), so no client can send another.
7. **`max_tokens_ceiling` set only in `settings.json`** — the edit route preserves it
   deliberately (`settings.rs:271` copies the stored value back), so the dashboard
   cannot damage it; it can only fail to create it.
8. **"Send Original" posts the original text, not the corrected text** — commented as
   intended (`templates.rs:96` "The corrections are suggestions the player opts into, so
   Send Original keeps the primary style.").
9. **`MAX_TOKENS`-style numeric guards on the preset form** — the form owns no numeric
   field at all, so there is no range to compare.
10. **Autoplay's Budget range** (`budget.rs:22-27`) — the dashboard is the only entry
    point that sets a Budget, so nothing diverges.
11. **The `/options` regeneration button posting to `/action/check`** while no other
    Options surface exists (`templates.rs:64`) — one route, so no second rule.
