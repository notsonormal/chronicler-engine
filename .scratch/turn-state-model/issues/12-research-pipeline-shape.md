# Research: survey the pipeline's shape for the fix-or-rewrite decision

Type: research
Status: claimed
Blocked by: —

## Question

What is the current shape of the generation pipeline (modules, entry paths, shared phases, seams to `MessageService` and `GenerationGate`), where do the entry paths duplicate or diverge, and how often has this code changed and broken? The answer gives the facts that the "Fix or rewrite" decision weighs.

## Scope

1. Module map of `src/application/pipeline/` (`pipeline_run.rs`, `narration_generation.rs`, `phase_error.rs`, `action_pipeline/*`), `message_service.rs`, `generation/`, `arrival_service.rs`: size, public seams, and callers.
2. Per entry path (action, each retry form, Retrigger, Options, arrival): which phases it shares through `PipelineRun`, and which it re-implements.
3. Where Snapshot saves and pointer rules live: in one module, or spread across phases (use the count from [Trace the Snapshots each generation path saves](04-research-trace-generation-snapshots.md) if it is resolved; otherwise count them).
4. History: `git log` churn of these files, and the closed tickets and plans that fixed defects here (`.scratch/_closed/`, `docs/plans/`).
5. Seams for tests: can a test drive one entry path without HTTP? Where would a model-level test sit?

Use the `/codebase-design` vocabulary (module depth, seams). Report facts. Do not recommend fix or rewrite; that is a later decision.

## Output

An asset `../assets/research-12-pipeline-shape.md`: the module map, the entry-path-by-phase matrix, the churn and defect history, and the test seams. Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

The map's fog patch "Fix or rewrite".
