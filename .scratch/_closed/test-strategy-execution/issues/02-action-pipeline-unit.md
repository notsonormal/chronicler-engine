# Port action pipeline unit tests down from the component tier

Type: task (HITL)
Status: resolved

## Question

Port action pipeline branch-coverage assertions from the component tier
into the in-memory unit suite
(`src/application/pipeline/pipeline_tests.rs`), and fill the
`GameCatalogue` branch coverage gap. These are branch-coverage questions
asserted on internal state with fakes — unit tests doing their job per
the tier rules (07/13).

### From the diff asset (action_pipeline/actions.rs + pipeline.rs)

Per the [diff asset](../../test-strategy/assets/pipeline-suite-diff.md):

- **~16 port down:** input-before-narration ordering (S1.2),
  room-not-found error + message (S2.1, currently drift — assert `Error`
  not just `!is_generating()`), `last_trigger` cleared (S3.1), phase
  stays `Narrating` on error (S3.4), empty input → continuation (S1.5),
  quantifier movement detection (S1.3), trigger firing +
  `event_header` + narration count (S1.4), empty-LLM message contains
  `"empty"` + no narration persisted (S2.3), S2.4 trigger-only failure
  (Idle + main preserved + System log — currently mislabeled in-memory),
  sync cancellation (S4.1), snapshot presence (S5.1, S5.2).
- **6 async → `#[tokio::test]`:** deadlock prevention (S3.3), async
  cancellation timing (S4.2, S4.3), mid-flight observation (S3.2).
  These are unit tests doing their job — async timing with fakes, not a
  separate tier.
- **9 covered → delete:** pipeline logic duplicated in the in-memory
  suite. Delete after confirming the unit assertions are complete.
- **4 drift → resolved by retry spec (ticket 05 on planning map).** The
  spec defines correct behaviour; write assertions to match the spec,
  not the old test.

### From collaborators.rs (2 tests)

- `persists_input_message` → port the assertion down (input-before-
  narration ordering). If already covered by the S1.2 port above, delete.
- `self_heals_stale_generating_status` → port if `heal_stale` branch
  isn't already covered in unit; otherwise delete.

### GameCatalogue branch coverage gap

`GameCatalogue` (`src/application/games/catalogue.rs`) has uncovered
validation/orchestration branches: world-not-found, persona-not-found,
can't-delete-active-game, name-generation uniqueness, rollback on persist
failure. The 5 integration tests in `collaborators.rs` were all
happy-path. Per the tier rules, add unit tests with in-memory `Storage`
for every branch.

While doing this, note whether other application-layer classes
(`GameViewQuery`, `MessageService`, `GameCatalogue.reset()`) have similar
gaps. If this turns out to be a pattern, it graduates from the map's
Not-yet-specified into its own ticket.

### Acceptance

- Every branch in `ActionPipeline` and `GameCatalogue` has a unit test.
- The 9 covered component-tier tests are deleted.
- The component-tier files (`action_pipeline/actions.rs`,
  `action_pipeline/pipeline.rs`) are deleted after their assertions are
  ported.
- `collaborators.rs` is deleted (5 redundant + 2 ported).
- Suite green.

## Answer

Component tier dissolved for the action pipeline + collaborators +
GameCatalogue. All branch-coverage assertions now live in the unit tier
(`src/`); the component-tier files are deleted; suite green (1358 pass,
2 skipped; 26/26 guardrails; clippy 0 errors).

### New unit test files

- `src/application/games/catalogue_tests.rs` (12 tests) — every
  `GameCatalogue` branch: world-not-found, persona-not-found, name
  uniqueness, rollback on persist failure (restores `current_game_id`),
  switch success + not-found, delete non-active + active-rejected,
  list, reset replaces current, current_game_id mirror. Wired via
  `#[cfg(test)] mod catalogue_tests;` in `games/mod.rs`.
- `src/application/generation/gate_tests.rs` (3 tests) — `heal_stale`
  branches: resets stale `Generating` when no slot owns the game;
  leaves `Generating` intact when a slot is active; noop on `Idle`.
  Covers the `test_process_action_self_heals_stale_generating_status`
  collaborator test. Wired via `#[cfg(test)] mod gate_tests;` in
  `generation/mod.rs`.

### Ported into `src/application/pipeline/pipeline_tests.rs`

Sync port-downs (new tests, SCENARIO-tagged):
- S1.2 input-before-narration ordering
- S1.3 quantifier movement detection (two-room map, destination move)
- S1.5 empty input → continuation narration, no Input message
- S2.1 room-not-found → `GenerationStatus::Error` with `"Room not found"`
- S3.1 `execute_action` clears `last_trigger`
- S3.4 phase stays `Narrating` on narration failure (verified empirically
  — `finalize_phase_error` does not overwrite the pre-main snapshot's
  phase; matches spec S3.4)
- S4.1 sync cancellation (token already cancelled)
- S5.1 pre-main snapshot present after action
- S5.2 pre-event snapshot present after trigger continuation

Strengthened existing test:
- S1.4 `test_pipeline_trigger_happy_path` gained `event_header` presence
  + narration-count ≥ 2 assertions (the partial port-down).

Async port-downs (the "6 async → `#[tokio::test]`/thread" set):
- S3.3 delayed LLM completes without deadlock (sync, 200ms delay)
- S3.2 streaming narration saved before quantifier completes
  (mid-flight observation via `thread::spawn` + 400ms polling; quantifier
  delayed 500ms)
- S4.1 cancellation resets state to idle (sync, pre-cancelled token)
- S4.2 cancel after main narration (`#[tokio::test]`, `spawn_blocking`,
  poll `narration_started`, cancel)
- S4.3 cancel during trigger continuation (`#[tokio::test]`, poll
  `trigger_started`, cancel, main narration preserved)

Note: `run_from_input` does not consult `shutdown_token` directly — the
S4.2/S4.3 tests faithfully port the component-tier assertions (status
Idle after cancel); the real cancellation seam is `check_game_unchanged`
(game-id flip), already exercised in `retry_tests.rs`. The port keeps
the component-tier contract rather than inventing a stronger one.

### S2.3 empty-LLM assertion

S2.3 was a partial port-down — the existing
`test_pipeline_returns_error_on_empty_narration_text` asserted error
status only. Added `test_pipeline_empty_narration_sets_error_message_and_no_narration`
asserting the message contains `"empty"` and no narration is persisted.

### Deleted component-tier files

- `tests/integration/application/collaborators.rs` — 5 redundant CRUD
  happy-paths (covered by `catalogue_tests.rs`) + 2 ported
  (`persists_input_message` → S1.2 unit; `self_heals_stale_generating_status`
  → `gate_tests.rs`). `mod application_collaborators;` removed from
  `tests/integration/mod.rs`.
- `tests/integration/application/action_pipeline/actions.rs` (8 tests) —
  3 covered (deleted) + 5 ported down.
- `tests/integration/application/action_pipeline/pipeline.rs` (15 tests) —
  3 covered (deleted) + port-downs + async port-downs. `mod pipeline_actions;`
  and `mod pipeline_tests;` removed from `tests/integration/mod.rs`.

### Drift deferred to ticket 05 (per ticket 02's note) — RESOLVED by ticket 11

S2.4 (`test_failing_trigger_narration_does_not_crash`) was one of the 4
drift cases assigned to ticket 05 (retry spec). The component test was
deleted with `pipeline.rs`. **Ticket 11 resolved the drift** during the
spec restructure: S2.4 spec corrected from Idle → Error (matches
implementation: trigger narration failure → `set_error`), HTTP E2E test
added in `tests/http/actions.rs` using `with_trigger_narration_fail()`.

The mislabeled `test_pipeline_trigger_complete_failure` (uses
`with_fail()` not `with_trigger_narration_fail()`) remains as unit-tier
cleanup fog, now tracked by ticket 04 (see ticket 04's ticket-11 follow-ups
section).

### Not-yet-specified graduation (no new ticket yet)

`GameCatalogue.reset()` has three error branches not exercised by the
new unit tests: current-game-not-found, world-not-found after delete,
persona-not-found. These are unreachable through the public storage
seam in an in-memory setup (the world/persona that created the game
exist; `delete_world` is blocked by the game row). `reset`'s
`persist_initial_state_with_swipes` failure is also silently swallowed
(`let _ = …`). This matches the map's Not-yet-specified "branch coverage
pattern" fog: if `GameViewQuery` and `MessageService` show similar
gaps when audited, it earns its own ticket. Not sharp enough to ticket
now — recorded as a signpost.

### Verification

- `cargo nextest run -p chronicler_engine`: 1358 passed, 2 skipped
  (platform-specific). Was 1367 before; net -9 (deleted 23 component
  tests, added 14 unit tests, 2 of which replace collaborators ports).
- `cargo nextest run -p chronicler_engine guardrails`: 26/26 pass
  (import-ordering, test-layer-boundaries, test-file-location all green
  for the new files).
- `cargo clippy --tests`: 0 errors. 4 pre-existing
  `format!("...{var}")` warnings in `retry_tests.rs` (unchanged).
