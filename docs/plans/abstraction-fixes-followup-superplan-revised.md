# Super-Plan: Abstraction debt (refresh)

**Date:** 2026-08-12  
**Status:** Planning  
**Scope:** `chronicler_engine/` root (project moved out of the `chronicler_engine/` subdirectory; paths below are root-relative)

## What changed since the original plan
- The original plan relied on `reports/abstraction-antipatterns-summary.md` and `docs/reviews/zone-*.md`; those files no longer exist.
- Several tracks were already completed by the refactor:
  - **T3 Service-layer cleanup** — `MessageEditingService` deleted; delegates flattened; spawn helper exists.
  - **T4 MockBackend modernization** — fields are `pub(crate)`; builder methods exist; only `MockBackend::succeeding()` is missing.
  - **T1 Error-model unification** — `error_return` helper removed; pipeline now uses `PhaseError`; `GenerationStatus::Error` is UI-only. A small boundary-hardening plan covers the rest.
- The `docs/adr/` directory was removed, so any sub-plan step that says "write/amend ADR-XXX" must target a `docs/diataxis/explanation/` doc or a code comment instead.

## Remaining tracks

| # | Track | Current location | Status | Owner |
|---|-------|------------------|--------|-------|
| T5 | Type collapses | `src/domain/model/quantifier.rs`, `src/domain/model/template.rs` | ready | standalone plan |
| T6 | `MessageHistory` encapsulation | `src/domain/model/message_history.rs` | ready | standalone plan |
| T10 | Low-priority cleanup bundle | see `t10-low-priority-cleanup-bundle-revised.md` | ready | standalone plan |
| T2-ARCH | Narration deepening | `src/application/pipeline/phases.rs`, `src/application/arrival_service.rs` | needs grilling first | standalone plan |
| T9 | Docs/migration alignment | `docs/diataxis/`, module doc anchors | partially done | small follow-up |

## Closed tracks (do not re-open)
- **T3** — done.
- **T4** — done except `MockBackend::succeeding()`; that can be folded into T5/T10 opportunistically.
- **T1** — superseded by `t1-error-model-unification-revised.md`.
- **T7 Storage API polish** — completed by earlier storage/backend refactor.

## Recommended order
1. **T9** — low-risk doc/anchor fixes; unblocks accurate audits.
2. **T1 boundary hardening** — small, closes the error-model discussion.
3. **T5** — mechanical, many call sites but no behavior change.
4. **T6** — isolated, can run parallel to T5.
5. **T2-ARCH** — requires `improve-codebase-architecture` grilling before any interface design.
6. **T10** — opportunistic; pick items from the revised bundle.

## Cross-references to revised sub-plans
- `t1-error-model-unification-revised.md`
- `t5-type-collapses.md`
- `t6-messagehistory-encapsulation.md`
- `t10-low-priority-cleanup-bundle-revised.md`
- `t2-arch-narration-deepening.md`
- `t9-00-follow-up-3-apply-now-review-fixes-revised.md` (covers some T9 cleanup)

## T9: Docs/migration alignment (mini-plan)

### Scope
1. Update module doc anchors that still point to `docs/architecture/system.md`; they should point to the specific domain doc (`docs/diataxis/reference/game_flow.md`, `startup.md`, etc.).
2. Remove or update doc references to `docs/adr/adr-XXX.md` in source comments and plans; point to `docs/diataxis/explanation/*.md` where appropriate.
3. Regenerate `docs/AGENTS.md` structure index after any doc moves.

### Acceptance criteria
- `python scripts/validate_docs.py` passes.
- `rg 'docs/adr/adr-' src/ docs/plans/` returns only historical notes or archived plans.
- `rg 'docs/architecture/system\.md' src/` returns only expected exempt files per `tests/infrastructure/guardrails/structure.rs`.

## Verification
- Each sub-plan defines its own tests.
- Project gate: `python build.py` green after each track merges.
