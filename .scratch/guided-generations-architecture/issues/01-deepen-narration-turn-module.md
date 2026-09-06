# 01 — Deepen a narration-generation module (architecture candidate 2)

Type: grilling
Status: resolved
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

## Answer

Resolved 2026-08-30 by grilling (rounds 1–3: Q1–Q7). The deepening is
**committed**. Evidence was re-verified against the tree at `67c7824`
before grilling (the review artifact predates the narrator-modes work):
the choreography duplication, the `pub(crate)` test surface, and the
anchor/classification split all hold; the narrator-modes work sharpened
the duplication, since `phase_narrate` now resolves Impersonate steering
itself and `retry_reimpersonate` re-implements that resolution.

**Settled decisions:**

1. **Commit; scope = the narrate-and-persist prefix** (Q1→A). The prefix:
   load world bundle → resolve room → consume the preset choice → build
   `PromptContext` → call narrator → `check_game_unchanged` → add Message
   or Swipe → save Message + Snapshot. The post-narration tail
   (quantifier → engine commit → trigger) stays with the calling paths —
   it diverges legitimately by path. Excluded callers:
   `retry_event_continuation` (never enters the narrate path) and
   `arrival_service` (ticket 05's territory).
2. **Interface** (Q4→accept):
   `run(state: &mut GameState, inputs: GenerationInputs) -> Result<NarrationOutcome, PhaseError>`.
   - `GenerationInputs` is caller-resolved: input text, optional guide,
     optional Impersonate steering (direction + preset id). On a redo the
     caller reads the Swipe's stored inputs; the core never reads
     `retry_target` for steering.
   - `NarrationOutcome` = narration text + backend name + model name (the
     main path's tail needs all three).
   - No redo flag: redo-ness rides in `state.narrative.retry_target`;
     `push_message` and `save_message_and_snapshot` already turn it into
     a Swipe.
   - Errors return as `PhaseError`; the caller runs the existing
     `finalize_phase_error` helper, replacing the inline finalization
     `retry.rs` does today.
   - Consequence: `phase_narrate`'s guide/impersonate branches
     (`resolve_guide`/`resolve_impersonate`, including the replay
     fallback) move out to the callers as input preparation; the core is
     branch-free. Ticket 04 decides who produces the preset choice; this
     module consumes it.
3. **Impersonate redo folds into the impersonate generation flow**
   (Q2→A, Q5→A). `retry_reimpersonate` dissolves as a separate
   implementation; the redo re-runs the impersonate generation from the
   Swipe's stored inputs through the same path as a fresh Impersonate,
   and runs the full tail (quantifier → engine commit → trigger). This
   changes today's stop-after-save redo behavior — believed accidental,
   no recorded reason (inferred). Marinara prior art: Regenerate replays
   stored inputs back through the impersonate path; there is no separate
   retry-impersonate. UserRegen stays tail-less: a plain user input
   never gets a tail in the fresh world either (its responding Narration
   does). Swipe snapshots are captured pre-tail at message-add time in
   every path, so `switch_swipe` semantics are unchanged.
4. **No redo-policy seam** (Q3, dissolved on the user's correction).
   Ticket 06 settled redo as internal plumbing; classification falls out
   of the last message's type. The anchor query stays in
   `message_service` as a history query; classification and
   reconstruction stay `pub(crate)` plumbing in the pipeline's redo entry
   path. The review's "localise both" supporting friction is rejected on
   those grounds.
5. **Name and placement** (Q6→A): new module `narration_generation`
   under `src/application/pipeline/`, beside `pipeline_run.rs`. Not
   `application/generation/` — that namespace holds per-game gating
   (`GenerationGate`, `GenerationSlot`), not generation mechanics.
6. **Tests** (Q7→A): new tests drive `narration_generation::run`
   directly — one per prefix failure mode (bundle load, room not found,
   preset missing, narrator error, cancellation, save failure) plus one
   per input kind (free action, Guided Generation, Impersonate).
   Redo-mode coverage moves to flow tests through the public `retry()`
   entry, one per mode (narration, impersonate, user-regen, event). All
   19 direct `pub(crate)` helper tests retire (13 choreography +
   6 plumbing); the 29 flow tests survive unchanged. Execution must
   extend `test_support` fixtures to build replay-carrying swipes so the
   impersonate and user-regen modes can be driven through `retry()` —
   the pre-existing gap the review flagged.

**Consequences for the map:** weakens ticket 02's remaining case — with
caller-resolved inputs and no redo seam, the stored-inputs flow is plain
data flow (Swipe stores inputs → caller reads them → core consumes
them). Ticket 03's dispatcher consumes the same classification the redo
entry keeps. Ticket 04 gains a fixed consumer: `narration_generation`
consumes the preset choice it produces. No fog graduates; "coordinated
vs independent landing" stays unspecifiable until 02–05 resolve.
