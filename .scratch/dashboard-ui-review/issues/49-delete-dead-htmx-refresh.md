# Delete the dead `htmx:refresh` calls and fix the docs

Type: task (AFK)
Status: open
Blocked by: 48

## Question

Six `htmx.trigger(..., "htmx:refresh")` calls in `assets/index.html` fire an event nothing listens for. Decide whether to delete them or give the refresh a real event name, then fix the docs that describe them as working.

## Context

- htmx 1.9.10 has no `htmx:refresh` event and no `refresh` trigger. The bundle's only two `refresh` strings are `refreshOnHistoryMiss`. `git grep "htmx:refresh"` finds no listener and no test. The string first appears in `da46d199`, when the page loaded htmx 1.9.10 from unpkg — so it has never worked here.
- Call sites: `assets/index.html:341,362,394` (`#story-log`), `:421` (`#visual-sidebar`), `:422` (`#header`), `:740` (the LLM panel).
- A seventh dead event sits beside them: the `document.body.addEventListener("action-area-refresh", ...)` at `assets/index.html:778` has no dispatcher either (noted by [Rework the action area](42-rework-the-action-area.md)). Fold it into the same cleanup.
- Do not confuse it with `HX-Refresh`, the response header in `src/adapters/driving/http/utils/response.rs:22`. That one is real and tested (`tests/http/games_create.rs`, `games_switch.rs`, `tests/http/requires_migration/fragment.rs`, `worlds_fragment_handlers.rs`). This ticket does not touch it.
- Docs that describe the dead event as working: `docs/diataxis/reference/frontend/dashboard.md:96,118`.
- Source: the implementer's report on ticket 27, recorded in [Decide how the story-log poll keeps DOM state](10-decide-story-log-poll-swap.md).
- After [Morph the story-log poll swap](48-morph-story-log-poll-swap.md) ships, the dead calls are harmless: a morph onto equal content is a no-op, and the 2s poll covers the update within a cycle. Deleting is therefore acceptable.
- If an immediate refresh is wanted instead, name a real event in each container's `hx-trigger` (for example `load, every 2s, panel-refresh`) and fire that name. Do **not** fire `load`: htmx 1.9.10 handles `load` as a one-shot init trigger with an internal `loaded` guard, so re-firing it reaches nothing.
- Blocked by 48 because both edit `assets/index.html`.

## Done when

- No dead `htmx:refresh` (or `action-area-refresh`) remains, and `docs/diataxis/reference/frontend/dashboard.md` matches the shell.
- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.
