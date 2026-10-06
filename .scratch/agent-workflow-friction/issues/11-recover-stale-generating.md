# Recover a stale Generating from the live registry

Type: task
Status: open
Blocked by: —

## Work

Graduated from [Recover a dashboard stuck on Generating with no live generation](09-recover-stuck-generating.md).

1. `generating_status_handler` answers from the live registry: report the persisted phase while
   the current game holds a generating slot, and `idle` when it does not. No write on the GET.
2. `heal_stale` runs at `/options`, retrigger, retry and game switch, each persisting the healed
   state. Today `/options` early-returns before the heal closure, and retrigger and retry only
   validate.
3. Rewrite `tests/http/requires_migration/fragment.rs::test_generating_status_variants` as a
   tagged test against a new `docs/specs/dashboard.md` — the HTTP-observable dashboard-chrome
   spec, mirroring `browser_dashboard.md`. The stale case expects `idle`; the phase assertion
   moves to the slot-claiming `is_generating(true)` helper. Lower
   `REQUIRES_MIGRATION_TEST_COUNT` in `scripts/validate_feature_spec.py` from 83 to 82.
4. Add a `browser_dashboard.md` scenario and a tier-2 test in `tests/browser/stub/dashboard.rs`:
   script the poll from `Phase("narrating")` to `Idle`, assert `#status-display` leaves
   generating and `#submit-btn` re-enables, then press a real Enter on
   `#command-form input[name="command"]` and assert a request fires.
5. Repair `docs/diataxis/explanation/two-state-channels.md`: "the atomic" becomes the registry,
   and "on the next `process_action`" is scoped to the paths that heal.

## Done when

- `GET /status/generating` answers `idle` for a persisted `Generating` with no live slot, and the
  phase while a slot is generating, proven at tier 1.
- A tier-2 test proves a page that read "Generating..." becomes actionable and a real Enter
  submits.
- `heal_stale` runs at `/options`, retrigger, retry and game switch.
- `python build.py` is green.
