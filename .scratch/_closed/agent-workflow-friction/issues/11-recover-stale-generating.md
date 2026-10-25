# Recover a stale Generating from the live registry

Type: task
Status: resolved
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

## Answer

Resolved 2026-10-06. The page-visible generating answer comes from the live registry; the
persisted status is repaired by every entry point that touches the game.

### What changed

1. `generating_status_handler` answers from `GenerationGate::is_busy(current game)`: it reports
the persisted phase while a slot is live, and `idle` otherwise. The error-fragment branch is
unchanged. No write on the GET.
2. The stale-status heal is centralized in `claim_and_spawn`, so every action entry point
(`/action` free actions, `/options`, retrigger, retry) heals before it validates or claims. The
reset is persisted only when the status actually changed.
3. `switch_game_handler` heals the game it switches to through `ActionPipeline::heal_stale_status`.
4. `test_generating_status_variants` moved out of the quarantine into `tests/http/dashboard.rs`
as tagged scenario 39.1 of the new `docs/specs/dashboard.md` (HTTP-observable dashboard chrome):
stale `Generating` → `idle`, live slot → the persisted phase. `REQUIRES_MIGRATION_TEST_COUNT`
83 → 82.
5. `browser_dashboard.md` scenario 16.28 and its tier-2 test drive the poll from `narrating` to
`idle`, then press a real Enter on the command input.
6. `two-state-channels.md` repaired: "the atomic" is the registry, and the heal sites are named.

### Review and follow-ups

`/code-review` (Standards + Spec) found the heal+persist pair duplicated four ways; unconditional
snapshot writes on every entry point, including `ConcurrentGeneration` rejections and
validation-error paths; the live-`Quantifying` phase lost its only test; no entry point other
than narration had a heal-on-entry proof; one unit test discarded its `try_claim` result; and
`gate.rs`'s module summary drifted. Fixed in the same session: the heal is centralized in
`claim_and_spawn` and persisted only on change, the live-`Quantifying` case is restored, each
entry point (`/options`, retrigger, retry, switch) gained a heal-on-entry test, the `try_claim`
result is asserted, and the `gate.rs` summary is corrected.

### Arrival is not a writer of a stale record

Ticket 09's Decision 3 listed arrival among the writers. Arrival sets `Generating` in memory but
persists only after resolving to `Idle`/`Error`, so it never leaves a stale persisted `Generating`
and needs no heal coverage. Its separate missing-slot defect is ruled out of scope on the map.
