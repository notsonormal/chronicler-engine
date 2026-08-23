# 05 — Single injection point for narrative-voice setting (architecture candidate 5)

Type: grilling
Status: open
Blocked by: (none)

## Question

Do we commit to routing narrative-voice application through a single owner —
the prompt-construction module — so `set_narrative_voice` stops being called in
both `assembler.assemble` and `arrival_service`, and if so, what is the shape
of the deepened seam?

## Background

**Friction (from `architecture-review.html`, candidate 5, Worth exploring):**
`TemplateVars::set_narrative_voice` (in
`src/domain/model/template.rs`, +46 lines on the branch) is called in two
places:

- `src/application/prompting/assembler.rs` — `assemble` applies it from
  `AppSettings`.
- `src/application/arrival_service.rs` — `ArrivalTaskContext::run` applies it
  separately.

The setting is therefore not encapsulated by the assembler; any other caller
that builds a `PromptContext` must remember to apply it.
`TemplateVars::from_persona` defaults to third-person-past, so forgetting the
call silently produces wrong voice. A leaking seam.

**This candidate is independent and small** — the report calls it "a safe
parallel win." It has no relationship to tickets 01–04 and can be grilled
first or last without affecting them.

## What to decide

- Commit or reject the single-injection-point deepening.
- If committed: which module owns voice application (the assembler, or a
  shared `TemplateVars` builder both callers route through), the **interface**
  change (does `PromptContext` construction apply voice, or does the
  assembler's `assemble` remain the sole caller?), and what **tests** guard
  against the silent-wrong-voice failure mode.
- Whether `arrival_service` routes through the assembler for its prompt, or
  whether a thinner shared builder is warranted (avoiding the assembler's full
  fit/budget path for the arrival case).
