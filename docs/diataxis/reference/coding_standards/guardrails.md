---
diataxis: reference
title: Architecture Guardrails
---

## Overview

The Chronicler Engine enforces coding standards through three complementary layers: clippy lints at compile time, declarative `arch-lint` rules at test time, and custom `syn`-based convention walkers at test time. A coverage exclusion policy (`cargo-llvm-cov` with `--ignore-filename-regex`) keeps the wiring/lifecycle code from being unit-tested separately from its integration tests.

## 1. Clippy (Compile-Time)

Clippy lints are denied at the crate root in `src/lib.rs` via `#![deny(...)]`. Tests are exempt via `#![cfg_attr(test, allow(...))]`. `cognitive-complexity-threshold` and `too-many-arguments-threshold` thresholds live in `clippy.toml`.

<!-- AUTO-GUARDRAILS: clippy START -->
| Lint | Rationale |
|------|-----------|
| `clippy::unwrap_used` | Prevent panics in production; propagate errors with `?`. |
| `clippy::expect_used` | No "should never happen" assumptions in production. |
| `clippy::dbg_macro` | No debug prints in committed code. |
| `clippy::todo` | No unfinished code in production. |
| `clippy::unimplemented` | No stubs in production. |
| `clippy::print_stdout` | No `println!` in library code. |
| `clippy::print_stderr` | No `eprintln!` in library code. |
| `clippy::panic` | No explicit panics in production. |
| `clippy::let_underscore_must_use` | No silently dropped `#[must_use]` result; propagate or log it. |
<!-- AUTO-GUARDRAILS: clippy END -->

## 2. arch-lint (Test-Time)

Declarative architecture rules in `arch-lint.toml`.

<!-- AUTO-GUARDRAILS: arch-lint START -->
Scan scope: `root = "./src"`, excluding `**/target/**`, `**/tests/**`, `**/*_tests.rs`, `**/.pi/**`, `**/tmp/**`.

| From Scope | To Scope(s) | Rationale |
|------------|-------------|----------|
| `model` | `server, narrative, application` | Model layer must be pure; cannot depend on outer layers. |
| `model` | `storage` | Model layer must be pure; the game rules cannot depend on the storage adapter. |
| `application` | `server` | Application layer must not depend on server layer. |
| `agent-edge` | `storage` | Agents and the narrator get values; the step owns the storage reads and writes. |
| `model` | `storage-models` | Model layer must not depend on storage DB models/mappers. |
| `narrative` | `storage-models` | Narrative layer must not depend on storage DB models/mappers. |
| `application` | `storage-models` | Application layer must not depend on storage DB models/mappers. |
| `server` | `storage-models` | Server layer must not depend on storage DB models/mappers. |
| `bootstrap` | `storage-models` | Bootstrap layer must not depend on storage DB models/mappers. |
| `test-support` | `storage-models` | Test support must not depend on storage DB models/mappers. |
| `server` | `storage` | Server layer must not import from storage directly; use ApplicationService instead. |
| `ports` | `driven-llm, driven-text-check, narrative, server, storage, storage-models, bootstrap, test-support` | Port traits must not depend on adapter impls, application services, or outer layers. |
| `driven-llm` | `driven-text-check` | driven-llm adapter must not depend on driven-text-check adapter. |
| `driven-text-check` | `driven-llm` | driven-text-check adapter must not depend on driven-llm adapter. |
| `storage` | `application` | Storage adapter must not depend on the application layer; depend on domain models only. |
<!-- AUTO-GUARDRAILS: arch-lint END -->

Deferred rules and the `DebugPort` exemption live in `arch-lint.toml`'s inline comments and the hexagonal-reorganization superplan. The four arch-lint rules (`no-unwrap-expect`, `require-doc-comments`, `require-tracing`, `no-sync-io`) are configured in the same file with their severities (`error`, `warning`, `disabled`, `disabled`); two are disabled, and the project logs through `tracing` with no `log` or `env_logger` dependency, while startup performs sync I/O by design.

## 3. Custom syn-Based Convention Tests

`tests/infrastructure/guardrails/` holds the AST walkers and the registered convention set (see `tests/infrastructure/guardrails/mod.rs` for the current rule count and per-rule registration). Per-rule details (which patterns are checked, which scopes apply, which exemptions exist) live in the individual rule files alongside the walker code. The walker code is the source for each rule's standard and severity — this section is a navigation pointer, not a content restatement.

Four table-driven walkers enforce the design rules of the game-state lifecycle: banned name in a scope, call-site allowlist, type check, and call count. Their rows live in `tests/infrastructure/guardrails/walker_rows.rs`, one row per rule, and each row carries its reason on the comment lines above it. The Rule column holds the design rule's number. The two tables below are generated from those rows and from the `pub fn check_*` docs. A row with no reason or no `rule:` field fails `build.py guardrails-doc-check`.

<!-- AUTO-GUARDRAILS: syn START -->
| Rule | Kind | Reason | Source |
|------|------|--------|--------|
| 1 | banned name | Rule 1: every read and write names its Game. The active-Game pointer lives in `ActiveGame`, so a `static` item or an `Atomic*` type in the storage adapter would be a second hidden Game id. | `tests/infrastructure/guardrails/walker_rows.rs:14` |
| 6 | banned name | Rule 6: the guard never spans a model call. `gate.rs` holds the guard for one check and its writes, so the narrator or an Agent (an after-narration Agent, an end-of-Job Agent, their runner) named in that file would let a call run under it. | `tests/infrastructure/guardrails/walker_rows.rs:22` |
| 2 | call site | Rule 2: the active-Game pointer has one reader and one writer. `GameCatalogue` is both — `CurrentGame` reads through it once per request — and `ActiveGame::set` is `pub(super)`, so the compiler holds the write side too. | `tests/infrastructure/guardrails/walker_rows.rs:49` |
| 3 | call site | Rule 3: only `GameWriter` writes game records. The unit of record writes is opened with a capability token, so `gate.rs` is where a unit starts and no other caller can open one; the storage seam contract gets its token from `test_support`, because its crate cannot call a crate-only constructor. | `tests/infrastructure/guardrails/walker_rows.rs:59` |
| 13 | call site | Rule 13: one generation path. Only `job.rs` calls the narrator and the hooks of the Agent runner, and its step functions are the one place that does, so no second route to a model call grows beside the Job. The row sees files, not functions, and `job.rs` holds only Job code. A method call is matched through the receiver's name (`narrator`, `agent_runner`), which `JobDeps` keeps. | `tests/infrastructure/guardrails/walker_rows.rs:71` |
| 7 | type check | Rule 7: between steps, pass ids. A Job holds no record field, so a record a step loads cannot go stale in another step's hands; and no function of `job.rs` takes a record parameter, because each step loads the records it uses itself. | `tests/infrastructure/guardrails/walker_rows.rs:95` |
| 18 | call count | Rule 18: each writer takes the guard once, so a check and its writes are one critical section. Look again when a second lock (for example one guard per Game), an atomic with weak ordering in the write path, or a background Agent save that checks "this Swipe is still active" outside `hold` arrives; then the new primitive earns a `loom` seam contract. | `tests/infrastructure/guardrails/walker_rows.rs:109` |

| Rule | Description | Source |
|------|-------------|--------|
| post routes have words | Fails for a `post` route of the router that has no row in the words table. | `tests/infrastructure/guardrails/answer_words.rs:15` |
| words rows are router routes | Fails for a row of the words table that is no route of the router. | `tests/infrastructure/guardrails/answer_words.rs:46` |
| action paths have words | Fails for an `hx-get` or `hx-post` path of a template or the shell that matches no row. | `tests/infrastructure/guardrails/answer_words.rs:97` |
| enum variant docs | Requires every enum variant to have a `///` doc unless the enum is marked `/// [TRIVIAL_ENUM]`. | `tests/infrastructure/guardrails/enums.rs:119` |
| free fn location | Restricts top-level free functions to allowlisted category folders or exempt paths. | `tests/infrastructure/guardrails/free_fn.rs:31` |
| inherent impl locality | Checks all files in `src/` for inherent impls that violate the module-per-type rule. | `tests/infrastructure/guardrails/inherent_impl.rs:15` |
| wiredapp scope | Restricts `WiredApp` imports to composition-root, HTTP, test-support, and test scopes. | `tests/infrastructure/guardrails/layers.rs:9` |
| handler return type | Requires server handlers to return `Response<Body>` instead of `(StatusCode, String)`. | `tests/infrastructure/guardrails/layers.rs:58` |
| form fields urlencoded safe | Flags collection-typed fields on HTTP form structs. | `tests/infrastructure/guardrails/layers.rs:100` |
| server layer boundaries | Prevents HTTP-layer files from referencing the whole-state types directly. | `tests/infrastructure/guardrails/layers.rs:166` |
| http storage leak | Prevents HTTP layer files from directly referencing the driven `Storage` namespace. | `tests/infrastructure/guardrails/layers.rs:203` |
| test file naming | Rejects unit-test files with the singular `_test.rs` suffix in favor of `_tests.rs`. | `tests/infrastructure/guardrails/location.rs:7` |
| test file pairing | Requires every `_tests.rs` file in `src/` to have a matching source file or module directory. | `tests/infrastructure/guardrails/location.rs:42` |
| test file location | Combines test-file naming and pairing checks for `src/` test files. | `tests/infrastructure/guardrails/location.rs:86` |
| nesting depth | Fails when function-body control-flow nesting exceeds `MAX_NESTING_DEPTH`. | `tests/infrastructure/guardrails/nesting.rs:12` |
| parts in shell | Checks the dashboard shell, `assets/index.html`. A part element has a `data-part` | `tests/infrastructure/guardrails/screen.rs:48` |
| shell parts match the server | Checks the part names of the shell against the names that the server renders | `tests/infrastructure/guardrails/screen.rs:148` |
| stub answers the parts of the server | Checks the source of the stub server against the names that the server renders | `tests/infrastructure/guardrails/screen.rs:192` |
| part table matches shell | Checks a hand-written table of parts against the parts of the shell: each part of | `tests/infrastructure/guardrails/screen.rs:242` |
| scripts write no part | Checks one `assets/dashboard-*.js` script other than `dashboard-screen.js`, the | `tests/infrastructure/guardrails/screen.rs:354` |
| part write exceptions are used | Checks the exceptions of the part-write rule against the scripts they name: a | `tests/infrastructure/guardrails/screen.rs:451` |
| scripts use action fetch | Checks one `assets/dashboard-*.js` script other than `dashboard-screen.js`. A | `tests/infrastructure/guardrails/screen.rs:485` |
| handler headers | Checks one handler file of the HTTP adapter. A handler answers its own element | `tests/infrastructure/guardrails/screen.rs:506` |
| template targets | Checks one template source: a file of the HTTP adapter that renders markup, or | `tests/infrastructure/guardrails/screen.rs:539` |
| no out of band swaps | Checks one template source for out-of-band swaps. An element that shows data | `tests/infrastructure/guardrails/screen.rs:599` |
| after request forms are listed | Checks one template source, or the shell: a `<form>` with an `hx-on::after-request` | `tests/infrastructure/guardrails/screen.rs:639` |
| after request form rows are used | Checks the rows against the sources: a row whose form is gone is stale. | `tests/infrastructure/guardrails/screen.rs:662` |
| draft controls only in dialog forms | Checks one source file of the HTTP adapter. A draft control is a form control | `tests/infrastructure/guardrails/screen.rs:767` |
| dialog fragments carry the title | Checks that a literal of a dialog fragment carries the title that names the | `tests/infrastructure/guardrails/screen.rs:811` |
| draft control exceptions are used | Checks the exceptions of the draft-control rule against the files they name: a | `tests/infrastructure/guardrails/screen.rs:829` |
| scripts listen no part swaps | Checks one `assets/dashboard-*.js` script. A direct morph writes each part, so an | `tests/infrastructure/guardrails/screen.rs:870` |
| scripts write no disabled | Checks one first-party script of `assets/`. The server renders every control | `tests/infrastructure/guardrails/screen.rs:907` |
| doc standards | Enforces module-level doc-anchor standards on production files and rejects DOC anchors in test files. | `tests/infrastructure/guardrails/structure.rs:38` |
| mod purity | Enforces mod.rs purity: only module declarations, imports, and module docs are allowed. | `tests/infrastructure/guardrails/structure.rs:128` |
| empty rust file | Flags `.rs` files that contain only comments and blank lines. | `tests/infrastructure/guardrails/structure.rs:204` |
| no std thread all | Exposes the internal no-std-thread check for use outside the standard walker. | `tests/infrastructure/guardrails/structure.rs:238` |
| browser interactions use htmx settle | Full-stack browser tests must interact only through htmx-settled helpers. | `tests/infrastructure/guardrails/structure.rs:259` |
| file length | Enforces a maximum of 2000 non-blank lines per file. | `tests/infrastructure/guardrails/structure.rs:309` |
| test module header | Test files must have a single-line `//!` summary on the first non-blank line: | `tests/infrastructure/guardrails/structure.rs:333` |
| import ordering | Enforces import ordering: std/core/alloc, then external crates, then crate/super/self. | `tests/infrastructure/guardrails/style.rs:9` |
| long comment runs | Warns when five or more countable comment lines appear consecutively. | `tests/infrastructure/guardrails/style.rs:125` |
| template raw strings | Flags `r#"` raw-string literals in askama template sources. | `tests/infrastructure/guardrails/style.rs:166` |
| separator comments | Warns on visual separator comments such as `// === ... ===`. | `tests/infrastructure/guardrails/style.rs:245` |
| single letter vars | Warns on single-letter variable names in functions with more than ten statements. | `tests/infrastructure/guardrails/style.rs:260` |
| no ignored tests | Rejects every `#[ignore]`d fn, also through `cfg_attr`; the caller exempts `tests/llm/`. | `tests/infrastructure/guardrails/test_hygiene.rs:12` |
| sleep has reason | Rejects a `sleep(..)` call in a test that has no `//` comment on the line above it. | `tests/infrastructure/guardrails/test_hygiene.rs:37` |
| unit test allowlist | Fails a unit test outside the one rule, and a list entry that names no file. | `tests/infrastructure/guardrails/unit_test_allowlist.rs:180` |
| banned name rows | Runs the banned-name table: each row's scope must not name a banned name. | `tests/infrastructure/guardrails/walkers.rs:168` |
| call site rows | Runs the call-site table: only what a row allows may call its callees. | `tests/infrastructure/guardrails/walkers.rs:395` |
| type check rows | Runs the type-check table: no listed field or parameter holds a banned type. | `tests/infrastructure/guardrails/walkers.rs:427` |
| call count rows | Runs the call-count table: no function calls a row's callee more than the row allows. | `tests/infrastructure/guardrails/walkers.rs:518` |
<!-- AUTO-GUARDRAILS: syn END -->

## 4. Coverage Exclusion Policy

Coverage is measured via `cargo-llvm-cov` with file-level exclusions configured through the `--ignore-filename-regex` flag in `build.py`. The flag is used in preference to `#[coverage(off)]` attributes because the latter requires nightly Rust (feature `coverage_attribute`) and stable Rust compatibility is required. The excluded files and the reason for each are in `_generate_coverage_report` in `build.py`.

## Document References

- [`../../explanation/architecture.md`](../../explanation/architecture.md) — [§Architectural commitments](../../explanation/architecture.md#architectural-commitments) lists the guarantees the static guardrails enforce.
- [`../game_flow.md#trigger-evaluation`](../game_flow.md#trigger-evaluation) — the order in which the game rules apply a Quantifier result, which the game-rules tests observe.
- `.scratch/_closed/turn-state-model/assets/sketch-25-new-design.md` §6.2 — the design rule numbers the four table-driven walkers enforce.
