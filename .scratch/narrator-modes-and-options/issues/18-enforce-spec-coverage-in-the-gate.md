# Task: Enforce spec coverage in the gate

Type: task
Status: pending
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
