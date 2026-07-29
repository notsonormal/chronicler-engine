# ADR-033: Dissolve `domain/engine/` Subfolder

**Date:** 2026-07-27
**Status:** Accepted
**Supersedes:** `### \`domain/engine/\` Subfolder Kept` block in [ADR-027](./adr-027-hexagonal-architecture-migration.md) (lines 88–90)

## Context

ADR-027 recorded the decision to keep `src/domain/engine/` as a 7-file subfolder separating "types (`model/`) vs rules (`engine`)" at zero cost. The subfolder was aspirationally a rules module, but every resident turned out to be a parked method — each had a single behavioral owner elsewhere in `domain/model/` (typically `GameState` or `Room`). That made `domain/engine/` a misc-folder in practice: a category-less dumping ground for "engine stuff" that belonged on the owning type.

The free-fn scanner effort relocated the residents and deleted the folder. ADR-027's "Subfolder Kept" block is now factually stale.

## Decision

Dissolve `src/domain/engine/`. No generic "rules" subfolder replaces it. Pure-rule functions live as `impl` methods on their owning domain type, in the type's module file or a sibling `_*_tests.rs` block. `domain/mod.rs` carries `pub mod model;` only — no `engine` module.

The free-fn allowlist in `tests/infrastructure/guardrails/free_fn.rs` has no entry for `domain/engine/` and never will — the folder does not exist. The free-fn guardrail applies immediate-parent category enforcement to every non-test source file; `domain/model/` is not structurally exempt. Free fns there remain subject to the standard scanner triage (genuine free-fn category vs parked method).

## Consequences

- ADR-027 no longer documents a folder that does not exist. `domain/model/` is the only domain subfolder; "types vs rules" communicates through `impl` methods on owning types, not folder partitioning.
- The misc-folder pattern that motivated this ADR is structurally prevented from recurring in `domain/`.
- ADR-027's grandfathered-path list agrees with `tests/infrastructure/guardrails/layers.rs`.

Trade-off: `GameState` method count grew as the relocations landed. Large `impl` blocks are the cost of behavioral cohesion over file-size orthodoxy — acceptable because the methods are the type's behavior, not a separate "engine" concern.

## Related ADRs

- [ADR-027: Hexagonal Architecture Migration](./adr-027-hexagonal-architecture-migration.md) — superseded `domain/engine/ Subfolder Kept` block; corrected `gate.rs` paths in the grandfathered list.
- [ADR-028: Test Module Header Convention](./adr-028-test-module-header-convention.md) — test siblings moved alongside their relocated `impl` blocks per this convention.

## History

- **2026-07-27**: Initial decision. Records prior effort's folder dissolution + path correction in one superseding ADR.
