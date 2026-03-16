---
name: Local LLM
description: Ability to offload coding and analysis tasks to a local LLM (Ollama). Only use when explictly requested via the Antigravity IDE. Visual Studio Code and OpenCoder calls Ollama directly when needed.
---

# Local LLM Skill

This skill allows Antigravity to offload specific tasks to a local LLM instance running on the user's hardware (e.g., RTX 4080).

## Usage

When the user asks to "offload", "run locally", or "use local model", use the following command to communicate with the local Ollama instance:

```powershell
python scripts/ask-local.py "<prompt>"
```

## Recommended Models

- **qwen2.5-coder:7b**: Default for coding tasks.
- **llama3.1:8b**: General chat and reasoning.

## Guidelines

1. **Privacy**: Use this skill when the user explicitly requests local execution for privacy-sensitive code.
2. **Performance**: Use this skill for heavy-duty text transformations or code generation that doesn't require Gemini's multi-file context but benefits from the 4080's speed.
3. **Hybrid Flow**: Always report the output of the local model back to the user or use it to inform your next steps in the Antigravity interface.
4. **Antigravity IDE Only**: Visual Studio Code and OpenCoder calls Ollama directly when needed. You do not need to use this skill in OpenCoder
