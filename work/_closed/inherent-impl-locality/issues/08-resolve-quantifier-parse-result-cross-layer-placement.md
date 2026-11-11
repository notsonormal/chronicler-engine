# 08 — Resolve QuantifierParseResult cross-layer placement

Type: grilling
Status: resolved
Blocked by: (none)

> **Resolved by events.** The cross-layer impls were removed before this ticket
> was worked; no grilling was held. See `## Answer`.

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

## Answer

**Resolved by events — no grilling held.** The cross-layer impls this ticket was
opened to decide between (`impl QuantifierParseResult` and
`impl QuantifierResult` in `application/agents/quantifier/parser.rs`) no longer
exist. Current state on `main`:

- **`QuantifierParseResult`**: def in `domain/model/quantifier.rs`; the sole
  remaining `impl QuantifierParseResult` is co-located in the same file
  (`quantifier.rs:60`). `impl_path == def_path` → clean.
- **`QuantifierResult`**: def in `domain/model/quantifier.rs`; it now has **no
  inherent impls anywhere** in `src/` (`grep -rn "impl QuantifierResult" src/`
  returns nothing). Vacuously clean.

The parser still exists (relocated to `application/agents/quantifier/utils/parser.rs`
per commit `4c704bd`), but it no longer carries `impl` blocks for either type —
the parse logic either moved into the domain-side impl or became free
functions. Either way, the locality violation is gone.

The layer-placement decision (A/B/C) this ticket was meant to grill through
became moot: option B (behavior down in domain, or no inherent impl) is what
`main` ended up with, but it was not a conscious choice made under this ticket.
If a future change re-introduces an application-layer `impl QuantifierParseResult`,
the decision re-opens — but that is a new effort, not a resumption of this one.
