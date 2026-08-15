# 07 — Resolve PromptPreset and PromptContext placement

Type: grilling
Status: ready-for-agent
Blocked by: (none)

> **Re-pathed and scope widened.** The `narrative_prompt/` module was renamed to
> `prompting/` (visible in `src/application/prompting/`). `PromptContext` —
> flagged by 01's trial, previously un-ticketed fog — is folded into this ticket
> because it lives in the same `prompting/` cluster as `PromptPreset`'s
> application-layer impl.

## Question

Two types in the `prompting` cluster violate `guardrails_inherent_impl_locality`.

**`PromptPreset` — cross-layer split:**

- Struct defined in `domain/model/prompt_preset.rs` (`domain` layer)
- `impl PromptPreset { assemble_text }` in `application/prompting/assembler.rs` (`application` layer) — was `application/narrative_prompt/assembler.rs`

The earlier grilling agreed: if a type has behavior that belongs to the application layer, the type itself belongs to the application layer. Cross-layer impls mean the **type is misplaced**, not the impl.

**`PromptContext` — same-layer, cross-file split:**

- Struct defined in `application/prompting/types.rs`
- `impl PromptContext` in `application/prompting/assembler.rs`

Both def and impl are in the `application` layer, but `assembler.rs`'s parent dir (`prompting`) does not end with `/prompt_context` = `snake(PromptContext)`, and `impl_path != def_path`. Same-layer split, not cross-layer.

The decision this ticket resolves: where do `PromptPreset` and `PromptContext` belong, and how are their impls co-located?

Candidate answers to grill between (apply per type — they need not resolve the same way):

- **A) Move the type to application.** The type carries app-layer behavior (`PromptPreset::assemble_text` uses token budget utils, preset XML builders, NPC context). It's a prompt-system concept, not a pure domain model. New home: `application/prompting/prompt_preset.rs` (or `prompt_preset/` folder if split). For `PromptContext`, it is already in `application/prompting/` — A means consolidating its impl into `types.rs` (or a `prompt_context.rs`).

- **B) Move the behavior down to domain** (`PromptPreset` only). Strip the app-layer dependencies from `assemble_text` (pass `render_preset_xml_parts` result up, or have the caller do the joining), and keep `PromptPreset` as a pure data struct in domain. The method becomes unnecessary or moves to a free function in domain. `PromptContext` is already application-layer, so B does not apply to it — its resolution is A (consolidate) regardless.

- **C) Extract a trait in domain, impl in application.** `domain::model::prompt_preset::PromptPresetAssembler` trait, `impl PromptPreset` stays in domain with no methods, `impl PromptPresetAssembler for PromptPreset` lives in application. NOTE: this is a trait-impl locality pattern and is governed by a separate policy (out of scope for this effort). Only choose C if the trait-impl rule would also be enforced to keep it honest.

This is HITL — the agent does not pick. Open a `/grilling` session with the user and surface the tradeoffs before deciding, per type.

Constraints:
- After the decision, the refactor must be executed in this ticket (or split into a follow-up task ticket if SP ≥ 8).
- `build.py` green at every landed step.
- Preserve `PromptPreset`'s public API (callers in `assembler_tests.rs`, `arrival_service.rs`, `application/pipeline/phases.rs` — was `action_pipeline/phases.rs`, `domain/model/prompt_preset_tests.rs`).
- Preserve `PromptContext`'s public API (callers in `assembler.rs` / `assembler_tests.rs`).
- Do NOT touch trait impls.

Acceptance:
- Decision recorded in `## Answer` for **each** type, with the chosen option and a one-sentence justification.
- If the chosen option requires execution, the code change lands in this ticket (or a follow-up is filed and linked).
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `PromptPreset` and zero `PromptContext` violations after refactor.
- Full `build.py` green.

## What changed since this ticket was written

- **`narrative_prompt/` renamed to `prompting/`.** All paths in the original ticket body updated. The AGENTS.md structure index reflects `src/application/prompting/` (`assembler.rs`, `types.rs`, `builders/`, `utils/`, `prompt_merge.rs`, `sanitize.rs`, `token_budget.rs`).
- **`PromptContext` folded in.** Previously un-ticketed fog (map's old "Not yet specified" item). It shares `assembler.rs` with `PromptPreset`'s impl, so resolving one without the other leaves half the cluster dirty. The map's fog item for `PromptContext` is cleared by this fold.
- **`AppState` is NOT folded here.** The old fog item contemplated folding `AppState` into 07 too, but `AppState` lives in `adapters/driving/http/` — a different layer and cluster. It stays as its own fog item, to graduate to a separate task ticket when the frontier reaches the http layer.
