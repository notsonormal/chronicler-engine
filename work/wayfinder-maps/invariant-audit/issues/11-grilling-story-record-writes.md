# Decide the story record writes (tier 4)

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

What is the one verdict for the story records group, and what task tickets put it in place? A part of the group can get its own verdict when one mechanism does not fit it.

| Finding | Invariant that nothing enforces |
|---|---|
| storage-01 | A Message row always has a Swipe row. The insert of the Message and the insert of its first Swipe are two separate `Tx` calls. |
| storage-02 | A Swipe update finds its row. `set_swipe_text` and `set_swipe_state` ignore 0 affected rows. |
| storage-03 = story-02 | `active_swipe_index` names an existing Swipe. Three readers give three results: an error, an empty text, and a silent repair to 0. |
| story-01, storage-08 | `swipe_index` is `0..n` with no gap, and never negative. The schema has only `UNIQUE(message_id, swipe_index)`. |
| story-03 | A missing start state is an error. The read turns it into "this is the opening", so a Retry applies the movement twice. |
| story-04 | `kind` agrees with `message_type`. They are two independent fields. |
| story-05 | A Retry Swipe goes on the Message that is still last. Only the busy rule keeps this true, and the write does not check it. |

The candidate mechanisms, from the reports:

- One `Tx` write that inserts a Message with its first Swipe (enforcement preference 3).
- Row-count checks on the Swipe updates, and on `set_active_swipe` for the target row.
- `message_type` derived from `kind`, so only `kind` is stored (preference 1 or 2).
- `read_tail` returns an error when the Message before the last has no active Swipe state, and the silent repair to 0 goes (no read-side repair, map Notes).
- `CHECK` constraints on the two index columns. SQLite needs a table rebuild for that, and the v28 rebuild is the precedent.

## Context

- Findings in full: [the story report](../assets/story.md) and [the storage report](../assets/storage.md). The coordinator read the code of story-05 (`write_new_swipe`, `src/application/generation/gate.rs:924-941`) and found it safe today under the busy rule.
- These are only reachable by a code bug, a hand edit of the database or data from an older version. The owner named the story write as one of the fixes that need design (charting, Q1).
- [Decide and close the schema group](12-task-schema-group.md) waits for this ticket, so that one rebuild of the `messages` table serves both.
- The writes are in `gate.rs`, which `verify-gate` covers (map Notes).
- Use `/grilling` and `/domain-modeling`. Read the findings in the code again first (map Notes).

## Answer
