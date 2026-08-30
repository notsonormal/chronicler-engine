# 01 — Deepen a narration-generation module (architecture candidate 2)

Type: grilling
Status: open
Blocked by: (none)

> Re-framed 2026-08-29 after ticket 06 resolved: the name **NarrationTurn**
> is avoided (Turn is a deprecated term). The narration-core question
> stands. Ticket 06 also confirmed that impersonate redo conceptually
> belongs with the impersonate flow — where its *code* lives is decided
> here.

## Question

Do we commit to extracting a deep module that owns "run one narration
generation from a prepared `GameState`" — collapsing the choreography
duplicated between `phase_narrate` and the new-swipe paths — and if so,
what is the shape of the deepened module?

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
stable narration-generation seam informs the grillings of tickets 02 (the
stored-inputs flow — its consumer), 03 (the dispatcher — hands off to it),
and 04 (prompt policy — plugs into it).

## What to decide

- Commit or reject the narration-generation deepening.
- If committed: the module's **interface** (method(s), params, return —
  `run(state, inputs) → Outcome` is the report's sketch, re-skinned after
  06, to be sharpened), its **seam** (where it lives between `pipeline_run`
  and `retry`), what **implementation** moves behind it (bundle load, room
  resolve, preset select, prompt build, narrator call, save), and what
  **tests survive** at the new interface vs the retired `pub(crate)`
  helpers.
- Whether new-swipe policy (anchor + last-message classification)
  concentrates here or stays split.
- Whether `phase_narrate`'s slash-command branches (guide / impersonate)
  stay in the phase or move behind the module.
- Where impersonate-redo's code lives: folded into the impersonate
  generation flow (06's conceptual answer, Marinara's single-path prior
  art), or kept in `retry.rs`.

## Grilling notes (paused findings — 06 has now resolved)

Grilling began, then surfaced the conceptual-model question that ticket 06
has now settled. Findings to carry into the resumed session:

- **Scope (recommendation, not yet confirmed).** The deepened core should
  own the narrate-and-persist prefix only (load world bundle → resolve room
  → select preset → build `PromptContext` → call narrator →
  `check_game_unchanged` → add Message → save Message + Snapshot). The
  post-narration tail diverges by path: `run_from_input` runs quantifier →
  engine commit → trigger; impersonate and user-regen stop after save
  and append the redo target; `retry_event_continuation` runs trigger
  continuation and never uses the narrate prompt path. A single "run the
  whole generation" interface cannot cover all paths without widening.
- **Naming.** 06 confirmed "Turn" stays retired. "Generation" aligns with
  the existing `generation/` namespace (`GenerationGate`, `GenerationSlot`,
  `GenerationStatus`) and with 06's settled model (the engine generates
  text; a new swipe redoes the last generation).
- **Structural finding (confirmed by 06).** Impersonate redo is closer to
  the impersonate generation flow than to the narration redo flow.
  `retry_reimpersonate` merges with the impersonate generation path, not
  with the narrate core — and this ticket's scope shrinks to the narration
  core only (ReNarrate + the narrate step of `run_from_input`), excluding
  impersonate, unless the grilling reverses it.
