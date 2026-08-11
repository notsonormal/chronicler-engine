# Map: assign leftover tests/http tests to specs

Labels: wayfinder:map

## Destination

`tests/http/` contains only test files whose behaviour is covered by a spec in `docs/specs/*.md`. Test files without specs are quarantined in `tests/http/requires_migration/`. The way is clear when every quarantined file has a spec, has been moved back to `tests/http/`, and its tests are tagged with `// [spec.md] SCENARIO: N.N` annotations, and `scripts/validate_feature_spec.py` reports 0 gaps and 0 orphans.

## Notes

- Domain: Rust HTTP E2E tests under `tests/http/`.
- Existing context: the [integration-tier-migration map](../integration-tier-migration/map.md) just moved the settings and prompt-presets tests to `tests/http/` with tags. This effort handles the rest of `tests/http/`, including pre-existing untagged tests and the newly migrated files if any scenarios were skipped.
- Related but distinct: [test-strategy-execution ticket 18](../test-strategy-execution/issues/18-missing-spec-audit.md) audits the whole HTTP E2E surface for missing specs from a code-coverage angle. This map is narrower: it starts from the tests we already have and decides where each belongs.
- Use `scripts/validate_feature_spec.py` to confirm tag/scenario consistency after the work lands.
- Work is split into one ticket per quarantined file; each ticket writes the spec(s), migrates the file, and tags its tests.
- A grilling ticket (#14) decides the standard spec intro/summary format before the file tickets start.
- Skills to consult: `/grilling` and `/domain-modeling` for spec naming and assignment decisions.

## Decisions so far

- [Catalog untagged tests/http tests and propose spec assignments](issues/01-catalog-untagged-http-tests.md) — scanned `tests/http/`; produced a catalog of 85 tagged, 83 untagged tests, with proposed assignments, 8 new spec candidates, and 2 exemption candidates. Migration gap check: 0 gaps.
- [Grill the proposed test-to-spec assignments](issues/02-grill-spec-assignments.md) — accepted all proposals: 11 tests assigned to existing specs, 8 new specs defined for 70 tests, 2 server-wiring tests exempted. Catalog complete; assignments grilled.
- [Quarantine tests/http files without specs](issues/03-quarantine-specless-http-tests.md) — moved 9 test files and the `endpoints/` subtree into `tests/http/requires_migration/`; updated `mod.rs`; 168 tests pass.


## Not yet specified

<!-- none yet -->

## Out of scope

- Domain, application, or storage unit tests.
- Browser tests, unless they surface as part of an HTTP-test exemption discussion.
- Production-code changes beyond minimal test wiring.
