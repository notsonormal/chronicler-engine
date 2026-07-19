---
diataxis: reference
title: Architecture Guardrails
---

## Overview

The Chronicler Engine enforces coding standards through three complementary layers: clippy lints at compile time, declarative `arch-lint` rules at test time, and custom `syn`-based convention walkers at test time. A coverage exclusion policy (`cargo-llvm-cov` with `--ignore-filename-regex`) keeps the wiring/lifecycle code from being unit-tested separately from its integration tests. Seven runtime invariants (INV-001..INV-007) describe engine behavior at runtime; they sit alongside the static guardrails but are enforced by the invariant contract suite rather than by clippy or `syn` walking. All four layers run in CI via `python build.py`.

This document is a navigation map to the rule registries. The qualifier tables in §1, §2, and §5 carry stable identifiers (lint names, scope rows, INV-NNN identifiers); per-rule standard/severity/scope/exemptions/checks for the syn-based rules live alongside the walker code.

## 1. Clippy (Compile-Time)

Clippy lints are denied at the crate root in `src/lib.rs` via `#![deny(...)]`. Tests are exempt via `#![cfg_attr(test, allow(...))]`. `cognitive-complexity-threshold` and `too-many-arguments-threshold` thresholds live in `clippy.toml`.

| Lint | Rationale |
|------|-----------|
| `clippy::unwrap_used` | Prevent panics in production; propagate errors with `?` |
| `clippy::expect_used` | No "should never happen" assumptions in production |
| `clippy::dbg_macro` | No debug prints in committed code |
| `clippy::todo` | No unfinished code in production |
| `clippy::unimplemented` | No stubs in production |
| `clippy::print_stdout` | No `println!` in library code |
| `clippy::print_stderr` | No `eprintln!` in library code |
| `clippy::panic` | No explicit panics |

## 2. arch-lint (Test-Time)

Declarative architecture rules in `arch-lint.toml`. Scope/layer enforcement rows are stable identifiers a reader can grep for.

| From Scope | To Scope(s) | Rationale |
|------------|-------------|----------|
| `model` | `server`, `narrative`, `engine`, `application` | Domain layer must be pure; no outer layer dependencies |
| `engine` | `server`, `application`, `narrative` | Engine layer isolated from orchestration and I/O |
| `application` | `server` | Application orchestration independent of driving adapters |
| `server` | `storage` | Driving adapter must not access storage directly; use application ports |

Deferred rules and the `DebugPort` exemption live in `arch-lint.toml`'s inline comments and the hexagonal-reorganization superplan. The four arch-lint rules (`no-unwrap-expect`, `require-doc-comments`, `require-tracing`, `no-sync-io`) are configured in the same file with their severities (`error`, `warning`, `disabled`, `disabled`); two are disabled because the project intentionally uses `log` instead of `tracing` and sync I/O during startup.

## 3. Custom syn-Based Convention Tests

`tests/infrastructure/guardrails/` holds the AST walkers and the registered convention set (21 rules). The full set of registered tests lives in `tests/infrastructure/guardrails/mod.rs`; per-rule details (which patterns are checked, which scopes apply, which exemptions exist) live in the individual rule files alongside the walker code. The walker code is the source for each rule's standard and severity — this section is a navigation pointer, not a content restatement.

## 4. Coverage Exclusion Policy

Coverage is measured via `cargo-llvm-cov` with file-level exclusions configured through the `--ignore-filename-regex` flag in `build.py`. The flag is used in preference to `#[coverage(off)]` attributes because the latter requires nightly Rust (feature `coverage_attribute`) and stable Rust compatibility is required. The exclusion regex targets wiring/lifecycle files (server infrastructure, `test_support`, bootstrap CLI entry, LLM backend clients), which are covered by integration tests rather than unit coverage.

## 5. Runtime Invariants

Machine-checkable statements about engine runtime behavior. The INV-NNN identifiers are stable seams an engineer or LLM can grep for; the actual guarantee text and the test that enforces it live in the invariant contract tests, not in this document. INV-NNN identifiers and their corresponding test names share a prefix (`test_inv001_*`, `test_inv002_*`, etc.) so a grep for either finds the other. The invariant contract tests live at `tests/infrastructure/invariant_contract.rs`. INV-003 is enforced via `tests/infrastructure/guardrails/mod.rs::guardrails_no_std_thread`, and INV-005's poison-recovery site is exercised by `tests/poison_recovery.rs::test_settings_recover_from_poisoned_rwlock`. Whether other INV-NNN identifiers have a corresponding runtime test is determined by grepping the contract tests; this document does not assert which invariants have tests.

| Identifier | Short name |
|------------|------------|
| INV-001 | Generation Status Lifecycle |
| INV-002 | State Mutation Order |
| INV-003 | No Raw OS Thread Spawning |
| INV-004 | LLM Calls Are Cancellable |
| INV-004b | No Concurrent Async Actions |
| INV-005 | Lock Poison Recovery |
| INV-006 | All Actions Are Async |
| INV-007 | Actions Return Immediately |

## Running Guardrails

```bash
cargo nextest run --test architecture
cargo nextest run --test guardrails
```

## Document References

- [`../../explanation/architecture.md`](../../explanation/architecture.md) — [§Quality Story](../../explanation/architecture.md#quality-story) quality-attribute table cites INV-001, INV-003, INV-004, INV-004b, INV-005, INV-006, INV-007.
- [`../game_flow.md#trigger-evaluation`](../game_flow.md#trigger-evaluation) — INV-002 mutation sequence is the same one the trigger phase observes.
- [ADR-010: Concurrency and Generation Gate Model](../../../docs/adr/adr-010-concurrency-generation-gate.md) — tokio migration rationale for INV-003 / INV-004.
- [ADR-014: Action Pipeline Architecture](../../../docs/adr/adr-014-action-pipeline.md) — phase-based pipeline rationale for INV-002 mutation order.
- [ADR-027: Hexagonal Architecture Migration](../../../docs/adr/adr-027-hexagonal-architecture-migration.md) — port ownership and the `DebugPort` exemption.
- [ADR-030: `is_generating` Dual-Source Invariant](../../../docs/adr/adr-030-is-generating-invariant.md) — dual-source consistency rationale for INV-001.
