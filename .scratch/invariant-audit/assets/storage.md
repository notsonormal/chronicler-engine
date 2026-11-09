# Storage invariants: relied on, not explicitly enforced

Scope: SQLite schema, migrations, PRAGMA, and the row-to-domain mapping.
Read-only investigation. No `cargo`, no `build.py`. Every claim cites `file:line`.

Out of scope by instruction: the story semantics of the active Swipe and of a
contiguous `swipe_index` (story scout). This report covers only where storage
relies on them.

---

## storage-01 — A Message row and its first Swipe row land in one transaction

**1. Invariant.** A `messages` row never exists without at least one
`message_swipes` row.

**2. Relied on at.** `src/adapters/driven/storage/mappers/message.rs:63-69` turns
a Message with no Swipe into an `EngineError::Internal("message {} has no swipe")`.
`load_message` (`src/adapters/driven/storage/stories.rs:299`) uses that mapper, and
`read_tail`, `window` and `messages` all call it. `stories.rs:229-237` raises the
same error for the window read. So one Swipe-less Message makes the tail, the
window and the whole-story read of that Game fail: the dashboard shows no story and
the prompt assembler has no history.

**3. Write paths that could violate it.** `Tx::insert_message`
(`src/adapters/driven/storage/transaction.rs:139-155`) and `Tx::insert_swipe`
(`transaction.rs:183-204`) are two independent record writes. The three call sites
pair them (`src/application/generation/gate.rs:635-636`, `826-831`, `911-916`), but
nothing in `Transaction` or in the schema requires the pair.

**4. Enforcement.** NONE at the writer. The schema has no constraint that can
express it (`UNIQUE(message_id, swipe_index)` does not require a row to exist).
The reader is loud in `mappers/message.rs:63` and `stories.rs:229`, and silent in
`stories.rs:19`, whose `COALESCE(s.text, '')` shows an empty entry for the same
broken row.

**5. Consequence if violated.** The Game becomes unreadable: `Tail` fails, so no
narration, no Retry, no Options, and the story window answer is an error.

**6. Likelihood.** Code bug only: a new call site that inserts a Message without its
first Swipe, or a future path that inserts the Swipe after the commit. A player, an
LLM output and a World author cannot trigger it today.

**7. Status.** VERIFIED (all three call sites read; the mapper and both readers read).

**8. Suggested enforcement point.** Add a `Tx::insert_message_with_swipe` that
performs both writes under one `TxWrite`, and make `insert_message` crate-private
to that pair.

---

## storage-02 — A swipe write names a Swipe row that exists

**1. Invariant.** `SwipeAt { message, swipe }` names an existing
`message_swipes` row of the Message.

**2. Relied on at.** The `UPDATE`s at `transaction.rs:209-213` (`set_swipe_text`)
and `transaction.rs:221-228` (`set_swipe_state`) match on
`message_id = ?2 AND swipe_index = ?3`. Their result is discarded (`Ok(())`), so a
write that matches no row is a silent success.

**3. Write paths that could violate it.** `tx.set_swipe_text` and
`tx.set_swipe_state` are called from the Job with a `SwipeAt` built from the tail
read. `SwipeAt` carries a bare `u64`/`usize` pair (`domain/model/message.rs`), so
no constructor validates it.

**4. Enforcement.** PARTIAL. `require_message_of_game`
(`src/adapters/driven/storage/utils/records.rs:13-31`) checks the Message belongs
to the `Tx`'s Game, and `set_game_config` (`transaction.rs:293-295`) checks
`affected == 0`. The two swipe writes do not. `UNIQUE(message_id, swipe_index)`
stops a duplicate insert, not an update that misses.

**5. Consequence if violated.** Silent lost write. The Job reports success and the
stored Swipe state stays stale, so the next generation reads the old Room and the
old Characters present, and the Retry writes the wrong state.

**6. Likelihood.** Code bug only (a miscomputed index). Not reachable from a form,
a World or an LLM output.

**7. Status.** VERIFIED.

**8. Suggested enforcement point.** Raise `EngineError::MessageNotFound`-style
error in both functions when `execute` returns 0, and give `SwipeAt` a validating
constructor.

---

## storage-03 — `messages.active_swipe_index` names a Swipe of that Message

**1. Invariant.** For every `messages` row, a `message_swipes` row exists with
`message_id = id AND swipe_index = active_swipe_index`.

**2. Relied on at.** `stories.rs:24` joins the active Swipe
(`LEFT JOIN message_swipes s ON s.message_id = m.id AND s.swipe_index = m.active_swipe_index`).
`stories.rs:229-237` fails when that join yields no `state`. `stories.rs:159`
(`has_text`) joins the same way and silently skips a broken Message.

**3. Write paths that could violate it.** `Transaction::set_active_swipe`
(`transaction.rs:158-170`) writes any `at.swipe` into `active_swipe_index` after
checking only the Message's Game. `Transaction::insert_message`
(`transaction.rs:150`) writes `message.active_swipe_index` before any Swipe row
exists. `delete_message` is safe (cascade), but an `UPDATE` of `swipe_index` from a
future migration would not be.

**4. Enforcement.** NONE. The schema has no foreign key from
`messages(id, active_swipe_index)` to `message_swipes(message_id, swipe_index)`
(`plumbing.rs:674-682`, the v28 rebuild), and no `CHECK`.

**5. Consequence if violated.** The window read errors, so the dashboard's story
part shows a failure banner; `entries` shows an empty text for the same Message;
`load_message` repairs the index to 0 (`domain/model/message.rs:148-155`), so the
prompt path silently retells the story from Swipe 0.

**6. Likelihood.** Code bug only. The Retry and swipe-switch paths compute the index
from the loaded Swipes, so a normal player does not reach it.

**7. Status.** VERIFIED (write paths and all three readers read). The repair branch
in `ensure_valid_swipe_index` is not exercised by any production path.

**8. Suggested enforcement point.** Make `set_active_swipe` verify the target row
exists in the same transaction, or store the active Swipe as a row reference that a
FK can check.

---

## storage-04 — `update_world` is atomic like `write_world`

**1. Invariant.** A `worlds` row and its `maps` row change together.

**2. Relied on at.** `get_world` (`worlds.rs:84-120`) and `get_world_by_id`
(`worlds.rs:341-379`) read the two rows as one `WorldWithMap` and hand both to the
movement and scene rules. `write_map_row` failing leaves the map on the old Rooms
while the card names new ones.

**3. Write paths that could violate it.** `write_world`
(`worlds.rs:163-187`) opens `unchecked_transaction` and writes both rows. The
comment there states the rule: "World and map are one write".
`update_world` (`worlds.rs:297-335`) writes the `UPDATE worlds` and then calls
`Self::write_map_row(&conn, ...)` on the bare connection, with no transaction.

**4. Enforcement.** PARTIAL: the seed and create path is atomic, the edit path is
not. `update_world` is the only writer behind the Worlds edit form
(`src/adapters/driving/http/worlds/handlers/worlds.rs:211-213`) and the posture
form (`worlds.rs:290-292`).

**5. Consequence if violated.** A failed map write (disk full, a deleted map row,
a serialization error) leaves a World whose card and map disagree. The next Jobs
read a stale map, so movement and `state.room(&map)` resolve against Rooms that the
card no longer holds.

**6. Likelihood.** A storage error at the wrong moment. A World author triggers it
only through that error; the handler itself is normal.

**7. Status.** VERIFIED.

**8. Suggested enforcement point.** Wrap the body of `update_world` in
`unchecked_transaction` exactly as `write_world` does.

---

## storage-05 — One `maps` row for each World

**1. Invariant.** A `worlds` row has exactly one `maps` row.

**2. Relied on at.** `write_map_row` (`worlds.rs:273-296`) relies on it to
"never leave a second map behind": the `UPDATE` is unconditional and the `INSERT`
runs only when `updated == 0`. `get_world` (`worlds.rs:101-105`) and
`get_world_by_id` (`worlds.rs:357-361`) use `query_row`, which takes the first row
and ignores the rest.

**3. Write paths that could violate it.** The schema (`plumbing.rs:179-186`) has
`idx_maps_world` on `maps(world_id)` and no `UNIQUE`. A manual `INSERT`, an import,
or a future writer that inserts without the `updated == 0` check creates a second
row.

**4. Enforcement.** NONE. Only the `write_map_row` sequence protects it, and that
sequence depends on the row being unique to stay correct.

**5. Consequence if violated.** Two map rows drift apart: `write_map_row` updates
both, but nothing keeps their `created_at` or their existence in step with a delete.
A later `DELETE FROM worlds` cascades both away, so the damage is a read that
returns a stale one after a hand edit.

**6. Likelihood.** Manual DB edit or import only. Not reachable from the app.

**7. Status.** VERIFIED (schema and both reads read).

**8. Suggested enforcement point.** `CREATE UNIQUE INDEX idx_maps_world_unique ON
maps(world_id)` in a new migration.

---

## storage-06 — A row struct reads the columns its `SELECT` lists, in order

**1. Invariant.** The positional reads in `from_row` match the column order of every
`SELECT` that feeds them.

**2. Relied on at.** `DbGame::from_row` (`models/game.rs:30-49`) reads 18 columns by
index. Three separate `SELECT` strings feed it: `games.rs:16` (`list_games`),
`games.rs:46` (`list_saved_games_page`), `games.rs:113` (`get_game`).
`stories.rs:19-22` (`ENTRY_COLUMNS`) plus `ENTRY_COLUMN_COUNT = 8`
(`stories.rs:22`) feed `story_row` (`stories.rs:246-266`), which reads indices 0..7
and, in the window read, index 8.

**3. Write paths that could violate it.** Any edit of one `SELECT` list that is not
copied to the other two, and any edit of `ENTRY_COLUMNS` that does not update
`ENTRY_COLUMN_COUNT`.

**4. Enforcement.** NONE. The only guard is the comment at `stories.rs:21`
("keep it equal to the columns that `story_row` reads") and the comment at
`games.rs` naming `DbGame::from_row`. `DbWorld` reads by name
(`models/world.rs:25-44`) and `WORLD_COLUMNS` (`worlds.rs:399-411`) derives every
world list from one array, so those two are safe.

**5. Consequence if violated.** A shift between two columns of the same TEXT type is
silent: `games.name` and `games.world_name`, or `persona_key` and `persona_name`,
swap and the header and the names are wrong with no error. A shift that lands a
JSON column into a plain one (or the reverse) fails loudly in `serde_json`.

**6. Likelihood.** Code bug only, and a plausible one: three copies of one list.

**7. Status.** VERIFIED (all three lists compared against `DbGame::from_row`).

**8. Suggested enforcement point.** Make `DbGame` read by column name as `DbWorld`
does, or derive the three lists from one `const GAME_COLUMNS: &[&str]`.

---

## storage-07 — `games.name` (the stable generated name) is unique

**1. Invariant.** No two `games` rows share `name`.

**2. Relied on at.** `generate_game_name` (`domain/model/utils/game_name.rs:6-18`)
counts the existing names to pick the next ordinal, and `games.rs:84` states "The
uniqueness scan keys on `name`, so a rename must not touch it".

**3. Write paths that could violate it.** `Tx::write_game_row`
(`transaction.rs:86-112`) inserts whatever `NewGame.name` holds, with no scan.
`GameCatalogue::create_game` (`application/games/catalogue.rs:50-58`) and
`reset_game` (`catalogue.rs:266-274`) read the names in one `Storage` call and write
in a later one, so two concurrent creates can compute the same ordinal.

**4. Enforcement.** TEST-ONLY. `application/games/catalogue_tests.rs:151-153`
asserts the names differ. The schema has no `UNIQUE` on `games.name`
(`plumbing.rs:66-73`), which is deliberate: the display name may collide.

**5. Consequence if violated.** Two Games share the stable name; the `1`..`n` suffix
scan produces a third with the same name. Nothing indexes on `name`, so the visible
effect is two identically named Games in the Games list and in the log line at
`bootstrap/run.rs:113`.

**6. Likelihood.** Concurrency only: two create or reset requests for the same World
at the same time. A single player does not reach it.

**7. Status.** VERIFIED.

**8. Suggested enforcement point.** `UNIQUE(games.name)` in a new migration, with
the create retrying on a unique violation.

---

## storage-08 — `swipe_index` and `active_swipe_index` are never negative

**1. Invariant.** Both integer columns hold a value in `0..=usize::MAX`, so an
`i64` to `usize` conversion keeps its meaning.

**2. Relied on at.** The conversion sites: `mappers/message.rs:60`
(`db.active_swipe_index as usize`), `stories.rs:141`, `stories.rs:265`,
`stories.rs:354`. A `-1` becomes `18446744073709551615`, which then flows back into
`at.swipe as i64` at `transaction.rs:211` and `stories.rs:334`.

**3. Write paths that could violate it.** The schema declares
`active_swipe_index INTEGER NOT NULL DEFAULT 0` (`plumbing.rs:680`) and
`swipe_index INTEGER NOT NULL` (`plumbing.rs:688`) with no `CHECK`.
`insert_message` (`transaction.rs:150`) and `insert_swipe` (`transaction.rs:191`)
bind a `usize` cast to `i64`, so the app cannot write a negative value today.

**4. Enforcement.** NONE in the schema. The app side is safe by the `usize` type,
which is why this is a low-severity finding: only a manual edit, an import, or an
older/newer version can produce the value.

**5. Consequence if violated.** `ensure_valid_swipe_index`
(`domain/model/message.rs:148-155`) silently resets the out-of-range index to 0 in
the full-story read and reports the repair through a `bool` that
`mappers/message.rs:72` discards. The window read errors instead. Two readers give
two answers for one row.

**6. Likelihood.** Manual DB edit only.

**7. Status.** VERIFIED (sites and both readers read).

**8. Suggested enforcement point.** `CHECK (swipe_index >= 0)` and
`CHECK (active_swipe_index >= 0)`, or read both as `u32` with `try_from`.

---

## storage-09 — `messages.game_id` is always written explicitly

**1. Invariant.** Every `messages` row names the Game that owns it.

**2. Relied on at.** Every story read filters on `m.game_id` (`stories.rs:160`,
`191`, `221`, `273`, `291`). A Message in the wrong Game appears in that Game's
story, its tail and its prompt history.

**3. Write paths that could violate it.** The column is declared
`game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE`
(`plumbing.rs:676`). An `INSERT` that omits `game_id` takes Game 1 and succeeds
whenever Game 1 exists, because the foreign key is satisfied.

**4. Enforcement.** NONE. No `CHECK` and no trigger forbids the default. The one
production insert names it (`transaction.rs:143-153`).

**5. Consequence if violated.** Another Game's Message lands in Game 1's story:
wrong prompt history, wrong tail, wrong Retry target. Silent.

**6. Likelihood.** Code bug only (a new insert statement, a test fixture, a manual
SQL session while Game 1 exists).

**7. Status.** VERIFIED (schema and the one production insert read).

**8. Suggested enforcement point.** Drop the `DEFAULT 1` in a new migration
(SQLite needs the 12-step table rebuild; the v28 rebuild is the precedent).

---

## storage-10 — A preset id in a Game slot and in the settings registry names a preset

**1. Invariant.** `games.active_*_prompt_preset_id` and
`settings.mode_preset_registry` name `prompt_presets` rows that exist.

**2. Relied on at.** `PromptPresetService::chosen_preset`
(`application/prompt_preset_service.rs:66-72`) turns a missing id into
`EngineError::PresetNotFound`, so the Job fails.

**3. Write paths that could violate it.** `PromptPresetService::delete_preset`
(`prompt_preset_service.rs:110-141`) checks `get_preset`, `get_settings`,
`mode_preset_registry.references` and `count_games_using_preset` in four separate
`Storage` calls, then deletes in a fifth. Each call takes the backend mutex; the
sequence does not. A `create_game` that reads the settings in that window writes a
Game that names the deleted preset. A hand edit of `settings` or of a `games` row
also breaks it.

**4. Enforcement.** PARTIAL: the delete path checks, no path holds the check and the
delete together, and no `FOREIGN KEY` exists (by design, `storage.md` puts these
cross-cluster references on the application).

**5. Consequence if violated.** The Game's next Job fails with `PresetNotFound`; the
failure line shows on the dashboard and the Game stays idle. The preset card list
shows a mode default that no longer exists.

**6. Likelihood.** Two overlapping requests (a delete and a create, or a delete and
a preset switch). Not reachable from one player action.

**7. Status.** VERIFIED for the multi-call sequence; the race itself is INFERRED
from the lock granularity of `Storage::with_backend_mut`.

**8. Suggested enforcement point.** Move the check and the delete into one
`Storage::transaction`-style call, or add a `PresetDeletion` guard on `GameWriter`.

---

## storage-11 — The settings singleton row exists

**1. Invariant.** `settings` holds row `id = 1`.

**2. Relied on at.** `storage.md` states settings are "the engine's only settings
read source". Every model call resolves its Connection from this row.

**3. Write paths that could violate it.** `seed_settings`
(`settings.rs:117-123`) uses `INSERT OR IGNORE`, and `bootstrap/run.rs:63-64` makes
a seed failure fatal, so boot normally creates the row. There is no delete path. A
manual `DELETE FROM settings`, or a database whose seed never ran, leaves no row.

**4. Enforcement.** READ-SIDE-ONLY. `read_settings` (`settings.rs:28-37`) returns
`Ok(AppSettings::default())` on `QueryReturnedNoRows`. `update_settings` then writes
that default back (`settings.rs:100-107`), so the first write after a lost row
persists the defaults, including the narration Connection id.

**5. Consequence if violated.** The engine runs on `AppSettings::default()`: the
configured Connections, Agents and mode registry are gone with no error, and the
dashboard shows defaults. The player sees their settings reset.

**6. Likelihood.** Manual DB edit, or a future code path that skips the fatal seed.

**7. Status.** VERIFIED.

**8. Suggested enforcement point.** Make `read_settings` return
`EngineError::SettingsMissing` and keep the default only in `bootstrap`.

---

## storage-12 — Datetime columns are comparable as TEXT

**1. Invariant.** Every datetime column holds RFC3339 in UTC with the `+00:00`
offset, so text order equals time order.

**2. Relied on at.** `games.rs:18` and `games.rs:49`
(`ORDER BY updated_at DESC, id DESC`) order the Games list and the saved-games page;
`games.rs:149` (`MAX(timestamp)`) picks the newest Message that decides the boot
Game; `llm_messages.rs:42` (`ORDER BY created_at DESC LIMIT 50`) prunes; and
`presets.rs:20` orders the preset cards.

**3. Write paths that could violate it.** Every write makes its own
`Utc::now().to_rfc3339()` call: `transaction.rs:87`, `:255`, `games.rs:91`,
`core.rs:264`, `settings.rs:48`, `personas.rs:57`, `worlds.rs:172`, `:306`,
`characters.rs:54`, `presets.rs:123`, plus `transaction.rs:149`
(`message.timestamp.to_rfc3339()`). `parse_datetime` (`plumbing.rs:10-16`) accepts
any offset on read and normalizes it, so a read never reveals the mismatch.

**4. Enforcement.** NONE. No `CHECK` on the text shape, and no single helper that
all writes must call. The current writers are all `Utc`, which is why the ordering
holds today.

**5. Consequence if violated.** A row written as `+02:00` sorts wrongly against
`+00:00` rows: the boot picks the wrong Game of a World, the saved-games page
orders wrongly, and the LLM-message retention prunes the wrong row.

**6. Likelihood.** Code bug only (a new writer using `Local::now()` or a date-only
format).

**7. Status.** VERIFIED (all writers grepped; `Local::now()` appears only at
`bootstrap/logging.rs:32`, outside the database).

**8. Suggested enforcement point.** One `storage::now_text()` helper for every
datetime bind, plus a `CHECK (created_at LIKE '____-__-__T__:__:__%.*+00:00')` on a
rebuilt table.

---

## storage-13 — The database version never exceeds `LATEST_VERSION`

**1. Invariant.** `PRAGMA user_version` is at most `LATEST_VERSION` (34,
`plumbing.rs:17`).

**2. Relied on at.** `migrate_up_to` (`plumbing.rs:24-40`) treats a higher version
as "nothing is due" and returns `Ok(())`. Every storage read then runs against a
schema this build does not know.

**3. Write paths that could violate it.** The version is raised only by this file,
so the violating state comes from outside: a newer build opens the same file, a
downgrade follows, or the file is copied between checkouts.

**4. Enforcement.** NONE. Nothing compares `version` with `LATEST_VERSION` above the
limit.

**5. Consequence if violated.** Every read names its columns explicitly, so an added
column is ignored safely. A renamed or dropped column fails loudly (`no such
column`) at the first read, which stops boot. Silent wrong data needs a rename that
keeps the position and the type, for example a future `messages.kind` reuse.

**6. Likelihood.** Operator action only: running an older binary on a newer
database.

**7. Status.** VERIFIED (the `due` closure and every `SELECT` list read).

**8. Suggested enforcement point.** Return `EngineError::Config` from
`run_migrations` when `version > LATEST_VERSION`.

---

## Checked and enforced

One line each, so the reader sees what is ruled out.

- **`PRAGMA foreign_keys = ON`.** Set once per `DbPool` (`db.rs:20-21`) on the only
  `Connection`, so every `REFERENCES ... ON DELETE CASCADE` is live for all reads
  and writes through that pool. Only one pool exists per server (`bootstrap/run.rs:51`).
  `db_tests.rs:162-229` asserts the cascade. Note: the pragma is set *after*
  `run_migrations` (`db.rs:19`), so migrations run with the foreign keys off; the
  `DELETE`/`DROP` steps of v28 and v30 delete their children explicitly, so
  nothing depends on it today.
- **Migration atomicity.** One `unchecked_transaction` for the whole run, one commit
  at `plumbing.rs:798`. A crash mid-run rolls back, and `user_version` moves inside
  the same transaction.
- **Migration ordering and re-entry.** Steps run in ascending order, and the
  `column_exists` guard protects every `ALTER TABLE ADD COLUMN` except the v13 pair
  (`plumbing.rs:264-265`), where a re-entry fails loudly with `duplicate column
  name`. `user_version` moves per step, and a version above `LATEST_VERSION` runs
  nothing (see storage-13).
- **`game_state_snapshots`.** Created at v9 (`plumbing.rs:76`), indexed at `:88`,
  dropped at v28 (`plumbing.rs:670`). No reader and no writer anywhere in `src/`;
  the only remaining mentions are the historical migration steps.
- **`game_docks` orphan rows.** The table carries a real foreign key
  (`game_id INTEGER PRIMARY KEY REFERENCES games(id)`, `plumbing.rs:655`), and
  every write takes the `GameId` from the `Tx` (`transaction.rs:308-341`). The
  `ON CONFLICT(game_id)` upsert relies on that primary key, which exists.
- **`maps` orphan rows.** `maps.world_id REFERENCES worlds(id) ON DELETE CASCADE`
  (`plumbing.rs:181`), written only through `write_world`'s transaction or
  `update_world` (storage-04 covers the second).
- **`llm_messages` orphan rows.** The table has no foreign key because an LLM
  message belongs to no Game, Message or World (`CONTEXT.md`, *LLM message*). There
  is nothing to orphan.
- **`characters` and `personas` keys.** `UNIQUE(key, world_id)` on `characters`
  (`plumbing.rs:225`) and `UNIQUE` on `personas.key` (`plumbing.rs:192`). Both seeds
  use `INSERT OR IGNORE` (`characters.rs:63`, `personas.rs:62`) and depend on it.
- **`worlds.key`.** `UNIQUE` (`plumbing.rs:165`); `create_world` maps the violation
  to `EngineError::WorldAlreadyExists` (`worlds.rs:239-247`), and the World edit
  handler never takes the key from the form
  (`worlds/handlers/worlds.rs:176-205`), so `games.world_key` cannot be orphaned by
  a rename.
- **`games.world_key → worlds.key` at world delete.** `delete_world`
  (`worlds.rs:372-393`) counts the Games of the World and refuses
  (`EngineError::WorldHasGames`). The logical reference is checked on every delete.
- **Prompt preset name uniqueness.** A scan, not a constraint, and the code says so
  (`presets.rs:65-68`). The scan and the write share one backend acquisition
  (`presets.rs:112-116`), and `save_preset` is the only writer, including bootstrap
  (`bootstrap/run.rs:268`, `:274`). A duplicate can only arrive from outside the app.
- **`game_docks` singleton.** Primary key on `game_id`, `INSERT OR REPLACE` /
  `ON CONFLICT` only (`transaction.rs:308-341`).
- **`settings` singleton.** `CHECK (id = 1)` (`plumbing.rs:232`) rejects a second
  row, and every write hardcodes `id = 1` (`settings.rs:69-99`).
- **JSON TEXT columns.** `connections`, `text_check`, `agents`,
  `mode_preset_registry` (`models/settings.rs:44-53`), `inventory`, `triggers`,
  `relationships` (`models/character.rs:50-57`), `global_rules`, `scenarios`
  (`models/world.rs:46-53`), `map_data` (`worlds.rs:107-109`), `state` and `maker`
  (`mappers/message.rs:33-45`), `offered` and `notice` (`docks.rs:33-38`),
  `parameters`, `allowed_modes` (`models/prompt_preset.rs:45-56`), `message_type`
  and `kind` (`mappers/message.rs:15-31`), and `failure` (`models/game.rs:83-96`)
  all fail loudly with `EngineError::Parse` on corrupt text. No reader substitutes
  an empty collection.
- **`option_count`.** `u32::try_from` with a `Parse` error (`models/prompt_preset.rs:59-68`);
  no silent truncation.
- **`preset_type`.** `PresetType::try_from` (`models/prompt_preset.rs:46-47`), loud.
- **Posture enum text.** `NarratorMode`, `NarrativePerspective` and
  `NarrativeTense` are read with `parse_or_default` (`models/game.rs:63-71`,
  `models/world.rs:56-65`), which logs a warning and falls back. Every writer
  converts an enum with `as_str()` (`transaction.rs:100-102`, `worlds.rs:232-234`)
  or parses the form value first (`worlds/handlers/worlds.rs:200-208`), so the
  columns cannot hold an unknown value through the app. The fallback is reachable
  only by a manual edit.
- **`busy_timeout`, WAL, pool size.** One `Connection` behind one `Mutex`
  (`db.rs:13-33`), and one server per port with its own file
  (`bootstrap/run.rs:45-51`). No writer runs concurrently, so no busy timeout is
  needed and none is set. A second process on the same file would fail loudly with
  `Migration failed` or a `Database` error, not corrupt data.
