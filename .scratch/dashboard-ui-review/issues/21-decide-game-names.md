# Decide how games are named and renamed

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The header and Games tab show the raw generated game name, for example `Redmist Estate_2026-09-29_1`. There is no way to rename a Game. How should a Game's name be shown, and can the user rename it?

## Context

- Finding 5.3. Screenshots 01, 16.
- Name generation: `src/domain/model/utils/game_name.rs`.
- In `CONTEXT.md`, a Game binds World and Persona by identifier, with display names denormalized. Check whether a display name for the Game itself fits the glossary (`/domain-modeling`).

## Done when

- The decision is in the ticket answer. Implementation tickets are created.

## Answer

**Option A — a stored display name, with rename.**

- A Game gains a display name. The creation default is a readable form of the
  generated name: world display name, date, ordinal — for example
  `Redmist Estate — 29 Sep 2026 (1)`. No underscores.
- The generated `{WorldName}_{YYYY-MM-DD}_{N}` value stays as the stable
  creation default; it is no longer shown raw. The uniqueness scan keeps working
  against the stable name, so a rename cannot disturb it.
- The header and the Games tab show the display name. A rename control edits it;
  this is the first write to a game's name after insert.
- Display names may collide. They are for display, so no uniqueness constraint.
- `CONTEXT.md` gains the Game display-name term, mirroring World and Persona,
  which already denormalize a display name onto the Game.

### Graduated

[Name and rename Games](59-game-display-names.md).
