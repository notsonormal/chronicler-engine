# Deferred arch-lint Rules — Hexagonal Reorganization

Tracks arch-lint deny-scope-dep rules intentionally NOT enforced yet, with rationale and target phase.

**Decision context:** arch-lint 0.4.3 lacks TOML-level scoped file-level exemptions for `deny-scope-dep` rules. Adding the rules below would fail the build on pre-existing layer leaks that Phase 1 ("move-only") is out of scope to fix. Phase 2 closes the leaks; rules added after as cleanup, or replaced with a grep-based guardrail test where exemption scoping is needed.

## Deferred rules table

| # | Rule (`from` → `to`) | Rationale | Blocker / pre-existing leaks | Target phase |
|---|----------------------|-----------|------------------------------|--------------|
| 1 | `application` → `adapters/driven` | Application layer must not import driven adapters directly; route through application ports | Needs scoped file-level exemptions for `context.rs`, `application_service.rs`, `game_service.rs`, `action_pipeline/*.rs` — arch-lint 0.4.3 cannot express TOML-level exemptions | Phase 2.5 (comment-only doc) |
| 2 | `server` → `storage`, `narrative` | Driving adapters must not import driven adapters directly; route through ports | Phase 2.3 closed the `check_player_input` leak sites — driving adapters now call through `TextCheckService`. `templates.rs` + `view_models.rs` import `LlmMessage` and `CheckResult` — these are now port types at `application/ports/`, re-exported by driven adapters. Imports point to port types, so no longer leaks. | After Phase 2 closes the leaks (VERIFY current import paths) |
| 3 | `storage` → `narrative` | Driven adapters must not depend on other driven adapters (cross-adapter coupling forbidden) | None — rule passes today. Paired with reverse (rule #4) for symmetry. Add when #4 added. | After Phase 2 closes the leaks |
| 4 | `narrative` → `storage` | Driven adapters must not depend on other driven adapters (cross-adapter coupling forbidden) | Phase 2.1 removed `LlmBackend` trait default impls (renamed to `LlmProvider`, transport-only). `src/application/agents/registry.rs` and `src/application/agents/quantifier/agent.rs` still import `Storage` directly (`use crate::adapters::driven::storage::Storage`). These are application → driven leaks, not narrative → storage. Rule #4 itself has no narrative→storage violators post-Phase-2. | After Phase 2 closes the leaks (rule may be obsolete) |
| 5 | `application/ports` → anything | Ports are contracts; depend only on `domain` and `error` | Subsumed by rule #1 (`application` → `adapters/driven`) plus existing `application` → `server` rule | After Phase 2 closes the leaks |

## Rules explicitly NOT added (plan subsumed / already enforced)

| Rule | Reason |
|------|--------|
| `domain` → anything | Already covered by existing `model` → `{server, narrative, engine, application}` + `model` → `storage-models` rules |
| `application` → `application/ports` and `application` → `domain` (explicit ALLOW) | arch-lint deny-scope-dep only enforces DENY; ALLOW = absence of deny rule. No rule needed. |
| `adapters/driving` → `application` ALLOW | Same — ALLOW = absence of deny |
| `adapters/driving` → `domain` ALLOW | Same |
| `adapters/driven` → `application/ports` ALLOW | Same |
| `adapters/driven` → `domain` ALLOW | Same |

## Working session log

- **Task 0 (pre-flight):** arch-lint 0.4.3 capability gap discovered (no TOML-level scoped exemptions). User chose **Option B**: defer `application → adapters/driven` enforcement entirely until Phase 2.5 (comment-only documentation).
- **Phase 1.7 worker attempt:** Add the 3 new deny rules (#2, #3, #4). All 3 reverted — rule #2 and #4 hit 7+ true-positive violations (pre-existing layer leaks); build went red. User decision: revert to "scope paths only, no new deny rules" (Option-A philosophy from Task 0 extended). Phase 1.7 ships as scope-path-only.
- **Phase 1.9 review audit:** Deferred-leak catalog audited against actual codebase. Rule #2 (`server → narrative`) leak list expanded from 2 sites to 4 — added `src/adapters/driving/http/fragments/actions.rs` + `src/adapters/driving/http/fragments/misc/text_check.rs` (both import `check_player_input` from `crate::adapters::driven::text_check`, same shape as `templates.rs`/`view_models.rs`). Dead `crate::` paths in `docs/architecture/system.md` + 5 `docs/system/*.md` files rewritten to match hexagonal layout. `lib.rs` What comment removed.
- **Phase 2 plan:** Each Phase 2 sub-phase closes one leak. `tests/guardrails.rs` (or item-level `#[arch_lint::allow(...)]` if scope permits) re-evaluated per-leak; final enforcement added in Phase 2.5.
- **Phase 2 status:** Phase 2 complete (commits `4b018d3`, `33d8874`, `b1caa98`, `0819391`, `aeb7b3a`, `0c87b12`). Key changes:
  - Phase 2.1: `LlmBackend` → `LlmProvider` (transport-only, default impls removed)
  - Phase 2.3: `check_player_input` leak sites closed via `TextCheckService` orchestrator
  - Phase 2.4-2.5: `LlmCallRecorder` orchestrator owns forensics + postprocessing; `ActionPipelineBackend` deleted
  - Remaining intentional exemptions: `context.rs`, `application_service.rs`, `game_service.rs` with `// arch-lint: storage-direct — intentional, see ADR-027` markers

## Source

- Plan: [`hexagonal-reorganization-plan.md`](./hexagonal-reorganization-plan.md)
- Guardrails doc: [`../architecture/guardrails.md`](../architecture/guardrails.md)
