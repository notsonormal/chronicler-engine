# Make the test standards docs match the code

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Which claims in the test standards docs are false today, and which parts only copy what the code already states?

## Context

- Scope: `docs/diataxis/reference/coding_standards/{testing,integration_test_standards,unit_test_standards}.md` and the non-generated parts of `tests/AGENTS.md`.
- Drift confirmed in the audit session [known]:
  - `SqliteTestAppBuilder` does not exist, yet integration Pattern 1 and Cross-cutting 6 are built on it. The guardrail message at `tests/infrastructure/guardrails/structure.rs:214` tells people to use it. Find the builder that tests really use, and fix both places.
  - `tests/helpers/sqlite_test_app_builder.rs`, `tests/infrastructure/invariant_contract.rs`, and `src/application/narrative_prompt/` do not exist. The last one is probably `src/application/prompting/`.
  - The integration Pattern 6 example of `Args` lacks `host` (`src/utils/cli.rs:35`).
  - `testing.md` says "nine-pattern", but `unit_test_standards.md` has eight patterns.
- Unverified claims from a scout report: the file counts ("~10 files" is really 15, "14 files" is really 27). Check them before you change them. [docs_drift.md](../assets/test-audit/docs_drift.md) has a known error: it treats `tests/storage/` as untiered, but `tests/STRATEGY.md:15` gives it a row.
- `integration_test_standards.md` is 352 lines. Most of it copies fixture signatures, helper names, and counts that the code states. Those copies are what drifted. Per the writing-for-agents skill ("Pruning"), keep what the code cannot say, such as why the port lock lives in `/tmp` and why mocks are injected over HTTP. Replace the rest with pointers.
- This ticket does not touch `tests/STRATEGY.md`. That is [Rewrite the test strategy around one checkable tier-1 rule](32-rewrite-test-strategy.md).

## Done when

- Every path, type, and function named in the scoped docs exists.
- Every count is correct or removed.
- The `structure.rs:214` message names a real builder.
- `python build.py` is green. The user reviews the diff. Commit after approval.

## Answer

Every path, type, function and count in the scoped docs was checked against the code before and after the change. The ticket's drift list was right, and there was more.

**False claims fixed.**
- `integration_test_standards.md` — phantom `SqliteTestAppBuilder` and its methods (`.backends()`, `.mock_backend()`, `.separate_backends()`, `.pipeline_fn()`) → the real `TestAppBuilder` (`src/test_support/test_app_builder.rs`) with `.storage()`, `.pipeline()`, `make_test_pipeline_with_backends` / `make_test_pipeline_with_mock_quantifier`. Its default backend is `InMemory`, not SQLite; Pattern 1 is now "In-memory app + builder handoff". Also: `tests/helpers/sqlite_test_app_builder.rs`, `tests/infrastructure/invariant_contract.rs` and `tests/http/connections.rs` do not exist; `narrative_prompt/` → `prompting/`; Pattern 6 `Args` now lists all five fields incl. `host`; Pattern 2 helpers → real `wait_for_element_children` / `count_log_entries`; `SERVER_MANAGED` is an `AtomicBool` (PIDs live in `PORT_PIDS`); `TestServer::start` is private → `new` / `new_with_mock`; `get_available_port` takes explicit bounds, `get_config_port` reads `tests/test_config.json`; the CC7 factory/closure API does not exist, and the quantifier comes from the `AgentRegistry` argument; the test-binary list now matches the seven `Cargo.toml` `[[test]]` targets.
- `unit_test_standards.md` — `TestPersona::default()` → `standard()`; `LlmProvider` has `model()`, not `model_name()`; no `persistence_gate` → `app.message_service.load_or_fresh()`; no `app.process_action` / `execute_action` on `AppState`; corrected `src/application/pipeline/action_pipeline/{core,retry}_tests.rs` and `src/application/ports/text_checker_tests.rs` paths; `world_seeded_default` → `seed_persona`.
- `testing.md` — "nine-pattern" → "eight-pattern".
- `tests/AGENTS.md` non-generated parts: verified clean, no change.

**Pruned to pointers.** `integration_test_standards.md` went from 352 to 250 lines. Copied code blocks are replaced by exemplar pointers (`tests/http/retrigger.rs`, `tests/http/games_create.rs`, `tests/storage/*.rs`, `tests/bootstrap/run_branches.rs`, `tests/infrastructure/guardrails/*_tests.rs`). **Kept** what the code cannot say: why the port lock lives in `/tmp`, why mocks are injected over HTTP, `SettingsTestGuard`'s race and poisoning recovery (and that a settings mutation without it is a bug), `TestOverride::internal` vs `::config`, `TestServer` teardown, `capture_failure_state`'s panic contract, the `127.0.0.1:0` port opt-out, the harness decision rule, no sync polling (nextest retries instead), and not using Service-direct for `process_action`.

**Counts.** The scout's counts were right (15 HTTP one-shot files, 27 `TestOverride` test files). Both counts were removed, not updated, since they drift.

**Guardrail message.** `tests/infrastructure/guardrails/structure.rs` ~214 now reads "Integration tests must not call make_test_context(; use TestAppBuilder instead." `guardrails.md` regenerated.

**Gate:** worktree `nextest: 1639 passed, 0 failed, 2 skipped`, browser 24 passed (`build_20260930_200414.log`). After applying on main (`2ee998aa`): guardrails 142 passed, validate-docs OK.

**Code review** (`/code-review`, verdict ISSUES → fixed): the first rewrite introduced five false claims (SQLite default backend, `infrastructure` binary, `get_available_port` config read, CC7 quantifier wiring, "SQLite" in the guardrail message) and dropped five pieces of advice. All fixed and restored as listed above. Implementer side findings went to [Follow up on small issues found during review](36-follow-up-small-review-issues.md).

