---
trigger: always_on
---

# Antigravity IDE Rules

- **IDE Awareness**: Unless otherwise specified, you are running in the **Google Antigravity IDE**. This is a specialized fork of VS Code designed for agentic AI coding.
- **Local Integration**: You have access to a local LLM via the `Local LLM` skill (`.agents/skills/local_llm/`). Use it when the user requests local offloading.
- **Style**: Follow the user's preferred style as documented in `Notes.md`.
- **Memory Bank**: You must utilize the Memory Bank in `.ag-memory/` for operational state and the `docs/` folder for project facts.
  - **Read**: Check `.ag-memory/MEMORY.md`, `.ag-memory/TODO.md`, and relevant files in `docs/` at the start of tasks.
  - **Write**: Update `TODO.md` with progress, `SCRATCHPAD.md` for thoughts, and `MEMORY.md` for behavioral habits.
  - **Clean Up**: Regularly prune `TODO.md` and wipe `SCRATCHPAD.md` after task completion.