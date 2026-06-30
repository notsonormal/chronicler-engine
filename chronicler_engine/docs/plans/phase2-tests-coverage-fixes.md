# Plan: Phase 2 Tests + Coverage Fixes

**Date:** 2026-06-30
**Status:** Draft (not yet started)
**Scope:** `chronicler_engine/`
**Branch target:** `hexagon-phase2` fix-up commits or new `hexagon-phase2-tests-fixes` branch off `hexagon-phase2`

## Context

External thermonuclear review of branch `hexagon-phase2` flagged that Phase 2 deliverables lack unit tests and that the integration test folder/file structure no longer matches the post-Phase-1+2 `src/` tree. In addition, the project's own AGENTS.md §"Tests as Documentation" contract — "If you don't understand how a component works, read its tests in `tests/` before reading the source code" — is broken for every new module the refactor introduced.

This plan covers **only** test coverage + test reorganization. Code-level fixes for the review findings live in the sibling plan: `phase2-thermonuclear-review-fixes.md`. The two plans are independent — either can be implemented first, but they should be sequenced so that the code-level review fixes land before (or alongside) any new tests, so the tests cover the final shape rather than the broken shape.

## Related

- Sibling plan: `docs/plans/phase2-thermonuclear-review-fixes.md` (code-level review fixes)
- Prior plan: `docs/plans/hexagonal-reorganization-plan.md` (Phase 2 complete)
- ADR-027: `docs/adr/adr-027-hexagonal-architecture-migration.md`
- AGENTS.md §"THE TEST-FIRST PHILOSOPHY" (line 160) — "Tests as Documentation" contract
- `docs/architecture/system.md` §"11. Test Binaries (`tests/`)" (line 290) — current test binary catalog
- `scripts/check_test_structure.py` — enforces "no inline `#[cfg(test)] mod` blocks" only; no naming/mirror enforcement

## Goals

Two independent problem axes, both need to be solved:

### Axis A — Add unit + integration tests for Phase 2 deliverables

Close the coverage gap for the 6 new production files introduced by Phase 2. Reviewer flagged that the orchestrators the whole refactor was built around (`LlmCallRecorder`, `TextCheckService`) have zero direct test coverage — they appear in test files only as scaffolding for test doubles, never as subjects under test.

### Axis B — Reorganize `tests/` to mirror `src/`

The integration test folder structure and file names have drifted from the post-Phase-1+2 `src/` tree. Reviewer + user flag: "completely misaligned with the production infrastructure — hard to know what the integration tests are covering or not covering now." Without a documented + enforced convention, drift will recur after every phase.

## Success criteria

### Axis A success criteria

- Every new Phase 2 production file has a sibling `*_tests.rs` (per repo convention) OR a corresponding `tests/integration/...` mirror file, covering:
  - `src/application/llm_recorder.rs`
  - `src/application/text_check_service.rs`
  - `src/bootstrap/llm_factory.rs`
  - `src/bootstrap/text_check_factory.rs`
  - `src/application/ports/llm_message_repository.rs`
  - `src/application/ports/text_checker.rs`
- At minimum, tests verify:
  - **Port traits:** trait enforces the expected method signatures; the impl contract (e.g. `Storage::save_llm_message` round-trip through `list_latest_llm_messages`) is exercised.
  - **Factories (`llm_factory`, `text_check_factory`):** the prod wiring path (`get_llm_recorder_for`, `create_text_check_service`) is exercised at least once with a real-ish `Connection` + `Storage` — not only the `with_mock_quantifier` test bypass.
  - **Orchestrators (`LlmCallRecorder`, `TextCheckService`):** the orchestration logic itself is the subject under test — provider gets called, forensics get saved, sanitization runs, etc. Currently only the test-double wiring duplicates the orchestrator; nobody tests what the orchestrator actually does.
- `python build.py --coverage` shows coverage improvement on the 6 flagged files:
  - `src/application/ports/llm_provider.rs` (was 32.7%) — target ≥70%
  - `src/application/ports/text_checker.rs` (was 44.4%) — target ≥70%
  - `src/bootstrap/text_check_factory.rs` (was 75%) — target ≥85%
  - `src/bootstrap/llm_factory.rs` (was 78.6%) — target ≥85%
  - `src/application/llm_recorder.rs` (untested) — target ≥80%
  - `src/application/text_check_service.rs` (untested) — target ≥80%
- Overall coverage holds ≥80% (project threshold).

### Axis B success criteria

- `tests/` folder structure mirrors `src/` folder structure — every `src/<tier>/<module>/` has a `tests/<tier>/<module>/` counterpart (or a documented reason why it doesn't).
- Flat-file integration tests (`tests/integration/application_service.rs`, `game_service.rs`, `lifecycle.rs`, `llm_client.rs`, `mod.rs`) are either:
  - Reorganized into mirrored subdirs matching `src/application/`, OR
  - Documented as an intentional exception with rationale (e.g., cross-cutting tests that legitimately span multiple src dirs).
- `tests/integration/pipeline/` → `tests/integration/action_pipeline/` (rename to match `src/application/action_pipeline/`).
- `tests/http/` → `tests/integration/driving/http/` OR `tests/driving/http/` (decide: integration-only or keep separate binary) — match the `src/adapters/driving/http/` path.
- AGENTS.md §"THE TEST-FIRST PHILOSOPHY" extended with a "Test Mirror Convention" subsection documenting:
  - Unit tests: `<src_dir>/<file>_tests.rs` sibling pattern (already enforced by `check_test_structure.py`).
  - Integration tests: `tests/<src_tier_path_mirror>/...` — same subpath as `src/` module being tested.
  - Update `docs/architecture/system.md` §11 to reflect the actual mirrored structure post-reorganization.
- A new guardrail (script or doc-only) lists the canonical `src/` ↔ `tests/` mapping; if a future change adds an unmirrored test file, `python build.py` warns (or fails — decide).
- All tests green; `python build.py` green.

---

## Problem inventory (verified)

### Axis A — Coverage gaps

Verified across `src/application/`, `src/bootstrap/`, `src/application/ports/`:

| src file (NEW in Phase 2) | unit tests (`*_tests.rs`) | integration tests | Coverage (pre-fix) | Role |
|---|---|---|---|---|
| `src/application/llm_recorder.rs` | ❌ none | ❌ only as scaffolding | 0% direct | Orchestrator — THE central Phase 2 deliverable |
| `src/application/text_check_service.rs` | ❌ none | ❌ only as scaffolding | 0% direct | Orchestrator — the Phase 2.3 deliverable |
| `src/bootstrap/llm_factory.rs` | ❌ none | ❌ `get_llm_recorder_for` has zero callers in tests | 78.6% (indirect) | Prod factory wiring |
| `src/bootstrap/text_check_factory.rs` | ❌ none | ⚠️ indirectly via `test_app_builder.rs` + 2 fragment tests | 75% | Prod factory wiring |
| `src/application/ports/llm_message_repository.rs` | ❌ none | ⚠️ `tests/integration/storage/llm_message_storage.rs` tests Storage's impl | n/a | Port trait + DTO + builder (builder to be deleted per review Fix 11) |
| `src/application/ports/text_checker.rs` | ❌ none | ❌ `NoopTextChecker` in `poison_recovery.rs` is the only impl | 44.4% | Port trait |

For context — Phase 2 files that DO have coverage (mostly via the sibling `*_tests.rs` convention from Phase 1.6):
- `src/application/ports/llm_provider.rs` has `src/application/ports/llm_provider_tests.rs` (sibling) — but only 32.7% covered, and the new orchestrator-side methods aren't tested.

### How the orchestrators appear in tests today (verified by grep)

`LlmCallRecorder` is mentioned in test files:
- `tests/integration/flow/arrival_persistence.rs:14-15` — used as scaffolding inside `make_test_recorder` helper
- `tests/integration/flow/retry_event.rs:4,24,43` — same pattern
- `tests/infrastructure/invariant_contract.rs:34,56,189` — same pattern
- `tests/test_utils/mod.rs` — defines `make_test_recorder` helper
- `tests/poison_recovery.rs` — uses it

In every case, the recorder is constructed with a `NoopForensics` to wrap a `MockBackend` for use as input to `GameService::with_mock_quantifier`. **Nobody tests what the recorder itself does** — that it calls the provider, sanitizes, persists to `LlmMessageRepository`, returns the result, errors propagate, etc.

`TextCheckService` follows the same pattern — only `NoopTextChecker` (in `tests/poison_recovery.rs`) as scaffolding; orchestration logic untested.

### Axis B — Structure misalignment inventory

Verified via `find src -type d` + `find tests -type d`:

#### `src/` tree (post-Phase-1+2)

```
src/adapters/driven/llm/{providers,transport}
src/adapters/driven/storage/{backend,mappers,models}
src/adapters/driven/text_check
src/adapters/driving/http/{fragments/{misc,renderers},games_fragment,prompt_presets_fragment,settings_fragment,worlds_fragment}
src/application/{action_pipeline,agents/{quantifier,narrative_prompt,ports}
src/bootstrap
src/domain/{engine,model/state}
src/test_support
```

#### `tests/` tree (current)

```
tests/browser
tests/helpers
tests/http/endpoints
tests/infrastructure/guardrails
tests/integration/{flow,model,pipeline,storage}
tests/llm
tests/test_utils
```

#### Mapping (src → tests)

| src location | tests location | Status |
|---|---|---|
| `src/adapters/driven/storage/` | `tests/integration/storage/` | ✅ Mirrors (good citizen) |
| `src/domain/model/` | `tests/integration/model/` | ✅ Mirrors |
| `src/application/action_pipeline/` | `tests/integration/pipeline/` | ❌ **renamed** (drops `action_`) |
| `src/adapters/driving/http/` | `tests/http/` | ❌ **driving tier dropped from path; dir renamed** |
| `src/application/application_service.rs` | `tests/integration/application_service.rs` | ⚠️ flat file, no subdir |
| `src/application/game_service.rs` | `tests/integration/game_service.rs` | ⚠️ flat file, no subdir |
| `src/application/lifecycle` (implicit) | `tests/integration/lifecycle.rs` | ⚠️ flat file, no src counterpart |
| `src/application/llm_recorder.rs` (NEW) | — | ❌ **no test** |
| `src/application/text_check_service.rs` (NEW) | — | ❌ **no test** |
| `src/application/ports/llm_message_repository.rs` (NEW) | `tests/integration/storage/llm_message_storage.rs` | ⚠️ tests exist but file name doesn't reflect port coverage |
| `src/application/ports/text_checker.rs` (NEW) | — | ❌ **no test** |
| `src/application/ports/llm_provider.rs` | — | ❌ **no test directory mirror** (sibling `*_tests.rs` exists at `src/.../llm_provider_tests.rs`) |
| `src/application/agents/` | — | ❌ **no `tests/integration/agents/`** |
| `src/application/agents/quantifier/` | — | ❌ no test dir (only `src/.../agent_tests.rs` sibling) |
| `src/application/narrative_prompt/` | — | ❌ no `tests/integration/narrative_prompt/` |
| `src/adapters/driven/llm/` | `tests/integration/llm_client.rs` | ❌ **flat file, doesn't mirror the subdir** |
| `src/adapters/driven/text_check/` | — | ❌ **no test dir** |
| `src/bootstrap/` | — | ⚠️ no `tests/integration/bootstrap/` (sibling `*_tests.rs` files inside `src/bootstrap/` only) |
| `src/domain/engine/` | — | ❌ **no test** |

### Existing guardrail coverage

`scripts/check_test_structure.py` enforces only one rule: no inline `#[cfg(test)] mod` blocks in `src/` files. It does NOT enforce:
- Naming for integration test files
- Mirror mapping between `src/` and `tests/`
- Coverage minimums per file

`build.py --coverage` enforces ≥80% overall (currently 86.3%) but does NOT enforce per-file minimums — that's why several Phase 2 files ship with <50% coverage without failing the build.

### Doc claims vs reality

- `docs/architecture/system.md:296-302` catalogs test binaries by count only, not by src-mirror mapping.
- `AGENTS.md:160` asserts "Tests as Documentation" contract broken by Phase 2.
- ADR-027 does not mention test structure.

---

## Implementation outline

### Phase A.1 — Add unit tests for orchestrators (highest leverage)

**Why first:** The orchestrators are the central Phase 2 deliverable. Without these tests, the refactor is semantically unverified — all current tests just use the orchestrators as plumbing.

#### A.1.1 — `src/application/llm_recorder_tests.rs` (NEW)

Unit tests for `LlmCallRecorder`. Per sibling-file convention. Cover at minimum:

1. **Happy path:** `complete()` calls provider, returns `LlmCallResult`, persists to `LlmMessageRepository` via `save_llm_message`, returns sanitized text.
2. **Sanitization step:** verify `sanitize_llm_output` actually ran (e.g., inject a `<thought>...</thought>` tag in mock response, assert it's stripped from the saved message's `parsed_response`).
3. **Forensics persistence:** inject a recording `LlmMessageRepository` mock (not `NoopForensics` — actually count `save_llm_message` calls) + assert the saved `LlmMessage` has correct `agent_name`, `backend_name`, `model_name`, `system_prompt`, `user_prompt`, `raw_request_json`, `raw_response_json`, `parsed_response`, `created_at`, `id: 0`.
4. **Forensic safety — sanitize doesn't drop raw response:** the current `LlmCallRecorder::complete` (llm_recorder.rs:35-42) sets `message.parsed_response = sanitized_text` while leaving `raw_response_json` UNMODIFIED from the chat result. This is the ADR-012 audit trail invariant — raw response preserved as forensic evidence, sanitized text in `parsed_response`. Test must assert BOTH:
   - `saved_message.parsed_response == sanitized_version_of(text)`
   - `saved_message.raw_response_json == original_raw_json_from_provider` (NOT sanitized, NOT truncated)
5. **Provider error propagation:** provider returns `Err(...)` — recorder returns `Err(...)`, no forensics write happens.
6. **Forensics error propagation:** provider succeeds, `save_llm_message` errors — recorder returns `Err(...)`.
7. **Provider injection:** verify `provider()` accessor returns the injected `Arc<dyn LlmProvider>` (used by `GameService::with_mock_quantifier` at game_service.rs:96 to extract recorder's provider for `QuantifierAgent::with_backend`).
8. **Send + Sync static assert:** add a compile-time assertion `const _: fn() = || { fn assert<T: Send + Sync>(); assert::<LlmCallRecorder>(); };`. LlmCallRecorder holds only `Arc<dyn LlmProvider>` + `Arc<dyn LlmMessageRepository>` — both immutable, no shared mutable state — so concurrent-call testing at runtime is a no-op. The static assert alone is sufficient.

**Note on shared test doubles:** the `NoopForensics` type the sibling plan extracts to `test_support/` is a NOOP (does nothing, returns `Ok(())` / empty vec). It is NOT suitable for tests #3, #4, #5, #6 above which need a RECORDING repo (counts calls, captures last message, returns configurable errors). Plan should add a separate `RecordingLlmMessageRepository` (or `SpyForensics`) to `test_support/` alongside the Noop one. Or unit tests can define a small local spy inline.

#### A.1.2 — `src/application/text_check_service_tests.rs` (NEW)

Unit tests for `TextCheckService`. Per sibling-file convention. Cover:

1. **Happy path — `check_player_input` routes to the injected `TextChecker`**, returns `Ok(Some(CheckResult))` when the checker returns issues.
2. **Mode handling — `TextCheckMode::Off` returns `Ok(None)`** without calling the checker.
3. **Mode handling — `TextCheckMode::Preview` / `Editor` routing** — verify the checker gets called with the right `mode` + `ignored_words`.
4. **Empty input — returns `Ok(None)`** or empty `CheckResult`, depending on contract (verify against actual implementation).
5. **Checker error propagation — `Err(...)` from `TextChecker::check` propagates.**
6. **Checker injection — `with_checker` / constructor takes `Arc<dyn TextChecker>`.**

**Coverage justification note:** The orchestration logic IS exercised indirectly via 2 production HTTP fragment handlers (`src/adapters/driving/http/fragments/actions.rs:97` and `src/adapters/driving/http/fragments/misc/text_check.rs:38`) — BUT zero HTTP fragment handler tests currently call `check_player_input` or `TextCheckService` (verified by grep across `src/adapters/driving/http/fragments/**/*_tests.rs`). So the orchestration is effectively untested. This unit test file is the right place to cover it directly rather than relying on fragment handler tests to start doing so.

### Phase A.2 — Add unit tests for factories

#### A.2.1 — `src/bootstrap/llm_factory_tests.rs` (NEW)

Tests for `get_llm_recorder_for`. Per sibling-file convention. Cover:

1. **Mock backend path — `Connection { provider: Mock, .. }` returns a recorder wrapping `MockBackend`** whose `provider().name()` matches expected.
2. **OpenRouter/DeepSeek/Ollama paths — `from_connection` succeeds**, returns recorder. Use test `Connection` fixtures (no real API keys).
3. **Error path — provider construction fails (e.g., missing required field on `Connection`)** — error propagates as `Err(EngineError)`. This is the failure mode currently silently swallowed by Fix 2 in the sibling plan.
4. **Storage wiring — the LlmMessageRepository impl passed in is the one the recorder uses** (inject a recording repo, verify `save_llm_message` reaches it after `complete()` call).

#### A.2.2 — `src/bootstrap/text_check_factory_tests.rs` (NEW)

Tests for `create_text_check_service`. Per sibling-file convention. Cover:

1. **Production path — `create_text_check_service(&settings)` returns a `TextCheckService` wrapping a real `HarperTextChecker`.**
2. **Settings propagation — `TextCheckMode`, `ignored_words`, dictionaries flow from `AppSettings` into the `TextChecker`.**
3. **Harper init failure — error propagation (if any).**

### Phase A.3 — Add unit tests for port traits

#### A.3.1 — `src/application/ports/llm_message_repository_tests.rs` (NEW)

Tests for the `LlmMessageRepository` port trait itself + the `LlmMessage` DTO:

1. **Port trait contract — `dyn LlmMessageRepository` dispatch works correctly between `Storage` and a test double.** This is the unit-test concern: the trait-as-interface. Do NOT duplicate the `Storage` impl round-trip here — that's already covered at `tests/integration/storage/llm_message_storage.rs`.
2. **Trait signature enforcement — compile-time guarantee just from `impl LlmMessageRepository for X`.**

**Scope guard:** Do NOT add Storage round-trip tests here. `tests/integration/storage/llm_message_storage.rs` already covers `save_llm_message` → `list_latest_llm_messages` ordering, limit, error path. This file is port-trait-only.

**Note:** `LlmMessage` derive is just `#[derive(Debug, Clone)]` — no `Serialize`/`Deserialize`. Skip any serde round-trip test; `LlmMessage` is not serialized at the port layer (persistence happens via Storage's own serde for SQLite blob storage, internal to the adapter).

#### A.3.2 — `src/application/ports/text_checker_tests.rs` (NEW)

Tests for `TextChecker` port trait:

1. **Trait enforces the method signature** (compile-time guarantee just from `impl TextChecker for X`).
2. **`HarperTextChecker::check` happy path** (if not already covered by `src/adapters/driven/text_check/*_tests.rs` — verify and skip duplicates).
3. **Trait polymorphism — `dyn TextChecker` dispatch works correctly between `HarperTextChecker` and any test double.**

If `HarperTextChecker` already has a sibling `*_tests.rs`, only add the polymorphism test here. Don't duplicate.

### Phase A.4 — Add integration tests exercising prod wiring

#### A.4.1 — One integration test that exercises the prod factory path

Currently `GameService::with_mock_quantifier` (test-only factory in prod code) is heavily used across `tests/integration/` — 45 occurrences total (verified by grep):
- `tests/integration/pipeline/pipeline.rs` (14)
- `tests/integration/pipeline/retry.rs` (7)
- `tests/integration/mod.rs` (1)
- `tests/integration/flow/retry_main.rs` (10)
- `tests/integration/flow/sequence.rs` (8)
- `tests/integration/flow/retry_event.rs` (3)
- `tests/integration/game_service.rs` (2)

The production path (`GameService::with_storage` → `get_llm_recorder_for`) is exercised by 10 sites total (verified by grep):
- **2 production sites:** `src/adapters/driving/http/server_impl.rs:38,45`
- **8 test sites:** `src/test_support/test_app_builder.rs:305`, `src/adapters/driving/http/settings_fragment/handlers_tests.rs` (2 sites, lines 22, 45), `src/adapters/driving/http/prompt_presets_fragment/handlers_tests.rs` (2 sites, lines 26, 307), `tests/poison_recovery.rs` (2 sites, lines 42, 77)

`get_llm_recorder_for` itself has zero direct unit tests — it's only reached via `with_storage`/`with_backends` callers above.

Add at least one integration test (suggested home: `tests/integration/pipeline/pipeline.rs` or a new `tests/integration/pipeline/wiring.rs`) that:
1. Constructs a `GameService` via `with_storage` + a test `Connection { provider: Mock, .. }`.
2. Asserts the `llm_recorder` field is wired correctly (provider name, forensics repo is the storage).
3. Drives one full pipeline call.
4. Asserts a forensics `LlmMessage` row made it into `Storage` (proving the recorder + factory + storage are all in sync).

This catches the silent-fallback regression (sibling plan Fix 2) at the integration level — if someone reintroduces `unwrap_or_else(Mock+Noop)`, the test should fail on the forensics assertion.

### Phase B.1 — Decide test mirror convention (write it down)

Before reorganizing, decide and document. **No written convention exists today** — not in AGENTS.md, not in `docs/architecture/system.md` §11, not in `scripts/check_test_structure.py`. The `tests/integration/storage/` mirror is a one-off that was never propagated or codified. This plan therefore **ESTABLISHES a new convention** (not restores an existing one):

1. **Unit tests:** `<src_dir>/<file>_tests.rs` sibling — already enforced by `check_test_structure.py`. No change.
2. **Integration tests:** `tests/<src_tier_path_mirror>/<module>.rs` — full path mirror.
   - Example: `src/application/action_pipeline/pipeline.rs` ↔ `tests/application/action_pipeline/pipeline.rs`
   - Example: `src/adapters/driving/http/fragments/actions.rs` ↔ `tests/adapters/driving/http/fragments/actions.rs`
   - Exception: tests that span multiple src dirs (lifecycle, integration) — document per-file.
3. **Test binaries:** Decide if `tests/integration/mod.rs`, `tests/http/mod.rs`, `tests/browser/mod.rs` stay as separate `[[test]]` binaries in `Cargo.toml` (current state) or get reorganized. Recommendation: keep separate binaries for build parallelism, but reorganize the directory structure inside each binary.

Update AGENTS.md §"THE TEST-FIRST PHILOSOPHY" + `docs/architecture/system.md` §11 with the convention. Codify in `scripts/check_test_structure.py` (new rule) so drift can't recur silently.

### Phase B.2 — Reorganize integration tests per convention

Per B.1 decision, reorganize:

1. **`tests/integration/pipeline/` → `tests/integration/action_pipeline/`** (rename to match `src/application/action_pipeline/`).
2. **`tests/http/` → either `tests/integration/driving/http/` OR keep as separate binary but rename `tests/driving/http/`** — decide in B.1.
3. **Create missing test directories:**
   - `tests/integration/ports/` (for `llm_message_repository`, `text_checker`, `llm_provider` port trait tests)
   - `tests/integration/application/` (for `llm_recorder`, `text_check_service`, `application_service`, `game_service`, `lifecycle` — move the flat files here)
   - `tests/integration/agents/` + `tests/integration/agents/quantifier/` (mirror src)
   - `tests/integration/narrative_prompt/`
   - `tests/integration/text_check/` (driven adapter)
   - `tests/integration/bootstrap/` (or move `src/bootstrap/*_tests.rs` tests out if they belong at integration level — verify per case)
   - `tests/integration/action_pipeline/` (created in step 1)
4. **Move flat-file tests into mirrored subdirs:**
   - `tests/integration/application_service.rs` → `tests/integration/application/application_service.rs`
   - `tests/integration/game_service.rs` → `tests/integration/application/game_service.rs`
   - `tests/integration/lifecycle.rs` → `tests/integration/application/lifecycle.rs` (verify mirror; "lifecycle" might be cross-cutting — document if so)
   - `tests/integration/llm_client.rs` → `tests/integration/adapters/driven/llm/client.rs` (or similar mirror)
5. **Update `Cargo.toml` `[[test]]` entries if paths changed.**
6. **Update `tests/integration/mod.rs` (or equivalent) to declare the new submodules.**
7. **Update `docs/architecture/system.md` §11 test binary catalog.**

### Phase B.3 — Add test-mirror guardrail

Extend `scripts/check_test_structure.py` (or add a new `scripts/check_test_mirror.py`) to verify:

1. For every `tests/<tier>/<module>.rs`, the corresponding `src/<tier>/<module>.rs` exists. (Or document exceptions.)
2. For every new `src/` module with substantial code (say >50 lines non-test), a corresponding `tests/` file exists (either sibling `*_tests.rs` OR an integration test in the mirrored path).
3. Run as part of `python build.py` — fail or warn (decide) on violations.

This prevents drift recurring after future phases.

---

## Sequencing vs sibling plan

Two valid orderings:

### Option 1 — Code fixes first, then tests (safer)

1. Implement `phase2-thermonuclear-review-fixes.md` first.
2. Then implement this plan — tests cover the FINAL shape of the code, not the broken shape.
3. Risk: longer total time before tests catch regressions.

### Option 2 — Tests first, then code fixes (faster regression catch)

1. Write tests for the CURRENT (broken) shape first — tests will pass against current code.
2. Implement code fixes — tests should still pass (behavior-preserving) OR tests need adjustment (behavior-changing fixes).
3. Risk: rework on tests that targeted the broken shape.

### Recommendation

Option 1 (code first, then tests). The code fixes change behavior in 3 places (Fix 2 silent fallback removal is behavior-changing — prod errors propagate instead of silent Mock). Writing tests against the silent-fallback behavior would be wasted work. Write tests against the fixed shape.

If time pressure demands parallelism: Phase A.1 (orchestrator unit tests) is mostly behavior-preserving and could be written against current code in parallel with code fixes, then adjusted if needed. Phase B (test reorganization) is fully independent — can run any time.

---

## Open decisions (need user input before implementation)

1. **Axis A scope — per-file coverage targets:** proposed 70% for ports, 85% for factories, 80% for orchestrators. Confirm or adjust.
2. **Axis B — strict vs documented mirror:** does every `src/` subdir need a `tests/` mirror, or are documented exceptions OK (e.g., `src/test_support/` obviously has no test mirror)?
3. **Axis B — `tests/http/` fate:** merge into `tests/integration/driving/http/` (one binary) or rename to `tests/driving/http/` (separate binary)?
4. **Axis B — flat-file handling:** `lifecycle.rs` is cross-cutting — does it stay as a flat file with documented exception, or get split into per-module tests?
5. **Phase B.3 guardrail severity:** fail build on mirror violation, or warn only?
6. **Sequencing:** Option 1 (code first) vs Option 2 (tests first) vs parallel.
7. **Shared test doubles:** sibling plan Fix 4 extracts `NoopForensics` to `test_support/`. This plan references it for the new orchestrator tests. Should there be a recording `LlmMessageRepository` (counts calls, captures last message) shared in `test_support/` too, or duplicated per test file?
8. **`src/application/ports/llm_provider_tests.rs` (already exists, 32.7% coverage):** extend it to cover the orchestrator-side methods + LlmCallResult shape, or leave as-is and focus on the 6 zero-coverage files?

---

## What this plan does NOT cover

- **Code-level fixes** for the 14 review findings (`get_llm_backend_for` deletion, sanitize relocation, MockBackend storage drop, NoopForensics dedup, etc.) — sibling plan: `phase2-thermonuclear-review-fixes.md`.
- **arch-lint rule activation** — Phase 1.7 deviation persists across both plans. Out of scope.
- **T2 reliability plan work** — separate plan: `docs/plans/reliability-and-cancellation-plan.md`.
- **Manual LLM smoke test + text check UI verification** — skill requires these before "done" claim; out of scope for this automation-focused plan.
- **Browser/E2E test suite** (`tests/browser/`) — already follows a different convention (Playwright); out of scope.
- **Real-LLM smoke tests** (`tests/llm/`) — `#[ignore]` by default, separate concern.

---

## Verification Log (all confirmed)

| Concern | Source | Verdict | Evidence |
|---|---|---|---|
| 6 new Phase 2 files have zero unit tests | User + reviewer | ✅ CONFIRMED | `find src/application src/bootstrap -name "*_tests.rs"` returns none matching the 6 new files; existing list has 0 of: `llm_recorder_tests.rs`, `text_check_service_tests.rs`, `llm_factory_tests.rs`, `text_check_factory_tests.rs`, `llm_message_repository_tests.rs`, `text_checker_tests.rs` |
| Orchestrators appear in tests only as scaffolding | Reviewer (extra concern) | ✅ CONFIRMED | `LlmCallRecorder` mentioned in 5 test files; in all 5, it's constructed inside `make_test_recorder` helper as scaffolding for `with_mock_quantifier`. Zero tests assert on what the recorder does. |
| Integration test folder doesn't mirror `src/` structure | User | ✅ CONFIRMED | `find src -type d` + `find tests -type d` comparison shows: `tests/integration/pipeline/` ≠ `src/application/action_pipeline/`; `tests/http/` ≠ `src/adapters/driving/http/`; no `tests/integration/{ports,agents,narrative_prompt,text_check,bootstrap,action_pipeline}` dirs; flat files in `tests/integration/` root where `src/application/` has subdirs |
| `get_llm_recorder_for` has zero direct test callers | Reviewer + user | ✅ CONFIRMED | `grep -rn "get_llm_recorder_for" tests/` returns 0 direct callers; reached only via `GameService::with_storage`/`with_backends` (10 total call sites: 2 prod in `server_impl.rs:38,45` + 8 test sites) |
| Coverage on new files is low | Coverage report | ✅ CONFIRMED | `llm_provider.rs` 32.7%, `text_checker.rs` 44.4%, `text_check_factory.rs` 75%, `llm_factory.rs` 78.6%; `llm_recorder.rs` + `text_check_service.rs` show no coverage line in the report (ильнообработать) |
| AGENTS.md "Tests as Documentation" contract broken | User (implicit) | ✅ CONFIRMED | `AGENTS.md:162` asserts contract; no `tests/` file exists for Phase 2 deliverable modules |
| `scripts/check_test_structure.py` doesn't enforce mirror | Reviewer (implicit) | ✅ CONFIRMED | Script content read: enforces only "no inline `#[cfg(test)] mod` blocks" — no naming/mirror rule |
| Only 4 sites use prod `with_storage` factory | — | ✅ CONFIRMED (count corrected) | `grep -rn "GameService::with_storage" src/ tests/` returns 10 sites: 2 prod (`server_impl.rs:38,45`) + 8 test (`test_app_builder.rs:305`, `settings_fragment/handlers_tests.rs:22,45`, `prompt_presets_fragment/handlers_tests.rs:26,307`, `poison_recovery.rs:42,77`). Original plan text said "only 4 in poison_recovery" — that undercounted; poison_recovery has 2, not 4 |
| 20+ sites use test-only `with_mock_quantifier` factory | — | ✅ CONFIRMED (count corrected) | `grep -rcn "with_mock_quantifier" tests/` returns 45 total occurrences across `pipeline.rs` (14), `retry.rs` (7), `mod.rs` (1), `flow/retry_main.rs` (10), `flow/sequence.rs` (8), `flow/retry_event.rs` (3), `game_service.rs` (2). Original plan said "20+" — technically correct but undersold by 2× |
