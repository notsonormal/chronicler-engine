# Research 06: the concepts in the composition root and the cross-cutting code

Ticket: [06](../issues/06-research-composition-and-cross-cutting.md). Labels: **known** (the code shows it), **inferred** (the code suggests it, not tested), **guessed** (a weak signal).

## What was read and what was sampled

**Read in full (all non-test files in the scope):** `src/main.rs`, `src/lib.rs`, `src/error.rs`, `src/bootstrap/{mod,init_game,load,logging,run,wiring}.rs`, `src/utils/{mod,cli}.rs`. That is 11 files, 1,220 lines.

**Read as tests of the scope (the unit-test files of the scope):** the test names of `src/bootstrap/{load,run,wiring}_tests.rs`, `src/utils/cli_tests.rs` and `src/error_tests.rs`. The bodies of the tests that touch a claim below were read too (`src/bootstrap/run_tests.rs:284-297`, `src/bootstrap/load_tests.rs:16-29`, `src/bootstrap/wiring_tests.rs:144-211`). `tests/bootstrap/run_branches.rs` was sampled (first 60 lines): it shows how a test starts the real `run`.

**Sampled for the cross-cutting shapes (all layers).** Every sample is a search over `src/` with `rg`, then a read of the hit sites:

| Shape | Layers sampled | How |
|---|---|---|
| Lock handling | application, driven adapters, test support | every `.lock()` site (about 40) and every `into_inner` site |
| Logging | domain, application, driven, driving, bootstrap | counts of `tracing` macros per layer and level; read of the sites in `logging.rs`, `job.rs`, `driver.rs`, `client.rs`, `response.rs`, `fragment.rs` |
| Error types | domain, application, driven, driving, bootstrap | every error enum and `type ... Result`; counts of `EngineError::*` variants per layer; read of `application/errors.rs`, `http/error.rs`, `http/utils/error.rs`, `app_state.rs`, `generation_status.rs` |
| Settings and configuration reads | all | every `get_settings()` caller, every `env::var`, the `settings_defaults.rs` file, the `AppSettings` and `LlmProviderConfig` types |
| Ambient clock and id | all | every `Utc::now`, `SystemTime::now`, `Uuid::new_v4` site |
| Guardrails | whole crate | `arch-lint.toml`, `clippy.toml`, `tests/infrastructure/guardrails/layers.rs` (first 130 lines), `Cargo.toml` |
| Test seams in production code | whole crate | every `cfg(feature = "testing")` site; `src/test_support/{mod,context,env_guard,test_app_builder}.rs` (heads only) |

**Not read:** the bodies of the storage, LLM, HTTP handler and application files, except the sites named above. Their own tickets cover them. `docs/diataxis/reference/startup.md` was read to compare it with `bootstrap/`.

The researcher did not open any other research asset. The researcher read no list of concepts.

## 1. Concepts

Extent words: *everywhere* (no counter-example found in the scope), *mostly*, *in one place*, *split* (two shapes share the area).

### 1.1 Composition and startup

| # | Concept | Opposite | Evidence | Extent | Label |
|---|---|---|---|---|---|
| C1 | **Composition Root** (one place builds the object graph) | Service Locator; objects that build their own collaborators | `src/bootstrap/wiring.rs:98-141`, `src/bootstrap/wiring.rs:143-177`; guard `tests/infrastructure/guardrails/layers.rs:5-6` | mostly (boot builds a second small graph, MP-6) | known |
| C2 | **Pure (manual) Dependency Injection**: collaborators passed to `new`, no container | DI container; global registry; `static` singletons | `src/bootstrap/wiring.rs:105-120`, `src/application/settings_service.rs:15-23`, `src/application/games/active.rs:14-26` | everywhere in `application/` | known |
| C3 | **Humble Object / thin `main`**: the binary only calls the library | logic in `main` | `src/main.rs:13-18`, `src/bootstrap/run.rs:28-42` | everywhere | known |
| C4 | **Explicit phases in startup** (prepare data, prepare state, start server) | one long `main` that mixes the steps | `src/bootstrap/run.rs:34-40`, `src/bootstrap/run.rs:44`, `src/bootstrap/run.rs:130`, `src/bootstrap/run.rs:141` | in one place | known |
| C5 | **Sync `main`, runtime built by hand** (`Runtime::new`, `block_on`) | `#[tokio::main]`, async from the top | `src/bootstrap/run.rs:131-132`, `src/bootstrap/run.rs:143-148`, `src/main.rs:13` | in one place | known |
| C6 | **Sync core, async edge**: model calls are blocking `reqwest`; async is for HTTP; blocking work moves to `spawn_blocking` | async all the way down | `src/application/pipeline/job.rs:85`, `src/application/autoplay/driver.rs:54`, `src/adapters/driving/http/settings/handlers/settings.rs:272` | mostly | known |
| C7 | **Strategy by closed enum** (config enum picks the provider; `match` returns a trait object) | open registry / plugin lookup | `src/bootstrap/wiring.rs:35-42` | in one place | known |
| C8 | **Late binding of a collaborator** (a resolver closure runs on every call) | early binding (resolve once at boot) | `src/bootstrap/wiring.rs:44-54`, `src/application/llm_recorder.rs:11-14`, `src/application/llm_recorder.rs:39-41` | mostly for connections; not for Agents (MP-1) | known |
| C9 | **Function injection** (a `Fn` closure is the seam) | a trait object, or a concrete type | `src/application/llm_recorder.rs:11-17`, `src/application/connection_test_service.rs:18`, `src/bootstrap/wiring.rs:69-72` | split with the `dyn Trait` ports | known |
| C10 | **Fail fast at the boot edge** (check a reference before any write) | defer the error to first use | `src/bootstrap/wiring.rs:145-148`, `src/bootstrap/load.rs:66-70`, `src/bootstrap/run.rs:62-64` | split (MP-2, MP-4, MP-5) | known |
| C11 | **Skip and continue on bad seed input** (log a warning, go on) | abort the boot | `src/bootstrap/load.rs:103-112`, `src/bootstrap/load.rs:155-157`, `src/bootstrap/run.rs:54-60` | split (MP-2) | known |
| C12 | **Idempotent seeding, seed-if-absent**: after the first run the database outranks the files | sync the files into the database on every boot | `src/bootstrap/load.rs:116-126`, `src/bootstrap/load.rs:150-153`, `src/adapters/driven/storage/settings.rs:118-122`; doc `docs/diataxis/reference/startup.md` (section "Settings") | mostly; presets also refresh an empty row (`src/bootstrap/run.rs:235-248`) | known |
| C13 | **Convention over configuration** for paths: the data dir and the DB sit next to the executable | an explicit config path or flag | `src/utils/cli.rs:42-53`, `src/bootstrap/run.rs:46-51` | everywhere in the scope | known |
| C14 | **Declarative CLI** (clap derive struct with defaults) | hand-parsed `argv` | `src/utils/cli.rs:17-36` | everywhere | known |
| C15 | **Convention-based discovery** (scan `data/worlds/*/world.json`) | an explicit manifest or registry | `src/utils/cli.rs:55-87`, `src/bootstrap/load.rs:77-112` | in two places (MP-19) | known |
| C16 | **Data-driven content**: Worlds, Personas, presets and settings are JSON files read through generic `read_json_file<T>` | content in code | `src/bootstrap/load.rs:12-26`, `src/bootstrap/load.rs:64`, `src/bootstrap/load.rs:179` | mostly (presets differ, MP-3) | known |
| C17 | **Facade by `mod.rs`** (`bootstrap` shows `init_logging` and `run`, hides the rest) and visibility as the boundary (`pub(crate)`, `pub(super)`) | every item `pub` | `src/bootstrap/mod.rs:5-12`, `src/bootstrap/run.rs:97`, `src/bootstrap/init_game.rs:8`, `src/application/games/active.rs:35-39` | mostly (`init_game` and `wiring` stay `pub mod`) | known |

### 1.2 Configuration and settings

| # | Concept | Opposite | Evidence | Extent | Label |
|---|---|---|---|---|---|
| C18 | **Settings as one stored singleton row**, edited as a whole aggregate | many small config records; config files read at run time | `src/adapters/driven/storage/settings.rs:92-125`, `src/application/settings_service.rs:25-34` | everywhere | known |
| C19 | **Read-modify-write inside one closure** (`update_settings(f)`) | get, edit, then `save` (lost update) | `src/adapters/driven/storage/settings.rs:106-117`, `src/application/settings_service.rs:43-48` | mostly (a whole-row `save_settings` still exists, MP-23) | known |
| C20 | **Whole-aggregate read**: a caller that needs one field gets a clone of all of `AppSettings` | a narrow query per field | `src/application/games/catalogue.rs:149-150`, `src/application/prompt_preset_service.rs:122-125`, `src/application/pipeline/job.rs:241` | everywhere (10 production call sites) | inferred |
| C21 | **Layered configuration sources**: built-in default, JSON seed, database row, environment fallback for the API key | one source | `src/domain/model/utils/settings_defaults.rs:1-20`, `src/bootstrap/load.rs:49-75`, `src/domain/model/settings.rs:317-331` | everywhere | known |
| C22 | **Secrets from the environment** (`.env` through `dotenv`, then `env::var` by provider name) | secrets in the stored settings | `src/main.rs:14`, `src/domain/model/settings.rs:317-319`, `src/domain/model/settings.rs:333-335` | mostly (the stored `api_key` field can also hold one) | known |
| C23 | **Parameterise the effect** (`..._with(lookup)` makes the key rule a pure function of its inputs) | read the environment inside the rule | `src/domain/model/settings.rs:323-331`, `src/domain/model/settings.rs:338-340` | in one place | known |
| C24 | **Serde default-function pointers** for forward-compatible stored shapes | `deny_unknown_fields` and strict shapes | `src/domain/model/utils/settings_defaults.rs:1-2`, `src/domain/model/settings.rs:395-400`; no `deny_unknown_fields` in `src/` | everywhere | known |
| C25 | **Named constants for tuning values** (budgets, retention, window sizes) | per-connection settings or a config file | `src/application/prompting/token_budget.rs:4-12`, `src/application/games/view_query.rs:22`, `src/adapters/driven/llm/transport/utils/client.rs:25` | split with the settings fields (MP-18) | known |

### 1.3 Errors

| # | Concept | Opposite | Evidence | Extent | Label |
|---|---|---|---|---|---|
| C26 | **Typed error enums with `thiserror`** | `anyhow`, `Box<dyn Error>`, `String` | `src/error.rs:6-7`, `src/error.rs:40-41`, `src/error.rs:112-113` | mostly (five HTTP helpers return `Result<_, String>`, MP-9) | known |
| C27 | **One crate-wide error enum** (`EngineError`, 20 variants) that every layer imports | an error type per module or layer | `src/error.rs:112-205`, `src/domain/model/utils/game_rules.rs:14`, `src/adapters/driven/storage/settings.rs:92` | everywhere in domain, driven and bootstrap; application wraps it (C28) | known |
| C28 | **Error translation at a layer boundary** (wrap the inner error in an outer envelope; turn it into a response at the edge) | let the inner error type leak out | `src/application/errors.rs:6-11`, `src/application/errors.rs:66-70`, `src/adapters/driving/http/error.rs:13-28` | mostly | known |
| C29 | **Error hierarchy by `From` conversion** (`LlmFailure` -> `EngineError` -> `ApplicationError`) | one flat enum | `src/error.rs:76-80`, `src/error.rs:106-110`, `src/application/errors.rs:66-70` | everywhere | known |
| C30 | **Context-adding extension trait** for a foreign error (no blanket `From<rusqlite::Error>`) | blanket `#[from]` | `src/error.rs:207-217`, `src/error.rs:200-204` | everywhere in driven storage | known |
| C31 | **Error taxonomy into a closed enum of kinds** (`GenerationFailureKind`) with a fixed short line and a raw-text disclosure | show the raw error | `src/domain/model/state/generation_status.rs:20-45`, `src/domain/model/state/generation_status.rs:125-137`, `src/domain/model/state/generation_status.rs:204-221` | in one place (Job failures) | known |
| C32 | **Hide internals from the user** (generic "Internal Server Error" unless the error is user-displayable) | echo `err.to_string()` to the client | `src/adapters/driving/http/error.rs:15-24`, `src/application/errors.rs:32-39` | mostly (the Job failure line is a separate path) | known |
| C33 | **Refusal as a value** (`Refusal`, `PlanRefusal`, `RunRefusal`: expected "no" answers are `Err` values with their own HTTP mapping) | exceptions or `Option` | `src/application/generation/gate.rs:26-36`, `src/domain/model/generation.rs:214-220`, `src/adapters/driving/http/error.rs:30-43` | in the Job path | known |
| C34 | **No panics in production code**, enforced by lint (`unwrap_used`, `expect_used`, `panic`, `todo`) | `unwrap` and `expect` for "cannot happen" | `src/lib.rs:4-23`, `src/lib.rs:25-35` | everywhere, with named exceptions (MP-12 lists them) | known |
| C35 | **Crash on a programmer error at init** (`expect` on a constant regex or a static response) | return `Result` for it | `src/application/agents/options/utils/parser.rs:7-10`, `src/application/prompting/sanitize.rs:8-17`, `src/adapters/driving/http/utils/response.rs:6-12` | in three files, plus `core.rs:122-124` | known |
| C36 | **Crate-root `Result` alias** (the `io::Result` idiom) | spell `Result<T, EngineError>` each time | `src/error.rs:269`, `src/lib.rs:44` | split: spelled both ways (SN-5) | known |
| C37 | **Degrade to a default and log** (a failed Agent gives the empty result) | propagate the failure | `src/application/agents/runner.rs:69-79`, `src/adapters/driving/http/app_state.rs:85-91` | in a few places (MP-5) | known |

### 1.4 Logging

| # | Concept | Opposite | Evidence | Extent | Label |
|---|---|---|---|---|---|
| C38 | **Logging behind a facade** (`tracing`); no print in library code | `println!`, or the `log` crate | `src/lib.rs:15-18`, `src/bootstrap/logging.rs:11-69`; prints only in `src/utils/cli.rs:93-97` and `src/bootstrap/logging.rs:28-44` | everywhere | known |
| C39 | **Level by severity**, with `warn` for a degraded path and `error` at a failed edge | one level | `src/bootstrap/load.rs:106-109`, `src/application/agents/runner.rs:75-78`, `src/adapters/driving/http/utils/fragment.rs:20` | mostly | known |
| C40 | **Environment-driven filter** (`EnvFilter` from `RUST_LOG`, default `info`) | a hard-coded level | `src/bootstrap/logging.rs:15-16`, `src/bootstrap/logging.rs:54` | mostly (the fallback path hard-codes `debug`, `logging.rs:46`) | known |
| C41 | **Non-blocking file appender, daily rotation, RAII flush guard held by `main`** | synchronous stderr | `src/bootstrap/logging.rs:35-39`, `src/bootstrap/logging.rs:53`, `src/main.rs:15` | in one place | known |
| C42 | **Span with fields as the context** (`#[instrument]`, `Span::current()`) | a string tag in each message | `src/application/pipeline/job.rs:63-70`, `src/application/pipeline/job.rs:83-87` | in one file (MP-16) | known |
| C43 | **Manual correlation id in the message text** (`[LLM][req:N]`) | a span field | `src/adapters/driven/llm/transport/utils/request.rs:8-12`, `src/adapters/driven/llm/transport/utils/client.rs:21`, `src/adapters/driven/llm/transport/utils/response.rs:51` | in the LLM adapter | known |
| C44 | **Log at the point of failure and also return the error** | log only at the edge that handles it | `src/adapters/driven/llm/transport/utils/response.rs:11-16`, `src/adapters/driving/http/utils/fragment.rs:19-22` | split (MP-15) | inferred |

### 1.5 Locks, time, globals

| # | Concept | Opposite | Evidence | Extent | Label |
|---|---|---|---|---|---|
| C45 | **Ignore lock poisoning** (`unwrap_or_else(\|e\| e.into_inner())`): a panic in one holder does not stop the next | propagate poison with `unwrap` or `expect` | `src/application/games/active.rs:28`, `src/application/generation/gate.rs:152-155`, `src/adapters/driven/storage/db.rs:27-32` | everywhere in production (17 sites, plus 5 in the storage test-failure file; three spellings) | known |
| C46 | **Coarse lock around one resource** (std `Mutex`) | fine-grained locks, lock-free types, an actor | `src/adapters/driven/storage/db.rs:12-14`, `src/adapters/driven/storage/core.rs:17`, `src/application/generation/gate.rs:128-133` | everywhere | known |
| C47 | **Execute-around: the closure runs under the lock** | give the guard to the caller | `src/application/generation/gate.rs:145-158`, `src/adapters/driven/storage/core.rs:205-232`, `src/adapters/driven/storage/settings.rs:106-117` | mostly (the connection guard is returned, MP-13) | known |
| C48 | **Documented lock order** (guard, then backend, then connection) | no order | `src/application/generation/gate.rs:145-147` | in one place | known |
| C49 | **Ambient clock and ids** (`Utc::now`, `Uuid::new_v4` called in place) | an injected `Clock` or id source | `src/adapters/driven/storage/games.rs:40`, `src/adapters/driving/http/utils/handler_helpers.rs:19-27`, `src/application/llm_recorder.rs:112` | everywhere (10 sites in storage, 6 in domain, application and driving) | known |
| C50 | **Process-global counter** (`static AtomicU64`) for a request id | a counter on a collaborator | `src/adapters/driven/llm/transport/utils/request.rs:8-12` | in one place | known |
| C51 | **Lazy-initialised static** for compiled regexes | compile at each use | `src/application/agents/options/utils/parser.rs:15-21`, `src/application/prompting/sanitize.rs:9-18` | in two files | known |

### 1.6 Guardrails and tests of the shell

| # | Concept | Opposite | Evidence | Extent | Label |
|---|---|---|---|---|---|
| C52 | **Architecture fitness functions**: dependency rules as an executable test (`arch-lint`, guardrail walkers) | review by convention | `arch-lint.toml:88-94`, `arch-lint.toml:162-166`, `tests/infrastructure/guardrails/layers.rs:5-6`, `tests/infrastructure/architecture.rs:3` | whole crate | known |
| C53 | **Lints as architecture** (`#![deny(...)]` with a reason comment per lint) | style guide in prose | `src/lib.rs:4-23`, `clippy.toml:6-17` | whole crate | known |
| C54 | **Escape hatch that names its reason** (`arch-lint: allow(...) reason="..."`) | silent `#[allow]` | `src/adapters/driven/storage/core.rs:122-124`, `src/test_support/context.rs:52`, `src/test_support/scripted_model.rs:125-126` | mostly (24 comment sites; plain `#[allow(clippy::expect_used)]` has no reason line, `response.rs:6`) | known |
| C55 | **Unit tests in a sibling `*_tests.rs` file**, registered in `mod.rs` under `#[cfg(test)]` | inline `mod tests` | `src/bootstrap/mod.rs:13-18`, `src/lib.rs:54-55`, `src/utils/mod.rs:6-7` | everywhere in the scope | known |
| C56 | **Test support shipped in the library behind a feature** (`testing`, on by default) | `#[cfg(test)]` only, or a separate crate | `Cargo.toml:54-57`, `src/lib.rs:46-52`, `src/bootstrap/wiring.rs:179-220` | whole crate (MP-21) | known |
| C57 | **Real fake over mock framework**: in-memory SQLite, `MockBackend`, `ScriptedModel` | mock objects with expectations | `src/bootstrap/load_tests.rs:6`, `src/bootstrap/wiring.rs:189-193`, `src/test_support/scripted_model.rs:125-132` | mostly | inferred |
| C58 | **Serialise a global with a static lock in tests** (`ApiKeyEnvGuard`) | inject the environment | `src/test_support/env_guard.rs:5-23`, `src/bootstrap/wiring_tests.rs:177-183` | in one place (MP-17) | known |
| C59 | **Test Data Builder** (`TestAppBuilder`, `TestDataBuilder`) | Object Mother or inline setup | `src/test_support/test_app_builder.rs:25-50`, `src/lib.rs:46-49` | sampled only | inferred |
| C60 | **Traceability header** (`[DOC: path]` in a file's first line; `[TRIVIAL_ENUM]` marker) | docs with no link to code | `src/error.rs:1`, `src/bootstrap/wiring.rs:1`, `src/domain/model/state/generation_status.rs:93` (a `[TRIVIAL_ENUM]` marker) | 210 of 307 `.rs` files | known |

## 2. Meeting points

Each one is a question. The evidence is the whole case. No point is ruled on.

**MP-1. Which settings are read at each use, and which are read once at boot?**
The narrator, Quantifier and Options connections come from the stored row on every call: the resolver runs `get_settings()` inside the closure (`src/bootstrap/wiring.rs:52-54`) and the comment says a dashboard change takes effect with no restart (`src/bootstrap/wiring.rs:44-45`). The Agent list is read once at boot (`src/bootstrap/wiring.rs:144`, `src/bootstrap/wiring.rs:161-166`). No production code outside the boot reads `settings.agents` (search of `src/`; `src/domain/model/settings.rs:397` is the field). Does an Agent change in the stored row need a restart, and does any text say so? (inferred)

**MP-2. In seeding, which bad file skips and which one stops?**
A bad `world.json` is skipped with a warning (`src/bootstrap/load.rs:103-112`). A bad NPC file is skipped (`src/bootstrap/load.rs:155-157`). A bad map file, a bad persona file and a bad preset file return `Err` with `?` (`src/bootstrap/load.rs:123`, `src/bootstrap/load.rs:179`, `src/bootstrap/run.rs:200-203`). The caller then logs a warning and goes on (`src/bootstrap/run.rs:54-60`). The doc says a malformed world, persona or preset file "is skipped without halting bootstrap" (`docs/diataxis/reference/startup.md`, section "Seeding Order"). A test asserts that `ensure_presets` returns `Err` on a bad preset file (`src/bootstrap/run_tests.rs:284-297`). When one preset file is bad, do the later preset files of that run seed? The `?` at `src/bootstrap/run.rs:179` suggests no. (inferred)

**MP-3. Why is a preset seed read as raw JSON, while a persona, an NPC and settings are read as typed values?**
The persona, NPC and settings seed files use `read_json_file::<T>` (`src/bootstrap/load.rs:64`, `src/bootstrap/load.rs:148`, `src/bootstrap/load.rs:179`). A preset is parsed into `serde_json::Value` and each field is pulled by hand, with silent defaults such as `"Default"` and `true` (`src/bootstrap/run.rs:200`, `src/bootstrap/run.rs:219-231`). `PromptPreset` derives `Deserialize` (`src/domain/model/prompt_preset.rs:63-64`). `docs/diataxis/reference/startup.md` lists four schema files and none for presets. Is the hand mapping a choice (for example, for the "refresh an empty row" rule at `src/bootstrap/run.rs:235-248`), or a leftover? (guessed)

**MP-4. Fail fast or defer: how does the boot treat a connection that cannot work?**
A connection id that points to nothing stops the boot (`src/bootstrap/wiring.rs:145-148`, `src/bootstrap/load.rs:66-70`). A missing API key is checked when the resolver runs, not at boot (`src/bootstrap/wiring.rs:55`). A DeepSeek connection passes the key check and fails on each call (`src/domain/model/settings.rs:344`, `src/adapters/driven/llm/providers/deepseek.rs:25-30`). A test names this "defers error" and keeps it (`src/bootstrap/wiring_tests.rs:144-146`). Where is the line between a dangling reference (boot stops) and an unusable connection (call fails), and who draws it? (known for the facts; the line is a question)

**MP-5. Why does a bad Agent config not stop the boot as a bad connection does?**
`AgentRunner::from_configs` returns `Err` for an unknown Agent type (`src/application/agents/runner.rs:61-63`). The wiring turns every such `Err` into the default runner with no Agents, and logs nothing (`src/bootstrap/wiring.rs:161-166`, `src/bootstrap/wiring.rs:199-201`). A runner with no Agent gives the empty Quantifier result (`src/application/agents/runner.rs:69-74`), so a Job runs with no Character present and no destination. The same function checks the two connection ids and stops the boot (`src/bootstrap/wiring.rs:147-148`). Is the silent default on purpose? (inferred)

**MP-6. Does anything rely on two copies of the Game wiring at boot?**
The boot builds its own `GameWriter`, `ActiveGame` and `GameCatalogue` to pick the Game (`src/bootstrap/run.rs:102-107`). The wired graph builds a second set (`src/bootstrap/wiring.rs:105`, `src/bootstrap/wiring.rs:113-119`). The comment on the first says the graph does not exist yet (`src/bootstrap/run.rs:95-96`). `GameWriter` is "the one registry of job entries and run holds" (`src/application/generation/gate.rs:128`). The boot copy is dropped, so its registry is empty and unused. Is there a rule that says one registry per process, and does the boot copy fit it? (inferred)

**MP-7. Is the test graph the production graph with a swapped port, or a second assembly?**
Production: `build_app_graph` builds three recorders from stored settings (`src/bootstrap/wiring.rs:150-160`). Test: `build_app_graph_for_tests` builds one recorder over a fixed `MockBackend` (`src/bootstrap/wiring.rs:189-193`) and then still builds the Options recorder from stored settings (`src/bootstrap/wiring.rs:198`). Both end in `build_wired_app` (`src/bootstrap/wiring.rs:170`, `src/bootstrap/wiring.rs:203`). In a test graph with no override, which provider answers an Options run? Which one answers after a test changes the settings row? (inferred)

**MP-8. `WiredApp` and `AppState`: a bag of services, or collaborators passed by need?**
`WiredApp` is a public struct of 11 services (`src/bootstrap/wiring.rs:84-96`). `AppState` copies it field by field (`src/adapters/driving/http/app_state.rs:44-57`). Handlers take `State<AppState>` and reach any service (`src/adapters/driving/http/settings/handlers/settings.rs:24`, `src/adapters/driving/http/utils/current_game.rs:23-32`). The guardrail message says to pass collaborators as `Arc<...>` parameters, and it limits where `WiredApp` may be named (`tests/infrastructure/guardrails/layers.rs:5-6`, `tests/infrastructure/guardrails/layers.rs:41`). The limit covers `WiredApp`. It does not cover `AppState`. Which of the two shapes (Service Locator through `AppState`, or per-handler injection) does the HTTP layer follow? (inferred)

**MP-9. How many error types does one failed request cross, and who maps between them?**
Types: `EngineError` (`src/error.rs:112`), `ApplicationError` (`src/application/errors.rs:6`), `Refusal` (`src/application/generation/gate.rs:27`), `JobFailure` (`src/application/generation/gate.rs:40`), `RunRefusal` (`src/application/generation/gate.rs:210`), `PlanRefusal` (`src/domain/model/generation.rs:215`), `GenerationFailure` (`src/domain/model/state/generation_status.rs:126`), and plain `String` in HTTP helpers (`src/adapters/driving/http/settings/handlers/settings.rs:100`, `src/adapters/driving/http/worlds/handlers/worlds.rs:42`, `src/adapters/driving/http/utils/error.rs:41`). Mappers: `from_engine_error` (`src/application/errors.rs:25-30`), `boot_error` (`src/bootstrap/run.rs:123-128`), `GenerationFailure::from_engine_error` (`src/domain/model/state/generation_status.rs:204`), two `IntoResponse` impls (`src/adapters/driving/http/error.rs:13`, `src/adapters/driving/http/error.rs:30`). `ApplicationError` has two ways in: `From` keeps every engine error on `Engine`, `from_engine_error` lifts `Validation` (`src/application/errors.rs:22-30`). Is there one map of these types and their conversions? Which conversion does a `?` pick in each file?

**MP-10. What does `EngineError::Config` mean?**
Its doc line says "Settings/config value missing or invalid" (`src/error.rs:171-173`). It is also the variant for a failed view read in `AppState` (`src/adapters/driving/http/app_state.rs:95`, `src/adapters/driving/http/app_state.rs:103`, `src/adapters/driving/http/app_state.rs:139`), a bind failure (`src/adapters/driving/http/bootstrap/server.rs:41-46`), a missing connection id in a handler (`src/adapters/driving/http/settings/handlers/settings.rs:319`), and a missing Quantifier text (`src/application/agents/quantifier/utils/orchestration.rs:23`). The HTTP helper treats `Config` like `Validation` and drops the layer prefix (`src/adapters/driving/http/utils/error.rs:53-61`). Counts of `Config` per layer: domain 4, application 3, driven 2, driving 13, bootstrap 2 (search of `src/`). Is `Config` the name of a cause or a catch-all for "the request cannot continue"?

**MP-11. When does a foreign error keep its type, and when does it become a `String`?**
`Serde` keeps `serde_json::Error` through `#[from]` (`src/error.rs:119-120`). `Database` keeps `rusqlite::Error` and adds a context string, with no `From` (`src/error.rs:200-217`). `Io` flattens `io::Error` to `String` (`src/error.rs:115-116`, `src/error.rs:219-223`). `Parse`, `Config`, `Template` are `String` (`src/error.rs:122-124`, `src/error.rs:172-173`, `src/error.rs:179-181`). `DataLoad` boxes the inner `EngineError` with its path (`src/error.rs:187-192`). Is there a rule for which foreign errors keep a `source`? Does a `std::error::Error::source()` chain reach the log for each variant?

**MP-12. Lock poison: an ignored risk or a decided policy? And where do panics stay?**
Every production `lock()` ignores poison, in three spellings: `unwrap_or_else(|e| e.into_inner())` (`src/application/games/active.rs:28`), `unwrap_or_else(|poisoned| poisoned.into_inner())` (`src/application/generation/gate.rs:152-155`), and `match ... Err(poisoned) => poisoned.into_inner()` (`src/adapters/driven/storage/db.rs:27-32`, `src/adapters/driven/storage/core.rs:145-148`). The test fakes use `expect` and name the reason (`src/test_support/scripted_model.rs:125-126`). `parking_lot` (no poison) is a dependency (`Cargo.toml:28`) and no file in `src/` or `tests/` uses it (search). The lint bans `expect_used` (`src/lib.rs:7-8`), and the code keeps `expect` in four places (C35 and `src/adapters/driven/storage/core.rs:122-124`). Is `parking_lot` meant to replace `std::sync::Mutex`? After a panic while a holder runs a half-finished write, which state does the next holder see? (inferred for the second question)

**MP-13. Two locks in series: why a backend `Mutex` and a connection `Mutex`?**
A storage call locks the backend mutex (`src/adapters/driven/storage/core.rs:213-216`, field at `src/adapters/driven/storage/core.rs:17`). Inside the closure the code calls `pool.conn()`, which locks the connection mutex and returns the guard (`src/adapters/driven/storage/worlds.rs:72`, `src/adapters/driven/storage/db.rs:27-32`). The lock order is written for the guard and the two (`src/application/generation/gate.rs:145-147`). `DbPool::conn()` is also called from test fixtures (`src/test_support/fixtures.rs:417`). What does the second lock protect that the first does not? Can code call `conn()` outside a backend closure? (inferred; the driven-adapter asset may hold more)

**MP-14. Does the pure model log and read the environment?**
The architecture rule says "Model layer must be pure" (`arch-lint.toml:88-94`). The model logs `warn` on a parse fallback (`src/domain/model/settings.rs:56`, `src/domain/model/settings.rs:187`, `src/domain/model/settings.rs:225`) and `debug` in the movement walk (`src/domain/model/utils/game_rules.rs:125`). It reads `std::env::var` in `resolve_api_key` (`src/domain/model/settings.rs:318`), while `resolve_api_key_with` takes the lookup (`src/domain/model/settings.rs:323`). Is a log line an effect for the purpose of "pure"? Is the `_with` split a pattern or one case?

**MP-15. In which layer is an error logged, and how many times?**
The LLM adapter logs `error` at each parse failure and then returns an `EngineError` (`src/adapters/driven/llm/transport/utils/response.rs:11-16`, `src/adapters/driven/llm/transport/utils/response.rs:51`). The recorder logs `warn` when a failed-attempt save fails and returns the model error (`src/application/llm_recorder.rs:70-73`). The Job writes the failure line to the Game. The HTTP edge logs `error` again for a failed fragment (`src/adapters/driving/http/utils/fragment.rs:19-22`). Counts of `error!` per layer: domain 0, application 1, driven 12, driving 7, bootstrap 0 (search). Does one failed model call appear in the log once or more than once? (inferred)

**MP-16. What ties a log line to its Job?**
The Job span has `game`, `job`, `kind` and `step` fields (`src/application/pipeline/job.rs:63-70`) and moves into `spawn_blocking` (`src/application/pipeline/job.rs:83-87`). The Autoplay driver logs fields with no span (`src/application/autoplay/driver.rs:46`). The LLM adapter tags each message with `[LLM][req:N]` from a global counter (`src/adapters/driven/llm/transport/utils/request.rs:8-12`, `src/adapters/driven/llm/transport/utils/response.rs:51`). Only `job.rs` uses `#[instrument]` (search of `src/`). Does an LLM adapter line carry the Job id, through the span, when the line is written? (inferred: yes, if the span is entered on that thread)

**MP-17. Environment and globals: read in place, injected, or locked in tests?**
In place: the log mode (`src/bootstrap/logging.rs:14`), the exe path (`src/utils/cli.rs:43`, `src/bootstrap/run.rs:46`). Injected: the key lookup (`src/domain/model/settings.rs:323-331`). Locked: tests that set or remove a key take a static lock (`src/test_support/env_guard.rs:5-23`), and one test removes the variable inside `unsafe` (`src/bootstrap/wiring_tests.rs:181-183`). `CHRONICLER_LOG_CONSOLE` is set by the test harness (`tests/test_utils/server.rs:209`). Is the injected form the route for new reads, and is the `unsafe` removal covered by the lock for every test that reads the same variable? (inferred)

**MP-18. Two homes for one default.**
The default context size is `MAX_CONTEXT_TOKENS = 32768` (`src/application/prompting/token_budget.rs:4`, `src/application/prompting/token_budget.rs:23`). The per-connection default is a `match` of three literals (`src/domain/model/settings.rs:378-384`), and the Job passes the second to the prompt (`src/application/pipeline/job.rs:470`). The response default is `2048` twice: `MAX_RESPONSE_TOKENS` (`src/application/prompting/token_budget.rs:8`) and `DEFAULT_MAX_TOKENS` (`src/adapters/driven/llm/transport/utils/request.rs:14`). The transport timeout is a literal `180` (`src/adapters/driven/llm/transport/utils/client.rs:25`). Which value wins on each path? Are the two `2048` the same on purpose?

**MP-19. One reader of the World folder, or two?**
The CLI lists Worlds by scanning `data/worlds/*/world.json` and swallows parse errors with `.ok()?` (`src/utils/cli.rs:84-85`). The seeder scans the same folder and logs a warning for a bad manifest (`src/bootstrap/load.rs:82-112`). `utils/` holds the CLI reader but names a domain type (`src/utils/cli.rs:15`, `src/utils/mod.rs:2`). Should the list command and the seeder agree on which folder is a World?

**MP-20. Why does a missing World fall back, and a missing Persona stop?**
A World that is not found falls back to the first World in the list, with a warning (`src/bootstrap/run.rs:66-82`). A Persona that is not found stops the boot (`src/bootstrap/run.rs:84-86`). The CLI defaults name a content pair, `redmist_estate` and `julian` (`src/utils/cli.rs:22-26`). Which World is first is set by the storage order of `list_worlds` (`src/adapters/driven/storage/worlds.rs:69-81`), which this ticket did not trace. Is the fallback a convenience for a dev run, and is a content name in the engine's CLI on purpose? (guessed)

**MP-21. What does the production binary contain from the test layer?**
`testing` is a default feature (`Cargo.toml:54-57`). Under it the library exports `TestAppBuilder`, `TestDataBuilder` and `test_support` (`src/lib.rs:46-52`) and `build_app_graph_for_tests` (`src/bootstrap/wiring.rs:179-180`). The comment on `JobTasks` says it is "built into production because the `testing` feature is a default one" (`src/application/generation/job_tasks.rs:6-7`). Separately, `#[cfg(test)]` holds the `*_tests.rs` files (`src/lib.rs:54-55`). Does `python build.py` package with `--no-default-features`? (I did not read `build.py`; it is out of scope.)

**MP-22. Ambient clock and id: one source or many?**
`Utc::now()` is called in 10 storage sites (for example `src/adapters/driven/storage/games.rs:40`, `src/adapters/driven/storage/transaction.rs:70`) and in 6 domain, application and driving sites (`src/domain/model/message.rs:70`, `src/application/llm_recorder.rs:112`). The HTTP helper builds a storage id from the clock and a UUID (`src/adapters/driving/http/utils/handler_helpers.rs:19-27`). The doc comment there says the pair avoids a same-millisecond collision. `generate_storage_id` is in the driving adapter, and the id is a storage key. Which layer owns a storage key? (see also the domain asset on the ambient clock)

**MP-23. A whole-row `save_settings` exists. Who may call it?**
Production writes settings through `update_settings` (closure) and `seed_settings` (insert if absent) (`src/application/settings_service.rs:29-34`, `src/bootstrap/load.rs:72`). `save_settings` is public on `Storage` (`src/adapters/driven/storage/settings.rs:96-100`). Its only caller is the test builder (`src/test_support/test_app_builder.rs:193`, search of `src/`). Is the lost-update risk the reason for the closure form, and is the plain `save_settings` kept as test-only?

**MP-24. Is the forensics record part of a model call's success?**
When a call fails, a save error of the failure record is logged and the model error returns (`src/application/llm_recorder.rs:70-73`). When a call succeeds, a save error fails the call, though the text exists (`src/application/llm_recorder.rs:83`). Which of the two is the rule?

## 3. Shapes with no name

- **SN-1. The boot builds a graph twice, small then full.** `src/bootstrap/run.rs:102-107` and `src/bootstrap/wiring.rs:105-119` repeat three constructors with the same arguments.
- **SN-2. `WiredApp` is copied field by field into `AppState`.** `src/bootstrap/wiring.rs:84-96` and `src/adapters/driving/http/app_state.rs:31-57` hold the same ten service fields. The first adds `storage`. The rule that limits `WiredApp` stays in a test file (`tests/infrastructure/guardrails/layers.rs:5-6`).
- **SN-3. A `*_with(lookup)` twin for each ambient read.** `resolve_api_key` and `resolve_api_key_with`, `check_api_key_available` and `check_api_key_available_with` (`src/domain/model/settings.rs:317-340`). The twin takes the effect as an argument. The plain one passes `std::env::var`.
- **SN-4. An error variant used as a context wrapper.** `EngineError::Config(Self::render_error_context("header", e))` (`src/adapters/driving/http/app_state.rs:95`), with the name of the view inside the message string.
- **SN-5. Two `Result` spellings in one file.** `Result` from `crate::error` and `std::result::Result<_, ApplicationError>` side by side (`src/application/settings_service.rs:13`, `src/application/settings_service.rs:41`; `src/bootstrap/run.rs:28` and `src/bootstrap/run.rs:191`).
- **SN-6. Seeding by directory shape.** A folder `data/characters/<group>/` where `<group>` is `characters_dir` or the World id (`src/bootstrap/load.rs:128-133`). The seed-only pointers `map_file` and `characters_dir` are dropped when the manifest becomes a `WorldCard` (`src/bootstrap/load.rs:120-121`).
- **SN-7. Message-string tags as a logging type.** `[LLM][req:N]` (`src/adapters/driven/llm/transport/utils/response.rs:51`), `[LLM]` alone (`src/application/llm_recorder.rs:72`). The tag is a convention in text, with no type behind it.
- **SN-8. The failure line plus the raw text.** One short sentence for the status column and the raw error behind a disclosure, as a pair type (`src/domain/model/state/generation_status.rs:125-137`) and as an HTML shape (`src/adapters/driving/http/utils/error.rs:24-35`). The CONTEXT.md glossary calls it the *Failure line*.
- **SN-9. A guard message that names a function that does not exist.** `app_err_to_response()` appears in a guardrail message (`tests/infrastructure/guardrails/layers.rs:86`) and in no `src/` file (search). `ApplicationService` appears in `arch-lint.toml:162-166` and in no `src/` file. The messages describe an earlier shape of the code.
- **SN-10. A real DB file named by the port.** `chronicler_{port}.db` beside the executable (`src/bootstrap/run.rs:50`), with the tests deleting `-wal`, `-shm` and `-journal` files by that name (`tests/bootstrap/run_branches.rs:25-28`).
- **SN-11. Fall back, warn, go on, in the boot.** Seven sites follow the form "on `Err`, log a warning with `{e}` and use a default or skip" (`src/bootstrap/run.rs:54-60`, `src/bootstrap/run.rs:75-80`, `src/bootstrap/run.rs:115-116`, `src/bootstrap/load.rs:106`, `src/bootstrap/load.rs:156`, `src/bootstrap/run.rs:210-214`). Elsewhere in the boot the form is `?`.

## 4. Questions

1. **Doc against code on seeding.** `docs/diataxis/reference/startup.md` says a malformed world, persona or preset file is skipped (section "Seeding Order"). The code skips a world manifest and an NPC file, and returns `Err` for the others (MP-2). The same doc says "the loader calls `default_scenario()`" with a `"start"` fallback (section "Game-State Initialization"). That call is now in the domain, not in `bootstrap/` (`src/domain/model/utils/opening.rs:16`, `src/domain/model/world.rs:74-76`). Does the doc describe the earlier loader? (inferred)
2. **A log line that may name the wrong file.** The log message prints `chronicler_<date>.log` (`src/bootstrap/logging.rs:33`, `src/bootstrap/logging.rs:66`). The appender uses prefix `chronicler_<date>` with daily rotation (`src/bootstrap/logging.rs:36-37`), which adds its own date. Does the real file name equal the printed one? (inferred: no; not run)
3. **A flush guard for a writer nobody uses.** In the console path and the fallback path, `init_logging` returns `tracing_appender::non_blocking(std::io::stdout()).1` (`src/bootstrap/logging.rs:22`, `src/bootstrap/logging.rs:49`). The subscriber there writes to its own stdout writer. The returned guard belongs to a second, unused one. Is that guard needed? (inferred)
4. **The fallback logger ignores `RUST_LOG`.** `EnvFilter::new("debug")` replaces the env filter in the file-appender failure path (`src/bootstrap/logging.rs:46`), so a failed log directory switches on debug logs. Is that on purpose? (known for the fact)
5. **A dir that is a file.** `prepare_data` falls back to `data_dir` when `current_exe` fails (`src/bootstrap/run.rs:46-49`). `db_path.to_str().unwrap_or("chronicler.db")` (`src/bootstrap/run.rs:51`) swaps in a relative path when the path is not UTF-8. Both cases would put the database in a place the doc does not name. Do they happen? (guessed)
6. **`resolve_game_id` and `resolve_boot_game`.** Two functions resolve the boot Game, in two files, and one wraps the other (`src/bootstrap/init_game.rs:8-24`, `src/bootstrap/run.rs:97-119`). The error of the inner one is `ApplicationError`, mapped by `boot_error`. `init_game.rs` has one caller, `src/bootstrap/run.rs:108` (search of `src/` and `tests/`), and it is a `pub mod`. Is the module of its own for a reason, or a leftover?
7. **`pub mod wiring` and `pub mod init_game`, but `mod load`, `mod run`.** `src/bootstrap/mod.rs:5-12` shows `wiring` and `init_game` to the whole crate and tests, and hides the rest. `tests/http/support/app_wiring.rs:10` uses `build_app_graph`. No file outside `bootstrap/` names `init_game` (search of `src/` and `tests/`). Is `pub` needed?
8. **`unwrap_or_default()` on `Result`.** The wiring uses it on a `Result` (`src/bootstrap/wiring.rs:166`, `src/bootstrap/wiring.rs:200`). The same call on `Option` reads as "no value". On `Result` it drops the error (MP-5). 42 production sites call `unwrap_or_default` (search); I did not sort them into `Option` and `Result`.
9. **Concept check for the catalogue.** The `[DOC: ...]` header is in 210 of 307 files. Which files lack it, and does a rule list them? (`scripts/validate_docs.py` is out of scope.)
10. **`print` allowed in `bootstrap/mod.rs`.** The module attribute allows `print_stdout` and `print_stderr` for the whole `bootstrap` tree (`src/bootstrap/mod.rs:3`), and only `logging.rs` prints there (`src/bootstrap/logging.rs:28-44`). Is the wide allow needed?
