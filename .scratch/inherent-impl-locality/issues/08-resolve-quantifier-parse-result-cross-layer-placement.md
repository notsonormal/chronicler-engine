# 08 — Resolve QuantifierParseResult cross-layer placement

Type: grilling
Status: ready-for-agent
Blocked by: (none)

## Question

`QuantifierParseResult` (and its sibling `QuantifierResult`) violates `guardrails_inherent_impl_locality` with a cross-layer split:

- Struct defined in `domain/model/quantifier.rs` (`domain` layer)
- `impl QuantifierParseResult { parse }` (and `impl QuantifierResult { parse_with_movement }`) in `application/agents/quantifier/parser.rs` (`application` layer)

Same decision shape as ticket 07. Which layer do these types belong to?

Candidate answers:

- **A) Move `QuantifierParseResult` / `QuantifierResult` to application.** They carry parse behavior (LLM response parsing, movement extraction) and live next to the quantifier agent. Conceptually they're the agent's output type, not a pure domain model. New home: `application/agents/quantifier/quantifier_parse_result.rs` (or folder if split).

- **B) Move `parse` / `parse_with_movement` down to domain.** The parse methods take an LLM response string + maybe a movement result and construct the type. If the parsing logic is pure (no app-layer imports), it can live in domain next to the struct definition.

- **C) Extract a trait, impl in application.** Same caveat as ticket 07 — only if trait-impl locality is also enforced (out of scope).

This is HITL — run `/grilling` with the user. Note that this ticket likely resolves the same way as ticket 07 (both are cross-layer parse-result types); consider whether the two tickets should be merged before working. If merging, close this as duplicate and record decision on 07.

Constraints:
- After the decision, refactor executes in this ticket (or follow-up if SP ≥ 8).
- `build.py` green at every landed step.
- Preserve public API (callers in `parser_tests.rs`, quantifier agent code).
- Do NOT touch trait impls.

Acceptance:
- Decision in `## Answer` with chosen option + one-sentence justification.
- If merged with 07, note that here and close as duplicate.
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `QuantifierParseResult` / `QuantifierResult` violations after refactor.
- Full `build.py` green.
