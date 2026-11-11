# R2: the story window

Type: task (AFK)
Status: resolved
Blocked by: 29

## Question

Build §4 of [the build plan of the response](../../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §4 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §4 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built with R3 in one commit on `wf/rebuild-response`, after a green full gate (`logs/build_20261021_011115.log`: 189 guardrail, 1349 integration, 29 browser tests pass).

- `GameViewQuery::story_window` owns the size (`STORY_WINDOW_SIZE = 50`). `StoryReads::window` reads the newest entries and the Retrigger fact under one connection lock (new key `StorageCall::StoriesWindow`).
- The value is the domain `StoryWindow` (entries and `retrigger_allowed`); the last Message id is `entries.last().id`, so it holds no separate id.
- `render_story_log` only renders; `MAX_LOG_DISPLAY`, `take` and `get_story_log_entries` are gone.
- Scenarios 8.11 and 8.12 with tier-1 tests; storage cases `story-window-newest-entries` and `story-window-retrigger-allowed`. `dashboard.md` says "the story window of the Game: its newest Messages, in order."
