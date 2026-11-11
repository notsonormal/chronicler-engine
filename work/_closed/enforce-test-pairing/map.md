# Enforce test-file-pairing guardrail

## Destination

The test-file-pairing guardrail (`check_test_file_pairing`) enforces in CI, and every `*_tests.rs` under `src/` has a matching source module — including `retry_tests.rs`, which requires splitting `pipeline.rs`. The guardrail's F1 path-mismatch bug is fixed and lands only after every orphan complies, so CI stays green throughout.

## Notes

- **Domain:** Rust guardrails (`tests/infrastructure/guardrails/`) + the `src/application/pipeline/` module structure. Review the originating review comments and `old-docs/archived-plans/split-pipeline-rs-into-entry-modules.md`.
- **Skills every session should consult:** `/grilling`, `/domain-modeling`, `/code-review`, `/code-simplification`.
- **Standing preferences:**
  - The pipeline split is a no-behavior-change refactor. Any ticket that needs to alter retry/retrigger *behaviour* is out of scope here.
  - For the split decision, **coupling is the gate**: a decomposition that forces ugly `pub(crate)` exposure or back-edges is rejected. Cohesion and testability are supporting checks.
  - F1 (the path-mismatch fix that makes the guardrail actually run) lands **last**, after every orphan test file complies — never before.
  - Task tickets deliver. Research tickets present **multiple viable options** when more than one is practical, not a single answer.

## Decisions so far

- [Verify pipeline.rs split options](issues/01-verify-pipeline-split-options.md) — Adopt modified 4-way split: `core.rs` (state + shared orchestration incl. `retry_event_continuation`), `action.rs`, `retry.rs`, `retrigger.rs`; no `pub(crate)` exposure needed; see [research summary](01-verify-pipeline-split-options-summary.md). Unblocks [Execute pipeline split](issues/07-execute-pipeline-split.md).
- [Move parser_tests.rs into utils/](issues/02-move-parser-tests-to-utils.md) — Moved to `src/application/agents/quantifier/utils/parser_tests.rs`, re-exported via `utils/mod.rs`, parent mod cleaned up; `cargo test --lib quantifier` passes (72 tests).
- [Move context_tests.rs into utils/](issues/03-move-context-tests-to-utils.md) — Moved to `src/application/prompting/utils/context_tests.rs`, declared in `utils/mod.rs`, removed from `prompting/mod.rs`; `cargo test --lib prompting` passes (54 tests).
- [Consolidate game_state_*_tests.rs into game_state_tests.rs](issues/04-consolidate-game-state-tests.md) — Merged three orphan `game_state_*_tests.rs` files into `src/domain/model/state/game_state_tests.rs`; deleted orphans; `cargo test --lib game_state` passes (61 tests).
- [Fix minimal_app_no_game to use the canonical helper](issues/05-fix-minimal-app-no-game-helper.md) — Rewrote `src/application/games/view_query_tests.rs` helper to build the pipeline via `make_test_pipeline_with_backends`, removing duplicated `MessageService`/`AppSettings` construction; `cargo test --lib view_query` passes (11 tests).
- [Rename heal_stale_* tests to test_ prefix](issues/06-rename-heal-stale-tests.md) — Prefixed the three pre-existing `heal_stale_*` test functions in `src/application/generation/gate_tests.rs` with `test_`; `cargo test --lib generation` passes (18 tests).
- [Execute pipeline split](issues/07-execute-pipeline-split.md) — Split `pipeline.rs` into `core.rs`/`action.rs`/`retry.rs`/`retrigger.rs` with a shared `claim_and_spawn` helper in `core.rs`; moved 3 `PipelineRun` methods to `phases.rs`; split `pipeline_tests.rs` into `core_tests.rs`/`action_tests.rs` and retrigger tests into `retrigger_tests.rs`; `retry_tests.rs` now paired with `retry.rs`. Full gate green, no behaviour change.

- [Enable the test-file-pairing guardrail (fix F1)](issues/08-enable-guardrail-f1.md) — Passed the full `src/...` path to `check_test_file_location` in the guardrail harness, fixing the F1 path-mismatch no-op; `cargo nextest run --test guardrails` and `python build.py` are green with zero orphan test files.

## Not yet specified

_(None — map is complete; the destination is reached.)_

## Out of scope

- **Behaviour changes to retry/retrigger logic.** This effort is structural compliance + refactor; any logic change is a separate effort.
- **The `bun.lock` deletion** flagged in review — already excluded by the user, not part of this work.
