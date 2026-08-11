# Validate spec/tag consistency

Type: task (AFK)
Status: pending
Blocked by: 04, 05, 06, 07, 08, 09, 10, 11, 12

## Question

Run `scripts/validate_feature_spec.py` and fix any gaps, orphans, or formatting issues so the HTTP E2E spec surface is internally consistent.

## Work

1. Run `python scripts/validate_feature_spec.py` from the repo root.
2. If it reports gaps (declared scenario not covered by a test):
   - Add a covering test, or
   - Remove/adjust the scenario in the spec
3. If it reports orphans (tagged scenario not declared in any spec):
   - Add the scenario to the correct spec, or
   - Fix/remove the stale tag
4. Fix any broken spec formatting or links.
5. Verify `requires_migration/` is empty and remove it.
6. Run `cargo test --test http`.

## Output

- `scripts/validate_feature_spec.py` reports 0 gaps and 0 orphans.
- `requires_migration/` is gone.
- `cargo test --test http` passes.
