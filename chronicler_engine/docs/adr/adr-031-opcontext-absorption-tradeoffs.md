# ADR-031: OpContext Absorption Trade-offs

**Date:** 2026-07-10
**Status:** Accepted

## Context

The OpContext-kill (commit 20cacf9) deleted `application/context.rs` (284 lines) and absorbed its responsibilities into `DefaultApplicationService` via 5 accessor methods + `pub(crate)` fields. The OpContext was a grab-bag struct wrapping storage, preset_storage, settings, cancel_token, is_generating, and game_service.

Doubt-driven review (verdicts D1 + D2, recorded in simpler-hexagon-pre-merge-superplan.md Appendix D) found:

- Deletion moved complexity rather than removing it. `DefaultApplicationService` became a 49-method god-object (723 LOC) — a mirror of the deleted OpContext under a different name.
- Test factory sprawl grew: 11 inline builders + `DefaultApplicationService::new` became the most-replicated 6-arg signature in the codebase (13+ inline constructions).
- Tests grew longer (`retry_main.rs` +99 lines), not shorter.
- No ADR documented this decision. Plan tasks §T2.2 (OpContext FromRequestParts extractor) + §T2.3 (GameState::from_snapshot WorldSnapshot variant) were silently invalidated without ceremony — a process violation per AGENTS.md Plan Adherence rule.

The hexagonal boundary (ADR-027) was preserved — verified: 0 OpContext references in `src/`. The improvement is real but was overstated; the trade-off cancelled much of the leverage.

## Decision

Accept the absorption as a temporary trade-off. The hexagonal seam is preserved, but complexity concentrated in `DefaultApplicationService`. T2 (god-class split) is the corrective action.

T2 has since landed (tickets 00–04, 2026-07-09): 4 cohesive modules carved out — `PersistenceGate`, `GenerationGate`, `GameCatalogue`, `WorldCatalogue` — and the `DefaultApplicationService` façade shrunk from 723 to 275 LOC. The `PresetStore` newtype (ADR-027 port+adapter alignment) closes the phantom-`Arc<Storage>` seam that the OpContext previously masked.

## Consequences

### Positive

- Hexagonal seam preserved (ADR-027 compliance: no driving-side leakage of infrastructure types into application/domain).
- Simpler call-site ergonomics: handlers use direct `state.application_service.X()` access.
- T2 modular split delivered the long-term shape — facade-first preserved ~30 caller signatures untouched.

### Negative

- `DefaultApplicationService` briefly became a 49-method god-object (723 LOC) — a regression in module depth masked as an OpContext detour.
- Test factory sprawl (11 builders) — corrective action is T5 (TestApp builder collapse).
- Plan tasks §T2.2 + §T2.3 silently invalidated — process violation per AGENTS.md Plan Adherence. T6 (this ADR + plan VOID markers) is the honesty corrective.

### Trade-offs

- Accept temporary god-class regression in exchange for preserved hexagonal boundary — corrected by T2 rather than reverted.
- T2 modular split landed 2026-07-09 (facade now 275 LOC) rather than immediate re-tightening — the gap was tolerated for one branch.
- T9 (WorldSnapshot removal) will supersede the §T2.3 WorldSnapshot variant entirely — deferred to its own sub-plan. **[Resolved 2026-07-11]** T9-01 removed `WorldSnapshot` + `load_world_snapshot` + `world_snapshot_or_empty`; GameState no longer bundles world-data fields; orchestrators fetch directly from `app.storage()`.
- T6 marks §T2.2, §T2.3, §A6.4, §B1.3 VOID in the stale plans with rationale pointing here, rather than rewriting or deleting those plans.

## Alternatives Considered

- **Resurrect OpContext as a thin struct.** Rejected — reintroduces the seam without solving cohesion; the 4-module T2 carve-out achieves the original intent more cleanly.
- **Full caller-site migration instead of facade-first (T2 Decision G1=B).** Rejected — too high blast radius (~30 caller files) for a single branch; facade-first deferred G1-B to a follow-up.
- **Revert the OpContext-kill.** Rejected — the hexagonal boundary gain (no driving-side infrastructure leakage) is worth keeping; the cost was modularity, addressed by T2.
