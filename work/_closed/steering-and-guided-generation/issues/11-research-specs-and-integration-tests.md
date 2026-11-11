# Research: existing specs and requires_migration audit, then spec + integration-test authoring

Type: research
Status: completed
Assignee: pi-agent

## Question

Before implementation tickets land, audit existing specs and the `tests/http/requires_migration/` directory for behavior this effort changes, then author the feature specs and integration tests that will gate the implementation.

This is a research ticket because the spec/test landscape must be understood before writing it — there may be existing specs covering narrator/guide/impersonate behavior, or migrated-pending tests in `tests/http/requires_migration/` that this effort must reconcile with.

### Steps

1. **Audit existing specs.** Read `docs/specs/` (today: `actions.md`, `swipe_new.md`, `retrigger.md`, per `docs/CHANGELOG.md`). Check whether any existing spec covers narrator messages, guide/steering injection, or impersonate. The `validate_feature_spec.py` script ties specs to integration tests — understand its rules before authoring.

2. **Audit `tests/http/requires_migration/`.** The directory holds migrated-pending test files: `connections.rs`, `core.rs`, `debug.rs`, `fragment.rs`, `games_fragment_handlers.rs`, `index_handler.rs`, `server_impl_wiring.rs`, `text_check.rs`, `worlds_fragment_handlers.rs`. Determine which (if any) cover behavior this effort changes — e.g. action dispatch, message rendering, swipe behavior — and must be migrated or updated as part of this effort.

3. **Author feature specs** under `docs/specs/` for the three steering surfaces, following the existing endpoint-named convention. Each spec's scenarios must map to integration tests (per `validate_feature_spec.py`).

4. **Author integration tests** under `tests/http/` covering: narrator type persistence + bare rendering; guide layer position + transience (not in history); guide retry via replay blob; impersonate preset selection + context-layer filtering + persona injection + player-voiced output; impersonate retry via blob; slash-command parser dispatch; mutual exclusivity of guide/impersonate; narrator generate-then-add.

5. Produce a markdown summary asset (like the other research tickets) of the audit findings + the spec/test plan.

### Why this gates implementation

The map's standing preference is test-first: "If you don't understand how a component works, read its tests before the source." Implementation tickets 05–10, 12 all carry `Blocked by: 11` so the specs and tests exist before the code. This ticket's number (11) is referenced by those blockers.

Blocked by: 04 (design synthesis must be resolved first).

## Answer

Audit complete. Existing specs (`docs/specs/*.md`) and `tests/http/requires_migration/` do not cover narrator/guide/impersonate behavior; steering is a new surface requiring new specs and tests.

Asset produced: `research/11-specs-and-integration-tests.md` containing:
- Audit findings for `docs/specs/` and `tests/http/requires_migration/`.
- Full proposed `docs/specs/steering.md` with scenarios 22.1–25.3 and invariants.
- Proposed `tests/http/steering.rs` integration-test outline and full proposed test code.
- Six open questions for the follow-up grilling ticket.

The proposed spec/test files are intentionally not committed to the repo from this research ticket; a new grilling ticket (14) will review the asset and decide the open questions before implementation tickets proceed.
