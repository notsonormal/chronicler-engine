# Decide the shape of a generation: its kind, its analysis output and the Game rules

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

What types carry one generation from the request to the saved Swipe state, so that the states the design calls impossible cannot be built? Decide, as one design:

1. **The kind carries its data** (finding 3): an event generation carries its event record, an Impersonate carries its direction (or none), and the runtime checks that guard the missing data go away. This decides the answer to a bare `/impersonate` (finding 1, UI F2) and which last Messages allow a Retrigger (UI F3: spec 14.3 against `Tail::retrigger_allowed`).
2. **One analysis output type** from the analysis hook to the Game rules (candidate M4, finding 17): one confidence enum, no `QuantifierResult` to `StatePatch` round trip, no dead `SceneState.quantifier_confidence`.
3. **The Game rules as one call per generation** (candidate M1, findings 4 and 12): one room lookup that respects `dynamic_rooms`, the event rules, the one-shot Trigger mark, the opening room report, and the rendered event record (the Retrigger direction through `TemplateVars`). Inside the `arch-lint.toml` rule that keeps `WorldBundle` out of the inputs of the Game rules.
4. **Where the Job plan rules live** (candidate S1, finding 5): the `Tail` or `gate.rs`.

The answer is the interfaces and the tests that pin them; the build is a later slice.

## Context

- Inputs: [synthesis.md](../assets/architecture-review/synthesis.md) rows M1, M4, S1 and §4; [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md) findings 1, 3, 4, 5, 12, 17; [the UI findings](../assets/ui-review/findings.md) F2, F3.
- Thermo-nuclear's proposal: `../assets/whole-rebuild-review/findings/full-review-thermo.md`, "What would make this approvable" (H1, H3, H4).
- Skills: `/grilling`, `/domain-modeling`, `/codebase-design` (its design-it-twice pattern for the interfaces). Terms: `CONTEXT.md`. Design: `.scratch/turn-state-model/assets/target-design.md`.

## Answer
