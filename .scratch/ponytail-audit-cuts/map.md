# Ponytail audit cuts

## Destination

Apply the highest-impact ponytail-audit cuts to reduce dependency surface and inline small hand-rolled helpers: remove unused dependencies and stdlib-replaceable crates, and apply the batch of small mechanical shrinks. The codebase should build and pass the full guard after every cut.

## Notes

- Skills: ponytail, code-simplification, code-consistency-check
- Standing preference: keep changes mechanical; do not redesign behavior; verify with `python build.py` after each cut
- Background: [ponytail-audit run on 2026-08-12](../AGENTS.md) surfaced volume and dependency bloat.

## Decisions so far

<!-- As tickets close, append a one-line gist + link here. -->

## Not yet specified

- None yet.

## Out of scope

- Collapse the generation gate machinery (`GenerationSlot`, `GenerationGuard`, `release_owned_slot`) — changes concurrency semantics and needs a dedicated design review past this volume-reduction effort.
- Remove the `Agent` trait and `AgentRegistry` — ruled out for this effort.
- Remove the thin application-layer service wrappers (`SettingsService`, `PersonaCatalogue`, `PromptPresetService`, `WorldCatalogue`) — ruled out for this effort.
- Collapse empty module directories under `src/adapters/driving/http/*`, `src/application/prompting/utils/`, `src/application/prompting/builders/`, and `src/application/debug/` — ruled out for this effort.
