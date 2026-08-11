# Storage integration spec decision

Type: grilling (HITL)
Status: resolved
Blocked by: 10 (claimed → resolving)

## Question

Graduates from [ticket 10](10-partition-codebase-audit.md).

STRATEGY.md currently says driven-adapter tests cover "the storage seam —
CRUD, error handling, referential integrity, query correctness" and that
specs live at HTTP E2E + browser only. Under that rule, the 6 files in
`tests/integration/storage/` (`preset_storage.rs`, `snapshot_storage.rs`,
`llm_message_storage.rs`, `message_storage.rs`, `world_storage.rs`,
`prompt_presets.rs` — 93 tests total across the integration tier) do not
get specs; the test code is the definition.

This ticket decides:

- Does the storage seam get specs, or stay spec-less per STRATEGY.md?
- If spec-less: is "test code is the definition" sufficient, or does the
  strategy need an explicit exemption clause (like `invariants.rs` got
  in ticket 07)?
- If specs: what form — one spec per storage table, or one combined
  `storage.md`?

This is a **decision**, not a classification — it blocks ticket 17
(integration migration) because the storage rows of 17's disposition
table depend on the answer. Storage is ~6 of the 14 integration files,
so the decision shapes most of 17's work.

## Answer

Storage integration tests stay **spec-less** per `STRATEGY.md`.

- No `docs/specs/` file for the storage seam.
- `STRATEGY.md` should add an explicit exemption clause for driven-adapter storage tests: the test code is the definition, equivalent to `tests/browser/invariants.rs`.
- If the seam later needs a human-readable contract, write one spec; per-table specs are overkill.

Physical position of storage tests is deferred to ticket 17 (user input).

## Out of scope

- Migrating any storage tests — that's ticket 17.
- Changing STRATEGY.md tier rules — they're settled on this map.
