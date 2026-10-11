# Research: check the docs and prior decisions against the code

Type: research
Status: claimed
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
