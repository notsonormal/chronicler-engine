# 07 — Move Agent preset resolution (Quantifier + Options) into AgentContext

Type: grilling
Status: open
Blocked by: 13
Assignee: (unclaimed)

## Question

Do we commit to moving the prompt-preset lookup out of
`QuantifierAgent::execute` and `OptionsAgent` (both reach into storage for
settings → active preset id → preset) and into whatever builds `AgentContext`,
landing the resolved preset on `AgentContext` — and if so, what is the shape of
the deepened Agent seam?

(Scope widened 2026-10-04: the 2026-08-16 question named only the Quantifier and
put the lookup in `AgentRegistry`. `OptionsAgent` now repeats the pattern, and
ticket 13 may delete the registry.)

## Background

This is **candidate 6** of the architecture review. See
`assets/architecture-review.html` for the leak diagram and evidence.

The friction: the `Agent` seam is narrow — `execute(&self, ctx: &AgentContext)
-> Result<AgentResult, EngineError>`. But `QuantifierAgent::execute`
(now `quantifier/agent.rs:75-83`) breaks it by reaching into storage for
`get_settings` → active quantifier preset id → `get_preset`. `AgentContext` carries
`state`, `main_response`, `player_input`, `current_room`, `map`, `persona`,
`npcs` — but no prompt override. `registry.rs`
(`from_configs_with_storage`) constructs the agent with
`from_config_with_storage`, baking the storage dependency into the agent.

The deletion test *reappears*: if the reach is removed from the agent, the
preset-loading code moves to `AgentRegistry` or the orchestrator that builds
`AgentContext` — which is the correct home. The agent then satisfies its own
seam honestly.

## What this ticket resolves

- **Commit or reject.** Is the storage/settings reach a real leak, or does the
  agent legitimately own its preset resolution?
- **Interface shape.** What `AgentContext` gains (the resolved preset); what
  the `Agent` trait no longer requires; whether `QuantifierAgent` still takes
  storage at construction.
- **What survives.** Whether agent tests can drop their storage mock; which
  tests cross the seam unchanged.

## Constraints

- Must keep the Agent seam narrow — the deepening is about honouring the
  existing interface, not widening it.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`.
- Domain terms: Agent, Quantifier, Prompt Preset (CONTEXT.md).
- Independent of the storage-seam tickets (01–03): the core decision is the
  Agent seam, not the Storage seam.
- Blocked by ticket 13 (Agent registry): if the registry goes, the preset
  lookup cannot land "in `AgentRegistry`". 13 decides who builds
  `AgentContext`.
- Related: ticket 12 (shared Agent call core) covers the rest of the
  Options/Quantifier duplication (scene-context prompt blocks, retry loop). It
  leaves the preset fetch to this ticket.

## Current state (2026-10-04)

Worse. The pattern now exists twice:

- `QuantifierAgent`: `quantifier/agent.rs:34` `from_config_with_storage`; the
  fetch is at `:75-83` (`get_settings` at `:77`, `get_preset` at `:79`).
- `OptionsAgent`: `options/agent.rs:33` `from_config_with_storage`; the same
  fetch at `:68-79` (`get_settings` at `:72`, `get_preset` at `:74`).

`AgentRegistry` is now built by `from_configs_with_storage` (`registry.rs:20`),
with a string switch for `"quantifier"` / `"options"`. `AgentContext`
(`domain/model/agent.rs`) still has no preset field.
