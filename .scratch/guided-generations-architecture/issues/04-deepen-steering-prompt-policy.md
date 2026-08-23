# 04 — Deepen a SteeringPromptPolicy module (architecture candidate 4)

Type: grilling
Status: open
Blocked by: (none)

## Question

Do we commit to a deep **SteeringPromptPolicy** module that owns "what a
steering surface does to the prompt" — returning the preset choice and layer
set for a given steering — so impersonate's prompt shape stops being split
between `assembler.rs` and `pipeline_run.rs`, and if so, what is the shape of
the deepened module?

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

**Relationship to ticket 01 (NarrationTurn).** The report says this policy
"layers on top of a stable narration-turn seam." This ticket can be grilled
independently — the policy's output (preset + layers) is self-contained — but
the grilling should confirm where the policy's output is consumed
(NarrationTurn, if 01 is accepted).

**Relationship to ticket 02 (ReplaySteering).** The guide/impersonate
mutual-exclusion rule could live here or in ReplaySteering. Decide which owns
it.

## What to decide

- Commit or reject the SteeringPromptPolicy deepening.
- If committed: the module's **interface** (the report sketches
  `resolve(steering) → PresetChoice + LayerSet`), its **seam** (between the
  pipeline and the assembler), what **implementation** moves behind it (preset
  swap, layer drop, mutual-exclusion), and what **tests** cover "what does
  steering X do to the prompt" at one interface.
- Whether `PromptContext` drops its `guide` / `impersonate` fields (the policy
  feeds it pre-resolved layer/preset data instead).
- Where the guide/impersonate mutual-exclusion rule lives — here or in
  ReplaySteering (ticket 02).
