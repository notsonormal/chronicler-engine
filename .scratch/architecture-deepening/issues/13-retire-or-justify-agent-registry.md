# 13 — Retire or justify the Agent registry

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we keep the `Agent` trait + `AgentRegistry` + `ExecutionPhase` dispatch, or
replace it with two named collaborators on `ActionPipeline` (the Quantifier
and the Options Agent) — and if replaced, what is the test seam, and what
happens to `BackendSelector`, `PreGeneration` and the unused `AgentResult`
variants?

## Background

This is **candidate E** of the 2026-10-04 review, rated **Worth exploring**.
See `assets/architecture-review-2026-10-04.html`, card E.

The friction: the registry's interface is wider than what it runs.

- Two Agents exist (`QuantifierAgent`, `OptionsAgent`), one per phase.
- `ExecutionPhase` has 3 variants. `PreGeneration` (the `#[default]`) has no
  Agent and no dispatcher.
- `OptionsGeneration` is dispatched at its own gated site
  (`action_pipeline/options.rs:83`), not through the generic loop at
  `action_pipeline/core.rs:442`.
- `BackendSelector` is set in `AgentConfig::defaults` and returned by
  `Agent::backend_selector`, but nothing reads it to choose a backend.
- `registry.rs` (82 lines) builds Agents with a two-arm string switch
  (`"quantifier"`, `"options"`). `with_agent` / `add_agent` / `is_empty` are
  test-only.
- `AgentResult` has 4 variants. The scout found only `StatePatch` and
  `Options` constructed in production. Check `NoOp` and the fourth variant in
  the grilling.

Deletion test: deleting `registry.rs`, `trait_def.rs` (11 lines) and the
config dispatch removes about 230 lines with no change in production
behaviour. The cost is the test injection point: `with_agent` is used by
`core_tests.rs` and `action_tests.rs` to inject a synchronous Quantifier.

## What this ticket resolves

- **Commit or reject.** Does "pipeline indifferent to which Agents exist"
  still earn its place with two Agents on two separate dispatch paths?
- **Shape if retired.** How `ActionPipeline` holds the two Agents, and the
  replacement test seam (one adapter in production plus one in tests counts
  as a real seam).
- **Shape if kept.** Whether `PreGeneration`, `BackendSelector` and unused
  `AgentResult` variants are deleted anyway, and whether Options moves onto
  the generic dispatch.
- **Doc impact.** `docs/diataxis/explanation/agent_system_design.md` still
  says `ExecutionPhase` has two variants and argues for the registry. Either
  outcome needs that doc updated.

## Constraints

- No ADR covers this. The design rationale lives in
  `agent_system_design.md`. If the grilling keeps the registry for a
  load-bearing reason, offer an ADR so later reviews don't re-suggest this.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`. CONTEXT.md defines
  **Agent** as "a pipeline step that runs at a defined phase". Retiring
  phases may sharpen that entry.
- Blocks ticket 07: 07's preset lookup was meant to land in `AgentRegistry`.
- Related: ticket 12 (shared Agent call core).
