# Assess SwarmForge for agent coordination patterns

Type: research
Status: resolved
Assignee: pi
Blocked by: 01

## Question

What coordination patterns does `swarm-forge` use for multi-agent AI systems, and are any of them applicable to Chronicler Engine's agent registry, quantifier agent, or future agent orchestration?

## Answer

Investigated `https://github.com/unclebob/swarm-forge` (documentary `main` branch plus `two-pack`, `four-pack`, `six-pack` workflow branches) and the two example projects built with it: `experiment-htw-go-swarm` and `experiment-htw-clj-swarm`.

### SwarmForge coordination patterns

1. **Role-based topology via config.** `swarmforge/swarmforge.conf` declares windows as `window <role> <backend> <worktree>`. Example roles are `specifier`, `coder`, `refactorer`, `architect`, `hardender`, `QA`.
2. **Worktree isolation.** Each role works in its own git worktree (or `master`), so agents do not step on each other's files.
3. **Layered constitution prompts.** `constitution.prompt` points to `constitution/articles/`; shared articles from `main` combine with branch-local articles and per-role `.prompt` files to shape behavior.
4. **File-based handoff protocol.** Agents drop drafts in `.swarmforge/handoffs/outbox/`. A `handoffd` daemon copies them to recipient `inbox/new/`, sends a generic tmux wake-up, and moves the original to `sent/`.
5. **Two message types.** `git_handoff` carries a commit SHA and a stable task name; `note` carries a one-line message. `git_handoff` always forwards down the chain until the terminal broadcast.
6. **Queue helpers.** `ready_for_next.sh` and `done_with_current.sh` move files through `new/`, `in_process/`, and `completed/` and enforce task/batch receive modes.
7. **Audit trail.** Handoff headers (`id`, `from`, `to`, `recipient`, `priority`, `type`, `created_at`, `enqueued_at`, `dequeued_at`, `completed_at`) and filesystem location replace a logbook.
8. **Per-role backend selection.** Each role can launch a different agent backend (`claude`, `codex`, `copilot`, `grok`).

### Chronicler Engine comparison

- `Agent` trait (`src/application/agents/trait_def.rs`): exposes `name`, `phase`, `backend_selector`, `execute`.
- `AgentRegistry` (`src/application/agents/registry.rs`) holds agents and selects by `ExecutionPhase` (`PreGeneration`, `PostGeneration`).
- Only `QuantifierAgent` exists today; it runs in `ExecutionPhase::PostGeneration` inside the action pipeline (`run_post_generation_agents` in `src/application/pipeline/action_pipeline/core.rs`).
- Agents execute synchronously per turn, receive an `AgentContext`, and return `AgentResult::StatePatch`, `PromptDirective`, or `NoOp`. Patches merge to update scene NPCs.
- `BackendSelector::UseNamed` already allows per-agent backend routing.

### Applicability assessment

| SwarmForge pattern | Direct applicability | Notes |
|---|---|---|
| Role topology + worktree isolation | Low | SwarmForge coordinates multiple agents editing code across worktrees. Chronicler agents run inside one runtime engine for one turn. |
| File-based durable handoffs | Low | Useful for async multi-agent workflows; Chronicler's pipeline is synchronous and in-process. |
| Handoff daemon + tmux wake-ups | Low | Assumes a desktop tmux environment, not a server-side Rust engine. |
| Layered constitution / role prompts | Medium | Chronicler already has prompt presets (`active_quantifier_prompt_preset_id`). The constitution idea could extend agent prompts with shared + per-role articles, but that is a prompt-engineering layer, not an architecture change. |
| Pipeline chain forwarding | Low/Medium | Chronicler already composes agents in a phase by merging `StatePatch` results. The "always forward" rule is specific to code-review chains, not narrative generation. |
| Per-role backend selection | Already present | `BackendSelector::UseNamed` covers this. |
| Audit trail / task queue | Low | Chronicler has `LlmCallRecorder` for forensics, not inter-agent handoff audit. |

### Conclusion

- **No direct fit.** `swarm-forge` is a desktop multi-agent coding coordination framework. Chronicler Engine's agent system is an in-pipeline, single-turn, state-patch extension point.
- **One transferable idea:** structured, layered agent prompts ("constitution articles" plus per-role prompts) could enrich how Chronicler configures agents, but this is already partially expressed through prompt presets.
- **No recommendation to adopt SwarmForge patterns as code.** If Chronicler later grows true multi-agent coordination (e.g., pre-generation + post-generation agents negotiating across turns, or agents running in separate processes), the durable handoff and queue patterns would be worth revisiting. For the current architecture they are unnecessary complexity.

### Suggested map update

Ticket `02` closes. The finding that multi-agent coordination is not applicable removes the need for follow-up SwarmForge/HTW-swarm tickets unless the destination explicitly changes. The prompt-constitution insight could be captured under the existing prompt-preset workstream rather than as a new Uncle Bob ticket.
