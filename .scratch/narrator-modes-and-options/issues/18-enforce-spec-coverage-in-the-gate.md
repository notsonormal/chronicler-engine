# Task: Enforce spec coverage in the gate

Type: task
Status: resolved
Blocked by: (none)

## Question

The spec-coverage guardrail (`scripts/validate_feature_spec.py`) is not wired into the build gate — `build.py` invokes `check_test_structure.py` (line 358) but never the feature-spec validator, so orphan tags and uncovered scenarios are caught only when someone runs it manually. On top of that, nothing requires behavioral HTTP E2E tests to carry a SCENARIO tag at all: the ticket-17 follow-up found two free-loading tests (a re-impersonate redo test and a prompt_presets activation-refusal test) and fixed them, but only by hand.

### Scope

1. Wire `validate_feature_spec.py` into `build.py`, next to the test-structure guardrail.
2. Extend the validator: every test under `tests/http/` + `tests/browser/behaviour.rs` must carry a `// [spec-path] SCENARIO: N.N` tag or live inside a declared exemption constant in the script (`tests/http/requires_migration/**`, `tests/browser/invariants.rs`).
3. Ratchet the legacy quarantine: pin `tests/http/requires_migration/**`'s test count (83 at writing); the validator accepts only decreases. Migration cleanups lower the number deliberately.
4. Document the rule and the legacy bucket in `tests/STRATEGY.md` under "SCENARIO tags"; regenerate `guardrails.md` if STRATEGY.md is one of its sources.

### Notes-for-the-session

- Audit at writing: 178 tests in `tests/http/`, 93 SCENARIO tags. Outside `requires_migration/` only two tests were untagged (both fixed in the ticket-17 follow-up); `tests/browser/behaviour.rs` is fully tagged; `invariants.rs` already has its named exemption in STRATEGY.md.
- `tests/http/requires_migration/` holds 83 untagged e2e tests across 10 files and is mentioned nowhere in STRATEGY.md — an undocumented quarantine.
- The count ratchet closes the exemption's exit-hatch risk: a new test sneaked into the folder fails the ratchet even though the exemption spares it the tag rule.
- Known accepted edge: adding one test while deleting one inside the folder keeps the count flat — review-visible, not worth mechanical detection.
- Design ruling from the user: enforcement must be mechanical ("we would be forced to follow it"); a social STRATEGY.md rule alone was rejected.

## Answer

Shipped 2026-09-12. The guardrail is mechanical and wired into the gate.

- **Gate wiring** — `build.py` gains the `spec-coverage` step (`python scripts/validate_feature_spec.py`), ordered right after `test-structure` (runs as step [5/15] of the full gate). `scripts/tests/test_build_cli.py` updated: `EXPECTED_COMMANDS` pins the new command; the full-gate step-count assertions moved 14→15 and 13→14.
- **Mandatory tag rule** — every `#[test]`/`#[tokio::test]` under `tests/http/` and in `tests/browser/behaviour.rs` must carry a `// [spec] SCENARIO: N.N` tag. Violations list file:line + fn name and exit 1.
- **Declared exemptions** (constants in the script, each with its reason): `TAG_EXEMPT_DIRS` = `tests/http/requires_migration/` (quarantine); `TAG_EXEMPT_FILES` = `tests/browser/invariants.rs` (STRATEGY.md named exemption); `TAG_EXEMPT_TESTS` = `behaviour.rs::test_engine_output_teed_to_file` — the audit's only untagged test; its own comment already declared it an infrastructure health check (engine stdout tee), not a spec scenario.
- **Quarantine ratchet** — `REQUIRES_MIGRATION_TEST_COUNT = 86` pins the untagged count; the gate accepts only decreases. The ticket said 83 at writing; ticket-11-era commits raised it to 86 before this ticket ran, so the pin is the observed 86.
- **STRATEGY.md** — the "SCENARIO tags" section rewritten: enforcement is no longer a social convention; the three mechanical rules (coverage, mandatory tags, quarantine ratchet) are documented with the exemption mechanics. `guardrails.md` was NOT regenerated — STRATEGY.md is not a source of `generate_guardrails_doc.py` (its sources: `src/lib.rs` clippy lints, `arch-lint.toml`, guardrails test doc comments).
- **Verification** — positive: validator reports 140 declared / 140 covered / 0 untagged / quarantine 86/86, exit 0; `python build.py spec-coverage` OK; py-tests 106 OK. Negative: an injected untagged test file under `tests/http/` fails with `1 untagged` + file:line (removed after); the pin temporarily lowered to 85 fails with "Quarantine ratchet exceeded: 86 > 85" (reverted to 86, clean re-run). Full gate green: `nextest: 1614 passed, 0 failed, 2 skipped` (logs/build_20260912_211931.log).
- **Known accepted edge** (unchanged): +1/−1 inside the quarantine keeps the count flat; review-visible only.
- **Follow-up slim-down (same day, user review pass)**: module docstring 34→20 lines — the tag-rule contract's single home is now the exemption-constants block and the coverage-keying note's single home is the `main()` comment; STRATEGY.md 118→95 lines ("What was dissolved" compressed to one sentence in the tier section, "Placement test" cut as a table restatement, SCENARIO tags deduplicated); dissolved `tests/integration/` path references replaced with `tests/storage/`; the word "ratchet" replaced with pinned-count wording per the technical-writing skill (code failure message aligned to "Quarantine count exceeded"). Full gate green after (logs/build_20260912_213811.log).
