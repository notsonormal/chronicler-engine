# Research 10: where the docs and the code meet, and where they do not

Ticket: [10-research-synthesis.md](../issues/10-research-synthesis.md), last question: "Where do the docs (ticket 09) name a concept that the code does not follow, or miss one that it does?"

This asset records what both sides show. It rules on nothing. A mismatch is a question, not a verdict.

## 0. How this was built

**Inputs.**

- `research-09-docs.md` (74 docs concepts C1–C74, 22 meeting points MP1–MP22, 13 shapes S1–S13, 12 questions Q1–Q12) and the `## Answer` of ticket 09.
- The code assets `research-02-domain.md` … `research-08-test-bodies.md`, and the `## Answer` of tickets 02–08. Short names in this asset: **A02** domain, **A03** application, **A04** driven adapters, **A05** driving adapters, **A06** composition and cross-cutting, **A07** test infrastructure, **A08** test bodies.
- `docs/diataxis/` (all files named below), `src/`, `tests/`, `Cargo.toml`, `arch-lint.toml`, and a few files under `.scratch/` where a doc points to them. I did not open `research-01-lens.md`.
- `CONTEXT.md` for domain words (Game, Swipe, Job, job entry, run hold, save unit, Narration, Quantifier).

**Method.**

1. I read asset 09 first, then each code asset, one at a time.
2. For each hazard that asset 09 and ticket 09 list, I opened the doc line and the code file, and wrote what each shows.
3. For each concept that a code asset names, I searched `docs/diataxis/` with `rg` for the name and for synonyms (the terms are in each row of section 3).
4. For section 4, I ran a script that extracts every `path:line` citation of asset 09 that points into the six edited files (153 citations), printed the current line next to the claim, and checked each quoted string against the current file. I then read the `git diff` of the six files. The scratch scripts are in `tmp/` (`cite09.py`, `show09.py`, `quotes09.py`; git-ignored).

**What I sampled, and what I did not do.**

- I did not re-read all of `src/`. For code evidence I used the line numbers of the code assets. I opened the code myself for every row of section 2, and for rows of section 1 that I mark "read". A row without "read" takes its code evidence from the asset and has the label of the asset.
- I checked citations into the six edited docs for every one of the 153. I checked citations into the other 17 docs only where a row in section 2 uses them (about 25 lines).
- I did not run the engine or the tests. A claim about run-time behaviour is "inferred".

**Labels.** *known* = I read the lines on both sides. *inferred* = one reading step between the lines and the claim. *guessed* = weak evidence.

**Citation form.** A doc path with no prefix is relative to `docs/diataxis/`. A code path with no prefix is relative to the repo root.

**Counts.** Section 1: 34 rows. Section 2: 23 questions. Section 3: 25 rows. Section 4: 153 citations checked (in 77 rows of asset 09), 0 broken by the edits, 3 quotes imprecise (details there). Section 5: 14 questions.

## 1. Docs name it, and the code follows it

"Docs concept" uses the IDs of asset 09. "Held" says how far the code follows the doc text. A row whose "Held" cell says "in part" has a matching question in section 2 (the number is in the cell).

| # | Concept | Doc evidence | Code evidence | Code assets | Held | Label |
|---|---|---|---|---|---|---|
| 1 | Ports and adapters for the LLM provider and the text checker; the core owns the trait | `reference/architecture_system.md:19-26`; `explanation/architecture.md:65` (C1, C2) | `src/application/ports/llm_provider.rs:44`; `src/application/ports/text_checker.rs:64`; `src/adapters/driven/llm/providers/ollama.rs:62` | A03 #1, A04 #1, A06 C1 | In part: storage has no port (2.1) | known (read) |
| 2 | Concrete type over trait; one `Backend` enum with one variant; `BackendKind` wraps one `Backend` | `explanation/rust_idioms.md:8,12`; `explanation/storage_design.md:12-16`; `reference/storage.md:12-14` (C5, C6, C7) | `src/adapters/driven/storage/core.rs:20-24`, `:74-86`; `src/domain/` holds no `trait` (A02 search) | A04 #9, #12, #13; A02 #3, #17 | Everywhere in storage | known |
| 3 | Composition root with plain constructor injection | `reference/architecture_system.md:16,23`; `explanation/architecture.md:46,60-62` (C3) | `src/bootstrap/wiring.rs:98-177`; `src/bootstrap/run.rs:28-42` | A06 C1, C2; A03 #3 | Everywhere; boot builds a second small graph (A06 MP-6) | known |
| 4 | Single writer: one owner per record kind, with a capability handle | `reference/architecture_system.md:44-59`; `reference/storage.md:49` (C10, C12) | `src/application/generation/gate.rs:160-165`, `:246-250`, `:674-681`; `tests/infrastructure/guardrails/walker_rows.rs:45-64` | A03 #8, #9; A04 #21; A07 G7, T12 | In part: config and display-name writes (2.6) | known (read `gate.rs:405-420`) |
| 5 | Unit of Work: the save unit lands whole or not at all | `reference/storage.md:44-54`; `reference/game_flow.md:8` (C13) | `src/adapters/driven/storage/transaction.rs:280-328`; `src/application/generation/gate.rs:168-174`, `:806-817` | A03 #12; A04 #20 | Story, dock, failure-line writes | known |
| 6 | A write checks the job id; a stop removes the job entry; a late save lands nowhere | `reference/game_flow.md:47-48,98`; `explanation/rust_idioms.md:34-36` (C15, C16) | `src/application/generation/gate.rs:700-703`, `:511-514` | A03 #10 | Every `JobWriter` save | known |
| 7 | No hold of the registry spans a model call; short critical section | `reference/game_flow.md:49`; `explanation/storage_design.md:46,54`; `reference/coding_standards/guardrails.md:66,71` (C14) | `src/application/generation/gate.rs:145-146`; `tests/infrastructure/guardrails/walker_rows.rs:19-26`, `:103-116` | A03 #11; A07 G9, G10 | `gate.rs` | known |
| 8 | Read handles and `GameViewQuery` for reads; `Tx` for writes | `reference/storage.md:39-42,48`; `reference/game_flow.md:94,118` (C18) | `src/adapters/driven/storage/docks.rs:19-43`; `src/adapters/driven/storage/stories.rs:42-186`; `src/application/games/view_query.rs:2,41-45` | A03 #29; A04 #23; A05 C30 | Story and dock records (A03 M8: `GameCatalogue` also reads) | known |
| 9 | A `Tx` serves one Game; the dock record is a table of its own | `reference/storage.md:48`; `explanation/storage_design.md:50` (C19) | `src/adapters/driven/storage/transaction.rs:44-53`; `src/adapters/driven/storage/utils/records.rs:9-26`; `src/domain/model/dock.rs:6-7` | A02 #7; A04 #22 | Story writes | known (read `dock.rs`) |
| 10 | Pure plan and rule functions; the Job loads, calls and returns | `reference/game_flow.md:22,78,145`; `reference/architecture_system.md:12` (C20) | `src/domain/model/utils/game_rules.rs:41-83`; `src/domain/model/generation.rs:228-273`; `src/application/pipeline/job.rs:180-183`, `:334` | A02 #1; A03 #7; A04 #38 | Mostly; effects leak (2.13) | known |
| 11 | Between steps pass ids; each step loads what it uses | `reference/game_flow.md:78`; `reference/coding_standards/guardrails.md:70` (C22) | `src/application/pipeline/job.rs:180-183`; `tests/infrastructure/guardrails/walker_rows.rs:79-101` | A03 #24; A07 G9 | `job.rs` (it loads the World bundle five times, A03 #24) | known |
| 12 | Current state is derived from the active Swipe; each Swipe holds its own Swipe state | `reference/storage.md:121`; `reference/game_flow.md:104-106,110,114` (C23, C24) | `src/domain/model/tail.rs:50-68`; `src/domain/model/state/swipe_state.rs:16-24`; `src/domain/model/message.rs:25-37` | A02 #9, #10 | Mostly; `StoryWindow.retrigger_allowed` is stored (A02 Q8) | known |
| 13 | Manifest type converts to a runtime card; the file pointers are dropped | `reference/startup.md:80`; `explanation/storage_design.md:26` (C27) | `src/bootstrap/load.rs:120-121`; `src/domain/model/world.rs:35-60`, `:80-95` | A02 #20; A04 #4; A06 SN-6 | Worlds | known |
| 14 | Singleton settings row; edits run as read-modify-write under one storage acquisition | `explanation/rust_idioms.md:20-24`; `reference/storage.md:65` (C30) | `src/adapters/driven/storage/utils/plumbing.rs:232`; `src/adapters/driven/storage/settings.rs:28`, `:106-117` | A04 #19, #27; A06 C18, C19 | Settings | known |
| 15 | Typed error with context; a rusqlite error becomes `EngineError` only through `db_context` | `reference/storage.md:76`; `explanation/storage_design.md:66-68` (C33) | `src/error.rs:207-217`; `src/adapters/driven/storage/games.rs:20,23-26` | A04 #47; A06 C30 | Storage | known |
| 16 | Poison recovery with `into_inner()` | `explanation/rust_idioms.md:28-30`; `explanation/architecture.md:78` (C35) | `src/application/generation/gate.rs:152-155`; `src/adapters/driven/storage/db.rs:27-32`; `src/application/games/active.rs:28` | A03 #28; A04 #48; A06 C45 | In part: three spellings, test fakes use `expect` (2.7) | known (read) |
| 17 | No panics in production code, enforced by lint | `reference/coding_standards/guardrails.md:12,17-18,24` (C36) | `src/lib.rs:4-23`; five `expect` on static regexes (A03 #16) | A03 #16; A06 C34; A04 #49 | Everywhere, with named exceptions | known |
| 18 | Accept now, run in the background, poll for status | `reference/game_flow.md:8,12`; `explanation/dashboard_design.md:20` (C37, C40) | `src/adapters/driving/http/action/handlers/actions.rs:42-45`; `src/adapters/driving/http/layout/handlers/endpoints.rs:70-89` | A05 C8 | Job and Autoplay routes | known |
| 19 | Polling, not push. WebSocket and SSE were considered | `explanation/dashboard_design.md:20,25` (C40) | No WebSocket or SSE code in `src/` (A05 Q1, checked by the coordinator; `AGENTS.md` still says "HTTP/WebSocket") | A05 Q1 | Whole area | known |
| 20 | Synchronous services on the blocking pool; no `std::thread::spawn` in `src/` | `explanation/rust_idioms.md:12,14-18`; `explanation/architecture.md:77` (C38) | `src/application/pipeline/job.rs:85-88`; `src/application/autoplay/driver.rs:54-56`; `rg "thread::spawn" src` finds none (read) | A03 #27; A04 #14; A06 C6; A07 G18 | Whole crate | known |
| 21 | The server renders HTML fragments; thin client | `reference/frontend/dashboard.md:12-14`; `explanation/dashboard_design.md:18` (C39) | `src/adapters/driving/http/builders/router.rs:62-90`; `src/adapters/driving/http/templates.rs:16` | A05 C5 | Everywhere; JSON only on `/debug/*` | known |
| 22 | Busy lives in memory; the failure line lives on the Game record; one process | `explanation/architecture.md:69,79`; `explanation/two-state-channels.md:8-20`; `reference/game_flow.md:40` (C41-C43) | `src/application/generation/gate.rs:128-157`; `src/domain/model/game.rs:44`; `src/application/games/view_query.rs:204-218` | A03 #8; A02 MP15 | In part: the status reads the two stores in two steps (2.12) | known (read `view_query.rs`) |
| 23 | An Agent hook is a trait; zero Agents is valid; a runner holds at most one Agent per hook | `explanation/agent_system_design.md:8,10`; `reference/narrative/agent_system.md:8-12,19-21` (C44, C45) | `src/application/agents/trait_def.rs:8,17`; `src/application/agents/runner.rs:15-18`, `:40-63`, `:69-79` | A03 #35, S3 | Agents (read `runner.rs`) | known |
| 24 | The model reports a list; the engine computes the transition | `explanation/agent_system_design.md:26-30`; `reference/game_flow.md:134-139` (S6, S11) | `src/domain/model/quantifier.rs:23-25`, `:41-65`; `src/domain/model/utils/game_rules.rs:41-83` | A02 #10, #1 | Quantifier path | known |
| 25 | `BackendSelector` is metadata; the before-narration hook has no code until its first Agent | `explanation/agent_system_design.md:18,22` (MP14) | `src/domain/model/agent.rs:108-112`, `:130,136` (the only uses of `UseNamed`; `rg` finds no reader of `.backend`); `rg -i "before.?narration" src` finds no code | A02 #4 | Both | known (read) |
| 26 | Sanitise the player input (`{{…}}` → `[FILTERED]`) and the model output | `reference/narrative/prompt_system.md:43`; `reference/narrative/narration_system.md:26` (C48) | `src/application/prompting/builders/sections.rs:8-29` (read); `src/application/prompting/sanitize.rs:8-33`; `src/application/llm_recorder.rs:79` | A03 #21 | Player input, model output (a closing tag is not filtered, 2.4) | known |
| 27 | Narrator prompt is a stack of layers; instructions and data stay apart | `reference/narrative/prompt_system.md:12,14-23`; `explanation/prompt_system_design.md:8,10,32-36` (C49, C50) | `src/application/prompting/assembler.rs:168-184`, `:335-338` | A03 #26, S6 | Narrator prompt; Agents build their own (A03 M15) | known |
| 28 | Seed once; after that the database is the source of truth; seeding is idempotent | `reference/startup.md:21-24`; `explanation/storage_design.md:22-24,40` (C28, C29) | `src/bootstrap/load.rs:116-126`; `src/adapters/driven/storage/settings.rs:118-122`; `src/adapters/driven/storage/characters.rs:59` | A04 #28; A06 C12 | In part: see 2.2, 2.3 | known (read `load.rs`, `run.rs`) |
| 29 | A dangling connection reference stops the boot | `reference/startup.md:33`; `reference/architecture_system.md:36` (C31) | `src/bootstrap/wiring.rs:145-148`; `src/bootstrap/load.rs:66-70` | A06 C10 | Connections only; the Agent list is not checked (2.2) | known (read) |
| 30 | Test tiers split by what is faked; seven test binaries | `reference/coding_standards/testing.md:12-16`; `reference/coding_standards/integration_test_standards.md:6-8` (C56) | `tests/STRATEGY.md:7-30`; `Cargo.toml:58-93` (seven `[[test]]` names, read) | A07 T1; A08 C1, C2 | Everywhere under `tests/` | known |
| 31 | Real SQLite `:memory:`; strict `ScriptedModel` with a hold; lenient `MockBackend` at run time | `explanation/storage_design.md:56`; `reference/coding_standards/integration_test_standards.md:226-229,246-257` (C57, C58) | `src/adapters/driven/storage/core.rs:120-127`; `src/test_support/scripted_model.rs:1-8`, `:143-157`, `:262-268` | A07 T2, T4, T5; A08 C10, C13 | Tier 1 and seam contracts (A08 MP-1: unit files use three fakes) | known |
| 32 | Fault injection by typed key | `reference/storage.md:50-51`; `reference/coding_standards/integration_test_standards.md:108-114` (C59) | `src/adapters/driven/storage/core.rs:30-71`; `src/adapters/driven/storage/transaction.rs:14-30` | A04 #10; A07 T11; A08 C20 | Storage | known |
| 33 | Table-driven rules, a reason on each row; paired positive and negative tests; fitness functions | `reference/coding_standards/guardrails.md:8,56,60,65-71`; `reference/coding_standards/integration_test_standards.md:134-146` (C52, C54, C55) | `tests/infrastructure/guardrails/walker_rows.rs:6-27`; `tests/infrastructure/guardrails/mod.rs:206-430`; `tests/infrastructure/guardrails/registry_tests.rs:190` | A07 G1, G3, G4, G5 | Four walkers over seven rows; 25 other `check_*` fns are bespoke (A07 G3) | known |
| 34 | Docs are tied to code; docs follow Diátaxis | `explanation/diataxis.md:10-12`; `reference/coding_standards/guardrails.md:87` (C63, C66) | `//! [DOC: …]` on line 1 of 210 of 307 files (A06 C60); `tests/infrastructure/guardrails/structure.rs:38-100` | A03 S11; A06 C60; A07 G16 | `src/`; asset 09 Q1 is answered: the anchors are in the code | known |

## Notes on section 1

- Row 19 and row 25 are "the doc says the code does not have it, and it does not". The docs follow the code here.
- Rows 4, 10, 16, 22, 28 and 29 are true as the doc states them in the lines I cite. The same doc has a stronger sentence elsewhere that the code does not meet. Section 2 lists those sentences.

## 2. Docs name it, and the code does not follow it, or follows it only in part

Each item is a question. Both sides carry evidence. I opened every code file named here. Items 2.4, 2.5, 2.8, 2.9, 2.10 and 2.11 are the hazards that asset 09 and ticket 09 list. Items 2.1–2.3, 2.6, 2.7 and 2.12–2.23 are new from this comparison.

**2.1. The dependency invariant says "port traits only". Which sentence does a new persistence call follow?**

- Docs: "`domain/` + `application/` depend on port traits only" (`reference/architecture_system.md:21`); "The application layer reaches LLM, storage, and text-check through port traits" (`explanation/architecture.md:14`); "Only `bootstrap/` imports both port traits and adapter impls" (`reference/architecture_system.md:23`). The same doc set says "`Storage` is concrete-by-design" (`explanation/architecture.md:65`) and "Application code may call `Storage` directly when the call is part of the persistence boundary" (`reference/architecture_system.md:30`).
- Code: `src/application/ports/` holds two traits, `LlmProvider` and `TextChecker`, and no storage trait (read: `rg "trait " src/application`). Nine non-test files under `src/application/` import `crate::adapters::driven::storage` (`settings_service.rs:9`, `world_catalogue.rs:6`, `persona_catalogue.rs:10`, `games/catalogue.rs:9,14`, `games/view_query.rs:6-7`, `games/active.rs:6`, `pipeline/job.rs:8-10`, `prompt_preset_service.rs:6`, `generation/gate.rs:8`). Three of them import more than `Storage`: `Tx` and `TxToken` (`gate.rs:8`), `WorldBundle` and `STORY_LIMIT` (`job.rs:8-9`), `LLM_MESSAGE_RETENTION_LIMIT` (`view_query.rs:6`). `arch-lint.toml:103-111` denies `application → server` and `agent-edge → storage`, and does not deny `application → storage`. `src/adapters/driving/http/app_state.rs:26` imports `bootstrap::wiring::WiredApp`.
- What the code shows: it follows the line-30 sentence and the lint. It does not follow the line-21 sentence. The two sentences stand in one doc.
- Label: known. (A03 M1, A04 MP-1, asset 09 MP1.)

**2.2. "Every consumer resolves settings at the point of use". Does the Agent list follow it?**

- Docs: "Every consumer resolves settings from storage at the point of use, so a connection or preset change takes effect on the next request without a restart. Bootstrap seeds the row once from `data/settings.json` … after that the file is never consulted" (`reference/architecture_system.md:36`); "Components … call `Storage::get_settings` at the point of use … nothing holds a long-lived settings handle" (`explanation/rust_idioms.md:22`).
- Code: the three model connections resolve per call through a closure (`src/bootstrap/wiring.rs:44-54`, `src/application/llm_recorder.rs:11-17`). The Agent list is read once in `build_app_graph`: `AgentRunner::from_configs(&settings.agents, …)` (`wiring.rs:161-166`, read). `rg "\.agents" src` finds no other production reader. `seed_settings` parses `data/settings.json` on every boot when the file exists, and rejects a dangling connection in it before it asks the database whether a row exists (`src/bootstrap/load.rs:49-72`, read). So a malformed file, or a file with a dangling connection, stops a boot that already has a row.
- What the code shows: connections follow the doc. The Agent list is a boot-time read. The seed file is read at every boot, and its content can stop the boot.
- Label: known for the reads. The effect of an Agent change without a restart is inferred (A06 MP-1).

**2.3. "A malformed world, persona, or preset file is skipped without halting bootstrap". Which files does the code skip?**

- Docs: `reference/startup.md:35`; "A malformed JSON file is skipped with a tracing warning, and the seeder continues with the remaining files" (`explanation/storage_design.md:38`); "Once a row exists, the JSON file that produced it is no longer consulted" (`explanation/storage_design.md:40`).
- Code (read `src/bootstrap/load.rs:77-185`, `src/bootstrap/run.rs:170-262`): a bad `world.json` and a bad NPC file are skipped (`load.rs:103-112`, `:155-157`). A bad map file (`load.rs:123`), a bad persona file (`load.rs:179`) and a bad preset file (`run.rs:200-203`, with `?` at `run.rs:179`) return `Err`. The caller logs one warning and goes on (`run.rs:54-60`). The files after the bad one do not seed. A test pins the `Err` of `ensure_presets` (`src/bootstrap/run_tests.rs:284-297`, A06 MP-2). A preset row that holds no content is overwritten from its file on every boot (`run.rs:235-248`).
- What the code shows: skip for two file kinds, stop-the-pass for three, one-way refresh for presets.
- Label: known. The doc text of `startup.md:35` did not change in the six-file edit.

**2.4. Does the filter of `{{…}}` hold the trust boundary of the player input? What does a closing `</PlayerInput>` in the text do?**

- Docs: "The assembler passes it through `sanitize_for_prompt`, which replaces any `{{…}}` pattern with `[FILTERED]`" (`reference/narrative/prompt_system.md:43`). No doc says what happens to an XML tag in the player text.
- Code (read): `sanitize_for_prompt` replaces `{{…}}` only (`src/application/prompting/builders/sections.rs:8-29`). `render_user_layer` writes `<PlayerInput>\n`, the sanitised text and `\n</PlayerInput>\n` (`src/application/prompting/assembler.rs:335-338`). A `</PlayerInput>` in the text passes through unchanged.
- What the code shows: the code does what the doc says. The doc is silent on the tag. The same delimiter `{{…}}` also serves as the template macro syntax for presets (asset 09 MP15).
- Label: known for the code path. The effect on the model is guessed.

**2.5. "Four LLM impls". Is the Mock one of them, and is the choice a "wiring change in `bootstrap/`"?**

- Docs: "Swapping OpenRouter for Ollama is a wiring change in `bootstrap/`" and "The four LLM impls exist because the seam makes them cheap" (`explanation/architecture.md:12`). The diagram shows three: OpenRouter, DeepSeek, Ollama (`explanation/architecture.md:40-42`). `reference/coding_standards/integration_test_standards.md:257` names `MockBackend` as "the lenient runtime provider of `LlmBackendType::Mock`".
- Code (read): `src/adapters/driven/llm/providers/mod.rs:18-21` re-exports four backends: `DeepSeekBackend`, `MockBackend`, `OllamaBackend`, `OpenRouterBackend`. `LlmBackendType` has four variants (`src/domain/model/llm_backend.rs:12-18`). `provider_from_config` matches all four (`src/bootstrap/wiring.rs:35-42`) and runs on every call, from the stored connection (`wiring.rs:44-54`). `DeepSeekBackend::complete` fails on every call (A04 MP-16, `deepseek.rs:25-30`).
- What the code shows: four impls, one of them the Mock and one of them a stub that fails at the call. The count "four" matches the code. The diagram is the one that is short by the Mock. The choice between providers is a stored setting read per call, not a bootstrap edit. `explanation/rust_idioms.md:22-24` says the same about a connection edit.
- Label: known.

**2.6. "Every record of a Game's story" has one writer. Which writes of the Game record escape the registry hold?**

- Docs: `GameWriter` may write "every record of a Game's story, through a save unit" (`reference/architecture_system.md:48`); `GameCatalogue` may write "the config columns of the Game record" (`:54`); a config write is "refused as busy" while a run hold exists (`reference/game_flow.md:38`); the dock is a table of its own "because the config routes write the Game record with no busy check" (`explanation/storage_design.md:50`).
- Code (read `src/application/games/catalogue.rs:75-92`, `:195-215`; `src/application/generation/gate.rs:405-420`): `config_write` refuses only when a run hold exists, not when a job entry exists (`gate.rs:410-417`). `rename_game` calls `storage.update_game_display_name` with no `config_write` (`catalogue.rs:77-86`). `delete_game` and `switch_game` read the active pointer outside the registry hold (A03 M18).
- What the code shows: the "no busy check" sentence and the "refused as busy" sentence both hold. The first is true for a job entry. The second is true for a run hold. A rename passes both.
- Label: known. The effect of a rename during an Autoplay run is guessed (A03 hazards).

**2.7. "Every `Mutex` site … recovers … with a single consistent shape". Is that every site?**

- Docs: `explanation/rust_idioms.md:28-30`; "Lock-poison recovery — every `Mutex` site recovers via `into_inner()`" (`explanation/architecture.md:78`).
- Code (read `rg "\.lock\(\)" src`): production sites recover (`gate.rs:152-155`, `db.rs:27-32`, `core.rs:145-148`, `:213-216`, `active.rs:28`). They use three spellings (A06 MP-12). `src/test_support/scripted_model.rs:126,132,150,164,175,188,288,307,323` call `.lock().expect(…)`. That file is part of the library because `testing` is a default feature (`Cargo.toml:54-57`). `parking_lot` is a dependency that no file uses (A06 MP-12, `Cargo.toml:28`).
- What the code shows: all production `Mutex` sites recover. The test fake sites do not.
- Label: known.

**2.8. Is `AppSettings` a database row or a global static?**

- Docs: "`AppSettings` lives in a global static, so parallel tests would race without it" (`reference/coding_standards/integration_test_standards.md:153-154`) against "`AppSettings` lives as a singleton row in the database" (`explanation/rust_idioms.md:22`).
- Code (read): `rg "static " src` finds no settings static. The only statics in production are the regexes (`prompting/sanitize.rs:10-16`, `agents/options/utils/parser.rs:15-20`) and `REQUEST_COUNTER` (`adapters/driven/llm/transport/utils/request.rs:8`). `SettingsTestGuard` is a `static Mutex<()>` (`tests/test_utils/settings_guard.rs:5`). Each test builds its own `:memory:` storage (A08 C16). The build runs nextest with one process per test (A07 MP-11), so the guard may serialise nothing (A08 MP-7).
- What the code shows: the settings are a row. A static lock exists, and the doc sentence names the wrong thing as static. What the lock guards today is not clear from the code.
- Label: known for the row and the lock; inferred for "guards nothing".

**2.9. The guardrail doc takes its rule numbers from `.scratch`. Do the numbers exist there?**

- Docs: "`.scratch/_closed/turn-state-model/assets/sketch-25-new-design.md` §6.2 — the design rule numbers the four table-driven walkers enforce" (`reference/coding_standards/guardrails.md:114`). The generated table uses rules 1, 2, 3, 6, 7, 13 and 18 (`:65-71`). `docs/AGENTS.md` exempts plans from the doc rules.
- Code and plans (read): the file exists and is tracked. Its §6.2 table has rules 1–15 (`sketch-25-new-design.md:617-640`). Rule 18 is not in it. Rule 18 comes from `.scratch/_closed/platform-choice/issues/06-research-race-tests-for-guard.md:28` and ticket 12 of the same map. The status line of the sketch says that ticket 24 replaced its rules 8, 11 and 13 (`sketch-25-new-design.md:7`). `tests/infrastructure/guardrails/walker_rows.rs:11-104` carries the rule numbers in comments. `sketch-25-new-design.md:752` says "The final code does not keep the numbers".
- What the code shows: the numbers in the code and the doc come from two plan trees. The doc names one of them.
- Label: known.

**2.10. What happens to a Job that runs when the process stops?**

- Docs: "The server stops accepting requests when the process receives the shutdown signal, and it serves the requests it already holds until they end" (`explanation/rust_idioms.md:38`). A restart "leaves every Game idle" (`explanation/architecture.md:79`).
- Code (read `src/adapters/driving/http/bootstrap/server.rs:48-62`, `src/bootstrap/run.rs:140-152`): `with_graceful_shutdown` covers the HTTP server. The Job runs on the blocking pool after its request ended (`pipeline/job.rs:85-88`). `start_server` blocks on the server handle and then returns, and the runtime drops. No code in the scope waits for a Job.
- What the code shows: the doc sentence is true for requests. No file says what the runtime drop does to a running Job. I do not know the Tokio behaviour for a blocking task at runtime drop here.
- Label: guessed for the effect; known for the missing wait.

**2.11. Is `settings` a standalone table that the doc describes?**

- Docs: the cluster list names "settings, prompt presets, and LLM call forensics" (`reference/storage.md:84`); the table list under "Standalone tables" describes `llm_messages` and `prompt_presets` only (`:102-103`); the settings row is described at `:65`.
- Code (read `src/adapters/driven/storage/utils/plumbing.rs:231-232` and `rg "CREATE TABLE" plumbing.rs`): a `settings` table with `CHECK (id = 1)` exists. Counting the `CREATE TABLE` statements and the drop of `game_state_snapshots` (`plumbing.rs:670`) gives 11 live tables, which matches "The schema has 11 tables" (`storage.md:80`) and "eleven-table schema" (`startup.md:93`). The doc uses `message_swipes` and `game_docks`, which match the code.
- What the code shows: the table exists and the count holds. The standalone list does not describe it in place.
- Label: known.

**2.12. "The two stores cannot disagree". Does the status read them as one?**

- Docs: "Neither store holds the other's answer, so the two cannot disagree" (`explanation/two-state-channels.md:8`); "The status of a Game has one rule" (`reference/game_flow.md:92`).
- Code (read `src/application/games/view_query.rs:204-218`): `status` asks the registry for `busy`, then asks storage for the Game row, in two steps. They are not one hold. A Job that ends between the two reads can show an old failure line for one poll (A03 M19, hazard, inferred).
- What the code shows: the rule is one rule. The read is two reads.
- Label: known for the shape; inferred for the effect.

**2.13. "Domain purity — no `tokio`, no I/O, no HTTP types". Does the domain read the environment, the clock and the log?**

- Docs: `explanation/architecture.md:75`; "Pure data structures AND pure rule methods" (`reference/architecture_system.md:12`); `arch-lint.toml:92` "Model layer must be pure".
- Code: no `async`, `tokio` or `trait` in `src/domain/` (A02 search). `resolve_api_key` reads `std::env::var` (`src/domain/model/settings.rs:317-319`). `Message::new` reads `Utc::now` (`src/domain/model/message.rs:70`). `Room::new_dynamic` reads `SystemTime::now` (`src/domain/model/map.rs:52-58`). `apply` logs with `tracing` (`game_rules.rs:125,179-205`). `NarratorMode::display_label` is "the label the dashboard renders" (`settings.rs:45-52`). `Step::as_endpoint_str` names an HTTP value (`state/generation_status.rs:239-246`). Four domain files import `crate::error` (A02 MP7).
- What the code shows: no `tokio`, no async, no HTTP types. Env, clock and log reads, and UI labels, exist.
- Label: known (A02 MP6, MP7; A06 MP-14).

**2.14. "Every game-scoped read and write takes its Game as the first argument, typed `GameId`". Which calls do?**

- Docs: `reference/storage.md:18-20` ("so no caller converts"); `reference/architecture_system.md:30`.
- Code (read `rg "pub fn" src/adapters/driven/storage`): the read handles and `Tx` take `GameId` (`stories.rs:44,54,117,132,155`, `docks.rs:20`, `core.rs:241`, `transaction.rs:44-53`). `get_game`, `require_game` and `update_game_display_name` take `id: u64` (`games.rs:34,56,80`). `world_bundle_for` takes `game_id: u64` (`worlds.rs:127`). `GameCatalogue` takes `u64` and wraps it (A03 M11). The handlers that carry the Game id in the path take a raw `u64` (`http/games/handlers/games.rs:119,131`, A05 MP-13).
- What the code shows: the typed form holds for the story and dock calls. The Game row calls take a raw id.
- Label: known.

**2.15. Which entities follow the `get_*` / `require_*` pair, and are Character reads roster-only?**

- Docs: "`Storage` exposes two absence policies for entity reads" (`reference/storage.md:69-72`); "character reads stay roster-shaped" (`explanation/storage_design.md:70`).
- Code (read `rg "pub fn (get|require)_" src/adapters/driven/storage`): pairs exist for Game, Persona and World. Presets have `get_preset` only. Settings have `get_settings`, which returns a default (A04 #6). `get_character(world_id, key)` exists (`characters.rs:29`). Only unit tests call it (`characters_tests.rs:32,42,56,99`).
- What the code shows: the pair holds for three entities. The doc says "for the entities where this distinction matters". A single-Character read exists and has no production caller.
- Label: known.

**2.16. "Tests poll for conditions rather than `sleep`". Which tests sleep?**

- Docs: `reference/coding_standards/testing.md:28`; "never a fixed `sleep`" (`reference/coding_standards/integration_test_standards.md:68`).
- Code (read `rg -n sleep`): `tests/browser/stub/options.rs:57` (`SUBMIT_SETTLE`), `tests/llm/flow_llm_tests.rs:38` (500 ms), `tests/http/requires_migration/text_check.rs:29` (100 ms), `src/adapters/driven/storage/llm_messages_tests.rs:76` (10 ms to order two rows). The wait helpers also sleep between polls (A07 T27).
- What the code shows: the tier-1 tests outside the quarantine have no sleep (A07 T27, A08 C38). Three other tiers and the unit tier keep a few.
- Label: known.

**2.17. "The loader calls `default_scenario()`". Which code is the loader?**

- Docs: `reference/startup.md:45`. The six-file edit changed line 43 only.
- Code (read `rg "default_scenario" src`): `src/domain/model/world.rs:74-76` (the initial room, `"start"` fallback), `src/domain/model/utils/opening.rs:16` (the opening Narration), `src/application/generation/gate.rs:666` (the create unit). Nothing in `src/bootstrap/` calls it (A06 Q1).
- What the code shows: the function and the fallback exist as the doc says. The caller is the create unit, not a loader.
- Label: known.

**2.18. "Adding an Agent of a known type is a wiring change in bootstrap". Where is the Agent set chosen?**

- Docs: `explanation/agent_system_design.md:10`.
- Code (read `src/application/agents/runner.rs:21-63`, `src/bootstrap/wiring.rs:161-166`): the set comes from `settings.agents` (the stored row), or from `AgentConfig::defaults()` when the list is empty. A second config of one hook replaces the first with a warning. An unknown type gives `Err`, and the wiring turns it into the default runner with no Agent and no log (`wiring.rs:166`, A06 MP-5).
- What the code shows: the choice is data in the settings row, read once at boot. The doc names it as wiring.
- Label: known (A03 #35, A06 MP-5).

**2.19. "Pure data structures AND pure rule methods". Where do the rules live?**

- Docs: `reference/architecture_system.md:12` (C21, inferred in asset 09). `reference/game_flow.md:145` names `game_rules::apply` in `domain/model/utils/game_rules.rs`, a free function.
- Code: `Message` and `Tail` carry rule methods (`message.rs:63-177`, `tail.rs:19-69`). `Game` carries almost none (`game.rs:47-65`). The game rules and the plan rules are functions (`game_rules.rs:41`; `generation.rs:227-273` as an associated fn). The application services are scripts over `Storage` (A03 #6).
- What the code shows: both forms. The doc names both in different lines.
- Label: inferred (A02 MP-9).

**2.20. "The status shows the first failure; a failure of the Options run goes to the log". Is the Options failure the only one that does?**

- Docs: `reference/game_flow.md:18`.
- Code: the Options retry returns `Err` (A03 M5 d). The after-narration hook turns a failed Agent into the empty result (`runner.rs:69-79`), and the end-of-Job hook returns the error (`runner.rs:84-98`). The Quantifier gives static Characters on any failure (`quantifier/utils/orchestration.rs:142-155`).
- What the code shows: three degrade paths and one stop path for Agents. The doc names one of them.
- Label: inferred. I did not trace which of these reaches the failure line.

**2.21. "One Job per Game … refused with 'Still thinking...'". What does the client see?**

- Docs: `explanation/architecture.md:80`; `reference/game_flow.md:15,50`. `reference/game_flow.md:22` names 400 and 500 for the other refusals.
- Code (read `src/adapters/driving/http/error.rs:33-37`): `Refusal::StillThinking` answers `200` with markup (`BUSY_STATUS`). `BadRequest` answers 400. A `Validation` refusal answers 400 in one place and 200 in another (A05 MP-2, MP-19).
- What the code shows: "refused" is a 200 reply for busy. The docs name the codes for two of the three refusals.
- Label: known.

**2.22. "Ubiquitous language": do the docs and the code use the glossary words?**

- Docs: bold terms in `reference/game_flow.md:8,32,34,98` (C46). `CONTEXT.md` lists "NPC" under Avoid for Character and "Turn" under Deprecated Terms.
- Code and docs: `rg -c "NPC|Npc" docs/diataxis` finds hits in nine docs (for example `reference/startup.md`: 5, `explanation/storage_design.md`: 4, `reference/storage.md:97`). The code has `NpcCard`, `NpcEvent`, `NpcEncounterLog`, `npcs_in_area` (A02 MP-13, A03 M9) and `struct Turn` in a test (A08 MP-22).
- What both show: the rebuilt code (`present`, `destination`, `RunHold`, `JobEntry`) follows the glossary. The docs and the older code use "NPC".
- Label: known.

**2.23. HTTP handlers "own request parsing and response rendering" and write "Nothing". Where do the business rules of a settings, world or preset edit live?**

- Docs: the row `HTTP handlers | Request parsing and response rendering | Nothing. They call … the catalogues and PromptPresetService` (`reference/architecture_system.md:59`). `PromptPresetService` "owns … the rules of a preset edit and a preset delete" (`:56`).
- Code (read `src/adapters/driving/http/settings/handlers/settings.rs:313-340`): `delete_connection_handler` passes a closure to `settings_service.update_settings`. The closure holds three rules: "Connection not found", "Cannot delete the last connection" and "role in use". A05 MP-1 lists three more sites: `update_world_handler` (`http/worlds/handlers/worlds.rs:172-190`), `duplicate_preset_handler` (`http/prompt_presets/handlers/prompt_presets.rs:254-260`) and `activate_preset_handler` (`:287-289`). In the same area `rename_game` and `delete_game` are one service call each (`http/games/handlers/games.rs:136,145`).
- What the code shows: the handler writes through a service, so "writes nothing" holds in the strict sense. The rules sit in the handler for these four routes and in a service for the Game routes.
- Label: known (A05 C3, MP-1, MP-17).

## 3. The code follows it, and the docs do not name it

Each row is a concept that two or more code assets name, or one asset names with strong evidence, and that asset 09 does not list. "Docs searched" gives the `rg` terms I ran over `docs/diataxis/` (case-insensitive). "Docs result" says what I found. A row says "named, but not as a concept" when a doc uses the thing but gives it no name or reason. A row says "docs name it; asset 09 missed it" when the doc has the text and asset 09 has no row.

| # | Concept (pretrained name) | Code evidence | Code assets | Docs searched | Docs result |
|---|---|---|---|---|---|
| 3.1 | Test hooks and doubles compiled into the production crate behind a default-on `testing` feature | `Cargo.toml:54-57` (read); `src/lib.rs:46-52`; `src/adapters/driven/storage/core.rs:76`; `src/bootstrap/wiring.rs:179-180`; `src/application/generation/job_tasks.rs:6-7` | A03 #32; A04 #8, #9; A05 C32; A06 C56, MP-21; A07 T9, T10 (22 sites in 8 files); A08 C4 | "testing feature", "cfg(feature", "default feature", "test hook" | None. Docs name the `Test` layer (`reference/storage.md:13`) and `test_support/` (`:49`), not the feature or the fact that it is on by default |
| 3.2 | Design by contract: assertions behind a `diagnostics` feature, on in the default build | `src/domain/model/utils/game_rules.rs:15`, `:222-243`; `Cargo.toml:55-56` (`testing = ["diagnostics"]`, read) | A02 #16, Q1 | "diagnostics" | None for the feature. The hits are about test diagnostics folders (`reference/coding_standards/testing.md:24`) |
| 3.3 | Ambient clock and ids: `Utc::now`, `SystemTime::now`, UUID called in place | `src/adapters/driven/storage/games.rs:40`; `src/domain/model/message.rs:70`; `src/domain/model/map.rs:52-58`; `src/adapters/driving/http/utils/handler_helpers.rs:19-28`; `src/application/llm_recorder.rs:112` | A02 MP6; A04 #34, MP-10; A05 MP-15; A06 C49, MP-22; A08 MP-10 | "clock", "Utc::now", "system clock", "injected clock" | None |
| 3.4 | One crate-wide error enum, a translation at each layer boundary, refusal as a value | `src/error.rs:112-205`; `src/application/errors.rs:6-30`; `src/application/generation/gate.rs:26-52`; `src/adapters/driving/http/error.rs:13-43` | A03 #16, #17, M3, M4; A05 C13, C14; A06 C26-C33, MP-9 (seven error types on one request) | "ApplicationError", "EngineError", "Refusal", "thiserror" | Four lines: `reference/storage.md:76` (`EngineError::Database`), `reference/game_flow.md:22` (400 and 500), `reference/coding_standards/integration_test_standards.md:114`. No doc names `ApplicationError`, `Refusal`, `JobFailure` or the chain |
| 3.5 | Strict parse and lenient parse (`parse_or_default`) of the same enum | `src/domain/model/llm_backend.rs:20-34`; `src/domain/model/settings.rs:54-59`, `:185-190`; `src/adapters/driving/http/worlds/handlers/worlds.rs:74-88` against `:238-257` | A02 #5, #15, MP1; A04 #35, MP-6; A05 C18, MP-4 | "parse_or_default", "lenient", "unknown value", "falls back" | Only the seed-file skip (asset 09 C32) and `MockBackend` as "lenient" (`integration_test_standards.md:257`) |
| 3.6 | Tolerant reader of model output, bounded retry, degrade to a default (Agents) | `src/application/agents/options/utils/parser.rs:25-41`; `src/application/agents/quantifier/utils/parser.rs:62-115`; `…/orchestration.rs:90-93`, `:142-155`; `src/application/agents/runner.rs:69-79` | A03 #18, #19, #20, M5, M6 | "confidence", "parse", "attempt", "fallback", "falls back" in `reference/narrative/` and `explanation/agent_system_design.md` | The JSON shape only (`explanation/agent_system_design.md:26`). `reference/game_flow.md:18` says an Options failure goes to the log. No doc names three parse strategies, two attempts or the static-Characters fallback |
| 3.7 | Closed-enum dispatch outside storage (provider factory, `Role`, settings by provider) | `src/domain/model/llm_backend.rs:12-18`, `:37-42`; `src/bootstrap/wiring.rs:35-42`; `src/domain/model/agent.rs:14-41`; `src/domain/model/settings.rs:365-384` | A02 #3; A05 C28; A06 C7 | "enum", "closed" | Named for `Backend` only (`explanation/rust_idioms.md:8`) |
| 3.8 | Front controller with RPC-style routes: verb in the URL, POST for every write, no `PUT` or `DELETE` | `src/adapters/driving/http/builders/router.rs:22-224`, `:92-103` | A05 C4, C6, C7 | "REST", "RPC", "PUT", "front controller" | `reference/frontend/http_routes.md` lists the routes (generated). No sentence states the style |
| 3.9 | HTTP status as a UI control channel (an htmx swap stops on non-2xx); busy answers 200; one `Validation` answers 400 and 200 | `src/adapters/driving/http/error.rs:13-43`; `…/utils/error.rs:48-50`, `:63-65`; `…/utils/fragment.rs:26-37`; `…/games/handlers/games.rs:225-226` | A05 C44, C45, MP-2, MP-19 | "HX-", "swap", "200", "400", "500" | Only 400 and 500 for plan refusals (`reference/game_flow.md:22`) and 500 for the failure line (`explanation/two-state-channels.md:18`). No hit for `HX-`, "swap" or 200 |
| 3.10 | Compile-time inline Askama templates, view models, a `SafeHtml` marker type, composite views from pre-rendered strings | `src/adapters/driving/http/templates.rs:14-19`; `…/view_models.rs:15-25`; `…/utils/response.rs:35-41` | A05 C19-C25, MP-6 to MP-9 | "askama", "SafeHtml", "escap", "view model" | Two lines: `reference/coding_standards/guardrails.md:96` (raw strings in Askama sources) and `testing.md:32` (an escaping assertion) |
| 3.11 | Trusted-network assumption: no authentication, no CSRF token; `data/` served over HTTP | `src/adapters/driving/http/builders/router.rs:219-222` (only `TraceLayer`) | A05 C39, Q2, Q5 | "auth", "csrf", "trusted", "single-user", "localhost" | None relevant (four noise hits) |
| 3.12 | Identity made at the edge (clock plus UUID in the adapter) | `src/adapters/driving/http/utils/handler_helpers.rs:19-28`; `…/prompt_presets/handlers/prompt_presets.rs:184,255`; `…/settings/handlers/settings.rs:185` | A05 C41, MP-15; A06 MP-22 | "uuid", "storage id", "generate_storage_id" | None. `reference/storage.md:103` says only that a preset `id` is a text key |
| 3.13 | Immediate apply (autosave on change) beside explicit save, on one form | `src/adapters/driving/http/worlds/templates/worlds.rs:114`, `:119-123`, `:222-223`; `…/games/templates/games.rs:154` | A05 C40, MP-14 | "autosave", "auto-save", "immediately" | None |
| 3.14 | The domain serde shape is the stored JSON; forward-only `PRAGMA user_version` migrations; two migrations delete every Game | `src/adapters/driven/storage/utils/plumbing.rs:17-61`, `:662-671`, `:716-726`; `…/transaction.rs:169`, `:196` | A02 #17 (derive on every type); A04 #24, #25, MP-8, MP-9 | "migration", "user_version", "JSON column", "serialized" | Test docs only (`integration_test_standards.md:128,179`). `docs/AGENTS.md` keeps migration version numbers in code by rule; the concept is not in `reference/storage.md` |
| 3.15 | Hand-written SQL on one connection behind a coarse mutex; `DbPool` holds one connection | `src/adapters/driven/storage/db.rs:12-14`; `…/core.rs:17`, `:213-216`; `…/games.rs:16-18` | A04 #16, #18, Q1; A06 C46 | "ORM", "rusqlite", "pool", "connection" | The backend mutex is named (`reference/storage.md:57`, `explanation/storage_design.md:46`). Hand SQL, no ORM and one connection are not |
| 3.16 | Service layer with transaction scripts; some rules in the handler | `src/application/world_catalogue.rs:22-65`; `src/application/games/catalogue.rs:222-259`; the handlers of 2.23 | A03 #5, #6; A05 C3, MP-1 | "service", "orchestration", "transaction script" | `reference/architecture_system.md:13` says the application layer is "Orchestration". No doc names the pattern |
| 3.17 | A process-wide active-Game pointer as ambient context, read once per request | `src/application/games/active.rs:14-39`; `src/adapters/driving/http/utils/current_game.rs:17-39`; `tests/infrastructure/guardrails/walker_rows.rs:45-64` | A03 #9, M18; A05 MP-13; A07 G7 | "ActiveGame", "CurrentGame", "pointer" | Named (`reference/storage.md:24,35`; `reference/game_flow.md:126`; `reference/coding_standards/guardrails.md:67`). The contrast between the pointer and an explicit Game id in each route is not stated |
| 3.18 | `WiredApp` copied into one `AppState` bag that every handler takes (service locator or per-need DI) | `src/bootstrap/wiring.rs:84-96`; `src/adapters/driving/http/app_state.rs:31-57` | A05 C31; A06 MP-8, SN-2 | "AppState", "WiredApp", "service locator" | `reference/architecture_system.md:40` names the bundle. The shape is not named or argued |
| 3.19 | Structured logging with `tracing`; spans in one file, bracket tags and a global request counter elsewhere | `src/application/pipeline/job.rs:63-70`; `src/adapters/driven/llm/transport/utils/request.rs:8-12`; `src/bootstrap/logging.rs:11-69` | A03 M14; A04 #43; A06 C38-C44, MP-16 | "tracing", "logging", "RUST_LOG", "log level" | One line: `explanation/storage_design.md:38` ("a tracing warning"). No logging convention |
| 3.20 | Layered configuration: built-in default, seed file, database row; API keys from the environment | `src/domain/model/utils/settings_defaults.rs:1-20`; `src/domain/model/settings.rs:317-340`; `src/main.rs:14` | A02 #13, #25; A06 C21-C24 | "API_KEY", "environment variable", ".env", "env var" | Only the real-LLM test gate (`reference/coding_standards/testing.md:8`, `integration_test_standards.md:181,263`) |
| 3.21 | Executable specification (`SCENARIO` tag per test) and seam contracts (`CASE` tag per clause) | `tests/STRATEGY.md:116-143`, `:222-249`; `src/application/narration/narrator_tests.rs:269`; `src/application/agents/options/agent_tests.rs:93` | A07 T13, T14, S-8; A08 C31, C32 | "SCENARIO", "CASE", "seam contract", "spec" | None in `docs/diataxis/` (the word "case" hits are prose). `docs/AGENTS.md` names specs, and it is outside the scope |
| 3.22 | Ratchet allowlists, a quarantine folder (`requires_migration`), guards for the guards, no vacuous pass | `tests/infrastructure/guardrails/unit_test_allowlist.rs:5-7`, `:187-207`; `…/registry_tests.rs:190`; `…/mod.rs:164-190`; `tests/http/requires_migration/mod.rs:1` | A07 T17, T18, G5, G6; A08 C7, SN-10 | "ratchet", "quarantine", "requires_migration", "vacuous", "meta-test", "stale entry", "allowlist" | `reference/coding_standards/guardrails.md:99` says the unit-test rule fails "a list entry that names no file" (the ratchet, with no name). The quarantine and the meta-tests have no hit |
| 3.23 | Held model call as a latch for deterministic interleaving | `src/test_support/scripted_model.rs:143-157`, `:301-331`; `tests/http/races.rs:26-30` | A07 T5; A08 C36 | "hold", "latch", "ScriptedModel" | Docs name it: `reference/coding_standards/integration_test_standards.md:250` (`ScriptedModel::hold(role)` returns a `ModelHold`). Asset 09 lists only the strictness (C58). Asset 09 missed it |
| 3.24 | The Autoplay run: ordinary Jobs one after another under a run hold | `src/application/autoplay/driver.rs:16`, `:54-56`, `:104-110`; `src/application/pipeline/job.rs:109-123` | A03 S13, #14, #15 | "autoplay", "run hold", "cycle" | Docs name it since the uncommitted edit (`reference/architecture_system.md:53,59`; `reference/game_flow.md:12,34-38,92`; `reference/frontend/dashboard.md:30-36`; `reference/frontend/http_routes.md:31-35`). Asset 09 mentions the run hold only in C42, C46 and S1. It has no concept row for the run |
| 3.25 | Package by role (`handlers/`, `templates/`, `builders/`, `utils/`) beside package by feature; free functions only in listed folders | `src/adapters/driving/http/mod.rs:4-22`; `tests/infrastructure/guardrails/free_fn.rs:8-15`, `:49-58` | A03 Q11; A05 C34-C36, MP-21; A07 G13 | "package by", "by feature", "by layer" | `reference/coding_standards/guardrails.md:76` names the free-function rule. The folder concept is not named |

## 4. Citation re-check

**Question.** Do the citations of `research-09-docs.md` that point into the six edited docs still hold?

**Facts about the edits (known).**

- `git status` shows the six files as modified. `git diff --stat` shows 16 insertions and 15 deletions. Five files keep their line count. `reference/architecture_system.md` has one more line than `HEAD` (80 → 81): the new `AutoplayRun` row at line 53.
- The mtime of the six files is 2026-10-21 21:18. The mtime of `research-09-docs.md` is 22:31. The asset was finished after the edits.
- The edits add the **run hold** and the **Autoplay run** to the text. They change these lines:
  - `explanation/architecture.md:69`, `:76` (run holds in the registry).
  - `explanation/two-state-channels.md:12`, `:14` (busy means a job entry or a run hold).
  - `reference/architecture_system.md:48` (the `GameWriter` row), `:53` (new `AutoplayRun` row), `:59` (the handlers call `AutoplayRun::start`).
  - `reference/game_flow.md:12`, `:32`, `:34`, `:36-38`, `:92`.
  - `reference/startup.md:43`.
  - `reference/storage.md:35`.

**Method.** A script took every `path:line` and `path:line-line` citation of asset 09 whose path is one of the six files (153 citations, in 77 rows of the asset). For each, I printed the current lines beside the claim of the row. A second script took each quoted string next to a citation and searched it in the cited file. I read the 153 pairs. I then read the diff to see which edited line any citation touches.

**Result: no citation is broken by the edits.**

- All 153 citations point to a line that exists. No file is shorter than its cited range.
- The cited lines that the edits changed are `explanation/architecture.md:69`, `explanation/two-state-channels.md:12` and `:14`, `reference/architecture_system.md:48`, `:59`, `reference/game_flow.md:12`, `:32`, `:34`, `:38`, `:92`, and `reference/startup.md:43`. In each case the cited line still supports the claim of the row. Two rows were written against the new wording, which is a sign that the researcher read the edited files: the quote "is refused as busy" for `game_flow.md:38` (asset 09 MP7; the old text said "Still thinking..."), and the row for `architecture_system.md:58` (`SaveLlmMessageFn`), which sits at line 58 only after the new row at line 53.
- No citation points to `reference/storage.md:35`, the one other line that the edits changed.

**Quotes that are imprecise, not stale** (the edits did not cause them; all three are in the six files). One more case sits outside the six files: asset 09 MP4 cites `reference/coding_standards/integration_test_standards.md:152-153` for "`AppSettings` lives in a global static", and the phrase ends on line 154.

| Asset 09 line | Citation | What the asset says | Current text |
|---|---|---|---|
| 169 (MP19) | `explanation/two-state-channels.md:8` | quotes "did the last Job fail?" | `:8` reads "did the last Job or the last refused write fail, and why?". The text is the same in `HEAD`. The quote is a shortening. |
| 46–59 range, 133 | `reference/architecture_system.md:48` | quotes "every record of a Game's story" | `:48` reads "Every record of a Game's story, through a save unit: …". Only the case differs. |
| 127 | `reference/game_flow.md:124` | quotes "one save unit" | `:124` reads "One save unit creates a Game." Only the case differs. |

**Things that the edits add and that asset 09 does not use.**

- The `AutoplayRun` row of the ownership table (`reference/architecture_system.md:53`). Asset 09 C12, S9 and MP7 describe the table without this row. The row says the part writes "the run hold only, through `GameWriter`".
- `game_flow.md:34-38`: a run hold "names its run and its budget" (the pre-edit text also named "the count of ended cycles", and the end rules of a run are gone from this doc). Asset 09 does not cite the removed text.
- `explanation/architecture.md:76` and `explanation/two-state-channels.md:14` name the run hold or the Autoplay run as a source of "busy". Asset 09 C41, C42 and C43 describe "busy" as the job entry. The citations still hold. The concept now has two sources of "busy" in the docs (the job entry and the run hold).

## 5. Questions

1. **Which doc sentence is the claim?** In 2.1, 2.2, 2.3 and 2.7 one doc holds a strong sentence and a narrower one. Does the catalogue record the narrower sentence as the concept, or record both and the gap?
2. **Do guardrails make a doc sentence true?** `explanation/architecture.md:73` says each guarantee "is machine-checked or relied on directly by the code". "No `std::thread::spawn` in `src/`" has a guardrail (`no_std_thread`, A07 G18, which skips any path that contains `mock.rs`, A07 Q-6). `rg -i "poison|into_inner" arch-lint.toml tests/infrastructure` finds no rule for lock recovery. `arch-lint.toml:103-111` has no rule that bars `application → storage`. Which doc sentences have a check, and which are descriptions?
3. **Is the Mock a LLM impl?** The count "four" (2.5) includes `MockBackend` and a stub. Does the catalogue count it as a provider, a test double that ships, or both (A04 #8)?
4. **Where is the line between a Reference doc that holds an invariant and a doc that copies the code?** `docs/AGENTS.md` says a doc that needs an edit for most changes "copies the code". The six-file edit is a small change that touched 16 lines for one new part (the run hold). Is that the expected size of an edit for a new part?
5. **What do the docs say about the testing feature?** The test layer is named in `reference/storage.md:13,49` and in the test standards. `Cargo.toml:55` turns it on by default, and no doc says so (3.1). A04, A06, A07 and A08 each ask whether a release build ships it (A06 Q2, A07 Q-2). I did not trace `build.py` packaging, and I do not know. Should the catalogue wait for that fact?
6. **Do the docs follow the glossary?** `CONTEXT.md` avoids "NPC", and `rg` finds "NPC" in nine docs (2.22). The same term sits in code names. Is the glossary rule meant to reach doc prose and type names?
7. **Which doc owns the failure rules of seeding?** `reference/startup.md:33,35`, `explanation/storage_design.md:38` and the code (2.3) give three versions. Asset 09 MP10 asks where the line between fail loud and skip is. The code has a line, and it is not the doc's line.
8. **What does "busy" mean after the edit?** `reference/game_flow.md:34` says "A Game is busy when its job entry or its run hold exists". `reference/game_flow.md:92` says "A Game with a job entry shows its step". `explanation/two-state-channels.md:12` titles the channel "the job entry". Is the run hold a third kind of fact, or a second kind of job entry?
9. **Are the doc rules numbers a concept?** The guardrail rows keep numbers from two plan trees (2.9). Asset 07 Q-5 asks the same. Does the catalogue point at the rows or at the plans?
10. **Is a concept that the docs name only for tests a concept of the code?** Examples: `ScriptedModel::hold` (3.23), the strict fake, `SettingsTestGuard` (2.8). The docs describe them in `reference/coding_standards/`. Asset 09 puts them in its 1b group.
11. **Do the docs treat `data/` as a template or as a served folder?** `explanation/storage_design.md:40` says seed files are templates read once. The router serves `data/` over HTTP (`router.rs:220`, A05 Q2), and the CLI list command reads `data/worlds/*/world.json` (A06 MP-19). I do not know whether any served file needs to stay private.
12. **Is asset 09 Q1 closed?** Yes by the code: `//! [DOC: …]` is on line 1 of 210 of 307 `.rs` files (A06 C60, A07 G16). I did not count the files. The docs in `docs/diataxis/` carry no anchor back. Does a reference doc need a list of the files that point to it (asset 09 Q5, "Document References")?
13. **Which of the six hazards of ticket 09 are still open?** After this comparison: the "four impls" count holds (2.5); the settings static is a wrong word for a lock (2.8); the `.scratch` numbers resolve to two plan trees (2.9); the closing tag has no filter and no doc (2.4); the shutdown wait is unwritten (2.10); the `settings` table is listed but not described in place (2.11). Which of these need an owner answer, and which are doc edits for a later effort?
14. **Do the doc statements that the code meets only in part come from the rebuild?** `explanation/architecture.md:14` says "the engine's storage seam, before hexagonal formalised it" and the doc keeps both stories. A03 Q1 asks whether the catalogue treats the rebuilt shapes as the target. Does the same question apply to the docs that were edited for the rebuild (the six files) against the ones that were not?
