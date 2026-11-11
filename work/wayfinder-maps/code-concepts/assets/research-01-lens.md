# Research 01: the lens

**This lens is a starting checklist, not a limit.** It lists the axes on which code can take more than one valid shape. The area tickets record what the lens misses. The open pass of each area ticket (done before it reads this file) is the test of the lens.

This file does not say what the code does. It does not rule on any concept. It names the poles of each axis so that an area ticket can ask: "Which pole does this code take, and where does it take the other?"

Status of this file: written for ticket 01. Code reading was light (module names, `Cargo.toml`, a few greps).

## How to read

- **Level.** Stance = a discipline that crosses modules. Pattern = a named structure of types and modules. Idiom = a local language-level choice (Rust here).
- **Axis.** One question with two or more answers. Each axis has an ID (`S` stance, `P` pattern, `I` idiom) so area tickets can cite it.
- **Pole.** One answer. Each pole has its pretrained name: the name a strong LLM already knows, so the name works as a leading word.
- **Source.** The family source is named once, at the head of each family. It is the place where the pole names come from.
- **Live.** "Live" means the repo can take either pole. The live mark is a guess from light reading: **inferred** unless stated.
- A pole can appear that the repo does not use. Keep it. An agent can only choose it on purpose if it has a name.

## Families

The ticket gives 14 families as a floor. They are numbered F1 to F14. I added 11 more, numbered F15 to F25.

| # | Family | Origin |
|---|---|---|
| F1 | Architecture styles | ticket |
| F2 | Application and enterprise patterns | ticket |
| F3 | Domain modelling | ticket |
| F4 | Object-oriented and functional design | ticket |
| F5 | Type-driven design | ticket |
| F6 | Error handling | ticket |
| F7 | Concurrency and state ownership | ticket |
| F8 | Persistence and transactions | ticket |
| F9 | Module and API design | ticket |
| F10 | Configuration and composition | ticket |
| F11 | Web and hypermedia | ticket |
| F12 | Rust idioms | ticket |
| F13 | Testing schools and test design | ticket |
| F14 | Documentation design | ticket |
| F15 | Evolution and change management | **added** |
| F16 | Background work and long-running operations | **added** |
| F17 | Serialization and wire formats | **added** |
| F18 | Data-driven content and templating | **added** |
| F19 | LLM-application design | **added** |
| F20 | Observability and diagnostics | **added** |
| F21 | Resource limits and performance | **added** |
| F22 | Quality gates and enforcement | **added** |
| F23 | Naming and code comments | **added** |
| F24 | Time, identity and randomness | **added** |
| F25 | Security and trust | **added** |

### Families judged not live (or live in one pole only)

| Family or pole | Why not live (label) |
|---|---|
| Distributed systems (consensus, saga, eventual consistency, outbox) | One process and one SQLite file. No network peer owns state. The only remote call is to an LLM provider. (inferred) |
| Microservices vs monolith (axis S1, pole "microservices") | One binary. The pole stays in the table so the name exists. (inferred) |
| Authentication and authorisation (F25) | No user, session or login in the module names. The only secret is a provider API key. The `auth` hits in `src/` look like the word "author". (inferred; one grep) |
| Active record and ORM (axis P9, poles "active record", "ORM") | `Cargo.toml` has `rusqlite` and no ORM crate. (known for the crate list) |
| Event sourcing and CQRS write side (axis P11) | The word "event" in `CONTEXT.md` means Trigger event Narration, not an event log. Keep the pole: the Swipe state looks like a snapshot per Swipe. (inferred) |
| Internationalisation | I found no locale code. Not listed as an axis. (guessed) |
| Plugin and dynamic loading (axis S3 pole) | No `libloading` or WASM crate in `Cargo.toml`. (known) |

---

# Level 1: Stance

A stance is a discipline that crosses modules. Most stances have an enforcement tool (a lint, a test, a type). That tool is not part of the stance.

## F1 Architecture styles

Source: Cockburn, "Hexagonal architecture" (2005); Martin, "Clean Architecture"; Palermo, "Onion architecture"; Fowler, "MonolithFirst".

| ID | Axis | Poles |
|---|---|---|
| S1 | Deployment unit | **Modular monolith** · **Microservices** · **Serverless functions** |
| S2 | Dependency direction | **Ports and adapters (hexagonal)** · **Layered architecture (UI → service → data)** · **Big ball of mud** |
| S3 | Where the extension seam sits | **Plugin architecture** · **Compile-time composition** · **Hard-wired concrete types** |
| S4 | Primary driver of structure | **Domain-driven (package by feature/domain)** · **Technical layering (package by layer)** |
| S5 | How the architecture is kept true | **Fitness function / architecture test** · **Convention and review** · **Documentation only** |

Note on S2: "driving" and "driven" adapters are Cockburn's own words.

## F5 Type-driven design (stance part)

Source: Alexis King, "Parse, don't validate" (2019); Yaron Minsky, "Make illegal states unrepresentable"; Wlaschin, "Domain Modeling Made Functional".

| ID | Axis | Poles |
|---|---|---|
| S6 | Where input is checked | **Parse, don't validate (check once at the boundary, then trust the type)** · **Validate at every layer (defensive programming)** · **Shotgun parsing (checks scattered through the code)** |
| S7 | Where invalid states are excluded | **Make illegal states unrepresentable** · **Runtime invariant check** · **Convention and comment** |

## F6 Error handling (stance part)

Source: Nygard, "Release It!"; Armstrong, "Let it crash"; Postel's law (RFC 761); Hunt and Thomas, "Dead programs tell no lies".

| ID | Axis | Poles |
|---|---|---|
| S8 | Reaction to a bad state | **Fail fast** · **Graceful degradation** · **Let it crash (supervise and restart)** |
| S9 | Strictness toward input | **Robustness principle (be liberal in what you accept)** · **Strict reader (reject the unknown)** |
| S10 | Who owns a failure record | **Error as value in the call path** · **Error recorded on the entity** · **Error only in logs** |
| S11 | Recovery after partial work | **Idempotent operation (converge on retry)** · **Compensating action (saga-style undo)** · **Atomic all-or-nothing write** · **Manual repair** |

## F7 Concurrency and state ownership (stance part)

Source: Hoare, "Communicating Sequential Processes"; Hewitt, "Actor model"; Thompson, "Single writer principle" (Mechanical Sympathy blog); Klabnik/Nichols, *The Rust Book* ch. 16.

| ID | Axis | Poles |
|---|---|---|
| S12 | Who may write a piece of state | **Single writer** · **Shared mutable state under a lock** · **Share nothing (message passing, actors)** · **Optimistic concurrency (version check)** |
| S13 | Where the writer rule lives | **Structural (types make a second writer impossible)** · **Runtime guard (registry, flag, lock)** · **Convention** |
| S14 | Source of truth for "busy" | **Persisted state** · **In-memory state (lost at restart)** |

## F8 Persistence and transactions (stance part)

Source: Fowler, PoEAA ch. 3, 9–11; Kleppmann, *Designing Data-Intensive Applications*; Vernon, IDDD ch. 10 (aggregate and consistency boundary).

| ID | Axis | Poles |
|---|---|---|
| S15 | Where truth lives | **Database as source of truth, memory as cache** · **Memory as truth, database as snapshot** · **Derived state, recompute from the log** |
| S16 | What a write unit covers | **One transaction per use case** · **One write per call (autocommit)** · **Eventual consistency across writes** |
| S17 | Stored vs derived | **Store the derived value (denormalise)** · **Derive on read (normalise)** |

## F15 Evolution and change management (added)

Source: Fowler, "StranglerFigApplication", "ParallelChange" (expand/contract) and "Refactoring"; Ford et al., "Building Evolutionary Architectures"; Hyrum's law.

| ID | Axis | Poles |
|---|---|---|
| S18 | Replacing an old API | **Migrate callers, delete the old API** · **Compatibility shim or deprecation period** · **Strangler fig (old and new live together)** · **Big-bang rewrite** |
| S19 | Stored data after a format change | **Migration script (versioned schema)** · **Tolerant reader (old rows still load)** · **Reset (drop and recreate)** |
| S20 | Speculation | **YAGNI (build when needed)** · **Speculative generality (hooks for later)** |
| S21 | Duplicate logic | **Rule of three (wait, then extract)** · **DRY at first sight** · **WET (accept the copy)** |

## F22 Quality gates and enforcement (added)

Source: Ford et al., "fitness functions"; Google, "Software Engineering at Google" (ch. on static analysis); Rust `clippy` docs.

| ID | Axis | Poles |
|---|---|---|
| S22 | How a rule is kept | **Executable check (lint, test, script)** · **Written standard** · **Review habit** |
| S23 | Strictness of warnings | **Warnings are errors** · **Warnings are advice** |
| S24 | Where the gate runs | **Pre-commit hook** · **CI server** · **Developer discretion** |

## F25 Security and trust (added; mostly not live)

Source: OWASP Top 10; Saltzer and Schroeder, "The protection of information in computer systems" (1975).

| ID | Axis | Poles |
|---|---|---|
| S25 | Untrusted text (user text and LLM output) | **Sanitise at input and at output** · **Escape at render time only (context-aware encoding)** · **Trust the producer** |
| S26 | Where secrets come from | **Environment variable** · **Config file** · **Secret store** |

---

# Level 2: Pattern

A pattern is a named structure. It has a shape that a reader can find in the types and the module tree.

## F2 Application and enterprise patterns

Source: Fowler, *Patterns of Enterprise Application Architecture* (PoEAA); Hohpe and Woolf, *Enterprise Integration Patterns*; Cockburn, "Use case"; Martin, "Clean Architecture" (interactor).

| ID | Axis | Poles |
|---|---|---|
| P1 | How application logic is organised | **Transaction script** · **Domain model** · **Table module** · **Service layer over a domain model** |
| P2 | Unit of an application operation | **Use-case class (interactor)** · **Service with many methods** · **Handler function** · **Command object (command bus)** |
| P3 | Where orchestration lives | **Application service (orchestrates, holds no rules)** · **Controller does the work (fat controller)** · **Domain object orchestrates** |
| P4 | Result of an operation | **Return a DTO** · **Return the domain object** · **Return a view model** |
| P5 | Read and write paths | **One model for both** · **CQRS (separate read side)** · **Query object / read-side service** |
| P6 | Process control | **Pipeline (ordered steps)** · **State machine** · **Workflow engine** · **Chain of responsibility** |
| P7 | Extension hooks | **Hook points with registered handlers (observer, template method)** · **Strategy object** · **Direct call** |

## F3 Domain modelling

Source: Evans, *Domain-Driven Design* (2003); Vernon, *Implementing Domain-Driven Design*; Fowler, "AnemicDomainModel", "ValueObject"; Wlaschin, *Domain Modeling Made Functional*.

| ID | Axis | Poles |
|---|---|---|
| P8 | Behaviour and data | **Rich domain model (behaviour on the entity)** · **Anemic domain model (data + service)** · **Functional domain model (data + pure functions)** |
| P9 | Identity | **Entity (identity)** · **Value object (equality by value)** |
| P10 | Consistency boundary | **Aggregate with a root** · **Free-standing records joined by id** |
| P11 | Record of change | **Current-state record** · **Snapshot per version** · **Event sourcing** · **Audit table** |
| P12 | Vocabulary | **Ubiquitous language (glossary drives names)** · **Technical naming** |
| P13 | Domain rule placement | **Pure rules module (game rules as a function of state)** · **Rules inside handlers** · **Rules in the database (constraints, triggers)** · **Rules as data (config-driven)** |
| P14 | Variants of a concept | **Sum type (closed set)** · **Class hierarchy (open set)** · **Flags and optional fields** |
| P15 | Cross-context mapping | **Anti-corruption layer** · **Shared kernel** · **Conformist (use the foreign model as is)** |

## F4 Object-oriented and functional design

Source: Bernhardt, "Boundaries" and "Functional Core, Imperative Shell" (Destroy All Software, 2012); Gamma et al., *Design Patterns*; Martin, SOLID; Wlaschin, "Functional design patterns"; Ousterhout, *A Philosophy of Software Design*.

| ID | Axis | Poles |
|---|---|---|
| P16 | Where effects live | **Functional core, imperative shell** · **Effects mixed through the code** · **Effect system (monad, IO type)** |
| P17 | Code reuse | **Composition over inheritance** · **Inheritance** · **Trait default methods (mixin)** · **Delegation** |
| P18 | Module depth | **Deep module (small interface, much inside)** · **Shallow module (thin wrapper)** · **Classitis (many tiny classes)** |
| P19 | Object lifetime | **Stateless service** · **Stateful service (owns a mutable field)** · **Immutable value passed through** |
| P20 | Dependency supply | **Dependency injection (constructor)** · **Service locator** · **Global / singleton** · **Pass as function argument** |
| P21 | Data flow through steps | **Pass state in, get new state out (immutable update)** · **Mutate in place (`&mut`)** · **Builder** |
| P22 | Interface polymorphism | **Strategy via trait object** · **Strategy via closure** · **Strategy via enum match** |

## F7 Concurrency (pattern part)

Source: Tokio docs ("Bridging with sync code"); Nystrom, "What color is your function?"; Goetz, *Java Concurrency in Practice*; Hewitt, actors.

| ID | Axis | Poles |
|---|---|---|
| P23 | Async model | **Async all the way** · **Synchronous core in `spawn_blocking`** · **Threads and channels** · **Actor per entity** |
| P24 | Cancellation | **Cooperative cancel (token, flag)** · **Abort the task** · **Run to completion** |
| P25 | Lock scope | **Coarse lock around the registry** · **Fine-grained lock per entity** · **Lock-free (atomics)** |
| P26 | Wait for a task | **Join handle** · **Notify / condvar** · **Poll** · **Fire and forget** |

## F8 Persistence (pattern part)

Source: Fowler, PoEAA ch. 10 (Data source architectural patterns), ch. 11 (O/R behavioural); Evans, "Repository".

| ID | Axis | Poles |
|---|---|---|
| P27 | Access to rows | **Repository (collection-like)** · **Table data gateway / DAO** · **Row data gateway** · **Active record** · **Data mapper (ORM)** · **Raw SQL in the caller** |
| P28 | Transaction scope in code | **Transaction object passed in (unit of work)** · **Implicit per-call transaction** · **Closure that runs in a transaction** |
| P29 | Row ↔ domain mapping | **Separate row struct and mapper** · **Domain type as row** · **Document (JSON blob)** |
| P30 | Schema ownership | **Migrations in code** · **Schema file applied at start** · **ORM auto-migrate** |
| P31 | Storage access object | **One storage struct (facade)** · **One trait per aggregate** · **Free functions on a connection** |
| P32 | Connection model | **Pool** · **Connection per request** · **Single shared connection** |

## F9 Module and API design

Source: Parnas, "On the criteria to be used in decomposing systems into modules" (1972); Rust API Guidelines; Ousterhout, ch. 4–6; Fowler, "Published Interface"; Evans, "Intention-revealing interfaces".

| ID | Axis | Poles |
|---|---|---|
| P33 | Module split | **By change reason (information hiding)** · **By processing step** · **By type kind (`models/`, `utils/`)** |
| P34 | Visibility | **Narrow `pub` (facade re-exports)** · **Wide `pub`** · **Crate-private by default** |
| P35 | Error-and-edge handling in an interface | **Define errors out of existence (Ousterhout)** · **Report every error to the caller** |
| P36 | Function signature style | **Take owned values** · **Borrow** · **Generic over `impl Into` / `AsRef`** |
| P37 | Interface width | **Many small methods** · **One method with options struct** · **Builder** |
| P38 | Grab-bag modules | **`utils/` and `helpers/` modules** · **Place each helper beside its one user** |
| P39 | Re-use of a seam | **Port defined by the caller (dependency inversion)** · **Interface defined by the provider** |

## F11 Web and hypermedia

Source: Fielding, REST (2000); Gross et al., *Hypermedia Systems* (2023); htmx essays ("HATEOAS", "Locality of behaviour", "Hypermedia-driven applications"); Fowler, "Presentation Domain Data Layering", "Model View Controller"; Hotwire docs.

| ID | Axis | Poles |
|---|---|---|
| P40 | Page model | **Hypermedia-driven app (server returns HTML fragments)** · **SPA (JSON API + client rendering)** · **MPA (full page loads)** |
| P41 | Where UI state lives | **Server (HATEOAS)** · **Client (JS store)** · **URL** |
| P42 | Template location | **Server-side template engine, compile-time checked** · **Runtime template** · **String building in code** |
| P43 | View data | **View model per template** · **Pass the domain type to the template** · **Anonymous map** |
| P44 | Handler shape | **Thin handler (parse, call, render)** · **Handler holds logic** · **Extractor does the parsing** |
| P45 | Route design | **Resource nouns (REST)** · **Action verbs (RPC)** · **One endpoint with a message type** |
| P46 | Progress of long work | **Polling** · **Server-sent events** · **WebSocket** · **Long poll** · **Reload** |
| P47 | Behaviour in markup | **Locality of behaviour (attributes in the HTML)** · **Separation of concerns (JS files bind later)** |
| P48 | Page refresh | **Swap a targeted fragment** · **Out-of-band swap** · **Morph the whole page** · **Redirect after POST** |

## F10 Configuration and composition

Source: Seemann, *Dependency Injection: Principles, Practices, and Patterns* ("Composition Root"); The Twelve-Factor App (factor III); Fowler, "InversionOfControl".

| ID | Axis | Poles |
|---|---|---|
| P49 | Where objects are wired | **Single composition root** · **Wiring at point of use** · **DI container / framework** |
| P50 | Settings source | **Environment variables** · **File (JSON/TOML)** · **Database (editable at runtime)** · **CLI flags** · **Compile-time constants** |
| P51 | Config when absent | **Default function per field** · **Fail on missing** · **Option and handle at use** |
| P52 | Feature variants | **Runtime config** · **Cargo feature flag** · **Separate binaries** |
| P53 | Test doubles in production code | **Mock provider in the shipped code** · **Fakes only in test crates** |

## F19 LLM-application design (added)

Source: Anthropic, "Building effective agents" (2024); Yan et al., "Patterns for building LLM-based systems"; LangChain/LlamaIndex vocabulary (chain, agent, tool); OpenAI structured-outputs docs.

| ID | Axis | Poles |
|---|---|---|
| P54 | Control flow around the model | **Workflow (fixed steps, code decides)** · **Autonomous agent (the model decides)** · **Single call** |
| P55 | Model result format | **Free text, parse with a tolerant parser** · **JSON schema / structured output** · **Tool calls / function calling** |
| P56 | Model vendor boundary | **Provider port with adapters (transport only)** · **One vendor SDK in the core** · **Gateway service** |
| P57 | Prompt construction | **Layered/sectioned assembler** · **Template string** · **Prompt as data (preset)** · **Prompt in code** |
| P58 | Context budget | **Fit by trimming oldest** · **Summarise** · **Retrieve (RAG)** · **Fail when over** |
| P59 | Model output into state | **Model proposes, code decides (rules apply the result)** · **Model writes state directly** |
| P60 | Role of the model | **Narrator only (text)** · **Analyser (extracts facts from text)** · **Generator of options** |
| P61 | Test substitute for the model | **Scripted fake (strict, ordered)** · **Recorded replay (VCR / cassette)** · **Mock library** · **Live call** |

## F20 Observability and diagnostics (added; pattern part)

Source: Majors et al., *Observability Engineering*; Rust `tracing` docs; Google SRE book ch. 6; Nygard, "Release It!" (transparency).

| ID | Axis | Poles |
|---|---|---|
| P62 | Record of a model call | **Forensics record in the database** · **Structured log line** · **Trace span** · **Nothing** |
| P63 | Log style | **Structured fields (`tracing`)** · **Formatted strings** · **`println!`** |
| P64 | Error to the user | **Short reason with raw text behind a disclosure** · **Generic message** · **Full stack to the user** |

---

# Level 3: Idiom

An idiom is a local language-level choice. Most idioms here are Rust. Some are web or SQL.

## F6 Error handling (idiom part)

Source: Rust Book ch. 9; Rust API Guidelines (C-GOOD-ERR); `thiserror` and `anyhow` docs; Blessed.rs; BurntSushi, "Error handling in Rust" (blog).

| ID | Axis | Poles |
|---|---|---|
| I1 | Error type | **Typed error enum (`thiserror`)** · **`anyhow::Error` (opaque)** · **`Box<dyn Error>`** · **`String` errors** |
| I2 | Error layering | **One error enum per layer, converted with `From`** · **One crate-wide error** · **Reuse a lower-layer error upward** |
| I3 | Propagation | **`?` operator** · **Explicit `match`** · **`.map_err(..)?`** · **`unwrap` / `expect`** |
| I4 | Panic policy | **Panic only on broken invariant** · **No panics (deny `unwrap`)** · **Panic in tests only** |
| I5 | Absence | **`Option`** · **Sentinel value** · **`Result` with a "not found" error** |
| I6 | Error to HTTP | **`IntoResponse` mapping in one place** · **Per-handler mapping** · **Always 200 with an error body** |

## F5 Type-driven design (idiom part)

Source: Rust API Guidelines (C-NEWTYPE, C-CUSTOM-TYPE); the "newtype pattern" in the Rust Book ch. 19; Cliff Biffle, "The typestate pattern in Rust"; Rust Design Patterns book.

| ID | Axis | Poles |
|---|---|---|
| I7 | Identifiers | **Newtype ids (`GameId(u64)`)** · **Raw `u64` / `String`** · **Type alias** |
| I8 | Closed sets | **`enum`** · **Strings with constants** · **Booleans** |
| I9 | Match | **Exhaustive match (no `_`)** · **Wildcard arm** · **`if let` chain** |
| I10 | State in the type | **Typestate (a type per state)** · **State enum field** · **Booleans and options** |
| I11 | Optional config | **`Option<T>` with `Default`** · **Builder** · **Struct update syntax** |
| I12 | Untyped data | **Typed struct (serde derive)** · **`serde_json::Value`** · **`HashMap<String, String>`** |

## F12 Rust idioms (other)

Source: Rust API Guidelines; Rust Design Patterns book; Rust Book ch. 13, 15, 16; *Rust for Rustaceans* (Gjengset); `clippy` lint groups.

| ID | Axis | Poles |
|---|---|---|
| I13 | Trait dispatch | **Enum dispatch** · **`dyn Trait` (`Arc<dyn Trait>`)** · **Generics (static dispatch)** · **Closures (`dyn Fn`)** |
| I14 | Async in traits | **`async_trait` macro** · **Native `async fn` in trait** · **Return `Pin<Box<dyn Future>>`** · **Sync trait, async outside** |
| I15 | Shared ownership | **`Arc<Mutex<T>>`** · **`Arc<RwLock<T>>`** · **Channels** · **`&mut` threaded down** |
| I16 | Lock crate | **`std::sync`** · **`parking_lot`** · **`tokio::sync`** |
| I17 | Strings | **Owned `String`** · **`&str`** · **`Cow<str>`** · **`Arc<str>`** |
| I18 | Clone policy | **Clone freely** · **Borrow and thread lifetimes** · **Share with `Arc`** |
| I19 | Initialisation | **`once_cell` / `OnceLock` for statics** · **`lazy_static`** · **Build at start and pass** |
| I20 | Derives | **`derive(Default, Clone, Debug)` on everything** · **Derive only what is used** |
| I21 | Conversions | **`From`/`Into`/`TryFrom` impls** · **Named `to_x` / `from_x` methods** · **`as` casts** |
| I22 | Iteration | **Iterator chains** · **`for` loops with mutation** |
| I23 | Constructors | **`new()`** · **Struct literal** · **Named constructors (`with_x`)** · **`Default`** |
| I24 | Module layout | **`mod.rs` files** · **`foo.rs` + `foo/`** · **Re-export from the crate root** |
| I25 | Unsafe | **Forbid `unsafe`** · **Contained `unsafe`** |
| I26 | Macros | **Derive macros only** · **`macro_rules!` for repeats** · **Plain functions** |
| I27 | Lints | **`#![deny(warnings)]` / clippy pedantic** · **Default lints** |

## F8 Persistence (idiom part)

Source: SQLite docs (WAL, `BEGIN IMMEDIATE`); `rusqlite` docs; Fowler, "Migrations" (Evolutionary Database Design).

| ID | Axis | Poles |
|---|---|---|
| I28 | Query building | **Hand-written SQL strings** · **Query builder** · **Compile-time checked SQL (`sqlx`)** · **ORM DSL** |
| I29 | Blocking driver in async code | **`spawn_blocking`** · **Block the executor** · **Dedicated thread** |
| I30 | JSON in a column | **JSON text column** · **Normalised table** |
| I31 | Row id | **Auto-increment integer** · **UUID** · **Natural key** |
| I32 | Delete | **Hard delete** · **Soft delete (flag)** · **Cascade by foreign key** · **Cascade in code** |

## F17 Serialization and wire formats (added)

Source: `serde` docs ("Container attributes"); Fowler, "TolerantReader"; Kleppmann, ch. 4 (encoding and evolution); JSON Schema spec.

| ID | Axis | Poles |
|---|---|---|
| I33 | Domain type vs wire type | **Serde derive on the domain type** · **Separate DTO and mapper** |
| I34 | Unknown fields | **Ignore (tolerant reader)** · **`deny_unknown_fields`** |
| I35 | Missing fields | **`#[serde(default = "fn")]`** · **`Option`** · **Required** |
| I36 | Schema authority | **JSON Schema file as the authority** · **Rust type as the authority** · **No schema** |
| I37 | Enum encoding | **Externally tagged** · **Internally tagged** · **String repr** |

## F18 Data-driven content and templating (added)

Source: Nystrom, *Game Programming Patterns* ("Data locality", "Type object", "Bytecode"); Hunt and Thomas, "Metaprogramming" (configure, don't integrate); Askama and Handlebars docs.

| ID | Axis | Poles |
|---|---|---|
| I38 | Where authored content lives | **JSON files under `data/`** · **Compiled-in constants** · **Database rows** |
| I39 | Content loading | **Seed into the database at start** · **Read the file at each use** · **Embed at compile time** |
| I40 | Author text substitution | **Placeholder replace in a domain module** · **Template engine** · **`format!`** |
| I41 | Behaviour in data | **Declarative (rules as data)** · **Scripted (embedded language)** · **Hard-coded** |
| I42 | Validation of content | **Validate files before run (script/schema)** · **Validate at load** · **Trust** |

## F13 Testing schools and test design

Source: Fowler, "Mocks Aren't Stubs" and "TestPyramid"; Freeman and Pryce, *Growing Object-Oriented Software, Guided by Tests* (GOOS); Beck, *Test-Driven Development by Example*; Meszaros, *xUnit Test Patterns*; Khorikov, *Unit Testing Principles*; MacIver, "Hypothesis"; Dodds, "Testing Trophy"; Feathers, "seam" (*Working Effectively with Legacy Code*); Metz, "Magic Tricks of Testing".

| ID | Axis | Poles |
|---|---|---|
| T1 | Test school | **Classicist / Detroit (real collaborators, state check)** · **Mockist / London (mock collaborators, interaction check)** |
| T2 | Test double kind | **Fake (working lightweight)** · **Stub** · **Mock (verifies calls)** · **Spy** · **Real dependency** |
| T3 | Double strictness | **Strict scripted fake (fails on unexpected call)** · **Lenient fake** |
| T4 | Test size mix | **Test pyramid** · **Testing trophy (integration-heavy)** · **Ice-cream cone** · **Honeycomb** |
| T5 | What a test asserts | **Observable behaviour** · **Implementation (calls, internals)** · **Snapshot (`insta`)** |
| T6 | Input choice | **Example-based** · **Property-based (`proptest`)** · **Golden file / snapshot** · **Fuzzing** |
| T7 | Unit location (Rust) | **Inline `#[cfg(test)] mod tests`** · **Sibling `*_tests.rs` file** · **`tests/` crate (black box)** |
| T8 | Test entry point | **Through the HTTP surface** · **Through the application service** · **Through the domain function** · **Through the browser (E2E)** |
| T9 | Test data | **Fixtures / builders** · **Object mother** · **Inline literals** · **Production data files** |
| T10 | Test process | **TDD red-green-refactor** · **Test after** · **Characterisation test (pin old behaviour)** |
| T11 | Seam for tests | **Port with a fake adapter** · **Feature flag in production code** · **Monkey-patch / global override** · **Test-only constructor** |
| T12 | Shared state in tests | **Fresh state per test** · **Shared fixture with a lock (serialised)** · **Process-wide env mutation with a guard** |
| T13 | Waiting in async tests | **Wait on a condition (poll with timeout)** · **Fixed sleep** · **Hold/release gate in the fake** |
| T14 | Spec link | **Feature spec with scenario → test trace** · **Test names only** · **Comments** |
| T15 | Failure of a test | **Fix the cause; never weaken the test** · **Mark ignored / flaky retry** · **Delete** |
| T16 | Coverage | **Coverage as a metric with a floor** · **Coverage as a report** · **None** |
| T17 | Test of structure | **Architecture test (parse source with `syn`/`arch-lint`)** · **Review** |
| T18 | Browser test | **Real browser (Playwright) against stub back end** · **HTTP-only DOM check** · **Manual** |

## F14 Documentation design

Source: Procida, *Diátaxis* (diataxis.fr); Parnas and Clements, "A rational design process: how and why to fake it"; Nygard, "Documenting architecture decisions" (ADR); Brown, "C4 model"; Knuth, "Literate programming"; Rustdoc book; Hunt and Thomas, "DRY" (single source); Evans, "Ubiquitous language" (glossary).

| ID | Axis | Poles |
|---|---|---|
| D1 | Doc organisation | **Diátaxis (tutorial, how-to, reference, explanation)** · **Topic tree (README per folder)** · **Wiki** · **Single README** |
| D2 | Doc modes kept apart | **One mode per page** · **Mixed pages** |
| D3 | Source of doc content | **Generated from code (index, route table)** · **Hand-written** · **Rustdoc comments** · **Literate** |
| D4 | Keeping docs true | **Checker script (links, anchors, paths)** · **Review habit** · **Drift** |
| D5 | Decision record | **ADR (one file per decision)** · **Explanation page** · **Commit message** · **Issue tracker** |
| D6 | Domain vocabulary | **Glossary file as single source (ubiquitous language)** · **Terms defined where used** |
| D7 | Structure map | **Generated module map** · **Hand-drawn diagram (C4)** · **Tree listing** |
| D8 | Agent-facing docs | **Root file with a pointer index (`AGENTS.md`)** · **One long file** · **Per-folder files** |
| D9 | Plans and history | **Plans in the repo (`plans/`, `.scratch/`)** · **In an issue tracker** · **Deleted when done** |
| D10 | Old docs | **Delete (git keeps history)** · **Archive folder (`old-docs/`)** · **Keep in place with a banner** |
| D11 | Prose style | **Controlled language (STE)** · **Plain prose** · **Marketing prose** |

## F23 Naming and code comments (added)

Source: Ousterhout, ch. 12–13 (comments describe what is not obvious); Martin, *Clean Code* ch. 2, 4; Evans (names from the domain); Rust API Guidelines (naming, C-CASE, C-CONV).

| ID | Axis | Poles |
|---|---|---|
| D12 | Comment content | **Why-comment (intent, constraint)** · **What-comment (restates code)** · **No comments (self-documenting)** · **Module summary line on each file** |
| D13 | Naming source | **Domain term from the glossary** · **Technical term** · **Pattern suffix (`Manager`, `Helper`, `Service`)** |
| D14 | Doc comments | **Rustdoc on public API** · **Module header only** · **None** |
| D15 | Banned words | **Avoid-list in the glossary** · **No list** |

## F16 Background work and long-running operations (added)

Source: Hohpe and Woolf, *Enterprise Integration Patterns* ("Message Queue", "Competing Consumers"); Fowler, "Asynchronous request-reply"; Nygard, "Release It!" (timeouts); Tokio tutorial ("Spawning").

| ID | Axis | Poles |
|---|---|---|
| P65 | Work that outlives the request | **In-process background task** · **Job queue with a worker** · **External scheduler** · **Block until done** |
| P66 | Busy rule for one entity | **Reject a second request** · **Queue it** · **Cancel the first** · **Run both** |
| P67 | Restart of an interrupted job | **Mark failed and keep partial state** · **Resume** · **Roll back** |
| P68 | Run of many jobs (batch, "autoplay") | **Loop that submits ordinary jobs** · **Dedicated batch path** · **Scheduler** |
| P69 | Timeouts | **Per-call timeout** · **Job deadline** · **None** |

## F21 Resource limits and performance (added)

Source: Knuth, "premature optimization"; Ousterhout, ch. 20 (design for performance); Gregg, *Systems Performance*; Nygard ("bulkhead", "timeouts").

| ID | Axis | Poles |
|---|---|---|
| I43 | Optimisation rule | **Measure first** · **Design for the fast path** · **Ignore until slow** |
| I44 | Budget | **Explicit budget type (token budget)** · **Ad hoc limit** · **None** |
| I45 | Load time | **Eager (at start)** · **Lazy (at first use)** · **Cached with invalidation** |
| I46 | Memory | **Bounded structures** · **Unbounded growth** |

## F24 Time, identity and randomness (added)

Source: Seemann, *Dependency Injection* (ambient context); Feathers, seams; Fowler, "TemporalObject"; UUID RFC 4122.

| ID | Axis | Poles |
|---|---|---|
| I47 | Time source | **Injected clock** · **`Utc::now()` at the call site** · **Time stored by the database** |
| I48 | Time format | **RFC 3339 string** · **Unix integer** · **`chrono` type in domain** |
| I49 | Id source | **Database assigns** · **Application assigns (UUID, counter)** · **Name derived from content/date** |
| I50 | Randomness | **Seeded / injected** · **Global RNG** |

---

# Count of axes per level

| Level | Count | IDs |
|---|---|---|
| Stance | 26 | S1–S26 |
| Pattern | 69 | P1–P69 |
| Idiom | 50 | I1–I50 |
| Test (idiom/pattern mix, `T`) | 18 | T1–T18 |
| Doc (`D`) | 15 | D1–D15 |

The `T` and `D` axes sit at the pattern and idiom levels. I gave them their own letters so the area tickets for `tests/` and `docs/diataxis/` can cite them.

---

# Axes most likely to show meeting points here (inferred)

I read little code. Each reason below comes from module names, `Cargo.toml`, `CONTEXT.md` and the repo's own rules in `AGENTS.md`. Treat each as a question for the area ticket.

| Rank | Axis | Reason (inferred) |
|---|---|---|
| 1 | **P1 / P8 / P16** transaction script vs domain model vs functional core | The tree has `application/pipeline/job.rs` (a script of steps), `domain/utils/game_rules.rs` (a pure function of state) and `domain/model/*` (data types). All three poles fit one use case. |
| 2 | **S12 / S13 / S14 / P23 / I15** writer rule: single writer vs `Arc<Mutex>` | `application/generation/gate.rs` holds a registry (runtime guard), `Arc<Mutex` has 10 hits, and `parking_lot` is a dependency. The structural and runtime poles can both apply. |
| 3 | **S2 / S4 / P56 / P39** ports and adapters vs layering | `adapters/driving`, `adapters/driven` and `application/ports` use hexagonal words. `domain/model/llm_backend.rs` and `adapters/.../mappers/` suggest layer-style mapping too. |
| 4 | **I13 / P22** enum dispatch vs `dyn Trait` | `dyn LlmProvider` has 19 hits, `dyn TextChecker` 6, and agents use `dyn EndOfJobAgent`. Other places may use enums. |
| 5 | **I1 / I2 / S10 / I6** error shapes | `thiserror` is the only error crate, but `src/error.rs`, `application/errors.rs` and `adapters/driving/http/error.rs` all exist. The "failure line" in `CONTEXT.md` adds an on-entity record. |
| 6 | **P40 / P46 / P47 / P42** hypermedia choices | 46 `hx-swap` hits and `axum::` WebSocket features, plus Askama. Polling, push and `hx-trigger` can serve the same need. |
| 7 | **T1 / T2 / T3 / T5 / T7 / T8** test schools | `tests/` has `http/`, `browser/`, `storage/`, `llm/`; `src/` has 87 `*_tests.rs` files; `ScriptedModel` is a strict fake; `insta` and `proptest` are used. Many school choices coexist. |
| 8 | **P27 / P28 / P31 / S16** storage shape | `rusqlite` has 116 hits. `storage/transaction.rs`, `games.rs`, `stories.rs` and `docks.rs` show both a facade and per-aggregate files. |
| 9 | **S18 / S19 / D10** migrate-and-delete vs archive | `old-docs/` and `plans/` exist beside the live tree. The map Notes mention a rebuild. `requires_migration` is a test folder. |
| 10 | **P59 / P54 / P55** model proposes, code decides | The Quantifier result, `agents/*/parser.rs` and Game rules sit in one flow. Free-text parsing and structured output are both possible. |
| 11 | **I33 / I36 / I12** wire type vs domain type | `serde_json::Value` has 11 hits. The `data/schemas` folder shows a schema file as authority. `domain/model` types may carry serde derive. |
| 12 | **D1 / D3 / D4** docs: Diátaxis vs generated indexes | `docs/diataxis/` has only `explanation` and `reference`. Generated index blocks sit in `AGENTS.md`. The ticket scope names Diátaxis. The mode set looks partial. |
| 13 | **S22 / T17** enforcement: executable check vs written rule | `arch-lint.toml`, `scripts/check_*`, `tests/AGENTS.md` and `CODING_STANDARDS.md` all state rules. |
| 14 | **I29 / P23** `spawn_blocking` around SQLite and `reqwest::blocking` | `reqwest` has the `blocking` feature; the tokio runtime is `full`. Both sync and async calls fit. |

---

# Check against the pstack principles

I used the pstack principles last, as a check. The "code shape" and "in between" items of `assets/pstack-principles.md` map as below. A gap means the lens had no axis for it before the check. I filled each gap.

| pstack principle | Axis in this lens | Gap? |
|---|---|---|
| boundary-discipline | S6, S7, P16 | No |
| model-the-domain | P8, P13, P14, I41 | No |
| type-system-discipline | S6, S7, I7–I10 | No |
| separate-before-serializing-shared-state | S12, S13 | No |
| make-operations-idempotent | S11, P67 | No |
| migrate-callers-then-delete-legacy-apis | S18 | No |
| minimize-reader-load | P18, P33, P38 | No |
| test-behavior-not-implementation | T1, T5 | No |
| encode-lessons-in-structure | S22, T17 | No |
| foundational-thinking | P13, I41 (data structure before code) | Weak: axis "data structure first vs algorithm first" is only implied. Added as note below. |
| laziness-protocol | S20, S21 | No |
| subtract-before-you-add | S20, P38 | No |
| outcome-oriented-execution | — | Process. Left out. |

Note added by the check: **data-first vs behaviour-first** (Brooks, "show me your tables"; Torvalds on data structures) is an axis on the P8 / P13 line. It is a stance. I did not give it an ID because I could not name two distinct poles in code form. Area tickets can add it if they see it.

---

# Open questions that I decided (for the delegating agent)

1. **Level of an axis that spans levels.** I placed each axis at one level only. Where a family spans levels (F5, F6, F7, F8), I split the family into a stance part and a pattern/idiom part.
2. **Tests and docs get their own letters (`T`, `D`).** The ticket says "cover the shapes of tests and of docs". It does not say how to count them by level. I did not force them into S/P/I.
3. **Pole count.** Several axes have four or five poles. Some poles are not live here. I kept them so each pole has a name. I did not trim to the live ones.
4. **Total axes.** About 178. This is more than an area ticket can walk. I expect each area ticket to use the "most likely" table and the family heads, not every row.
5. **Not-live families.** The ticket asks for one line per family that is not live. None of the 14 given families is fully absent. I wrote lines for the added or partial cases in the "judged not live" table.
