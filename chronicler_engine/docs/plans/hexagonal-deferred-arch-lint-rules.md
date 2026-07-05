# Deferred arch-lint Rules — Hexagonal Reorganization

Tracks arch-lint deny-scope-dep rules intentionally NOT enforced yet.

**Update 2026-07-05:** Phase B of the hex gap closure plan landed the split scopes (`ports`, `driven-llm`, `driven-text-check`, `narrative`) and deny rules for `ports → [driven-llm, driven-text-check, narrative, server, storage, storage-models, bootstrap, test-support, engine]` plus `driven-llm ↔ driven-text-check`. Rules #1, #2, #5 below are now either enforced or superseded; only #3 + #4 remain deferred.

## Enforced rules (Phase B, 2026-07-04)

| Rule | arch-lint.toml entry |
|------|----------------------|
| `ports` → adapters/outer layers | `from = "ports" to = [driven-llm, driven-text-check, narrative, server, storage, storage-models, bootstrap, test-support, engine]` |
| `driven-llm` ↔ `driven-text-check` | bidirectional deny |
| `application` → `storage-models` | already enforced pre-Phase-B |
| `server` → `storage` | already enforced pre-Phase-B |

## Deferred rules table

| # | Rule (`from` → `to`) | Rationale | Blocker | Target |
|---|----------------------|-----------|---------|--------|
| 3 | `storage` → `driven-llm`/`driven-text-check` | Driven adapters must not depend on each other | Paired with reverse (rule #4) for symmetry | When #4 added |
| 4 | `driven-llm`/`driven-text-check` → `storage` | Driven adapters must not depend on each other | Currently no violations — rule passes today | Add when #3 paired |

## Rules explicitly NOT added (already enforced or no-op)

| Rule | Reason |
|------|--------|
| `domain` → anything | Covered by existing `model → {server, narrative, engine, application}` + `model → storage-models` rules |
| `application` → `application/ports` ALLOW | arch-lint deny-scope-dep only enforces DENY; ALLOW = absence of deny rule |
| `adapters/driving` → `application`/`domain` ALLOW | Same — ALLOW = absence of deny |
| `adapters/driven` → `application/ports`/`domain` ALLOW | Same |
| `application` → `adapters/driven` (former rule #1) | Deferred — needs scoped file-level exemptions for `context.rs`, `application_service.rs`, `game_service.rs` (Storage direct access — see ADR-027 Deviation 3); arch-lint 0.4.3 cannot express TOML-level exemptions. Storage exemption section in ADR-027 documents the 3 exempt files with `// arch-lint: storage-direct` markers. |
| `server` → `storage`/`narrative` (former rule #2) | `server → storage` already enforced; `server → narrative` no longer a leak post-Phase-2 |
| `application/ports` → anything (former rule #5) | Subsumed by `ports → [many]` rule added Phase B |

## Source

- Plan: [`hexagonal-architecture-gap-closure.md`](../old-docs/archived-plans/hexagonal-architecture-gap-closure.md) (archived)
- Guardrails doc: [`../architecture/guardrails.md`](../architecture/guardrails.md)
