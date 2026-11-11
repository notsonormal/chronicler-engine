# Decide and close the schema group (tier 4)

Type: task (AFK, decide and do)
Status: open
Blocked by: 11

## Question

Give the one verdict for the schema group under the enforcement order of the map Notes, and put it in place. A part of the group can get its own verdict when one mechanism does not fit it.

| Finding | Invariant that nothing enforces |
|---|---|
| storage-05 = world-10 | One `maps` row for each World. No `UNIQUE` on `maps.world_id`. |
| storage-06 | The positional reads of `DbGame::from_row` (18 columns) match three copied `SELECT` lists. A swap of two TEXT columns is silent. |
| storage-09 | `messages.game_id` is always written. The column has `DEFAULT 1`, so an insert that leaves it out lands in Game 1. |
| storage-11 | The settings row exists. A missing row silently becomes `AppSettings::default()`, and the next save writes the defaults. |
| storage-12 | All datetime text is UTC RFC3339. The sort order depends on it, and each writer formats its own value. |
| storage-13 | `user_version` is at most `LATEST_VERSION`. An older binary runs on a newer database with no error. |

## Context

- Findings in full: [the storage report](../assets/storage.md), and world-10 in [the world report](../assets/world.md). Each finding there has a suggested enforcement point. The coordinator did not read these in the code.
- The Q5 rule applies: a migration that meets a row that breaks a new constraint fails loudly and names the row (map Notes).
- Dropping `DEFAULT 1` needs a table rebuild in SQLite. The v28 rebuild in `src/adapters/driven/storage/utils/plumbing.rs` is the precedent. Blocked by [Decide the story record writes](11-grilling-story-record-writes.md), so that one rebuild of `messages` serves both.
- The six end conditions of the map Notes apply.

## Answer
