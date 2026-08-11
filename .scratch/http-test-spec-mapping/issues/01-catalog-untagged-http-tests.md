Status: resolved

# Catalog untagged tests/http tests and propose spec assignments

Type: research (AFK)

## Question

Scan every `*.rs` file under `tests/http/` and list every test function (both whole files and individual methods) that is not tagged with a `// [docs/specs/<spec>.md] SCENARIO: N.N` annotation. For each untagged test, decide which existing spec scenario it should cover, or propose a new spec title and scenario if none fit.

Scope includes the newly migrated `settings.rs` and `prompt_presets.rs` files: if any spec scenarios from `docs/specs/settings.md` or `docs/specs/prompt_presets.md` are not yet covered by a tagged test, those are also gaps to record.

## Output

A markdown asset linked from this ticket with three sections:

1. **Covered tests** — test file/function and the spec/scenario it already tags.
2. **Assignable tests** — untagged tests that map cleanly to an existing spec scenario (one row per test: file, function, proposed spec, proposed scenario ID or new scenario title).
3. **New-spec candidates** — untagged tests whose behaviour is HTTP-observable but not described by any current spec. Propose a spec filename, a one-line purpose, and the list of tests that belong there.
4. **Exemption candidates** — tests that should not carry a SCENARIO tag (e.g. pure wiring or harness tests). Explain why each is exempt.

No spec edits, no new tests, no grilling in this ticket — this is the catalog and proposal only.

## Answer

Scanned all `*.rs` files under `tests/http/`. Full catalog is in the linked asset:

[.scratch/http-test-spec-mapping/assets/http-test-catalog.md](.scratch/http-test-spec-mapping/assets/http-test-catalog.md)

Summary:

- **168 test functions** in `tests/http/`.
- **85 already tagged** with `// [spec.md] SCENARIO: X.Y`.
- **83 untagged**.
- `validate_feature_spec.py` reports **92 declared / 92 covered / 0 gaps / 0 orphans**, so the newly migrated `settings.rs` and `prompt_presets.rs` already cover every scenario in their specs.
- Assignable to existing specs (with proposed new scenarios): **11 tests** across `actions.md`, `games_create.md`, `reset.md`, `story_log.md`.
- New-spec candidates: **8 specs** (`connections.md`, `debug.md`, `text_check.md`, `swipe_switch.md`, `status.md`, `dashboard_fragments.md`, `games_list.md`, `worlds.md`) covering **70 tests**.
- Exemption candidates: **2 tests** in `server_impl_wiring.rs` (server binding tests, not endpoint behaviour).

No specs or tests were edited.

## Out of scope

- Editing specs or writing tests.
- Deciding the final assignments (that is ticket 02).
- Browser, unit, storage, or driven-adapter tests.
