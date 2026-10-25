---
diataxis: reference
title: Narration System
---

## Model Configuration

LLM connections are stored as a list on the `settings` singleton row.

Each role names the connection it calls: `narration_connection_id` (the narration call), `quantifier_connection_id` (the Quantifier call), and the connection whose id is `options` (the options agent). Roles may resolve to the same connection record or to different ones; the wiring builds a dedicated `LlmCallRecorder` for each. The recorder resolves its connection from the settings row at call time, so a connection switch takes effect on the next LLM call without a restart.

A connection carries a base URL (`base_url`); the OpenRouter and Ollama backends call it, and an empty field follows the provider default.

A connection carries a context window (`max_context_tokens`), which the Connection form sets; an empty field follows the provider default.

A connection may also carry a reasoning effort (`reasoning_effort`), which the Connection form sets. The OpenRouter and Ollama backends send it as `reasoning.effort` in the request. An empty field sends no reasoning field, so the model default applies.

## Game Master Role

The narrator is an LLM acting as a Game Master for a text adventure.

The narrator writes Narrations only. State mutation is the engine's job, run by the Job through the game rules after the LLM has spoken.

## Prompt Rules of a Generation

The narrator owns the prompt rules of each kind of generation. The Job loads the values of one narration and gives them to the narrator with the generation. The narrator then makes the user message, sets the guide and the Impersonate flag, and names the role whose health the call feeds: the Trigger for an event Narration, the narrator for every other kind. The narrator works from the values that it is given.

## Response Sanitization

`LlmCallRecorder::complete()` runs `sanitize_llm_output` on every response before saving the LLM message and returning the text to its caller. The sanitizer strips leaked reasoning artifacts that some chat-template-less models emit.
