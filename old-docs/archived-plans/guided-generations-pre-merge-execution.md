# Guided-Generations Pre-Merge Execution

**Status:** Ready for execution
**Created:** 2026-08-30
**Source:** [guided-generations-architecture map](../../.scratch/guided-generations-architecture/map.md) — resolved tickets [06](../../.scratch/guided-generations-architecture/issues/06-revise-branch-conceptual-model.md), [01](../../.scratch/guided-generations-architecture/issues/01-deepen-narration-turn-module.md), [02](../../.scratch/guided-generations-architecture/issues/02-deepen-replay-steering-module.md), [03](../../.scratch/guided-generations-architecture/issues/03-collapse-steering-entry-dispatcher.md), [04](../../.scratch/guided-generations-architecture/issues/04-deepen-steering-prompt-policy.md), [05](../../.scratch/guided-generations-architecture/issues/05-single-narrative-voice-injection.md), [07](../../.scratch/guided-generations-architecture/issues/07-pre-merge-handoff-shape.md)
**Blocks:** the narrator-modes effort (its tickets 06, 07, 09–12 resume after this lands and `guided-generations` merges)

---

## Goal

Execute the four accepted architecture pieces on the `guided-generations`
branch so it can merge to main: remove Narrator Action, make the
prompt-construction module the sole owner of narrative-voice application,
collapse the action entry methods into one `process_action(Action)`
dispatcher, and extract the `narration_generation` module that owns the
narrate-and-persist prefix.

## Settled decisions

The architecture map resolved every decision. This plan does not re-decide;
each task links the ticket that holds the detail.

- Narrator Action is retired entirely — command, `Action::Narrator`,
  `MessageType::Narrator`, render branches, tests (ticket 06, decision 5).
- Voice application has one owner: the prompt-construction module.
  `TemplateVars::set_narrative_voice` is deleted; a module-private helper in
  `assembler.rs` stamps from world posture (ticket 05).
- One public gated `process_action(&GenerationGate, Action)` entry;
  payloads ride the `Action` enum; the impersonate preset pin stays at
  entry time inside the dispatcher's match (ticket 03).
- `narration_generation::run(state, GenerationInputs) ->
  Result<NarrationOutcome, PhaseError>` owns the narrate-and-persist
  prefix; callers resolve inputs; the core is branch-free (ticket 01).
- Impersonate redo folds into the impersonate flow and gains the full tail
  — **the one deliberate behavior change** (ticket 01, decision 3).
- Rejected: an owner module for the stored-inputs flow (ticket 02); a
  prompt-policy module (ticket 04). Their execution notes fold into Task 5.

## Execution constraints (ticket 07)

1. **One single commit.** All four pieces land together on
   `guided-generations`. Tasks below are a work order that keeps the tree
   green at each checkpoint — not separate commits.
2. **The commit message flags the behavior change** (impersonate redo's
   full tail) so blame and bisection readers can find it. See
   [Commit message](#commit-message).
3. **Gate:** `python build.py` green with the commit landed.
4. **Sequencing:** this work lands first; the narrator-modes effort resumes
   after the merge. Do not pick up narrator-modes ticket 06's scope here.

---

## Task List

### Phase 1: Narrator Action removal

- [ ] Task 1: Remove Narrator Action from code
- [ ] Task 2: Remove Narrator Action from specs, docs, and browser tests

#### Task 1: Remove Narrator Action from code

**Description:** Delete the retired feature from the domain model, the
pipeline, the HTTP handler, and the render branches (ticket 06, decision
5).

**Acceptance criteria:**
- [ ] The `/narrator` slash command no longer parses: `Action::Narrator`
      and its parse arm are deleted from `src/domain/model/action.rs`.
- [ ] `narrator_action` and `process_action_with_narrator` are deleted
      from `src/application/pipeline/action_pipeline/action.rs`.
- [ ] The HTTP handler path for the command is deleted from
      `src/adapters/driving/http/action/handlers/actions.rs`.
- [ ] `MessageType::Narrator` and every render branch for it are deleted:
      `src/application/prompting/assembler.rs` (history rendering),
      `src/application/agents/quantifier/prompt.rs`,
      `src/adapters/driving/http/view_models.rs`.
- [ ] The slash-command palette offers `/guide` and `/impersonate` only:
      delete the `/narrator` entry from the command list in
      `assets/index.html` (line ~423).
- [ ] The two Narrator HTTP tests (`tests/http/actions.rs:748,773`) are
      deleted in this task — they exercise the deleted handler and fail
      the moment it is gone. Their spec scenario leaves in Task 2; the
      feature-spec validator runs at the Phase-1 checkpoint, after Task 2.
- [ ] The guide HTTP test (`tests/http/actions.rs:747-754`) loses its
      `MessageType::Narrator` block (the filter and the
      `assert_eq!(narrators.len(), 0, ...)` lines) — with the variant
      deleted it no longer compiles. The test otherwise survives: its
      Narration and Input assertions stay.
- [ ] Persistence consequence (review finding R1, resolved Option A):
      after this task a game whose stored messages contain a legacy
      `"Narrator"` message type fails loudly at load — a deserialization
      error, not silent corruption. Only branch-era dev databases can
      hold such rows; the feature never shipped on main. No shim, no
      migration, affected dev games are deleted manually.

**Verification:**
- [ ] `cargo test --lib` passes (the three `narrator_action` unit tests in
      `action_tests.rs` retire with the code).
- [ ] `cargo nextest run --test http actions` passes.

**Dependencies:** None

**Files likely touched:**
- `src/domain/model/action.rs`, `src/domain/model/message.rs`
- `src/application/pipeline/action_pipeline/action.rs`, `action_tests.rs`
- `src/adapters/driving/http/action/handlers/actions.rs`, `actions_tests.rs`
- `src/application/prompting/assembler.rs`
- `src/application/agents/quantifier/prompt.rs`
- `src/adapters/driving/http/view_models.rs`
- `assets/index.html` (palette command list, line ~423)
- `tests/http/actions.rs` (the two Narrator tests; the guide test's
  `MessageType::Narrator` block at 747-754)

**Estimated scope:** 5 SP (many files, but shallow deletions)

#### Task 2: Remove Narrator Action from specs, docs, and browser tests

**Description:** Align the spec and documentation layers with the removal.
`CONTEXT.md` is already correct — Narrator Action sits in Deprecated Terms.

**Acceptance criteria:**
- [ ] `docs/specs/actions.md` scenario 1.11 (`/narrator` dispatches as
      narrator action) is deleted. Its two covering HTTP tests already
      left in Task 1.
- [ ] `docs/specs/browser.md`: the palette scenario (line ~83) lists
      `/impersonate` and `/guide` only; scenario 17.10 (submitting
      `/narrator` persists a Narrator log entry) is deleted with its
      browser test in `tests/browser/behaviour.rs`.
- [ ] `docs/diataxis/reference/narrative/ai_steering.md` is rewritten for
      two surfaces: the intro sentence, the comparison table, the command
      table row, and the `## Narrator Action` section.
- [ ] `docs/diataxis/reference/frontend/dashboard.md` (slash-command
      palette paragraph, line ~82) lists two commands.

**Verification:**
- [ ] `python scripts/validate_feature_spec.py` passes (every remaining
      scenario has a covering test; no orphaned test annotations).
- [ ] `python scripts/validate_docs.py` passes.
- [ ] `cargo nextest run --test browser` passes.

**Dependencies:** Task 1

**Files likely touched:**
- `docs/specs/actions.md`, `docs/specs/browser.md`
- `docs/diataxis/reference/narrative/ai_steering.md`
- `docs/diataxis/reference/frontend/dashboard.md`
- `tests/http/actions.rs`, `tests/browser/behaviour.rs`

**Estimated scope:** 3 SP

### Checkpoint: Narrator Action removed
- [ ] `cargo test --lib` and the targeted integration suites pass
- [ ] Spec and docs validators pass
- [ ] `grep -rn "Narrator" src/ docs/specs/` returns only the surviving
      uses: `NarratorMode` (the narrator-modes setting) and the AI
      narrator role — no references to the retired command or
      `MessageType::Narrator`

### Phase 2: Voice ownership

- [ ] Task 3: Single injection point for narrative voice

#### Task 3: Single injection point for narrative voice

**Description:** Make the prompt-construction module the sole owner of
voice application (ticket 05). Behavior-neutral today: `assemble` already
overwrites caller-stamped voice, so arrival's stamping is dead code.

**Acceptance criteria:**
- [ ] `TemplateVars::set_narrative_voice` is deleted from
      `src/domain/model/template.rs` (lines ~50-57), including the stale
      comment that names `AppSettings` as the source of truth.
- [ ] A module-private helper (e.g. `apply_posture(&mut TemplateVars,
      perspective, tense)`) in `src/application/prompting/assembler.rs`
      performs the two field assignments; `assemble` is its only caller,
      stamping from world posture (source unchanged — the game-posture
      source belongs to narrator-modes ticket 06, post-merge).
- [ ] The posture resolution and stamping in
      `src/application/arrival_service.rs` (lines ~152-156) are deleted.
- [ ] The ownership test (`assembler_tests.rs`,
      `test_assemble_injects_narrative_voice_from_world_posture`, line
      ~759) is extended: incoming `template_vars` deliberately carry
      conflicting voice; the assertion is that the owner's stamp wins.

**Verification:**
- [ ] `cargo test --lib` passes; the arrival flow tests pass **unmodified**
      (the deletion is behavior-neutral).
- [ ] No named stamp operation exists outside the owner module (structural
      enforcement — no guardrails test; ticket 05, decision 2).

**Dependencies:** None (independent of Phase 1; sequenced after it to keep
one concern per checkpoint)

**Files likely touched:**
- `src/domain/model/template.rs`
- `src/application/prompting/assembler.rs`, `assembler_tests.rs`
- `src/application/arrival_service.rs`

**Estimated scope:** 3 SP

### Checkpoint: Voice ownership settled
- [ ] `cargo test --lib` passes
- [ ] `cargo clippy --all-targets -- -D warnings` clean

### Phase 3: Action dispatcher

- [ ] Task 4: Collapse action entries into `process_action(Action)`

#### Task 4: Collapse action entries into one dispatcher

**Description:** One public gated entry on `ActionPipeline` (ticket 03).
After Phase 1 the entry set is already free of the narrator methods.

**Acceptance criteria:**
- [ ] `ActionPipeline` exposes one public gated entry:
      `process_action(&GenerationGate, Action) ->
      Result<ProcessActionResult, EngineError>`. The gated slash-command
      entries fold in: `process_action_with_guide`, `guide_narration`,
      and `impersonate` (all that remain after Task 1).
- [ ] Payloads ride the existing `Action` enum — no new variants, no side
      channel. The empty-input → continue rule moves behind the dispatcher
      (the handler's `input.is_empty()` branch folds in).
- [ ] The variant → (persisted message, replay inputs) mapping
      concentrates in one `match` behind the dispatcher. `GenerationReplay`
      construction — including the impersonate preset pin read from
      settings — lives inside that match (ticket 03: entry-time pinning is
      required; a swipe stores the inputs that produced it).
- [ ] `pipeline_run.rs`'s generation-time fallback (resolve the preset
      when the stored pin is `None`) stays as fallback.
- [ ] The HTTP handler shrinks to parse-plus-call; it keeps
      `Action::parse` for the `is_steering` text-check path.
- [ ] `execute_action_with_replay` drops to `pub(crate)` — the sync runner
      for spawn closures and sync tests.
- [ ] New unit tests drive the dispatcher with `Guide` and `Impersonate`
      variants and assert the replay inputs stored on the resulting swipe.
- [ ] Gate-behavior unit tests (cancellation on reset, heal-stale,
      persona-missing) re-point to the dispatcher with `Action::FreeAction`;
      sync tests stay at the now-`pub(crate)` runner.

**Verification:**
- [ ] `cargo test --lib` passes with the new dispatcher tests.
- [ ] HTTP integration tests (`tests/http/actions.rs`, ~687-724 region)
      pass, extended to assert the swipe's stored replay inputs.

**Dependencies:** Task 1 (narrator entries gone — no temporary `Narrator`
arm needed)

**Files likely touched:**
- `src/application/pipeline/action_pipeline/action.rs`, `core.rs`,
  `action_tests.rs`
- `src/adapters/driving/http/action/handlers/actions.rs`
- `tests/http/actions.rs`

**Estimated scope:** 5 SP

### Checkpoint: Dispatcher landed
- [ ] `cargo test --lib` and `cargo nextest run --test http` pass

### Phase 4: Narration-generation module

- [ ] Task 5: Extract `narration_generation` (subtasks 5.1–5.4)

Ticket 01 holds the full interface contract. Ticket 02's and 04's
execution notes fold into subtask 5.4.

#### Task 5.1: Module skeleton and main-path rewiring

**Description:** Create `src/application/pipeline/narration_generation.rs`
(beside `pipeline_run.rs` — not `application/generation/`, which holds
per-game gating). Move the narrate-and-persist prefix behind it; re-point
`phase_narrate`'s main path.

**Acceptance criteria:**
- [ ] Interface exists as settled:
      `run(state: &mut GameState, inputs: GenerationInputs) ->
      Result<NarrationOutcome, PhaseError>`.
- [ ] `GenerationInputs` is caller-resolved: input text, optional guide,
      optional Impersonate steering (direction + preset id). No redo flag —
      redo-ness rides in `state.narrative.retry_target`.
- [ ] `NarrationOutcome` carries narration text + backend name + model
      name.
- [ ] The module owns the prefix: load world bundle → resolve room →
      consume the preset choice → build `PromptContext` → call narrator →
      `check_game_unchanged` → add Message or Swipe → save Message +
      Snapshot. The core is branch-free: `phase_narrate`'s
      `resolve_guide`/`resolve_impersonate` (including the replay
      fallback) move out to the callers as input preparation.
- [ ] Errors return as `PhaseError`; the caller runs the existing
      `finalize_phase_error` helper.
- [ ] The post-narration tail (quantifier → engine commit → trigger) stays
      with the calling paths. Excluded callers:
      `retry_event_continuation` (never enters the narrate path) and
      `arrival_service`.
- [ ] New direct tests drive `narration_generation::run`: one per prefix
      failure mode (bundle load, room not found, preset missing, narrator
      error, cancellation, save failure) and one per input kind (free
      action, Guided Generation, Impersonate).
- [ ] The 13 choreography `pub(crate)` helper tests retire in this step —
      their coverage is replaced by the direct tests above, not dropped.

**Verification:**
- [ ] `cargo test --lib` passes with the new module tests.

**Dependencies:** Task 4 (the dispatcher is the caller that resolves
inputs)

**Files likely touched:**
- `src/application/pipeline/narration_generation.rs` (new) + its tests
- `src/application/pipeline/pipeline_run.rs`
- `src/application/pipeline/action_pipeline/retry.rs`, `retry_tests.rs`

**Estimated scope:** 5 SP

#### Task 5.2: Test fixtures for replay-carrying swipes

**Description:** Extend `src/test_support` fixtures so a Swipe carrying
stored generation inputs can be built directly — the pre-existing gap the
architecture review flagged. Prerequisite for the redo flow tests in 5.3.

**Acceptance criteria:**
- [ ] A fixture builds a game state whose last message has a Swipe with
      stored inputs (guide, impersonate direction + preset id, or plain).
- [ ] The fixture is used by at least one passing test.

**Verification:**
- [ ] `cargo test --lib` passes.

**Dependencies:** None against 5.1; land before 5.3

**Files likely touched:**
- `src/test_support/fixtures.rs` (or `test_data_builder.rs`)

**Estimated scope:** 3 SP

#### Task 5.3: Redo paths through the module

**Description:** Rewire the redo entry paths (ticket 01, decisions 3–4).
**This subtask carries the plan's one behavior change.**

**Acceptance criteria:**
- [ ] Impersonate redo re-runs the impersonate generation from the Swipe's
      stored inputs through the same path as a fresh Impersonate, and runs
      the full tail (quantifier → engine commit → trigger).
      `retry_reimpersonate` dissolves as a separate implementation.
- [ ] ReNarrate redoes the narration through `narration_generation::run`;
      UserRegen stays tail-less.
- [ ] The anchor query stays in `message_service` as a history query;
      classification and reconstruction stay `pub(crate)` plumbing in the
      pipeline's redo entry path.
- [ ] Redo-mode coverage moves to flow tests through the public `retry()`
      entry — one per mode (narration, impersonate, user-regen, event).
- [ ] The 6 plumbing helper tests (`resolve_retry_target`,
      `reconstruct_retry_state`, and friends) retire in this step.
- [ ] `switch_swipe` semantics are unchanged: Swipe snapshots are captured
      pre-tail at message-add time in every path.

**Verification:**
- [ ] `cargo test --lib` passes; the four redo flow tests pass.
- [ ] The 29 existing flow tests survive unchanged.

**Dependencies:** 5.1, 5.2

**Files likely touched:**
- `src/application/pipeline/action_pipeline/retry.rs`, `retry_tests.rs`
- `src/application/pipeline/pipeline_run.rs`
- `src/test_support/` (fixture use)

**Estimated scope:** 5 SP

#### Task 5.4: Plumbing consolidation

**Description:** Fold in the execution notes from rejected tickets 02 and
04 — plumbing, not new seams.

**Acceptance criteria:**
- [ ] The `pending_replay` staging buffer is removed
      (`src/domain/model/state/narrative_state.rs` and the staging
      assignment in `action.rs`): the core holds `GenerationInputs` at
      message-add time and writes the stored inputs onto the Swipe
      directly (ticket 02's execution observation). `GameState::push_message`'s
      attachment and redo-swipe inheritance stay on `GameState`.
- [ ] The two preset loaders
      (`load_preset_and_response_length` /
      `load_impersonate_preset_and_response_length`) merge into one private
      helper: the impersonate/system branch produces the preset id (pinned
      id or active fallback), one loader fetches preset + response length
      (ticket 04's execution note).
- [ ] The defensive guide/impersonate exclusion line
      (`action_pipeline/core.rs:230-234` region) moves to caller-side
      input preparation, where the dispatcher and redo path build
      `GenerationInputs` (ticket 04, decision 2).

**Verification:**
- [ ] `cargo test --lib` passes; no behavior change.

**Dependencies:** 5.1, 5.3

**Files likely touched:**
- `src/domain/model/state/narrative_state.rs`, `game_state.rs`
- `src/application/pipeline/action_pipeline/action.rs`, `core.rs`
- `src/application/pipeline/pipeline_run.rs`

**Estimated scope:** 3 SP

### Checkpoint: Module landed
- [ ] `cargo test --lib` passes
- [ ] `cargo nextest run --tests` passes (full integration suite)

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| The impersonate-redo full tail is a behavior change riding inside a refactor commit | Medium | Flagged in the commit message (below); covered by the new `retry()` flow test; believed to fix accidental behavior — no recorded reason for stop-after-save (ticket 01, inferred) |
| Single commit harms bisection | Medium | User-decided (ticket 07 Q1, disagreement on record); tasks keep the tree green at each checkpoint so the change is reviewable in phases |
| Helper tests retire before replacements exist | Medium | 5.1 and 5.3 retire old tests only in the same step that adds their replacements |
| Narrator-modes effort resumes on stale assumptions | Low | Sequencing comment appended to its ticket 06; the `allowed_modes` edge is recorded as its ticket 15 |
| Legacy `"Narrator"` rows in branch-era dev databases fail to load after Task 1 | Low | Accepted (review finding R1, Option A): the failure is loud at load; the feature never shipped on main; delete affected dev games |

## Open Questions

None. The architecture map resolved every decision this plan needs. The
`allowed_modes` generation-time edge is deliberately **not** this plan's
scope — it lives on the narrator-modes map as ticket 15.

## Final Verification

- [ ] `python build.py` green (fmt + clippy + guardrails + full tests) —
      the merge gate (ticket 07, Q2).
- [ ] `cargo nextest run --test architecture` and `--test guardrails`
      pass (run as part of `build.py`).
- [ ] The docs index regenerates via the pre-commit hook
      (`scripts/generate_docs_index.py`).

## Commit message

Land all phases as **one commit** (ticket 07, Q1). The message must flag
the behavior change. Skeleton:

```
Refactor guided-generations architecture pre-merge

- Remove Narrator Action (command, Action::Narrator, MessageType::Narrator)
- Route narrative-voice application through the prompt-construction module
- Collapse action entries into the process_action(Action) dispatcher
- Extract narration_generation, owning the narrate-and-persist prefix

BEHAVIOR CHANGE: an impersonate redo now runs the full post-narration
tail (quantifier, engine commit, trigger). Before, it stopped after save.
```
