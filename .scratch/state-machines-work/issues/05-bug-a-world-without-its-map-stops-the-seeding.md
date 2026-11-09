# A World seed file with a missing or bad map stops the seeding of the other Worlds and of all Personas

Status: needs-triage
Source: ticket 02 of the state-machine map, the second scan (scan 2, 2026-11-09): `scan2-driven.md`, *Seed files vs database* ([catalogue](../../state-machines-next/assets/catalogue.md), row E3)

## The defect

`process_world_dir` (`src/bootstrap/load.rs:92`) skips a World whose `world.json` does not parse, with a warning (`:102-110`). But a missing or malformed map file returns an error: `read_json_file(&map_path)?` (`:122`). `seed_worlds` passes that error up with `?` (`:87`), so the loop stops, and the Worlds after it in the directory order are not seeded. `seed_game_data` passes it up again (`:34`) before `seed_personas` runs (`:37-40`), so no Persona is seeded either. The caller only logs a warning (`src/bootstrap/run.rs:58-60`).

This contradicts `docs/diataxis/reference/startup.md:26`: "A malformed world, persona, or preset file is skipped without halting bootstrap." Here one bad file is not skipped alone: it also drops the files after it.

## Impact (inferred)

Medium on a first run, low later. On a fresh database, the boot then fails at `require_persona` (`run.rs:86`) for a reason that only an earlier warning names. After the first run, the stored Worlds and Personas already exist, so only a newly added World or Persona file is lost.

## Done when

- A World with a missing or malformed map file is skipped with a warning, and the other Worlds and the Personas are still seeded.
- A test seeds a data folder with one World whose map file is missing, and finds the other World and the Personas in the database.
