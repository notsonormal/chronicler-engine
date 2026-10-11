# Doc-spec boundary follow-ups

## Summary
The frontend docs rewrite and the Phase 4 reference-doc review (commits `a58af5b9`, `654f32e5`, `28841e6f`) left follow-ups in three files under `.scratch/doc-spec-boundary/issues/`. These files are not wayfinder tickets. This plan carries all of their content. When the plan is complete, the folder `.scratch/doc-spec-boundary/` is deleted.

The plan has three phases:

- **Phase A** (the lead): small decisions, DOC-anchor retargets, and one hand-off to an existing ticket.
- **Phase B** (one implementer agent): a spec scenario and a tagged test for each event rule that the doc review removed.
- **Phase C** (the lead): validate and delete the scratch folder.

Three pipeline defects found by the same review moved to the wayfinder map [Turn state model](../../.scratch/turn-state-model/map.md): the Swipe Snapshot set before the quantifier (issue 03), re-impersonate against `swipe_new.md` I.6, and arrival narration without the generation slot (G8). They share one unwritten rule: what state belongs to a turn, and who may write it. Fixing them one at a time would guess at that rule.

The boundary for every item is `docs/AGENTS.md` §Where a fact lives.

### Items that need no work
- **Retry naming.** Commit `28841e6f` removed the last "event retrigger" text. `rg -i 'event retrigger'` outside `tmp/`, `old-docs/` and `.scratch/` finds nothing.
- **Settings read rule.** `architecture_system.md` §Settings is the Reference home. `startup.md`, `storage.md`, `storage_design.md` and `rust_idioms.md` each restate it in one sentence, about their own subject. The one-home-per-subject rule allows this.
- **G6, text-check fail-open.** The owner kept it in `game_flow.md` §Text-Check Branch.
- **G9, empty-DB bootstrap from `--world` and `--persona`.** A client cannot observe it over HTTP, so no spec fits. The code (`bootstrap::run`) carries it.

## Implementation

### Phase A: Decisions and anchors (lead)

- [x] #### Task A.1: Hand the text-check spec to its existing ticket
  - Ticket "Spec and migrate text_check.rs" (`.scratch/http-test-spec-mapping/issues/11-spec-and-migrate-text-check.md`) already writes `docs/specs/text_check.md`.
  - Add to its `## Work` list: "Add a scenario: when the preview opens, no Action is dispatched until the player picks Send with edits or Send Original (G7). Browser scenario 16.36 pins only the input text."

- [x] #### Task A.2: Scope the "Complete specs" rule
  - In `tests/STRATEGY.md` §Spec scenarios, after the **Complete specs** paragraph's first sentence, add: "The rule is per route. Fine-grained UI behaviour inside one response stays in the code and its tests."

- [x] #### Task A.3: Retarget DOC anchors
  Change line 1 of each file. `CODING_STANDARDS.md` allows `reference/` targets only.

  | Files | New target | Reason |
  |---|---|---|
  | `src/application/settings_service.rs`, `src/utils/settings.rs` | `reference/architecture_system.md` | §Settings is the settings home. |
  | `src/application/prompt_preset_service.rs` | `reference/narrative/prompt_system.md` | §Prompt Presets. |
  | `src/application/connection_test_service.rs` | `reference/narrative/narration_system.md` | §Model Configuration holds the connections. |
  | `src/application/generation/guard.rs` | `reference/game_flow.md` | §Cancellation names `GenerationGuard`. |
  | `src/adapters/driven/llm/mod.rs`, `llm/providers/*.rs`, `llm/transport/**/*.rs`, `src/adapters/driven/utils/mock.rs`, `src/application/llm_recorder.rs`, `src/application/ports/llm_provider.rs`, `src/application/prompting/sanitize.rs`, `src/domain/model/llm_backend.rs` | `reference/narrative/narration_system.md` | Connections, the recorder and response sanitization live there. `sanitize.rs` holds only `sanitize_llm_output`. |
  | `src/adapters/driven/storage/llm_messages.rs` | `reference/storage.md` | The `llm_messages` table. |
  | `src/domain/model/{world,character,scenario}.rs`, `src/domain/model/utils/{world,scenario}_defaults.rs` | `reference/storage.md` | The World catalogue cluster. `agent_system.md` carries no World content. |
  | `src/application/pipeline/action_pipeline/options.rs` | `reference/narrative/options.md` | §Where Options Come From. |

  The `prompting/*` files, `template.rs` and `xml.rs` keep `prompt_system.md`.

### Phase B: Spec scenarios for removed event rules (implementer)

Each task adds one scenario to the named spec and one test tagged `// [docs/specs/<file>.md] SCENARIO: n.m`. Write the test first. The rule comes from the deleted doc text, so the code may disagree with it. **If a test fails, stop.** Report the failure to the lead and do not change the code or the rule.

Existing coverage and code: Scenarios 24.7 and 24.12 start from an empty offered set, so they cannot show a set kept or cleared. Scenario 24.5 already pins "a failed on-demand regeneration keeps the previous set". The turn-end rules live in `ActionPipeline::rewrite_options_after_turn`; the on-demand failure path is `finalize_options_failure` (both in `src/application/pipeline/action_pipeline/options.rs`). Start each new scenario from a non-empty offered set.

- [ ] #### Task B.1 (G1): Quantifier failure → `docs/specs/actions.md`
  - When the quantifier call fails, or its answer stays unparseable after its retry, the Action still completes: the status returns to Idle and the scene keeps its prior NPC set.
- [ ] #### Task B.2 (G2): Impersonate keeps the offered set → `docs/specs/options.md`
  - When an impersonate turn completes, a non-empty offered set is unchanged. Spec 24.7 asserts only "empty" from an empty start.
- [ ] #### Task B.3 (G3): Turn end clears the offered set → `docs/specs/options.md`
  - When a non-impersonate narration turn ends with always-on off, the offered set is cleared.
  - When an always-on regeneration fails, or no Options agent is registered, the set stays empty and a System message is added. Both are branches of `rewrite_options_after_turn`.
- [ ] #### Task B.4 (G4): No Options agent on demand → `docs/specs/options.md`
  - With no Options agent, `/options` reports that the agent is not available and keeps the set.
- [ ] #### Task B.5 (G5): Mode switch keeps the offered set → `docs/specs/options.md` §Mode neutrality
  - When a Game switches narrator mode, the offered set is neither cleared nor regenerated.

### Phase C: Close

- [ ] #### Task C.1: Validate and delete the scratch folder
  - Run `python build.py`. The full gate passes, with the browser tier included.
  - `python scripts/validate_feature_spec.py` reports 0 gaps and 0 orphans.
  - Delete `.scratch/doc-spec-boundary/`.

## Test Plan
- Phase A: `python scripts/validate_docs.py --strict` (it checks `BROKEN_DOC_ANCHOR`) and `python build.py guardrails`.
- Phase B: `python build.py test-pattern <test name>` for each new test while iterating. Then `python build.py --no-browser`.
- Phase C: the full gate.

## NOT in scope
- Any change to pipeline code. The three pipeline defects live in the Turn state model map.
- New prose in Diátaxis docs for the Phase B rules. A rule gets a spec scenario and a test, or it stays in the code.
- The text-check spec itself. Ticket "Spec and migrate text_check.rs" owns it.
- G9 and the settings read rule (see "Items that need no work").

## What already exists
- Spec scenarios 24.5, 24.7 and 24.12 in `docs/specs/options.md`. Phase B adds scenarios beside them and does not rewrite them.
- `ActionPipeline::rewrite_options_after_turn` and `finalize_options_failure` hold the Options rules that Phase B pins.
- `scripts/validate_docs.py` checks DOC anchors (`BROKEN_DOC_ANCHOR`). `scripts/validate_feature_spec.py` checks that each scenario has a tagged test.

## Failure modes
- A Phase B test fails: the code does not follow the rule from the deleted doc text. The implementer stops and reports. The owner decides whether the code or the rule changes.
- A retargeted DOC anchor names a missing file: `validate_docs --strict` fails with `BROKEN_DOC_ANCHOR`.

## Unresolved decisions
- None for Phases A, B and C.

## Assumptions
- A Phase B rule that the code does not follow is a decision for the owner, not a defect to fix in this plan.
