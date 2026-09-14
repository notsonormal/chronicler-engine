# 15 — Grill: does `allowed_modes` cover preset ids pinned in a Swipe's stored inputs?

Type: grilling
Status: resolved
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

## Answer

Resolved 2026-09-06 by grilling (one round: Q1–Q3, revised Q2 confirmed).

**Settled decisions:**

1. **Drop the preset pin entirely** (Q1-A). `impersonate_preset_id` leaves
   the Swipe. Impersonate generation — first run and redo — resolves the
   game's current active impersonate preset at generation time; the
   fallback branch in `narration_generation.rs::resolve_preset_choice`
   becomes the only path. A redo after a preset change deliberately uses
   the NEW preset — the settled model: a preset is game configuration,
   not swipe data ("if we change the preset for a game then every new
   message uses the new preset"). This also removes the
   pinned-then-deleted-preset hard-fail on redo and the entry-time
   storage read in `action.rs` (the entry-path→preset coupling
   guided-generations ticket 03 flagged).
2. **The ticket's question is moot.** With no pinned id there is nothing
   to re-validate; ticket 13's rule ("`allowed_modes` gates selection
   surfaces only") then holds absolutely — generation always uses the
   active preset, which the selection surfaces keep mode-legal. Residual
   edge noted, not covered: shrinking a preset's `allowed_modes` while a
   game has that preset active leaves an active-but-disallowed preset.
   That edge predates this ticket and is independent of the pin (the pin
   bypassed flags entirely, so removing it narrows the hole). Ticket it
   only if it ever bites.
3. **The stored inputs flatten onto the Swipe** (Q2 revised in-session).
   The two surviving inputs become direct Swipe properties:
   `impersonated: bool` + `direction: Option<String>` — guide text and
   impersonate direction merge into one field (same kind of thing:
   player-typed steering for one generation; the bool is the
   discriminator). All four representable states are valid generations
   (plain / guided / bare impersonate / directed impersonate), so no
   illegal combos need guarding; one `is_guided()` helper
   (`!impersonated && direction.is_some()`) keeps the retry/anchor reads
   explicit. `GenerationReplay`, the `replay` field name, and the
   `replay` column leave the codebase; a migration converts old rows
   from the JSON, then drops the column. This completes
   guided-generations ticket 06's "replay blob is not a concept" ruling
   — the mechanism was kept then; its vestigial packaging goes now.
4. **Sequencing** (Q3-A): graduates to a task ticket scheduled before
   ticket 06 — ticket 06 threads game preset-ids through the same
   resolver, and landing this first leaves it a pin-free resolver to
   thread.

**Supersessions on record:** guided-generations ticket 03's entry-time-pin
rationale ("a redo must not silently use the new preset") is reversed —
using the new preset is the settled intent. Guided-generations ticket 06's
"data flow unchanged" clause is superseded (blob un-nests, preset id
drops); its "not a concept" stance is affirmed. **Behavior change for the
implementation commit to flag:** a redo of an impersonated swipe can now
produce output under a different preset than the swipe's original
generation.
