# T12: Rewrite `docs/system/action_pipeline.md`

> **Status:** Implementing (post-audit).
> **Scope:** One doc, no code, no sibling docs. T13 follow-up charters the dead-variant removal surfaced during this audit.

## Summary

`docs/system/action_pipeline.md` was flagged in three earlier passes: a now-deleted audit plan (5 accuracy findings, migrated into this rewrite), the closed `enum-doc-pruning` wayfinder noted pre-existing Phase 1 sediment + Phase 5 ghost features and deferred them to "a future docs-hygiene effort", and recent code review surfaced broader structural problems — the diagram does three jobs at once (happy path + cancel + error), the prose is indexing-style rather than explanatory, em-dash chains and parenthetical stacks make the doc hard to scan. This plan is the rewrite. One doc, no siblings in scope, no code changes in T12. T13 is a 2 SP follow-up that removes the dead `PhaseError::SnapshotMissing` variant surfaced during the audit.

## Key Changes

- One file rewritten: `chronicler_engine/docs/system/action_pipeline.md`
- Diagram collapsed to one main-pipeline flowchart (phases named by purpose, not by function); cancel + error paths moved to tables/prose
- Prose rewritten declarative present-tense, leading with the invariant and following with the mechanism that serves it
- `PhaseError` variant table kept (per user direction) with light cleanup: each row is contract + recovery policy, no who-constructs / who-consumes / Rust side-effect enumeration
- `SnapshotMissing` row dropped from the table (variant is unfired — T13 removes it from source)
- Five accuracy findings from the deleted audit plan migrated into the rewrite:
  - Four `persist_snapshot_or_err` sites named in the `PersistFailed` row: pre-main, pre-event, post-trigger, post-engine
  - Retry postcondition/precondition split paragraph restored (preconditions skip finalize; rely on heal-stale-state path)
  - `retry_target` re-insertion timing qualifier restored (after engine commit and before trigger continuation)
  - `pub(crate)` modifier and `reconcile_post_trigger_npcs` symbol already absent
  - Cancel-checkpoints prose folded into the Cancellation section
- `chronicler_engine/docs/AGENTS.md` AUTO-INDEX entry unchanged (still `./system/action_pipeline.md`)
- New follow-up plan: `docs/plans/t13-remove-snapshotmissing-variant.md` (dead variant removal)

## Constraints (vs prior plans)

- **No subagents.** Primary agent rewrites one file end-to-end.
- **No code changes in T12.** Doc-only.
- **No sibling docs edited.** `game_flow.md` has a small PhaseError paragraph that overlaps; out of scope.
- **No new wayfinder.** The deferred "future docs-hygiene effort" referenced by `enum-doc-pruning/map.md` remains a placeholder; this plan is one piece, not the whole effort.
- **T13 is a separate plan.** T12 ships first; T13 follows. They share no files (T12 touches the doc, T13 touches `phase_error.rs` + `pipeline.rs` + ADR-032).

## Implementation

### Phase 1: Rewrite the doc

- [x] #### Task 1.1: Rewrite `docs/system/action_pipeline.md` (5 SP)
  - Read the existing file in full before writing.
  - Apply the structural moves (full-file rewrite, not incremental edits):
    - Add `**Scope:**` preamble matching `game_flow.md` / `narration_engine.md` convention.
    - Drop the `Arc`-sharing clause from the Scope paragraph (implementation fact, not contract).
    - Replace the single `Phase Flow` mermaid diagram with one main-pipeline flowchart. Phases named by purpose (`pre-main snapshot`, `narrate`, etc.), not by function name.
    - Replace the `PhaseError variants` bullet list with a row-per-variant table. Each row: variant name, contract, recovery policy. Drop the helper→variant mapping and the Rust-jargon side-effect enumerations. Drop the `SnapshotMissing` row entirely.
    - Rewrite the `phase_finalize` paragraph so the invariant leads ("Errors must survive finalize.") and the mechanism follows.
    - Rewrite the `Cancellation` paragraph as two prose paragraphs (one per mechanism) + a 3-bullet list of alpha-check sites with semicolon-joined timing qualifiers instead of parenthetical stacks.
    - Drop module paths (`application::message_editing`, `generation_gate`) from the cancellation prose; name handlers by role only.
    - Keep the `Retry` bullet list but drop the "Retry path splits" monster sentence; replace with a two-sentence paragraph distinguishing postcondition (finalize seam) and precondition (direct persist + heal-stale-state) failures.
    - Keep the `Document References` block verbatim.
  - Do not introduce new code references, new paths, or new ADR references. The cross-ref list at the bottom stays as-is.

  - [x] ##### SubTask 1.1.1: Draft the rewrite (3 SP)
    - One full-file write. Write replaces, not incremental.
    - Target length: ~80 lines (current is 122). Length is not the goal; structure is.
  - [ ] ##### SubTask 1.1.2: Mechanical validation (1 SP)
    - `python scripts/validate_docs.py chronicler_engine/docs/system/action_pipeline.md` → expect 0 errors, 0 warnings.
    - `python scripts/validate_docs.py` (no path) → expect full pass.
  - [ ] ##### SubTask 1.1.3: Semantic audit + grep spot-checks (1 SP)
    - Run `chronicler-docs-hygiene` audit on the new doc. Phases of interest: Phase 1 (sediment), Phase 3 (tone rot), Phase 4 (code-indexer), Phase 8 (enum paraphrase).
    - Verify carried-over implementation-leveled claims against `src/`:
      - Four `persist_snapshot_or_err` sites: pre-main, pre-event, post-trigger, post-engine
      - `retry_persist_error` is the precondition-failure helper (not `save_retry_error`) at `application/action_pipeline/retry.rs`
      - `heal_stale_generating` exists at `application/application_service.rs`
      - `state.narrative.last_trigger` is the absent field for `TriggerMissing`
      - `phase_finalize` resets to Idle unless Error
      - `load_world_bundle` exists as the precondition-fetch failure point

### Phase 2: Charter the T13 follow-up

- [x] #### Task 2.1: Write `docs/plans/t13-remove-snapshotmissing-variant.md` (1 SP)
  - Content: 2 SP plan to remove the unfired `PhaseError::SnapshotMissing` variant.
  - Two file edits + one ADR amendment:
    - Delete variant line at `src/application/action_pipeline/phase_error.rs:20`
    - Delete match arm at `src/application/action_pipeline/pipeline.rs:206` (`PhaseError::SnapshotMissing => "World data unavailable for current game".to_string()`)
    - Amend ADR-032 History section: dated note that `SnapshotMissing` was removed (never constructed).
  - Validation: `cargo check` (exhaustive matches still compile — `finalize_phase_error` is the only consumer) + `python build.py` (full pipeline).
  - Note: T13 is a separate plan, executed after T12 ships.

## Test Plan

After the T12 rewrite (single end-of-plan gate):

1. `python scripts/validate_docs.py chronicler_engine/docs/system/action_pipeline.md` → 0 errors, 0 warnings.
2. `python scripts/validate_docs.py` (no path) → 0 errors, 0 warnings across all docs.
3. `chronicler-docs-hygiene` audit on the rewritten doc, Phase 1 (sediment) + Phase 3 (tone rot) + Phase 4 (code-indexer) + Phase 8 (enum paraphrase) → 0 findings.
4. Grep spot-checks on the carried-over implementation-leveled claims (see SubTask 1.1.3) → all verified.
5. Diff: full-file rewrite of `action_pipeline.md` (not incremental) + new `docs/plans/t13-remove-snapshotmissing-variant.md`; no other files touched.

T13's test plan is internal to its own plan file.

## Per Task/Sub Task Validation Steps

- SubTask 1.1.1 (draft): no validation — drafting is its own gate.
- SubTask 1.1.2 (mechanical): `validate_docs.py` on the file + full pass. Both must be 0/0 before SubTask 1.1.3 starts.
- SubTask 1.1.3 (semantic): `chronicler-docs-hygiene` audit on the file + grep spot-checks. Zero findings + all spot-checks pass to ship.
- Task 2.1 (T13 plan): no validation beyond the plan file existing and the plan being internally consistent.

## Assumptions

- Primary agent rewrites the file in a single session; no `general-purpose` subagent.
- Cross-doc drift on shared PhaseError language (`game_flow.md`'s paragraph references `Cancelled`, `PersistFailed`) is out of scope and accepted as residual.
- New variant-list table format survives `chronicler-docs-hygiene` Phase 8 because each row carries a Data-Flow Claim (which variant escapes / where it terminates) and a behavior-specific contract (the recovery policy).
- The original `fix-pre-existing-doc-audit-findings.md` plan (including its Task 1.11 on this doc) has been deleted. The five accuracy findings it documented are migrated directly into the T12 rewrite (see Key Changes); no separate mega-plan coordination needed.
- The deferred "future docs-hygiene effort" noted in `.scratch/enum-doc-pruning/map.md` remains a placeholder; this plan is one piece, not a charter of the whole effort.
- T13 removes the `SnapshotMissing` variant from source after T12 ships. If T13 freezes (variant stays in source for some reason), the T12 doc still reads correctly — it never mentions the variant, and the absence is consistent with "doc reflects only contracts the runtime actually fires".
- Title remains `Action Pipeline` (mixed convention in `docs/system/`; minimal diff preferred).
- No `python build.py` required for T12 — doc-only change. `build.py` is required for T13 (touches `src/`).
