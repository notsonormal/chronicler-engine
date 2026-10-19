---
diataxis: reference
title: Narration System
---

## Model Configuration

LLM connections are stored as a list on the `settings` singleton row.

Each role names the connection it calls: `narration_connection_id` (main narrative call), `quantifier_connection_id` (post-narration scene analysis), and the connection whose id is `options` (the options agent). Roles may resolve to the same connection record or to different ones; the wiring builds a dedicated `LlmCallRecorder` for each. The recorder resolves its connection from the settings row at call time, so a connection switch takes effect on the next LLM call without a restart.

## Game Master Role

The narrator is an LLM acting as a Game Master for a text adventure.

The narrator is narrative-only. State mutation is the engine's job, run by the Job through the game rules after the LLM has spoken.

## Response Sanitization

`LlmCallRecorder::complete()` runs `sanitize_llm_output` on every response before saving forensics and returning the text to its caller. The sanitizer strips leaked reasoning artifacts that some chat-template-less models emit.
