# 01 — Deepen a NarrationTurn module (architecture candidate 2)

Type: grilling
Status: claimed
Blocked by: (none)

## Question

Do we commit to extracting a deep **NarrationTurn** module that owns "run one
narration turn from a prepared `GameState`" — collapsing the choreography
duplicated between `phase_narrate` and the retry paths — and if so, what is
the shape of the deepened module?

## Background

**Friction (from `architecture-review.html`, candidate 2, Strong):**
`retry_reimpersonate` and `retry_user_regen` (in
`src/application/pipeline/action_pipeline/retry.rs`, +345 lines on the branch)
duplicate `phase_narrate`'s choreography: load world bundle → resolve room →
load preset → build `PromptContext` → call narrator → save message/snapshot.
No deep module owns "run one narration turn from a prepared `GameState`"; both
`retry.rs` and `pipeline_run.rs` independently implement half of it.

Retry tests reach into `pub(crate)` helpers (`resolve_retry_target`,
`reconstruct_retry_state`, `retry_reimpersonate`, `retry_user_regen`) because
the public `retry()` flow can't be driven with replay blobs — evidence that the
interface is the wrong test surface.

**Supporting friction:** retry-mode policy is itself split —
`message_service.rs::find_retry_anchor_msg` picks the anchor, `retry.rs::
resolve_retry_target` classifies the mode (`ReNarrate` / `ReImpersonate` /
`UserRegen`). Deepening NarrationTurn should localise both.

**This is the report's top recommendation.** It is numbered 01 because a
stable narration-turn seam informs the grillings of tickets 02 (ReplaySteering
— its consumer), 03 (steering dispatcher — hands off to it), and 04
(SteeringPromptPolicy — plugs into it).

## What to decide

- Commit or reject the NarrationTurn deepening.
- If committed: the module's **interface** (method(s), params, return —
  `run(state, steering) → Outcome` is the report's sketch, to be sharpened),
  its **seam** (where it lives between `pipeline_run` and `retry`), what
  **implementation** moves behind it (bundle load, room resolve, preset
  select, prompt build, narrator call, save), and what **tests survive** at
  the new interface vs the retired `pub(crate)` helpers.
- Whether retry-mode policy (anchor + mode classification) concentrates here
  or stays split.
- Whether `phase_narrate`'s steering branches (guide / impersonate) stay in
  the phase or move behind NarrationTurn.
