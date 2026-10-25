# Research 03: the concepts in the application layer

Ticket: [03-research-application](../issues/03-research-application.md). Map: [map.md](../map.md).

Method: one reader read every non-test `.rs` file in `src/application/` in full, with line numbers. The reader opened no other `research-*.md` asset and had no concept list. Each concept below comes from a shape that the code repeats or from a choice that the code makes. The names are the names of books and well-known essays. `git log` and the plans in `.scratch/_closed/rust-rebuild-build/` explain the old and new shapes. The asset records what the code shows. It rules on nothing.

Labels:

- **known**: the code shows it directly, and the name is the standard name.
- **inferred**: the code shows a shape, and the reader chose the name or the opposite.
- **guessed**: the reader is not sure of the shape or of the name.

Terms: `CONTEXT.md`. Where the code says "npc", the asset says "npc" only when it quotes an identifier.

## Scope read

All 54 non-test `.rs` files of `src/application/` were read in full (5,083 lines). Nothing was sampled.

| Module | Files read |
|---|---|
| root | `mod.rs`, `errors.rs`, `llm_recorder.rs`, `connection_test_service.rs`, `persona_catalogue.rs`, `world_catalogue.rs`, `settings_service.rs`, `prompt_preset_service.rs`, `text_check_service.rs` |
| `ports/` | `mod.rs`, `llm_provider.rs`, `text_checker.rs` |
| `agents/` | `mod.rs`, `trait_def.rs`, `runner.rs`; `options/` (`mod.rs`, `agent.rs`, `prompt.rs`, `types.rs`, `utils/mod.rs`, `utils/orchestration.rs`, `utils/parser.rs`); `quantifier/` (`mod.rs`, `agent.rs`, `prompt.rs`, `types.rs`, `utils/mod.rs`, `utils/orchestration.rs`, `utils/parser.rs`) |
| `generation/` | `mod.rs`, `gate.rs`, `job_tasks.rs` |
| `pipeline/` | `mod.rs`, `job.rs` |
| `autoplay/` | `mod.rs`, `driver.rs` |
| `narration/` | `mod.rs`, `narrator.rs` |
| `games/` | `mod.rs`, `active.rs`, `catalogue.rs`, `view_query.rs` |
| `debug/` | `mod.rs`, `dto.rs` |
| `prompting/` | `mod.rs`, `assembler.rs`, `prompt_merge.rs`, `sanitize.rs`, `token_budget.rs`, `types.rs`, `builders/mod.rs`, `builders/sections.rs`, `utils/mod.rs`, `utils/context.rs` |

Unit tests: the reader read `agents/runner_tests.rs` lines 1-50, and the test function names of `games/catalogue_tests.rs` (by `rg 'fn '`). The reader opened no other `*_tests.rs` file. The reader did not read `bootstrap/wiring.rs`, so every claim about the composition root is inferred.

Search facts (non-test files of `src/application/`):

| Search | Count |
|---|---|
| `trait` definitions | 4 (`LlmProvider`, `TextChecker`, `AfterNarrationAgent`, `EndOfJobAgent`) |
| lines with `dyn` | 12 (`Box<dyn` on 2 lines, both in `agents/runner.rs`) |
| lines with `Arc<` / `Arc::` | 48 / 14 |
| `Mutex` fields | 3 (`generation/gate.rs:132`, `games/active.rs:16`, `generation/job_tasks.rs:10`); `RwLock`: 0 |
| `AtomicU64` | 2 fields (`generation/gate.rs:253-254`) |
| `async fn` | 2, both in `generation/job_tasks.rs` |
| `spawn_blocking` | 2 (`pipeline/job.rs:88`, `autoplay/driver.rs:56`) |
| `.unwrap()` | 0 |
| `.expect(` | 5, all on a static `Regex::new` (`prompting/sanitize.rs`, `agents/options/utils/parser.rs:11`) |
| `panic!`, `unreachable!` | 0 |
| `unwrap_or_else(.. into_inner())` (poisoned lock) | 6 |
| `#[allow(clippy::..)]` | 3 |
| `#[cfg(feature = "testing")]` | 5 |
| `#[instrument` | 3 (`pipeline/job.rs:70,99,219`) |
| `tracing::` macros | 45 |
| `impl Drop` | 2 (`generation/gate.rs:843`, `autoplay/driver.rs:106`) |
| `impl From<` | 2 (`errors.rs:66`, `generation/gate.rs:48`) |
| files that import `adapters::` | 9 of 54; none under `agents/`, `narration/` or `prompting/` |

## 1. Concepts

"Holds" says how far the concept reaches in `src/application/`.

| # | Concept (name, source) | Opposite(s) | Evidence | Holds | Label |
|---|---|---|---|---|---|
| 1 | **Ports and adapters** (Cockburn, hexagonal architecture): the layer owns the trait, and the infrastructure implements it | Direct call to the concrete infrastructure | `ports/llm_provider.rs:44`, `ports/text_checker.rs:64`, `text_check_service.rs:11` | In two places: LLM and text check. Storage has no port (see M1) | known |
| 2 | **Dependency rule** (Martin, Clean Architecture): inner code imports no outer code | Any-to-any imports | `rg 'adapters::'` finds 9 files; `agents/`, `narration/` and `prompting/` have none. `narration/narrator.rs:27-28` ("the narrator holds no storage"). Commit `ac938c25` (P4, "rule 11: agents and narration cannot import storage") | Agents, narration, prompting: yes. The 9 other files import the concrete `Storage` | known |
| 3 | **Constructor injection** (Fowler, "Inversion of Control Containers and the Dependency Injection pattern"), with the composition root outside this layer | Service locator; global state; construction inside the class | `text_check_service.rs:15`, `pipeline/job.rs:35-41` (`JobDeps`), `llm_recorder.rs:19-37` | Every service. No `static` state found (see S10 for the one global) | inferred (the root, `bootstrap/wiring.rs`, was not read) |
| 4 | **Function injection**: a closure type is the seam | Trait object as the seam (concept 1) | `llm_recorder.rs:14` (`ProviderResolver`), `llm_recorder.rs:17` (`SaveLlmMessageFn`), `connection_test_service.rs:18` (`ProviderFactory`) | The recorder and the connection test | known |
| 5 | **Service layer** (Fowler, PoEAA): the driving adapter calls a service, not storage | Fat controller; the handler reads storage directly | `persona_catalogue.rs:4` (the comment names this goal), `world_catalogue.rs:22-65`, `settings_service.rs:25-31`, `games/catalogue.rs:41` | Every non-Job use case | known |
| 6 | **Transaction script** (Fowler, PoEAA): one procedure per use case | Domain model with rich objects | `pipeline/job.rs:184-215` (`generate`), `games/catalogue.rs:222-259` (`reset`), `prompt_preset_service.rs:113-142` (`delete_preset`) | Most services. The rules come from the domain through concept 7 | inferred |
| 7 | **Functional core, imperative shell** (Bernhardt, "Boundaries"): pure rules inside, I/O at the edge | Logic mixed with I/O | `pipeline/job.rs:334` (`game_rules::apply`, one caller), `narration/narrator.rs:27-42` (`NarrationValues`: the narrator gets values), `pipeline/job.rs:315-319` (`HookInput` from values) | Job to narrator, to agents, to rules: yes. The catalogues and `GameWriter` mix rules and I/O | known |
| 8 | **Single writer principle** (Thompson, "Mechanical Sympathy"): one registry owns every write of a Game | Many writers with per-row locks; optimistic merge | `generation/gate.rs:128-157` (`JobRegistry`), `gate.rs:246-250` ("the only code that opens a unit"), `gate.rs:674-681` (`JobWriter`) | Job writes, create, replace, delete, stop, swipe, edit: yes. Other writes: see M2 | known |
| 9 | **Capability** (object-capability model; "make illegal states unrepresentable", Minsky) | Ambient authority: anyone who holds `Storage` may write | `gate.rs:160-165` (`Units`: only `hold` makes one), `gate.rs:674-681` (only `accept` makes a `JobWriter`), `games/active.rs:35-39` (`pub(super)` on `set`) | `gate.rs`, `active.rs`. `Storage` itself stays open to every service (see M2) | known |
| 10 | **Fencing token** (Kleppmann, *Designing Data-Intensive Applications*): a stale holder is refused by id | Cooperative cancel by flag; kill the task | `gate.rs:700-703` (job id compared under the lock), `gate.rs:311` (run id), `gate.rs:511-514` ("a running model call ends on its own") | Every Job save and every run step | known |
| 11 | **Lock ordering** (classic deadlock rule) | Ad hoc locking; lock-free | `gate.rs:145-146` ("guard, then the backend mutex, then the connection mutex"), `gate.rs:409` ("the lock is not reentrant"), `gate.rs:649` ("read before the unit opens, because the unit holds the backend mutex") | `gate.rs` only | known |
| 12 | **Unit of work** (Fowler, PoEAA): writes land whole or not at all | Autocommit of each statement | `gate.rs:168-174` (`Units::open`), `gate.rs:806-817` (one unit: message, swipe, dock). Glossary: "Save unit" | Writes through `GameWriter` and `JobWriter`. Config writes use single `Storage` calls (see M2) | known |
| 13 | **Loan pattern / execute-around** (Kerievsky, Rust closure form): a closure runs inside a scope the callee owns | A guard object that the caller holds (explicit lock and unlock) | `gate.rs:147` (`hold(f)`), `settings_service.rs:390` (`update_settings(f)`), `games/catalogue.rs:206-214` (`write_config(f)`) | The lock, the settings and the config writes | known |
| 14 | **RAII / drop guard** (Stroustrup; Rust idiom) | Explicit cleanup call; `finally` | `gate.rs:843-853` (`impl Drop for JobWriter`), `autoplay/driver.rs:104-110` (`impl Drop for AutoplayRun`) | The Job entry and the run hold | known |
| 15 | **Sum types for outcomes** (algebraic data types; "make illegal states unrepresentable") | Booleans and `Option<String>` flags; error codes | `gate.rs:26` (`Refusal`), `gate.rs:40` (`JobFailure`), `gate.rs:190` (`Busy`), `pipeline/job.rs:45` (`JobEnd`), `autoplay/driver.rs:16` (`RunEnd`), `games/view_query.rs:32` (`Status`) | Job, run and status. Two booleans stay in `Job::run` (M10) | known |
| 16 | **Errors as values** (`Result` with typed enums; no exceptions, no panics) | Exceptions; `unwrap` and `panic!` | zero `.unwrap()`; five `.expect(` on static regexes (`prompting/sanitize.rs:8`, `agents/options/utils/parser.rs:9-12`, each under `#[allow(clippy::expect_used)]`). `errors.rs:6` | Everywhere | known |
| 17 | **Error translation at a layer boundary** (the error type of a layer hides the lower layer) | One error type for all layers; leak the storage error | `errors.rs:6-30` (`ApplicationError`), `gate.rs:40-52` (`JobFailure` from `GenerationFailure`), `narration/narrator.rs:126,137` (`GenerationFailure::from_engine_error`) | Three envelopes sit side by side (M3) | known |
| 18 | **Retry with a bounded count** (Nygard, *Release It!*) | Fail on the first error; retry with back-off; circuit breaker | `agents/options/utils/orchestration.rs:14-16,50`, `agents/quantifier/utils/orchestration.rs:90-93,125-137` | Both Agents; the narrator does not retry (`narration/narrator.rs:129-137`) | known |
| 19 | **Graceful degradation / fail-safe default** (Nygard) | **Fail fast** (Shore) | `agents/runner.rs:69-79` (failed Agent gives the empty result), `quantifier/utils/orchestration.rs:142-155,187-199` (static Characters), `options/prompt.rs:8-10` (fallback system prompt) | Quantifier path, runner after-narration path | known |
| 20 | **Tolerant reader** (Fowler; Postel's law): accept varied model output | Strict parse; schema-enforced output | `options/utils/parser.rs:25-41` (three strategies, first wins), `quantifier/utils/parser.rs:62-115` (direct JSON, code fence, brace span, then text match) | Both Agent parsers | known |
| 21 | **Sanitise at the boundary** (defensive programming; input and output filtering) | Trust the model and the player | `prompting/sanitize.rs:8-33` (strip thought markers), `llm_recorder.rs:79`, `prompting/builders/sections.rs:8-29` (`{{..}}` to `[FILTERED]`), `prompting/assembler.rs:336` | Narration, Agents, recorder | inferred |
| 22 | **Decorator / proxy in spirit**: one wrapper adds recording and clean-up to each provider call | Each caller records its own call | `llm_recorder.rs:51-88` (`complete`: save on success and on failure, sanitise); `connection_test_service.rs:1-7` (the probe skips the wrapper on purpose) | Narrator, Quantifier, Options. Not a `LlmProvider` itself, so not a true Decorator | inferred |
| 23 | **Parameter object** (Fowler, *Refactoring*) | Long argument lists | `narration/narrator.rs:29-42` (`NarrationValues`), `agents/options/types.rs:9`, `agents/quantifier/types.rs:69`, `prompting/types.rs:7` (`NpcContext`) | Prompt and Agent code. See M7 | known |
| 24 | **Claim check** (Hohpe and Woolf, *Enterprise Integration Patterns*): pass a reference, load the payload late | Pass the loaded bundle from step to step | `pipeline/job.rs:180-183` ("each step loads by id what it uses ... only ids, texts and step results move"). Plan: `.scratch/_closed/rust-rebuild-build/issues/20-shape-of-the-job.md` (round 4) | Job steps. `world_bundle()` loads 5 times (`job.rs:234,255,311,332,350`) | known |
| 25 | **Builder** (GoF, fluent, consuming `self`) | Telescoping constructor; struct literal | `prompting/assembler.rs:107-146` (`PromptContext::new`, `with_guide`, `with_impersonate`) | Assembler only. See M7 | inferred |
| 26 | **Pipes and filters** (Hohpe and Woolf): ordered layer functions that each give a string, and empty ones drop | One template; one big `format!` | `prompting/assembler.rs:168-184` (eight layers, `filter(!is_empty)`, `join`) | Prompt assembly | inferred |
| 27 | **Blocking I/O on worker threads** | Async/await all the way | `ports/llm_provider.rs:48` (sync `complete`), `pipeline/job.rs:88`, `autoplay/driver.rs:56`; the only `async fn` are in `generation/job_tasks.rs:20,26` | The whole layer | known |
| 28 | **Poison-tolerant lock** (Rust idiom) | `lock().unwrap()`: the poison panic spreads | `gate.rs:155`, `games/active.rs:28,40,53`, `generation/job_tasks.rs:15,29` | All 3 `Mutex` types | known |
| 29 | **Command-query separation at the service level** (Meyer; CQRS-lite) | One service that reads and writes | `games/view_query.rs:2,41-45` ("read-side queries that don't mutate"), `gate.rs:250` (`GameWriter`), `persona_catalogue.rs:2` ("read-side") | Reads for the dashboard sit in `GameViewQuery`; writes in `GameWriter`. `GameCatalogue` holds both (M8) | inferred |
| 30 | **Ubiquitous language** (Evans, DDD): the code uses the domain terms | Technical or legacy names for domain things | `agents/quantifier/types.rs:58` (`present`, `destination`: glossary terms), `agents/runner.rs:71,84` (hook names by position), `gate.rs:98` (`RunHold`). Plan ticket `32-r4-quantifier-result-and-game-rules.md` | Rebuilt code: yes. Older code: M9 | known |
| 31 | **`Result<Option<T>>` for "absent is not a fault"** (Rust idiom) | Not-found as an `Err` | `pipeline/job.rs:439-441` (`present`), `gate.rs:602-606` (`Ok(None)` gives "missing"), `games/catalogue.rs:216-220` | Reads. See M11 | known |
| 32 | **Test hooks in production code** (xUnit Test Patterns calls this "Test Logic in Production") | Test doubles in a test-only module | `agents/options/agent.rs:39`, `agents/quantifier/agent.rs:42`, `connection_test_service.rs:88` (all `#[cfg(feature = "testing")]`); `generation/job_tasks.rs:6-7` ("built into production because `testing` is a default feature"); `games/view_query.rs:188-191` and `narration/narrator.rs:65-67` ("only tests call it") | 6 sites | known |
| 33 | **Sibling unit-test file** (project rule; the opposite is inline `mod tests`) | Inline `#[cfg(test)] mod tests {}` | `mod.rs:7-8,26-42`, `agents/mod.rs:12-13`, `games/mod.rs:8-12` | Every module with tests | known |
| 34 | **Shared handle by `Arc` and cheap `Clone`** (Rust idiom) | Borrow `&T`; global `static`; single owner | `gate.rs:249-255` (`GameWriter` is `Clone` with four `Arc`s), `games/catalogue.rs:17-23`, `settings_service.rs:15-19` | All services (48 `Arc<` lines) | known |
| 35 | **Registry by config string** (plug-in lookup) | Compile-time wiring; enum dispatch | `agents/runner.rs:40-63` (`match config.agent_type.as_str()`), `runner.rs:44-48` (a second config of one hook replaces the first) | One place. See M5 | known |

## 2. Meeting points

Each one is a question. None carries a ruling.

**M1. Port or concrete type for storage.** Two outbound systems have a `trait` in `ports/` (concept 1). The third, `Storage`, has none: nine files take `Arc<Storage>` (`gate.rs:8`, `pipeline/job.rs:10`, `games/catalogue.rs:14`, `settings_service.rs:9`, ...). The adapter lives in `adapters/driven/storage/` and `gate.rs:8` also imports its `Tx` and `TxToken`. Is the storage seam meant to be a port, or a concrete type that the tests build for real? (Plan: ticket `29-r1-storage-seam.md`, commit `19d1968e`. The reader did not read it.)

**M2. Single writer or direct `Storage` write.** `GameWriter` says it is "the only code that opens a unit of record writes" (`gate.rs:246`). The catalogues also write: `games/catalogue.rs:83` (`update_game_display_name`), `:134,166,201` (`update_game_config`, inside `config_write`), `games/active.rs:44` (`touch_game_updated_at`), `prompt_preset_service.rs:140` (`delete_preset`), `world_catalogue.rs:45` (`create_world`). Of these, only the config writes go through `config_write`, which refuses them while a run holds the Game (`gate.rs:410-417`). `rename_game` (`catalogue.rs:77-86`) does not. Which writes does the single-writer concept cover: the story and the Job records, or every record of a Game?

**M3. One error envelope or three.** `ApplicationError` serves the catalogues (`errors.rs:6`). `JobFailure`, `Refusal` and `GenerationFailure` serve the Job and the accept (`gate.rs:26,40`). Bare `EngineError` serves the `Result<T>` methods of `SettingsService` (`settings_service.rs:25,29,82`) and `PromptPresetService` (`prompt_preset_service.rs:38,42,63,76`). The same two files also return `ApplicationError` (`settings_service.rs:41,57`; `prompt_preset_service.rs:56,83,97,113,147`). Which method gets which envelope, and who decides?

**M4. Two ways to lift an `EngineError` into `ApplicationError`.** `from_engine_error` keeps a validation on the user-facing variant (`errors.rs:25-30`). `From` keeps every error on `Engine` (`errors.rs:66`). `settings_service.rs:48,75` use the first. `world_catalogue.rs:45-48` writes its own `match` for one variant. `games/catalogue.rs` uses `?` and so the `From` form. The code comment at `errors.rs:22-24` explains the two. Is the manual `match` in `world_catalogue.rs` a third form, or the same as the first?

**M5. Fail fast or degrade, for a missing input.**
(a) A missing preset ends the Job: `prompt_preset_service.rs:62-72` ("A missing preset is an error"), `pipeline/job.rs:490-493`. Commit `aed5eb9d` (R6) says "no registry fallback". (b) The Options prompt keeps a fallback text "when the active options preset is unreadable" (`agents/options/prompt.rs:8-10,33-36`). (c) The after-narration hook turns a failed Agent into the empty result (`agents/runner.rs:69-79`), but the end-of-Job hook returns the error (`runner.rs:84-98`). (d) The Quantifier turns every failure into "static Characters" (`quantifier/utils/orchestration.rs:142-155`), and the Options retry returns `Err` (`options/utils/orchestration.rs:66`). Which of these is the rule for a hook, and which is a leftover? Old shapes: `options/prompt.rs` and `quantifier/utils/parser.rs` date from before the rebuild (`3c5cee0f` 2026-09-11, `d8644394`). The runner and the traits are `e2483e0c` (R4).

**M6. Retry shape.** Options keeps `const MAX_OPTIONS_ATTEMPTS: u32 = 2` (`options/utils/orchestration.rs:16`) and a helper `options_call_attempt` (`:70`). The Quantifier uses a local `let max_attempts = 2` (`quantifier/utils/orchestration.rs:90`) and inlines the loop. The Options loop retries an empty parse and a transport error; the Quantifier loop retries a low-confidence reading and a transport error. Is "two attempts" one rule written twice, or two rules that agree today?

**M7. Parameter object, struct literal, builder.** `PromptContext::new` takes eight arguments under `#[allow(clippy::too_many_arguments)]` (`assembler.rs:107-132`), then `with_guide` and `with_impersonate` set the rest. `NarrationValues`, `OptionsPromptContext` and `QuantifierPromptContext` are built as struct literals (`narrator.rs:29`, `options/utils/orchestration.rs:28-35`, `quantifier/utils/orchestration.rs:45-54`). `narrator.rs:109-118` copies the fields of one parameter object (`NarrationValues`) into the arguments of another constructor. Which form is the default for a prompt input?

**M8. Reads inside the writing service.** `GameViewQuery` says it holds only reads (`view_query.rs:2`). `GameCatalogue` also reads: `list_games`, `active_game`, `newest_game_of_world` (`catalogue.rs:98-120`). Both read the dock: `GameViewQuery::dock` (`view_query.rs:200`) and `GameWriter::dock_of` (`gate.rs:555`). Both read the story tail: `view_query.rs:77-87` and `gate.rs:559-564`. Where do reads go, and why do the two dock reads stay apart? (The writer's read runs inside its hold on purpose, as a comment at `gate.rs:434-435` says for `stop`.)

**M9. Domain term or old name.** The rebuilt Quantifier result says `present` and `destination` (`quantifier/types.rs:58-59`, commit `e2483e0c`). The code around it says `npc_ids`, `determine_npcs_in_room`, `NpcsInArea`, `npcs_in_area`, `NpcCard` (`quantifier/types.rs:30`, `quantifier/utils/orchestration.rs:16`, `options/prompt.rs:57`, `options/types.rs:11`). The glossary lists "NPC" under Avoid for Character. Is the model's wire name (`npcs_in_room`, `quantifier/utils/parser.rs:14`) a reason to keep the term, or are these two cases?

**M10. Booleans and sum types in `Job::run`.** Concept 15 holds in the types. In `Job::run` two local booleans still carry facts: `narration_saved` (`job.rs:156`) and `entry_moved` (`job.rs:165`). Ticket 20 had named "three booleans" as the problem and chose "one fact" (`.scratch/_closed/rust-rebuild-build/issues/20-shape-of-the-job.md`, Q1). Is a local `bool` the "one fact" that the ticket meant?

**M11. Newtype id or raw `u64`.** `GameId` and `RunId` are newtypes (`gate.rs:11,96`). Job ids stay raw: `JobEntry.id: u64`, `JobWriter.id: u64`, `job_id() -> u64` (`gate.rs:91,680,688`). A Message id is a raw `u64` in `switch_swipe` and `edit` (`gate.rs:445,472`). `GameCatalogue` takes `id: u64` and wraps `GameId(id)` inside (`catalogue.rs:67,77,88`), while `GameViewQuery` and `GameWriter` take `GameId` (`view_query.rs:52`, `gate.rs:294`). Where does the newtype begin, and why?

**M12. Primitive tuple or named result.** Named results exist: `AcceptedJob`, `CycleEnd`, `RunJobEnd`, `ConnectionTestResult`, `RoleHealth`, `DebugStateView`. Tuples also exist: `get_current_room_view -> (String, Option<String>)` (`view_query.rs:52`), `get_world -> Option<(i64, WorldCard, MapDef)>` (`world_catalogue.rs:26-29`), `update_connection -> (LlmProviderConfig, bool, bool)` (`settings_service.rs:57`), `backend_info -> (String, String)` (`narrator.rs:73`), `fit_messages_to_context -> (String, String, u32)` (`prompting/utils/context.rs:13`), `build() -> (String, String)` (both prompt builders). `update_connection` returns two bare booleans (`is_narrator`, `is_quantifier`). When does a result get a name?

**M13. Parse to an enum, or keep the string.** `MovementJson.movement_type` is `Option<String>` (`quantifier/utils/parser.rs:22`), and `parse_with_movement` matches lowercase text by hand (`:168-172`). The same struct reads `destination` as a `String`, and the response uses `serde`. `AgentRunner` matches `config.agent_type.as_str()` (`runner.rs:40`). Where does a string become a type: at the `serde` edge or at the match?

**M14. Structured log fields or bracket-tag messages.** `autoplay/driver.rs:48,62,88` and `job.rs:70-78` use `tracing` fields (`game = %game, run = ?run`) and `#[instrument]`. The Agents use text with a bracket tag: `"[Quantifier] Calling provider: ..."` (`quantifier/utils/orchestration.rs:78,97,111`), `"[Options] attempt {attempt}/{} exhausted"` (`options/utils/orchestration.rs:60`). `job.rs:170,397` put the job id inside the text (`"job {}: {line:?}"`). Which style is the default for a new event?

**M15. History rendered three ways.** `<RecentHistory>`/`<Entry sender="..">` in `options/prompt.rs:67-85`; the same block again in `quantifier/prompt.rs:81-99`; `<ConversationHistory>` with `Narrator: text` lines in `prompting/assembler.rs:302-332`. The `<CurrentRoom>` block also appears in both Agent prompts (`options/prompt.rs:43-54`, `quantifier/prompt.rs:57-68`). Is two copies below the line where a shared function is due, and is the third form a different thing?

**M16. Recorded or unrecorded provider call.** All Agent and narrator calls go through `LlmCallRecorder` (`llm_recorder.rs:51`). The connection test calls the provider directly so that it leaves no row (`connection_test_service.rs:1-7,67`). The Quantifier passes the literal `"quantifier"` as the agent name (`quantifier/utils/orchestration.rs:94`); Options passes the constant `AGENT_OPTIONS` (`options/utils/orchestration.rs:77`); the narrator passes `role.agent_name()` (`narrator.rs:132`). `AGENT_DIALOGUE` has a constant but no `Role` (`ports/llm_provider.rs:11`). Which of the four spellings of an agent name is the one to copy?

**M17. Where a prompt helper lives.** `prompting/prompt_merge.rs` sits in `application/` but two adapters import it: `adapters/driven/llm/providers/openrouter.rs:9`, `ollama.rs:9`. An outer layer thus imports an inner module, which concept 2 allows. Is the helper a prompt rule of the application, or a transport rule that sits in the wrong place?

**M18. Check, then act.** `GameCatalogue::delete_game` reads the active pointer and then calls the writer (`catalogue.rs:89-94`). `switch_game` reads the Game, then moves the pointer (`catalogue.rs:68-72`). Each check lies outside the writer's registry hold, where concept 8 puts its own checks (`gate.rs:145`, `gate.rs:285-292`). Does the active pointer belong to the same rule as the job entry?

**M19. Two sources of "busy".** `GameViewQuery::status` reads the registry, and then the Game row for the failure line (`view_query.rs:204-218`). The two reads are not one hold. `get_debug_state` reads the tail twice (`view_query.rs:124-125` through `current_state`, then `tail`) and the dock once. Is a read that crosses two sources meant to be a snapshot?

## 3. Shapes with no name

| # | Shape | Evidence |
|---|---|---|
| S1 | **Persist first, move the in-memory pointer second**: the unit lands before the job entry, the create unit lands before the old Game goes, the pointer moves after the create | `gate.rs:340-343`, `gate.rs:511-514`, `games/catalogue.rs:60-62`, `games/active.rs:31-34` |
| S2 | **One failure, one writer of the line**: a refused request writes the Game's failure line and returns the same failure as the answer | `gate.rs:590-597` (`refuse`), `gate.rs:32-35` |
| S3 | **Hook by position, one Agent per hook**: one trait per hook, a runner with one `Option<Box<dyn>>` per hook, last config wins with a warning | `agents/trait_def.rs:8,17`, `agents/runner.rs:15-18,44-48` |
| S4 | **Agent split into four files**: `agent.rs` (shell), `prompt.rs` (builder), `types.rs` (context and result), `utils/orchestration.rs` (call and retry), `utils/parser.rs` | `agents/options/` and `agents/quantifier/` have the same file list |
| S5 | **A "builder" that only formats**: `OptionsPromptBuilder::new(ctx).build()` returns a pair of strings and never builds an object | `agents/options/prompt.rs:16-27`, `agents/quantifier/prompt.rs:8-19` |
| S6 | **Prompt as a string of tagged blocks**: XML-like tags built with `push_str` | `prompting/assembler.rs` (53 `push_str`), `agents/quantifier/prompt.rs` (20), `agents/options/prompt.rs` (13) |
| S7 | **Parse with two confidences**: the Quantifier keeps an enum `High/Medium/Low` private to itself; "Low" selects the static Characters | `agents/quantifier/types.rs:9-26`, `utils/orchestration.rs:158-185` |
| S8 | **`finish_non_exhaustive` Debug by hand** for a type that holds a closure or a recorder | `agents/options/agent.rs:20-26`, `agents/quantifier/agent.rs:23-29`, `narration/narrator.rs:49-55` |
| S9 | **Constructor named after the config, config unused**: `from_config(_config, recorder)` ignores `_config` and hard-codes the name | `agents/options/agent.rs:29-37`, `agents/quantifier/agent.rs:32-40` |
| S10 | **Join handles in a global-per-app list for tests**: `JobTasks.watch` and `wait`, and one instance in production | `generation/job_tasks.rs:9-44`, `pipeline/job.rs:92`, `autoplay/driver.rs:56` |
| S11 | **Module header as doc anchor**: every file starts with `//! [DOC: path]` | all 54 files |
| S12 | **Static regex helper**: `Lazy<Regex>` with a `compile()` helper in one file and inline `Regex::new(..).expect` in another | `agents/options/utils/parser.rs:9-21` vs `prompting/sanitize.rs:8-17` |
| S13 | **Run-for-run**: a second entry point beside `Job::start` that runs on the calling thread under a run id | `pipeline/job.rs:79-94` vs `:109-123`; commit `749c01a3` (autoplay slice 1) |

## 4. Questions

1. **Old and new side by side.** The rebuild (`ac938c25` P4, `e2483e0c` R4, `7e2eb13a` R8, `749c01a3` autoplay) changed `generation/`, `pipeline/`, `narration/`, `agents/runner.rs`, `agents/trait_def.rs`. The two parsers, both prompt builders, `world_catalogue.rs`, `persona_catalogue.rs`, `text_check_service.rs` and `prompting/builders/sections.rs` are older (their last change is 2026-10-03 or before). Does the catalogue treat the rebuilt shapes as the target and the older ones as the context? Which commit or plan says so? The reader found no plan that names the parsers or the prompt builders.
2. **Docs promise.** `games/active.rs:35-37` says "a request cannot end up writing two Games". Is that a stance (single active Game) or a detail of the pointer?
3. **`TokenBudget::fallback`** (`prompting/token_budget.rs:21`) has no non-test caller in `src/application/`. The reader did not search `tests/` or other layers. Is it live?
4. **`Self::span_of(...)` in `Job::run_for_run`** (`pipeline/job.rs:116`) drops its return value. The function records fields on the current span, so the result is not needed. Is the unused return the intent, or is it a leftover of `Job::start` (`:86`, where the span moves into the thread)?
5. **`Drop` that takes the registry lock** (`gate.rs:843-853`) and a registry lock that is not reentrant (`gate.rs:409`). Is a `JobWriter` dropped inside `hold` a case that the design rules out, and where is that written?
6. **`Busy::Run` after a failed Job.** `RunEnd::JobFailed` ends the run with "no later accept clears it" (`driver.rs:19`). The run hold goes by `Drop` (`driver.rs:106`). Is the order "failure line first, run hold second" something a test pins?
7. **`JobTasks::wait` is `pub async` in production code** (`job_tasks.rs:20`), yet the layer has no other `async`. Is the single async island a test seam by design (concept 32), or the start of a wider move?
8. **Unused `AgentConfig`.** The runner reads `config.enabled`, `config.agent_type` and `config.name`; both Agent constructors ignore the config (S9). Is the config meant to carry more later?
9. **Open hook traits, closed hook set.** There are two hook traits, and the runner builds one `Box<dyn>` per hook from a string. The hook set cannot grow without a code change in `runner.rs`. Is `dyn` here for the tests (`ScriptedModel`, `with_provider`), or for extension?
10. **Two spellings of a default.** `AgentConfig::defaults()` supplies Agents when the list is empty (`runner.rs:28-34`); `AppSettings` has its own defaults elsewhere. Which one owns "the default Agents"?
11. **Naming of the layer folders.** `agents/*/utils/`, `prompting/utils/` and `prompting/builders/` hold the main logic of their module (`orchestration.rs`, `context.rs`, `sections.rs`). `utils/` here is not a drawer of helpers. Does the catalogue need a name for this layout (package by layer inside a feature folder)?
12. **Two roles for `GameCatalogue`.** It is a lifecycle service (create, reset, delete), a config service (posture, mode, presets) and a read service (`list_games`). Is that one concept (catalogue) or three?

## Hazards seen (inferred, not tested)

- `rename_game` writes the display name with no `config_write` guard (`catalogue.rs:77-86`), so it may run while an Autoplay run holds the Game (M2). The effect of that is not known.
- `delete_game` (`catalogue.rs:89-94`) and `switch_game` check the active pointer outside the writer's hold (M18). A race between a switch and a delete is possible in theory. The reader did not try it.
- `GameViewQuery::status` reads two stores in two steps (M19). A Job that ends between the two reads can show an old failure line for one poll.
- `JobWriter::drop` takes the registry lock (`gate.rs:848`). A drop inside a `hold` closure would block on itself. The code today does not do it (the reader checked `accept_in_hold` and `accept`), but nothing in the types prevents it.
- A panic inside a plain `Job` (not a run) runs `JobWriter::drop` during the unwind and frees the entry (`gate.rs:843-853`); the lock is poison-tolerant (`gate.rs:155`). Only the autoplay driver has a comment about a panic (`driver.rs:104`). The reader did not test the plain `Job` path.
- The Quantifier prompt builder takes `QuantifierPromptContext` by value and `quantify_room_with_llm_call` rebuilds a second one field by field from a reference (`quantifier/utils/orchestration.rs:65-74`). A new field in the context needs two edits.
