# Name and rename Games

Type: task (AFK)
Status: open
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
