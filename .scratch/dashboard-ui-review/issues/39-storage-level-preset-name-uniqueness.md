# Enforce preset name uniqueness in storage

Type: task (AFK)
Status: open
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
