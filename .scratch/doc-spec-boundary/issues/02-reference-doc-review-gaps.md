# Event rules and doc drift left by the Phase 4 reference-doc review

Status: needs-triage
Type: task

Phase 4 of plan `docs/plans/frontend-docs-rewrite-spec-boundary-and-document-review-cali.md` ran the `document-review` skill over `docs/diataxis/reference/narrative/*`, `game_flow.md`, `storage.md`, `two-state-channels.md` and `storage_design.md` §Messages-and-swipes. The boundary is `docs/AGENTS.md` §Where a fact lives. The deletes removed the event rules below from the Diátaxis docs. No spec scenario carries them yet. No new prose was written for them: a rule gets a spec scenario and a test, or it stays in the code.

## 1. Event rules with no spec scenario

| # | Rule | Removed from | Spec that should own it |
|---|---|---|---|
| G1 | When the quantifier call fails, or its answer stays unparseable after its retry, the Action still completes (status returns to Idle) and the scene keeps its prior NPC set. | `agent_system.md` §Retry Behaviour | `docs/specs/actions.md` |
| G2 | When an impersonate turn completes, the offered Options set is left unchanged. Spec 24.7 asserts only "empty" from an empty start. | `options.md` §Two Triggers | `docs/specs/options.md` |
| G3 | When a non-impersonate narration turn ends, the offered set is cleared even with always-on off. When an always-on regeneration fails, the set stays empty and a System message is added. | `options.md` §Offered-Set Lifecycle | `docs/specs/options.md` |
| G4 | When no Options agent is configured, `/options` reports that the agent is not available and keeps the set. With always-on on, the same report replaces the set with an empty one. | `options.md` §Errors and Limits | `docs/specs/options.md` |
| G5 | When a Game switches narrator mode, the offered set is neither cleared nor regenerated. | `options.md` §Mode Neutrality | `docs/specs/options.md` or `docs/specs/narrator_mode.md` |
| G7 | When the text-check preview opens, no Action is dispatched until the player picks Send with edits or Send Original. `browser_dashboard.md` 16.36 pins only the input text. | `game_flow.md` §Text-Check Branch | the text-check spec (see issue 01 §1) |
| G8 | When arrival narration runs, the status display never shows Generating and no generation slot is taken. Inferred from `ArrivalTaskContext`; confirm it is intended before writing a scenario. | `two-state-channels.md` §Who writes each channel | `docs/specs/` (arrival) |
| G9 | Empty-DB bootstrap creates a Game from `--world`/`--persona`, and an unknown persona fails startup. Not client-observable over HTTP. | `storage.md` §Personas | `startup.md` only if it is a startup invariant; else the code (`bootstrap::run`) |

G6 (text-check fail-open) is not here: the owner kept it in `game_flow.md` §Text-Check Branch as an invariant.

## 2. Doc drift outside the Phase 4 scope (not reviewed in depth)

- **Settings read rule stated three times.** "Settings are read from the DB at the point of use" is in `startup.md` §Settings, `architecture_system.md` §Settings and `storage_design.md`. Keep it in one Reference doc.
- **Misrouted DOC anchors.** `application/llm_recorder.rs`, `application/prompting/sanitize.rs`, `adapters/driven/llm/providers/*.rs`, `adapters/driven/llm/transport/**` and `domain/model/llm_backend.rs` anchor to `prompt_system.md`; their content is in `narration_system.md`. `domain/model/{world,character,scenario}.rs` and `domain/model/utils/{world,scenario}_defaults.rs` anchor to `agent_system.md`, which carries no World content. `application/pipeline/action_pipeline/options.rs` anchors to `game_flow.md`; `options.md` carries it.
- **Spec against code (inferred).** `swipe_new.md` I.6 and 22.1 say a re-impersonate retry does not re-run the quantifier. `ActionPipeline::retry_last_response` (`RetryMode::ReImpersonate`) calls `retry_main_narration`, whose comment says "full tail (quantifier → engine commit → trigger)". One of the two is wrong.
- **Retry naming.** Some code and older text call event retry "event retrigger". `game_flow.md` uses "Event retry"; "Retrigger" is the separate `POST /retrigger` path.
