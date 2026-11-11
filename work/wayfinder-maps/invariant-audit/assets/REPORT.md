# Invariants the engine relies on but does not enforce

Six read-only scouts searched one theme each: story, gate, world, llm, http, storage.
Their full reports are next to this file (`<theme>.md`). Each finding there has the
reliance site, the write paths, the evidence and a suggested enforcement point.

- 56 findings. 3 pairs are duplicates, so 53 invariants are unique.
- I read the code of 14 findings myself (column "Me" = ✔). 13 hold. I disagree with
  one (http-07, see "Disagreements").
- The other findings carry the scout's status. Most are VERIFIED by the scout, not by me.

Ranking rule: first, who can break the invariant today; second, the consequence.

## Tier 1: a player, an LLM answer or a World author can break it today

| # | ID | Invariant that is not enforced | What happens | Me |
|---|---|---|---|---|
| 1 | llm-01 | An empty Characters present list from the Quantifier means "nobody is here" | `parser.rs:33-42` labels any parsed JSON High. `process_quantifier_result` then drops every id that is not a Character key. A model that answers names (`"Carla"`) instead of ids, or omits the field, empties the scene: every Character gets a `Left` transition. The prompt shows both `id` and `name`, so the name answer is a natural one. | ✔ |
| 2 | world-04 | "The first matching Trigger fires" has one fixed order | `evaluate_triggers` iterates `HashMap::values()` and `fired_trigger` takes `.next()` (`game_rules.rs:87-90,173`). The order is random per map instance. With two satisfiable Triggers, a Retry can fire a different event. `CharacterCard::in_id_order` exists but the rules do not use it. | ✔ |
| 3 | llm-02 | A destination names a Room by id | `walk` matches ids only (`game_rules.rs:136`). Any other text, including a Room name the prompt showed, makes a new dynamic Room. The same place twice gives two Rooms, and the Room image is lost. | ✔ |
| 4 | llm-03 | A move goes through an exit of the current Room | `Room::exits` has no reader in `src/`. The Quantifier prompt lists every Room, so the Persona can teleport. | |
| 5 | story-06 | A one-shot Trigger writes at most one event Narration | Retrigger trusts the event record and skips the fired mark. Delete the event Narration, then Retrigger: the same event is written again. | partial |
| 6 | world-02 | The current Room of a Game resolves in the map | A World with `scenarios_json: []` (the form placeholder) or a wrong `starting_room_id` passes the create form. `starting_room_id()` then falls back to the literal `"start"` (`world.rs:74-76`). The new Game is unplayable. | ✔ |
| 7 | llm-04 | Every `user`/`assistant` prompt part has text | `edit_history_handler` and `GameWriter::edit` accept an empty text (`gate.rs:560-571`). The transport drops only an empty system part. A provider that refuses empty content then fails every later generation of the Game. | ✔ |
| 8 | gate-06 | One Job reads one config | `save_config` refuses only an Autoplay holder (`gate.rs:501-508`). A mode, posture or preset change during a plain Job is accepted. The Job reads the Game row again per step, so the narration and the Quantifier can use different presets. | ✔ |
| 9 | http-08 | A World key is safe in a URL path and as an element id | Only `UNIQUE` checks the key. A key with `/ ? # %` is stored, and after that the World cannot be edited or deleted from the dashboard. | |
| 10 | world-03 | Scenario Characters exist in the World | BUILD-TIME only (`validate_data.py`). The HTTP World forms skip the check. A wrong id silently drops the Character from the opening scene. | |
| 11 | world-05 | A Trigger `room_id` names a Room of the map | BUILD-TIME only. The Trigger never fires, and a DEBUG line is the only sign. | |
| 12 | world-07 | A Relationship `with` names a Character of the World | No check. The relationship silently leaves the prompt. | |
| 13 | llm-05 | Agent prompts fit the context window | Only the narrator has a `TokenBudget`. A context overflow makes the Quantifier fall back, so the scene silently freezes. | |
| 14 | llm-10 | `max_tokens` is at least 1 | The narrator clamps with `.max(1)`. The Quantifier and Options calls send a preset value of 0 unchanged. | |
| 15 | llm-06 | The token estimate is an upper bound | Fixed 4 characters per token. Non-Latin text overruns the window. | |
| 16 | http-09 | A posture value from a form is valid | 3 routes refuse a bad value. The 2 World write paths silently replace it with the default. | |
| 17 | gate-08 | The Games list follows the newest Message | A comment in `changes.rs:180` says so. The SQL orders by `updated_at`, and no Message write touches that column. | |
| 18 | world-01 | A Game starts in the Scenario that `default_scenario_id` names | The field is stored but never read. Both `default_scenario()` functions return `scenarios.first()` (`world.rs:63-71`). This is latent: no shipped World sets the field. | ✔ |

## Tier 2: two requests or two tabs race

| ID | Invariant | What happens | Me |
|---|---|---|---|
| gate-02 = http-01 | The Game that the header check verifies is the Game that the handler writes | `refuse_stale_game` reads the active Game pointer (`request_layers.rs:45`). `CurrentGame` reads it again (`current_game.rs:34`). A switch or create on another thread in between makes `/action`, `/reset` or `/status/reset-busy` act on the new Game. The window is small but real. Two scouts found it independently. | ✔ |
| storage-07 | `games.name` is unique | The create and reset paths read the names and write them in separate calls. The schema has no `UNIQUE`. | |
| storage-10 | A preset id in a Game or in the mode registry names a preset | The delete path checks in 4 calls and deletes in a 5th call, with no hold over the sequence. | |

## Tier 3: a provider fault or a storage fault breaks it

| ID | Invariant | What happens | Me |
|---|---|---|---|
| llm-07 | A truncation index falls on a character boundary | `&error_body[..500]` (`response.rs:140`) and `&response[..len.min(200)]` (`quantifier/.../orchestration.rs:110`) slice by bytes. A long non-ASCII error body causes a panic in the model call. | ✔ |
| gate-05 | A Job that ends with no records writes a failure line | Only the `Result` error paths write it. A panic (for example llm-07) releases the hold through `Drop` but writes no line. The accept already cleared the old line, so the status says Ready and the Action is gone. | |
| gate-07 | A refused reset changes nothing | `replace` removes the old holder before the delete unit (`gate.rs:609-610`). When the delete fails, the running Job is stopped and the new Game stays as an orphan. | ✔ |
| storage-04 | The `worlds` row and the `maps` row change together | `write_world` uses a transaction. `update_world` does not (`worlds.rs:305-332`). A failed map write leaves the card and the map different. | ✔ |

## Tier 4: only a code bug, a manual DB edit or an operator mistake

These are defence-in-depth gaps. The code paths of today keep the invariant, but nothing
makes a violation fail at the write.

| Group | IDs | Invariant |
|---|---|---|
| Story records | storage-01 | A Message row always has a Swipe row. Insert of the Message and of its first Swipe are two separate `Tx` calls. |
| | storage-02 | A Swipe update finds its row. `set_swipe_text` and `set_swipe_state` ignore 0 affected rows. |
| | storage-03 = story-02 | `active_swipe_index` names an existing Swipe. Three readers give three results: an error, an empty text, and a silent repair to 0. |
| | story-01, storage-08 | `swipe_index` is `0..n` with no gap and never negative. The schema has only `UNIQUE(message_id, swipe_index)`. |
| | story-03 | A missing start state is an error. The read turns it into "this is the opening", so a Retry applies the movement twice. |
| | story-04 | `kind` agrees with `message_type`. They are two independent fields. |
| | story-05 | A Retry Swipe goes on the Message that is still last. Only the busy rule keeps this true, and the write does not check it. I read the code, and it is safe today. |
| Schema | storage-06 | Positional `from_row` reads match 3 copied `SELECT` lists for `DbGame`. |
| | storage-09 | `messages.game_id DEFAULT 1`: an insert that omits it lands in Game 1. |
| | storage-05 = world-10 | One `maps` row per World. No `UNIQUE`. |
| | storage-11 | The settings row exists. A missing row silently becomes `AppSettings::default()`, and the next save persists the defaults. |
| | storage-12 | All datetime text is UTC RFC3339. Sort order depends on it, and each writer formats its own value. |
| | storage-13 | `user_version` ≤ `LATEST_VERSION`. An older binary runs on a newer DB with no error. |
| Gate | gate-01 | The active Game pointer names an existing Game. Boot only logs a warning, and `touch_game_updated_at` ignores 0 rows. |
| | gate-03 | Lock order: no `Storage` call inside a `Tx` closure, no hold inside a hold, no `ActiveGame` read under the guard. Only comments state it. A violation hangs the whole server. |
| | gate-04 | One process per DB file. The busy registry is in memory, and no file lock or `busy_timeout` exists. |
| Wiring and config | llm-08 | Every configured Agent is registered. `wiring.rs:152` replaces a `from_configs` error with `unwrap_or_default()`, so both Agents silently disappear. I read the code. |
| | llm-09 | The recorded agent name is a `Role` name. The Quantifier passes the literal `"quantifier"`. |
| | world-06 | A stored `trigger_fired` index names the same Trigger for the life of a Game. |
| | world-08 | Preset ids in settings exist and have the slot's type. No boot check exists, but one exists for Connection slots. |
| | world-09 | A preset seed's `preset_type` matches its folder. |
| HTTP and browser contract | http-02 | Every Game-bound route is in `game_bound_routes`. The test compares the list with its own source. |
| | http-03 | The slash-command names in the palette match `Action::parse`. |
| | http-04 | Form field names in the JS match the Rust form structs. |
| | http-05, http-06 | The `?v=` cache bust is updated. The embedded `index.html` matches the `assets/` read at run time. |

## Disagreements and corrections

- **http-07 (empty Autoplay budget becomes 10): I disagree that this is a gap.**
  `Budget::parse` returns `Budget::DEFAULT` for an empty field on purpose
  (`budget.rs:16-17`), and the range check holds for every other value. It is a product
  choice. If you want an empty field refused, that is a spec change.
- **gate-04 compared with the storage report "a second process fails loudly".** Both are
  partly right. SQLite serializes the writes, and with no `busy_timeout` a competing write
  fails at once (loud). The busy registry is per process, so two processes can both accept
  a Job on one Game (silent). Status: INFERRED. I did not confirm the gate scout's claim
  that the migrations run before the port bind.
- **llm-01 is worse than the scout said.** The scout gave the missing field as the
  trigger. A model answer with names instead of ids has the same result. That answer is
  likely, because the prompt prints `id` and `name` side by side.

## Fixes with the most effect

Each line names one enforcement point that closes several findings.

1. **One resolve function for Quantifier output** (id, then name, over map Rooms, added
   Rooms and Characters). An absent field or an unresolved name counts as Low confidence.
   This closes llm-01, llm-02, and llm-03 if it also checks exits.
2. **Sort the Characters in `evaluate_triggers`** with `CharacterCard::in_id_order`.
   This is a one-line fix for world-04.
3. **One runtime `World` validation** that the seed, the create form and the update form
   all call. Move the `validate_data.py` checks into it: Scenario Room, Scenario
   Characters, Trigger Rooms, Relationships, key characters, posture values,
   `default_scenario_id`. This closes world-01/02/03/05/07, http-08 and http-09.
4. **One story write for a Message and its first Swipe**, row-count checks on the Swipe
   updates, and `message_type` derived from `kind`. This closes storage-01/02/03,
   story-01/02/04.
5. **Build `CurrentGame` from the one read in `refuse_stale_game`** (a request
   extension). This closes gate-02/http-01.
6. **A guard in `Job::run` whose `Drop` writes the failure line**, plus one truncation
   helper that is safe for characters. This closes gate-05 and llm-07.
7. **Refuse empty text in `GameWriter::edit`**, and make `save_config` refuse any holder
   or read the config once per Job. This closes llm-04 and gate-06.
8. **A transaction in `update_world`**, and move `holders.remove(&old)` after the delete
   unit in `replace`. This closes storage-04 and gate-07.
9. **Propagate the `AgentRunner::from_configs` error** in `wiring.rs`, as the line above
   does for the Connection slots. This closes llm-08.

Fixes 2, 5, 7 and 9 are small and local. Fixes 1, 3 and 4 change a module boundary, and
each one needs a short design pass first.
