# 03 — Replace CRUD services with a Repository port

Type: grilling
Status: open
Blocked by: 02
Assignee: (unclaimed)

## Question

Do we commit to replacing the fleet of one-method CRUD application services
(`PersonaCatalogue`, `SettingsService`, `PromptPresetService`,
`WorldCatalogue`) with a single `Repository` port that `Storage` implements —
and if so, what is the shape of the deepened module?

## Background

This is **candidate 2** of the architecture review. See
`architecture-review.html` for the mass diagram and evidence.

The friction: `arch-lint.toml:136-140` forbids the HTTP layer from importing
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
