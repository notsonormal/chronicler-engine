# Enforce preset name uniqueness in storage

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Ticket 28 refuses a duplicate preset name in `PromptPresetService::save_preset`, comparing trimmed and case-insensitively within one preset category. Two gaps remain: the check lists siblings and then writes under two separate lock acquisitions, so two concurrent creates can both pass; and `Storage::save_preset` (used by startup seeding) is unvalidated, so seeding could still introduce a duplicate. Can storage make check-and-write one step on both backends, without breaking existing databases that may already hold duplicates?

## Context

- From [Decide whether duplicate connection and preset names are allowed](28-decide-duplicate-names.md) and its review, and the round-2 follow-up [ticket 37](37-follow-up-small-review-issues-2.md). Connections already check inside one `update_settings` lock.
- Precedent: tickets 29 and 35 made the `worlds.key` UNIQUE constraint authoritative on SQLite. Here a plain `UNIQUE` would fail on old databases that hold duplicates, so a single-lock check-and-write may be the safer route.
- Shared scan: `name_is_available` in `src/application/utils/name_uniqueness.rs`.

## Done when

- Both backends refuse a same-category duplicate in one step, with a tier-1 storage pair (InMemory/SQLite, identical apart from construction).
- Existing databases with duplicates still start.
- `python build.py` is green.

## Answer

Yes — check-and-write is now one step on both backends, with no `UNIQUE` constraint. `Storage::save_preset` runs `ensure_preset_name_available` and `write_preset` inside the same `with_backend_mut` acquisition, so a duplicate is refused in-process with no TOCTOU gap. `PromptPresetService::save_preset` no longer lists siblings; it delegates to storage and converts `EngineError::Validation` to `ApplicationError::Validation`, so the user path and the startup-seeding path share one check. A `UNIQUE` constraint was rejected because legacy databases holding duplicates must still open; the check is a scan, proven by a raw-insert test. (`ensure_presets` failures are already caught and logged at `src/bootstrap/run.rs`, so a seed that would duplicate aborts seeding, not startup.)

**Scope note.** `arch-lint.toml` bans `storage → application`, so storage could not import `application::utils::name_is_available`. The predicate moved to `src/domain/model/utils/name_uniqueness.rs` (storage may depend on domain), with `application/utils/mod.rs` re-exporting it so the two service import sites are unchanged. That is two domain files beyond the ticket's file list.

Tests (driven-adapter tier, `tests/storage/preset_storage.rs` — `tests/STRATEGY.md` places the storage seam there): `test_save_preset_refuses_duplicate_name_in_same_category` (InMemory) and `test_sqlite_save_preset_refuses_duplicate_name_in_same_category` (SQLite), identical apart from construction, each asserting `EngineError::Validation` and that nothing was written; plus `test_sqlite_opens_database_holding_preexisting_duplicate_names`, which raw-inserts duplicates and reads both back. No spec change was needed.

Full gate green (`build_20261003_200736.log`): 16/16 steps, arch 1, guardrails 137, integration 1531 (2 skipped), browser 31. No commit made.
