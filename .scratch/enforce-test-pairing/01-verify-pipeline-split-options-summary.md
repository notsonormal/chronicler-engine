# Pipeline.rs Decomposition Research Summary

**Ticket:** [Verify pipeline.rs split options](issues/01-verify-pipeline-split-options.md)  
**Scope:** Decide how to split `src/application/pipeline/pipeline.rs` (802 lines) so `retry_tests.rs` and any other split test files gain matching source modules, without changing behavior.

## Executive Summary

`pipeline.rs` currently mixes four responsibilities: shared `ActionPipeline` state/constructors, the action entry path, the retry entry path, and the retrigger entry path. The two viable decompositions are a **4-way split** and a **3-way split**. Both satisfy the coupling gate if one shared helper (`retry_event_continuation`) is placed in the shared core. A 2-way split is also possible but sacrifices the cohesion that justifies the refactor.

**Recommendation:** Use the **modified 4-way split** — `core.rs`, `action.rs`, `retry.rs`, `retrigger.rs` — with `retry_event_continuation` living in `core.rs` rather than `retry.rs`. This keeps every entry module focused on a single public path, avoids a `retrigger → retry` back-edge, and creates a natural home for `retrigger_tests.rs`.

## Candidate Decompositions

### Option 1 — Modified 4-way split (recommended)

| Module | Contents |
|--------|----------|
| `core.rs` | `ActionPipeline` struct, constructors, `backend_info`, `recorder`, `prompt_assembler`, `rebind_for_test`, `is_shutting_down`, `reset_persisted_status`, `load_world_bundle`, `finalize_phase_error`, `persist_generation_error`, `phase_trigger_continuation`, `run_post_generation_agents`, `run_from_input`, `log_cancellation`, `retry_main_narration`, `retry_event_continuation`, optional `claim_and_spawn` helper |
| `action.rs` | `process_action`, `continue_narration`, `execute_action` |
| `retry.rs` | `retry`, `retry_last_response`, `check_retry_anchor` |
| `retrigger.rs` | `retrigger`, `retrigger_event` |

**Coupling map:**
- All entry modules depend on `core.rs` only.
- `core.rs` depends on no entry module.
- `phases.rs` continues to read `ActionPipeline` fields through `PipelineRun`; fields can stay `pub(super)` because `core.rs` is a sibling under `application::pipeline`.
- `spawn.rs` needs the `ActionPipeline` type and the entry methods it calls in closures (`execute_action`, `retry_last_response`, `retrigger_event`). All are already public or can be raised to `pub(super)`.
- `retry_event_continuation` is called by both `retry_last_response` and `retrigger_event`, so it lives in `core.rs` and is callable from both modules. This eliminates a `retrigger → retry` back-edge.

**Cohesion verdict:** High. Each entry module has exactly one public entry path. `core.rs` is the only shared module and is clearly "state + shared orchestration".

**Testability verdict:** High. The existing `pipeline_tests.rs` (1738 lines) splits naturally into:
- `core_tests.rs` for `run_from_input` and shared orchestration tests (the majority of current tests).
- `action_tests.rs` for `process_action` / `continue_narration` / `execute_action` tests.
- `retry_tests.rs` keeps its retry tests and loses retrigger tests.
- `retrigger_tests.rs` is created for the ~6 retrigger tests currently inside `retry_tests.rs`.

### Option 2 — 3-way split (retry + retrigger merged)

| Module | Contents |
|--------|----------|
| `core.rs` | Same as Option 1, but `retry_event_continuation` can stay in `rerun.rs` because it is shared only within that module |
| `action.rs` | Same as Option 1 |
| `rerun.rs` | `retry`, `retry_last_response`, `check_retry_anchor`, `retrigger`, `retrigger_event`, `retry_event_continuation` |

**Coupling map:**
- `action.rs` and `rerun.rs` depend only on `core.rs`.
- `core.rs` depends on no entry module.
- Same visibility story as Option 1, except `retry_event_continuation` is private to `rerun.rs`.

**Cohesion verdict:** Medium-high. Retry and retrigger are both "re-run an existing generation" paths, so they share a conceptual domain. However, they have distinct public APIs (`retry` vs `retrigger`) and distinct preconditions (input anchor vs trigger context), so a single module is slightly less focused than separate modules.

**Testability verdict:** Medium-high. `retry_tests.rs` can be renamed to pair with `rerun.rs`, or a new `rerun_tests.rs` can be created. The test file stays larger because it contains both retry and retrigger tests. If the project later adds more retrigger behavior, the file will grow past the desired size again.

### Option 3 — 2-way split (all entries in one module)

| Module | Contents |
|--------|----------|
| `core.rs` | State, constructors, and shared orchestration |
| `entries.rs` | `process_action`, `continue_narration`, `execute_action`, `retry`, `retry_last_response`, `check_retry_anchor`, `retrigger`, `retrigger_event` |

**Coupling map:**
- `entries.rs` depends on `core.rs` only.
- `core.rs` depends on no entry module.
- Fewer files, so fewer visibility edges to reason about.

**Cohesion verdict:** Low. The file still mixes action, retry, and retrigger paths; it is only slightly better than the current `pipeline.rs`.

**Testability verdict:** Low. `entries_tests.rs` would be large and unfocused, and we would still need to split it later to satisfy the test-file-length guardrail. The `retry_tests.rs` orphan problem is also solved less cleanly because the source module would be `entries.rs`, not `retry.rs`.

## Claim-and-Spawn Helper

The plan proposes extracting a `claim_and_spawn` helper from the duplicated sequence in `process_action`, `retry`, and `retrigger`. The shared steps are:

1. `is_shutting_down()` check.
2. Load or fresh game state.
3. Read `storage.current_game_id()`.
4. `generation_gate.heal_stale(game_id, &mut game_state)` (action and retry only; retrigger does not need this).
5. Pre-claim validation (input exists for retry; trigger context exists for retrigger; action has no extra check).
6. `generation_gate.try_claim(...)`.
7. Optional post-claim validation (`check_retry_anchor` for retry only).
8. `spawn_pipeline_task(...)` with a closure that calls the real work.

A generic helper can remove the duplication, but the pre-check and post-check differences make the generic signature non-trivial. Two practical choices:

- **Helper in `core.rs`:** `claim_and_spawn(pre_check, post_check, work)` style. Reduces duplication but introduces a generic closure API. Worth doing if the duplication is judged to be a maintenance burden.
- **Keep duplication explicit:** Each entry method keeps its own gate/claim/spawn sequence. Simpler to read, slightly more lines. This is acceptable if the team prefers readability over DRY for this small surface.

The recommendation is **agnostic on the helper**: either choice fits the 4-way split. If the helper is added, it lives in `core.rs` and is called by `action.rs`, `retry.rs`, and `retrigger.rs`.

## Visibility Notes

- `ActionPipeline` fields are currently `pub(super)`. Because `core.rs` is a sibling of `action.rs`, `retry.rs`, `retrigger.rs`, `phases.rs`, and `spawn.rs` under `application::pipeline`, `pub(super)` remains sufficient. No `pub(crate)` exposure is required.
- Methods that move from `pipeline.rs` to `core.rs` and are called from sibling modules need to be at least `pub(super)`. Methods already at `pub(crate)` or `pub` can keep their current visibility.
- `check_retry_anchor` can remain private to `retry.rs` because no other module calls it.

## Final Recommendation

Adopt **Option 1 — the modified 4-way split**:

- `core.rs` for `ActionPipeline` state, constructors, and shared orchestration, including `retry_event_continuation`.
- `action.rs` for the action entry path.
- `retry.rs` for the retry entry path.
- `retrigger.rs` for the retrigger entry path.

This is the best balance of cohesion, testability, and minimal coupling. It creates the matching source module for `retry_tests.rs`, provides a clean home for `retrigger_tests.rs`, and keeps all entry modules dependent only on `core.rs`.

## Deliverables for the Next Ticket

The next ticket, [Execute pipeline split](issues/07-execute-pipeline-split.md), should implement this decomposition. It will need to:

1. Create `core.rs`, `action.rs`, `retry.rs`, and `retrigger.rs`.
2. Move the methods listed above, adjusting visibility to `pub(super)` where needed.
3. Update `src/application/pipeline/mod.rs` to declare the new modules and re-export `ActionPipeline` from `core.rs`.
4. Move tests from `pipeline_tests.rs` into `core_tests.rs`, `action_tests.rs`, and `retrigger_tests.rs`, keeping `retry_tests.rs` for retry tests.
5. Verify with `cargo check --all-targets`, `cargo nextest run -p chronicler_engine`, and `python build.py`.
