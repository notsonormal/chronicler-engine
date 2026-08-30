# 15 — Grill: does `allowed_modes` cover preset ids pinned in a Swipe's stored inputs?

Type: grilling
Status: pending
Blocked by: (none)

## Question

Ticket 13 settled: `allowed_modes` gates selection surfaces only (per-game
picker, mode-switch retarget, activation). Does that rule intend to cover a
preset id that was pinned into a Swipe's stored generation inputs — or
should a redo re-validate the pin against the preset's current flags?

## Background

Handed off from [guided-generations-architecture ticket 07 —
Pre-merge handoff](../../guided-generations-architecture/issues/07-pre-merge-handoff-shape.md)
(resolved 2026-08-30), surfacing an observation from that map's ticket 04
grilling:

A preset's `allowed_modes` is checked when a user selects the preset over
HTTP, not when a generation uses it. The guided-generations branch pins the
active impersonate preset id into the Swipe's stored inputs at entry time
(the `process_action` dispatcher, per that map's ticket 03 — required
because a Swipe stores the inputs that produced it, so a redo after the
user changes the active preset must not silently use the new preset). If
the pinned preset's `allowed_modes` later shrink to exclude the game's
mode, a redo of that swipe still generates with the now-disallowed preset.

Ticket 13 decided selection-surface gating as a general rule; it is not
recorded whether the pinned-id edge was considered. This ticket asks the
edge case only — it does not reopen the general rule.

## What to decide

- Is redo-with-a-pinned-now-disallowed-preset intended under ticket 13's
  rule (the pin is historical fact about what produced the swipe), or a
  hole (the game's current mode should govern what generates)?
- If a hole: where enforcement lives — at redo time (re-validate the
  pinned id before generation, with what fallback: active preset, or
  refuse?), or at flag-change time (refuse to shrink modes while pins
  reference the preset — likely unworkable, pins are per-swipe data).
