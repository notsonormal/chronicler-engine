# 06 — Deepen PromptAssembler, absorb shallow helpers

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to deepening `PromptAssembler` by absorbing the shallow helpers
around it (`prompt_merge.rs`, the public `token_budget.rs` constants,
`PromptContext::build_narration_prompt`) as private internals — and if so,
what is the shape of the deepened module?

## Background

This is **candidate 5** of the architecture review. See
`assets/architecture-review.html` for the call-graph diagram and evidence.

The friction: `prompt_merge.rs` (`merge_single_user_message`) is one
`format!` call. `token_budget.rs` exposes public constants callers must know
(seven on 2026-08-16, five now — see Current state).
`PromptContext::build_narration_prompt` (now `assembler.rs:181`) duplicates
the `assemble` call site. The genuine depth — layer ordering, context fitting,
budget math — already lives in `LayerRenderer::render_and_fit` (now private,
`assembler.rs:212`), but the module boundaries do not reflect that depth.

The deletion test splits: `prompt_merge.rs` and the duplicate call site
*vanish*; the token-budget constants *reappear* (callers need them) — so the
deepening absorbs them as private internals rather than deleting them.

## What this ticket resolves

- **Commit or reject.** Does the cluster's current split earn its locality, or
  does it scatter one deep concept?
- **Interface shape.** What `PromptAssembler` exposes; which constants and
  helpers go private; whether `PromptContext` survives at all.
- **What survives.** Which tests cross the assembler interface unchanged.

## Constraints

- Must not regress the prompt-preset placement decided in
  `.scratch/inherent-impl-locality/` ticket 07 (`PromptContext` was moved into
  `assembler.rs` and re-exported).
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`.
- Domain terms: Narrative, Prompt Preset (CONTEXT.md).
- Per AGENTS.md, if this decision leads to changes in
  `src/application/prompting/`, the LLM-test policy requires
  `python build.py --llm-only` at implementation time — note this in the
  hand-off, not here.
- Related: ticket 10 (arrival narration). Arrival is the only production
  caller of `build_narration_prompt`. If 10 routes arrival through the shared
  narration generation, that method has no caller left. Resolving 10 first
  settles part of this ticket. Not a hard block.

## Current state (2026-10-04)

Slightly better, still present. `token_budget.rs` now exposes 5 public
constants (`MAX_CONTEXT_TOKENS`, `MAX_HISTORY_TOKENS`, `MAX_RESPONSE_TOKENS`,
`SAFETY_MARGIN_TOKENS`, `MIN_INPUT_BUDGET_TOKENS`) plus `estimate_tokens` and
`truncate_to_budget`. `prompt_merge.rs` is unchanged. `build_narration_prompt`
still exists at `assembler.rs:181`, and its only production caller is
`arrival_service.rs:153`. `render_and_fit` is private, so part of the depth is
already behind the interface.
