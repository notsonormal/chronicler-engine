# Name and rename Games

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Implements [Decide how games are named and renamed](21-decide-game-names.md).

- Add a Game display name. The creation default is a readable form of the
  generated name: world display name, date, ordinal — for example
  `Redmist Estate — 29 Sep 2026 (1)`. No underscores.
- Keep the generated `{WorldName}_{YYYY-MM-DD}_{N}` value as the stable name.
  The uniqueness scan in `generate_game_name` keeps reading it, so a rename
  cannot disturb it. Put the display name in its own field.
- The header and the Games tab show the display name instead of the raw key.
- Add a rename route, handler, and control. This is the first write to a game's
  name after insert.
- Display names may collide; no uniqueness constraint.
- Add the Game display-name term to `CONTEXT.md`.

## Done when

- `python build.py` is green.
- A spec scenario covers rename, and the header and Games tab show the display
  name.
- The user has reviewed the diff.

## Answer

Option A, as decided: a Game gains a stored display name beside its stable generated name.

- `Game` / `NewGame` carry `display_name`; the creation default comes from `display_name_from_key` — `World — DD Mon YYYY (N)`, no underscores. The generated `{WorldName}_{YYYY-MM-DD}_{N}` value stays as the stable name: `generate_game_name`'s uniqueness scan still reads it, so a rename cannot disturb it.
- Migration v25 adds `games.display_name TEXT NOT NULL DEFAULT ''` and backfills legacy rows in Rust from the stable name; the `column_exists` guard keeps it idempotent and old databases openable.
- `POST /games/:id/rename` takes `display_name`, trims it, rejects blank with `400` and "Display name cannot be empty", and never writes the stable name. No uniqueness constraint — display names may collide. No server-side length cap; the control caps at 120 characters. The response is `HX-Refresh: true`, like create and switch, so the header and Games tab re-render from stored state.
- The header and the Games tab show the display name. The rename control is a no-JS `<details>` on the Active Game card and each Saved Games card; the header only shows the name.
- `GameViewQuery::get_current_game_name` became `get_current_game_display_name`. `CONTEXT.md` gains the **Game Display Name** term.
- **Decision recorded at review: a rename bumps `updated_at`.** `list_games` orders by `updated_at DESC`, so renaming a game moves it to the top of the Saved Games list. Kept deliberately — the list is recency-ordered and a rename is a write to the game.

Tiers: unit (`game_tests`, `catalogue_tests`, `view_query_tests`), driven-adapter (`games_tests` on both InMemory and SQLite, `plumbing_tests` for the v25 backfill), and tier 1 `tests/http/games_rename.rs` for scenarios 17.4 and 17.5. Reverting the display-name reads in `view_mappers` and `view_query` made 17.4 fail at the header assertion. `python build.py` is green.
