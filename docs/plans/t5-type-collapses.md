# T5: Type Collapses (A3 + A6)

**Parent Plan:** [abstraction-fixes-followup-superplan.md](./abstraction-fixes-followup-superplan.md)
**Status:** Planning — ready
**Date:** 2026-06-28
**Depends on:** none (mechanical; can run alongside other tracks opportunistically)
**Blocks:** none
**Priority:** P2
**Findings owned:** A3 (false seam — architecture-lens candidate #6), A6, N14

---

## Summary

Two type collapses deferred in source plan as "rippling struct field + many call sites."

### A3: `Confidence` vs `QuantifierConfidence`

- `model/agent.rs:76` `pub enum Confidence { High, Medium, Low }`
- `model/quantifier.rs:8` `pub enum QuantifierConfidence { High, Medium, Low }`
- Bidirectional `From` impls (`quantifier.rs:14-50`)
- **75 references** across `application/`, `engine/`, `model/`. Identical variants; module boundary is the only reason for split.

Architecture-lens reframe: **false seam**. Two modules, bidirectional `From` impls each trivially wrapping the other. Per DEEPENING.md "one adapter = hypothetical seam" — there was never a domain reason for two types.

### A6: `TemplateVars`

- `model/template.rs:5` `pub struct TemplateVars { pub user: String }` + `pub fn render_template(text, vars)`. One field, one function, one consumer, 6+ callers (`assembler.rs:169`, `bootstrap/state.rs:28`, `bootstrap/scenario.rs:22`, `context.rs:124`, `quantifier/prompt.rs:21`, `types.rs:36`).

## Key Changes

### A3

- Delete `QuantifierConfidence`, use `Confidence` everywhere. Update ~75 refs mechanically across ~15 files.
- Decide placement: keep `Confidence` in `model/agent.rs`, or split out to new `model/confidence.rs`.
- Derive `Ord` on `Confidence` (covers N14) so `StatePatch::merge` simplifies to `min()`.

### A6

- Delete `TemplateVars`; replace signature `render_template(text, user: &str)`. Accept keeping the struct only if a 2nd field is on the roadmap (it is not per current plan).

## Decisions to Lock

- Move `Confidence` to new `model/confidence.rs`, or keep in `agent.rs`?

## Blast Radius

A3 = 75 refs across ~15 files. A6 = 6+ callers. Both mechanical. Risk: test fixtures may rely on the type names.

## Verification

- `python build.py` — fmt + clippy + tests + coverage must pass clean.
- After A3 collapse, search for any remaining `QuantifierConfidence` references — should be zero.
- After N14 (`Ord` derive), run `StatePatch::merge` tests — should pass with `min()` simplification.
- After A6 collapse, run all `render_template` callers — 6 sites must compile unchanged signature.

## Pre-Implementation Checklist

- [ ] Confirm no test fixture relies on `QuantifierConfidence` type name (correlated with A3 removal).
- [ ] Confirm `Ord` derive produces correct `High > Medium > Low` ordering before using in `StatePatch::merge`.
