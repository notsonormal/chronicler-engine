---
diataxis: reference
title: Narration System
---

## Model Configuration

LLM connections are stored as a list on the `settings` singleton row.

Each role names the connection it calls: `narration_connection_id` (the narration call), `quantifier_connection_id` (the Quantifier call), and the connection whose id is `options` (the options agent). Roles may resolve to the same connection record or to different ones. The wiring builds a dedicated `LlmCallRecorder` for each. The recorder resolves its connection from the settings row at call time. So a connection switch takes effect on the next LLM call without a restart.

## Game Master Role

The narrator is an LLM acting as a Game Master for a text adventure.

The narrator writes Narrations only. State mutation is the engine's job, run by the Job through the game rules after the LLM has spoken.

## Steering

Two slash commands steer one generation. `/guide` (a Guided Generation) steers what the narrator writes, and the result is a Narration. `/impersonate` steers who speaks, and the result is an Impersonate Input. A steering lives for one generation and never becomes a history entry. A generation takes one steering source at a time. An Impersonate generation uses the impersonate preset in place of the system preset.

## Prompt Rules of a Generation

The narrator owns the prompt rules of each kind of generation. The Job loads the values of one narration and gives them to the narrator with the generation. The narrator then makes the user message, sets the guide and the Impersonate flag, and names the role whose health the call feeds. That role is the Trigger for an event Narration and the narrator for every other kind. The narrator works from the values that it is given.
