---
diataxis: reference
title: Options
---

# Options

## Overview

Options are pre-written player inputs the engine offers while a game is idle. Using an option submits its text through the normal action path — it becomes the player's input and a narration follows. The offered set itself never enters history: it is game state, not a message.

## Where Options Come From

The options agent generates the set. It is a named agent (`options`) with its own preset, prompt, and parser, and it runs in its own pipeline phase — never inside the per-turn post-generation merge. The pipeline dispatches it at exactly two gated sites:

1. **Turn end** — after a narration turn, when the game's always-on toggle is on.
2. **On demand** — when the player enters the `/options` slash command (one of the three commands in the command palette).

The agent's prompt carries a scene summary — current room, NPCs in area, recent history — and asks for a fixed number of options (three by default). The parser accepts a numbered list or suggestion-tagged lines; zero parsable options after the retry budget surfaces an error to the caller instead of failing the turn.

## Two Triggers

| Trigger | When it fires | Where the toggle lives |
|---|---|---|
| Always-on | After every non-impersonate narration turn | A game setting, inherited from the World at game creation |
| On demand | When the player enters `/options` | Always available |

Impersonate turns never touch the offered set — the player is speaking, so there is nothing to offer.

## Offered-Set Lifecycle

The set lives in game state and is persisted with the turn's snapshot, so it survives a page reload mid-turn. The rules:

- A successful generation **replaces** the set.
- A failed on-demand regeneration **keeps** the previous set and adds a System message naming the failure.
- Every non-impersonate narration turn **clears** the set first — it must describe the current scene, never a stale one. When the always-on toggle is on, the turn then refills it; a failed regeneration leaves it empty plus the System message.
- No generation path adds history entries.

## Using an Option

The options dock renders one button per option and self-polls; it renders nothing while the set is empty. Each option carries two actions:

- **Use** — submits the option text as the player's input (the same form submit a typed action uses). An Input entry carrying that text enters history and a narration follows.
- **Edit** — copies the text into the command input and focuses it, for the player to modify before sending. No history entry is created.

## Errors and Limits

- `/options` on a fresh game with no scene history is refused: the response is a 500 whose body names the validation failure ("No scene to generate options for yet"), and the dock stays empty. This is the specified behaviour, pinned as-is.
- When no options agent is configured, on-demand use reports that the agent is not available and keeps the set; with the always-on toggle on, the same report replaces the set.
- The agent can use a dedicated LLM connection (the `options` connection id); without one it falls back to the narrator's connection.

## Mode Neutrality

Options work in both narrator modes — the offered set does not depend on who writes the protagonist. Mode switching never clears or regenerates the set by itself.

## Document References

- [`./agent_system.md`](./agent_system.md) — the agent registry and agent lifecycle the options agent joins.
- [`./narrator_mode.md`](./narrator_mode.md) — narrator modes and posture; options are mode-neutral.
- [`../game_flow.md`](../game_flow.md) — the pipeline phases and the generation gate the on-demand path claims.
- [`../frontend/dashboard.md`](../frontend/dashboard.md) — the command palette that lists `/options` and the dashboard areas around the dock.
- [`../../../specs/options.md`](../../../specs/options.md) — the HTTP behaviour contract (scenarios 24.1–24.13).
- [`../../../specs/browser_options.md`](../../../specs/browser_options.md) — the browser behaviour contract (scenarios 26.2–26.4).
