# Chronicler Engine Knowledge Base

## Overview
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## Every-run guardrails
- Repository health outranks the current task: a green build matters more than this task succeeding.
- Never delete or revert unknown or untracked files, even when they block you.
- During code reviews, change no code; report findings only.
- Never commit without explicit approval.
- Never circumvent the permission system; never edit its config without approval.

## Where to read next

| Doc | Holds | Reach it when |
| --- | --- | --- |
| `docs/agents/README.md` | Agent-process router. | Orient yourself on how to work here. |
| `docs/agents/development-loop.md` | Working norms, build loop, commands, concurrent builds. | Building, testing, or iterating. |
| `docs/agents/permissions.md` | Permission rules and approval boundaries. | Touching git, config, or permission-gated actions. |
| `docs/agents/delegation.md` | Subagent choice and sizing. | Delegating to a subagent. |
| `docs/agents/documentation.md` | Doc index and generated-index rules. | Writing docs or committing. |
| `tests/AGENTS.md` | Test structure, tiers, failure handling. | Writing, running, or debugging tests. |
| `CODING_STANDARDS.md` | Coding-rules router. | Writing or reviewing code. |
| `CONTEXT.md` | Domain glossary. | Naming a domain concept. |
| `ENVIRONMENT.md` | Build-environment limits and diagnostics. | Builds slow, blocked, or out of memory. |
| `STRUCTURE.md` | Generated source-tree index. | Finding a module's file or summary. |
