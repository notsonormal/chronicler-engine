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
- Motivating artifact: `architecture-review.html` (this directory) — the
  `/improve-codebase-architecture` report with the 7 candidates, before/after
  diagrams, and evidence. Open it first every session.
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
- Standing constraint: the `arch-lint.toml:136-140` rule ("Server layer must
  not depend on storage directly; all storage access goes through
  ApplicationService") is load-bearing for candidate 2. Any deepening that
  changes how HTTP reaches storage must keep this rule satisfied (or propose a
  rule change as part of the decision).
- Each grilling ticket resolves one candidate. Do not resolve more than one
  ticket per session.

## Decisions so far

<!-- one line per closed ticket: gist + link. Empty until the first ticket resolves. -->

_None yet._

## Not yet specified

<!-- fog: suspected decisions that can't be pinned until the frontier advances -->

- **Cross-cutting migration shape.** Once the accepted deepenings are known, a
  question may graduate: do they land as one coordinated migration or as
  independent refactors? Not yet ticketable — the set of accepted decisions
  isn't known. Revisit after the storage-seam tickets (01–03) resolve.
- **Port-trait granularity for the Repository.** If candidate 2 is accepted,
  the Repository port may split into per-concern sub-traits (World / Persona /
  Settings / Preset) or stay one fat trait. Can't be sharpened until the
  storage-seam shape (ticket 02) is decided.

## Out of scope

<!-- work ruled beyond the destination; never graduates -->

- **Implementing the deepenings.** This map produces decisions and module
  shapes. Writing the refactor code is the next effort (planning + execution),
  reached only after the destination is met.
- **Rewrites not surfaced by the review.** Deepenings beyond the 7 candidates
  in `architecture-review.html` belong to a future architecture review, not
  this effort.
