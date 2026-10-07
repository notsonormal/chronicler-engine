# Delegation

Reach this doc when you delegate work to a subagent.

## Subagent count

Prefer one long-running subagent over several short ones. One subagent loads context once, so it costs less than two.

Some workflows and skills use several subagents on purpose — some code-review workflows, and work that implements several wayfinder tickets at once.

## Scout subagents

Create `scout` subagents only when the current model is Anthropic (for example Opus or Sonnet). With a non-Anthropic model, read the information you need in the current session.

This rule covers `scout` subagents only.
