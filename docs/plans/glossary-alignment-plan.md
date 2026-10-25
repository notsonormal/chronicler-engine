# Glossary alignment: the new `CONTEXT.md`, and the code, copy and docs that follow it

> **Status:** Done 2026-10-25. Phases 1, 2 and 3 and the after-plan review are complete. The full gate passed on `logs/build_20261025_104552.log`.
> **Scope:** Three phases, run one after the other, each by its own subagent. Each phase starts only after the one before it passes its gate.
> **Source:** the glossary audit of 2026-10-23. The new glossary text is now `CONTEXT.md`.

## Summary

A glossary audit found that most of the confusion comes from `CONTEXT.md` itself: avoided words that other entries use, terms with no entry, entries that disagree, and one wrong behaviour sentence. The owner agreed a new glossary with stricter rules. This plan puts that glossary in place (Phase 1) and then aligns the code, the product copy and the docs with it. Phase 1 holds the cheap, internal and player-visible fixes. Phase 2 holds the large Character rename and the names that are stored or that the model sees, which need compatibility work.

## Decisions (settled; do not reopen)

- **Entry test.** A term gets an entry only when it is shared (writers in more than one module need it), contested (a competing word is in the repo or its history) and not already general.
- **Plain words.** Prefer a plain word, or a plain word with an entry name in front of it ("Options failure"). Make up a word only when every plain word clashes with an entry, a programming meaning or a product word. A code name that is a plain description needs no entry and no rename.
- **When to add an entry.** When drift is observed, when a new concept lands and its name crosses a module boundary, or when another entry cannot be defined without it. A planned term waits in its plan.
- **Placement.** An entry goes under "Shown in the product" when the dashboard, the World files or the prompt presets use its word, in any word form. Otherwise it goes under "Used only in the code and docs".
- **Screen regions get no entry.** `docs/diataxis/reference/frontend/dashboard.md` names them.
- **Character, not NPC.** Character and Persona are the convention of LLM chat applications.
- **The glossary governs model-facing names too**, such as prompt tags and JSON field names. They change last, with compatibility.
- **"transaction"** is the word for a group of writes that lands whole. One save unit was one `Storage::transaction` call; the term "Save unit" goes.
- **"entry" and "line"** keep their plain meaning. `StoryWindowEntry` keeps its name.
- **Plan review (2026-10-23).** The Connection's code name is `LlmConnection`. The `/status/generating`, `/status/reset-generating` and `/debug/is_generating` routes are renamed to `busy`. The `/debug/state` JSON values change to `"Busy"` and `"Failed"`, with no compatibility.
- **Failures.** The status shows "the failure" of a Game; the dock shows the "Options failure". "error message" and "information message" keep their plain meaning in UI copy and UI docs.

## Rules for both phases

- Read `AGENTS.md`, `CODING_STANDARDS.md` and `tests/AGENTS.md` first.
- Do not commit. Do not delete or revert files that you did not change.
- A rename changes no behaviour. Where a test name or a test string changes, the test asserts the same thing as before.
- Build only through `build.py`, with a bash timeout of 1200 s. Read the result from the newest `logs/build_*.log`.
- Docs follow `docs/AGENTS.md`: Simplified Technical English, symbols and not `path:line`.
- Line numbers in this plan are from 2026-10-23. Find each site again with `rg` before you change it.
- Do not rename the items in "Out of scope".

## Implementation

### Phase 1: The glossary, wrong statements, product copy, doc words and small renames

Subagent: `implementer`. Gate: `python build.py` (the full gate, including the browser tier).

- [x] #### Task 1.1: Replace the glossary
  - [x] ##### SubTask 1.1.1: Replace `CONTEXT.md` with the draft glossary text, unchanged.
  - [x] ##### SubTask 1.1.2: `rg -n "CONTEXT.md" AGENTS.md docs .agents .pi` and check that no doc cites an entry that the new glossary removes (Narrative, Story window, Tail, Offered set, Options run, Dock record, Notice, Failure line, Save unit, Holder, Job entry, Pre-analysis state, Start state, Encounter record, Story log). A doc may keep the plain words; it must not present them as glossary terms.
- [x] #### Task 1.2: Fix wrong statements
  - [x] ##### SubTask 1.2.1: `docs/diataxis/reference/storage.md` (near line 46) says a save unit is "the record writes of one request". One request runs several transactions. State that one `Storage::transaction` call is one transaction, and that a Job writes through several.
  - [x] ##### SubTask 1.2.2: `docs/diataxis/reference/narrative/options.md` (near line 14) says the Options agent runs at "the Options hook". It runs at the end-of-Job hook (see `agent_system.md`). Also fix the module doc in `src/application/agents/options/agent_tests.rs:1`.
  - [x] ##### SubTask 1.2.3: `docs/AGENTS.md` (near line 235) cites `PhaseError` as an example of a seam identifier. `src/` has no `PhaseError`. Use a type that exists.
  - [x] ##### SubTask 1.2.4: `data/schemas/character.schema.json:4` says "a character (Player or NPC)". A Character is a person in a World whom the player does not play; the player plays a Persona. Run `python build.py validate-data` after the edit.
- [x] #### Task 1.3: Product copy
  Change each string, then the tests that assert it (`rg` the old string in `tests/` and `assets/`).
  - [x] ##### SubTask 1.3.1: `assets/index.html`: the label "Command" (line 132) → "Action"; the placeholder "Enter command..." (line 137) → "Enter an action...". Keep the element id `command-input`.
  - [x] ##### SubTask 1.3.2: `src/adapters/driving/http/templates.rs:79`: the Options button title and `aria-label` "Regenerate options" → "New options".
  - [x] ##### SubTask 1.3.3: `src/adapters/driving/http/worlds/templates/worlds.rs:97`: "Auto-generate options after each turn" → "Generate options after each narration".
  - [x] ##### SubTask 1.3.4: `src/adapters/driving/http/builders/headers.rs:31-32`: "the turn was left unnarrated" → "no narration was written"; "using fallback NPC IDs" → words that state the same effect with "Characters" (read `quantifier/utils/orchestration.rs` near line 143 for what the fallback does).
  - [x] ##### SubTask 1.3.5: `src/adapters/driving/http/templates.rs:48`: "No Location Image" → "No Room Image" (it pairs with "Default Room Image" in `worlds.rs:87`). Keep CSS class names.
  - [x] ##### SubTask 1.3.6: `src/adapters/driving/http/prompt_presets/templates/prompt_presets.rs:56` "NPC presence" → "which Characters are present"; line 130 "pickable next-action suggestions" → "Options to pick from".
  - [x] ##### SubTask 1.3.7: `assets/dashboard-action-area.js:240` "Quantifying scene..." → "Quantifying..."; `assets/dashboard-slash-menu.js:6` "Suggest actions (generate pickable options)" → "Offer Options to pick from".
  - [x] ##### SubTask 1.3.9: Update the tests and fixtures that hold the old strings: `tests/http/failure_display.rs` ("using fallback NPC IDs", "fallback NPC IDs used") and the browser stub fixture `tests/test_utils/stub_fixtures/prompt_presets.html` (the preset description). Find the way that fixture is made before you edit it; if a test or script writes it, run that instead of a hand edit.
  - [x] ##### SubTask 1.3.8: Keep the button text "Generating…". It is a plain verb for what the engine does, not the name of the busy state.
- [x] #### Task 1.4: Doc and comment words
  - [x] ##### SubTask 1.4.1: "save unit" → "transaction" in `docs/diataxis/` (about 27 lines, including the heading "## Save units" in `explanation/storage_design.md`) and in the comments of `src/application/generation/gate.rs` and `src/application/pipeline/job.rs`. The heading "## Transactions" in `storage.md` stays.
  - [x] ##### SubTask 1.4.2: "live state" → "current state" (`storage_design.md`, `game_flow.md`); "session" for a Game → "Game" (`storage.md`, the module doc of `src/domain/model/game.rs`); "forensics record" → "LLM message" (`storage.md`; the `AGENTS.md` structure summary comes from the module docs of `src/domain/model/llm_message.rs` and `src/application/llm_recorder.rs`, so change those docs and run `python scripts/generate_structure_index.py`, because no commit runs the hook); "label" for the Display name → "Display name" (comments in `game.rs` near lines 30 and 174).
  - [x] ##### SubTask 1.4.3: "Options hook", "Options regeneration" and "suggestions" for Options in `docs/` and doc comments, such as `src/domain/model/action.rs:17` ("Regenerate the pickable next-action set").
  - [x] ##### SubTask 1.4.4: "turn" for a Job or a Cycle: `src/domain/model/world.rs:29`, the comment in `assets/styles.css:588`, and the doc comments that `rg -n -i "\bturn\b" src assets` finds. `ScriptedModel::turn` is out of scope.
- [x] #### Task 1.5: Small internal renames
  - [x] ##### SubTask 1.5.1: `GenerationStatus::Generating` → `Busy` and `GenerationStatus::Error` → `Failed`, with every match site. `GenerationStatus` is a view of the application `Status` (`src/application/games/view_query.rs`), which already has `Status::Busy`; it is not stored (the storage adapter reads only `GenerationFailure`). It is serialized in the `/debug/state` JSON (`DebugStateView.generation_status`), so the values `"Generating"` and `"Error"` change to `"Busy"` and `"Failed"`: update the tests that assert them (`rg -l '"Generating"|"Error"' tests`: `tests/http/dashboard.rs`, `tests/http/autoplay/run.rs`, `tests/http/requires_migration/debug.rs`, `tests/browser/dashboard.rs`, `tests/browser/stub/dashboard.rs`). Rename the routes `/status/generating` → `/status/busy`, `/status/reset-generating` → `/status/reset-busy` and `/debug/is_generating` → `/debug/is_busy`, with their handlers, the `hx-get` / `hx-post` attributes in `assets/index.html`, the comment in `assets/dashboard-action-area.js:261`, the tests, and the spec `docs/specs/failure_display.md` (it names `GET /status/generating` in its scenarios). Regenerate `docs/diataxis/reference/frontend/http_routes.md` with `scripts/extract_http_routes.py`, then run `python build.py py-tests`.
  - [x] ##### SubTask 1.5.2: `Notice` (in `src/domain/model/dock.rs`) → `OptionsFailure`, `DockNoticeView` → `OptionsFailureView`, the field `DockRecord.notice` → `failure`, `put_dock_notice` → `put_options_failure` and `TxWrite::PutDockNotice` → `PutOptionsFailure`. Storage keeps its names: the SQL table `game_docks` keeps its column `notice`, and the JSON in that column keeps the fields `reason` and `raw`. So no migration and no serde attribute is needed. Keep the CSS classes `options-notice*` and the element `restore-notice`.
  - [x] ##### SubTask 1.5.3: `Generation::UserRegen` (`src/domain/model/generation.rs`) and `Kind::UserRegen` (`src/domain/model/agent.rs`) → `InputRetry`, and the test enum `JobKind::TypedInputRetry` → `InputRetry`. Not `Retry`: a Retry of a Narration is `Generation::Narration`, so `Generation::Retry` would name only one case of a Retry. Change the comments that say "user regen" (`generation.rs`, `gate.rs`) to "the Retry of a typed Input".
  - [x] ##### SubTask 1.5.4: `NarrativeFailure` → `NarrationFailure` and `NarrativeLogTemplate` → `NarrationLogTemplate`. `NarrativePerspective` and `NarrativeTense` stay (another sense: posture). The folder `docs/diataxis/reference/narrative/` stays.
  - [x] ##### SubTask 1.5.5: Test names with avoided words: `*_sets_error_status_http` → `*_sets_failed_status_http`; `test_regenerate_*` in `tests/http/options.rs` → `test_new_options_*`. `tests/STRATEGY.md:139` ("each error status") stays: it means an HTTP status code.
- [x] #### Task 1.6: Gate
  - [x] ##### SubTask 1.6.1: `python build.py clippy`, then `python build.py integration`, then the full `python build.py`.
  - [x] ##### SubTask 1.6.2: `rg -n -i "save unit|enter command|regenerate options|each turn|no location image|npc presence|quantifying scene|GenerationStatus::(Generating|Error)|UserRegen|NarrativeFailure" src tests assets docs/diataxis data` gives no hits.
  - [x] ##### SubTask 1.6.3: Report each file changed and any site that you left, with the reason.

### Phase 2: Character in the code, and the stored and model-facing names

Subagent: `strong-implementer` (the change must keep the storage, the World files, the prompts and the parsers consistent). Starts after Phase 1 passes its gate. Gate: `python build.py`.

Compatibility rules for this phase:
- **Stored records** (the Swipe state, the dock record, the settings in each database): rename the Rust name and keep the stored key with `#[serde(rename = "old_key")]`. `EventRecord.header_name` is the precedent. No migration.
- **Author keys in World files** (`"npcs"` in a Room): accept both keys with `#[serde(alias = "npcs")]`, write `"characters"` in the seeded World files and in `data/schemas/world.schema.json`, and test that a World file with the old key still loads.
- **Model-facing names** (prompt tags, the JSON fields that the model writes): the parser accepts both the old and the new name, as the Options parser already "accepts both seeded prompt shapes". Change the seeded presets in `data/prompt_presets/`. Preset copies that players edited keep the old words, so the old names must keep working.

- [x] #### Task 2.1: Character in the domain and application code (about 700 lines in 75 files)
  - [x] ##### SubTask 2.1.1: `NpcCard` → `CharacterCard`; `npc_id` → `character_id`; `npcs_in_area` → `characters_present`; `NpcEvent` / `NpcEventList` → `CharacterTransition` / `CharacterTransitions` (the module doc of `quantifier.rs` already calls them "Character transitions"); `NpcEncounterState` → `Encounter`; `NpcEncounterLog` → `Encounters` (it holds the current counts, not a history); the field `SwipeState.npc_encounter_log` → `encounters`. Keep every stored key (compatibility rule 1).
  - [x] ##### SubTask 2.1.2: The other `npc` / `Npc` identifiers that `rg -n "\bNpc[A-Z]|npc_|npcs_|_npc\b" src tests` finds, including test helpers (`make_npc`, `triggered_npc`) and test names (`test_quantifier_npc_fires_trigger_http`).
  - [x] ##### SubTask 2.1.3: The `"npcs"` key of a Room in the World files (compatibility rule 2): the Rust Room type, `data/schemas/world.schema.json`, the seeded `data/worlds/*/world.json`, and `scripts/validate_data.py` (line 132 reads `room.get("npcs", [])`; it must read both keys), with a Python test for each key. Run `python build.py validate-data` and `python build.py py-tests`.
  - [x] ##### SubTask 2.1.4: Doc comments and `docs/diataxis/` text that say "NPC" for a Character.
- [x] #### Task 2.2: Event record
  - [x] ##### SubTask 2.2.1: `SwipeState.trigger_context` → `event_record`, with every use (about 43 lines in 15 files). Keep the stored key `trigger_context`. Change "trigger context" in docs to "Event record".
- [x] #### Task 2.3: Connection
  - [x] ##### SubTask 2.3.1: `LlmProviderConfig` (`src/domain/model/settings.rs`; id, name, provider, model, key, address) is the Connection: rename it to `LlmConnection` (about 119 lines in 26 files). Not `Connection`: the storage adapter imports `rusqlite::Connection`. It is serialized in the settings, so keep the stored keys. `LlmBackendType` (OpenRouter, DeepSeek, Mock, Ollama) is the provider of a Connection, not a Connection; it stays.
- [x] #### Task 2.4: Model-facing names (compatibility rule 3)
  - [x] ##### SubTask 2.4.1: The Quantifier JSON field `npcs_in_room` → `characters_present`: the parser in `src/application/agents/quantifier/utils/parser.rs` accepts both; the seeded preset `data/prompt_presets/quantifier/default.json` asks for the new one and says "Characters", not "NPCs".
  - [x] ##### SubTask 2.4.2: The prompt tags `<KnownNpcs>` → `<KnownCharacters>`, `<NpcsInRoom>` → `<CharactersPresent>`, `<PlayerCharacter>` → `<Persona>` in `src/application/prompting/assembler.rs`. Only the assembler writes these tags, and no seeded preset names them, so they need no compatibility code. A preset copy that a player edited can name them; such a copy keeps working, but its words no longer match the tags.
  - [x] ##### SubTask 2.4.3: The prompt copy "Current Location:" → "Current Room:" (`assembler.rs` near line 198) and "Trigger event:" → "Trigger direction:" (`narrator.rs` near line 181). Update the tests that assert the prompt text.
- [x] #### Task 2.5: Gate
  - [x] ##### SubTask 2.5.1: `python build.py clippy`, `python build.py integration`, then the full `python build.py`.
  - [x] ##### SubTask 2.5.2: Tests show that a Swipe state, a dock record and the settings stored with the old keys still load, that a World file with `"npcs"` still loads, and that a Quantifier answer with `npcs_in_room` still parses.
  - [x] ##### SubTask 2.5.3: `rg -n -i "\bnpc" src tests assets data docs/diataxis` gives hits only at the compatibility sites (the serde attributes, the alias, the parser's old name and its tests). Report each one.

### Phase 3: Owner decisions after the review (2026-10-25)

Subagent: `implementer`. Gate: `python build.py`.

- [x] #### Task 3.1: `EngineError::Narrative` → `EngineError::Narration`, and its message "Narrative generation error" → "Narration generation error" (`src/error.rs`), with every use and every test that matches the text.
- [x] #### Task 3.2: In `src/application/generation/gate.rs` (private names): `Units` → `TxOpener`, `create_unit` → `create_in_tx`, `accept_unit` → `accept_in_tx`, `Self::unit` → `Self::in_tx`, and the `units` parameters and variables → `opener`.
- [x] #### Task 3.3: The prompt line "No Characters are present in this location." → "No Characters are present in this Room." (`src/application/prompting/assembler.rs`), with its tests; the module doc of `src/domain/model/map.rs` "Map and location data structures" → "Map and Room data structures", then `python scripts/generate_structure_index.py`. The CSS classes `location*` stay.
- [x] #### Task 3.4: `StartingScenario.characters` → `characters_present` (`src/domain/model/scenario.rs`), with `#[serde(rename = "characters", alias = "npcs")]`, so the author key stays `"characters"` and the stored `worlds` rows do not change. The compatibility test `world_tests::test_world_manifest_with_the_old_scenario_key_still_loads` still passes.

## What already exists (reuse it)

- Keeping a stored key under a new Rust name: `EventRecord.header_name` with `#[serde(rename = "name")]` (`src/domain/model/state/event_record.rs`).
- A parser that accepts two shapes of model output: the Options parser (`src/application/agents/options/utils/parser.rs`).
- `Status::Busy` in `src/application/games/view_query.rs`: the name that `GenerationStatus::Busy` matches.
- `scripts/extract_http_routes.py` (the routes reference) and `scripts/generate_structure_index.py` (the `AGENTS.md` structure block).
- The browser stub fixtures under `tests/test_utils/stub_fixtures/`.

## Failure modes

| Change | What fails if it is wrong | Where the plan catches it |
|---|---|---|
| A serde rename on a stored field is missing or wrong (Phase 2) | Old Swipe states or settings do not load; Games or Connections disappear | SubTask 2.5.2: a test loads a record with the old keys |
| The alias for `"npcs"` is missing (Phase 2) | A World file that an author wrote with the old key loses its Characters, or fails to load | SubTasks 2.1.3 and 2.5.2 |
| The Quantifier parser accepts only the new field (Phase 2) | A preset copy that a player edited makes the Quantifier fail on every call | SubTasks 2.4.1 and 2.5.2 |
| A renamed route is missed in `assets/index.html` or `assets/*.js` (Phase 1) | The status stops updating, or reset does nothing; the server returns 404 | The browser tier of the full gate (SubTask 1.6.1) |
| A test string is changed with the product string, but the assertion changes meaning (both phases) | A test passes that should fail | Rule: a rename changes no behaviour; the reviewer of each phase checks the changed assertions |
| A fixture is edited by hand when a script writes it (Phase 1) | The next run of the script undoes the edit | SubTask 1.3.9 |

## Out of scope

- `ScriptedModel::turn` (a scripted reply, not a play turn).
- The modules `http/history/` and `http/chat_window/` and their routes (a frontend naming question, not a glossary one).
- Code names that are plain descriptions: `StoryWindow`, `StoryWindowEntry`, `Tail`, `StoryEnd`, `DockRecord`, `OptionsRun`, `transaction.rs`, `Tx`.
- The `Record` labels "the start state" and "the saved state".
- `LlmBackendType` (the provider of a Connection).
- The SQL table `game_docks` and its column `notice`.
- The plain verbs in the status labels ("Generating…", "Generating narration...", "Generating options...").

## Terms that wait

No entry now. Each one gets an entry when a competing word appears, or when another entry cannot be defined without it.

| Term | Why it waits |
|---|---|
| Retrigger | No competing word. "Event retry" is a Retry of an event Narration, which is another act. |
| generation | No competing word found. |
| Posture | Its parts (perspective, tense) do not compete with it. |
| Prompt preset | No competing word found. |
| Reset | No competing word found. |
| Swipe switch | Gets an entry if "Restored swipe N" turns out to be the same act. |
| analysis | No entry needs it. |
| Start state | No competing word in the repo or its history. The fact that a Retry starts from the state before the last Message belongs in `game_flow.md`. |
| Pre-analysis state | Its name uses "analysis", which has no entry. The code says "the saved state". |
| Active Game | `ActiveGame` against `CurrentGame`: one concept or two? Check first. |
