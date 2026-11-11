# Migrate existing integration tests to the right tier

Type: research (AFK)
Status: closed
Blocked by: 16 (resolved)
Resolved: 2026-08-08
Asset: [integration-migration-disposition.md](../assets/integration-migration-disposition.md)

## Notes

- Physical position of the storage tests may need to move; this should be folded into each storage file's disposition/reason in the output asset.

## Question

Graduates from [ticket 10](10-partition-codebase-audit.md).

The `tests/integration/` directory has 14 files / 93 tests across four
subdirs, plus `tests/llm/` (2 files / 2 tests). These survived the
component-tier dissolution (tickets 02–06 only dissolved
`tests/integration/application/` + `flow/`). Each remaining file needs
a disposition: **port down to unit**, **port up to HTTP E2E + spec**,
**stay as integration**, or **delete**.

Files to classify:

- `tests/integration/adapters/driven/llm/llm_client.rs` — LLM adapter integration
- `tests/integration/bootstrap/run_branches.rs` — bootstrap integration
- `tests/integration/model/{css,settings,world}.rs` — model-layer integration
- `tests/integration/storage/{preset,snapshot,llm_message,message,world,prompt_presets}_storage.rs` — driven-adapter seam (6 files)
- `tests/llm/*` — LLM E2E (2 tests, driven adapter)

Per STRATEGY.md, the storage rows depend on [ticket 16](16-storage-spec-decision.md)'s
decision (spec vs no spec). All other rows classify independently.

## Output

A single markdown asset (linked from this ticket) with one row per file:

| File | Tier today | Tests | HTTP-observable? | Disposition | Reason |

Disposition values: `port-to-unit`, `port-to-http-e2e`, `stay`, `delete`.

This ticket **maps only** — no moves, no spec edits. Moves execute on a
future effort (out of scope for this map).

## Out of scope

- Executing any migration — investigation only on this map.
- The storage spec decision itself — that's ticket 16.

## Answer

Classification complete. See the linked asset for the full disposition table.

Summary:
- **Storage seam (6 files, 99 tests)**: `stay` in the driven-adapter tier, spec-less per ticket 16.
- **LLM transport + bootstrap (2 files, 7 tests)**: `stay` as integration/driven-adapter/infrastructure seams.
- **HTTP-level tests mis-located in `integration/` (4 files, 25 tests)**: `port-to-http-e2e` to `tests/http/`:
  - `model/css.rs` → `tests/http/assets.rs` (or similar)
  - `model/settings.rs` → `tests/http/settings.rs` with new `settings.md` spec
  - `model/world.rs` → split: delete the 2 data-loading tests, move the 1 `/fragment/visual-sidebar` test to `tests/http/`
  - `storage/prompt_presets.rs` → `tests/http/prompt_presets.rs` with new `prompt_presets.md` spec
- **LLM E2E (2 files, 2 tests)**: `stay` as the separate real-LLM tier.

No files are classified `port-to-unit`. No files are classified `delete` at the file level; only the two redundant data-loading tests inside `model/world.rs` should be deleted as part of the split.

Execution of these moves is out of scope for this map (ticket 17 was investigation only).
