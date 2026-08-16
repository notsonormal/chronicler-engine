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
`architecture-review.html` for the call-graph diagram and evidence.

The friction: `prompt_merge.rs:11-13` is one `format!` call.
`token_budget.rs:1-37` exposes seven public constants callers must know.
`PromptContext::build_narration_prompt` (`assembler.rs:91-109`) duplicates the
`assemble` call site. The genuine depth — layer ordering, context fitting,
budget math — already lives in `LayerRenderer::render_and_fit`
(`assembler.rs:139-168`), but the module boundaries do not reflect that depth.

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
