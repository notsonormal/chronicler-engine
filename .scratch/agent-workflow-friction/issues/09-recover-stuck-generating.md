# Recover a dashboard stuck on Generating with no live generation

Type: grilling (HITL)
Status: open
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
