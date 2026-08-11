# 07 — Resolve PromptPreset cross-layer placement

Type: grilling
Status: ready-for-agent
Blocked by: (none)

## Question

`PromptPreset` currently violates `guardrails_inherent_impl_locality` with a cross-layer split:

- Struct defined in `domain/model/prompt_preset.rs` (`domain` layer)
- `impl PromptPreset { assemble_text }` in `application/narrative_prompt/assembler.rs` (`application` layer)

The earlier grilling agreed: if a type has behavior that belongs to the application layer, the type itself belongs to the application layer. Cross-layer impls mean the **type is misplaced**, not the impl.

The decision this ticket resolves: which layer does `PromptPreset` belong to?

Candidate answers to grill between:

- **A) Move `PromptPreset` to application.** The type carries app-layer behavior (`assemble_text` uses token budget utils, preset XML builders, NPC context). It's a prompt-system concept, not a pure domain model. New home: `application/narrative_prompt/prompt_preset.rs` (or `prompt_preset/` folder if split).

- **B) Move `assemble_text` down to domain.** Strip the app-layer dependencies from `assemble_text` (pass `render_preset_xml_parts` result up, or have the caller do the joining), and keep `PromptPreset` as a pure data struct in domain. The method becomes unnecessary or moves to a free function in domain.

- **C) Extract a trait in domain, impl in application.** `domain::model::prompt_preset::PromptPresetAssembler` trait, `impl PromptPreset` stays in domain with no methods, `impl PromptPresetAssembler for PromptPreset` lives in application. NOTE: this is a trait-impl locality pattern and is governed by a separate policy (out of scope for this effort). Only choose C if the trait-impl rule would also be enforced to keep it honest.

This is HITL — the agent does not pick. Open a /grilling session with the user and surface the tradeoffs before deciding.

Constraints:
- After the decision, the refactor must be executed in this ticket (or split into a follow-up task ticket if SP ≥ 8).
- `build.py` green at every landed step.
- Preserve `PromptPreset`'s public API (callers in `assembler_tests.rs`, `arrival_service.rs`, `action_pipeline/phases.rs`, `domain/model/prompt_preset_tests.rs`).
- Do NOT touch trait impls.

Acceptance:
- Decision recorded in `## Answer` with the chosen option and a one-sentence justification.
- If the chosen option requires execution, the code change lands in this ticket (or a follow-up is filed and linked).
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `PromptPreset` violations after refactor.
- Full `build.py` green.
