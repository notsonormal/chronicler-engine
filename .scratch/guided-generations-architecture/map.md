# Map: Guided-Generations Architecture (pre-merge)

## Destination

For each of the 5 branch-introduced shallows surfaced by the architecture
review of `guided-generations`, a **decision**: commit to the deepening with a
defined module shape (interface, seam, what sits behind it, what tests
survive), or reject it with a load-bearing reason. The map is done when every
candidate is decided and the set is ready to hand off to pre-merge execution —
nothing left to decide before the branch either lands clean or carries a
defined refactor plan.

This is a **plan-don't-do** effort. Tickets resolve decisions and module
shapes, not code. Writing the accepted refactors is the next effort
(pre-merge execution), reached only after the destination is met.

## Notes

- Tracker: local markdown (`.scratch/guided-generations-architecture/`). See
  `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md`.
- Motivating artifact: `architecture-review.html` (this directory) — the
  `/improve-codebase-architecture` report with the 5 candidates, before/after
  diagrams, and evidence. Open it first every session.
- Branch under review: `guided-generations` (at `1898a53`).
- **Stale snapshot (2026-08-29).** The review artifact was produced at
  `1898a53`, before the narrator-modes-and-options work landed on this branch
  (tickets 05, 13, 14: posture relocation to World/Game, prompt-preset
  mode-allow flags, registry list reshape, `system_if_default` seed). Before
  grilling, re-verify each candidate's evidence against the current tree.
  Known affected: candidate 5 (narrative-voice injection) — the global
  narrative-voice setting was removed and injection centralized in
  `PromptAssembler::assemble`; candidate 1 (NarrationTurn) and candidate 4
  (SteeringPromptPolicy) touch `assembler.rs`/`pipeline_run.rs`, which the
  remaining narrator-modes tickets would modify further — resolve the
  architecture decisions before that implementation continues.
- Skills every session should consult: `/grilling`, `/domain-modeling`,
  `/codebase-design` (for the deep-module vocabulary: module, interface,
  implementation, depth, seam, adapter, leverage, locality — use these terms
  exactly), `/improve-codebase-architecture` (the source skill, step 3 grilling
  loop).
- Domain language: `CONTEXT.md` is the single source of truth for terms
  (Steering, Guided Generation, Narrator Action, Impersonate, Action, Action
  Pipeline, Message, Swipe, Snapshot, Narrative). Use these names. If a
  deepened module needs a term not in `CONTEXT.md`, add it via
  `/domain-modeling` during the grilling.
- **Relationship to the general architecture map.** A separate effort,
  `.scratch/architecture-deepening/` (7 candidates, general/main-branch
  architecture, all open), exists. It is **untouched** by this effort. Two of
  its tickets (04 ActionPipeline turn-lifecycle, 06 PromptAssembler helpers)
  touch related areas but ask *different questions* — general file-slicing and
  helper absorption, not the branch-introduced steering shallows. A decision
  here may sharpen those; it does not resolve them.
- **Ticket ordering.** Tickets are numbered in recommended resolution order,
  not candidate order. All seven tickets are resolved (06, then 01–05, then
  07). **Map complete (2026-08-30): the destination is met — every
  candidate is decided and the handoff shape is settled. The next effort is
  pre-merge execution.** Its spec: [docs/plans/guided-generations-pre-merge-execution.md](../../docs/plans/guided-generations-pre-merge-execution.md).
- No ADRs exist in `docs/adr/`. If a candidate is rejected with a
  load-bearing reason that future reviews should not re-suggest, offer an ADR
  during that ticket's grilling.
- Each grilling ticket resolves one candidate. Do not resolve more than one
  ticket per session.

## Decisions so far

<!-- one line per closed ticket: gist + link. Empty until the first ticket resolves. -->

- [06 — Revise the branch's conceptual model](issues/06-revise-branch-conceptual-model.md)
  — Steering, replay blob, and retry retired as concepts; Narrator Action
  retired entirely (command, `Action::Narrator`, `MessageType::Narrator` —
  removal is execution work). Settled model: the engine generates text;
  player inputs are free actions / Guided Generation / Impersonate (all
  slash-command inputs transient); every Message has Swipes; a Swipe stores
  the inputs that produced it; a new swipe redoes the last generation.
  Review terms NarrationTurn / ReplaySteering / SteeringPromptPolicy
  avoided. Tickets 01, 02, 04 unblocked and re-framed; 03 re-framed.
- [01 — Deepen a narration-generation module](issues/01-deepen-narration-turn-module.md)
  — Committed. New module `narration_generation` under
  `src/application/pipeline/` owns the narrate-and-persist prefix behind
  `run(state, GenerationInputs) → Result<NarrationOutcome, PhaseError>`;
  callers resolve inputs (branch-free core, no redo flag). Impersonate redo
  folds into the impersonate flow and gains the full tail (behavior
  change); `retry_reimpersonate` dissolves. No redo-policy seam — anchor
  query stays in `message_service`, classification/reconstruction stay
  `pub(crate)` plumbing. 19 `pub(crate)` helper tests retire; the core is
  driven directly, redo modes via `retry()` flow tests.
- [02 — Does the stored-generation-inputs flow need an owning module?](issues/02-deepen-replay-steering-module.md)
  — Rejected (no ADR). After 06 and 01 the stored-inputs flow is plain
  data flow; the deletion test shows an owner module would be shallow.
  Attachment and redo-swipe inheritance stay on `GameState::push_message`;
  the guide/impersonate mutual-exclusion question passed to ticket 04.
  Execution note for 01: the `pending_replay` staging buffer's removal is
  01 execution work.
- [03 — Collapse action entry methods into one dispatcher](issues/03-collapse-steering-entry-dispatcher.md)
  — Committed. One public gated `process_action(Action)` on
  `ActionPipeline`; payloads ride the existing `Action` enum. Handler
  shrinks to parse-plus-call (keeps parse for the text-check path);
  variant→replay-inputs mapping and the empty-input→continue rule move
  behind the dispatcher. Sync runner drops to `pub(crate)`; gated/sync
  modes stay distinct. Impersonate preset stays pinned at entry (06's
  swipe-stores-inputs rule); relocation to a prompt-policy module is
  ticket 04's call. Tests at both layers: new dispatcher unit tests for
  Guide/Impersonate stored inputs, plus extended HTTP integration
  assertions; narrator tests retire with 06 execution.
- [04 — Deepen a prompt-policy module for slash commands](issues/04-deepen-steering-prompt-policy.md)
  — Rejected (no ADR). After 01/03/06 the splits shrink to one preset
  branch in `narration_generation`'s prefix, two lines in the assembler,
  and one defensive line — the deletion test fails. `PromptContext`
  keeps its `guide`/`impersonate` fields; the entry preset pin stays
  behind 03's dispatcher; exclusion keeps 02's enforcement (the
  defensive line moves to caller-side input preparation under 01's
  execution). Execution note for 01: merge the two preset loaders.
  Observation: `allowed_modes` is enforced at HTTP selection time only,
  not at generation time.
- [05 — Single injection point for narrative-voice setting](issues/05-single-narrative-voice-injection.md)
  — Committed, scoped to what survives narrator-modes ticket 06. The
  prompt-construction module is the sole owner of voice application:
  `TemplateVars::set_narrative_voice` is deleted, the stamper becomes a
  module-private helper in `assembler.rs`, and arrival's (already-dead)
  stamping is deleted; the ownership test proves callers cannot inject
  voice. The voice *source* (game posture) is owned by the narrator-modes
  map; a threading note was appended to its ticket 06. Fog graduated: the
  accept/reject split is known, so both handoff questions became ticket 07.
- [07 — Pre-merge handoff: landing shape and merge strategy](issues/07-pre-merge-handoff-shape.md)
  — One single commit carries all four execution pieces (Narrator Action
  removal, 05's voice cleanup, 03's dispatcher, 01's
  `narration_generation` module with 02's and 04's folded execution
  notes); merge gates on `python build.py` green with that commit landed.
  No follow-up issue for the rejected shallows; the `allowed_modes`
  pinned-preset edge went to narrator-modes ticket 15. This work lands
  first; the narrator-modes effort resumes after the merge. **Map
  complete — destination met; hand off to pre-merge execution.**

## Not yet specified

<!-- fog: suspected decisions that can't be pinned until the frontier advances -->

<!-- both patches graduated to ticket 07 on 2026-08-30: the accept/reject
     split and the accepted set are now known -->

## Out of scope

<!-- work ruled beyond the destination; never graduates -->

- **Implementing the deepenings.** This map produces decisions and module
  shapes. Writing the refactor code is the next effort (pre-merge execution),
  reached only after the destination is met.
- **The general architecture map's candidates.** The 7 candidates in
  `.scratch/architecture-deepening/` belong to a separate effort (general
  main-branch architecture). This map does not resolve, supersede, or close
  them.
- **The voice source of truth.** Whether posture is world-current or
  per-game is owned by the narrator-modes map (its tickets 01, 05, 06),
  not by this effort.
- **Shallows not surfaced by this branch's review.** Branch-independent
  architecture issues belong to the general map, not here.
