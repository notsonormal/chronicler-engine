# T13: Remove `PhaseError::SnapshotMissing` variant

> **Status:** Chartered (post-T12). Run after T12 ships.
> **Scope:** 2 SP code change. One variant removed, one match arm removed, one ADR amended.

## Summary

`PhaseError::SnapshotMissing` is unused code. The variant exists at `src/application/action_pipeline/phase_error.rs:20` and has a match arm in `finalize_phase_error` at `src/application/action_pipeline/pipeline.rs:206` (returning `"World data unavailable for current game"`), but no construction site exists in `src/` or `tests/`. Confirmed via:

```
$ grep -rn SnapshotMissing chronicler_engine/src chronicler_engine/tests
src/application/action_pipeline/phase_error.rs:20:    SnapshotMissing,
src/application/action_pipeline/pipeline.rs:206:        PhaseError::SnapshotMissing => "World data unavailable for current game".to_string(),
```

Two matches, both declaration and consumer — zero producers. This is dead code.

Surfaced during T12's audit: the doc rewrite dropped the variant from the `PhaseError` variant table because the variant never fires; T13 removes it from source to keep the doc and enum in sync.

## Key Changes

- Remove `SnapshotMissing` variant from `PhaseError` enum
- Remove its match arm from `finalize_phase_error`
- Amend ADR-032 to note the variant's removal (History section)
- No behavior change — no caller fires the variant

## Implementation

### Phase 1: Code removal

- [ ] #### Task 1.1: Remove `SnapshotMissing` from `PhaseError` (1 SP)
  - [ ] ##### SubTask 1.1.1: Edit `src/application/action_pipeline/phase_error.rs` — delete the variant line at line 20.
  - [ ] ##### SubTask 1.1.2: Edit `src/application/action_pipeline/pipeline.rs` — delete the `PhaseError::SnapshotMissing => "World data unavailable for current game".to_string()` match arm at line 206.
  - [ ] ##### SubTask 1.1.3: Amend `docs/adr/adr-032-phaseerror.md` History section with a dated note: 2026-07-15, removed `SnapshotMissing`, never constructed.

### Phase 2: Verify

- [ ] #### Task 2.1: Validate (1 SP)
  - [ ] ##### SubTask 2.1.1: `cargo check` — confirms exhaustive matches still compile (only `finalize_phase_error` matches `PhaseError` exhaustively).
  - [ ] ##### SubTask 2.1.2: `python build.py` — full pipeline (fmt + clippy + tests + coverage).

## Test Plan

- `cargo check` — exhaustive match arms still compile.
- `python build.py` — green.
- Diff shows: 1 variant deleted, 1 match arm deleted, ADR History line added.

## Assumptions

- No external code depends on `PhaseError` — ADR-032 explicitly states "No cross-boundary bubble today; orchestrators consume variants inline; the HTTP layer never sees a `PhaseError`."
- No test asserts the variant's existence — grep on `tests/` returned 0 matches.
- If `cargo check` reveals an exhaustive-match site missed by grep, abort and re-scope; the variant stays.

## Story Points

2 SP total (1 SP code + 1 SP validation).

## Relationships

- **Follows:** T12 (action_pipeline.md doc rewrite). T12 dropped the `SnapshotMissing` row from the variant table; T13 removes the variant from source. The two plans share no files.
- **Amends:** ADR-032 (`docs/adr/adr-032-phaseerror.md`) — History section.
