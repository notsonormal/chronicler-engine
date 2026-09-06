# Execute pipeline split

Type: task
Status: resolved
Blocked by: 01

## Answer
Split `src/application/pipeline/pipeline.rs` (802 lines) into the modified 4-way decomposition from [Verify pipeline.rs split options](01-verify-pipeline-split-options.md):

- `core.rs` — `ActionPipeline` struct, constructors, `claim_and_spawn` gate/spawn helper, `run_from_input`, `finalize_phase_error`, `persist_generation_error`, `load_world_bundle`, `phase_trigger_continuation`, `run_post_generation_agents`, `log_cancellation`, `retry_event_continuation`, `retry_main_narration`.
- `action.rs` — `process_action`, `execute_action`, `continue_narration`.
- `retry.rs` — `retry`, `retry_last_response`, `check_retry_anchor`.
- `retrigger.rs` — `retrigger`, `retrigger_event`.

`pipeline.rs` and `pipeline_tests.rs` deleted. `pipeline/mod.rs` declares the four new modules and re-exports `ActionPipeline` from `core.rs`. The three `PipelineRun` methods that lived in `pipeline.rs` (`phase_pre_main_snapshot`, `phase_finalize`, `handle_cancellation`) moved into `phases.rs`. `spawn.rs` and `phases.rs` import `ActionPipeline` from `core.rs`; all external callers updated from `pipeline::pipeline::ActionPipeline` to the `pipeline::ActionPipeline` re-export.

`claim_and_spawn` (in `core.rs`) consolidates the duplicated `is_shutting_down → load → heal_stale → before_claim → try_claim → pre_spawn → spawn_pipeline_task` sequence used by all three entry points. Retrigger's full validation (trigger + last-message checks) runs in `before_claim` (pre-claim) to match the original no-claim-on-validation-failure behaviour; retry's `check_retry_anchor` runs in `pre_spawn` (post-claim) with slot release on failure, matching the original.

Test reorg: `pipeline_tests.rs` split into `core_tests.rs` (run_from_input / phase orchestration) and `action_tests.rs` (execute_action / process_action entry). `retry_tests.rs` retained and its retrigger tests moved to `retrigger_tests.rs`; shared helpers made `pub(super)` and imported from `retry_tests`. `retry_tests.rs` is now paired with `retry.rs` — the test-file-pairing guardrail passes with no orphans.

Verified: `cargo check --all-targets`, `cargo clippy --all-targets -- -D warnings`, `cargo nextest run -p chronicler_engine` (1355 passed, 2 LLM-skipped), `python build.py` (full gate green), `python scripts/check_test_structure.py`, and `guardrails_test_file_location` all pass. No behaviour change.

## Question

Split `src/application/pipeline/pipeline.rs` (802 lines) into entry modules so that `retry_tests.rs` (and any other split test file) gains a matching source module. **Execute the decomposition selected by [Verify pipeline.rs split options](issues/01-verify-pipeline-split-options.md)** — the modified 4-way split: `core.rs`, `action.rs`, `retry.rs`, `retrigger.rs`, with `retry_event_continuation` in `core.rs`. See the [research summary](01-verify-pipeline-split-options-summary.md) for the full mapping and visibility notes.

Constraints:
- **No behaviour change.** Pure file-level refactor; all existing tests must pass unchanged in behaviour.
- Sibling test files follow the `_tests.rs` convention: each new source module gets a matching `_tests.rs`, and tests move out of `pipeline_tests.rs` (1738 lines) into the appropriate sibling file. `retry_tests.rs` must end up paired with its module.
- `pipeline/mod.rs` re-exports `ActionPipeline` from wherever the struct/constructors land.
- `PipelineRun` and phase logic stay in `phases.rs`; `phase_error.rs` and `spawn.rs` stay unless the split decision moves the `claim_and_spawn` helper into `spawn.rs`/`core.rs`.

The existing plan (`old-docs/archived-plans/split-pipeline-rs-into-entry-modules.md`) proposes three phases (core+spawn helper → entry modules → test reorg) and rates this 8 SP. If the work exceeds one session, graduate sub-tickets from the map's **Not yet specified** section rather than overrunning. Verify with `cargo check --all-targets`, `cargo nextest run -p chronicler_engine`, and `python build.py`.
