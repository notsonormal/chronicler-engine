# Idea: Scroll back through the whole story

Status: seed, 2026-11-03. From [Long-term memory and state](../long-term-memory-and-state/map.md) (owner: a separate effort).

## Why

The dashboard shows only the newest 50 entries of a Game (`STORY_WINDOW_SIZE`, `src/application/games/view_query.rs`). A person cannot scroll back to an older entry. The database keeps every Message, so the limit is in the view only (known).

The owner thinks that the limit was a cheap fix for slow refreshes of the whole UI. It does not solve that problem (owner, 2026-11-03).

## What is separate

What the dashboard can scroll back to is not what the prompt holds (owner). The prompt history is a different limit: the history cut ([Decide the size and the point of the history cut](../long-term-memory-and-state/issues/08-history-cut.md)). This effort changes only the view.

## Open questions

- Why the whole story region renders again, and how often (the screen load, the changes answer, each Cycle of Autoplay).
- Load older entries when a person scrolls up (a reverse infinite scroll), so that only the newest entries render at first.
- What a person can do on an old entry: read only, or also edit (`GameWriter::edit` can change the active Swipe text of any Message).
- Later: the memory view of Long-term memory and state links to a Message, so that Message must be reachable.

## Read first

- `docs/diataxis/reference/frontend/dashboard.md` and `docs/diataxis/explanation/dashboard_design.md`.
- `/impeccable` for the UI, `/chronicler-ui-investigator` for a probe server.
