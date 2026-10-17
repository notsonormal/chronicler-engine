# 03 — Replace CRUD services with a Repository port

Type: grilling
Status: resolved
Blocked by: 02
Assignee: (unclaimed)

## Question

Do we commit to replacing the fleet of one-method CRUD application services
(`PersonaCatalogue`, `SettingsService`, `PromptPresetService`,
`WorldCatalogue`) with a single `Repository` port that `Storage` implements —
and if so, what is the shape of the deepened module?

## Background

This is **candidate 2** of the architecture review. See
`assets/architecture-review.html` for the mass diagram and evidence.

The friction: `arch-lint.toml` (lines 138-142) forbids the HTTP layer from importing
storage ("Server layer must not depend on storage directly; all storage
access goes through ApplicationService"). So the codebase grows thin services
whose entire job is to forward to `Storage` — `PersonaCatalogue`'s own header
admits it is "intentionally a one-method seam." Their interfaces are nearly as
large as their implementations (one constructor + one delegating method each).

The deepening: a `Repository` port trait satisfies the lint rule directly —
HTTP depends on the trait, `Storage` implements it — and retires the
delegates. The deletion test is *partial*: the delegates reappear in handlers
only because of the lint rule; a port removes that reason.

## What this ticket resolves

- **Commit or reject.** Is the Repository port worth it, or do the per-concern
  services earn their locality?
- **Interface shape.** One fat `Repository` trait vs per-concern sub-traits
  (World / Persona / Settings / Preset). This is also flagged as fog on the
  map — it graduates here.
- **Lint-rule interaction.** Does `arch-lint.toml` need to change to allow
  server → port-trait imports? If so, that change is part of the decision.
- **Migration scope.** Which services collapse into the port; whether any
  survive (e.g. `WorldCatalogue` if it holds real orchestration logic beyond
  delegation).

## Constraints

- Blocked by ticket 02: the storage seam's shape decides whether the
  Repository port sits above a trait or a struct, and changes its interface.
- Must keep the layer-rule intent satisfied (HTTP must not reach storage
  internals) even if the rule text changes.
- Decision ticket, no implementation.

## Notes

- Resolution uses `/grilling` and `/domain-modeling`.
- The map's "Not yet specified" fog on port-trait granularity graduates into
  this ticket and should be cleared from the map on resolution.

## Current state (2026-10-04)

Still present. The fleet is no longer uniformly one-method:

- `PersonaCatalogue`: still `new` + `list_personas`, and its header still says
  "intentionally a one-method seam".
- `WorldCatalogue`: 5 delegating methods.
- `PromptPresetService`: 5 delegating methods.
- `SettingsService`: now 4 methods (`get_settings`, `update_settings`,
  `add_connection`, `update_connection`). It was 1. `update_settings` takes a
  closure, so it is less purely a delegate. The grilling should check whether
  it now holds real orchestration and survives the port.

The arch-lint rule moved to `arch-lint.toml` lines 138-142 (text unchanged).
The HTTP adapter still honours it in `src/` (only `app_state_tests.rs`
imports storage, as an exempt test). Still Strong.

## Comments

**2026-10-14, from [Decide where the new design of the game-state lifecycle makes sense](../../turn-state-model/issues/24-decide-fix-or-rewrite.md).** The new design of the game-state lifecycle uses one repository per record, with record writes only on a transaction handle ([sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §3.1). Those repositories are the first instance of this ticket's question. Trait or struct is still open here.

## Answer

Closed 2026-10-16, superseded by [the Rust rebuild](../../platform-choice/map.md). Ruling of [Research: sort the open architecture-deepening tickets against the new design](../../platform-choice/issues/04-research-sort-architecture-deepening.md), [asset](../../platform-choice/assets/research-04-architecture-deepening-sort.md) §2.3: **moved to a ticket in the rebuild map.** The repositories' shape (trait or `Storage` method) is open in sketch 25 §3.1. Note: a port trait in `application/` breaks `arch-lint.toml:164-169`. [Decide the storage seam of the rebuild](../../platform-choice/issues/11-decide-storage-seam-of-rebuild.md), questions 2 and 3.
