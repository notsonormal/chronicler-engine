# T3: Glossary Drift Fix — Finalized Sub-Plan

## Summary

Code symbols drift from `CONTEXT.md` canonical glossary in 4 places (down from 5 — `WorldSnapshot` removal moved to new Track T9): Persona value type still called `PlayerCard`, a `TurnResult` type (Turn deprecated), Action Pipeline phase merge that contradicts the glossary's 8-phase list, and avoid-alias leaks (`StoryLogTemplate`, `parse_command`, doc comments using `session`/`command`/`text`/`output`).

3 decisions locked via grilling:
- **D1** — `WorldSnapshot` moved out of T3 to new Track T9 (A-Deep: remove immutable world fields from GameState + add AppState cache). Already recorded in superplan.
- **D2** — `TurnResult` → `ActionResult` (closes Action loop).
- **D3** — Amend CONTEXT.md (do not split code) for Action Pipeline phase merge; trigger eval reads post-commit state, splitting would force redundant snapshot read.

Doc-comment sweep scope rule locked: rename glossary-concept refs only (player Action = "command"/"input"/"verb"; Narrative = "text"/"output"/"story"; Game = "session"; Character = "npc"; Message = "line"; Swipe = "variant"; World/Scenario = "setting"/"campaign"). Keep general-English usage (HTTP session, function output, stdout, "text" as string contents).

No new ADR. PersonaCard rename is mechanical glossary alignment (applying CONTEXT.md `Persona`, already canonical, to drifted code) — not an architectural decision. ADR-026 represents its June-23 bindings-only decision as-is; not amended.

## Key Changes

### PersonaCard rename (~100+ sites, ~5 SP)
- `PlayerCard` → `PersonaCard` (struct + all impls)
- `PlayerCardWithKey` → `PersonaCardWithKey` (`adapters/driven/storage/backend/core.rs:51`)
- `TestPlayer` → `TestPersona` (`test_support/fixtures.rs:42`)
- `PromptLayer::Player` → `PromptLayer::Persona` (`application/narrative_prompt/types.rs`)
- `PromptContext::player` field → `PromptContext::persona`
- **`GameState::player` field → `GameState::persona`** (`domain/model/state/game_state.rs:22` — 6 sites in this file: struct field + 5 ctor/builder sites)
- Update `domain/model/character.rs:51`, `adapters/driven/storage/backend/core.rs:51`, `adapters/driven/storage/models/persona.rs:5` (doc comment)

### `TurnResult` → `ActionResult` (~3 sites, ~0.5 SP)
- `domain/engine/action_processing.rs:27` (struct def) + same file lines 143, 184 (return type)
- `application/action_pipeline/phases.rs:384` (return type)
- Verify no other consumers via `grep -rn 'TurnResult'` post-refactor

### Action Pipeline phase (per D3)
- 1-line CONTEXT.md correction: `Action Pipeline` entry describes actual behavior — trigger evaluation runs inside engine commit (reads post-commit state); code lives in `execute_freeaction_impl` called from `phase_engine_commit`
- No code changes

### Avoid-alias sweep (~35 sites, ~1 SP)
- `StoryLogTemplate` → `NarrativeLogTemplate` (`adapters/driving/http/templates.rs:28` — ~20 sites incl. `templates_tests.rs` + `fragment_renderers.rs:9`)
- `parse_command` → `parse_action` (`domain/engine/parser.rs:6` — ~15 sites incl. `parser_tests.rs`)
- Doc-comment sweep per agreed scope rule — rename glossary-concept uses only; word-boundary grep + manual review

## Implementation

Single track, 4 parallel-renamable work items. Bundle in one PR or split T3.1 (Persona) from T3.2-T3.4 (mechanical sweep).

### Phase 1: Current-state doc sync (BEFORE code, per chronicler-dev-workflow)

- [ ] #### Task 1.1: Sync `architecture/system.md` to upcoming code state (1 SP)
  - [ ] Update references to `TurnResult` → `ActionResult` wherever system.md uses the old name
  - [ ] Update Action Pipeline phase description to reflect the merge (matches CONTEXT.md D3 correction)
  - [ ] Update `PlayerCard` → `PersonaCard`, `GameState::player` → `GameState::persona` references
  - Not a decision log — current-state sync only
  - **Validate:** `grep -n 'TurnResult\|PlayerCard\|GameState::player' chronicler_engine/docs/architecture/system.md` matches removed/updated lines; no stale symbol names

- [ ] #### Task 1.2: Correct CONTEXT.md `Action Pipeline` entry (per D3) (1 SP)
  - [ ] Rewrite entry to describe actual behavior: trigger evaluation runs inside engine commit phase (reads post-commit state), not as a separate phase function. Code: `execute_freeaction_impl` called from `phase_engine_commit`.
  - CONTEXT.md is the glossary source-of-truth, not a decision log — this is honest maintenance
  - **Validate:** `grep -n 'trigger evaluation' chronicler_engine/CONTEXT.md` shows the corrected entry; phase list reflects code reality

### Phase 2: Code renames

- [ ] #### Task 2.1: PersonaCard rename (~100+ sites, 5 SP)
  - [ ] ##### SubTask 2.1.1: Rename `PlayerCard` → `PersonaCard` struct + impls in `domain/model/character.rs` (1 SP)
  - [ ] ##### SubTask 2.1.2: Rename `PlayerCardWithKey` → `PersonaCardWithKey` in `adapters/driven/storage/backend/core.rs` + all references (1 SP)
  - [ ] ##### SubTask 2.1.3: Rename `TestPlayer` → `TestPersona` in `test_support/fixtures.rs` + all test imports (1 SP)
  - [ ] ##### SubTask 2.1.4: Rename `PromptLayer::Player` → `PromptLayer::Persona` + `PromptContext::player` → `PromptContext::persona` in `narrative_prompt/types.rs` (1 SP)
  - [ ] ##### SubTask 2.1.5: Rename `GameState::player` field → `GameState::persona` in `domain/model/state/game_state.rs` (6 sites) + all callers reading `state.player` (1 SP)
    - `Validate per subtask:` `cargo check` after each subtask; word-boundary grep for renamed symbols returns 0
  - **Validate (task):** `python build.py` green; `grep -rnw 'PlayerCard\|TestPlayer\|PlayerCardWithKey\|PromptLayer::Player\|PromptContext::player' chronicler_engine/{src,tests}/` returns 0; `grep -rnw 'Player\b' chronicler_engine/{src,tests}/` yields no glossary refs (only string/date uses)

- [ ] #### Task 2.2: `TurnResult` → `ActionResult` rename (1 SP)
  - [ ] `domain/engine/action_processing.rs:27,143,184` + `application/action_pipeline/phases.rs:384`
  - **Validate:** `grep -rnw 'TurnResult' chronicler_engine/{src,tests}/` returns 0; `cargo test` for `action_processing_tests.rs` passing

- [ ] #### Task 2.3: Avoid-alias sweep (2 SP)
  - [ ] ##### SubTask 2.3.1: Rename `StoryLogTemplate` → `NarrativeLogTemplate` (~20 sites) (1 SP)
    - `adapters/driving/http/templates.rs:28` + `adapters/driving/http/templates_tests.rs` + `adapters/driving/http/fragments/renderers/fragment_renderers.rs:9,45`
    - **Validate:** `grep -rnw 'StoryLogTemplate' chronicler_engine/{src,tests}/` returns 0
  - [ ] ##### SubTask 2.3.2: Rename `parse_command` → `parse_action` (~15 sites) (0.5 SP)
    - `domain/engine/parser.rs:6` + `domain/engine/parser_tests.rs`
    - **Validate:** `grep -rnw 'parse_command' chronicler_engine/{src,tests}/` returns 0
  - [ ] ##### SubTask 2.3.3: Doc-comment sweep per agreed scope rule (0.5 SP)
    - Rename glossary-concept refs only; keep general-English uses (HTTP session, function output, stdout, "text" as string contents)
    - Files flagged in plan: `game.rs`, `logic.rs`, `narrative_state.rs`, `action.rs`, `quantifier.rs`
    - **Validate:** manual review pass; no new `grep` matches for `session`/`command`/`text`/`output` in doc comments referring to glossary concepts

### Phase 3: Verification

- [ ] #### Task 3.1: Full build + grep audit (1 SP)
  - [ ] Run `python build.py` (fmt + clippy + tests + coverage)
  - [ ] Run all per-rename word-boundary greps (see Test Plan)
  - **Validate:** all green; `build.py` exit code 0; all greps return 0

## Test Plan

- `python build.py` green (fmt + clippy + tests + coverage) — primary gate
- Pre-existing tests cover rename correctness (no new test logic needed; pure mechanical refactor):
  - `character_tests.rs`, `action_processing_tests.rs`, `parser_tests.rs`, `templates_tests.rs`, `pipeline_tests.rs`, `game_state_snapshot_tests.rs`
- Regression check: `cargo test` for `domain/engine/action_processing_tests.rs` (TurnResult rename)
- Manual review for doc-comment sweep (no automated gate; scope rule is judgment-based)

## Per Task/Sub Task Validation Steps

- **Every code task:** `cargo check` after edits; `cargo test` for affected module
- **Every task:** word-boundary grep returns 0 for the old symbol
- **Phase 2 Task 2.1 (PersonaCard):** `grep -rnw 'PlayerCard\|PlayerCardWithKey\|TestPlayer\|PromptLayer::Player\|PromptContext::player' chronicler_engine/{src,tests}/` returns 0; `grep -rnw 'Player\b' chronicler_engine/{src,tests}/` reviewed case-by-case
- **Phase 2 Task 2.2:** `grep -rnw 'TurnResult' chronicler_engine/{src,tests}/` returns 0
- **Phase 2 Task 2.3.1:** `grep -rnw 'StoryLogTemplate' chronicler_engine/{src,tests}/` returns 0
- **Phase 2 Task 2.3.2:** `grep -rnw 'parse_command' chronicler_engine/{src,tests}/` returns 0
- **Final:** `python build.py` exit code 0; `architecture/system.md` matches code (current-state sync, no decision log); CONTEXT.md D3 correction in place; no new ADR

## Assumptions

- T1 (Tier 1 blockers) lands first to keep validation honest — superplan `Depends on: T1`
- T9 (WorldSnapshot removal) is separate track, NOT in T3 — any `WorldSnapshot` references staying in code after T3 are expected and owned by T9
- Existing tests cover rename correctness; no new test logic needed
- `GameState::player` field rename (6 sites in game_state.rs + readers) is in scope for T3 — it's the Persona value-type field, follows from `PlayerCard` → `PersonaCard` rename directly
- Doc-comment sweep does NOT touch ADR bodies or `docs/` markdown other than `CONTEXT.md` D3 correction + `architecture/system.md` sync — other doc updates belong to `chronicler-docs-hygiene` skill (out of scope per superplan)
- Breaking T3.1 (Persona) from T3.2-T3.4 is allowed if review surface needs it — not pre-decided
- Estimated ~7 SP total (Persona ~5 + ActionResult 0.5 + phase-glossary 1 + avoid-alias 1) — under 8 SP threshold, no further breakdown required
- ADRs are not changelogs; architecture/system.md is current-state sync, not decision log. (Also affects T9 superplan entry's ADR-033 placeholder — re-evaluate when T9 grilling happens; non-trivial architecture change may still warrant new ADR on its own merits, but not as a default policy.)
