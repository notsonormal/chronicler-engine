# Open two Games in two browser tabs

Type: task
Status: needs-triage
Blocked by: —

## Question

The owner wants, later, two Games open in two browser tabs at the same time. What change lets each tab name its own Game?

## Context

- The Job system and the Autoplay run are keyed by `GameId`. None of them reads `ActiveGame`.
- The coupling to the one active-Game pointer is in `CurrentGame` (`adapters/driving/http/utils/current_game.rs`) and in `GameCatalogue::active_game_id`. `GameCatalogue::reset` already takes a `GameId`.
- The client polls (`/status/generating`, `/fragment/story-log`, `/fragment/games`) carry no Game id.
- Direction (from the plan, section 7): the request carries its Game (a route segment or a header), `CurrentGame` reads it from the request, and the client polls carry the Game id.
- Rules the Autoplay work kept: every application API takes its `GameId` as an argument, no new read of `ActiveGame` outside `GameCatalogue` and `CurrentGame`, and every per-Game fact comes from the request's `GameId`.
