# Recover a dashboard stuck on Generating with no live generation

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

A persisted `GenerationStatus::Generating` with no live generation slot leaves the dashboard
stuck, and the engine's own recovery contract cannot reach it.

The contract is documented in `docs/diataxis/explanation/two-state-channels.md`: after a
mid-flight crash the persisted status still says `Generating` while the process-local registry
holds no slot, and "on the next `process_action` ... the engine resets the persisted status to
`Idle`". `GenerationGate::heal_stale` implements that reset, and the action path
(`action_pipeline/action.rs`) calls it.

The page blocks the healing action. The status poll reports the persisted phase,
`onStatusPoll` maps it to `.status.thinking`, `statusIsGenerating()` returns true, and
`applyActionState` disables `#submit-btn`. The input stays enabled, but the submit button is
the form's default button, and Enter does not submit through a disabled default button — A19's
evidence for the swipe-restore bug recorded a real CDP Enter producing no request in exactly
this state. So a panicked generation (`GenerationGuard::Drop` frees the slot but leaves the
persisted status) strands the dashboard with no in-page way out; a page reload does not clear
it and only a server restart does.

The swipe-restore trigger was fixed by normalising the snapshot in `MessageService::switch_swipe`
(ticket 43). That is a per-trigger fix. This ticket decides the class-level recovery mechanism.

Options to weigh:

- **Heal in the status poll.** `generating_status_handler` calls `heal_stale` and persists when
  the registry holds no slot. Self-healing regardless of trigger, but it contradicts the
  documented single-writer rule ("the persisted status's `true` transitions are owned solely by
  the action path") and turns a GET into a writer.
- **Let the client submit while generating.** Keep the button disabled as a generating
  indicator, but let the command form submit anyway and let the server answer
  `ConcurrentGeneration`. Matches the documented contract and keeps the single writer, but
  re-opens the mid-generation submit that ticket 42 and 43's 6.6 deliberately closed.
- **A recovery control.** Show a control while the status has been generating implausibly long,
  wired to `/status/reset-generating`. Explicit and self-contained, but adds a new client state
  and a watchdog.

## Context

- From [Close the browser-test gap for stuck DOM states](04-browser-test-gap-stuck-dom-states.md)
  and its asset [dom-stuck-state-test-gap.md](../assets/dom-stuck-state-test-gap.md), which
  re-checked every stuck-state bug in the A19 window.
- The class was left open by [Reset the generation status and the options dock when a swipe
  switch restores a snapshot](../../dashboard-ui-review/issues/43-reset-status-and-dock-on-swipe-switch.md);
  its normalisation covers the swipe path only, and the dashboard-ui-review map records the
  other restore paths as unverified.
- The recovery wiring has no integration test today — only
  `gate_tests::test_heal_stale_resets_generating_status_when_no_active_slot`. `retrigger` and
  `retry` were traced during the research and do not reproduce a stuck status on their own.
- `/status/reset-generating` exists but no UI control reaches it; it is used only by test
  helpers.

## Done when

- The recovery mechanism is decided and recorded, with the rejected alternatives and their
  one-line cost.
- A tier-1 test proves a stale persisted `Generating` with no live slot is healed by the next
  action.
- A tier-2 or tier-3 test proves a page reading "Generating..." can reach the recovery, or the
  decision records why it cannot and what the player is told instead.
- Any implementation the decision adopts is graduated into its own ticket on this map.
- `python build.py` is green for any change made here.

## Answer

Resolved 2026-10-06. The class-level recovery: **the page-visible generating answer
comes from the live generation registry, not the persisted record.** `GET /status/generating`
answers a phase only while the current game holds a generating slot, and `idle` otherwise.
The GET stays read-only; the persisted status is still healed by the action path.

### Decisions

1. The submit button stays disabled while a generation is live. Recovery removes the stale
   status, not the double-submit guard ticket 42/43 built.
2. Recovery is silent — no control, no message. A stale status is an artifact, not something
   the player manages.
3. Coverage must hold for every writer: narration, options, retrigger, retry, arrival and
   game switch — not just the reproduced narration path.
4. No source-level prevention. `GenerationGuard::Drop` stays registry-only; the pipeline stays
   the sole persister. The state stays recoverable rather than prevented, and `SIGKILL` is
   already covered by the boot heal in `bootstrap/wiring.rs`.
5. `heal_stale` extends to `/options`, retrigger, retry and game switch, so any request that
   touches a game repairs a stale record.
6. `docs/diataxis/explanation/two-state-channels.md` is repaired: "the atomic" becomes the
   registry, and "on the next `process_action`" is scoped to the paths that actually heal.
7. Acceptance bar follows `tests/STRATEGY.md`'s placement rule: the poll's answer is
   `curl`-observable, so tier 1; the client's reaction is pure-client, so tier 2, and the
   client half presses a real Enter because no test ever has and `requestSubmit()` does not
   reproduce the disabled-default-button block A19 recorded.
8. One implementation ticket carries both halves.

### Rejected alternatives, with cost

- **Poll heals and persists.** Cost: a GET becomes a writer, breaking the documented
  single-writer rule; and the heal can race a generation start, since the persisted
  `Generating` is written before `try_claim` claims the slot, so a just-started turn's status
  could be reset live.
- **Poll reports a `stale` marker; the client auto-POSTs `/status/reset-generating`.** Cost:
  a new client state, and a production caller for a test-only endpoint.
- **Let the client submit while generating; the server answers `ConcurrentGeneration`.**
  Cost: re-opens the mid-generation submit 42/43 closed; the rejected answer renders a
  `.status.wait` span that has no CSS rule and no test.
- **An explicit recovery control with a watchdog.** Cost: a new client state and a timeout,
  for a state the player should never have to manage.
- **Fix the panic path so it cannot strand.** Cost: `Drop` is synchronous and deliberately
  owns no persistence, and it cannot cover `SIGKILL`, which boot heal already handles.
- **A tier-3 end-to-end test.** Cost: no mechanism exists to seed a stale persisted status
  against a live server — no mutating debug route, and the engine holds its SQLite file open
  with the repo warning against a second writer. The placement rule also files the client half
  down to tier 2.

### Facts the decision rests on (re-checked 2026-10-06)

- `generating_status_handler` is read-only and registry-blind, but `AppState` owns
  `generation_gate`, so the registry is already reachable from the handler.
- `heal_stale` has exactly two `src/` call sites: boot (`wiring.rs`) and the free-action path
  (`action.rs`, `FreeAction`/`Guide`/`Impersonate` only). `/options`, retrigger and retry never
  call it, and there is no heal on game switch.
- `GenerationGuard::Drop` frees only the registry slot; `gate.rs` states the persisted status
  is reset separately so the gate owns no persistence.
- `TestAppBuilder::generation_status(Generating, phase)` seeds a persisted `Generating` with
  **no** live slot, after boot — the tier-1 seam. `is_generating(true)` is the slot-claiming
  sibling.
- `tests/http/requires_migration/fragment.rs::test_generating_status_variants` asserts
  `narrating` for a persisted `Generating` with no slot, i.e. the pre-decision contract; it
  must flip. `REQUIRES_MIGRATION_TEST_COUNT` is 83.
- No test presses a real Enter to submit the command form; the helpers use `requestSubmit()`
  or `click()`.

### Graduated

- [Recover a stale Generating from the live registry](11-recover-stale-generating.md)
- [Decide the fate of the invariant test the recovery doc claims](12-doc-claimed-invariant-test.md)
