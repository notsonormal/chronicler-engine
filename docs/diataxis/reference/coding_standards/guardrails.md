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
<!-- AUTO-GUARDRAILS: clippy END -->

## 2. arch-lint (Test-Time)

Declarative architecture rules in `arch-lint.toml`.

<!-- AUTO-GUARDRAILS: arch-lint START -->
| From Scope | To Scope(s) | Rationale |
|------------|-------------|----------|
| `model` | `server, narrative, application` | Model layer must be pure; cannot depend on outer layers. |
| `application` | `server` | Application layer must not depend on server layer. |
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

Deferred rules and the `DebugPort` exemption live in `arch-lint.toml`'s inline comments and the hexagonal-reorganization superplan. The four arch-lint rules (`no-unwrap-expect`, `require-doc-comments`, `require-tracing`, `no-sync-io`) are configured in the same file with their severities (`error`, `warning`, `disabled`, `disabled`); two are disabled because the project intentionally uses `log` instead of `tracing` and sync I/O during startup.

## 3. Custom syn-Based Convention Tests

`tests/infrastructure/guardrails/` holds the AST walkers and the registered convention set (see `tests/infrastructure/guardrails/mod.rs` for the current rule count and per-rule registration). Per-rule details (which patterns are checked, which scopes apply, which exemptions exist) live in the individual rule files alongside the walker code. The walker code is the source for each rule's standard and severity — this section is a navigation pointer, not a content restatement.

<!-- AUTO-GUARDRAILS: syn START -->
| Rule | Description | Source |
|------|-------------|--------|
| enum variant docs | Requires every enum variant to have a `///` doc unless the enum is marked `/// [TRIVIAL_ENUM]`. | `tests/infrastructure/guardrails/enums.rs:69` |
| free fn location | Restricts top-level free functions to allowlisted category folders or exempt paths. | `tests/infrastructure/guardrails/free_fn.rs:34` |
| inherent impl locality | Checks all files in `src/` for inherent impls that violate the module-per-type rule. | `tests/infrastructure/guardrails/inherent_impl.rs:16` |
| wiredapp scope | Restricts `WiredApp` imports to composition-root, HTTP, test-support, and test scopes. | `tests/infrastructure/guardrails/layers.rs:13` |
| messages swipes separation | Ensures `storage/messages.rs` never references the `message_swipes` table. | `tests/infrastructure/guardrails/layers.rs:54` |
| handler return type | Requires server handlers to return `Response<Body>` instead of `(StatusCode, String)`. | `tests/infrastructure/guardrails/layers.rs:86` |
| server layer boundaries | Prevents server-layer files from referencing `GameState` directly. | `tests/infrastructure/guardrails/layers.rs:125` |
| http storage leak | Prevents HTTP layer files from directly referencing the driven `Storage` namespace. | `tests/infrastructure/guardrails/layers.rs:157` |
| test layer boundaries | Prevents component tests from constructing or importing `GameState` directly. | `tests/infrastructure/guardrails/layers.rs:191` |
| test file naming | Rejects unit-test files with the singular `_test.rs` suffix in favor of `_tests.rs`. | `tests/infrastructure/guardrails/location.rs:7` |
| test file pairing | Requires every `_tests.rs` file in `src/` to have a matching source file or module directory. | `tests/infrastructure/guardrails/location.rs:42` |
| test file location | Combines test-file naming and pairing checks for `src/` test files. | `tests/infrastructure/guardrails/location.rs:86` |
| nesting depth | Warns when function-body control-flow nesting exceeds `MAX_NESTING_DEPTH`. | `tests/infrastructure/guardrails/nesting.rs:12` |
| doc standards | Enforces module-level doc-anchor standards on production files and rejects DOC anchors in test files. | `tests/infrastructure/guardrails/structure.rs:59` |
| mod purity | Enforces mod.rs purity: only module declarations, imports, and module docs are allowed. | `tests/infrastructure/guardrails/structure.rs:158` |
| no legacy test context | Rejects legacy test-context helpers in integration tests. | `tests/infrastructure/guardrails/structure.rs:197` |
| empty rust file | Flags `.rs` files that contain only comments and blank lines. | `tests/infrastructure/guardrails/structure.rs:259` |
| no std thread all | Exposes the internal no-std-thread check for use outside the standard walker. | `tests/infrastructure/guardrails/structure.rs:293` |
| file length | Enforces a maximum of 2000 non-blank lines per file. | `tests/infrastructure/guardrails/structure.rs:298` |
| test module header | Test files must have a single-line `//!` summary on the first non-blank line. | `tests/infrastructure/guardrails/structure.rs:328` |
| import ordering | Enforces import ordering: std/core/alloc, then external crates, then crate/super/self. | `tests/infrastructure/guardrails/style.rs:9` |
| long comment runs | Warns when five or more countable comment lines appear consecutively. | `tests/infrastructure/guardrails/style.rs:124` |
| separator comments | Warns on visual separator comments such as `// === ... ===`. | `tests/infrastructure/guardrails/style.rs:219` |
| single letter vars | Warns on single-letter variable names in functions with more than ten statements. | `tests/infrastructure/guardrails/style.rs:234` |
<!-- AUTO-GUARDRAILS: syn END -->

## 4. Coverage Exclusion Policy

Coverage is measured via `cargo-llvm-cov` with file-level exclusions configured through the `--ignore-filename-regex` flag in `build.py`. The flag is used in preference to `#[coverage(off)]` attributes because the latter requires nightly Rust (feature `coverage_attribute`) and stable Rust compatibility is required. The exclusion regex targets wiring/lifecycle files (server infrastructure, `test_support`, bootstrap CLI entry, LLM backend clients), which are covered by integration tests rather than unit coverage.

## Document References

- [`../../explanation/architecture.md`](../../explanation/architecture.md) — [§Architectural commitments](../../explanation/architecture.md#architectural-commitments) lists the load-bearing guarantees the static guardrails enforce.
- [`../game_flow.md#trigger-evaluation`](../game_flow.md#trigger-evaluation) — the trigger-evaluation mutation sequence is the same one the `execute_freeaction_impl` tests observe.
