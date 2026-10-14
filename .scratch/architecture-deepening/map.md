# Map: Architecture Deepening

## Destination

For each deepening opportunity surfaced by the architecture review, a
**decision**: commit to the deepening with a defined module shape (interface,
seam, what sits behind it, what tests survive), or reject it with a
load-bearing reason. The map is done when every candidate is decided and the
set is ready to hand off to implementation planning — nothing left to decide
before someone goes and does the work.

This is a **plan-don't-do** effort. Tickets resolve decisions, not code.
Execution of accepted deepenings is the next effort's job, not this map's.

## Notes

- Tracker: local markdown (`.scratch/architecture-deepening/`). See
  `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md`.
- Motivating artifacts (`assets/`), both from
  `/improve-codebase-architecture`. Open the one your ticket cites first:
  - `assets/architecture-review.html` (2026-08-16): candidates 1–7 → tickets 01–08.
  - `assets/architecture-review-2026-10-04.html`: second pass. A drift table for
    tickets 01–08 plus new candidates A–I → tickets 09–17
    (A=09, B=10, C=11, D=12, E=13, F=14, G=15, H=16, I=17).
- 2026-10-04 re-check: none of tickets 01–08 was resolved, and none of their
  named files moved. Each ticket body now carries a `Current state
  (2026-10-04)` section. Where it conflicts with Background, trust it.
- Skills every session should consult: `/grilling`, `/domain-modeling`,
  `/codebase-design` (for the deep-module vocabulary: module, interface,
  implementation, depth, seam, adapter, leverage, locality — use these terms
  exactly), `/improve-codebase-architecture` (the source skill, step 3 grilling
  loop).
- Domain language: `CONTEXT.md` is the single source of truth for terms
  (Game, World, Persona, Character, Scenario, Action, Action Pipeline,
  Trigger, Narrative, Quantifier, Agent, Message, Swipe, Snapshot). Use these
  names, not ad-hoc ones. If a deepened module needs a term not in
  `CONTEXT.md`, add it via `/domain-modeling` during the grilling.
- No ADRs exist in `docs/adr/`. **But** prior wayfinder decisions in
  `.scratch/inherent-impl-locality/` already touched the Storage seam
  (tickets 03 and 11): the `backend/` folder was flattened into `storage/`
  root, single-file consolidation of the 13 `impl Storage` blocks was
  **rejected as undesirable**, and the inherent-impl-locality rule stays
  **name-only** (folder cohesion is review policy, not enforced). Candidate
  for the Storage trait (ticket 02) must not re-litigate these — ticket 01
  researches them as constraints.
- Standing constraint: the `arch-lint.toml` rule at lines 138-142 ("Server layer must
  not depend on storage directly; all storage access goes through
  ApplicationService") is load-bearing for candidate 2. Any deepening that
  changes how HTTP reaches storage must keep this rule satisfied (or propose a
  rule change as part of the decision).
- Each grilling ticket resolves one candidate. Do not resolve more than one
  ticket per session.
- **Ticket choice overrides the tracker default.** `docs/agents/issue-tracker.md`
  says "first by number wins". For this effort, pick by value instead: take the
  highest-ranked open, unblocked, unclaimed ticket in **Priority** below. Ticket
  numbers record creation order only.

## Priority

Strength comes from the review that raised each candidate (`Strong` >
`Worth exploring` > `Speculative`). Order within a tier is by leverage and by
what each decision unblocks. Re-rank here when a resolution changes the
picture. Do not edit the reports.

| Rank | Ticket | Strength | Source | Why this position |
|---|---|---|---|---|
| 1 | [Research the Storage seam constraints](issues/01-research-storage-seam-constraints.md) | (prerequisite) | 2026-08-16 | AFK research; gates the Storage pair. Can run alongside any grilling |
| 2 | [Deepen Storage into a StorageBackend trait](issues/02-deepen-storage-into-backend-trait.md) | Strong — top pick | 2026-08-16 · 1 | Widest interface in the codebase, and wider since 2026-08-16. Blocked by 01 |
| 3 | [Share one Agent call core between Options and Quantifier](issues/12-share-agent-call-core.md) | Strong | 2026-10-04 · D | Two copies of the same Agent body. Reshape around the hook traits of turn-state-model D-29 (note in the ticket) |
| 4 | [One generation-status strip for the action area and the poll](issues/11-single-generation-status-strip.md) | Strong | 2026-10-04 · C | Three copies of one display rule, one of them skipping `GameViewQuery` |
| 5 | [Replace CRUD services with a Repository port](issues/03-replace-crud-services-with-repository-port.md) | Strong | 2026-08-16 · 2 | Blocked by 02. The new game-state design's repositories are its first instance (note in the ticket) |
| 6 | [One definition of the connection card](issues/14-single-connection-card.md) | Strong (small) | 2026-10-04 · F | Clear fix, low leverage; a quick session |
| 7 | [One fragment error policy for the HTTP adapter](issues/15-single-fragment-error-policy.md) | Worth exploring | 2026-10-04 · G | Observable HTTP contract; needs a spec check |
| 8 | [Bootstrap creates and finds Games only through GameCatalogue](issues/16-bootstrap-games-through-game-catalogue.md) | Worth exploring | 2026-10-04 · H | Easier once 01's constraints exist |
| 9 | [Deepen PromptAssembler, absorb shallow helpers](issues/06-deepen-prompt-assembler-absorb-helpers.md) | Worth exploring | 2026-08-16 · 5 | Slightly better since 2026-08-16. Ticket 10's verdict deletes arrival, the only production caller of `build_narration_prompt` |
| 10 | [Flatten LLM transport/utils and share the result mapping](issues/17-flatten-llm-transport.md) | Speculative | 2026-10-04 · I | Low leverage. Consider reject |

Tickets 13 (registry) and 07 (preset) were decided by the turn-state-model
verdict on 2026-10-14, so the old exception for 12 and 13 no longer applies.

## Decisions so far

<!-- one line per closed ticket: gist + link. -->

- [Consolidate the ActionPipeline turn lifecycle](issues/04-consolidate-action-pipeline-turn-lifecycle.md) — rejected, superseded: the new game-state design replaces the pipeline with one Job and one `generate` (turn-state-model ticket 24).
- [Split MessageService by concern](issues/05-split-message-service-by-concern.md) — rejected, superseded: `MessageService` is deleted in the new design.
- [Fold generation slot and guard into GenerationGate](issues/08-fold-generation-slot-guard-into-gate.md) — rejected, superseded: `gate.rs` becomes `GameWriter` and `JobWriter`; slot and guard are deleted.
- [Give the input buffer named generation transitions](issues/09-input-buffer-generation-transitions.md) — rejected, superseded: the status is in memory, and `JobWriter::step` is its only writer.
- [Route arrival narration through the one narration-generation path](issues/10-route-arrival-through-narration-generation.md) — rejected, superseded: arrival is deleted (one create unit, no LLM opening).
- [Retire or justify the Agent registry](issues/13-retire-or-justify-agent-registry.md) — decided by turn-state-model ticket 24: Agents attach at four fixed hooks, one trait and output type per hook, run by `AgentRunner`.
- [Move Agent preset resolution into AgentContext](issues/07-move-quantifier-preset-into-agent-context.md) — decided by turn-state-model ticket 24: the step loads each Agent's settings and preset and passes them as values.

## Not yet specified

<!-- fog: suspected decisions that can't be pinned until the frontier advances -->

- **Cross-cutting migration shape.** Once the accepted deepenings are known, a
  question may graduate: do they land as one coordinated migration or as
  independent refactors? Not yet ticketable — the set of accepted decisions
  isn't known. Revisit after the storage-seam tickets (01–03) resolve.
- **Agent cluster shape.** Tickets 13 and 07 were decided by the
  turn-state-model verdict (Agent hooks; settings as values). Only 12 (the
  call core inside each Agent) is left, reshaped around the hook traits.
- **HTTP adapter cohesion.** Tickets 11 (status strip), 14 (connection
  card) and 15 (error policy) all point at `AppState` (21 public items, 7
  `render_*` methods that repeat one shape). A separate `AppState` interface
  question may surface after 15. Not sharp enough to ticket yet.
- **Port-trait granularity for the Repository.** If candidate 2 is accepted,
  the Repository port may split into per-concern sub-traits (World / Persona /
  Settings / Preset) or stay one fat trait. Can't be sharpened until the
  storage-seam shape (ticket 02) is decided.

## Out of scope

<!-- work ruled beyond the destination; never graduates -->

- **Implementing the deepenings.** This map produces decisions and module
  shapes. Writing the refactor code is the next effort (planning + execution),
  reached only after the destination is met.
- **Rewrites not surfaced by the reviews.** Deepenings beyond the candidates
  in `assets/architecture-review.html` (1–7) and
  `assets/architecture-review-2026-10-04.html` (A–I) belong to a future architecture
  review, not this effort.
