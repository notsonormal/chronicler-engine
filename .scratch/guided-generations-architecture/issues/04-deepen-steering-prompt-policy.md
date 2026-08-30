# 04 — Deepen a prompt-policy module for slash commands (architecture candidate 4)

Type: grilling
Status: open
Blocked by: (none)

> Re-framed 2026-08-29 after ticket 06 resolved: the name
> **SteeringPromptPolicy** is avoided (Steering is retired). The
> prompt-policy question stands: impersonate's prompt shape is still split
> across two files and a builder method.

## Question

Do we commit to a deep module that owns "what a slash command does to the
prompt" — returning the preset choice and layer set for a given command —
so impersonate's prompt shape stops being split between `assembler.rs` and
`pipeline_run.rs`, and if so, what is the shape of the deepened module?

## Background

**Friction (from `architecture-review.html`, candidate 4, Worth exploring):**
"What impersonate does to the prompt" is split across three places:

- `src/application/pipeline/pipeline_run.rs` (`phase_narrate`) — selects the
  impersonate preset (swaps the system preset).
- `src/application/prompting/assembler.rs` (`render_persona_layer`) — drops the
  player-character reference-card layer on impersonate.
- `PromptContext::with_impersonate` — clears `guide`, encoding the
  guide/impersonate mutual-exclusion rule inside the prompt-context builder.

Neither `assembler.rs` nor `pipeline_run.rs` is deep on its own for this
concern; they form a tightly coupled seam. A change to impersonate's prompt
shape touches both. `PromptContext` now carries both `guide` and `impersonate`
fields, exposing prompt-construction details to callers.

**Relationship to ticket 01 (narration-generation module).** The report
says this policy "layers on top of a stable narration-turn seam." This ticket
can be grilled independently — the policy's output (preset + layers) is
self-contained — but the grilling should confirm where the policy's output is
consumed (the deepened module, if 01 is accepted).

**Relationship to ticket 02 (stored-inputs flow).** The guide/impersonate
mutual-exclusion rule could live here or in ticket 02's module, if that
module survives its re-justification. Decide which owns it.

## What to decide

- Commit or reject the prompt-policy deepening.
- If committed: the module's **interface** (the report sketches
  `resolve(steering) → PresetChoice + LayerSet` — re-skin as
  `resolve(action)` against the settled model), its **seam** (between the
  pipeline and the assembler), what **implementation** moves behind it
  (preset swap, layer drop, mutual-exclusion), and what **tests** cover
  "what does slash command X do to the prompt" at one interface.
- Whether `PromptContext` drops its `guide` / `impersonate` fields (the
  policy feeds it pre-resolved layer/preset data instead).
- Where the guide/impersonate mutual-exclusion rule lives — here or in
  ticket 02's module, if it survives.
