# T3: Glossary Drift Fix

**Parent Plan:** [simpler-hexagon-pre-merge-superplan.md](./simpler-hexagon-pre-merge-superplan.md) (Track T3)
**Status:** Planning — 3 decisions to lock before implementation
**Date:** 2026-07-09
**Branch:** `simpler-hexagon`
**Depends on:** T1 (T1 fixes are independent of T3, but T1 should land first to keep validation honest)
**Blocks:** none (T3 is mechanical, parallelizable with T2)
**Priority:** P1
**Findings owned:** glossary-1, glossary-2, glossary-3, glossary-4, glossary-5

---

## Summary

Code symbols in `src/` and `tests/` drift from `CONTEXT.md`'s canonical glossary in 5 places: Persona value type still called `PlayerCard`, a load-bundle struct called `WorldSnapshot` (conflicts with the `Snapshot` = `GameStateSnapshot` rule), a `TurnResult` type (Turn is deprecated), an `Action Pipeline` phase merge that contradicts the glossary's 8-phase list, and a handful of avoid-alias leaks (`StoryLogTemplate`, `parse_command`, doc comments using `session`/`command`/`text`/`output`).

Two of these are pure naming; one is a code/glossary mismatch with a choice of which side to fix; one is genuinely contested (delete vs rename `WorldSnapshot`); one is mechanical sweep.

---

## Decisions to Lock (grill before implementation)

### D1 — `WorldSnapshot`: delete or rename?

Two tracks contradict on this:
- **Code-quality review R6** (`simpler-hexagon-review.md` Appendix A5): "Delete `WorldSnapshot` entirely (only retry.rs user, fallback chain)."
- **T2 superplan §Scope item 5 + T3 superplan §Scope item 2**: rename to `WorldContext` / `WorldBundle` / `WorldLoad`, ~12 sites, ~1 SP.

Current state on disk (uncommitted T2 WIP):
- `src/application/persistence_gate/dto.rs:13` — `pub struct WorldSnapshot { world, map, player, npcs }`
- `src/application/persistence_gate/gate.rs:41-180` — `load_world_snapshot()`, `world_snapshot_or_empty()`. **Always wrapped in fallback** (try load; on Err → log warning + `empty()`).
- `src/application/action_pipeline/retry.rs:67-75` — sole real consumer. Destructures fields to build `GameState::new`.
- `tests/helpers/fixtures.rs:382-394` — dead code: constructs `WorldSnapshot` then `let _ = world_snapshot;`. (Polish-8.)

**Candidate resolutions:**

- **A) Delete** (`R6`): delete the struct, the loaders, and the `empty()` fallback. `retry.rs` calls 4 separate `World` / `Map` / `Persona` / `Character` loaders directly. Removes ~120 lines + the `persistence_gate/dto.rs` file entirely. Aligns with CONTEXT.md `Snapshot` rule ("immutable world data is cached on AppState as Arcs and re-attached on load, not stored in the snapshot") — the load bundle is redundant.
- **B) Rename** (`T2/T3`): rename struct to `WorldContext` (or `WorldBundle` / `WorldLoad`) and keep the fallback pattern. ~1 SP mechanical.

**My recommendation: A (delete).** R6 is the right read. The struct + fallback pattern is a smell — `retry.rs` is the only consumer that needs all 4 fields; everywhere else `world_snapshot_or_empty()` exists to *not* crash on Err, but the err path is unreachable in practice (load errors are programmer errors, not runtime conditions, given AppState-Arc-cached immutable world data).

**Question for the human:** A or B? If B, which name — `WorldContext`, `WorldBundle`, or `WorldLoad`?

### D2 — `TurnResult` final name

`Turn` is deprecated per CONTEXT.md. Two candidates:
- **`ActionResult`** (recommended) — aligns with the `Action` glossary entry. Loop closure: Action enters pipeline → ActionResult exits.
- **`EngineCommitResult`** — emphasizes the producing phase. More precise; verbose.

**Question for the human:** `ActionResult` or `EngineCommitResult`?

### D3 — Action Pipeline phase: split code or amend glossary?

CONTEXT.md's `Action Pipeline` entry lists **8 distinct phases** including standalone "trigger evaluation." Code merges it into `phase_engine_commit` via `execute_freeaction_impl` → `evaluate_triggers`.

**Candidates:**
- **A) Split code** — extract `phase_trigger_evaluation`. Code matches glossary. Touches `application/action_pipeline/{pipeline,phases}.rs`.
- **B) Amend glossary** (recommended) — update CONTEXT.md to document the merge. Rationale: trigger evaluation reads post-commit state, so splitting would force a redundant snapshot read. Code is working; docs should match. 1-line edit.

**My recommendation: B.** Zero behavior gain from A; high blast radius.

**Question for the human:** A or B?

---

## Key Changes (after D1-D3 resolved)

### Persona rename (~60 sites, ~3 SP)

- `PlayerCard` → `PersonaCard` (struct + fields)
- `PlayerCardWithKey` → `PersonaCardWithKey`
- `TestPlayer` → `TestPersona`
- `PromptLayer::Player` → `PromptLayer::Persona`
- `PromptContext::player` → `PromptContext::persona`
- Update `domain/model/character.rs:51`, `adapters/driven/storage/backend/core.rs:51`, `adapters/driven/storage/models/persona.rs:5`
- ADR-026 amendment: note that bindings rename is now extended to the value type itself

### `TurnResult` rename (~6 sites, ~0.5 SP)
Per D2.

### Action Pipeline phase (per D3)

### Avoid-alias sweep (~10 sites, ~0.5 SP)

- `StoryLogTemplate` → `NarrativeLogTemplate` (`templates.rs:28`)
- `parse_command` → `parse_action` (`engine/parser.rs:6`)
- Doc-comment sweep: rename `session` / `command` / `text` / `output` only where they refer to glossary concepts. **Scope rule pending:** see "Pre-Implementation Checklist" #1.

### `WorldSnapshot` resolution (per D1)

---

## Out of Scope

- **B2 glossary gaps** — adding new terms (`NarrativeState`, `SceneState`, `MovementState`, `InputBuffer`, `GenerationStatus`, `MessageType`, `StartingScenario`) to CONTEXT.md. Domain modeling, separate doc-only plan.
- **B3 behavior drift** — `switch_swipe` not restoring snapshot (`application/message_editing.rs:35-77`); `arrival_service::run` silent return (`arrival_service.rs:52-58`). These are bugs, not glossary drift. Belong to T1 (Tier 1 Blockers).
- **Workflow gate enforcement** — formal `grep -wn` audit gate per PR. Belongs to T8 (Workflow Gates).
- **PR sequencing** — split T3.1 alone vs bundle T3.2-T3.5. Resolves implicitly from the resolved sub-task sizes + grep-miss risk observed during implementation. Not pre-decided.
- **Bulk ADR doc updates** — ADRs other than ADR-026 (Persona amendment) that reference old symbols. `chronicler-docs-hygiene` skill territory.

---

## Blast Radius

- ~80 sites across domain, application, adapters, tests
- 1 file deleted IF D1 = A (`persistence_gate/dto.rs`) or 1 file + ~12 sites renamed IF D1 = B
- 1 ADR-026 amendment (Persona rename)
- 1 conditional CONTEXT.md amendment (D3 = B)
- Possibly 1 conditional CONTEXT.md entry addition (B2, out of scope — separate plan)

---

## Verification

- `python build.py` green (fmt + clippy + tests + coverage)
- Per-rename word-boundary grep audit:
  - `grep -rnw 'PlayerCard' chronicler_engine/{src,tests}/` returns 0
  - `grep -rnw 'WorldSnapshot' chronicler_engine/{src,tests}/` returns 0 (after D1 resolution)
  - `grep -rnw 'TurnResult' chronicler_engine/{src,tests}/` returns 0
  - `grep -rnw 'Player\b' chronicler_engine/{src,tests}/` reviewed case-by-case for `PromptLayer::Player`, `PromptContext::player`, etc.
- `grep -rnw 'StoryLogTemplate\|parse_command' chronicler_engine/{src,tests}/` returns 0
- `architecture/system.md` still matches code after CONTEXT.md amendments (D3 = B)

---

## Pre-Implementation Checklist

- [ ] **Lock D1, D2, D3 with the human** (grilling).
- [ ] **Resolve "Not yet specified" patches** before implementation:
  - T3.5 doc-comment sweep scope rule: when does `session` / `command` / `text` / `output` refer to a glossary concept vs general English? E.g. HTTP session is general; player `command` (= player Action) is glossary.
  - ADR-026 scope: did the bindings-only rename leave a partial state that changes the blast radius of the value-type rename?
- [ ] **Run `improve-codebase-architecture` ADR-conflict check** vs ADR-026, ADR-027, ADR-030 before writing code.
- [ ] **Update `architecture/system.md`** BEFORE writing code (chronicler-dev-workflow step 2) — record the D1/D2/D3 decisions and any CONTEXT.md amendments.
- [ ] **Confirm AGENTS.md plan-adherence rule**: any deviation from super-plan scope requires user approval.

---

## Honest Tradeoff

T3 is safe to land mechanically. Risk: test fixture naming has been around long enough that grep misses are likely. The Persona rename is the largest blast radius (~60 sites); can split T3.1 from T3.2-T3.5 if review needs.

D1 is the only decision with real architectural consequences (delete = ~120 lines + 1 file gone, rename = 1 file stays + ~12 sites moved). Worth spending grilling time on this one.