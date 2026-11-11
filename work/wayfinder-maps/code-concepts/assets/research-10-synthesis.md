# Research 10: the areas put together

Ticket: [10-research-synthesis](../issues/10-research-synthesis.md). Map: [map.md](../map.md).
The map Notes apply: questions first, no rulings. Terms follow `CONTEXT.md`.

## 0. How this was built

**Inputs.** The eight area assets `research-02-domain.md` to `research-09-docs.md` and the `## Answer` section of each ticket 02 to 09 (they hold the coordinator corrections). I did not open `research-01-lens.md`. I did not open the other `research-*` files that parallel researchers write.

**Area names used in the table.** Domain (02), Application (03), Driven adapters (04), Driving adapters (05), Composition root and cross-cutting (06), Test infrastructure (07), Test bodies (08), Docs (09).

**Row numbers in the cells.** Each cell gives the area's "how far it holds" in a few words and the row number of the area asset. The number form follows the asset: `#n` for 02, 03 and 04; `Cn` for 05, 06, 08 and 09; `Tn` or `Gn` for 07. `MP-n` (05 to 08) and `Mn` (03) or `MPn` (09) point to a meeting point in that asset when the area has no concept row for the idea. A cell with `—` means the area did not name the concept.

**How names were merged.** I merged rows when two areas name the same shape under different names, or when one area names the concept and another names its opposite for the same kind of choice. I kept one row for each side when the two sides are both in use in the code (for example "fail fast" and "graceful degradation"). I put the names the areas used in the first column when they differ. I did not merge a concept only because the words are close.

**Levels** (from the map Notes): *stance* (a discipline that shapes the whole code base), *pattern* (a named design pattern or architecture style) and *idiom* (a language-level or file-level habit).

**Spread.** I count only the code areas 02 to 08 (seven). *all code areas* means five or more of them; *several* means two to four; *one* means one. A `+docs` tag means area 09 also names it. The Docs area describes the code and cites doc text, so it does not count for spread or for the label.

**Label.** *known*: two or more code areas (02 to 08) cite code for the concept. *inferred*: one code area cites code, or the code areas cite it only as the opposite of another concept, or the evidence is an absence. *guessed*: I am not sure the merge is right; a footnote says why. This label can differ from the label in an area asset. An area marks a concept "known" when its own lines show it; this table needs a second area before it says "known".

**Single-area concepts.** The tables in 1.1 to 1.3 hold every concept that two or more areas name (97 merged rows). Every other concept sits in the compact lists in 1.4 (by area, by row number), so that all 398 concept rows of the eight assets are reachable from this file. A script computed the lists in 1.4 from the cell references, so that no row is lost.

**What I did not do.** I did not rule on any concept or meeting point. I did not read the lens asset. I did not re-read every code line behind a citation. I spot-checked 14 citations against the code (listed in section 2). I did not check the docs against the code; the parallel file `research-10-docs-vs-code.md` does that.

**Moved lines (known).** Between the research and this synthesis, some lines moved. Example: asset 03 cites `gate.rs:246` for "the only code that opens a unit"; the doc line is now `gate.rs:240`. I kept the asset citations and give the current line only when I checked it.

## 1. Concepts across areas

### 1.1 Stance

| Row | Concept (names used) | Opposite(s) | Domain 02 | Application 03 | Driven 04 | Driving 05 | Composition 06 | Test infra 07 | Test bodies 08 | Docs 09 | Spread | Label |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| X01 | **Ports and adapters** (hexagonal; "hexagonal core: no I/O") | Direct call to a concrete infrastructure; core that calls infrastructure | core has no I/O, no trait, `#17` | LLM and text check only; storage has no port, `#1` | same two ports; storage none, `#1` | everywhere for storage, `C1` | — | fake at the port, `T3` | test seam, `C8` | holds; storage carve-out, `C1` | all code areas +docs | known |
| X02 | **Dependency rule** (inward imports) | Any-to-any imports | mostly; the domain knows UI words (MP-7), `#17` | mostly; 9 files import `Storage`, `#2` | mostly; one import breaks it, `#7` | everywhere; one inward-to-outer import, `C2` | enforced by lint, `C52` | enforced by scopes, `G2` | — | stated as the one rule, `C2` | all code areas +docs | known |
| X03 | **Composition root with pure manual DI** | Service locator; DI container; singletons | — | everywhere (root not read), `#3` | — | split: `AppState` bag, `C31` | mostly; boot builds a 2nd graph, `C1` `C2` `C3` | a second test graph, `T24` | two wirings, MP-6 | holds, `C3` | all code areas +docs | known |
| X04 | **Single writer principle** | Many writers with per-row locks; optimistic merge | aggregate drawn by the write path, `#7` | mostly; direct `Storage` writes remain, `#8` | handle scoped to one Game, `#22` | — | — | call-site allowlist, `G7` | — | holds; Game record has two writers, `C12` | several +docs | known |
| X05 | **Concrete type over trait** (port only where substitution is the point; YAGNI seam) | A trait or repository interface per collaborator | no trait, no `dyn` in the area, `#3` | storage has no port, M1 | one concrete `Storage`; one-variant enum, `#5` `#12` | — | split with `dyn` ports, `C9` | failure layer inside `Storage`, `T3` | no storage fake, `C10` | stated, `C4` `C5` | all code areas +docs | known |
| X06 | **Sync core, async edge** (blocking I/O on worker threads) | Async all the way down | no `async` in the area, `#17` | everywhere, `#27` | everywhere, `#14` | one site offloads, `C43` | mostly, `C5` `C6` | — | — | stated, `C38` | all code areas +docs | known |
| X07 | **Errors as values; no panics in production** (lint-enforced) | Exceptions; `unwrap`; `panic!` | zero `unwrap`, `#17` | everywhere, `#16` | everywhere, `#46` `#49` | — | everywhere, named hatches, `C34` `C35` | panic on arrange in fixtures, `T31` | — | stated, `C36` | all code areas +docs | known |
| X08 | **Architecture fitness functions** (rules run as tests) | Rule in a document or in review | tag guardrail (shape `[TRIVIAL_ENUM]`) | — | — | lint bars storage imports, `C1` | whole crate, `C52` `C53` | whole crate, `G1`–`G6` | — | stated, `C52` | several +docs | known |
| X09 | **Hypermedia-driven application** (server HTML fragments, thin client) | JSON API with a single-page client | — | — | — | everywhere, `C5` | — | — | — | stated, `C39` | one +docs | inferred |
| X10 | **Asynchronous request-reply** (accept, answer at once, poll status) | Synchronous request that waits | — | — (the Job is the other half) | — | Job and Autoplay routes, `C8` | — | — | — | stated, `C37` | one +docs | inferred |
| X11 | **Polling over push** | WebSocket or SSE push | — | — | — | no WS/SSE code found, Q1 | — | — | — | stated, `C40` | one +docs | inferred |
| X12 | **Test tiers split by what is faked** (test pyramid, one binary per tier) | One tier; tiers by size or speed | — | — | — | — | — | everywhere, `T1` | everywhere, `C1` `C2` | stated, `C56` | several (test areas) +docs | known |
| X13 | **Classicist, state-based tests**: real SQLite `:memory:`, fake only at the port (sociable tests; "real fake over mock framework") | Mockist, interaction-based tests | — | — | `Storage::new_in_memory`, `#11` | — | mostly, `C57` | everywhere in storage, `T2` `T3` | everywhere, `C8` `C10` `C11` | stated, `C57` | several +docs | known |
| X14 | **Test hooks compiled into production under a default-on `testing` feature** ("test logic in production") | `#[cfg(test)]` only; a dev-only crate | test-only members on types (shape) | 6 sites, `#32` | failure layer, mock provider, `#8` `#9` | 2 sites, `C32` | whole crate, `C56` | 22 sites in 8 files, `T9` `T10` | everywhere, `C4` | — | all code areas | known |
| X15 | **Seed once, then the database outranks the files** (idempotent seeding; seed-if-absent) | Reload the files at every boot | — | — | four seed methods, `#28` | — | mostly, `C12` | — | — | stated, `C28` `C29` | several +docs | known |
| X16 | **Settings as one stored singleton row, read at the point of use** | Config cached at boot; a global static | — | closure update, `#13` | `CHECK (id = 1)` row, `#27` | — | everywhere, `C18` `C19` `C20` | — | — | stated; one doc says static, `C30` | several +docs | known |
| X17 | **Separate stores for separate lifetimes** (in-memory job entry; failure line on the record; single process) | One persisted status; externalised state | status and failure kept apart, MP-15 | job entry in memory; failure line on the Game, `#8` | — | — | — | — | — | stated, `C41` `C42` `C43` | several +docs | inferred ⁵ |
| X18 | **Trusted-network assumption** (no auth, no CSRF) | Defence in depth | — | — | — | absence of code, `C39` | — | — | — | — | one | inferred |
| X19 | **Ubiquitous language** (glossary words in code) | Code words that differ from the glossary | in comments; type names differ (`Npc`), `#26` | rebuilt code yes; older code `npc`, `#30` | newer files yes, `#15` | — | — | — | `Turn` in tests, MP-22 | reference docs, `C46` | several +docs | known |

### 1.2 Pattern

| Row | Concept (names used) | Opposite(s) | Domain 02 | Application 03 | Driven 04 | Driving 05 | Composition 06 | Test infra 07 | Test bodies 08 | Docs 09 | Spread | Label |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P01 | **Unit of Work** (the Save unit; `Tx`) | Autocommit per statement | — | writes via `GameWriter`, `#12` | closure-scoped, scoped to one Game, `#20` | — | — | — | — | stated, `C13` | several +docs | known |
| P02 | **Functional core, imperative shell** | Logic mixed with I/O; transaction script | mostly (`apply`, `for_request`), `#1` | Job to narrator to rules, `#7` | at function scale, `#38` | — | — | — | pure-function tests, `C28` | stated, `C20` | several +docs | known |
| P03 | **Service layer; thin controller** | Fat controller; handler reads storage | — | every non-Job use case, `#5` | — | split: thin in games, fat in settings and worlds, `C3` | — | — | — | — | several | known |
| P04 | **Plain data with rule functions; transaction script** ("anemic" in 02; "rich model" is the opposite) | Rich domain model | mixed, `#18` | most services, `#6` | — | fat controllers, `C3` | — | — | — | docs say rich, `C21` | several +docs | inferred ¹ |
| P05 | **Closed-world enum dispatch** (enum plus `match`) | Trait objects; open dispatch | everywhere in the area, `#3` | — | storage keys and tests, `#13` | both enum and string, `C28` | one place, `C7` | — | — | storage only, `C6` | several +docs | known |
| P06 | **Strategy through a trait object chosen by a factory `match`** (registry by config string) | `match` at each call; inheritance | — | one place, `#35` | LLM only, `#2` | — | one place, `C7` | — | — | stated; new type needs a match arm, `C44` | several +docs | known ² |
| P07 | **Function injection** (a closure type is the seam) | Trait object as the seam | — | recorder and test service, `#4` | — | — | split with `dyn` ports, `C9` | no-op save fn, MP-1 | — | named, MP3 | several +docs | known |
| P08 | **Late binding of a collaborator** (resolver closure per call) | Early binding at boot | — | `ProviderResolver`, `#4` | per-call rebuild, Q7 | — | mostly for connections, `C8` | — | — | read at point of use, `C30` | several +docs | known |
| P09 | **Execute-around / loan pattern** (closure runs inside the scope or lock) | A guard held by the caller | — | the lock, settings, config writes, `#13` | everywhere in storage, `#17` | — | mostly, `C47` | — | — | — | several | known |
| P10 | **Coarse lock around one resource, with a written lock order** | Fine-grained locks; lock-free; actor | — | `gate.rs` only (order), `#11` | one connection lock, `#18` | — | everywhere, `C46` `C48` | — | — | two locks named, `C14` | several +docs | known |
| P11 | **Fencing token and cooperative cancellation** | No check; kill the task | — | every Job save, `#10` | — | — | — | — | — | stated, `C15` `C16` | one +docs | inferred |
| P12 | **Capability** (unforgeable token; `pub(super)`; module privacy) | Ambient authority | — | `gate.rs`, `active.rs`, `#9` | `TxToken`, one place, `#21` | — | visibility as boundary, `C17` | the one door a test may use, `T12` | — | three sites, `C10` | several +docs | known |
| P13 | **Sum type for outcomes** ("make illegal states unrepresentable") | Booleans and `Option` flags; error codes | in generation and message kinds, `#4` | Job, run, status, `#15` | intent enum, `#29` | enum against string, `C28` | refusal values, `C33` | — | — | stated, `C8` | all code areas +docs | known |
| P14 | **Refusal kept apart from failure** | One error class, one status | `PlanRefusal`, `#4` | `Refusal`, `JobFailure`, `#15` | — | everywhere that shows, `C13` | in the Job path, `C33` | — | — | typed 400 and 500, `C33` | several +docs | known |
| P15 | **Error translation at a layer boundary** | Leak the inner error; one catch-all | one place (`from_engine_error`), `#21` | three envelopes, `#17` | — | by trait impl and `?`, `C14` | mostly, `C28` `C29` | — | — | — | several | known |
| P16 | **One crate-wide typed error enum** (`thiserror`) | An error type per layer or module; `anyhow` | uses `EngineError`, `#21` | wraps it in `ApplicationError`, `#17` | context decoration on it, `#47` | — | 20 variants, `C26` `C27` | — | — | stated, `C33` | several +docs | known |
| P17 | **Fail fast** (fail loud) | Graceful degradation; skip | opposite of `#15` | missing preset ends the Job, M5 | construction versus call, MP-16 | opposite sites, `C17` | boot edge, split, `C10` | helpers dump state, then panic, `T29` | fail loud, `C39` | stated, `C31` | all code areas +docs | known |
| P18 | **Graceful degradation** (fail-safe default; skip and continue) | Fail fast | — | Quantifier and runner paths, `#19` | — | several sites, `C17` | boot seeds, `C11` `C37` | — | — | seed files, `C32` | several +docs | known ³ |
| P19 | **Tolerant reader** (Postel; lenient parse; "robustness principle") | Strict parse; schema-enforced output | in several places, `#15` | both Agent parsers, `#20` | lenient enum parse, MP-6 | both, even in one file, `C18` | serde defaults, `C24` | — | — | skip bad file, `C32` | all code areas +docs | known ³ |
| P20 | **Parse at the boundary** ("parse, don't validate") | Pass strings inward | mostly for enums, `#5` | asked in M13 | mostly, `#35` | mostly, `C11` | — | — | — | — | several | known |
| P21 | **Data Mapper / boundary DTO split / anti-corruption layer** | Active record; one type for file, wire and domain | worlds, games, rows, `#20` | — | row structs and mappers; vendor types, `#3` `#4` | form to domain, domain to view, `C29` | — | — | — | manifest to card, `C27` | several +docs | known |
| P22 | **Command-query separation; CQRS-lite read facade** | One model or service for read and write | story reads, `#11` | `GameViewQuery` against `GameWriter`, `#29` | read handles and a write unit, `#23` | route level, `C7` `C30` | — | — | — | stated, `C18` | several +docs | known |
| P23 | **Reference by identity; pass ids, reload (claim check)** | Object graph; pass the loaded record between steps | everywhere but `SceneState`, `#6` `#7` | Job steps, `#24` | scoped handle; FK cascade, `#22` `#26` | — | — | — | — | stated, `C19` `C22` | several +docs | known |
| P24 | **Command object; order as data** (plan, then execute) | Handler branching on raw text; execute while deciding | `generation.rs` only, `#2` | — | — | text to `Action` to `JobRequest`, `C9` | — | — | — | — | several | known |
| P25 | **Snapshot per version** (Memento) | Event sourcing; one mutable state with undo | everywhere, `#9` | — | JSON column holds it, `#24` | — | — | — | — | stated, `C24` | several +docs | inferred ⁵ |
| P26 | **Derive, do not store** (single source for a fact) | Cached or denormalised flags | mostly, `#10` | — | — | — | — | — | — | stated, `C23` | one +docs | inferred |
| P27 | **Data-driven content and rules** (JSON World, data-driven Triggers) | Content in code; a branch per rule | Triggers, `#22` | — | — | — | mostly, `C16` | — | — | stated, `C28` | several +docs | known |
| P28 | **Table-driven code, rules and tests** (reason on each row) | Unrolled, repeated code | — | — | — | both table and unrolled, `C27` | — | 7 rows run by 4 walkers, `G3` `G4` | table and per-case tests, `C33` | stated, `C54` | several +docs | known |
| P29 | **Single source of truth / "one home"** | Copy at each caller | mostly, `#12` | — | worlds and entries only, `#30` | where a comment names it, `C26` | — | — | — | stated, `C68` | several +docs | known |
| P30 | **Sensible defaults as data** (serde default-fn pointers; `Default`) | Required fields; `deny_unknown_fields` | everywhere in settings, world, scenario, `#25` | — | one default in three places (shape) | — | everywhere, `C24` | — | — | — | several | known |
| P31 | **Sanitise and escape at the boundary; separate instructions from data** | Trust the model and the player; pass text through | — | narration, Agents, recorder, `#21` | — | escape by default; sanitise Markdown, `C22` `C23` | — | — | — | stated, `C48` `C49` | several +docs | known |
| P32 | **Builder** (fluent, consuming `self`; "test data builder") | Telescoping constructor; struct literal | — | assembler only, `#25` | the mock, `#41` | — | sampled, `C59` | two builders, `T6` | app and story, `C17` | stated, `C60` | all code areas +docs | known |
| P33 | **Named constructor / factory** (`from_config`, `new`) | Struct literal at each call | mostly, `#14` | `from_config(_config)` ignores config (shape) | three providers, `#41` | — | — | — | — | — | several | known |
| P34 | **Object mother; creation method** | Builder; literal in each test | — | — | — | — | — | eight `Test*` structs, `T7` `T8` | mostly, `C18` | — | several (test areas) | known |
| P35 | **Parameter object** | Long argument list | — | prompt and Agent code, `#23` | — | presets only, `C10` | — | — | — | — | several | known |
| P36 | **Pipes and filters** (layered prompt sections) | One template; one big `format!` | — | prompt assembly, `#26` | — | — | — | — | — | narrator prompt only, `C50` | one +docs | inferred |
| P37 | **Decorator or wrapper** (fault layer; recorder) | Each caller records or injects its own | — | recorder, "in spirit", `#22` | `BackendKind`, `#9` | — | — | — | — | `BackendKind`, `C7` | several +docs | guessed ⁴ |
| P38 | **Test double shipped in the production tree** (selectable by config) | Doubles only in `cfg(test)` or `tests/` | — | — | `MockBackend`, `#8` | — | wiring picks `Mock`, `C7` | MP-1 | MP-1 | MP2 | several +docs | known |
| P39 | **Strict fake with verify-on-drop; test spy** | Lenient fake that answers any call | — | — | lenient mock, MP-18 | — | names `ScriptedModel`, `C57` | one type, `T4` | strict in 3 files, lenient in 6, `C13` `C14` | `ScriptedModel` and `MockBackend`, `C58` | several +docs | known |
| P40 | **Fault injection at the seam** (typed keys per call or write) | A real failure; no error-path test | — | — | `StorageCall`, `TxWrite`, `#9` `#10` | — | — | storage; LLM uses `failed`, `T11` | 11 and 42 uses, `C20` | stated, `C59` | several +docs | known |
| P41 | **Latch or held call for deterministic races** | Fixed sleeps | — | — | — | — | — | `ScriptedModel` hold, `T5` `T28` | tier 1 races, `C36` | — | several (test areas) | known |
| P42 | **Condition-based waiting** (bounded polling) | Fixed sleep; event wait | — | — | — | — | — | `T27` | 208 uses, `C38` | stated, `C61` | several (test areas) +docs | known |
| P43 | **Seam contract test** (one `CASE` per clause) and **specification by example** (`SCENARIO`) | Coverage-driven test per function | — | — | — | — | — | 3 files plus storage, `T13` `T14` | 272 of 272 tagged, `C31` `C32` | — | several (test areas) | known |
| P44 | **Ratchet** (shrink-only list; quarantine) | Permanent exemption; rewrite at once | — | — | — | — | — | `T17` `T18` | 72 quarantined, 78 of 87 listed, `C7` | — | several (test areas) | known |
| P45 | **Fresh fixture per test** | Shared fixture | — | — | — | — | — | tiers 2 and 3, `T20` (shared: `T21`) | everywhere, `C16` | stated, MP18 | several (test areas) +docs | known |
| P46 | **Process-wide `Mutex` to serialise tests on global state** (RAII guard) | Remove the global; process isolation | — | — | — | — | one place, `C58` | two guards, `T22` | 66 uses, MP-7 | stated, MP18 | several +docs | known |
| P47 | **Service virtualisation** (stub server for the engine) | The real server | — | — | — | — | — | tier 2, `T25` `T26` | tier 2, `C43` | — | several (test areas) | known |
| P48 | **In-process HTTP through `oneshot`** | A real socket; a spawned server | — | — | — | — | — | tier 1 against tier 3, `T23` | tier 1, `C9` | stated, `C62` | several (test areas) +docs | known |
| P49 | **Package by feature against package by role** | Package by layer or kind | — | layout of `utils/`, Q11 | — | mixed, `C34` | — | rule: free fn lives by role, `G13` | — | — | several | known |
| P50 | **Retry with a bounded count** (and its opposite: timeout without retry) | Fail on the first error; retry with back-off; circuit breaker | — | both Agents (two attempts), `#18` | timeout, no retry, `#44` | — | — | no retries in tests, `T19` | — | — | several | known |
| P51 | **Lock held as the atomicity mechanism** (read-modify-write under one hold) | Optimistic concurrency (version column); atomic SQL update | — | check and units in one registry hold, `#8` `#11` | settings, preset names, story window, `#19` | — | settings closure, `C19` | — | — | stated, `C14` (MP5) | several +docs | known |
| P52 | **Typed failure taxonomy that carries the evidence** (closed enum of kinds, fixed short line, raw text kept) | Stringly errors; a bare status code | `GenerationFailureKind`, `#21` | `JobFailure` from `GenerationFailure`, `#17` | `LlmFailure` with the raw body, `#40` | — | one place, `C31` | — | — | — | several | known |
| P53 | **Progressive disclosure of failure** (reason line, raw text behind a disclosure; hide internals from the player) | Raw text only; short line only; echo `to_string()` | `GenerationFailure`, `Notice` (shape) | failure line (S2) | — | status, banner, Settings; `"Internal Server Error"`, `C15` `C16` | `C32` | — | — | stated, S2 S3 | several +docs | known |
| P54 | **Admission control: one Job per Game, refuse the second** (busy answer) | Queue the request | — | `Busy`, `Refusal::StillThinking`, `#8` `#15` | — | busy is a normal reply, MP-19 | — | — | — | stated, `C17` | several +docs | known |
| P55 | **Record every model call** (forensics record; audit trail of raw I/O; bounded log) | Keep the parsed result only | — | one recorder for all calls, `#22` | `llm_messages`, 50-row prune, `#32` `#33` | — | success and failure saved, MP-24 | — | — | stated, `C47` | several +docs | known |
| P56 | **Stub that fails late** (registered behind the port, errors at the call) | Fail fast at construction | — | — | DeepSeek, `#42` | — | named "defers error" in a test, MP-4 | — | tests pin "not yet implemented", MP-17 | — | several | known |
| P57 | **Arrange by exercising the engine** (post an Action, wait, read) | Seed records straight into storage; raw SQL | — | — | — | — | — | through the real HTTP form, `T35` | tier 1 mostly; seeding in 11 of 24 files, `C19` `C21` | — | several (test areas) | known |
| P58 | **Black-box assertion through a read seam** (behavioural, outcome-based) | White-box assertion that reads `Storage` | — | — | — | — | — | by rule, `T15` `T16` | tier 1 yes; 14 unit files read `Storage`, `C22` `C23` | stated, `C62` | several (test areas) +docs | known |

¹ P04: 02 calls its own shape "plain data with public fields, rules as functions" and names Fowler's "anemic" model as the opposite; 03 names "transaction script"; 09 says the domain layer is "rich". I merged the three because they sit on the same axis (where a rule lives). 02 itself says the area is mixed (`Message` and `Tail` carry behaviour).
² P06 and P05 overlap: both are "choose an implementation by a `match`". P05 is the closed enum with `match` for data and keys; P06 returns a trait object. 09 `C44` and 03 `#35` are one-place registries.
³ P18 and P19 overlap and I kept both because the areas use two different names for two different acts: skipping a bad item (P18) and accepting a loose input (P19).
⁴ P37 is **guessed**. 03 says "Decorator in spirit" for the recorder and notes it is not a `LlmProvider`; 04 says "Decorator-like" for the fault layer chosen by an enum; 09 says "Decorator" for `BackendKind`. The three are not one structure. They share only "a wrapper adds behaviour".
⁵ X17 and P25 have two code areas in their cells, and I label them *inferred*. In X17, 02 and 03 name two stores for two lifetimes without naming the stance; the stance name comes from 09. In P25, the 04 cell is the JSON column that holds a Swipe state; 04 does not name a snapshot.

### 1.3 Idiom

| Row | Concept (names used) | Opposite(s) | Domain 02 | Application 03 | Driven 04 | Driving 05 | Composition 06 | Test infra 07 | Test bodies 08 | Docs 09 | Spread | Label |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| I01 | **Sibling `*_tests.rs` file** registered in `mod.rs` | Inline `#[cfg(test)] mod tests` | — | every module with tests, `#33` | everywhere, `#51` | 22 files, `C33` | everywhere in scope, `C55` | rule and mirror check, `T33` `G20` | everywhere but one file, `C6` | — | all code areas | known |
| I02 | **Poison-tolerant lock** (`into_inner`) | `lock().unwrap()` | — | all 3 `Mutex` types, `#28` | every lock site, `#48` | — | 17 sites, three spellings, `C45` | test fakes use `expect` (MP-12) | — | stated, `C35` | several +docs | known |
| I03 | **RAII drop guard** | Explicit cleanup call | — | Job entry and run hold, `#14` | — | — | flush guard in `main`, `C41` | settings and env guards, `T22` | — | — | several | known |
| I04 | **Newtype for an identifier** | Raw `u64` or `String` | two places, `#19` | `GameId`, `RunId`; Job id raw, M11 | — | `CurrentGame`; path id raw, `C12` | — | — | — | one id only, `C9` | several +docs | known |
| I05 | **Optional return plus a `require_` wrapper** (`Result<Option<T>>`) | Not-found as `Err`; sentinel value | `Option` and `Result` accessor pair (shape) | reads, `#31` | games, personas, worlds, `#6` | — | — | — | — | stated, `C34` | several +docs | known |
| I06 | **Context-adding extension trait** (`.db_context`) | Blanket `#[from]`; `anyhow` context | — | — | storage everywhere, `#47` | — | everywhere in storage, `C30` | — | — | stated, `C33` | several +docs | known |
| I07 | **Effect as a parameter** (`_with(lookup)` twin) | Ambient read of the environment | settings and name rules, `#13` | — | — | — | one place, `C23` | — | — | — | several | known |
| I08 | **Ambient clock and ids** (`Utc::now`, `Uuid::new_v4` in place) | Injected clock or id source | `Room::new_dynamic`, `Message::new`, MP-6 | — | 10 sites, `#34` | id from the clock in the handler, MP-15 | everywhere, `C49` | — | clock in fixtures, MP-10 | — | all code areas | known |
| I09 | **Module as facade** (`mod.rs` declares; `pub(crate)` as the boundary) | `pub mod` that exposes files | — | — | — | mostly, `C35` | mostly, `C17` | `mod.rs` is a pure facade, `G14` | — | — | several | known |
| I10 | **Traceability tags read by guardrails** (`[DOC: path]`, `[TRIVIAL_ENUM]`) | Free-form docs | tags (shape) | all 54 files, S11 | — | — | 210 of 307 files, `C60` | rule, `G16` | — | no anchor in docs, Q1 | several +docs | known |
| I11 | **Escape hatch that names its reason** (`arch-lint: allow(...) reason=`) | Silent `#[allow]` | — | 3 `#[allow]`, `#16` | `#49` | — | 24 comment sites, `C54` | per-call and file-level `allow`, MP-15 | — | — | several | known |
| I12 | **Intent enum instead of a boolean flag** | Flag argument | — | two booleans stay in `Job::run`, M10 | `WorldWriteMode`, `SettingsWrite`, `#29` | — | — | — | — | — | several | known |
| I13 | **Structured fields and spans for log context** (`#[instrument]`) | A string tag in each message | — | in `job.rs`, M14 | — | — | one file, `C42` | — | — | — | several | known |
| I14 | **Correlation tag in the message text** (`[LLM][req:N]`) | A span field | — | bracket tags in Agents, M14 | everywhere in LLM transport, `#43` | — | `C43` | — | — | — | several | known |
| I15 | **Log at the point of failure and also return the error** | Log only at the handling edge | — | — | `error!` 12 times, MP-15 (06) | logging in the handler, `C37` | split, `C44` | — | — | — | several | known |
| I16 | **Lazy static regex** and **process-global counter** | Compile at each use; counter on a collaborator | — | two styles, S12 | — | — | `C50` `C51` | — | — | — | several | known |
| I17 | **Shared handle by `Arc` and cheap `Clone`** | Borrow; global; single owner | — | all services, `#34` | — | — | — | — | — | — | one | inferred |
| I18 | **Sentinel value for "absent"** (empty string, `0`, `""` means unset) | `Option`; a new type | `Message::new` id 0; `Action("")`, MP-14 | — | empty string and `NULL` both absent, `#36` | absent checkbox, MP-16 | — | — | — | — | several | known |
| I19 | **Locality: an impl or helper lives with its type** | Impls spread across files | — | — | — | — | — | rule, `G12` | — | stated, `C11` | one +docs | inferred |
| I20 | **Shared test support compiled into each binary by `#[path]`** | A support crate | — | — | — | — | — | `T34` | `C3` | — | several (test areas) | known |

### 1.4 Single-area concepts (not in the table above)

These 87 concepts are named by one area only. Their spread is *one*. For areas 02 to 08 the label is *inferred* (one code area). For area 09 the concept is *docs only*: no code area names it and no one checked it against the code. The names are shortened; the full row is in the asset.

- **Domain (02)**, 4: #8 Template and instance split; #16 Design by Contract as assertions; #23 Guard clauses; #24 Temporal coupling, documented and pinned by a test.
- **Application (03)**, 0: every one of its 35 concepts is in a row above.
- **Driven adapters (04)**, 7: #16 Hand-written SQL on an embedded database; #25 Forward-only versioned migrations with `PRAGMA user_version`, guarded to be re-runnable; #31 Keyset (seek) pagination; #37 Shared HTTP core with thin per-provider adapters; #39 Provider quirks absorbed by the adapter; #45 Memoize keyed by the input; #50 Defensive bounds check at a foreign boundary.
- **Driving adapters (05)**, 18: C4 Front controller; C6 RPC-style endpoints; C19 Server-side rendering with compile-time templates; C20 Co-located (inline) templates; C21 Presentation model / view model; C24 Composite view; C25 Logic in the view model; C36 Utility module; C38 Runtime resource binding; C40 Immediate apply; C41 Edge-minted identity; C42 Graceful shutdown; the server returns its handle; C44 HTTP status as a UI control channel; C45 Out-of-band swaps and header-driven swaps; C46 Full refresh after a write; C47 Declarative behaviour in markup; C48 Accessible markup; C49 Null object for "no Game".
- **Composition and cross-cutting (06)**, 11: C4 Explicit phases in startup; C13 Convention over configuration for paths (data dir and DB sit next to the executable); C14 Declarative CLI; C15 Convention-based discovery; C21 Layered configuration sources (built-in default, JSON seed, database row, environment); C22 Secrets from the environment; C25 Named constants for tuning values; C36 Crate-root `Result` alias; C38 Logging behind a facade; C39 Level by severity, with `warn` for a degraded path and `error` at a failed edge; C40 Environment-driven filter.
- **Test infrastructure (07)**, 11: T30 Keep failure artifacts for a post-mortem; T32 Page Object; G8 No hidden global state in storage: ban `static` and `Atomic*`; G9 Boundary by banned name or type: the guard never spans a model call; a Job holds ids, not records; G10 Call-count budget: one `hold` per writer; G11 HTTP adapter boundary: no `Storage`, no whole `SwipeState`, handler return type, no collection in a form struct; G15 Complexity budget: nesting depth 3, file length 2,000 non-blank lines; G17 Comment minimalism: few comment runs, no separators, no one-letter names; G18 Idiom enforcement: no `std::thread` in production; G19 Tests only through the settled helpers in the full-stack browser tier; G21 Convention over configuration for imports: std, then external, then crate.
- **Test bodies (08)**, 19: C5 White-box unit tests in a child module, with private access; C12 Hand-written fake in place of a mocking framework; C15 Hand-rolled one-purpose stub, one struct per behaviour; C24 Assertion on markup by substring; C25 Assertion message that carries the context; C26 Typed error assertion with `matches!` or `match`; C27 One clause per test; C29 Round-trip testing of serialisation; C30 Schema migration test on an old database shape; C34 Property-based testing; C35 Smoke test; C37 Real threads for a race; C40 Opt-in slow test with `#[ignore]`; C41 Naming: `test_` prefix plus a snake-case sentence; C42 Suffix `_http` on a tier-1 test name; C44 A black-hole server at the network edge makes a Job hang; C45 Test suite object: one test function runs many checks on one shared browser; C46 Free helper functions, selectors and inline JS strings in the test; C47 A guard check with an inline exemption marker.
- **Docs (09)**, 17: C25 Append-only, non-destructive history: a Retry adds a Swipe and keeps the old one; C26 Linear history: only the last Message can take a new Swipe; C45 Optional component; C51 Template substitution of author-controlled text; C53 Defense in depth; C55 Rules tested by paired positive and negative cases; C63 Diátaxis: organise by reader need; C64 Reference describes only; the reason goes to a paired Explanation doc; C65 Explanation unfolds; it does not justify; C66 Docs as code, machine-validated; C67 Generated documentation; C69 Cite symbols, not line numbers; C70 Layers of record: code and unit tests, then integration tests and specs, then docs; C71 Edit direction: a behaviour change edits spec and test; a doc changes only when a part, invariant or name changes; C72 Diagram as code; C73 Link to a living example instead of copying a snippet; C74 Seam identifier versus mechanics leak.

## 2. Meeting points across areas

Each point is a question. Each one has evidence from two or more areas. `✔` marks a citation that I checked against the code in this session. A point does not rule on anything. "Areas" lists the assets and the meeting-point numbers behind it.

### 2.1 Boundaries and wiring

**XM-1. Is the storage seam a port, a concrete type, or a layer inside the type?**
- Port side: the LLM provider and the text check have a `trait` in `application/ports/` (`src/application/ports/llm_provider.rs:44`, `text_checker.rs:64`). The docs say the application layer reaches "LLM, storage, and text-check through port traits" (`docs/diataxis/explanation/architecture.md:14` ✔).
- Concrete side: `Storage` has no port. Nine non-test files in `src/application/` take `Arc<Storage>` (`gate.rs:8`, `pipeline/job.rs:10`, `games/catalogue.rs:14`, `settings_service.rs:9`). The same doc says `Storage` is "a concrete adapter" and "the trait is rejected as phantom" (`explanation/architecture.md:14,65` ✔). `reference/architecture_system.md:30` says "Application code may call `Storage` directly when the call is part of the persistence boundary. Architectural review owns the discipline for which files sit on the boundary" ✔.
- Layer-inside side: storage tests replace the adapter with `BackendKind::Test` inside the type (`src/adapters/driven/storage/core.rs:74-86`) and a real SQLite `:memory:` (`core.rs:120-127`). The LLM replacement is a fake behind the port (`llm/providers/mock.rs:104`).
- Also: the unit-test tier row calls the real SQLite a fake (`tests/STRATEGY.md:15` ✔), while the same file says "do not mock an internal part" (`STRATEGY.md:53-55`).
- Areas: 03 M1; 04 MP-1, MP-2; 07 Q-7; 08 C10; 09 MP1, MP2, MP3, Q11.

**XM-2. Which mechanism makes a write atomic, and where is the list?**
- Lock-held closure: settings read-modify-write (`storage/settings.rs:102-116`), preset name scan plus write (`storage/presets.rs:65-68`).
- Explicit SQL transaction: `write_world` (`storage/worlds.rs:175-182` ✔) and the migrations (`storage/utils/plumbing.rs:57-61`).
- The `Tx` unit with a capability token (`storage/transaction.rs:280-328`), and one registry hold around a check and its units (`application/generation/gate.rs:145-158`).
- No mechanism: `update_world` writes the world row and then the map row with no transaction (`storage/worlds.rs:314-334` ✔, next to `write_world` that has one); `save_llm_message` inserts and then prunes (`storage/llm_messages.rs:19-46`).
- Docs: a reset is "both writes in one hold of the registry" while a create is "one save unit" (`reference/game_flow.md:100,124`); the seed flow "sits outside this rule" (`explanation/storage_design.md:58`).
- Two locks nest: the registry hold, then the backend mutex of the unit, then the connection mutex (`gate.rs:145-147`; `storage/core.rs:213-216`; `storage/db.rs:27-32`). A `Storage` call inside a unit closure waits on the mutex the unit holds, and only a comment keeps it out (`storage/transaction.rs:47-48`).
- Areas: 03 `#11` `#12`; 04 MP-3, MP-4, MP-11 and hazards; 06 MP-13; 09 MP5, MP6.

**XM-3. How many writers does one Game record have, and what keeps them apart?**
- Single-writer side: `GameWriter` is "the only code that opens a unit of record writes" (`gate.rs:240` ✔, cited as `:246` in asset 03); a guardrail row allows few call sites (`tests/infrastructure/guardrails/walker_rows.rs:45-64`); the docs list an ownership table (`reference/architecture_system.md:44-59`).
- Other writers: `GameCatalogue` calls `update_game_display_name` and `update_game_config` (`games/catalogue.rs:83,134,166,201` ✔). Only the config writes go through `config_write`, which refuses while a run holds the Game (`gate.rs:410-417`). `rename_game` does not (`catalogue.rs:81-85` ✔). `refuse` writes the failure line on the same record (`gate.rs:590-597`). More write paths for `games` exist in storage (`storage/games.rs:34,119`; `storage/core.rs:241`; `storage/transaction.rs:66-95,205-220`).
- The dock is a table of its own "because the config routes write the Game record with no busy check" (`storage/utils/plumbing.rs:652-656`; `explanation/storage_design.md:50`).
- Adapter side: the World details save and the posture autosave each read and write the whole World (`http/worlds/handlers/worlds.rs:157-195,259-277`).
- Check then act outside the hold: `delete_game` and `switch_game` read the active pointer first (`catalogue.rs:68-94`).
- Areas: 03 M2, M18; 04 MP-12; 05 MP-17, hazard 3; 07 MP-9; 09 MP7.

**XM-4. How many error envelopes does one failed request cross, and who picks the conversion?**
- Types: `EngineError` (`src/error.rs:112`), `ApplicationError` (`application/errors.rs:6`), `Refusal` and `JobFailure` (`gate.rs:26,40`), `RunRefusal` (`gate.rs:210`), `PlanRefusal` (`domain/model/generation.rs:215`), `GenerationFailure` (`domain/model/state/generation_status.rs:126`), and plain `String` in HTTP helpers (`http/settings/handlers/settings.rs:100`).
- Two ways into `ApplicationError`: `From` keeps every engine error on `Engine`; `from_engine_error` lifts `Validation` (`application/errors.rs:22-31` ✔). `world_catalogue.rs:45-48` writes its own `match`. `SettingsService` and `PromptPresetService` return both `EngineError` and `ApplicationError` (`settings_service.rs:25,41`).
- At the edge: `Validation` is 400 in `http/error.rs:16` ✔ and 200 in `render_posture_result` (`http/games/handlers/games.rs:225-226` ✔, "house style"), in one file also both (`http/worlds/handlers/worlds.rs:141,255`).
- Text to the player: a fixed `"Internal Server Error"` (`http/error.rs:20-23`), the raw error (`http/worlds/handlers/worlds.rs:99,142`), or nothing logged (`http/games/handlers/games.rs:33-35`).
- `EngineError::Config` serves as "missing setting", a failed view read (`http/app_state.rs:95`), a bind failure and a missing Quantifier text (`06 MP-10`).
- Inside storage: a failed JSON step is `Parse` with a string in some files and `Serde` through `?` in others (`storage/characters.rs:51-56` against `storage/stories.rs:243`).
- Areas: 02 MP-15 (reason line and raw text twice); 03 M3, M4; 04 MP-17; 05 MP-2, MP-3, MP-22; 06 MP-9, MP-10, MP-11.

**XM-5. Is the wiring one graph with swapped ports, or a second assembly?**
- Production: `build_app_graph` builds three recorders from stored settings (`bootstrap/wiring.rs:150-160`).
- Test: `build_app_graph_for_tests` builds one recorder over a fixed `MockBackend` and then builds the Options recorder from stored settings (`wiring.rs:189-198`). `TestAppBuilder` wires through it (`src/test_support/test_app_builder.rs:207-208`); `app_with_production_graph` wires the real one (`tests/http/support/app_wiring.rs:79`). Unit tests in two handler files build their own graph by hand (`http/settings/handlers/settings_tests.rs:29`).
- Boot: `run.rs:102-107` builds a second `GameWriter`, `ActiveGame` and `GameCatalogue` before the wired graph exists (`wiring.rs:105-119`).
- HTTP: `WiredApp` is copied field by field into `AppState` (`http/app_state.rs:44-57`); handlers take the whole bag (`http/history/handlers/history.rs:20`); a guardrail limits where `WiredApp` may be named and not `AppState` (`layers.rs:5-6,41`).
- Areas: 03 `#3`; 05 C31, MP-23; 06 MP-6, MP-7, MP-8; 07 Q-8; 08 MP-6.

**XM-6. Where does a rule live: domain, application, adapter or storage?**
- The domain is "pure" (`domain/mod.rs:2`; `arch-lint.toml:88-94`) and yet logs with `tracing` (`domain/model/settings.rs:56,187,225`), reads `std::env::var` (`settings.rs:318`), carries dashboard words (`settings.rs:45-52`; `state/generation_status.rs:21-25,239-246`) and provider URLs (`settings.rs:365-376`).
- Storage holds rules: name uniqueness (`storage/presets.rs:91-109`), "refuse delete when Games exist" (`storage/worlds.rs:384-388`), retention of 50 rows (`storage/llm_messages.rs:11`), domain read views in `WorldBundle` (`storage/worlds.rs:35-55`).
- Handlers hold rules: three delete-connection rules in an `update_settings` closure (`http/settings/handlers/settings.rs:317-335`); a World merge (`http/worlds/handlers/worlds.rs:172-190`); "not allowed for mode" (`http/prompt_presets/handlers/prompt_presets.rs:287-289`). Beside them `rename_game` is one call (`http/games/handlers/games.rs:136`).
- A prompt helper in `application/` is imported by two adapters (`providers/ollama.rs:9`, `openrouter.rs:9`), and the Ollama quirk sits in the adapter (`ollama.rs:48-59`).
- Areas: 02 MP-7, MP-9; 03 M17; 04 MP-14, MP-15; 05 MP-1; 06 MP-14.

**XM-7. Which reads go through which read surface, and who fetches the data for a fragment?**
- `get_*` and `require_*` on `Storage`; read handles `stories()` and `docks()`; `GameViewQuery`; a roster read for Characters (`reference/storage.md:39-42,67-72`; `explanation/storage_design.md:70`).
- `GameViewQuery` says it holds only reads (`games/view_query.rs:2`), `GameCatalogue` also reads (`catalogue.rs:98-120`), both read the dock (`view_query.rs:200`; `gate.rs:555`) and the tail (`view_query.rs:77-87`; `gate.rs:559-564`).
- Role health, which belongs to Settings, is read through `GameViewQuery` (`http/settings/handlers/settings.rs:31`). The debug routes reach `job_deps.narrator` (`http/debug/handlers/debug.rs:26,42`).
- A fragment's data is fetched in `AppState::render_*`, in builders, in handler helpers or in `Template::from_*` (`http/app_state.rs:99-110`; `http/games/handlers/games.rs:240-252`; `http/settings/templates/settings.rs:100`).
- One lock for a snapshot or one per read: `window` uses one connection (`storage/stories.rs:53-61`), `world_bundle_for` takes four locks (`storage/worlds.rs:127-135`); `GameViewQuery::status` reads two stores in two steps (`view_query.rs:204-218`).
- Areas: 03 M8, M19; 04 MP-11; 05 MP-23, MP-24, MP-25; 09 MP9.

### 2.2 Settings, configuration, seeding

**XM-8. Which settings are read at each use, which once at boot, which from a global?**
- At each use: the three connections, through a resolver closure that runs `get_settings()` on every call (`bootstrap/wiring.rs:44-54`); the docs say the same (`explanation/rust_idioms.md:20-24` ✔).
- Once at boot: the Agent list (`wiring.rs:144,161-166` ✔); no production code outside the boot reads `settings.agents`.
- A global: `reference/coding_standards/integration_test_standards.md:152-153` ✔ says "`AppSettings` lives in a global static", and `tests/test_utils/settings_guard.rs:1-5` ✔ serialises "global settings state" with a static `Mutex`. A search of `src/` for a static or `Lazy` `AppSettings` finds none ✔. The guard is taken 66 times in tier 1 although each test builds its own storage (08 MP-7).
- The environment: the key rule is a pure function (`settings.rs:323-331`) with a wrapper that reads the environment (`settings.rs:317-319`); the log mode and the exe path are read in place (`bootstrap/logging.rs:14`; `utils/cli.rs:43`).
- A whole-row `save_settings` stays public beside the closure form; its only caller is the test builder (`storage/settings.rs:96-100`; `test_support/test_app_builder.rs:193`).
- Tuning values have two homes: `MAX_CONTEXT_TOKENS` against a per-connection `match` (`prompting/token_budget.rs:4,23`; `settings.rs:378-384`), `2048` twice (`token_budget.rs:8`; `llm/transport/utils/request.rs:14`), and the default Agents in two places (`agents/runner.rs:28-34`).
- Areas: 03 Q10; 04 Q7; 06 MP-1, MP-17, MP-18, MP-23; 07 MP-10, MP-11; 08 MP-7; 09 MP4.

**XM-9. What happens to a bad input at boot: stop, skip, fall back or log?**
- Stop: a connection id that names nothing (`wiring.rs:145-148`); a bad map, persona or preset file returns `Err` (`bootstrap/load.rs:123,179`; `bootstrap/run.rs:179` ✔ applies `?` per preset file, and `run.rs:54-60` only logs the error).
- Skip: a bad `world.json` or NPC file (`load.rs:103-112,155-157`).
- Default without a log: `AgentRunner::from_configs(...).unwrap_or_default()` drops an `Err` for an unknown Agent type (`wiring.rs:161-166` ✔); the Job then runs with no Quantifier result (`agents/runner.rs:69-74`).
- Fall back: a missing World falls back to the first World, a missing Persona stops (`run.rs:66-86`); a missing API key is checked at the call (`wiring.rs:55`); DeepSeek builds and fails on every call (`providers/deepseek.rs:25-30` ✔; a test names this "defers error", `wiring_tests.rs:144-146`).
- Docs: a malformed world, Persona or preset file "is skipped without halting bootstrap" (`reference/startup.md:35`; 09 MP10).
- In a Job: a missing preset ends the Job (`prompt_preset_service.rs:62-72`), the Options prompt keeps a fallback text (`agents/options/prompt.rs:8-10`), a failed Quantifier gives static Characters (`quantifier/utils/orchestration.rs:142-155`), a failed after-narration Agent gives the empty result and the end-of-Job hook returns the error (`agents/runner.rs:69-98`).
- At the edge: a failed `list_worlds` is a 500 and a failed `list_personas` an empty list (`http/games/handlers/games.rs:52-62`); a failed list renders "No worlds defined" (`http/worlds/handlers/worlds.rs:119-125`).
- Areas: 02 MP-3; 03 M5; 04 MP-16; 05 MP-5; 06 MP-2, MP-3, MP-4, MP-5, MP-20; 09 MP10.

**XM-10. Is a seed file a template read once, an authority, or a schema?**
- Once: "after the first run the database outranks `data/settings.json`" (`storage/settings.rs:118-123`); Character and Persona use `INSERT OR IGNORE` (`storage/characters.rs:59`; `storage/personas.rs:62`); a World re-seed replaces and keeps the row id (`storage/worlds.rs:144-150`). A probable result (inferred): an edited Character file does not reach a database that holds that Character (04 MP-13).
- Preset seed is read as raw JSON with hand-set defaults, while `PromptPreset` derives `Deserialize` (`bootstrap/run.rs:200,219-231`; `domain/model/prompt_preset.rs:63-64`).
- The JSON Schema files are "authoritative" (`reference/startup.md:68`) and name the Rust types they define (`startup.md:61-66`); no schema exists for presets (06 MP-3).
- The domain serde shape is the stored format: v28 and v30 deleted every Game when the shape changed (`storage/utils/plumbing.rs:662-665,716-726`); `serde` defaults let old rows read (`domain/model/utils/settings_defaults.rs:1-2`). Migrations are additive and re-runnable in most steps and destructive in two (`plumbing.rs:251,282,667-671,719-722`).
- Two readers of the World folder: the CLI swallows parse errors, the seeder warns (`utils/cli.rs:84-85`; `load.rs:82-112`).
- Areas: 02 MP-7; 04 MP-8, MP-9, MP-13; 06 MP-2, MP-3, MP-19; 09 MP21, S13.

**XM-11. Is the number of preset types a fixed fact or a list that can grow?**
- Fixed and unrolled: four fields for four preset slots on `Game`, `PresetSelection` and `NewGame` (`domain/model/game.rs:37-41,68-73,113-116`), with `Options` the odd slot (`prompt_preset.rs:31-36`); four `list_presets` calls and a 35-line block written four times (`http/prompt_presets/handlers/prompt_presets.rs:32-47`; `http/prompt_presets/templates/prompt_presets.rs:15-165`); four option fields on `GamePostureTemplate` (`http/games/templates/games.rs:207-210`).
- Table-driven: `PRESET_PICKER_TYPES` (`http/games/handlers/games.rs:256-270`) and a role table in the header (`http/builders/headers.rs:157-164`).
- Hand-parsed in the boot (`bootstrap/run.rs:200-231`).
- Areas: 02 MP-5; 05 MP-10; 06 MP-3.

**XM-12. Does the text form of a domain enum have one owner?**
- Domain: `as_str`, `FromStr` and `parse_or_default` for `NarratorMode`, perspective and tense (`domain/model/settings.rs:37-59,72-82,177-203`), used by the adapter (`http/builders/presets.rs:90,180`).
- Adapter copies: `TextCheckMode` mapped by a local `match` twice and again in the template (`http/settings/handlers/settings.rs:132-137`; `http/settings/templates/settings.rs:316-321,287-290`); literal `("second", "Second person")` pairs (`http/worlds/templates/worlds.rs:228-235`); a provider list that omits `Mock` (`http/view_models.rs:48-53`).
- Storage: five mapping styles from row to domain and a lenient parse for the same three enums (`storage/models/game.rs:63-72`; `storage/models/world.rs:56-65`), a strict one for `PresetType` (`storage/models/prompt_preset.rs:40-41`).
- Domain strings: `TemplateVars` holds perspective and tense as `String` (`domain/model/template.rs:21-23,35-36`); `AgentConfig.agent_type` is a `String` next to a `Role` enum (`agent.rs:117,14-23`); `MovementJson.movement_type` is `Option<String>` (`quantifier/utils/parser.rs:22`).
- Strict and lenient: `LlmBackendType::from_str` refuses an unknown name (`llm_backend.rs:23-31`); the World create and update handlers default (`http/worlds/handlers/worlds.rs:74-88`); the posture handler refuses (`worlds.rs:238-257`); an unknown `check_mode` turns text check off (`http/settings/handlers/settings.rs:132-137`).
- Areas: 02 MP-1, MP-10; 03 M13; 04 MP-5, MP-6; 05 MP-4, MP-12.

### 2.3 Identity, time, naming

**XM-13. Where does a newtype begin for an identifier?**
- Newtypes: `GameId(u64)` (`domain/model/game.rs:12-14`), `OfferedSet` (`agent.rs:143`), `RunId` (`gate.rs:96`), `CurrentGame(GameId)` (`http/utils/current_game.rs:17`).
- Raw: `Game.id` is `u64` (`game.rs:24`), `DockRecord.offered` is `Vec<String>` (`dock.rs:11`), Job ids and Message ids (`gate.rs:91,445,472`), `GameCatalogue` takes `id: u64` and wraps it inside (`catalogue.rs:67,77,88`), the path id is a raw `u64` (`http/games/handlers/games.rs:119,131`).
- Identity kinds: numeric Game and Message ids, string World and Persona keys, a text preset id, logical references that are not foreign keys (`reference/storage.md:18-20,97,103,140-141`; `explanation/storage_design.md:68`). Ids are minted by the adapter from the clock and a UUID (`http/utils/handler_helpers.rs:19-28`), by storage, or by the user (`http/worlds/handlers/worlds.rs:25`).
- Areas: 02 MP-4; 03 M11; 05 MP-13, MP-15, C12; 09 MP11.

**XM-14. Which effects does the code read in place, and which does it take as a parameter?**
- In place: `Room::new_dynamic` reads `SystemTime::now` in whole seconds (`domain/model/map.rs:52-58` ✔), so two new rooms in one second may share an id (inferred, 02 Q2); `Message::new` reads `Utc::now` (`message.rs:70`); storage stamps time at 10 sites (`storage/games.rs:40`; `storage/transaction.rs:70`); the adapter builds an id from the clock (`handler_helpers.rs:19-28`); `llm_recorder.rs:112`.
- By parameter: `resolve_api_key_with(lookup)` (`settings.rs:323-331` ✔); name rules take an iterator (`utils/name_uniqueness.rs:5-9`); the Message timestamp and the LLM message time come from the domain through the mapper (`storage/transaction.rs:121`; `storage/mappers/llm_message.rs:49`); `generate_game_name` takes names and reads the date (`utils/game_name.rs:6-8`).
- In tests: fixtures call `Utc::now()` in some files, fixed strings in others, a 10 ms sleep in one (`narrator_tests.rs:123`; `tests/storage/transaction_storage.rs:11-12`; `storage/llm_messages_tests.rs:76`); no test pauses time.
- Areas: 02 MP-6; 04 MP-10; 05 MP-15; 06 MP-22; 08 MP-10.

**XM-15. Does the glossary rule reach code names and test names?**
- Glossary: "NPC" is on the Avoid list for Character; "Turn" is a Deprecated Term (`CONTEXT.md`).
- Code: `NpcCard`, `NpcEvent`, `NpcEncounterLog`, `npcs_in_area` (`domain/model/character.rs:61`; `quantifier.rs:13-41`; `trigger.rs:43-52`); `npc_ids`, `determine_npcs_in_room` (`quantifier/types.rs:30`; `quantifier/utils/orchestration.rs:16`); the rebuilt Quantifier result uses `present` and `destination` (`quantifier/types.rs:58-59`). The model's wire name is `npcs_in_room` (`quantifier/utils/parser.rs:14`).
- Tests: `struct Turn` (`agents/quantifier/agent_tests.rs:34`), `ScriptedModel::turn` (`test_support/scripted_model.rs:81`), `test_a_turn_that_fits_passes_through_untrimmed` (`narrator_tests.rs:322`).
- Docs: bold glossary terms in Reference docs (`reference/game_flow.md:8,32,34,98`).
- Areas: 02 MP-13; 03 M9; 08 MP-22; 09 C46.

### 2.4 Tests

**XM-16. Which fake stands for "the model is not real", and which does a test choose?**
- `ScriptedModel`: strict, one entry per call, fails on a stray call, panics at drop (`src/test_support/scripted_model.rs:1-8,196-207,262-268`). Three seam-contract files and tier 1 use it (`narrator_tests.rs:23` ✔).
- `MockBackend`: lenient, a runtime provider, shipped in the production tree and chosen by config (`llm/providers/mod.rs:5,19`; `wiring.rs:36`); six unit files use it; `tests/STRATEGY.md:15` ✔ names it as the unit-tier fake, while the seam contracts use `ScriptedModel`; `tests/http/settings.rs:10` uses it in tier 1.
- Hand-written stubs: three files for the LLM port and three for the text checker; one Quantifier is tested twice, through the Agent with `ScriptedModel` and through an inner function with four stubs (`agents/quantifier/utils/orchestration_tests.rs:100-108`).
- `SilentModel`, a TCP listener that never answers (`tests/test_utils/server.rs:284-311`); `StubServer` for the whole engine (`tests/test_utils/stub_server.rs:242`); a no-op save fn (`test_support/fixtures.rs:551`).
- The mock records a hand-built raw response with unescaped text (`mock.rs:78`) and decides by agent name (`mock.rs:120-121,169`).
- One signal, three checks: the error text holds a marker, a drop panics, and a tier-1 helper scans stored failures for the marker (`scripted_model.rs:24`; `tests/http/support/http_requests.rs:128-141`).
- Tests of the fakes: `mock_tests.rs` (228 lines) and `core_tests.rs:4-25`; no test file for `ScriptedModel`.
- Areas: 04 MP-2, MP-18; 06 MP-7; 07 MP-1, MP-4; 08 MP-1, MP-16, MP-20, Q9; 09 MP2.

**XM-17. Where does one behaviour get its test, and which tier-1 constructor builds its state?**
- Handler behaviour at two tiers: text-check save (`http/settings/handlers/settings_tests.rs:59` and `tests/http/settings.rs:107`), set narrator (`settings_tests.rs:442` and `tests/http/settings.rs:253`), duplicate preset (`prompt_presets_tests.rs:98` and `tests/http/prompt_presets.rs:780`).
- Storage behaviour at two homes: `seed_world` idempotence (`storage/worlds_tests.rs:68` and `tests/storage/world_storage.rs:556`); 20 `test_sqlite_` twins that differ only in the constructor (`world_storage.rs:431,657` ✔ per the coordinator check).
- Five constructors for a test storage: `Storage::new_in_memory()`, `empty_sqlite_storage()`, `create_test_storage(id)`, `Storage::new_sqlite(pool)`, `sqlite_storage()` (`core.rs:123`; `fixtures.rs:427,475`; `tests/helpers/fixtures.rs:7`). Four ways to put a Game row into storage, two of them below the storage layer (`fixtures.rs:413-473`).
- Three fixture styles for the same Character: `TestNpc::named`, `make_npc`, an inline `NpcCard` (`fixtures.rs:91`; `quantifier.rs:22`; `test_data_builder.rs:75`).
- Helpers copied in several files (`state_in` in three files; `messages_of` in two) while `story_of` is shared (08 MP-11).
- Mirror rule: `tests/AGENTS.md:27-35` says tests mirror `src/`; browser files mirror specs (`STRATEGY.md:231-233`); `tests/http/races.rs` gathers seven specs (`tests/AGENTS.md:97`).
- Two registries of seam contracts: a Rust list of three files (`unit_test_allowlist.rs:91-96`) and a script that scans every `*_tests.rs` (`scripts/validate_seam_cases.py:23-24`).
- Docs: "tier-1" has no definition in `docs/diataxis/` (09 MP17).
- Areas: 07 MP-2, MP-3, MP-5, MP-6, MP-18; 08 MP-2, MP-3, MP-4, MP-5, MP-11, MP-12; 09 MP17.

**XM-18. Which scope of test serialisation holds, and how does a test wait?**
- Process-wide `Mutex` guards (`tests/test_utils/settings_guard.rs:5` ✔; `src/test_support/env_guard.rs:5`); a file lock per port across processes (`tests/test_utils/server.rs:417-433`); a runner that gives each test its own process (`.config/nextest.toml:5`; `Cargo.toml:95`). The env guard says it exists for `cargo test --lib` threads (`env_guard.rs:8-9`). Does a Mutex hold anything when each test is a process? (inferred: no; 07 MP-11)
- Production bans `static` and `Atomic` in storage; the test layer holds five statics (`walker_rows.rs:14-18`; `tests/test_utils/server.rs:13,17,27`). One `SERVER_MANAGED` flag serves all servers, and a drop sets it false (`server.rs:13,614`).
- Waiting: HTTP poll of `idle` (`tests/test_utils/wait.rs:9-35`), in-process read of the job entry (`http_requests.rs:102-114`), DOM poll (`wait.rs:114-134`), the htmx settle counter (`htmx_settle.rs:277-282`), the held-call handshake (`scripted_model.rs:305,321`), real sleeps (`requires_migration/text_check.rs:29`; `tests/llm/flow_llm_tests.rs:38`). The first one posts a reset after a timeout (`wait.rs:28-32`).
- Missing prerequisite: panic (`wait.rs:37-40`), return early (`tests/llm/flow_llm_tests.rs:50-53`), `#[ignore]`.
- Real ports 3010 to 3050 and database files next to the executable (`tests/bootstrap/run_branches.rs:14-50`); the docs describe the file-locked range and the opt-out (`integration_test_standards.md:159-171`).
- Areas: 06 MP-17; 07 MP-10, MP-11, MP-12; 08 MP-7, MP-8, MP-9; 09 MP18.

**XM-19. What does the shipped binary contain from the test layer?**
- `Cargo.toml:55` ✔ sets `default = ["testing"]`. Under it the library exports `TestAppBuilder`, `TestDataBuilder`, `test_support` and `build_app_graph_for_tests` (`src/lib.rs:46-52`; `wiring.rs:179-180`); storage failure hooks and `new_in_memory` (`storage/core.rs:76,123-136`); Agent and connection-test hooks (`agents/options/agent.rs:39`; `agents/quantifier/agent.rs:42`; `connection_test_service.rs:88`); a test-only `JobTasks` (`generation/job_tasks.rs:6-7`, "built into production because `testing` is a default feature"); `AppState` hooks (`http/app_state.rs:65-74`). 22 sites in 8 files (07 T10).
- `build.py` builds with `cargo build` (`--release` only when `args.release`, `build.py:1802` ✔), and a search of `build.py` finds no `--no-default-features` and no `--features` ✔. Asset 07 Q-2 wrote "`cargo build --release` with no feature flag" for line 1802; that line is the LLM-test build, and the `--release` flag is conditional. I did not trace the packaging step. So the default build has the `testing` feature on (known); whether the packaged release binary has it is still inferred.
- Production methods that "only tests call" (`view_query.rs:188-191`; `narrator.rs:65-67`; `tests/STRATEGY.md:98-104`).
- `diagnostics` is on in `testing`, so `assert_state_consistency` and the `Contradiction` failure may depend on the same flag (`Cargo.toml:55-57`; `game_rules.rs:218-241`; 02 Q1).
- Areas: 02 Q1; 03 `#32`; 04 `#8` `#9`; 05 C32; 06 MP-21; 07 T9, T10, Q-2; 08 C4, Q1.

### 2.5 Logging, single source, guards

**XM-20. Which style gives a log line its context?**
- Span and fields: the Job span has `game`, `job`, `kind`, `step` and moves into `spawn_blocking` (`pipeline/job.rs:63-87`); the Autoplay driver logs fields with no span (`autoplay/driver.rs:46,48,62,88`).
- Bracket tags: `"[Quantifier] ..."`, `"[Options] ..."` (`quantifier/utils/orchestration.rs:78,97,111`; `options/utils/orchestration.rs:60`); `[LLM][req:N]` from a global `AtomicU64` (`llm/transport/utils/request.rs:8-12`; `response.rs:51`); a job id inside the text (`job.rs:170,397`).
- Where an error is logged: the LLM adapter logs `error` and returns (`response.rs:11-16`), the recorder logs `warn` (`llm_recorder.rs:70-73`), the HTTP edge logs `error` again (`http/utils/fragment.rs:19-22`); counts of `error!` per layer: domain 0, application 1, driven 12, driving 7 (06 MP-15).
- Does an LLM adapter line carry the Job id through the span? (inferred: yes when the span is entered on that thread; 06 MP-16)
- Areas: 03 M14; 04 `#43`; 06 MP-15, MP-16.

**XM-21. Where does "one home" stop?**
- Claims: "the naming rule has one home" (`domain/model/game.rs:122-125`); "one home of the Message kind" (`generation.rs:90-93`); single sources named in comments for fragments (`http/games/templates/games.rs:128-130`; `prompt_presets.rs:25-27`; `http/view_models.rs:27-28`); a column list derived once (`storage/worlds.rs:400-412`, `WORLD_COLUMNS`; `storage/stories.rs:19-24`).
- Copies: the 18-column Game list in three places (`storage/games.rs:16,62`; `storage/models/game.rs:30-51`); the rename form twice (`http/games/templates/games.rs:40-46,102-108`); the World list refresh three times (`worlds.rs:96-105,117-125,196-203`); `<RecentHistory>` and `<CurrentRoom>` in two Agent prompts and a third history form in the assembler (`options/prompt.rs:43-85`; `quantifier/prompt.rs:57-99`; `assembler.rs:302-332`); the retry count written as a constant and as a local (`options/utils/orchestration.rs:16`; `quantifier/utils/orchestration.rs:90`); four spellings of the agent name (`quantifier/utils/orchestration.rs:94`; `options/utils/orchestration.rs:77`; `narrator.rs:132`; `llm_provider.rs:11`); test helpers copied per file (08 MP-11); the default context size in two homes (`token_budget.rs:4`; `settings.rs:378-384`).
- Docs: "single home per fact" (`docs/AGENTS.md:25,34,40`) with doc facts restated across Reference and Explanation (09 MP12, MP13).
- Areas: 02 `#12` and MP-5; 03 M6, M15, M16; 04 MP-7; 05 MP-11; 06 MP-18; 08 MP-11; 09 MP12.

**XM-22. Does a guard name only things that exist, and does a stale entry fail?**
- Stale entry fails: the unit-test allowlist, walker rows, path-limited rules (`unit_test_allowlist.rs:187-207`; `walkers.rs:94-109`; `guardrails/mod.rs:164-190`).
- Stale entry passes: `free_fn.rs:22-28` names `application/generation/slot.rs` and `application/pipeline/spawn.rs` ✔ (neither file exists in those folders), `layers.rs:67-68` names `renderers.rs` and `fragment_renderers.rs`, `structure.rs:135` names `server/mod.rs`, `layers.rs:280` tests with the path `server/mod.rs`.
- Messages name code that is not in `src/`: `app_err_to_response()` (`layers.rs:86`) and `ApplicationService` (`arch-lint.toml:162-166`) (06 SN-9, hazard).
- A substring match: `no_std_thread` skips any path that contains `mock.rs` (`structure.rs:169`).
- Rule numbers come from a plan: rows cite Rules 1, 2, 3, 6, 7, 13, 18 and the generated doc names a file in `.scratch/` as the source (`walker_rows.rs:11-104`; `reference/coding_standards/guardrails.md:114`; `docs/AGENTS.md:241`).
- Areas: 06 SN-9; 07 MP-7, MP-6, MP-16, MP-17, Q-5, Q-6; 09 Q2, Q3.

### 2.6 Meeting points that stay inside one area

These are listed by theme and by reference (asset and number). I did not merge them. A reference in parentheses points to the cross-area point that already holds a close case.

- **Parsing and typing (inside the domain).** 02 MP-2 (illegal state made unrepresentable or repaired), MP-3 (silent no-op or `Result`), MP-10 (typed value or string), MP-11 (three kind enums, two request enums), MP-14 (sentinel or `Option`, see XM-13 and row I18).
- **Copy against reference (inside the domain).** 02 MP-8 (copy of the card or the id), MP-12 (copy at creation or live reference), MP-15 (one failure shape or two; see XM-4).
- **Application style.** 03 M6 (retry shape), M7 (parameter object, literal or builder), M10 (booleans and sum types in `Job::run`), M12 (tuple or named result; see 05 shape S7), M16 (recorded or unrecorded call; see XM-21).
- **Storage style.** 04 MP-7 (const column list or literals; see XM-21).
- **Markup and views.** 05 MP-6 (Askama or `format!`), MP-7 (three marks for trusted markup, two escape functions), MP-8 (view model or domain type), MP-9 (logic in template or view), MP-20 (`hx-*` or `onclick`), MP-22 (what a handler returns), MP-26 (page shell embedded, assets from disk).
- **Interaction.** 05 MP-13 (Game by server pointer or by URL), MP-14 (autosave and explicit save), MP-16 (what an unchecked checkbox means), MP-18 (full refresh or fragment), MP-19 (busy as a normal reply), MP-21 (folder per feature or per kind).
- **Boot and shell.** 06 MP-12 (poison policy; `parking_lot` unused), MP-19 (one reader of the World folder; see XM-10), MP-20 (World fallback, Persona stop; see XM-9), MP-24 (forensics save as part of success).
- **Guardrails.** 07 MP-8 (three checks on one boundary), MP-9 (compile-time and test-time guard of the writer rule), MP-13 (canned or rendered stub fragments), MP-14 (a browser per test or one shared), MP-15 (three policies for a failed arrange step), MP-19 (production methods that only tests call), MP-20 (config and code layer rules).
- **Test bodies.** 08 MP-13 (one clause or one scenario), MP-14 (error assertion shapes), MP-15 (table or one function per row), MP-17 (a test of a behaviour that has no code yet), MP-18 (serde derive under test), MP-19 (stub fragment rendered or canned), MP-21 (read seam, `Storage` or raw SQL), MP-23 (fail the call or corrupt the record).
- **Docs.** 09 MP8 (ids between steps, values into Agents), MP13 (a Reference doc that gives a reason), MP14 (a seam kept ahead of need), MP15 (`{{…}}` with two jobs), MP16 (code-indexer docs or seam identifiers), MP19 (status and health signals), MP20 (a model call and shutdown), MP22 (diagram style).
- **Cross-area points that hold the rest.** 02 MP-1, 4, 5, 6, 7, 9, 13; 03 M1, 2, 3, 4, 5, 8, 9, 11, 13, 14, 15, 17, 18, 19; 04 MP-1 to MP-6, MP-8 to MP-18; 05 MP-1 to MP-5, MP-10 to MP-12, MP-15, MP-17, MP-23 to MP-25; 06 MP-1 to MP-11, MP-13 to MP-18, MP-21 to MP-23; 07 MP-1 to MP-7, MP-10 to MP-12, MP-16 to MP-18; 08 MP-1 to MP-12, MP-16, MP-20, MP-22; 09 MP1 to MP7, MP9 to MP12, MP17, MP18, MP21.

### 2.7 Citations spot-checked in this session

All matched the code. (1) `src/application/generation/gate.rs:240` (the doc line; asset 03 cites `:246`). (2) `src/application/games/catalogue.rs:81-85`. (3) `src/adapters/driven/storage/worlds.rs:175-182` and `:312-335`. (4) `src/bootstrap/wiring.rs:159-167`. (5) `tests/STRATEGY.md:15` and `src/application/narration/narrator_tests.rs:21-24`. (6) `Cargo.toml:53-57`. (7) `src/application/errors.rs:22-31`. (8) `src/domain/model/map.rs:50-60`. (9) `docs/diataxis/explanation/architecture.md:12-15` and `reference/architecture_system.md:30`. (10) `docs/diataxis/reference/coding_standards/integration_test_standards.md:150-154` and `explanation/rust_idioms.md:20-24`. (11) `tests/test_utils/settings_guard.rs:1-5` and a search of `src/` for a static `AppSettings` (no hit). (12) `src/bootstrap/run.rs:177-181`, `src/adapters/driving/http/error.rs:14-17`, `src/adapters/driving/http/games/handlers/games.rs:223-227`, `src/adapters/driven/llm/providers/deepseek.rs:18-30`. (13) `tests/infrastructure/guardrails/free_fn.rs:22-28` against `ls src/application/generation src/application/pipeline` (no `slot.rs`, no `spawn.rs`). (14) `build.py:795-822,1802` and a search of `build.py` for `--features`.

## 3. Shapes with no name found by more than one area

Each shape has no pretrained name in the assets. "Areas" lists who saw it. A shape that one area names as a concept and another area sees without a name is listed here too. Labels follow section 0; all of these are *inferred* unless the line says otherwise.

| Row | Shape | Areas and evidence |
|---|---|---|
| Sh-01 | **A reason line plus the raw text behind a disclosure**, as one pair | 02 (`state/generation_status.rs:125-135`; `dock.rs:17-22`); 03 S2 (one failure, one writer of the line: `gate.rs:590-597`); 05 C16 (`http/utils/error.rs:24-31`; `http/builders/headers.rs:185-187`); 06 SN-8; 09 S2, S3 (`two-state-channels.md:18,20`). *Known* (four code areas cite code). The glossary names it: Failure line and Notice. |
| Sh-02 | **A twin that differs in one detail** (two copies of ten lines, two macros, two error constructors, two tests, two serde structs) | 02 (`WorldCard` and `WorldManifest` repeat the same serde defaults, `world.rs:17-32,42-59`); 04 S7 (`ollama.rs:67-84` and `openrouter.rs:48-65`); 05 S6, S9, S10 (`prompt_presets.rs:58-79`; `http/utils/error.rs:11-20`); 08 SN-6 (`test_sqlite_` twins, `world_storage.rs:431,657`). *Known*. |
| Sh-03 | **A zero-argument wrapper over a `_with` function** that takes the effect | 02 (`settings.rs:317-323,333-340`); 06 SN-3 (same lines). *Known*; one site, seen twice. |
| Sh-04 | **A services struct copied field by field into a second struct** | 05 C31 (`http/app_state.rs:46-59`); 06 SN-1 (the boot builds the Game wiring twice, `run.rs:102-107`; `wiring.rs:105-119`) and SN-2 (`wiring.rs:84-96` against `app_state.rs:31-57`). *Known*. |
| Sh-05 | **An anonymous tuple across a layer** (two strings, three records, two booleans) | 03 M12 (`view_query.rs:52`; `world_catalogue.rs:26-29`; `settings_service.rs:57`; `narrator.rs:73`); 05 S7 (`http/app_state.rs:113-125`; `worlds.rs:139,157,259`); 02 shape "Option accessor and Result accessor" is the sibling. *Known*. |
| Sh-06 | **A marker string in a fake's error text, matched by substring in another layer** | 07 S-7 (`scripted_model.rs:22-24`; `tests/http/support/http_requests.rs:135`); 08 SN-5 (`http_requests.rs:117-141`). *Known*; one site, seen twice. |
| Sh-07 | **A text status protocol that the client and the tests both poll** (`idle`, `failed`, `autoplay 2/5 narrating`) | 05 S1 (`http/layout/handlers/endpoints.rs:75,91-103`); 07 MP-12 (`tests/test_utils/wait.rs:9-35` polls `idle`); 08 MP-8 (`http_requests.rs:102-113`); 09 C37. *Known*. |
| Sh-08 | **A seed file is a template that the engine reads once** (later edits do not reach an existing row) | 04 MP-13 (`storage/settings.rs:118-123`; `characters.rs:59`); 06 SN-6 (`load.rs:128-133`, directory shape names the group); 09 S13 (`storage_design.md:40`). *Known*. |
| Sh-09 | **One default in several places** | 04 S5 (the SQL `DEFAULT 'novel'`, the migration `COALESCE`, `parse_or_default`: `plumbing.rs:327,372-375`; `models/game.rs:63-65`); 06 MP-18 (context size, `2048` twice, `180`); 03 Q10 (default Agents). *Known*. |
| Sh-10 | **A rule kept by a comment only** (a deadlock rule, a lock order, a drop rule) | 03 #11 and Q5 (`gate.rs:145-146,409,843-853`); 04 S8 (`storage/transaction.rs:47-48,281-283`); 06 MP-13 (the lock order is written for the guard and the two); 09 MP6 ("Where is the lock order written?"). *Known*. |
| Sh-11 | **A record of its own, so that a second write path cannot overwrite it** | 02 #7 (`dock.rs:6-7`); 04 S4 (`plumbing.rs:652-656`; `docks.rs:9-17`; the failure line stays a `games` column, `transaction.rs:210-217`); 09 MP5, MP7 (`storage_design.md:50`). *Known*. |
| Sh-12 | **The model reports what it sees; the engine computes the transition** (a diff of two lists) | 02 #10 (`quantifier.rs:23-25,41-65`); 09 S6 (`agent_system_design.md:26-30`); 03 (Quantifier result and static Characters, `quantifier/utils/orchestration.rs:142-199`). *Known*. |
| Sh-13 | **A hook by position with one Agent per hook; the hook trait fixes the kind of result** | 03 S3 (`agents/trait_def.rs:8,17`; `runner.rs:15-18,44-48`); 09 S5, C44 (`agent_system.md:19-21`). Also the glossary entry Hook. *Inferred*. |
| Sh-14 | **A design rule as a data row, a generated doc table, and rule numbers from a plan** | 07 G3, G4, Q-5 (`walker_rows.rs:6-27`; the numbers 1, 2, 3, 6, 7, 13, 18); 09 S10, Q3 (`guardrails.md:60,65-71,114`). *Known* for the rows; the plan source is inferred. |
| Sh-15 | **A module header tag that tools read** (`[DOC: path]`, `[TRIVIAL_ENUM]`) | 02 (tags); 03 S11 (all 54 files); 06 C60 (210 of 307 files); 07 S-6 (the first doc line is also the index entry in `tests/AGENTS.md`); 09 Q1. *Known*. Row I10 holds it as an idiom. |
| Sh-16 | **Markup or page content as a contract with client script and tests** | 05 S2 (`actions.rs:24-26`; `presets.rs:121`); 07 T28 (the htmx settle counter, `htmx_settle.rs:1-12`); 08 SN-1, SN-7 (the wiring guard test; hand-scanned markup). *Inferred*; the three are not one structure. |
| Sh-17 | **A failed step that is stored before the answer goes out** (the failure line, the pre-analysis state, persist first and move the pointer second) | 03 S1, S2 (`gate.rs:340-343,590-597`; `catalogue.rs:60-62`); 09 S2, S3, S4 (`two-state-channels.md:18,20`; `storage_design.md:84`). *Inferred* (one code area). |

### Single-area shapes, by reference

- **Domain 02** (the list has no numbers; names are shortened): kind, payload and row enum trio; hand-written enum-to-string trio next to a serde rename; "one home" comment; fallback chain of `Option`s; lazy record on first write; `HashMap<usize, bool>` used as a set; positional identity of a Trigger; a selected item stored as an index or id beside its list; rebuild constructors on production types; error used as control flow (`walk`).
- **Application 03**: S4 (agent split into four files), S5 (a builder that only formats), S6 (prompt as a string of tagged blocks), S7 (two confidences), S8 (`finish_non_exhaustive` Debug by hand), S9 (constructor named after the config, config unused), S10 (join handles for tests), S12 (static regex helper), S13 (run-for-run second entry point).
- **Driven 04**: S1 (the storage method envelope, 38 times), S2 (hand-rolled "no rows is `None`"), S3 (the list-query ritual), S6 (orphan warning inside a migration).
- **Driving 05**: S3 (dual-use fragment), S4 (error as view input), S5 (four constructor shapes for data to view), S8 (checkbox grammar).
- **Composition 06**: SN-4 (an error variant as a context wrapper), SN-5 (two `Result` spellings), SN-7 (message-string tags), SN-9 (a guard message that names code that does not exist; see XM-22), SN-10 (a real DB file named by the port), SN-11 (fall back, warn, go on).
- **Test infrastructure 07**: S-1 (path-string allowlist constants), S-2 (rule triplet), S-3 (comment-skip prefilter), S-4 (wait loop that ends in a dump and a panic), S-5 (handle per topic), S-8 (scenario id chain), S-9 (default struct plus override).
- **Test bodies 08**: SN-2 (tagged-value mapping check), SN-3 (the spec-tag line), SN-4 (`STOP_ROWS`), SN-8 (a check named by its function), SN-9 (a test pinned to a migration step), SN-10 (the quarantine with spec-less tests).
- **Docs 09**: S1 (busy as the presence of an entry that is also the owner's token), S7 (a roster of all Characters and full cards for those in the room), S8 (constraints after the history), S9 (a "who owns what" table), S11 (the narrator is narrative-only), S12 (a named outside model for structure).

## 4. Questions

These are the questions and the "hazards to check" of the eight assets, merged. A line that merges two or more items names all its sources. Source form: `02 Q3` is question 3 of asset 02; `03 H2` is the second item of the "Hazards seen" list of asset 03 (counted in the order of that list); MP and shape references are as in sections 2 and 3. Hazards are inferred and not tested, unless the asset says more. I did not run any of them.

### 4.1 Release builds and test hooks

- **Q01.** Does a default build, and the release binary, carry the `testing` hooks (storage failure layer, `TestAppBuilder`, `build_app_graph_for_tests`, the `JobTasks` watch, `AppState` hooks)? The default build does (`Cargo.toml:55` and no `--features` in `build.py`, checked in 2.7); the release package is untraced. — 07 Q-2; 08 Q1, 08 H1; 06 MP-21.
- **Q02.** Is `diagnostics` on in a release build, so that `assert_state_consistency` and the `Contradiction` failure can fire? `testing` turns it on (`Cargo.toml:55-57`). — 02 Q1.
- **Q03.** Is `JobTasks::wait` (the only `async` in the application layer) a test seam by design or the start of a wider move? — 03 Q7.
- **Q04.** Which guardrails run on test support in `src/test_support/` and on `tests/test_utils/`? Only five rules scan `tests/`. — 07 Q-4.
- **Q05.** Why four homes for test support (`src/test_support/`, `tests/test_utils/`, `tests/helpers/`, `tests/http/support/`): by binary or by layer? — 07 Q-3.
- **Q06.** Does the test graph skip a step of the production graph? `build_app_graph_for_tests` builds the Options recorder from stored settings while the narrator and the Quantifier use the fixed mock. — 07 Q-8; 06 H5.
- **Q07.** Is the storage failure layer a fake of the storage seam or part of the storage box? — 07 Q-7.

### 4.2 Concurrency, locks and atomicity

- **Q08.** `rename_game` does not go through `config_write`. May it run while an Autoplay run holds the Game, and what is the effect? — 03 H1.
- **Q09.** `delete_game` and `switch_game` read the active pointer outside the writer's hold. Can a switch and a delete race? — 03 H2; 03 Q2 (is "a request cannot end up writing two Games" a stance or a detail of the pointer?).
- **Q10.** `GameViewQuery::status` reads two stores in two steps, and `get_debug_state` reads the tail twice. Is a read across two sources meant to be a snapshot? — 03 H3 (03 M19).
- **Q11.** `JobWriter::drop` takes the non-reentrant registry lock. Does any type rule out a drop inside a `hold` closure, and where is that written? — 03 H4, Q5.
- **Q12.** A panic in a plain Job (not a run) frees the entry through `Drop` during the unwind. Is that path tested? — 03 H5.
- **Q13.** After `RunEnd::JobFailed`, is the order "failure line first, run hold second" pinned by a test? — 03 Q6.
- **Q14.** A `Storage` call inside a unit closure waits for ever on the mutex the unit holds, and only a comment prevents it. Is a guard wanted? — 04 H6.
- **Q15.** `update_world` writes the world row and the map row with no SQL transaction. Is a failed map write left behind? — 04 H5.
- **Q16.** `DbPool` holds one connection and `Storage` wraps it in a second `Mutex`. Is "pool" a plan or a leftover, and what does the second lock protect? Can `conn()` run outside a backend closure? — 04 Q1; 06 MP-13.
- **Q17.** `PRAGMA foreign_keys` is set after `run_migrations`, and v28 and v30 delete children by hand. Is that order on purpose? — 04 Q3.
- **Q18.** Does any caller run `Storage` from an async task without `spawn_blocking`? — 04 Q5; 05 C43.
- **Q19.** The World details save and the posture autosave each read and write the whole World. Can two close requests overwrite each other? — 05 H3.
- **Q20.** `SettingsTestGuard` and `ApiKeyEnvGuard` are process-wide Mutexes and the build runs one process per test. Do they serialise anything? — 07 H2 (07 MP-11).
- **Q21.** One `SERVER_MANAGED` flag serves all servers and one drop sets it to false. — 07 H7.
- **Q22.** `parking_lot` is a dependency and nothing uses it, while every `lock()` ignores poison. Is it meant to replace `std::sync::Mutex`? After a panic in a half-finished write, which state does the next holder see? — 06 H6 (06 MP-12).
- **Q23.** `wait_for_llm_idle` posts a reset to the engine after a timeout. Does that change the thing it waits for? — 07 H4.

### 4.3 Boot, seeding and configuration

- **Q24.** `AgentRunner::from_configs` returns `Err` for an unknown Agent type and `.unwrap_or_default()` drops it with no log. Is the silent default meant? The same call on a `Result` reads differently from one on an `Option`; how many of the 42 `unwrap_or_default` calls are on a `Result`? — 06 H1, Q8 (06 MP-5).
- **Q25.** One malformed preset file ends `ensure_presets`, so later preset files do not seed; the same holds for a map file and a persona file. `startup.md` says such a file is skipped. Which is meant? — 06 H2; 06 Q1.
- **Q26.** Does `startup.md` describe an earlier loader (`default_scenario()` now lives in the domain)? — 06 Q1.
- **Q27.** When the log directory fails, the fallback logger sets `debug` and ignores `RUST_LOG`. Is that meant? — 06 H3, Q4.
- **Q28.** Does the log line name the real file? The appender adds its own date. — 06 Q2.
- **Q29.** Is the flush guard that `init_logging` returns for the console and fallback paths needed (it belongs to a second, unused writer)? — 06 Q3.
- **Q30.** When the exe path is not valid UTF-8, the database goes to a relative path (`to_str().unwrap_or("chronicler.db")`); when `current_exe` fails, the data dir is used. Do these happen? — 06 H8, Q5.
- **Q31.** A failed save of the success record fails a call whose text exists; a failed save of the failure record only warns. Which is the rule? — 06 H4 (06 MP-24).
- **Q32.** Why are there two functions to resolve the boot Game, in two files, and is `init_game` `pub mod` only for tests? Which of `wiring` and `init_game` need `pub`? — 06 Q6, Q7.
- **Q33.** Which files lack the `[DOC: ...]` header, and does a rule list them? — 06 Q9.
- **Q34.** Is the crate-wide `print_stdout` and `print_stderr` allow in `bootstrap/mod.rs` needed for more than `logging.rs`? — 06 Q10.
- **Q35.** After a World is re-seeded, does an edited Character file reach a database that already holds that Character? Which of schema, Rust type and row wins? — 04 MP-13 (inferred: no); 09 MP21.
- **Q36.** A provider is rebuilt on every call from settings. Is a provider meant to be cheap and stateless? The mock keeps counters that a rebuild resets. — 04 Q7.
- **Q37.** A DeepSeek connection can be saved and chosen, and then fails on every call. Is that the intended state until the provider exists? — 04 H7 (06 MP-4; 08 MP-17).
- **Q38.** `LLM_MESSAGE_RETENTION_LIMIT` is 50 and the prune runs in each save. Does the debug view depend on that number? — 04 Q6.

### 4.4 Domain

- **Q39.** `Room::new_dynamic` makes the id from the clock in whole seconds. Can two new rooms in one second get one id, so that the second replaces the first? — 02 Q2.
- **Q40.** `evaluate_triggers` walks a `HashMap`, and `fired_trigger` takes the first match. Which Trigger fires when two match? No test covers it. — 02 Q3.
- **Q41.** `TemplateVars::render` runs seven `replace` calls in a row. Does a persona text that holds `{{user}}` expand by order? — 02 Q4.
- **Q42.** `walk` turns any unknown destination, even `""`, into a new room. Is an empty destination meant to be a place? — 02 Q5.
- **Q43.** Does the domain need to know that DeepSeek is a stub (`check_api_key_available_with`)? — 02 Q6.
- **Q44.** Several domain files hold no test of their own (`dock.rs`, `llm_message.rs`, `name_uniqueness.rs`, `opening.rs`, `game_name.rs`). Is that on purpose? — 02 Q7.
- **Q45.** `StoryWindow.retrigger_allowed` is stored and `Tail::retrigger_allowed` is derived. Which one do callers trust? — 02 Q8.
- **Q46.** Is `GenerationStatus` and `RunProgress` in `domain/` because it is a rule or because several layers share it? — 02 Q9.
- **Q47.** Is "ports are traits in the application layer, never in the domain" a rule of the area or a result? — 02 Q10.

### 4.5 Application layer

- **Q48.** Does the catalogue treat the rebuilt shapes (`generation/`, `pipeline/`, `narration/`, the runner) as the target and the older ones (the parsers, the prompt builders, the catalogues, `sections.rs`) as context? Which plan or commit says so? — 03 Q1.
- **Q49.** Is `TokenBudget::fallback` live? — 03 Q3.
- **Q50.** Does `Self::span_of(...)` in `Job::run_for_run` drop its return value on purpose? — 03 Q4.
- **Q51.** `AgentConfig` is read for `enabled`, `agent_type` and `name`, and both Agent constructors ignore it. Is it meant to carry more? — 03 Q8.
- **Q52.** Are the hook traits open for extension or for tests, when the hook set is closed by a string match in `runner.rs`? — 03 Q9 (09 MP3).
- **Q53.** Which owns "the default Agents": `AgentConfig::defaults()` or `AppSettings`? — 03 Q10.
- **Q54.** Does the catalogue need a name for the layout "package by layer inside a feature folder" (`agents/*/utils/`, `prompting/builders/`)? — 03 Q11.
- **Q55.** Is `GameCatalogue` one concept (lifecycle, config, read) or three? — 03 Q12.
- **Q56.** The Quantifier prompt builder takes its context by value and the caller rebuilds a second one field by field. Does a new field need two edits? — 03 H6.

### 4.6 Driven adapters

- **Q57.** `&error_body[..500]` slices a `String` by byte and can panic on a multi-byte character. — 04 H1.
- **Q58.** A lint span that starts at the end of the text maps to byte 0 and covers the whole text. — 04 H2.
- **Q59.** An API error in an HTTP 200 body is recorded as `Http { status: 200 }`. — 04 H3.
- **Q60.** `mock.rs:78` builds raw response JSON with `format!` and unescaped text. — 04 H4.
- **Q61.** `ChatCompletionResult.system_prompt` and `.user_prompt` are filled and no reader was found. Are they for tests? — 04 Q2.
- **Q62.** Each LLM call builds a new `reqwest` client. Is no connection reuse chosen? — 04 Q4.

### 4.7 Driving adapters

- **Q63.** `AGENTS.md` says "HTTP/WebSocket server" and `src/` has no WebSocket or SSE code (checked by the coordinator). Is it an old claim? — 05 Q1.
- **Q64.** `/data` serves the `data/` folder over HTTP. Which files are in it at run time, and do any need to stay private? — 05 Q2 (the coordinator note names it a hazard too).
- **Q65.** `ok(...)` sets no `Content-Type` and `Html(...)` does. Do both reach htmx the same way? — 05 Q3, H4.
- **Q66.** Are the debug routes present in every build, on purpose? — 05 Q4.
- **Q67.** Is the lack of authentication and CSRF a stated rule (a local, single-user tool) or the state of the code? — 05 Q5.
- **Q68.** Where should presentation limits live (50 LLM messages, a 120-character preview, `maxlength="120"`)? — 05 Q6.
- **Q69.** Which of the three names `view_models`, `utils/view_models`, `view_mappers` holds which job? — 05 Q7.
- **Q70.** Should the constructor of a template take `view_only` instead of the caller setting it after? — 05 Q8.
- **Q71.** Is a handler suffix (`*_handler`, `*_fragment`, `*_form`, `*_panel`) a signal or history? — 05 Q9.
- **Q72.** `http/autoplay/handlers/mod.rs` registers no unit-test file. Is the E2E file the only test by design? — 05 Q10.
- **Q73.** If the deleted Game is the active one, what does the client show? — 05 Q11.
- **Q74.** After a successful World create, a failed `list_worlds` renders "No worlds defined". — 05 H1.
- **Q75.** A failed `list_personas` shows "No personas available". — 05 H2.
- **Q76.** `error_disclosure` escapes `message` but not `detail_html`, so a new caller can miss the escape. — 05 H5.
- **Q77.** `index.html` is embedded at compile time and the other assets are read from disk. Which needs a rebuild, and does a server started from another directory find them? — 05 H6 (05 MP-26).
- **Q78.** A mistyped or missing `check_mode` silently turns text check off. — 05 H7.

### 4.8 Test infrastructure

- **Q79.** Which "review-only" rules of `tests/STRATEGY.md` (10 places) gain a guard next? Is a guard cheap for the domain-outcome rule? — 07 Q-1.
- **Q80.** Where do the design rule numbers (1, 2, 3, 6, 7, 13, 18) come from, do the others exist, and where are they kept, when the generated doc names a `.scratch` plan and the convention exempts plans? — 07 Q-5; 09 Q3, H6.
- **Q81.** `no_std_thread` skips any path that contains the text `mock.rs`. Is a substring match meant? — 07 Q-6, H3.
- **Q82.** The quarantine pin counts tests, not files. Does a rename or a split hide a move out of the quarantine? — 07 Q-9.
- **Q83.** Some exemptions name paths that do not exist (`free_fn.rs:22-28`, `layers.rs:67-68`, `structure.rs:135`) and nothing fails; guard messages name `app_err_to_response()` and `ApplicationService`. Do those rules guard anything? — 07 H1; 06 H7.
- **Q84.** A `Warning` fails the build like an `Error`. What does the severity mean? — 07 H5.
- **Q85.** `TestAppBuilder` drops the result of the opening-Message seed and of the settings save with `let _ =`. — 07 H6.

### 4.9 Test bodies

- **Q86.** How many of the 78 legacy unit files hold a test that a tier-1 test also pins? (3 pairs by name were matched.) — 08 Q2.
- **Q87.** `tests/bootstrap/run_branches.rs` binds ports 3010 to 3050 and deletes database files next to the test executable. Do two runs from two checkouts meet? — 08 Q3, H2.
- **Q88.** Is a failed assertion inside a `check_*` function reported with its own name and text? — 08 Q4.
- **Q89.** Does `TestOverride::config` have a user outside the unit tier? — 08 Q5.
- **Q90.** Why does `tests/llm/flow_llm_tests.rs` have an inline `mod tests`, and does `scripts/check_test_structure.py` allow it? — 08 Q6.
- **Q91.** Is a `Storage` read in a unit test (14 files) a choice of the unit tier, when the tier-1 rule covers only tier 1? — 08 Q7 (08 MP-21).
- **Q92.** Does the confidence concept still exist in the Quantifier parser? The test file keeps three backends named by confidence. — 08 Q8.
- **Q93.** Who tests `ScriptedModel`? No `*_tests.rs` file was found. — 08 Q9.
- **Q94.** Which tier-1 names carry the `_http` suffix and why (128 do)? — 08 Q10.
- **Q95.** Why does `harper_text_checker_tests.rs` mark three imports `#[cfg(test)]`? — 08 Q11.
- **Q96.** `flow_llm_tests.rs:50-53` returns early (a pass) when the API key is absent; the `#[ignore]` hides this until it is lifted. — 08 H3.
- **Q97.** `llm_messages_tests.rs:76` orders two rows with a real 10 ms sleep. — 08 H4.
- **Q98.** The four canned stub fragments can differ from the templates. What stops that? — 08 H5 (08 MP-19).
- **Q99.** The `ActionForm` serde tests read JSON while the handler reads a form-encoded body. What do they pin? — 08 H6 (08 MP-18).

### 4.10 Docs

- **Q100.** Where do DOC anchors live? The ticket named them; none is in `docs/diataxis/`. The coordinator correction: they are `//! [DOC: ...]` headers in `src/` (210 lines), checked by `scripts/validate_docs.py`. Does any doc rely on an anchor in `src/`? — 09 Q1; ticket 09 Answer; 06 C60; 03 S11.
- **Q101.** What do the arch-lint scope names (`model`, `narrative`, `server`, `agent-edge`, `storage-models`) map to in the layer names of the docs? — 09 Q2.
- **Q102.** Where are the decisions recorded (the "hexagonal-reorganization superplan")? — 09 Q4.
- **Q103.** Does `docs/AGENTS.md` list the Document References that docs carry, and why do five Reference docs have no such section? — 09 Q5.
- **Q104.** Which place holds the page title, the H1 or the front matter (three reference docs have an H1, twenty do not)? — 09 Q6.
- **Q105.** Is `settings` a standalone table in `reference/storage.md`? The cluster list names it and the standalone list does not describe it. — 09 Q7, H5.
- **Q106.** Is "newest Game of a World" by creation or by last use? — 09 Q8.
- **Q107.** What covers a change to the prompt or the LLM parser when the real-LLM tests are `#[ignore]` and CI does not run them? — 09 Q9.
- **Q108.** Which negative statements fit the "positive constraint" form of the "no negative explaining" rule? — 09 Q10.
- **Q109.** Which of the two stated reasons holds for a concrete `Storage`, and does the other still stand as a doc sentence? — 09 Q11 (XM-1).
- **Q110.** What does the "Who Owns What" table not list (`AppState`, `CurrentGame`)? — 09 Q12.
- **Q111.** `explanation/rust_idioms.md:38` says what happens to requests at shutdown and not to a Job that runs after its request ended. — 09 H1 (09 MP20).
- **Q112.** `integration_test_standards.md:152-153` says `AppSettings` lives in a global static; `rust_idioms.md:22` says a database row. A search of `src/` finds no static. — 09 H2 (XM-8).
- **Q113.** `reference/narrative/prompt_system.md:43` filters `{{…}}` in player input; no doc says what happens to a closing `</PlayerInput>` tag in that text. — 09 H3 (09 MP15).
- **Q114.** `explanation/architecture.md:12` says "four LLM impls" and the diagram at `:40-42` shows three. — 09 H4 (09 MP14).

## 5. Docs against code

See `research-10-docs-vs-code.md`.
