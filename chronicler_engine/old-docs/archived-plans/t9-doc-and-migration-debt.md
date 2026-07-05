> Superseded by [t9-doc-quickwins-supersede.md](../../docs/plans/t9-doc-quickwins-supersede.md) (2026-07-05). Original tasks 1 (CHANGELOG retroactive), 4 (re-export shield — already DONE 2026-06-28 outside this plan), and 5 (`abstraction-antipatterns-summary.md` annotation — file lives in `old-docs/reviews/outdated/`, deprecated) dropped. Tasks 2 (`action_pipeline.md`), 3 (`message_model.md`), and 6 (test_support `[DOC:]` anchors) absorbed into the new plan.

# T9: Doc / Migration Debt

**Parent Plan:** [abstraction-fixes-followup-superplan.md](./abstraction-fixes-followup-superplan.md)
**Status:** Planning — ready
**Date:** 2026-06-28
**Depends on:** none
**Blocks:** none (do FIRST so future audits have anchors)
**Priority:** P0
**Findings owned:** N18, Phase 4 re-export shield, CHANGELOG Phase 4-7, `abstraction-antipatterns-summary.md` annotation, `docs/system/action_pipeline.md` missing, `docs/system/message_model.md` missing

---

## Summary

Pure docs + ~5 import-path-only changes. No code logic. Do FIRST so future audits have anchors.

## Key Changes

1. **CHANGELOG** — retroactive Phase 4-7 entries (module splits `state/`, `misc/`, `renderers/`; `GameLifecycleService` inline/delete; `Message` accessor pattern; `apply_to` deletion; `Operation` enum removal; `assemble_prompt_text` relocation). Mark "completed in `6a8531e`" where applicable.
2. **`docs/system/action_pipeline.md`** — spec covering `PipelineInputs` + `spawn_pipeline_task` helper contracts. Propose DELTA from current documented arch, not contradiction.
3. **`docs/system/message_model.md`** — spec covering accessor-pattern `Message` struct (reads from `swipes[active_swipe_index]`, no mirrored fields). ADR-017 alignment.
4. **Migrate top-5 high-churn callers** off the `state/mod.rs:12-18` re-export shield — `game_service.rs`, `application_service.rs`, `pipeline.rs`, `phases.rs`, +1. Architecture-lens candidate #8: re-export shields are pure pass-throughs failing the deletion test; **architecture debt, not migration debt**. **DONE 2026-06-28: shield deleted, all callers migrated to direct submodule paths (`crate::model::state::<sub>::<Symbol>`). This overturns the original deferral.**
5. **Annotate `docs/reviews/abstraction-antipatterns-summary.md`** with status notes ("resolved Phase 2", "deferred — see super-plan Finding State table"). Cross-reference each finding to its Finding State class + owner track — not just the 4 misclassifications (B2/B7/B8/B9) — all findings.
6. **Add `[DOC: ...]` anchors** to the 7 `src/test_support/` files (N18).

## Out of Scope

- New ADR for deliberate deferrals — dropped 2026-06-28. N19 closed: deliberate-defers are protected by the super-plan Finding State table (maintained copy) + task 5 annotation on `abstraction-antipatterns-summary.md` + existing ADR-018 + CHANGELOG `8e4acf5`. No new ADR.

## Blast Radius

docs + ~5 import-path-only changes.

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean.
- Manual check: each `[DOC: ...]` anchor added to `test_support/` files resolves to the file path it claims to document.
- Verify CHANGELOG Phase 4-7 entries match actual code state (run `git log --oneline 6a8531e~1..HEAD` for commit list).

## Pre-Implementation Checklist

- [ ] Confirm `docs/system/` directory exists; if not, create it.
- [ ] Enumerate the 7 `src/test_support/` files needing `[DOC:]` anchors via `find src/test_support -name '*.rs'`.
- [ ] Check `docs/reviews/abstraction-antipatterns-summary.md` exists and is annotation-ready.
