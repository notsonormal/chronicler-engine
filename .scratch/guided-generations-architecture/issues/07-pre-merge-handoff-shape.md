# 07 — Pre-merge handoff: landing shape and merge strategy

Type: grilling
Status: resolved
Blocked by: 01, 02, 03, 04, 05 (all resolved — unblocked)

## Question

With every candidate decided — 01, 03, 05 committed; 02, 04 rejected — how
does the pre-merge execution land, and what (if anything) gates the merge of
`guided-generations`?

## Background

The accept/reject split is now known, which graduates both of the map's
Not-yet-specified patches:

1. **Coordinated vs independent landing.** Do the three accepted deepenings
   (01's `narration_generation` module, 03's `process_action` dispatcher,
   05's voice-ownership cleanup) land as one coordinated pre-merge refactor,
   or as independent commits?
2. **Merge strategy given the rejections.** 02 and 04 were rejected, so
   those shallows stay as-is. Does the branch merge with them in place (with
   or without a follow-up issue), or does anything block?

## What to decide

- **Landing shape.** One coordinated refactor vs independent commits — and
  the order if independent. Note that 02's and 04's rejections left
  execution notes that fold into 01's work (the `pending_replay` staging
  buffer's removal; merging the two preset loaders), so 01 is the
  gravitational center either way.
- **Merge gating.** What must be green/landed before merge; whether the
  rejected-candidate shallows carry a follow-up issue.
- **Sequencing against the narrator-modes effort.** 05's execution is
  deliberately scoped to survive narrator-modes ticket 06 (pending on that
  map, with this map's note appended). Decide whether 05's cleanup lands
  without waiting for 06, and whether the branch merges with the
  narrator-modes implementation partially complete (its tickets 06–12).

## Answer

Resolved 2026-08-30 by grilling (rounds 1–2: Q1–Q4).

**Settled decisions:**

1. **Landing shape: one single commit** (Q1). All four execution pieces —
   the Narrator Action removal (ticket 06's execution scope), 05's
   voice-ownership cleanup, 03's `process_action` dispatcher, and 01's
   `narration_generation` module (absorbing 02's `pending_replay` removal
   and 04's preset-loader merge) — land as one commit on
   `guided-generations`. The user's decision, taken over the grill's
   recommendation of sequenced independent commits; the disagreement is on
   record (a single commit mixes three refactors with one deliberate
   behavior change, weakening bisection). Consequences: 03's temporary
   `Narrator` arm is moot — the removal rides the same commit; the commit
   message must flag the one deliberate behavior change inside the
   refactor — 01's decision 3, impersonate redo gains the full tail
   (quantifier → engine commit → trigger) — so blame and bisection
   readers can find it.
2. **Merge gating: `python build.py` green** (Q2), with the execution
   commit landed. No follow-up issue for the rejected shallows of 02 and
   04 (Q4a) — the rejections are the chosen shape, not debt, and the
   reasoning lives in the tickets and this map's index.
3. **Sequencing: this work lands first; the narrator-modes effort resumes
   after the merge** (Q3). 05's cleanup does not wait for narrator-modes
   ticket 06. Accepted consequence: merged main carries a partially live
   narrator-modes surface — mode-filtered preset selection already works
   via its landed ticket 14; the game posture fields stay write-only
   until its ticket 06 threads them. The settled order was appended to
   narrator-modes ticket 06's comments.
4. **The `allowed_modes` generation-time edge** (Q4b): recorded as
   [narrator-modes ticket 15 — `allowed_modes` vs preset ids pinned in a
   Swipe's stored inputs](../../narrator-modes-and-options/issues/15-grill-allowed-modes-pinned-preset-edge.md).
   Framed against that map's ticket 13, which settled "`allowed_modes`
   gates selection surfaces only" — the new ticket asks whether the rule
   intends to cover a preset id pinned into a Swipe's stored inputs whose
   modes later shrink. It does not reopen the general rule.

**Cross-map consequence (noted, not actioned):** the narrator-modes map's
destination point 3 and Out-of-scope section still enumerate
"narrator-action" as a feature that stays available. The pre-merge commit
removes Narrator Action entirely (ticket 06). The narrator-modes effort
should reconcile those references when it resumes; a note was appended to
its ticket 06.

**Consequences for the map:** all seven tickets are resolved. The
destination is met — every candidate is decided (committed: 01, 03, 05;
rejected: 02, 04; model revised: 06) and the handoff shape is settled.
The map is complete; the next effort is pre-merge execution.
