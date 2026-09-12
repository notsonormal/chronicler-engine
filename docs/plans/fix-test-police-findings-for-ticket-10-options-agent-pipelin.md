# Fix Test-Police Findings for Ticket 10 (Options Agent + Pipeline)

## Summary

Close the nine gaps raised against the ticket-10 working tree (eight review findings + the cancellation branch surfaced by the plan review): six unit-tier coverage gaps (F1 registry, F2 preset chain, F4 always-on no-agent, F5 event-retry rewrite, F6 retry-recovery/attempt count, F9 options-refresh cancellation), one spec drift (F3), one HTTP contract gap (F7), and one flaky browser test (F8) plus its stale helper catalog. No production-code changes. Land as three commits: (1) unit coverage, (2) spec + HTTP, (3) browser flake + catalog.

## Key Changes

- Assert options-agent registration from `AgentConfig::defaults()` (F1). Enabled-skip for the options arm is not re-tested — the generic `!config.enabled` path is already covered.
- Test `Storage::active_options_preset_id` (game-first + settings fallback) and the agent's preset-loading chain via saved LLM-message prompts (F2).
- Add unit tests for always-on-with-no-agent (F4), event-retry options rewrite (F5), retry-recovery + attempt-count pin (F6), and options-refresh cancellation (F9).
- Retitle spec scenario 1.12 to "engine commands"; add a `/options` leg to its covering HTTP test after the `/guide` leg; pin `Action::Options.is_steering() == false` (F3).
- Add `current_options` to the `/debug/state` required-fields list (F7).
- De-flake `test_delete_removes_message` by reusing `wait_for_element_children` + `wait_until_hidden`, and delete the orphaned `wait_for_log_entries_below` (F8); sync the stale `WAIT_HELPERS.md` catalog.

## Implementation

### Phase 1: Unit-tier coverage

- [ ] #### Task 1.1: F1 — Assert options registration in registry tests (1 SP)
  - In `src/application/agents/registry_tests.rs`, extend `test_registry_from_configs_empty_uses_defaults` to assert `agents_for_phase(ExecutionPhase::OptionsGeneration)` yields exactly one agent named `"options"`.
- [ ] #### Task 1.2: F2 — Test the preset-resolution chain (3 SP)
  - [ ] ##### SubTask 1.2.1: Storage accessor tests in `src/adapters/driven/storage/games_tests.rs` (1 SP)
    - Game-first: seed a game with a per-game `active_options_prompt_preset_id`; assert the accessor returns it.
    - Settings fallback: fresh `Storage::new_in_memory()` with no current game; assert fallback to `settings.active_options_prompt_preset_id`.
  - [ ] ##### SubTask 1.2.2: Agent preset-chain test in `src/application/agents/options/agent_tests.rs` (3 SP)
    - Build via `OptionsAgent::from_config_with_storage(config, make_test_recorder_with_storage(mock, storage), Some(storage), settings)`; seed an options preset with a distinctive `{{option_count}}`/`{{user}}` marker; execute with a tagged mock response.
    - Assert via the saved LLM message (`save_llm_message` seam, `llm_recorder.rs:34`): `agent_name == "options"` and the recorded system prompt carries the rendered marker.
- [ ] #### Task 1.3: F4 — Always-on rewrite with no agent registered (1 SP)
  - In `options_tests.rs`: `make_app(None)`, toggle on, seed stale options, run one narration turn; assert set cleared AND system message `Options agent is not available` present (covers `options.rs:179-182`).
- [ ] #### Task 1.4: F5 — Event-retry options rewrite (3 SP)
  - In `retry_tests.rs`, mirroring `test_retry_event_continuation_happy_path` + the `options_tests.rs::set_prior_options` pattern:
    - Toggle on + options agent present: assert set replaced with the tagged options after `retry_event_continuation`.
    - Toggle off: assert set cleared.
- [ ] #### Task 1.5: F6 — Retry-recovery and attempt-count pin (1 SP)
  - `with_prompt_responses(vec![unparseable, tagged])` → `execute` succeeds; assert `call_index == 2`.
  - Tighten `test_execute_unparseable_response_errors_after_attempts` to assert `call_index == MAX_OPTIONS_ATTEMPTS`.
- [ ] #### Task 1.6: F9 — Options-refresh cancellation (3 SP)
  - In `options_tests.rs`, mirror the cancellation setup of `test_retry_event_continuation_cancels_before_llm` (`retry_tests.rs:367`): arm the cancellation token, drive `execute_options_refresh`, assert the `Err(PhaseError::Cancelled)` → `handle_cancellation` path (`options.rs:133-139`) — no set rewrite, no failure system message, state restored.

### Phase 2: Spec + HTTP tier

- [ ] #### Task 2.1: F3 — Fix spec 1.12 drift and pin predicate semantics (1 SP)
  - Retitle `docs/specs/actions.md` scenario 1.12: "steering commands" → "engine commands"; note `/options` also bypasses the player-input text check. Keep the scenario id (validator count stays 116).
  - Extend `test_slash_command_bypasses_text_check_http` (`tests/http/actions.rs:769`): after the existing `/guide` leg (whose `wait_idle` creates scene history, avoiding the ticket-10 known 500-on-empty-history gap), POST `/action/check command="/options"`; assert `HX-Retarget` header set and body lacks `text-check-preview`. Do not assert generation outcome — this fixture carries no options agent (unit tests own dispatch; ticket 12 owns full HTTP options scenarios).
  - Add `assert!(!Action::Options.is_steering())` to `test_is_steering_false_for_free_action` in `action_tests.rs`.
- [ ] #### Task 2.2: F7 — Debug-state required-fields contract (1 SP)
  - Add `"current_options"` to `required_fields` in `tests/http/requires_migration/debug.rs:88-102` (+ `is_array` type assertion).

### Phase 3: Browser flake fix + catalog

- [ ] #### Task 3.1: F8 — De-flake `test_delete_removes_message` (3 SP)
  - After the `initial < 2` branch's `send_action` + `wait_for_status_ready`: wait `rendered = wait_for_element_children(&page, "#story-log .log-entry", (initial + 1) as u32).await` and `assert!(rendered >= initial + 1)` (the helper captures state but returns on timeout — the assert is load-bearing).
  - Rework the deletion: capture the last `.log-entry`'s `data-id`, click `.delete-btn`, then `wait_until_hidden(&page, &format!(".log-entry[data-id='{deleted_id}']"), Duration::from_secs(10))`. Drop the count-vs-baseline comparison.
  - Delete `wait_for_log_entries_below` from `tests/test_utils/browser.rs` (sole caller is `behaviour.rs:143`; it is not in the WAIT_HELPERS.md catalog).
- [ ] #### Task 3.2: Sync WAIT_HELPERS.md catalog (1 SP)
  - Rewrite `.agents/skills/test-police/WAIT_HELPERS.md` to match the actual API in `tests/test_utils/wait.rs` and `tests/test_utils/browser.rs`; remove the four nonexistent helpers (`wait_for_log_entries`, `wait_for_element_text`, `wait_for_story_log_change`, `wait_for_more_messages`); document the `wait_for_element_children("#story-log .log-entry", …)` growth idiom and `wait_until_hidden` for detachment.

### Phase 4: Full gate

- [ ] #### Task 4.1: Final validation (1 SP)
  - `python build.py --coverage` (1200s): suite green, overall ≥80%, `options/agent.rs` ≥80% band, no FLAKY marker for the delete test, guardrails/doc gates green (new helper deletion and doc edits pass style/structure rules).

## Test Plan

- Targeted per task: `python build.py nextest <pattern>` (`registry_`, `active_options_preset_id`, `options`, `retry_event_continuation`, `action_check`, `debug`, `test_delete_removes_message`).
- F6 regression signal: attempt-count assertions fail if `MAX_OPTIONS_ATTEMPTS` drifts from 2.
- Browser verification: `python build.py nextest test_delete_removes_message` green, then full suite via Phase 4.
- Final: `python build.py --coverage`; `parse_coverage.py --threshold 80` shows `options/agent.rs` ≥80%; `validate_feature_spec.py` stays 116/116.

## Per Task/Sub Task Validation Steps

- 1.1: `python build.py nextest registry_` green.
- 1.2: `python build.py nextest options` and `active_options_preset_id` green.
- 1.3: `python build.py nextest always_on` green.
- 1.4: `python build.py nextest retry_event_continuation` green.
- 1.5: `python build.py nextest options` green.
- 1.6: `python build.py nextest options` green; cancel test asserts restored state, not just no-panic.
- 2.1: `python build.py nextest action_check` green; `python scripts/validate_feature_spec.py` — 116 covered, no gaps/orphans.
- 2.2: `python build.py nextest debug` green.
- 3.1: `python build.py nextest test_delete_removes_message` green; no `wait_for_log_entries_below` references remain (`rg`).
- 3.2: validate-docs passes; catalog matches `rg 'pub async fn' tests/test_utils/*.rs` output.
- 4.1: Full gate green.

## Assumptions

- Ticket 12 still owns the full options spec + HTTP E2E suite; this plan closes only the 1.12 drift.
- Scenario 1.12 keeps its id; the retitle does not change validator counts.
- In-memory `Storage` with no seeded game exercises the settings-fallback branch (`games.rs:243-249`).
- The LLM-forensics save seam records the system prompt (`llm_recorder.rs:34`), making the preset override observable via `make_test_recorder_with_storage`.
- F8's root cause (2s story-log poll lagging the 5s status poll) is accepted as confirmed; the fix is verified by passing runs, not forced-failure reproduction.
- The cancel test can arm the pipeline cancellation seam the same way `test_retry_event_continuation_cancels_before_llm` does (assumed from the shared `PipelineRun`/gate machinery).
