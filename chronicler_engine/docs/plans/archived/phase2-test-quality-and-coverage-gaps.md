# Plan: Phase 2 Test Quality Cleanup + Coverage Gap Fixes

**Date:** 2026-07-02
**Status:** Draft (not yet started)
**Scope:** `chronicler_engine/`
**Branch target:** `hexagon-phase2` fix-up commits (off current `e0cd301`)
**Total story points:** 43

## Context

Two independent audits of the Phase 2 test layer surfaced complementary problems:

- **Test code quality** — External thermonuclear review of commit `ba35ac5` found 9 structural issues in test code: dead helper, tautological assertions, duplicate tests, fake regression guards, weak identity, over-engineered stub, doc-comment inaccuracy. Review verdict: **Block**. Tests pass but carry low-value / dead / tautological coverage violating the project's "no wrapper / no placeholder" bar at the test layer.
- **Coverage gaps** — Investigation of `tests/` vs `src/` tree found several production files with real coverage gaps, including `server_impl.rs` (0%, 62 LOC) and `transport/client.rs` (40.9%, 88 LOC).

Both audits target the same goal: **tests must earn their keep**. Reviewer findings remove tests that pretend to cover; coverage gaps identify src files that genuinely lack coverage. Combined plan addresses both.

## NOT in scope

- **Production code refactor** — tests + docs only. If a worker discovers a real bug, it stops and reports per AGENTS.md plan-adherence rule.
- **`tests/http/fragment.rs` god-file split** — large refactor (86 tests, 1300+ LOC). Held flat per prior locked decision.
- **Real LLM API coverage for `driven/llm/providers/`** — `openrouter.rs` (29.2%), `ollama.rs` (32.2%) low but expected; real-API tests live in `tests/llm/` (`#[ignore]`).
- **Per-file coverage gates** — overall ≥80% gate only.
- **`tests/integration/{ports,agents,narrative_prompt,text_check}/` deferred dirs** — fully covered via sibling `*_tests.rs`.
- **Test-police expected-low files (NOT targeted even if low coverage):** `cli.rs`, `port_utils.rs`, `bootstrap/` (incl. `load.rs`, `logging.rs`, `init_game.rs`), `main.rs`, `test_support/`, `router.rs` (100% already).

## Success criteria

### Axis C — Test quality cleanup (reviewer findings F1–F6, M1–M3)

| # | Finding | SP | Fix | Verification |
|---|---|---:|---|---|
| F1 | `make_recording_recorder` helper dead code | 1 | Delete helper + re-export in `mod.rs` | `grep make_recording_recorder src/ tests/` = 0 matches |
| F2 | Tautological `assert!(x.is_some() ‖ x.is_none())` | 1 | Delete 2 assertions in `text_check_factory_tests.rs` (sibling `harper_text_checker_tests.rs` covers real Harper behavior) | No `is_some() ‖ is_none()` patterns in `src/` or `tests/` |
| F3 | Duplicated/trivial tests | 1 | Delete entirely (no merging — survivors cover invariants): `method_signature_compiles`, `llm_message_clone_produces_equal_value`, `llm_message_debug_format_contains_field_names`, `llm_message_construction_all_fields`, `complete_preserves_raw_response_for_forensic_audit`, `injection_with_custom_checker` | `complete_strips_thought_tags_from_parsed_response` covers raw-vs-parsed invariant |
| F4 | `provider_accessor_returns_injected_provider` weak identity | 3 | Rewrite with `Arc::ptr_eq` (exact snippet in Phase 3a). Drop misleading "requires Weak or Rc" comment. | Test fails when different provider injected |
| F5 | Factory tests only assert `Ok` | 2 | Add `assert_eq!(recorder.unwrap().provider().name(), "DeepSeek" ‖ "OpenRouter" ‖ "Ollama")` to 3 path tests | Test fails if wiring returns wrong provider |
| F6 | `with_storage_provider_name_is_consistent_across_calls` fake Fix-6 guard | 1 | Drop test entirely. Delete dead `let _ = AgentRegistry::default();` line | `grep "let _ = AgentRegistry::default()" tests/` = 0 |
| M1 | Unused `Arc::clone` in `text_check_service_tests.rs` | 3 | Per-site: remove `.clone()` where `checker` not referenced after `TextCheckService::new`. Where referenced, keep clone. Worker verifies per-site. | `grep "checker.clone()" src/application/text_check_service_tests.rs` = 0 |
| M2 | Over-engineered `StubChecker` triple-nested `Option<Result<Option<...>>>` | 3 | Replace with `Mutex<VecDeque<Result<Option<CheckResult>, EngineError>>>` queue; `check()` pops front, returns `Ok(None)` when empty. | Tests still pass; stub state is a queue |
| M3 | `save_call_count` doc comment inaccurate | 1 | Update doc to "Number of times `save_llm_message` has completed without a configured error" | Doc matches behavior |
| | **Axis C total** | **16** | | |

### Axis D — Real coverage gaps (3-tier prioritization)

Lens per file: (1) overall coverage % from `target/llvm-cov/coverage.json`, (2) sibling `*_tests.rs` presence, (3) test-police expected-low list.

**Tier 1 — real coverage gaps (HIGH priority, clear ROI):**

| # | Target file | Cov % | LOC | New/extended file | SP |
|---|---|---:|---:|---|---:|
| D1 | `src/adapters/driving/http/server_impl.rs` | 0% | 62 | `tests/http/server_impl_wiring.rs` (new, http wiring test following existing `tests/http/` patterns) | 3 |
| D2 | `src/adapters/driven/llm/transport/client.rs` | 40.9% | 88 | `src/adapters/driven/llm/transport/client_tests.rs` (new sibling — cover `call_openrouter_with_model`, `call_ollama` non-API branches) | 5 |
| D3 | `src/adapters/driving/http/prompt_presets_fragment/fragments.rs` | 55.8% | 154 | Extend existing `fragments_tests.rs` | 3 |

**Tier 2 — sibling isolation tests (MEDIUM, add ONLY if unique value not covered by integration):**

| # | Target file | Cov % | Sibling? | New file | SP | Gate |
|---|---|---:|---|---|---:|---|
| D4 | `src/application/action_pipeline/phases.rs` | 90.2% | ✗ | `src/application/action_pipeline/phases_tests.rs` | 3 | Worker must identify unique value not reachable via `pipeline_tests.rs`. If none, skip + document. |
| D5 | `src/application/message_editing.rs` | 85.5% | ✗ | `src/application/message_editing_tests.rs` | 3 | Worker must identify unique value not reachable via `swipe_tests.rs` / `retry_tests.rs` / `retrigger_tests.rs`. If none, skip + document. |

**Tier 3 — borderline (<80%, small LOC or already has sibling):**

| # | Target file | Cov % | LOC | New/extended file | SP |
|---|---|---:|---:|---|---:|
| D6 | `src/adapters/driving/http/fragments/renderers/response.rs` | 74.6% | 71 | `src/adapters/driving/http/fragments/renderers/response_tests.rs` (new sibling) | 2 |
| D7 | `src/adapters/driven/llm/transport/response.rs` | 78.5% | 130 | Extend existing `response_tests.rs` | 2 |

**Axis D total:** 21 SP

**Each D-task acceptance:** tests cover ≥1 happy path + ≥1 error/failure path per public function. Construction-only tests forbidden. Tier 2 tasks must document unique value in plan doc before writing tests.

## Implementation outline

Each Phase has: owner model, story points, per-task build verification (where worker task), verification check.

### Phase 1 — Prep                              [primary · 1 SP]
- Sync plan doc with locked decisions.
- Verify working tree clean: `git status -sb`.
- Capture baseline: `python build.py --coverage`. Record test count + coverage % for Phase 4 comparison (expected: 1225 tests, 87.1% coverage).
- **Verify:** baseline numbers captured in commit/notes.

### Phase 2a — Mechanical deletions              [delegate · 3 SP]
Pure deletion, zero judgment. Single synchronous delegate subagent.

- F1 (1 SP): delete `make_recording_recorder` fn + `pub use` line in `src/test_support/mod.rs`.
- F2 (1 SP): delete 2 tautological `assert!()` lines in `src/bootstrap/text_check_factory_tests.rs:47,68`.
- F3 (1 SP): delete 6 test fns entirely (no merging — survivors cover invariants):
  - `src/application/ports/text_checker_tests.rs::method_signature_compiles`
  - `src/application/ports/llm_message_repository_tests.rs::llm_message_clone_produces_equal_value`
  - `src/application/ports/llm_message_repository_tests.rs::llm_message_debug_format_contains_field_names`
  - `src/application/ports/llm_message_repository_tests.rs::llm_message_construction_all_fields`
  - `src/application/llm_recorder_tests.rs::complete_preserves_raw_response_for_forensic_audit` (survivor covers invariant)
  - `src/application/text_check_service_tests.rs::injection_with_custom_checker` (subset of `happy_path_issues_found`)
- F6 (bundled F1-F3 SP): delete `with_storage_provider_name_is_consistent_across_calls` test fn in `tests/integration/application/wiring.rs` + dead `let _ = AgentRegistry::default();` line.
- M3 (bundled): fix `save_call_count` doc comment in `src/test_support/recording_forensics.rs` to "Number of times `save_llm_message` has completed without a configured error".

**Verify:** delegate runs `cargo nextest run --target-dir target/worker-phase2a` green before reporting done. Primary reviews diff before commit.

### Phase 2b — Judgment-required cleanup          [worker · 3 SP]
Single synchronous worker subagent. Per-site judgment.

- M1 (3 SP): remove `.clone()` calls in `src/application/text_check_service_tests.rs`. Worker verifies per-site that `checker` is not referenced after `TextCheckService::new` call. Where referenced, keep clone. Remove any `#[allow(dead_code)]` markers exposed by deletions.

**Verify:** worker runs `cargo nextest run --target-dir target/worker-phase2b` green before reporting done.

### Phase 3a — F4 + F5 strengthen LLM identity    [worker · 3 SP]
- F4 (2 SP): rewrite `provider_accessor_returns_injected_provider` with `Arc::ptr_eq`. Use exact snippet:
  ```rust
  let original: Arc<dyn LlmProvider> = Arc::new(MockBackend::new());
  let forensics = Arc::new(RecordingForensics::new());
  let recorder = LlmCallRecorder::new(original.clone(), forensics);
  assert!(Arc::ptr_eq(&original, recorder.provider()));
  ```
  Drop misleading "requires Weak or Rc" comment. Fallback: if ptr_eq fails to compile, drop the test entirely (delete) — do NOT modify port trait.
- F5 (1 SP): add `assert_eq!(recorder.unwrap().provider().name(), "DeepSeek" ‖ "OpenRouter" ‖ "Ollama")` to 3 factory path tests in `src/bootstrap/llm_factory_tests.rs`.

**Verify:** worker runs `cargo nextest run --target-dir target/worker-3a` green before reporting done.

### Phase 3b — M2 StubChecker queue rewrite       [worker · 3 SP]
- M2 (3 SP): replace `Mutex<Option<Result<Option<CheckResult>, EngineError>>>` with `Mutex<VecDeque<Result<Option<CheckResult>, EngineError>>>` queue in `src/application/text_check_service_tests.rs::StubChecker`. `check()` pops front; returns `Ok(None)` when empty. Update all test sites that construct `StubChecker::with_ok_response` / `with_error_response`.

**Verify:** worker runs `cargo nextest run --target-dir target/worker-3b` green before reporting done.

### Phase 3c — D1 server_impl http wiring test    [worker · 3 SP]
- D1 (3 SP): new file `tests/http/server_impl_wiring.rs`. Follow existing `tests/http/{connections,actions,debug}.rs` patterns (TestClient + router setup). Cover ≥1 happy + ≥1 error path. Register in `tests/http/mod.rs`.

**Verify:** worker runs `cargo nextest run --target-dir target/worker-3c` green before reporting done.

### Phase 3d — D2 transport/client sibling test  [worker · 5 SP]
⚠️ 5 SP task — primary MUST verify + run `python build.py` after (per AGENTS.md).

- D2 (5 SP): new file `src/adapters/driven/llm/transport/client_tests.rs`. Cover `call_openrouter_with_model` + `call_ollama` non-API branches (request building, response shaping). Do NOT hit real APIs — use stubs/mocks. Register in `src/adapters/driven/llm/transport/mod_tests.rs` (or `mod.rs` test block — follow existing pattern).

**Verify:** worker runs `cargo nextest run --target-dir target/worker-3d` green. Primary runs full `python build.py` after.

### Phase 3e — D3 extend prompt_presets/fragments [worker · 3 SP]
- D3 (3 SP): extend `src/adapters/driving/http/prompt_presets_fragment/fragments_tests.rs` covering the 44.2% uncovered lines in `fragments.rs` (56% → target ≥80%).

**Verify:** worker runs `cargo nextest run --target-dir target/worker-3e` green before reporting done.

### Phase 3f — D4 phases_tests.rs (contingent)     [worker · 3 SP]
- D4 (3 SP): worker first reads `src/application/action_pipeline/pipeline_tests.rs` and `phases.rs`. Identify unique value not reachable via integration (e.g., isolated phase invariants, error paths, edge cases). If unique value exists: write `src/application/action_pipeline/phases_tests.rs` with ≥3 tests targeting the gap. If no unique value: skip, document in plan doc ("D4 skipped — `pipeline_tests.rs` already covers all phase behavior at 90.2%").

**Verify:** worker runs `cargo nextest run --target-dir target/worker-3f` green (if tests written).

### Phase 3g — D5 message_editing_tests.rs (contingent) [worker · 3 SP]
- D5 (3 SP): worker first reads existing `swipe_tests.rs`, `retry_tests.rs`, `retrigger_tests.rs`. Identify unique value not reachable (e.g., `delete_last` edge cases, `switch_swipe` isolation). If unique value exists: write `src/application/message_editing_tests.rs` with ≥3 tests. If none: skip + document.

**Verify:** worker runs `cargo nextest run --target-dir target/worker-3g` green (if tests written).

### Phase 3h — D6 + D7 borderline coverage       [worker · 5 SP]
⚠️ 5 SP task — primary MUST verify + run `python build.py` after (per AGENTS.md).

- D6 (2 SP): new `src/adapters/driving/http/fragments/renderers/response_tests.rs` covering 25.4% uncovered paths in `response.rs` (75% → target ≥80%).
- D7 (2 SP): extend existing `src/adapters/driven/llm/transport/response_tests.rs` covering 21.5% uncovered paths (78.5% → target ≥80%).
- Bundle overhead (1 SP): two-file task coordination.

**Verify:** worker runs `cargo nextest run --target-dir target/worker-3h` green. Primary runs full `python build.py` after.

### Phase 4 — Validation                          [primary · 5 SP]
⚠️ 5 SP task — includes manual review of 10 test files.

1. Run `python build.py --coverage`. Confirm exit 0.
2. Parse `target/llvm-cov/coverage.json`. Confirm ≥80% overall (expect ≥87%).
3. Grep checks (all must return 0 matches):
   - `grep -rn "is_some() \|\| .*is_none()" src/ tests/` — no tautology
   - `grep -rn "make_recording_recorder" src/ tests/` — no dead helper
   - `grep -rn "let _ = AgentRegistry::default()" tests/` — no dead let-bindings
4. Test count sane: `cargo nextest run --no-fail-fast 2>&1 | grep "Summary"` ≥ 1220 (allow small variance).
5. Coverage not regressed: compare to Phase 1 baseline. ≥ baseline (expect ≥87.1%).
6. Manual review checklist for 10 new/modified test files — write 1-line verdict per file:
   1. Does it test ≥1 happy path?
   2. Does it test ≥1 error/failure path?
   3. Does each assertion verify behavior (not just types/compilation)?
   4. Are stubs/mocks minimal (no triple-nested Option)?

**Files for manual review:**
- F4: `src/application/llm_recorder_tests.rs` (modified)
- F5: `src/bootstrap/llm_factory_tests.rs` (modified)
- M2: `src/application/text_check_service_tests.rs` (modified)
- D1: `tests/http/server_impl_wiring.rs` (new)
- D2: `src/adapters/driven/llm/transport/client_tests.rs` (new)
- D3: `src/adapters/driving/http/prompt_presets_fragment/fragments_tests.rs` (extended)
- D4: `src/application/action_pipeline/phases_tests.rs` (new OR skipped)
- D5: `src/application/message_editing_tests.rs` (new OR skipped)
- D6: `src/adapters/driving/http/fragments/renderers/response_tests.rs` (new)
- D7: `src/adapters/driven/llm/transport/response_tests.rs` (extended)

**Verify:** all 6 checks green + 10 verdicts written.

### Phase 5 — Doc sync + archive                  [delegate · 3 SP]
- Sync plan doc status → Implemented.
- CHANGELOG entry.
- Regenerate `docs/README.md` index.
- Move plan to `archived/`.

**Verify:** plan in `archived/`, CHANGELOG entry exists, docs index updated.

## Story point summary

| Phase | Owner | SP | Parallel? |
|---|---|---:|---|
| 1 prep | primary | 1 | n/a |
| 2a mechanical deletes (F1+F2+F3+F6+M3) | delegate | 3 | no |
| 2b judgment cleanup (M1) | worker | 3 | no |
| 3a F4+F5 LLM identity | worker | 3 | no |
| 3b M2 StubChecker queue | worker | 3 | no |
| 3c D1 server_impl wiring | worker | 3 | no |
| 3d D2 transport/client | worker | 5 | no |
| 3e D3 extend fragments | worker | 3 | no |
| 3f D4 phases (contingent) | worker | 3 | no |
| 3g D5 message_editing (contingent) | worker | 3 | no |
| 3h D6+D7 borderline | worker | 5 | no |
| 4 validation | primary | 5 | n/a |
| 5 doc sync + archive | delegate | 3 | no |
| **Total** | | **43** | |

**Per Axis:** Axis C (test quality) = 16 SP · Axis D (coverage gaps) = 21 SP · Phases 1/4/5 (orchestration) = 9 SP.

**Per AGENTS.md:** No task ≥8 SP (all under break-up threshold). Two 5-SP worker tasks (3d, 3h) require primary verification + `python build.py` run after. Sequential Phase 3 (avoid rate limits per AGENTS.md). No top-tier subagents. No async (avoid startup flakiness).

## Locked decisions

1. **Scope:** Axis C (test quality, reviewer findings) + Axis D (real coverage gaps). No production code changes.
2. **`tests/http/fragment.rs` split:** Deferred (large, accepted per prior locked decision).
3. **`driven/llm/providers/` real-API coverage:** Out of scope (lives in `tests/llm/` `#[ignore]`).
4. **Coverage gate:** Overall ≥80% only. No per-file minimums.
5. **M3 fix:** Doc comment only (not split into two methods). Cheapest fix matches the scoped bar.
6. **F2 replacement:** Delete (do not replace). `harper_text_checker_tests.rs` already covers real behavior.
7. **D4/D5 contingent approach:** Tier 2 sibling tests must justify unique value not covered by integration. If worker finds no unique value, skip + document in plan doc.
8. **F4 `Arc::ptr_eq` approach approved.** Exact snippet provided in Phase 3a. No `provider_id()` trait method — port trait untouched. If ptr_eq fails to compile: drop the test entirely (delete) rather than modify the port trait.
9. **Subagent model:** delegate for Phase 2a (mechanical deletes) + Phase 5 (docs). worker for Phase 2b (judgment) + Phase 3 (real logic). primary for Phase 1 + Phase 4 (verification).
10. **No worker commits:** Workers write files + run build; primary agent commits in batches after verifying.
11. **Test-police expected-low list respected:** `cli.rs`, `port_utils.rs`, `bootstrap/` (incl. `load.rs`, `logging.rs`, `init_game.rs`), `main.rs`, `test_support/`, `router.rs` NOT targeted.
12. **Per-task build verification:** Every Phase 2 + Phase 3 worker task MUST run `cargo nextest run --target-dir target/worker-<phase>` before reporting done. If red, worker fixes or reports back; does not mark done with failing tests. `--target-dir` avoids lock conflicts per test-police skill.
13. **Phase 4 baseline capture:** Phase 1 captures test count + coverage % before implementing; Phase 4 compares to baseline. Pass: coverage ≥ baseline (expect ≥87.1%), test count within expected range (~1228-1236).
14. **Phase 4 manual review checklist:** Primary reads each new/modified test file + answers 4 questions (happy path? error path? behavior assertions? minimal stubs?). 1-line verdict per file.

## Assumptions

- No code-level production changes — tests + docs only. If worker discovers real bug, stops and reports per AGENTS.md plan-adherence rule (option A: stop and ask).
- Branch `hexagon-phase2`, latest commit `e0cd301`.
- Overall coverage today 87.1% (gate ≥80% passes with margin).
- 1225 tests pass currently.
- `server_impl.rs`, `transport/client.rs`, `prompt_presets/fragments.rs` have no production-side blockers to direct testing (no private items requiring `pub(crate)` exposure — to be verified by worker).

## Acceptance criteria

1. All 9 reviewer findings (F1–F6, M1–M3) addressed. Grep-verified where applicable.
2. Tier 1 tests (D1, D2, D3) added — 3 files.
3. Tier 2 tests (D4, D5) added OR skipped with documented rationale.
4. Tier 3 tests (D6, D7) added/extended — 2 files.
5. `python build.py --coverage` green. Overall ≥80% holds, ≥ baseline (87.1%).
6. No tautology patterns (`is_some() ‖ is_none()`, `assert!(true)`-equivalent) in new code.
7. Each new/modified test file passes manual review checklist (4 questions).
8. Plan doc archived, CHANGELOG entry added, docs index regenerated.
9. No production code changes.

## What already exists

- `tests/http/{connections,actions,debug,fragment,endpoints/text_check}.rs` — patterns for D1 `server_impl_wiring.rs`.
- `src/adapters/driven/llm/transport/{request_tests.rs, response_tests.rs}` — sibling test pattern for D2 `client_tests.rs`.
- `src/adapters/driving/http/prompt_presets_fragment/fragments_tests.rs` — extend (D3) rather than recreate.
- `src/application/llm_recorder_tests.rs::complete_strips_thought_tags_from_parsed_response` — survivor test covering raw-vs-parsed invariant (F3).
- `src/test_support/recording_forensics.rs::RecordingForensics` — spy for F4 identity test.
- `Arc::ptr_eq` — std lib, no new abstraction needed (F4).

## Failure modes

| Codepath | Failure mode | Handling |
|---|---|---|
| F4 `Arc::ptr_eq` on `Arc<dyn LlmProvider>` | Compile error (unlikely — API supports it) | Fallback: drop test, do not modify port trait |
| D4/D5 worker finds no unique value | No test to write | Worker skips + documents in plan doc |
| D1 `server_impl.rs` requires live HTTP server | Cannot isolate with sibling test | Handled: D1 is http wiring test under `tests/http/`, uses TestClient |
| Phase 2a delegate deletes wrong line | Tests break | Delegate runs `cargo nextest run --target-dir target/worker-phase2a` before reporting; primary reviews diff before commit |
| Worker hits rate limit mid-task | Partial state | Worker reports incomplete; primary reads files + runs build directly per AGENTS.md |
| Worker writes construction-only test | Fails Phase 4 review checklist | Primary sends back to worker for rewrite |
| 5-SP task (3d, 3h) worker skips `build.py` | Silent regression | Primary runs full `python build.py` after each 5-SP task (locked in Phase description) |

## Unresolved decisions

None. All open questions resolved during plan review:
- OQ1 (`Arc::ptr_eq` approach) — resolved (locked decision 8).
- OQ2 (D4 approach) — resolved (D1 is http wiring test under `tests/http/`).

## D4 skip rationale (Phase 3f)

D4 (sibling test for `phases.rs`) was skipped. `pipeline_tests.rs` has 13 integration tests covering all phase behavior through the pipeline (trigger continuation, save_post_trigger_error, happy path, cancel mid-run, etc.) at 90.2% coverage. Direct unit tests of `pub(super) fn phase_narrate`, `phase_post_generation`, `phase_trigger_continuation_raw`, etc. would require constructing complex state and provide no unique value beyond what the pipeline-level tests already verify.

## D5 skip rationale (Phase 3g)

D5 (sibling test for `message_editing.rs`) was skipped. The message-editing functions (delete_last, switch_swipe, etc.) are tested through HTTP fragment tests (swipe, retry, retrigger tests) and via the swipe HTTP endpoint. Direct unit tests would require constructing complex GameState + history slices, providing no unique value beyond existing coverage at 85.5%.

## Phase 4 validation results

| Check | Baseline | After | Pass |
|---|---|---|---|
| Build green (python build.py --coverage) | exit 0 | exit 0 | ✅ |
| Coverage overall | 87.1% | 88.3% | ✅ (≥87.1%) |
| Tautology grep (`is_some() \|\| is_none()`) | 0 | 0 | ✅ |
| Dead helper grep (`make_recording_recorder`) | 0 | 0 | ✅ |
| Dead let-binding grep (`AgentRegistry::default()`) | 0 | 0 | ✅ |
| Test count | 1225 | 1244 (+19) | ✅ (≥1220) |
| No LLM-skipped changes | 2 skipped | 2 skipped | ✅ |

**Manual review verdicts (10 files):**
- F4 `src/application/llm_recorder_tests.rs`: PASS — Arc::ptr_eq asserts identity; forensics saves verified; error paths exercised
- F5 `src/bootstrap/llm_factory_tests.rs`: PASS — 3 factory paths now assert provider name, not just `.is_ok()`
- M2 `src/application/text_check_service_tests.rs`: PASS — StubChecker queue simpler; no triple-nested Option
- D1 `tests/http/server_impl_wiring.rs`: PASS — 3 tests cover happy + abort + bind error
- D2 `src/adapters/driven/llm/transport/client_tests.rs`: PASS — smoke + error propagation; minimal stubs
- D3 `src/adapters/driving/http/prompt_presets_fragment/fragments_tests.rs`: PASS — covers default+active combo, None-field paths
- D4 `phases_tests.rs`: SKIP — see D4 rationale (pipeline_tests covers at 90.2%)
- D5 `message_editing_tests.rs`: SKIP — see D5 rationale (HTTP fragment tests cover at 85.5%)
- D6 `src/adapters/driving/http/fragments/renderers/response_tests.rs`: PASS — all 4 error variants + ctx_or_error
- D7 `src/adapters/driven/llm/transport/response_tests.rs`: PARTIAL — null-field fallbacks + whitespace input tests pass; see D7 reclassification below

## Post-archive addendum (2026-07-03)

Primary-agent audit after worker handoff found two issues. Both resolved below.

### D3 registration fix

Worker added 4 new tests to `fragments_tests.rs` but the file was never registered in `prompt_presets_fragment/mod.rs`. 11 pre-existing tests in the same file were already silently orphaned (prior plan's bug). Primary added `#[cfg(test)] mod fragments_tests;` to `mod.rs`.

Result: 15 tests now collected (was 0). All pass.

- Test count: 1244 → **1259** (+15, the orphaned tests now run)
- Overall coverage: 88.3% → **89.1%** (+0.8)
- `fragments.rs` coverage: 55.8% → **≥80%** (dropped off LOW list)
- D3 target met.

### D7 reclassification

Worker added 5 tests to `transport/response_tests.rs` (correctly registered, all pass). Tests target `extract_content_from_response` null-field fallbacks + `parse_chat_response` whitespace input. These functions were already mostly covered. The 28 uncovered lines (78.5% → 78.5%, unchanged) are in `handle_response`, which requires a mock `reqwest::blocking::Response` — genuinely hard to unit-test without significant test infrastructure.

D7 reclassified: the 5 added tests are valid behavior tests but do not move the coverage needle on the actual gap (`handle_response`). `handle_response` is best covered via integration test (real HTTP round-trip through the LLM transport layer) which is out of scope for this plan. Marking D7 as **partial** — tests added, coverage target not met, reclassification accepted.

## Post-archive fix-up (2026-07-03, second pass)

Comprehensive review of the post-archive state (two independent reviewer passes — Review 1 "REQUEST CHANGES", Review 2 "Approve with required cleanup") surfaced 6 follow-on items. All resolved below.

### R1 — server_impl_wiring test name lies about behavior

`run_server_serves_request_and_returns_404_for_unknown_route` sent no HTTP request and asserted no 404 — polish commit `3c5215e` only added an abort-semantics check. Resolved by renaming to `run_server_spawns_and_can_be_cancelled` (honest smoke test). The real 404 wiring path would require exposing the bound listener from `run_server_with_config`, which is out of scope; HTTP routing behavior is already covered by `tests/http/fragment.rs` via `TestAppBuilder`.

### R2 — duplicate `call_ollama_compiles_and_runs`

Polish commit `3c5215e` deleted the `call_openrouter_with_model_compiles_and_runs` twin but kept the Ollama twin, which now asserted `result.is_err()` — identical to the surviving `call_ollama_propagates_network_error` (same code path through `call_chat_completions` → `reqwest`). Resolved by deleting the duplicate. `client_tests.rs` now has 2 tests (was 3).

### P1 — orphan scout artifact at repo root

`context.md` (159 lines of agent session output) was `git add`'d to the repo root by mistake. Deleted.

### P2 — `text_check_factory_tests` no-op tests still assert nothing

Polish commit converted tautological `assert!(x.is_some() || x.is_none())` to `let _ = ...`, which removes the assertion entirely. Plan locked decision #6 said "Delete (do not replace)". Worker deviated. Resolved by deleting both `text_check_mode_spell_from_settings` and `ignored_words_from_settings_flow_to_checker` per the original locked decision. Spell-mode routing + ignored-words wiring are already exercised by `harper_text_checker_tests.rs`.

### P3 — `_tests.rs` mod-registration drift (structural guard)

Three cases this branch: `fragments_tests` (orphaned, fixed by `e08ad7c`), `renderers/response_tests` (orphaned, fixed), `transport/client_tests` (correct). Fix-by-patch will recur. Added structural guard to `scripts/check_test_structure.py`: every `*_tests.rs` file (under `src/` and `tests/`) must have a matching `mod <stem>;` declaration in its module root (`mod.rs`, sibling `.rs`, or crate root `lib.rs`/`main.rs`). First run caught 2 additional pre-existing orphans:

- `src/test_support/forensics_tests.rs` — never registered in `src/test_support/mod.rs`; entire `forensics` module (`src/test_support/forensics.rs`, 243 LOC of `ForensicsCollector` / `ForensicsLayer` tracing-subscriber infrastructure) was never compiled. Added in commit `208f46ae4` (2026-05-09) but the `pub mod forensics;` line was missed from day one. `tmp/diagnostics/` never populated. Grep-verified zero non-self callers. Resolved by **deleting both `forensics.rs` and `forensics_tests.rs` entirely** (option A2). The SQLite-backed `llm_messages` table via `LlmMessageRepository` (ADR-012) covers the same diagnostic need with a stable interface. Updated `docs/diagnostics/DEBUGGING.md`, `docs/architecture/system.md`, `docs/system/llm_processing.md`, `docs/plans/observability-and-forensics-plan.md`, `docs/reviews/docs-consistency-report.md` to reflect the deletion.
- `src/adapters/driven/text_check/types_tests.rs` — `mod.rs` comment said "types_tests removed - types tests moved to port" but the file was never deleted and the `IssueKind` Display test was never moved. Resolved by moving `test_issue_kind_display` → `issue_kind_display_formats_as_lowercase_snake_case` in `src/application/ports/text_checker_tests.rs` (now actually runs) and deleting the orphan file.

### P4 — `RecordingForensics::save_call_count` structural footgun (locked decision #5 overridden)

Plan locked decision #5 chose the doc-comment-only fix for M3 (cheapest option). The doc-comment patch ("has completed without a configured error") was honest but exposed the real bug: the counter was not incremented on the error path, so callers asserting `save_call_count` silently undercounted when `with_next_save_error` fired. User directed option A (structural fix) over option B (keep doc fix). Resolved by incrementing `save_calls` on entry unconditionally, before taking the configured error. Doc comment updated to "Number of times `save_llm_message` was called, including attempts that returned a configured error." One test (`recording_forensics_tests::next_save_error_is_returned_once_then_cleared`) updated to expect `2` after error + successful save (was `1`). Other 4 callers unaffected — their assertions all involve successful saves only.

### Optional items also addressed

- **O4** `build_test_resources` was `async` with no `await` interior. Dropped `async`.
- **O5 / P3.7** `AGENTS.md` typo `Mininmax` → `Minimax`, added missing trailing newline.
- **O6** Plan addendum D7 count reconciled in the doc itself (4 tests added, not 5 — `3d87871` diff confirms 4: `whitespace_only`, `null+reasoning`, `null+reasoning_content`, `all-null`).

### Items intentionally NOT addressed

- **O1 (D2 non-hermic, real API hit)** — `call_openrouter_with_model("fake-api-key", ...)` hardcodes `https://openrouter.ai/api/v1/chat/completions`. Plan locked "Do NOT hit real APIs — use stubs/mocks", worker deviated. Proper fix = wiremock/mockito at the `reqwest::blocking` layer. Larger scope — flagged as follow-up plan, not this fix-up.
- **O3 (StubChecker 3 Mutexes → 1)** — code correct today. Optional consolidation; left for a future cleanup pass.

### Final state

- Build green: `python build.py` exit 0.
- Test count: **1255** (was 1259 → -4: −2 `text_check_factory_tests`, −1 `client_tests` dup, −2 `forensics_tests` never-ran deleted, +1 `issue_kind_display` newly-registered-and-run).
- Overall coverage: **89.2%** (was 89.1% → +0.1; small gain from `forensics.rs` removal + `issue_kind_display` running for the first time).
- `python scripts/check_test_structure.py` passes — zero unregistered `*_tests.rs` files in `src/` or `tests/`.

### Post-archive fix-up (2026-07-03, third pass) — D2 removed

On re-examination, the two surviving `client_tests.rs` tests (`call_ollama_propagates_network_error` and `call_openrouter_with_model_propagates_network_error`) only verified that `reqwest::blocking::Client` returns `Err` on a failed connection. That is `reqwest`'s contract, not Chronicler's. They asserted zero Chronicler-specific behavior — request building, payload shaping, response parsing, error mapping all unexercised.

The `call_openrouter_with_model_propagates_network_error` test was additionally non-hermetic: `call_openrouter_with_model` hardcodes `https://openrouter.ai/api/v1`, so the test hit real DNS + TLS against OpenRouter's endpoint with a fake API key on every run — violating AGENTS.md's `no-internet` Docker network convention + the plan's locked decision "Do NOT hit real APIs — use stubs/mocks". The Ollama twin used `127.0.0.1:1` (hermic) but verified the same trivial reqwest behavior.

Resolution: deleted both tests and removed `client_tests.rs` entirely (plus `mod client_tests;` declaration in `transport/mod.rs`). D2 status changed from "complete" to **removed**.

The original D2 coverage gap (`transport/client.rs` at 40.9%) was not actually closed by these tests — they covered only the `Err(...)` construction arms, which are already covered via other tests in `transport/{request,response}_tests.rs` (overall coverage held at 89.2% after deletion). The real D2 intent (cover request building, response shaping, non-API branches) requires either HTTP-layer mocking (wiremock/mockito — deferred) or a refactor exposing the pure helpers for direct testing. Marked as a real follow-up: bring `client.rs` to ≥80% via hermic tests that assert actual Chronicler behavior.

Final state: 1253 tests pass, 2 LLM skipped, 89.2% coverage. The `client.rs` 40.9% gap remains open — tracked here, not in any active plan.
