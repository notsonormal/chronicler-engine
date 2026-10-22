# Research 09: the concepts the docs name

Ticket: [09-research-docs.md](../issues/09-research-docs.md).

## What I read and what I sampled

- **Read in full:** all 23 files under `docs/diataxis/` (1,775 lines): 8 explanation docs and 15 reference docs. Each file declares its mode in front-matter. No tutorial and no how-to exists.
- **Sampled:** `docs/AGENTS.md` (251 lines). It is outside the scope, but it holds the writing conventions of the docs tree (Diátaxis modes, front-matter, "where a fact lives", symbols not line numbers). I used it only as evidence of the shape of the docs. The agent-context copy that came with the task is the source of the line numbers; I re-read the file to cite them.
- **Not opened:** the other research assets, `scripts/validate_docs.py`, `src/` and `tests/`. The ticket says the evidence here is the doc text. This asset does not check any claim against the code. The synthesis ticket does that.
- **Citation form:** a path with no prefix is relative to `docs/diataxis/`. A path that starts with `docs/AGENTS.md` is relative to the repo root.
- **Labels:** known = the doc states the concept or its name. Inferred = the doc shows the shape and I give it the name. Guessed = weak evidence.
- **Domain words** follow `CONTEXT.md` (Game, Swipe, Job, job entry, save unit, Narration, Quantifier).
- **A fact about the doc set:** `rg "DOC:"` finds no DOC anchor in `docs/diataxis/`. The ticket names DOC anchors. See Q1.

## 1. Concepts

Columns: name; opposite; evidence; how far it holds in the docs; label.

### 1a. Architecture stance and pattern (what the docs say about the code)

| # | Concept | Opposite(s) | Evidence | How far it holds | Label |
|---|---|---|---|---|---|
| C1 | Hexagonal architecture (ports and adapters; driving and driven sides) | Layered architecture with a core that calls infrastructure | `reference/architecture_system.md:8,14-15`; `explanation/architecture.md:18-19` | Named in both architecture docs. Storage is a carve-out (MP1). | known |
| C2 | Dependency rule / dependency inversion (the core owns the port traits) | Core imports adapters | `explanation/architecture.md:8`; `reference/architecture_system.md:19-26`; `reference/coding_standards/guardrails.md:48` | Stated as the one rule of the engine. | known |
| C3 | Composition root (only `bootstrap` wires and imports both sides) | Service locator; construction inside the consumer | `reference/architecture_system.md:16,23`; `explanation/architecture.md:46,60-62` | Everywhere in the architecture docs. | known |
| C4 | YAGNI: pay for indirection only where substitution is the point | Speculative generality; one interface per class | `explanation/architecture.md:14`; `explanation/rust_idioms.md:8,12`; `explanation/agent_system_design.md:22` | Stated for storage and for the before-narration hook. Not stated for the `Backend` enum with one variant (MP14). | known |
| C5 | Concrete type over trait; trait object only for heterogeneous dispatch | Trait per collaborator; repository interface; async-trait everywhere | `explanation/rust_idioms.md:8,12`; `explanation/storage_design.md:12`; `explanation/architecture.md:65` | Mostly. Two port traits exist (MP3). | known |
| C6 | Closed-world enum dispatch (`Backend` enum) | Open trait-object dispatch | `explanation/rust_idioms.md:8`; `reference/storage.md:12`; `explanation/storage_design.md:12` | Storage only. | known |
| C7 | Decorator pattern (`BackendKind` wraps one `Backend`) | Per-trait mock structs | `explanation/storage_design.md:14,16`; `reference/storage.md:13-14`; `explanation/rust_idioms.md:8` | Storage only. | known |
| C8 | Make illegal states unrepresentable; enforce by the compiler | Check at run time or by convention | `explanation/storage_design.md:14`; `explanation/agent_system_design.md:10`; `reference/coding_standards/integration_test_standards.md:111-112` | Three claims. The docs say "compile error" or "does not compile" each time. | known |
| C9 | Newtype for an identifier (`GameId`) | Primitive obsession (raw integer) | `reference/storage.md:18-20`; `explanation/storage_design.md:68` | One id type has a newtype. Others are raw (MP11). | known |
| C10 | Capability token (`TxToken`) and information hiding by module privacy | Ambient authority; convention only | `reference/storage.md:49`; `reference/architecture_system.md:67`; `reference/coding_standards/guardrails.md:67-68` | Three sites: the unit token, the `gate.rs` lock, `ActiveGame::set`. | known |
| C11 | Module-per-type locality (an inherent impl lives with its type) | Impls scattered across files | `reference/architecture_system.md:65`; `reference/coding_standards/guardrails.md:77` | Stated as a rule with a guardrail. | known |
| C12 | Single writer principle (one owner for each record kind) | Many writers that each lock | `reference/architecture_system.md:44-59`; `reference/game_flow.md:44`; `reference/coding_standards/guardrails.md:67-68` | Stated for a Game's story records. The Game record has two writers (MP7). | known |
| C13 | Unit of Work (the save unit; `Tx`; whole or not at all) | Statement-by-statement autocommit | `reference/storage.md:44-54`; `explanation/storage_design.md:46-48`; `reference/game_flow.md:8` | Every write of a Game's story. The seed flow is outside (MP5). | known |
| C14 | Pessimistic lock with a short critical section; no lock across a model call | Optimistic concurrency; per-Game locks | `reference/game_flow.md:49`; `explanation/storage_design.md:46,54`; `reference/coding_standards/guardrails.md:66,71` | Two locks are named (MP6). | known |
| C15 | Fencing token: a write checks the job id; a stale writer is refused | No check; preemptive abort of the task | `reference/game_flow.md:47-48,98`; `explanation/rust_idioms.md:34,36` | Every save of a `JobWriter`. | inferred (the docs give no name) |
| C16 | Cooperative cancellation by removing the job entry | Preemptive abort of the running task | `explanation/rust_idioms.md:32-36`; `reference/game_flow.md:98`; `reference/game_flow.md:48` | Stop, reset and delete of a Game. | inferred |
| C17 | Mutual exclusion by admission control: one Job per Game; refuse the second request | Queue the request | `explanation/architecture.md:80`; `reference/game_flow.md:15,50` | Every request that starts a Job. | known |
| C18 | Command–query separation (read handles and `GameViewQuery` for reads; `Tx` for writes) | One repository with mixed reads and writes | `reference/storage.md:39-42,48`; `reference/architecture_system.md:55`; `reference/game_flow.md:94,118` | Story and dock records. Other entities keep `get_*` on `Storage` (MP9). | inferred |
| C19 | Aggregate and consistency boundary (a `Tx` serves one Game; the message aggregate holds Swipes) | A transaction across Games | `reference/storage.md:48`; `explanation/storage_design.md:18,74` | Game scoping is stated for every Game-scoped call. | inferred |
| C20 | Functional core, imperative shell (pure plan and rule functions; steps load, call, return) | Logic interleaved with I/O | `reference/game_flow.md:22,78,145`; `reference/architecture_system.md:12`; `explanation/architecture.md:75` | Plan and game rules. Steps in the Job. | inferred |
| C21 | Rich domain model (pure data and pure rule methods together) | Anemic domain model; rules in services | `reference/architecture_system.md:12`; `reference/game_flow.md:22,28` | Stated for the domain layer. | inferred |
| C22 | Pass references, reload by id (avoid stale copies) | Pass records through the pipeline | `reference/game_flow.md:78`; `reference/coding_standards/guardrails.md:70`; `reference/architecture_system.md:61` | Between Job steps. Agents get values (MP8). | known |
| C23 | Derive, do not store (one source of truth): the current state is derived from the active Swipe | A live state kept in step with the story | `explanation/storage_design.md:80`; `reference/game_flow.md:104-106,114`; `explanation/two-state-channels.md:8` | Current state, room, status. A few stored derivations (MP12). | known |
| C24 | Snapshot per version (Memento): each Swipe holds its own Swipe state | One mutable state with undo and restore | `explanation/storage_design.md:78-80`; `reference/game_flow.md:104,110` | Every Swipe. | inferred |
| C25 | Append-only, non-destructive history: a Retry adds a Swipe and keeps the old one | Update in place | `explanation/storage_design.md:74`; `reference/game_flow.md:149`; `reference/frontend/dashboard.md:55` | Retry. Edit and delete change records. | inferred |
| C26 | Linear history: only the last Message can take a new Swipe | Branching history (tree or DAG) | `explanation/storage_design.md:86-90`; `reference/frontend/dashboard.md:55`; `explanation/dashboard_design.md:11` | Stated with its reason. | inferred |
| C27 | Data Mapper / anti-corruption boundary: manifest type converts to runtime card; newtype unwrapped at the row | Shared type across layers; active record | `explanation/storage_design.md:26`; `reference/startup.md:80`; `reference/storage.md:19` | Seed path and Game ids. | inferred |
| C28 | Database seeding: seed once, then the database is the source of truth | Read files at runtime; sync both ways | `explanation/storage_design.md:22-24,40`; `reference/startup.md:21-24` | Worlds, Personas, presets, settings. | known |
| C29 | Idempotent operation (seed checks the primary key) | Re-run fails or duplicates | `reference/startup.md:23`; `explanation/storage_design.md:34` | Seeding only. | known |
| C30 | Singleton row for configuration, read at the point of use | Config cached at start; config in the environment or a static | `explanation/rust_idioms.md:20-24`; `reference/architecture_system.md:36`; `reference/storage.md:65` | Settings. A test doc says a global static (MP4). | known |
| C31 | Fail fast / fail loud | Graceful degradation; skip and continue | `reference/startup.md:33`; `explanation/storage_design.md:28`; `reference/architecture_system.md:36` | Settings seed, missing connection. | known |
| C32 | Tolerant reader (skip the bad file, continue) | Fail fast | `reference/startup.md:35`; `explanation/storage_design.md:38`; `reference/game_flow.md:165` | Seed files, text check. See MP10. | inferred |
| C33 | Typed error with context (`EngineError::Database { context, source }`; typed not-found variants) | Stringly errors; panics | `reference/storage.md:76`; `explanation/storage_design.md:66-68`; `reference/game_flow.md:22` | Storage and the 400/500 refusals. | known |
| C34 | Explicit absence policy: `get_*` returns an option, `require_*` returns an error | One lookup with a sentinel value | `reference/storage.md:67-72`; `explanation/storage_design.md:60-66` | Entities where the choice matters. Characters use a roster read. | known |
| C35 | Recover from a poisoned lock (`into_inner`) | Let it crash; propagate the poison | `explanation/rust_idioms.md:28-30`; `explanation/architecture.md:78`; `reference/coding_standards/integration_test_standards.md:155-156` | "Every `Mutex` site". | known |
| C36 | Deny panics in production code (clippy) | `expect` for "cannot happen" | `reference/coding_standards/guardrails.md:12,17-18,24` | Production code. Tests are exempt. | known |
| C37 | Accept now, process in the background; poll for status | Synchronous request and response | `reference/game_flow.md:8,12`; `explanation/rust_idioms.md:16,18`; `explanation/dashboard_design.md:20` | Every Job. | known |
| C38 | Synchronous core on the blocking pool (`spawn_blocking`) | Async all the way (async-trait) | `explanation/rust_idioms.md:12,14-18`; `explanation/architecture.md:77` | Job and its collaborators. | known |
| C39 | Hypermedia: the server renders HTML fragments; thin client | Single-page app with a JSON API and client state | `reference/frontend/dashboard.md:12-14`; `explanation/dashboard_design.md:18`; `reference/frontend/http_routes.md:48-51` | Every region and panel. | known |
| C40 | Polling over push (WebSocket and SSE considered) | Push channel | `explanation/dashboard_design.md:20,25`; `reference/frontend/dashboard.md:32` | Live regions. | known |
| C41 | Single-process deployment; process-local state | Shared-nothing, externalised state, horizontal scale | `explanation/architecture.md:69`; `explanation/two-state-channels.md:14`; `reference/game_flow.md:40` | A deployment contract. | known |
| C42 | Ephemeral state: do not save what a crash makes wrong (the busy mark) | A persisted status column | `explanation/two-state-channels.md:14`; `explanation/architecture.md:79`; `reference/game_flow.md:40`; `reference/startup.md:43` | Busy, step, run hold. | inferred |
| C43 | Separate stores for separate lifetimes (job entry in memory; failure line on the record) | One status field | `explanation/two-state-channels.md:8,12,18`; `reference/game_flow.md:92` | The status of a Game. | known |
| C44 | Strategy / plugin registry by hook trait; open–closed | Conditionals in the Job | `explanation/agent_system_design.md:8,10`; `reference/narrative/agent_system.md:8-9,19-21` | Agents. A new type still needs a match arm (MP3). | known |
| C45 | Optional component (zero Agents is a valid setup) | Mandatory component | `explanation/agent_system_design.md:8`; `reference/narrative/agent_system.md:11-12` | Agents. | known |
| C46 | Ubiquitous language (bold glossary terms: Job, job entry, run hold, save unit) | Ad-hoc synonyms | `reference/game_flow.md:8,32,34,98`; `reference/frontend/dashboard.md:28,34` | Reference docs. | inferred |
| C47 | Observability: record every model call, failures included; derive health from the record | Keep a health flag | `reference/frontend/dashboard.md:49,63`; `reference/storage.md:102`; `reference/narrative/narration_system.md:26` | Role health. | inferred |
| C48 | Untrusted input at the boundary: sanitize input and output of the model | Pass text through | `reference/narrative/prompt_system.md:43`; `reference/narrative/narration_system.md:26`; `reference/coding_standards/testing.md:32` | Player input; model output. | inferred |
| C49 | Separate instructions from data (system half and user half; labelled sections) | One concatenated prompt | `explanation/prompt_system_design.md:8,10,32-36`; `reference/narrative/prompt_system.md:8` | Every Narration call. | known |
| C50 | Layered pipeline of prompt sections (eight layers; two conditional) | Monolithic template | `reference/narrative/prompt_system.md:12,14-23` | Narrator prompt only. Agents build their own (prompt_system.md:51). | inferred |
| C51 | Template substitution of author-controlled text | Static text | `reference/narrative/prompt_system.md:47`; `reference/narrative/prompt_system.md:12` | Presets. See MP15. | known |

### 1b. Test and enforcement stance (what the docs say about how the code is held)

| # | Concept | Opposite(s) | Evidence | How far it holds in the docs | Label |
|---|---|---|---|---|---|
| C52 | Architecture fitness functions (arch-lint and syn walkers as tests) | Review only | `reference/coding_standards/guardrails.md:8,56,60`; `explanation/architecture.md:18` | Three layers of check. | known |
| C53 | Defense in depth (stated for the real-LLM tests; layered for guardrails) | One check | `reference/coding_standards/integration_test_standards.md:262-265`; `reference/coding_standards/guardrails.md:8` | Two sites. | known |
| C54 | Table-driven rules with a reason on each row | Ad-hoc rule code | `reference/coding_standards/guardrails.md:60,65-71` | The four walkers. | known |
| C55 | Rules tested by paired positive and negative cases | Untested rules | `reference/coding_standards/integration_test_standards.md:134-146` | Pattern 7. | known |
| C56 | Test tiers (unit, tier-1, stub browser, full-stack browser, real LLM) | One flat suite | `reference/coding_standards/testing.md:12-16,18`; `reference/coding_standards/integration_test_standards.md:6-8` | See MP17. | known |
| C57 | Real dependency where behaviour matters (SQLite `:memory:`) | A fake that mimics | `explanation/storage_design.md:56`; `explanation/rust_idioms.md:10`; `reference/coding_standards/integration_test_standards.md:16-19` | All storage tests. | known |
| C58 | Strict mock (fails on an unscripted call) | Lenient stub | `reference/coding_standards/integration_test_standards.md:226-229,246-248,257` | `ScriptedModel` against `MockBackend`. | inferred |
| C59 | Fault injection by typed key (`StorageCall`, `TxWrite`) | Per-trait mock structs | `reference/storage.md:50-51`; `reference/coding_standards/integration_test_standards.md:108-114`; `explanation/storage_design.md:16` | Storage tests. | known |
| C60 | Test data builder (`TestAppBuilder`) | Setup in each test | `reference/coding_standards/integration_test_standards.md:16-27,207-213` | Tier-1 and HTTP tests. | known |
| C61 | Condition-based waiting; no fixed sleep | Sleep | `reference/coding_standards/testing.md:28`; `reference/coding_standards/integration_test_standards.md:67-68` | Browser and HTTP tests. | known |
| C62 | Test through the public interface (`oneshot` router; start a Job by request) | Call the service direct | `reference/coding_standards/integration_test_standards.md:50-55,90-94` | Job starts. | known |

### 1c. The shape of the docs themselves

| # | Concept | Opposite(s) | Evidence | How far it holds | Label |
|---|---|---|---|---|---|
| C63 | Diátaxis: organise by reader need | Organise by topic | `explanation/diataxis.md:10-12`; `docs/AGENTS.md:107,125` | 23 of 23 docs declare `reference` (15) or `explanation` (8). | known |
| C64 | Reference describes only; the reason goes to a paired Explanation doc | Reference with rationale | `docs/AGENTS.md:178-180`; `reference/storage.md:152-156`; `reference/game_flow.md:169-172` | Pairs: game_flow↔two-state-channels, storage↔storage_design, agent_system↔agent_system_design. See MP13. | known |
| C65 | Explanation unfolds; it does not justify (no "Why X?" titles) | Apologia sections | `docs/AGENTS.md:201-207`; `explanation/storage_design.md:10,20,44` | `rg "^#+ Why"` finds no title. | known |
| C66 | Docs as code, machine-validated (front-matter, links, line-citation ban) | Docs checked by eye | `docs/AGENTS.md:91-97,160-162`; `reference/coding_standards/guardrails.md:60` | Three-layer model. | known |
| C67 | Generated documentation | Hand-written reference | `reference/frontend/http_routes.md:8`; `reference/coding_standards/guardrails.md:14,32,62`; `docs/AGENTS.md:44` | Three docs have generated blocks. | known |
| C68 | Single home per fact | Restate in many docs | `docs/AGENTS.md:25,34,40`; `reference/game_flow.md:94`; `reference/startup.md:92-95` | See MP12. | known |
| C69 | Cite symbols, not line numbers | `path:line` citations | `docs/AGENTS.md:158-162`; `reference/coding_standards/guardrails.md:65` (generated, exempt) | All hand-written docs cite symbols. | known |
| C70 | Layers of record: code and unit tests, then integration tests and specs, then docs | One doc set | `docs/AGENTS.md:11-15,19,21` | Stated rule. | known |
| C71 | Edit direction: a behaviour change edits spec and test; a doc changes only when a part, invariant or name changes | Doc edited with every change | `docs/AGENTS.md:36,38` | Stated rule. | known |
| C72 | Diagram as code (Mermaid flowchart; one diagram per cluster) | Binary diagrams; one big diagram | `docs/AGENTS.md:164-172,219`; `reference/storage.md:109,125`; `reference/game_flow.md:60` | 4 diagrams in 3 docs. | known |
| C73 | Link to a living example instead of copying a snippet | Inline snippets | `reference/coding_standards/integration_test_standards.md:8-11,46-48` | Test standards. | known |
| C74 | Seam identifier versus mechanics leak (the grep test) | Code-indexer docs | `docs/AGENTS.md:228,234-237` | See MP16. | known |

Count: 74 concepts (C1–C74).

## 2. Meeting points

Each point is a question with evidence. None is a ruling.

**MP1. Which rule governs a new persistence call: the port rule or the direct-call rule?**
The dependency rule says the application layer reaches storage "through port traits" (`explanation/architecture.md:14`). The same paragraph says a one-impl trait "is rejected as phantom". Later the doc calls `Storage` "concrete-by-design" (`explanation/architecture.md:65`). The diagram draws `app --> storage` as a direct edge (`explanation/architecture.md:53`). The reference says "Application code may call `Storage` directly when the call is part of the persistence boundary. Architectural review owns the discipline for which files sit on the boundary" (`reference/architecture_system.md:30`). Guardrails then forbid `server → storage` and `storage → application` (`reference/coding_standards/guardrails.md:47,51`). Two reasons are given for a concrete `Storage`: no second impl (`explanation/architecture.md:14`) and the Rust 2024 async-trait cost (`explanation/rust_idioms.md:12`). Which reason holds when a file wants to call `Storage`, and who lists the files on the boundary?

**MP2. When does a boundary get a fake, a decorator, or the real thing?**
Tests "substitute fakes at the trait boundary" (`explanation/architecture.md:10`). Storage tests use the real SQLite with a failure-injection decorator because "a fake would only mimic" rollback (`explanation/storage_design.md:56`). The LLM port has a strict fake that fails on a stray call (`reference/coding_standards/integration_test_standards.md:226-229`) and a lenient `MockBackend` that dev runs and the browser tier use (`reference/coding_standards/integration_test_standards.md:257`). The doc says a trait repository would add "the trait boilerplate and per-test custom mocks" (`explanation/rust_idioms.md:12`). What decides the choice for a new boundary?

**MP3. Four ways to cross a boundary: trait, closure, enum, concrete type. What picks one?**
Port traits: `LlmProvider`, `TextChecker` (`explanation/architecture.md:65`). Closure seam: `SaveLlmMessageFn` (`explanation/architecture.md:65`; `reference/architecture_system.md:58`). Enum: `Backend` (`explanation/rust_idioms.md:8`). Trait object for heterogeneous dispatch: `Box<dyn AfterNarrationAgent>` (`explanation/rust_idioms.md:8`). Concrete structs for the rest (`explanation/rust_idioms.md:8`). A new Agent type "needs an arm in `AgentRunner::from_configs`" (`explanation/agent_system_design.md:10`), so the open trait and a closed match meet in one place. Which one fits a new seam, and is the match arm part of the open–closed claim?

**MP4. Is `AppSettings` a database row, a global static, or both?**
The singleton row is read "at the point of use"; "nothing holds a long-lived settings handle" (`explanation/rust_idioms.md:22`). The reference says the row is "the read source for the whole engine" (`reference/architecture_system.md:36`). The test standard says "`AppSettings` lives in a global static, so parallel tests would race" (`reference/coding_standards/integration_test_standards.md:152-153`). A key can also come from the environment (`reference/coding_standards/testing.md:8`). Settings can come from file, database, environment and a static. Where does each one apply?

**MP5. Which atomicity mechanism applies where?**
A save unit (`reference/storage.md:44-54`). One storage acquisition for read-mutate-write of settings (`explanation/rust_idioms.md:22`). One hold of the registry for a check and its units (`reference/game_flow.md:49`). A reset: "Both writes land in one hold of the registry" (`reference/game_flow.md:100`) while a create is "one save unit" (`reference/game_flow.md:124`). The seed flow "sits outside this rule" (`explanation/storage_design.md:58`). The dock record is a table of its own, "because the config routes write the Game record with no busy check" (`explanation/storage_design.md:50`). Which write uses which mechanism, and where is the list?

**MP6. How do the two locks nest?**
The registry hold covers "one check and its save units" (`reference/game_flow.md:49`). A unit "holds the backend mutex for its whole run" (`explanation/storage_design.md:46`). A `Storage` call in a unit closure waits on the mutex the unit holds (`explanation/storage_design.md:54`). A guardrail row says "Look again when a second lock … arrives" (`reference/coding_standards/guardrails.md:71`). Each unit takes the mutex, and a hold covers the units, so the mutex nests inside the registry hold (inferred from these lines). Where is the lock order written, and what stops the reverse order?

**MP7. One writer, or one writer per record kind? The Game record has more than one.**
`GameWriter` writes "every record of a Game's story" (`reference/architecture_system.md:48`). `GameCatalogue` writes "the config columns of the Game record" (`reference/architecture_system.md:54`). `GameWriter::refuse` writes the failure line on the same record (`explanation/two-state-channels.md:18`). `PromptPresetService` writes preset rows "through `Storage`" (`reference/architecture_system.md:56`). During a run hold, a write of the config columns "is refused as busy" (`reference/game_flow.md:38`). The storage doc says the config routes write "with no busy check" (`explanation/storage_design.md:50`). How do the two writers of one Game record avoid each other, and do the two sentences about the busy check describe the same writes?

**MP8. Ids between steps, values into Agents. Where is the line?**
"Between steps, pass ids" and no function of `job.rs` takes a record (`reference/coding_standards/guardrails.md:70`; `reference/game_flow.md:78`). An Agent "reads values, never storage" (`reference/narrative/agent_system.md:33`). The narrator "works from the values that it is given" (`reference/narrative/narration_system.md:22`). `SavedText` carries a Swipe from the text save to the state update (`reference/game_flow.md:108`). Is a step the only place where an id becomes a value?

**MP9. Three read surfaces for stored data.**
`get_*` and `require_*` on `Storage` (`reference/storage.md:67-72`). Read handles `stories()` and `docks()` (`reference/storage.md:39-42`). `GameViewQuery` for the status rule and the story window (`reference/game_flow.md:94,118`). The characters use a roster read instead of a pair (`explanation/storage_design.md:70`). Which entity uses which surface, and do the Game-scoped reads follow the `get_`/`require_` pair?

**MP10. Which failure stops, which is skipped, which falls back?**
A malformed `settings.json` halts boot (`reference/startup.md:33`). A malformed world, Persona or preset file is skipped (`reference/startup.md:35`). A connection reference that names nothing "fails the operation … instead of falling back" (`reference/architecture_system.md:36`). A text-check failure "sends the original Action" (`reference/game_flow.md:165`). With no scenario, the initial room id "falls back to `"start"`" (`reference/startup.md:45`). A failed Options run "goes to the log" while other failures reach the failure line (`reference/game_flow.md:18`). Where is the line between fail loud, skip, fall back and log?

**MP11. Which entities use which identity?**
`GameId` is a newtype (`reference/storage.md:18-20`). Game and message ids are numeric (`explanation/storage_design.md:68`). Worlds and Personas use string keys (`explanation/storage_design.md:68`). Characters reference "the world's numeric id" (`explanation/storage_design.md:42`) while a character has a unique `key` within its world (`reference/storage.md:97`). Presets use a text `id` (`reference/storage.md:103`). `games.world_key` and `games.persona_key` are logical references, not foreign keys (`reference/storage.md:140-141`). Within the same schema, tables join by cascade foreign keys (`reference/storage.md:116-118`). Which rule picks a newtype, a numeric id or a key, and which links get a foreign key?

**MP12. A stored derivation or a recomputed one?**
The docs say the current state is derived, with "nothing copied or restored" (`explanation/storage_design.md:80`). The Swipe state stores `currently_meeting` and `times_met`, which the game rules update from a diff (`reference/game_flow.md:139`; `explanation/agent_system_design.md:30`). The room name of a Message is derived from the Swipe state when it is read (`reference/game_flow.md:120`). Retrigger is a derived rule on `Tail` (`reference/game_flow.md:28`), while the event record is stored (`reference/game_flow.md:112`). Which derived values does the system store, and which does it recompute at read time?

**MP13. A Reference doc that gives a reason; an Explanation doc that gives a mechanism.**
The rule: Reference describes "and only describes" (`docs/AGENTS.md:178`). Reference sentences that carry a reason: "so a swipe switch needs no second record" (`reference/storage.md:121`); the Reason column of the guardrail rows (`reference/coding_standards/guardrails.md:66-71`); the coverage reason (`reference/coding_standards/guardrails.md:108`). Reference docs that carry instructions: the Playwright setup and a command block (`reference/coding_standards/testing.md:12,20-22`); "call the local `cleanup_db_for_port(port)` first" (`reference/coding_standards/integration_test_standards.md:128`). Explanation docs that carry a step-by-step mechanism: the cancellation protocol (`explanation/rust_idioms.md:34-36`); the `Tx` rules (`explanation/storage_design.md:46-58`). Where does a reason sit when the Reference row is generated from code comments? Where do instructions go when no how-to doc exists (`docs/AGENTS.md:209-215`)?

**MP14. A seam kept ahead of need, against YAGNI.**
The docs state the YAGNI rule (`explanation/architecture.md:14`). They also keep: a `Backend` enum with one variant (`explanation/rust_idioms.md:8`); `BackendSelector`, "metadata today … a seam the design has not closed" (`explanation/agent_system_design.md:18`); a before-narration hook named before its first Agent (`explanation/agent_system_design.md:22`); and "four LLM impls" (`explanation/architecture.md:12`) beside a diagram of three (`explanation/architecture.md:40-42`) and a Mock backend (`reference/coding_standards/integration_test_standards.md:257`). Which kept seams are the substitution point, and which wait for a need?

**MP15. One delimiter, two jobs: `{{…}}`.**
The assembler replaces any `{{…}}` in player input with `[FILTERED]` (`reference/narrative/prompt_system.md:43`). The template engine substitutes variables into author-controlled preset fields at render time (`reference/narrative/prompt_system.md:47`), and the impersonate preset reaches persona data "through the impersonate preset's template macros" (`reference/narrative/prompt_system.md:12`). Player input also sits inside `<PlayerInput>` (`explanation/prompt_system_design.md:8`). Does the filter or the XML wrapper hold the trust boundary, and do the docs say what happens to a closing tag in player text?

**MP16. Code-indexer docs against seam identifiers.**
The rule keeps "values in the code" and drops "bare free-function names" (`docs/AGENTS.md:232,237`). Hand-written reference docs name files and functions: `generation/gate.rs`, `pipeline/job.rs`, `narration/narrator.rs` (`reference/architecture_system.md:48-51`); `game_rules::apply` with its full path (`reference/game_flow.md:145`); `stories.rs`, `docks.rs` (`reference/storage.md:41-42`); `default_scenario()` (`reference/startup.md:45`). `http_routes.md` lists every handler, and that list is generated (`reference/frontend/http_routes.md:8,12-110`). The rule also says "cite symbols, not line numbers" (`docs/AGENTS.md:158-162`) while generated guardrail rows carry `file:line` (`reference/coding_standards/guardrails.md:65-71`; exempt at `docs/AGENTS.md:162`). Which file names are seam identifiers, and does a generated list count as an index?

**MP17. How many test taxonomies are there?**
"tier-1" (`explanation/storage_design.md:18,56`; `reference/coding_standards/integration_test_standards.md:226,252`) is used with no definition in `docs/diataxis/`; the placement rule sits in `tests/STRATEGY.md` (`reference/coding_standards/testing.md:18`). Test binaries split by fixture weight: architecture, guardrails, storage, bootstrap, http, browser, llm (`reference/coding_standards/integration_test_standards.md:6-7`). The browser binary has a stub tier and a full-stack tier (`reference/coding_standards/testing.md:14,16`). Seven patterns and eight cross-cutting patterns (`reference/coding_standards/testing.md:36`). Which axis does each name cut, and where is the map between them?

**MP18. Shared test state with a lock, or an instance per test?**
A process-wide `Mutex<()>` guards the settings global (`reference/coding_standards/integration_test_standards.md:152-157`). Each test gets its own `:memory:` storage (`explanation/storage_design.md:18`) and each stub check gets "a fresh stub server and a fresh page" (`reference/coding_standards/testing.md:14`). Ports come from a file-locked range 3010–3050 (`reference/coding_standards/integration_test_standards.md:159-168`), and some tests bind an OS-assigned port and "opt out … by design" (`reference/coding_standards/integration_test_standards.md:170-171`). Which resources are shared and locked, and which are per test?

**MP19. How many status and health signals exist, and where do they meet?**
Two state channels answer "is a Job running?" and "did the last Job fail?" (`explanation/two-state-channels.md:8`). Role health comes from the LLM Messages record, engine-wide (`reference/frontend/dashboard.md:49`). Unreachable is a client-side state (`reference/frontend/dashboard.md:14-20,47`). The header refresh owns every role-health display (`reference/frontend/dashboard.md:49`). Does "two channels" count the role health and Unreachable, and who owns the join of all four?

**MP20. When does a model call end: the stop, the shutdown?**
After a stop, "a model call that runs when the stop comes runs to its end, and the id check refuses its answer" (`reference/game_flow.md:98`). The Job runs on the blocking pool after the response (`explanation/rust_idioms.md:16,18`). On shutdown the server "serves the requests it already holds until they end" (`explanation/rust_idioms.md:38`). Do the docs say what happens to a Job that runs after its request ended, when the process stops? A restart "leaves every Game idle" (`explanation/architecture.md:79`).

**MP21. Two definitions of the seed shape.**
The JSON Schema files are "authoritative for seed-file fields and validation" (`reference/startup.md:68`). The same doc names Rust types (`CharacterSheet`, `WorldManifest`, `MapDef`, `AppSettings`) as what each schema defines (`reference/startup.md:61-66`). After the first seed, "the JSON file … is no longer consulted" (`explanation/storage_design.md:40`). Which one wins when schema, type and database row differ, and how does an author change a World that already has a row?

**MP22. Which diagram style: Mermaid flowchart only, or also C4?**
The convention bans C4 for data tables (`docs/AGENTS.md:170`) and lists C4 directives with layout settings (`docs/AGENTS.md:221`). The docs under `docs/diataxis/` use flowcharts only (`explanation/architecture.md:20`; `reference/game_flow.md:60`; `reference/storage.md:109,125`). Is C4 allowed for component views?

## 3. Shapes with no name

| # | Shape | Evidence |
|---|---|---|
| S1 | The state of "busy" is the presence of an in-memory entry that also acts as the owner's token (job entry; run hold). | `reference/game_flow.md:32,34`; `explanation/two-state-channels.md:12` |
| S2 | A sticky failure line on the Game record. It survives a restart. Only the next player action clears it. | `explanation/two-state-channels.md:18,20`; `reference/game_flow.md:92` |
| S3 | A refusal (`500`) also writes its reason to the record before it answers. | `explanation/two-state-channels.md:18`; `reference/game_flow.md:22` |
| S4 | One writer writes one record twice (pre-analysis state, then the analysed state). A crash in between leaves a defined intermediate. | `explanation/storage_design.md:84`; `reference/game_flow.md:108-109` |
| S5 | Each kind of Agent result has one landing, and the hook trait fixes the result type. | `reference/narrative/agent_system.md:19-21`; `explanation/agent_system_design.md:10,22` |
| S6 | The model reports what it sees (a list). The engine computes the transition (a diff of two lists). | `explanation/agent_system_design.md:26-30`; `reference/narrative/agent_system.md:44`; `reference/game_flow.md:134-139` |
| S7 | A prompt carries a condensed roster of all Characters and full cards only for the Characters in the room. | `explanation/prompt_system_design.md:16-20`; `reference/narrative/prompt_system.md:18` |
| S8 | Constraints sit after the history and before the player input, near the point of generation. | `explanation/prompt_system_design.md:24-28`; `reference/narrative/prompt_system.md:12` |
| S9 | A "Who owns what / may write" table: one row per part, with a column that limits its writes. | `reference/architecture_system.md:44-59` |
| S10 | A numbered design rule that is a data row (kind, reason, source), and a doc table that is generated from the rows. | `reference/coding_standards/guardrails.md:60,65-71` |
| S11 | The narrator is narrative-only; "state mutation is the engine's job". | `reference/narrative/narration_system.md:18`; `explanation/agent_system_design.md:26` |
| S12 | The product borrows a named outside model (SillyTavern layers, swipes, chat layout) for its structure. | `explanation/dashboard_design.md:8-13`; `reference/narrative/prompt_system.md:12,14` |
| S13 | A seed file is a template that the engine reads once; later file edits do not reach an existing row. | `explanation/storage_design.md:40`; `reference/startup.md:23` |

Count: 13.

## 4. Questions

1. **Q1. Where do DOC anchors live?** The ticket names DOC anchors as part of the doc shape. `rg "DOC:"` finds none in `docs/diataxis/`. Only one row mentions them: "module-level doc-anchor standards on production files" (`reference/coding_standards/guardrails.md:87`). Does any doc rely on an anchor in `src/`?
2. **Q2. What do the arch-lint scope names map to?** The rows use `model`, `narrative`, `server`, `agent-edge`, `storage-models` (`reference/coding_standards/guardrails.md:35-51`). The layer list uses `domain::model`, `application`, `adapters::driving` (`reference/architecture_system.md:12-15`). A reader cannot see the map in the docs.
3. **Q3. Where are the design rule numbers kept?** The guardrail rows cite Rules 1, 2, 3, 6, 7, 13 and 18 (`reference/coding_standards/guardrails.md:65-71`). The source of the numbers is a `.scratch` plan (`reference/coding_standards/guardrails.md:114`). The convention exempts plans (`docs/AGENTS.md:241`). Do rules 4, 5, 8–12 and 14–17 exist, and where?
4. **Q4. Where are the decisions recorded?** The architecture doc says two port traits "are accepted" (`explanation/architecture.md:65`) and points to a "hexagonal-reorganization superplan" (`reference/coding_standards/guardrails.md:54`). Is there a record of decisions, or only the docs and the plans?
5. **Q5. Does `docs/AGENTS.md` list the Document References that the docs carry?** The rule lists "only the paired … doc" (`docs/AGENTS.md:40`). `explanation/rust_idioms.md:42-46` lists five docs. `reference/coding_standards/testing.md:36-39` lists four. Five reference docs have no section at all (`reference/narrative/options.md`, `ai_steering.md`, `narrator_mode.md`, `narration_system.md`, `reference/frontend/http_routes.md`).
6. **Q6. Which place holds the page title: H1 or front-matter?** The rule says the front-matter title mirrors the H1 (`docs/AGENTS.md:132`). Three reference docs have an H1 (`reference/narrative/options.md:6`; `reference/narrative/ai_steering.md:6`; `reference/narrative/narrator_mode.md:6`). The other 20 start at an H2.
7. **Q7. Is the `settings` table a standalone table?** The cluster list names "settings, prompt presets, and LLM call forensics" (`reference/storage.md:84`). The standalone list describes only `llm_messages` and `prompt_presets` (`reference/storage.md:102-103`), and the settings row is described elsewhere (`reference/storage.md:65`).
8. **Q8. What does "newest Game of a World" mean?** The newest Message decides, and `games.updated_at` decides when the Game has no Message (`reference/storage.md:59-60`). A Game switch touches `updated_at` (`reference/game_flow.md:126`). Is "newest" by creation or by last use?
9. **Q9. What covers a change to the prompt or the LLM parser?** The real-LLM tests are `#[ignore]`; "CI does not exercise the ignored tests" (`reference/coding_standards/integration_test_standards.md:262,268-269`). The doc asks for a local run.
10. **Q10. How do the negative statements fit the "no negative explaining" rule?** The rule bans defensive negatives (`docs/AGENTS.md:243-250`). The docs contain: "are **not** SQL foreign keys" (`reference/storage.md:138`); "never blocks an Action" (`reference/game_flow.md:165`); "rejected as phantom" (`explanation/architecture.md:14`). Which of these are the allowed "positive constraint" form?
11. **Q11. Which claim carries the reason for a concrete `Storage`?** The docs give two reasons (MP1). If only one holds, does the other still stand as a doc sentence?
12. **Q12. What does the ownership table not list?** `AppState` and `CurrentGame` appear in other places (`reference/architecture_system.md:40`; `reference/storage.md:24`). The "Who Owns What" table (`reference/architecture_system.md:46-59`) has no row for either.

Count: 12 questions.
