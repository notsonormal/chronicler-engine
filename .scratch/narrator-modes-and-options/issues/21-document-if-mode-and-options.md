# Task: Document IF mode and options

Type: task
Status: pending
Blocked by: 20

## Question

Last fog toward the destination: the documentation for the two shipped features. What does the doc tree need for Interactive Fiction narrator mode and options-autogeneration, and where does each piece live?

Seed facts from the map:

- **CONTEXT.md terms** — add/refresh the ubiquitous-language entries: Narrator Mode (Novel / InteractiveFiction, per-game posture inherited from the world, ticket 05/06/07), the per-mode preset bundle + `allowed_modes` selection gating (tickets 13/14), and Options (the offered set, `MessageType::Input` semantics, always-on toggle vs on-demand `/options`, tickets 04/09/10/11).
- **Diataxis reference** — a reference doc (or docs) for IF mode + options: what the narrator may do under the IF posture (the inverted Agency Rule, ticket 02), how the options dock + slash item behave, and how both interact with steering (guide/impersonate stay mode-agnostic, ticket 13's ruling). Cross-reference, don't restate, `docs/diataxis/reference/narrative/ai_steering.md` and the specs (`docs/specs/narrator_mode.md`, `docs/specs/options.md`, `docs/specs/browser_options.md`).
- **DOC anchors** — the doc-anchor standard applied to the new code surfaces (options agent, posture relocation, preset mode flags) per the structure guardrails.
- Known follow-the-docs items recorded by earlier tickets: the HTTP-tier `/options` empty-history rejection surfaces as a 500 (ticket 12, pinned as-is) — decide whether the reference doc mentions it or a fix ticket is warranted.

Scope the doc set first (grill if the split isn't obvious), write it in STE, keep the compass test honest (reference defers to source; no code-indexing), and land with `python build.py validate-docs` + full gate green.
