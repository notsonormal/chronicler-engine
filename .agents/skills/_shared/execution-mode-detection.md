# Execution Mode Detection

This protocol distinguishes two pi installations:
- **Agent Mode (Plugin Install)**: pi-subagents plugin is installed; full agent orchestration available.
- **Inline Mode (Skills-only Install)**: only skills are loaded; no subagent delegation.

## Detection

Check whether the relevant agent file exists at `../../agents/` (relative to the skill). The execution mode depends on whether that file is present:

| Condition | Mode |
|-----------|------|
| Agent file present | Agent Mode (Plugin Install) |
| Agent file absent | Inline Mode (Skills-only Install) |

Common agent file types: `crate-researcher.md`, `rust-changelog.md`, `std-docs-researcher.md`, `docs-researcher.md`, `clippy-researcher.md`.

## Agent Mode behavior

When the agent file exists, launch it as a background task and consume its result:

```
Task(
  subagent_type: "general-purpose",
  run_in_background: true,
  prompt: <content read from ../../agents/<agent>.md>
)
```

Summarize the agent's output to the user.

## Inline Mode behavior

When the agent file is NOT present, execute directly. Use `agent-browser` CLI (or WebFetch fallback) to read the relevant URL, parse, and format output. Use `mcp__actionbook__search_actions` / `mcp__actionbook__get_action_by_id` for selectors when available.
