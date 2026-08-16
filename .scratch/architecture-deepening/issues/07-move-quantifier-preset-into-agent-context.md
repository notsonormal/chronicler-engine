# 07 — Move QuantifierAgent preset resolution into AgentContext

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to moving the quantifier prompt-preset lookup out of
`QuantifierAgent::execute` (which reaches into `self.storage` and
`self.settings`) and into `AgentRegistry`, landing the resolved preset on
`AgentContext` — and if so, what is the shape of the deepened Agent seam?

## Background

This is **candidate 6** of the architecture review. See
`architecture-review.html` for the leak diagram and evidence.

The friction: the `Agent` seam is narrow — `execute(&self, ctx: &AgentContext)
-> Result<AgentResult, EngineError>`. But `QuantifierAgent::execute`
(`agent.rs:58-66`) breaks it by accessing `self.storage` and `self.settings`
to look up the active quantifier prompt preset. `AgentContext` carries
`state`, `main_response`, `player_input`, `current_room`, `map`, `persona`,
`npcs` — but no prompt override. `registry.rs:34-49` constructs the agent with
`from_config_with_storage`, baking the storage/settings dependency into the
agent.

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
