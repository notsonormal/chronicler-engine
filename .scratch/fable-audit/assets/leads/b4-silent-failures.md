# B4 — errors swallowed, and successes that leave the system inconsistent

Lens: every place that turns an error or a missing value into a default, a log line or
nothing; and every place where an operation reports success while the system is left
inconsistent.
Scope: `src/` without `*_tests.rs`, `assets/dashboard-*.js`, `scripts/`, `build.py`.
`CONTEXT.md` terms are used as written there.

## Pattern counts

| Pattern | Hits | Read | Rows that rest on it |
|---|---|---|---|
| `.ok()` | 17 | all | 1 (D3) |
| `unwrap_or` | 127 | all that default a *value*, not a display string | 4 (A4, A5, B3, B4) |
| `unwrap_or_default` | 42 | all | 2 (A3, A4) |
| `let _ =` | 3 | all, and all three are in `src/test_support/` | 0 |
| `if let Err` | 15 | all | 3 (A6, B2, D4) |
| `Err(..) =>` `warn!`/`debug!`/`continue` | 35 `warn!` lines | all | 5 (A1, C1, C2, D1, D4) |
| `map_err(|_|` | 4 | all, and each maps to a named, propagated error | 0 |
| `filter_map` over a `Result`/`Option` | 8 | all | 1 (C4) |
| JS `catch` / `.catch(` | 12 (4 `dashboard-panels.js`, 8 `dashboard-screen.js`; `htmx.min.js` excluded) | all | 0 |

Every row below cites the line it is built on. Label: **known** = I read the code that
shows it; **inferred** = the code says it but I did not run the trigger; **guessed** = a
consequence I reason about without proof.

---

## Group A — the player would want to know

### A1. A Quantifier that fails leaves the scene frozen, and the dashboard shows nothing
`src/application/agents/runner.rs:74-77`
```rust
        agent.run(input, settings).unwrap_or_else(|e| {
            tracing::warn!("Agent {} failed: {e}", agent.name());
            QuantifierResult::keeping_present(input.world.present)
        })
```
- **What failed:** the after-narration Agent (the Quantifier) returned an error, or no Quantifier is registered at all.
- **What the code does instead:** keeps the Room and the Characters present and lets the Job end as `Done`. The player sees a normal Narration with no movement and an unchanged scene. The failure is one `warn!` line. The two other signals a player could see contradict it: the Job ends `Done` (no failure line, `job.rs:385-387`) and the Role shows Healthy (see B1).
- **Who can see it today:** only the log file. The LLM Messages tab shows the failed call (`llm_recorder.rs:72`) unless that save also failed.
- **Who would want to know:** the player (the Persona stops moving) and the tuner (the prompt or the model is wrong).
- **Trigger:** point the Quantifier Connection slot at a host that answers 500; or disable the Quantifier Agent in settings — then `after_narration()` at `runner.rs:70-73` returns `keeping_present` with no log line at all.
- Label: **known**. Cross-reference: the report's llm-05 gives the context-overflow cause of the same fallback; this is the swallowing site and the "no line, no banner" half.

### A2. A Quantifier movement type other than "entering"/"leaving" is dropped without a word
`src/application/agents/quantifier/utils/parser.rs:171-175`
```rust
                .and_then(|t| match t.to_lowercase().as_str() {
                    "entering" => Some(MovementType::Entering),
                    "leaving" => Some(MovementType::Leaving),
                    _ => None,
```
- **What failed:** the model named a movement the engine does not know (`"walk"`, `"move"`, `"Moving"`, `"进入"`).
- **What the code does instead:** `None`, so no movement happens at all. `destination` is still parsed and then ignored, because `game_rules::apply` moves only when the reading carries a movement. The whole reading is High confidence, so the retry at `orchestration.rs:126` never runs.
- **Who can see it today:** nobody. Not even a debug line: `parse_with_movement` logs the Character result and not the rejected movement type.
- **Who would want to know:** the tuner (their Quantifier preset's output vocabulary does not match the parser) and the player (the Persona never moves).
- **Trigger:** edit the Quantifier preset to ask for `{"movement":{"type":"walk","destination":"…"}}`. The scene stays in one Room for the life of the Game.
- Label: **known**.

### A3. The failure detail of an HTTP answer disappears when the body is large or unreadable
`src/adapters/driving/http/utils/answer_words.rs:283-286`
```rust
            let bytes = to_bytes(body, UNBUILT_TEXT_LIMIT_BYTES)
                .await
                .unwrap_or_default();
            ErrorAnswer::unbuilt(status, &String::from_utf8_lossy(&bytes))
```
- **What failed:** the error body of a route that did not build an `ErrorAnswer` could not be read, or is larger than `UNBUILT_TEXT_LIMIT_BYTES` (64 KB).
- **What the code does instead:** `unbuilt(status, "")` → `detail: None` (`:109`) → the player gets the first line only ("The Action was not sent."), with no Details text and no reason. The 64 KB cap is also a silent truncation path, not an error path.
- **Who can see it today:** only the player, and only as a missing disclosure. The log has nothing.
- **Who would want to know:** the player (an Action vanished with no reason) and whoever reads the log next.
- **Trigger:** any handler that returns 500 with a large body and no `ErrorAnswer` — for example an axum form rejection echoing a pasted 100 KB World `map_json`.
- Label: **known** for the swallow, **inferred** for the 64 KB trigger.

### A4. A new Game's opening Narration can be empty, and nothing checks it
`src/domain/model/utils/opening.rs:16-19`
```rust
    let scenario = world.default_scenario();
    let text = scenario
        .map(|scenario| TemplateVars::new(persona_name).render(&scenario.text))
        .unwrap_or_default();
```
- **What failed:** the World has no Scenario, or the default Scenario has an empty `text`.
- **What the code does instead:** writes a Narration with `text: ""`. `narrator.rs:152-158` refuses an empty *model* answer (`EmptyText`), but the World-made opening has no such check.
- **Who can see it today:** the player sees a blank first entry in the story log. No log line; `create` returns Ok.
- **Who would want to know:** the World author and the player.
- **Trigger:** create a World through the form with `scenarios_json: []` (the form's own placeholder is `[]`), then start a Game on it.
- Label: **known**. Cross-reference: world-02 in the report names the same unvalidated World from the Room side; this is the empty-text side, which nothing else covers.

### A5. A Narration Retry with no preceding Input sends an empty user message
`src/application/narration/narrator.rs:176-181`
```rust
            Generation::Narration { .. } => match values.input {
                Some(text) if !text.is_empty() => text.to_string(),
                _ => Self::previous_input(values.history)
                    .unwrap_or_default()
                    .to_string(),
            },
```
- **What failed:** a `Narration` order carries no Input text, and the history window ends on a Message that is not an Input (the opening Narration, an event Narration, or a Narration whose Input was deleted).
- **What the code does instead:** builds the user message as `""`. Compare `options/prompt.rs:36-41`, which falls back to `FALLBACK_SYSTEM_PROMPT` for the same kind of absence. The transport drops only an empty *system* part, so an empty `user` part reaches the provider. A provider that refuses empty content fails every later Retry of that Message; one that accepts it narrates from nothing.
- **Who can see it today:** the tuner, if they open the LLM record and notice the User part is empty. The player sees only the outcome.
- **Who would want to know:** the tuner and the player.
- **Trigger:** `/history/delete` the only Input of a short Game, then Retry the Narration that followed it.
- Label: **known** for the empty string; **inferred** for the trigger reachability. Cross-reference: llm-04 in the report states the invariant and proposes `GameWriter::edit` as the enforcement point; this is a producer that check would not cover.

### A6. The accept reports a failure line it did not write
`src/application/generation/gate.rs:654-657`
```rust
        if let Err(error) = opener.open(game, |tx| tx.set_game_failure(Some(&failure))) {
            tracing::error!("accept: writing the failure line failed: {error}");
        }
        NotAccepted::Failed(failure)
```
- **What failed:** the write of the Game's failure line (a storage fault inside `TxOpener::open`).
- **What the code does instead:** returns `NotAccepted::Failed` anyway. The HTTP answer names the failure; the next screen load reads no failure line and shows Ready, so the status and the answer disagree — exactly what the comment at `gate.rs:33-35` says cannot happen. The Action that was accepted is gone and nothing explains it.
- **Who can see it today:** the player sees the answer once, then a clean status. Only the log has both halves.
- **Who would want to know:** the player.
- **Trigger:** a storage fault at the accept (the test suite already injects these through `StorageCall` overrides).
- Label: **known** for the divergence, **inferred** for the trigger.

---

## Group B — the tuner would want to know

### B1. Role health can only see transport failures, so a wrong *answer* reads as Healthy
`src/application/games/view_query.rs:262-266`
```rust
                RoleHealth {
                    role: *role,
                    backend_model: newest.map(|m| format!("{} {}", m.backend_name, m.model_name)),
                    last_error: newest.and_then(|m| m.error_message.clone()),
                }
```
- **What failed:** nothing the transport can see. The role is judged by `error_message` of its newest LLM message alone.
- **What the code does instead:** a Quantifier that answers prose twice (`parser.rs:56-59` → `Low`) or a movement type it rejects (A2) still records a *successful* call, so the Settings role cell says **Healthy**, `connections_degraded()` (`builders/headers.rs:136-140`) and the banner filter at `builders/headers.rs:159-162` both stay quiet, and no banner appears.
- **Who can see it today:** the tuner can find the raw response in the LLM Messages tab, but nothing points at it.
- **Who would want to know:** the tuner and the player.
- **Trigger:** set the Quantifier model to one that writes prose instead of JSON. Every Cycle keeps the scene, and Settings says the Role is Healthy.
- Label: **known**.

### B2. A failed call that fails to record is lost from the LLM Messages tab
`src/application/llm_recorder.rs:72-76`
```rust
                if let Err(save_error) = (*self.save_fn)(&failure) {
                    tracing::warn!(
                        "[LLM] failed to record the failed attempt for agent '{agent_name}': {save_error}"
                    );
                }
                return Err(error);
```
- **What failed:** the save of the failure record.
- **What the code does instead:** returns the call error anyway (correct for the player) and drops the record. The tuner's table then shows an older, successful call as "the newest attempt" for that Role, so B1's cell can say Healthy while the last call failed.
- **Who can see it today:** only the log file.
- **Who would want to know:** the tuner. Note the asymmetry that makes this a design smell: the *same* save failing for a successful call is fatal (`:85 (*self.save_fn)(&message)?`), so a good Narration is thrown away while a bad attempt vanishes quietly.
- **Trigger:** a storage fault or a full disk in the moment a model call fails.
- Label: **known**.

### B3. An empty Quantifier preset produces a prompt with no instructions at all
`src/application/agents/quantifier/prompt.rs:27-31`
```rust
        let mut prompt = vars.render(
            self.context
                .quantifier_prompt_override
                .as_deref()
                .unwrap_or(""),
```
- **What failed:** the Game's Quantifier preset has no assembled text (every instruction field empty, or the preset is a stub).
- **What the code does instead:** the system prompt becomes just `<available_character_ids>` and `<available_rooms>` with no output shape, so the model has to guess the JSON. `processor` then reads a Low reading and keeps the scene (A1) — the same silent freeze, reached from the tuning side. The Options Agent in the same situation falls back (`options/prompt.rs:36-41 unwrap_or(FALLBACK_SYSTEM_PROMPT)`); the Quantifier has no fallback and no warning.
- **Who can see it today:** the tuner sees the empty System part in the LLM record. Nobody sees a warning.
- **Who would want to know:** the tuner (they just cleared the prompt and nothing told them).
- **Trigger:** duplicate the default Quantifier preset, clear `instructions`, `writing_style` and `output_format`, activate it.
- Label: **known**.

### B4. A preset seed with a mistyped `id` silently writes the preset "default"
`src/bootstrap/run.rs:237-241`
```rust
    let id = seed["id"].as_str().unwrap_or("default").to_string();
    let preset = PromptPreset {
        id: id.clone(),
        name: seed["name"].as_str().unwrap_or("Default").to_string(),
```
- **What failed:** the seed file names no readable `id` (a misspelling such as `"Id"`, a number, a null).
- **What the code does instead:** uses the literal `"default"` for both the id and the name. The bad seed is written as the `default` preset, and a *second* bad seed in the same folder overwrites it. `existing_ids.contains(&id)` (`:259`) then treats it as already seeded and skips it forever.
- **Who can see it today:** the dashboard shows a preset named "Default" in the wrong list. The log says `Seeded … prompt preset: default`, which looks correct.
- **Who would want to know:** the tuner who wrote the seed.
- **Trigger:** rename `"id"` to `"Id"` in `data/prompt_presets/quantifier/*.json`, then boot on a fresh database.
- Label: **known**.

### B5. A storage error while reading an existing preset is treated as "no existing preset"
`src/bootstrap/run.rs:260-261`
```rust
        if let Ok(Some(existing)) = storage.get_preset(&id) {
            let has_content = existing.role.is_some()
```
- **What failed:** the read of the preset the seed would update.
- **What the code does instead:** `Err` and `Ok(None)` take the same branch — `return Ok(())`. The seed is silently not applied and the boot continues. The tuner's freshly edited seed never lands, with no line in the log.
- **Who can see it today:** nobody.
- **Who would want to know:** the tuner and the operator.
- **Trigger:** a database read fault during boot; or a preset row deleted while `existing_ids` still lists it (the id set is read at `run.rs:161-165`, separately from this read).
- Label: **known**.

---

## Group C — the author of a World would want to know

### C1. A Character file with invalid JSON makes the Character vanish from the World
`src/bootstrap/load.rs:154-156`
```rust
            Err(e) => {
                tracing::warn!("Failed to parse Character {}: {e}", path.display())
            }
```
- **What failed:** the parse of one Character card.
- **What the code does instead:** skips the Character and seeds the rest. The World then has no such Character, so a Scenario that names it drops it from the opening scene without a second word (`state/swipe_state.rs:69-70 if let Some(character) = characters.get(character_id)`), it never appears in the prompt, and the Quantifier can never resolve it. `seed_game_data` itself returns Ok, so the boot is clean.
- **Who can see it today:** one warn line at boot. The dashboard shows a smaller cast, which looks like the author's choice.
- **Who would want to know:** the World author.
- **Trigger:** a trailing comma in `data/characters/<world>/*.json`, then boot on a fresh database.
- Label: **known**. Cross-reference: world-03 in the report covers the Scenario-id side at the HTTP form; this is the seed side that produces the missing Character in the first place.

### C2. A World whose `world.json` fails to parse is missing at boot with one warn line
`src/bootstrap/load.rs:102-110`
```rust
    let manifest: WorldManifest = match read_json_file(&world_json) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(
                "Failed to parse world manifest {}: {e}",
                world_json.display()
            );
            return Ok(());
        }
    };
```
- **What failed:** the World manifest parse.
- **What the code does instead:** skips the whole World (its map, its Characters, its Scenarios) and seeds everything else. The boot then either falls back to another World (`run.rs:65-78`, with a warn that names the *requested* World, not the broken one) or stops with "No worlds available". Either way the author's typo is one line in the log, and a later `validate_data.py` run is the only thing that would have caught it earlier.
- **Who can see it today:** the operator via the log; the author only if they read it.
- **Who would want to know:** the author.
- **Trigger:** a comment or a trailing comma in `data/worlds/<key>/world.json`.
- Label: **known**.

### C3. A World update that matches no row reports success
`src/adapters/driven/storage/worlds.rs:313-333`
```rust
                conn.execute(
                    &format!("UPDATE worlds SET {assignments}, updated_at=? WHERE id=?"),
```
- **What failed:** nothing — that is the point. `execute` returns the number of rows changed and this code keeps only the error, so 0 rows is `Ok(())`. The call then runs `Self::write_map_row` and returns Ok.
- **What the code does instead:** the handler answers success, the map row is written (or inserted) for a World row that was not updated, and the form re-renders the old card. The World and its map now disagree.
- **Who can see it today:** the author sees their edit not stick after a success answer. No log line.
- **Who would want to know:** the author.
- **Trigger:** a World row removed between the read at `handlers/worlds.rs:167-171` and the write; a bug rather than a player.
- Label: **known**. Cross-reference: storage-04 in the report names the missing transaction in the same function; the row-count check is a separate gap in the same statement.

### C4. A present Character with no image disappears from the visual sidebar
`src/application/games/view_query.rs:108-116`
```rust
            .filter_map(|id| {
                let character = characters_list
                    .iter()
                    .find(|character| &character.id == id)?;
                let image_path = character.sheet.preferred_image()?.to_string();
```
- **What failed:** the Character has neither `headshot_image` nor `profile_image` (`character.rs:42-46`).
- **What the code does instead:** drops the entry from the returned list. The story, the scene and the Quantifier prompt still hold the Character present, so the sidebar shows fewer people than the story does, with no placeholder and no note.
- **Who can see it today:** the player, as a silently missing portrait. The author is not told which card lacks an image.
- **Who would want to know:** the author (their card is incomplete).
- **Trigger:** a Character card with no image field, present in the opening Scenario.
- Label: **known**.

---

## Group D — the operator and the agent tooling would want to know

### D1. A source file that cannot be read vanishes from the generated AGENTS.md structure
`scripts/generate_structure_index.py:43-44`
```python
    except Exception:
        return None
```
- **What failed:** the read, or the summary extraction, of one source file.
- **What the code does instead:** the file is left out of the Structure block. The pre-commit hook regenerates and stages `AGENTS.md`, the build passes, and `validate_docs.py` checks only that the paths AGENTS.md *names* exist — not that every source file is named. So a module can disappear from the map that the coding agents navigate by, with no error.
- **Who can see it today:** nobody, unless a reader notices a missing entry.
- **Who would want to know:** the operator and every agent that reads `AGENTS.md`.
- **Trigger:** one non-UTF-8 byte in a source file (the read uses `encoding="utf-8"`), or a permission change.
- Label: **known** for the swallow, **inferred** for the trigger.

### D2. A build whose journal row fails to append still exits 0, and the warm costs silently understate
`build.py:1597-1598`
```python
        except Exception:
            pass  # Best-effort bookkeeping; never fail a run over it.
```
- **What failed:** the append to `logs/build_history.txt`.
- **What the code does instead:** the run keeps its exit code and its log, and the journal lacks the row. `AGENTS.md` names that file as the source of the warm costs, so a later reader (a person, or an agent choosing `--test-target`) works from numbers that omit this run. The staleness warning of `verify_coverage.py` reads the same journal.
- **Who can see it today:** nobody. There is no line either way — the success case prints nothing.
- **Who would want to know:** the operator.
- **Trigger:** a read-only or full `logs/` while the build otherwise succeeds.
- Label: **known**.

### D3. A boot that cannot name its own executable silently opens a second database
`src/bootstrap/run.rs:47-51`
```rust
    let db_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::PathBuf::from))
        .unwrap_or_else(|| data_dir.clone());
    let db_path = db_dir.join(format!("chronicler_{}.db", args.port));
    let db_pool = DbPool::new(db_path.to_str().unwrap_or("chronicler.db"))?;
```
- **What failed:** `current_exe`, or the UTF-8 check on the path it gave.
- **What the code does instead:** opens `chronicler_<port>.db` next to the *data* directory, or the relative `chronicler.db` in the working directory. For SQLite that file simply does not exist yet, so it is created empty: the boot seeds a fresh database and the dashboard opens with no Games, no old story and no error.
- **Who can see it today:** the player, as an empty dashboard that looks like data loss. The log says nothing about the path unless the operator reads the seed lines.
- **Who would want to know:** the player and the operator.
- **Trigger:** run the binary from a mount where `/proc/self/exe` is unavailable, or move the binary while the server starts.
- Label: **known** for the fallback; **inferred** for the trigger. Note this is a different lie from gate-04 (two processes, one file): here it is one process, two files.

### D4. Boot seeding failures do not stop the boot
`src/bootstrap/run.rs:54-60`
```rust
    if let Err(e) = ensure_presets(&storage, &data_dir) {
        tracing::warn!("Failed to seed prompt presets: {e}");
    }

    if let Err(e) = seed_game_data(&storage, &data_dir) {
        tracing::warn!("Failed to seed game data: {e}");
    }
```
- **What failed:** the preset seed or the World/Character/Persona seed, for any reason (an unreadable directory, a failed `save_preset`, a bad `worlds/` entry).
- **What the code does instead:** warns and starts the server. `seed_settings` on the next line *is* fatal, so the two halves of the same seeding step disagree about how bad a failure is. The dashboard then runs with a partial catalogue: a missing preset surfaces later as `PresetNotFound` mid-Job (`prompt_preset_service.rs:70`), a missing World as "No worlds available" only when nothing at all seeded.
- **Who can see it today:** the operator, one `warn!` line at boot.
- **Who would want to know:** the operator and the tuner.
- **Trigger:** a read-only `data/prompt_presets` directory, or a Persona file with invalid JSON (`load.rs:178` propagates `?`).
- Label: **known**.

---

## Group E — nobody: the honest answer

These swallow an error, but no one is served by reporting it, or the reporting exists
elsewhere. No detail, deliberately.

- `src/main.rs:14` `dotenv::dotenv().ok();` — a missing `.env` is the normal case.
- `src/adapters/driving/http/settings/handlers/settings.rs:348` and `:393` — the name in an error answer falls back to the id.
- `src/adapters/driving/http/worlds/handlers/worlds.rs:230` — same, for the World delete answer.
- `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs:233` — same, for the preset delete answer.
- `src/adapters/driving/http/view_models.rs:266-267` — the pretty-print of a stored JSON blob falls back to the raw text.
- `src/bootstrap/load.rs:172` `unwrap_or("")` — a Persona file name that is not UTF-8 is skipped; the file cannot be named, so it cannot be reported usefully.
- `src/adapters/driven/llm/transport/utils/response.rs:135` `response.text().unwrap_or_default()` — a non-success status whose body cannot be read; the status is still named.
- `assets/dashboard-screen.js:360-362` `.catch((error) => { console.error("The screen ask failed:", error); })` — the poll chain keeps itself alive; every error the ask can raise is already handled and shown inside `runAsk`.
- `scripts/generate_docs_index.py:30-31` `except OSError: pass` — a doc with no H1 is indexed without a title.
- `scripts/build_slot.py:65,75`, `build.py:886` (`_stamp_target`), `build.py:1489` — machine-local optimisations and bookkeeping with no reader.
- `src/application/games/active.rs:32,38` `self.game.lock().unwrap_or_else(|e| e.into_inner())` — a poisoned pointer mutex is reused silently. It needs a panic under the hold to reach, and the panic has already failed loudly.
- `src/application/generation/job_tasks.rs:16` `handles.retain(|handle| !handle.is_finished());` plus `:47` `tracing::debug!` — a Job task's `JoinError` is discarded at debug level. This is the mechanism behind gate-05 in the report, so it is not a new finding.

---

## Already known (not re-reported)

One line each: Character cards stored inside each Swipe state; World settings copied into
a Game at creation; data files never re-read into an existing database; default host
duplicated in `src/utils/cli.rs` and `build.py`; typed Input gets no Quantifier analysis
but Impersonate Input does; Connection form parses provider with `ALL` but offers
`SELECTABLE`; `finish_reason=length` Narration shown as complete; HTTP status errors all
shown as "could not be reached"; Quantifier lists map Rooms only.

And from `.scratch/invariant-audit/assets/REPORT.md`: llm-01 (a Quantifier answer of
names, or a missing field, empties the scene — `parser.rs:33-42`,
`game_rules.rs:70`), llm-02/llm-03 (destination and exits), llm-05 (Agent prompts
overflow), llm-07 (`&error_body[..500]`), llm-08/llm-09, world-03 (`swipe_state.rs:69-70`
drops an unknown Scenario Character), world-04 (Trigger order), gate-01/gate-05/gate-08,
storage-02/storage-03/storage-11, http-09 (`NarratorMode::parse_or_default` in the World
write paths), and the `ensure_valid_swipe_index()` repair at `mappers/message.rs:72`.

## Dropped, and why

I opened 24 further candidates and dropped 18 of them:

- **5 were already checked.** The `map_err(|_| …)` sites (4) each build a named error and
  propagate it. `screen/handlers/parts.rs` + `screen.rs:47-53` refuse an unknown part name
  with a 400, and `morphParts` can therefore never be given a part the answer lacks.
  `answer_words.rs:276` (`a route with no words row returns the raw body`) is guarded by
  the route table test in `tests/http/answer_words.rs:625-630`.
- **6 are loud, not silent.** The storage mappers (`mappers/message.rs`,
  `mappers/llm_message.rs`) propagate every parse and datetime error; only
  `#[serde(default)]` fields degrade, and those are storage-03. `narrator.rs:152-158`
  refuses empty model text. `options/orchestration.rs:44-66` turns a zero-parse answer
  into an error after one retry, and the Job stores it as an Options failure with the raw
  text in the disclosure (`dock.rs`, `job.rs:363-366`). `app_state.rs:96-102` and
  `render_failure_banner` propagate a read failure. `autoplay/driver.rs:86-89` warns and
  the run ends with the holder released by `Drop`.
- **4 are a design choice with a stated reason in the file.** `budget.rs:16-19` (an empty
  field is `DEFAULT`), `run.rs:246-251` (`is_default` defaults to true),
  `bootstrap/run.rs:222-231` (invalid `allowed_modes` allows all modes, with a warn), and
  `settings.rs:372-376` (`max_tokens_for` clamps to the ceiling — the clamp chain ends in
  the already-known `finish_reason` truncation).
- **3 change nothing a reader could act on.** `start_server`'s `HeaderValue::from_str`
  guards (`screen.rs:89-95`) can only drop a header for a value the code builds itself;
  `plumbing.rs:32,52` (`PRAGMA user_version` unreadable → 0) is the documented
  storage-13 case; `narrator.rs:186-189`'s `unwrap_or("")` for the previous text only
  affects an event Narration whose history is empty, which the plan prevents.
