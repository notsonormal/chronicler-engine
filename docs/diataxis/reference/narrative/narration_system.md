---
diataxis: reference
title: Narration System
---

## Model Configuration

LLM connections are stored as a list on the `settings` singleton row.

Four Connection slots name the Connection of each kind of call. The Narration slot (`narration_connection_id`) serves a Narration, an event Narration and the Retry of a typed Input. The Impersonate slot (`impersonate_connection_id`) serves an Impersonate and the Retry of an Impersonate Input. The Quantifier slot (`quantifier_connection_id`) serves the Quantifier. The Options slot (`options_connection_id`) serves the Options agent.

The Impersonate and Options slots are optional. An empty slot uses the Narration Connection. Two slots may name the same Connection.

A Connection can set `max_tokens_ceiling`, the most that a call to it may ask for (see Prompt Presets in the Prompt System).

A call to OpenRouter sends a `session_id`. All OpenRouter calls of one Game send the same `session_id`. A call to OpenCode Go sends the header `x-opencode-session`. Each Game sends one value for each Role. An Impersonate sends the value of the Narrator Role.

The wiring builds one `LlmCallRecorder` for each slot. The recorder resolves its Connection from the settings row at call time. So a connection switch takes effect on the next LLM call without a restart. The health of an Impersonate shows under the Narrator Role, because Impersonate has no Role.

## Game Master Role

The narrator is an LLM acting as a Game Master for a text adventure.

The narrator writes Narrations only. State mutation is the engine's job, run by the Job through the game rules after the LLM has spoken.

## Steering

Two slash commands steer one generation. `/guide` (a Guided Generation) steers what the narrator writes, and the result is a Narration. `/impersonate` steers who speaks, and the result is an Impersonate Input. A steering lives for one generation and never becomes a history entry. A generation takes one steering source at a time. An Impersonate generation uses the impersonate preset in place of the system preset.

## Prompt Rules of a Generation

The narrator owns the prompt rules of each kind of generation. The Job loads the values of one narration and gives them to the narrator with the generation. The narrator then makes the user message, sets the guide and the Impersonate flag, and names the role whose health the call feeds. That role is the Trigger for an event Narration and the narrator for every other kind. The narrator works from the values that it is given.
