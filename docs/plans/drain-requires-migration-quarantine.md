# Plan: Drain the `requires_migration/` quarantine

**Date:** 2026-10-25
**Status:** Draft — not started
**Goal:** Every HTTP test carries a `SCENARIO` tag or a declared exemption. The folder `tests/http/requires_migration/` and its pin are gone.

## Origin

The map "assign leftover tests/http tests to specs" ([`.scratch/_closed/http-test-spec-mapping/map.md`](../../.scratch/_closed/http-test-spec-mapping/map.md)) catalogued the untagged HTTP tests, assigned each to a spec, and moved them into the quarantine (tickets 01–03). Its file tickets (04–14) went stale: they plan specs that were never written, and the 36 specs of today were written without them. The map closed on 2026-10-25 and this plan replaces its open tickets.

## Current state (known, 2026-10-25)

- `tests/http/requires_migration/` holds 72 untagged tests in nine files. `tests/http/mod.rs` declares `mod requires_migration;`.

| File | Tests |
|---|---|
| `fragment.rs` | 26 |
| `worlds_fragment_handlers.rs` | 15 |
| `connections.rs` | 14 |
| `text_check.rs` | 6 |
| `debug.rs` | 4 |
| `games_fragment_handlers.rs` | 3 |
| `server_impl_wiring.rs` | 2 |
| `core.rs` | 1 |
| `index_handler.rs` | 1 |

- `scripts/validate_feature_spec.py` exempts the folder (`TAG_EXEMPT_DIRS`) and pins its untagged count (`REQUIRES_MIGRATION_TEST_COUNT = 72`). The `spec-coverage` gate step fails when the count differs from the pin. `tests/STRATEGY.md` documents the quarantine and the pin.
- Outside the quarantine, the validator reports 0 gaps and 0 orphans.
- A file with no spec surface is exempted by name in `TAG_EXEMPT_FILES`, with a reason (for example `tests/browser/stub/layout.rs`).
- The catalogue of ticket 01 (`.scratch/_closed/http-test-spec-mapping/assets/http-test-catalog.md`) proposes a spec for each test. Its spec names predate the current specs. Use it as a hint, not as a decision.

## Inferred

- Some quarantined tests repeat tagged tests, for example the `test_switch_swipe_handler_*` tests next to `docs/specs/swipe_switch.md`. Nobody has compared them.
- `connections.rs`, `debug.rs` and `text_check.rs` have no matching spec today. `settings.md` may cover part of `connections.rs`.

## Rule for each test

Take one test at a time and apply the first rule that fits:

1. **Duplicate.** A tagged test already observes the same behaviour through HTTP. Delete the quarantined test.
2. **Spec behaviour.** The test observes a client-visible rule that a spec states, or should state. Add or reuse a scenario in an existing spec, tag the test, and move it to the matching file under `tests/http/`. Create a new spec only when no existing spec owns the subject.
3. **No spec surface.** The test checks wiring or markup that no scenario can state (for example `server_impl_wiring.rs`). Move it and add a `TAG_EXEMPT_FILES` entry with the reason.
4. **Wrong tier.** The test checks a rule inside one part. Move it to a unit test next to the code, or delete it if a unit test already covers the rule.

Spec scenarios follow `docs/AGENTS.md` ("Where a fact lives"): a scenario states an event rule at feature level, not a value or copy text.

## Steps

1. Work one file per change, smallest first: `core.rs`, `index_handler.rs`, `server_impl_wiring.rs`, `games_fragment_handlers.rs`, `debug.rs`, `text_check.rs`, `connections.rs`, `worlds_fragment_handlers.rs`, `fragment.rs`.
2. For each file, apply the rule to each test. Lower `REQUIRES_MIGRATION_TEST_COUNT` by the number of tests that left the folder. Delete the file when it is empty.
3. Check each file with `python build.py test-pattern "<pattern>" --test-target http` and `python scripts/validate_feature_spec.py`.
4. When the folder is empty: delete it, delete `mod requires_migration;`, the `TAG_EXEMPT_DIRS` entry, the pin and its check in `validate_feature_spec.py`, and the quarantine row and pin rule in `tests/STRATEGY.md`. Regenerate `tests/AGENTS.md`.
5. Run `python build.py --no-browser`, then the full gate once.

## Done when

- `tests/http/requires_migration/` does not exist.
- `validate_feature_spec.py` has no quarantine exemption and no pin.
- `spec-coverage` reports 0 gaps and 0 orphans.
- `python build.py` passes.

## Open questions

- Does a test of `GET /debug/state` belong in a spec, or is the debug route an exemption? Decide with the owner at `debug.rs`.
- Does `connections.rs` need its own `connections.md` spec, or do its scenarios belong in `settings.md`?
