# Decide how games are named and renamed

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The header and Games tab show the raw generated game name, for example `Redmist Estate_2026-09-29_1`. There is no way to rename a Game. How should a Game's name be shown, and can the user rename it?

## Context

- Finding 5.3. Screenshots 01, 16.
- Name generation: `src/domain/model/utils/game_name.rs`.
- In `CONTEXT.md`, a Game binds World and Persona by identifier, with display names denormalized. Check whether a display name for the Game itself fits the glossary (`/domain-modeling`).

## Done when

- The decision is in the ticket answer. Implementation tickets are created.
