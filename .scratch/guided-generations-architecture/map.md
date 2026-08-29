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
  not candidate order. Ticket 06 (revise the branch's conceptual model) is
  now the recommended starting point: it blocks 01, 02, and 04, which all
  assume the branch-introduced Steering / replay-blob / retry framing. Once
  06 resolves, 01 (narration core), 02 (replay carrier), and 04 (steering
  prompt policy) resume; 03 (steering entry dispatcher) and 05 (narrative
  voice) may also relate and are clarified by 06's grilling.
- No ADRs exist in `docs/adr/`. If a candidate is rejected with a
  load-bearing reason that future reviews should not re-suggest, offer an ADR
  during that ticket's grilling.
- Each grilling ticket resolves one candidate. Do not resolve more than one
  ticket per session.

## Decisions so far

<!-- one line per closed ticket: gist + link. Empty until the first ticket resolves. -->

_None yet._

## Not yet specified

<!-- fog: suspected decisions that can't be pinned until the frontier advances -->

- **Coordinated vs independent landing.** Once the accepted deepenings are
  known, a question may graduate: do the accepted refactors land as one
  coordinated pre-merge refactor, or as independent commits? Not yet
  ticketable — the set of accepted decisions isn't known. Revisit after the
  NarrationTurn ticket (01) and its dependents resolve.
- **Merge strategy if some candidates are rejected.** If one or more
  candidates are rejected, does the branch merge with the remaining shallows
  and a follow-up issue, or block on the accepted set only? Can't sharpen
  until the accept/reject split is known.

## Out of scope

<!-- work ruled beyond the destination; never graduates -->

- **Implementing the deepenings.** This map produces decisions and module
  shapes. Writing the refactor code is the next effort (pre-merge execution),
  reached only after the destination is met.
- **The general architecture map's candidates.** The 7 candidates in
  `.scratch/architecture-deepening/` belong to a separate effort (general
  main-branch architecture). This map does not resolve, supersede, or close
  them.
- **Shallows not surfaced by this branch's review.** Branch-independent
  architecture issues belong to the general map, not here.
