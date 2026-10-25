# Plan: How the player reads earlier Messages of a long story

**Date:** 2026-10-20
**Status:** Draft — investigation only, no code changes
**Goal:** Decide how the dashboard lets the player read the Messages that are older than the story window of the story log. Record the decision with evidence.

## Origin

[Decide what the story log shows past 50 Messages](../../.scratch/_closed/rust-rebuild-build/issues/23-story-log-past-50-messages.md) (ticket 23 of the map "build the Rust rebuild") chose option A (owner, 2026-10-20): the story log shows the newest 50 Messages. That fix makes the Messages before the window unreachable from the dashboard. The owner asked for this plan to find the long-term answer. This plan is not part of the rebuild map.

## Current state (known, 2026-10-20)

- `#story-log` polls `GET /fragment/story-log` every 2 s with `hx-swap="morph:innerHTML"` (`assets/index.html`). Each poll renders the whole window again. The morph keeps the DOM nodes, the focus and the selection (specs 30.11, 30.12, 30.14 in `docs/specs/browser_story_log.md`).
- Scroll rules: a poll that appends follows the bottom only when the log is at its bottom (30.20, 30.21). The first load goes to the bottom (30.23).
- `StoryReads::entries(game, before, limit)` (`src/adapters/driven/storage/stories.rs`) reads the `limit` newest Messages before the id `before`, oldest first. No caller passes `before` yet. So a cursor read exists.
- Each entry carries controls: Edit (`POST /history/:id`, any entry), the Swipe switch (`POST /message/:id/swipe/:index`), and on the last entry Delete, Retry and Retrigger. The last-entry controls act on the last stored Message (the `Tail`).
- An edit pauses the poll (`assets/dashboard-story-log.js`).

## Candidates

| Candidate | Short description | Main question |
|---|---|---|
| A. Show earlier | A control above the oldest entry loads the next older page with the `before` cursor and puts it above the window. | Where do the loaded entries live: in the polled element (the poll must carry the window size or the cursor) or in a separate element above it that the poll does not touch? How does the scroll position stay fixed on a prepend? |
| B. Paging | Numbered or older/newer pages. Only the newest page polls. | Does a page that does not poll show stale text after an edit or a Swipe switch? Is paging a good fit for reading a story? |
| C. Separate history page | A read-only page (for example `/games/:id/story`) that renders the whole story, or pages of it, with no poll. | Does the page need the Edit and Swipe controls, or is it read-only? Does it also serve as an export? |
| D. Tail-only poll | The poll returns only the entries that changed since a cursor, and the client appends them. Earlier entries load once and stay. | This changes the poll contract and specs 30.11 to 30.21. Is the gain worth it? |
| E. Larger window or all Messages | Raise the window size, or show all Messages. | What does a 2 s morph cost at 500, 1000 and 5000 Messages? |

The candidates can combine, for example C with A.

## Investigation questions

### Q1. Cost of the poll

Measure the server render time and the browser morph time of `GET /fragment/story-log` at 50, 200, 500, 1000 and 5000 Messages. Use a seeded SQLite Game and the browser tier (Chrome DevTools performance trace). The result decides if E is possible, and how large a polled window can be.

### Q2. What the player needs from earlier Messages

Decide with the owner: read only, or also edit and switch Swipes? Is a search or an export in scope? This decides between C (read-only) and A or B (controls on older entries).

### Q3. Freshness of entries outside the poll

An entry outside the polled window can go stale after an edit or a Swipe switch on it. Decide if that is acceptable, or if an action on an older entry re-renders that entry from its own response.

### Q4. Scroll and focus rules

For each candidate that keeps older entries in the story log (A, B, D), write the scroll and focus rules as browser scenarios in `docs/specs/browser_story_log.md` before code: a prepend keeps the visible entry in place; the poll keeps the loaded older entries; keyboard focus survives a prepend.

### Q5. Recommendation

Choose a candidate, or a combination, from Q1 to Q4. Write the spec scenarios. Write the build plan as a separate plan.

## Out of scope

- The size and the rule of the story window of the story log itself: ticket 23 decides those.
- The windows of the model calls (`AGENT_HISTORY_WINDOW` of the agents, the narrator history cap).
