# Follow up on small issues found during review

Type: task (AFK)
Status: open
Blocked by: —

## Question

Code reviews and implementers on this map keep finding small issues that don't block a merge: stale references, dead code, weak assertions, stale comments. Which are still true, and what fixes them?

## Context

- This ticket collects them so they don't get lost. The coordinator appends each new item under `## Items` with its source ticket. Check each item still holds before fixing it, because later tickets may have fixed it. Drop any that no longer apply, and say so in the answer.
- Anything that grows beyond a small fix becomes its own ticket instead, added to the final re-review's `Blocked by:`.

## Items

From [Stop world creation from silently overwriting an existing key](29-world-key-silent-overwrite.md):
- SQLite `create_world` refuses with a pre-check before `INSERT OR REPLACE`. It is atomic within one process only. Consider letting the `worlds.key UNIQUE` constraint (`src/adapters/driven/storage/utils/plumbing.rs` ~150) refuse, mapped to `WorldAlreadyExists`. May fold into [Keep the world id stable when SQLite re-seeds a world](35-sqlite-reseed-keeps-world-id.md) if that changes the same insert.
- Scenario 25.7 (`docs/specs/worlds.md`) asserts only name and key are unchanged after a refused create. Also assert description and map. Its When omits the endpoint, unlike 25.1–25.5.
- `tests/http/worlds.rs` file header does not mention create.
- `tests/storage/world_storage.rs`: the new InMemory/SQLite test pairs vary literals (`duplicate`/`dup_map` vs `sql_dup`/`sql_map`), against `unit_test_standards.md` Pattern 2. The file already deviates the same way.

From [Make the test standards docs match the code](33-fix-test-standards-drift.md) (implementer findings, unverified):
- `check_no_legacy_test_context` (`tests/infrastructure/guardrails/structure.rs`) is dead: it gates on `path.starts_with("integration/")`, and `tests/integration/` no longer exists.
- `arch-lint.toml` ~66 still lists `src/application/narrative_prompt/**` (now `src/application/prompting/`).
- `docs/diataxis/how-to/debugging.md` ~78 still points at `src/application/narrative_prompt/`.
- `PipelineHelpers` in `tests/helpers/application_ext.rs` is declared by no test binary and carries `#![allow(dead_code)]`. Delete it or wire it in.
- `docs/plans/t9-00-follow-up-3-apply-now-review-fixes-revised.md` mentions the removed `SqliteTestAppBuilder`. Likely a stale plan: archive or leave, don't rewrite history.

## Done when

- Each item is fixed or dropped with a one-line reason in `## Answer`.
- Tests follow the map's Notes. Pure comment and docs fixes need no test.
- `python build.py` is green. Commit after review.
