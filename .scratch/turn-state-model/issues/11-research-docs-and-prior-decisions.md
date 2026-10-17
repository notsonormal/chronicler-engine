# Research: check the docs and prior decisions against the code

Type: research
Status: resolved
Blocked by: —

## Question

What do the current docs, the specs and the closed maps state about a generation's states, its Snapshot pointers, the retry forms and the game-state writers, and where does the code disagree? Which prior decisions constrain the model?

## Scope

1. Docs: `docs/diataxis/reference/game_flow.md`, `docs/diataxis/explanation/storage_design.md` (§Per-swipe state binding), `docs/diataxis/explanation/two-state-channels.md`, `docs/diataxis/reference/storage.md`, and the Message, Swipe, Retry, Snapshot and generation entries in `CONTEXT.md`.
2. Specs: `docs/specs/swipe_new.md`, `swipe_switch.md`, `retrigger.md`, `actions.md`, `failure_display.md`.
3. Prior decisions: the closed map [guided generations architecture](../../_closed/guided-generations-architecture/map.md) and its tickets; tickets 09, 10, 15 and 16 of [the dashboard UI review phase 2](../../dashboard-ui-review-phase2/map.md); any ADR under `docs/` on these subjects.
4. For each claim: quote it, mark whether the code agrees (known from the code, or inferred), and cite the symbol that decides it.
5. List the decisions that a new model must keep (for example guided Retry starts from the state before its Narration), and the decisions that the seed says the owner already ruled (the docs are right, the code is wrong for per-Swipe state).

## Context

- Seed: [three pipeline defects with one root](../assets/seed-evidence.md).
- Out of scope for this map: the follow-ups in `old-docs/archived-plans/doc-spec-boundary-follow-ups.md`. Record a drift you find there, but do not plan its fix.

## Output

An asset `../assets/research-11-docs-and-decisions.md`: a claim table (source, claim, code agrees?, deciding symbol) and a list of binding prior decisions with links. Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

[Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md).

## Answer

[Asset](../assets/research-11-docs-and-decisions.md): a 44-row claim table with verbatim quotes, a code verdict and a deciding symbol per row; 18 binding prior decisions; 16 drifts; and the open questions.

The docs are right about intent and loose about mechanism. Only one doc claim is outright wrong in intent (`storage_design.md` §Per-swipe state binding), and the owner already ruled that the code — not the doc — is at fault. The rest split three ways:

- **True intent, wrong detail.** `storage.md`'s "switching swipes restores that exact state" omits that `switch_swipe` forces `Idle`, the default phase and no options. `CONTEXT.md` calls a Swipe "an alternate version of an AI-generated Message" and swiping "on the last message", but every Message gets a Swipe and only the *switch* is last-Message-bounded.
- **Wrong reason, true observable.** `swipe_new.md` I.6, 10.2 and `retrigger.md` 13.2 claim no Quantifier re-run on an event redo or a retrigger. `PipelineRun::reconcile_post_trigger_npcs` does run a Quantifier pass there. The pass cannot move the player, so each scenario's room claim holds and each parenthetical reason does not.
- **Not a doc drift at all.** The per-Swipe pointer fault, the missed writer rule and the arrival race are code defects that no document contradicts.

Two prior decisions conflict, and the model must resolve them before anything else. The owner ruled a Swipe points at the state its own narration produced, Quantifier included ([seed](../assets/seed-evidence.md) §1). The closed map's [Deepen the narration turn module](../../_closed/guided-generations-architecture/issues/01-deepen-narration-turn-module.md) decision 3 ruled that "Swipe snapshots are captured pre-tail at message-add time in every path". Both cannot stand. That conflict is the first question of [Model a generation's states](01-model-a-generations-states.md).

Other facts this ticket settles for the grilling tickets: `swipe_new.md` I.6 and 22.1 are stale in behaviour, not only in reason — a re-impersonate Retry runs the full tail, confirmed by `test_retry_flow_impersonate_mode_runs_full_tail`; a redo and a retrigger can each add a Message, so I.2, I.3 and `CONTEXT.md`'s Retry entry overstate; `gg03`'s entry-time preset pin is superseded by `NarrationGeneration::resolve_preset_choice`; no ADR exists; and `CONTEXT.md` has no generation entry, so the model must add its own terms.

### Verification

The coordinator checked the asset before acceptance: all 63 quoted strings match their source files verbatim after whitespace normalisation; all 44 rows attribute their quotes to the labelled source; and all 51 `src/` symbol citations resolve to a real path and a real symbol. One correction followed: the `gg03` row and drift 16 said `impersonate_preset_id` no longer exists in `src/`. `GenerationReplay` is gone and the stored-inputs blob no longer carries a preset id, but the name survives as the settings field `active_impersonate_preset_id` and as a game form field.
