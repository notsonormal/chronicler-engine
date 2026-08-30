# 07 — Pre-merge handoff: landing shape and merge strategy

Type: grilling
Status: claimed
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
