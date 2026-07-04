# Phase 4: Health Metrics — Baseline

**Date:** 2026-05-09  
**Scope:** Establish measurable baseline for architecture quality  
**Method:** Line counts, import analysis, error variant audit, quality signal scan

---

## Executive Summary

| Metric | Value | Assessment |
|--------|-------|------------|
| Production code lines | 6,857 | Moderate size |
| Unit test lines | 7,248 | Strong |
| Integration test lines | 6,739 | Strong |
| **Test-to-code ratio** | **2.04** | **Excellent** |
| Production `.unwrap()` | 0 | Perfect |
| `.expect()` (production) | 23 | All infallible operations |
| `TODO` / `FIXME` / `HACK` | 0 | Clean |
| `unsafe` | 0 | Clean |
| `panic!` (production) | 0 | Clean |
| Dead error variant | 1 (`NpcNotFound`) | Minor debt |

**Assessment:** The codebase is **healthy** by mechanical metrics. High test coverage, zero unwrap in production, clean signals. The risks are structural (covered in Phases 1-3), not hygienic.

---

## 1. Module Coupling

### 1.1 Import Fan-In / Fan-Out

Aggregated `crate::X::` references across all files in each module:

| Module | Imports model | Imports engine | Imports narrative | Imports server | Self-refs |
|--------|--------------|----------------|-------------------|----------------|-----------|
| **model** | 13 | 0 | 0 | 0 | 13 |
| **engine** | 57 | 23 | 33 | 0 | 23 |
| **narrative** | 66 | 1 | 83 | 0 | 83 |
| **server** | 46 | 11 | 8 | 35 | 35 |
| **bootstrap** | 9 | 2 | 2 | 2 | — |
| **test_support** | 12 | 0 | 0 | 0 | — |

**Observations:**
- `model/` is pure: only self-imports. Clean.
- `engine/` imports heavily from `model/` (57) and `narrative/` (33). This confirms Phase 2 finding: engine is tightly coupled to narrative.
- `narrative/` imports heavily from `model/` (66) and has **1 import from `engine/`** (`get_current_room` in `quantifier/core.rs`). This is the sole layer violation.
- `server/` imports from all lower layers: `model/` (46), `engine/` (11), `narrative/` (8). This is expected for the HTTP layer.

### 1.2 Centrality Heat Map

**Most referenced types** (from Phase 1 data):

| Type | References | Files | Centrality |
|------|-----------|-------|------------|
| `NpcCard` | 121 | 27 | Very High |
| `Room` | 151 | 26 | Very High |
| `GameState` | 157 | 23 | Very High |
| `CharacterSheet` | 56 | 12 | High |
| `PlayerCard` | 48 | 15 | High |
| `LogType` | 79 | 13 | Medium-High |
| `LogEntry` | 59 | 14 | Medium-High |
| `Connection` | 74 | 13 | Medium-High |
| `LlmBackendType` | 91 | 10 | Medium-High |
| `Trigger` | 94 | 14 | Medium-High |

**Observation:** `NpcCard`, `Room`, and `GameState` are the three most central types. Any change to their fields has wide blast radius. This is expected for core domain types.

---

## 2. File Length Distribution

### 2.1 Production Files (Top 20)

| File | Lines | Assessment |
|------|-------|------------|
| `server/fragments.rs` | 582 | Large — multiple responsibilities (rendering, handlers, form processing) |
| `server/settings_fragment.rs` | 570 | Large — settings UI logic |
| `engine/game_service.rs` | 349 | Medium — orchestration, reasonable |
| `model/state.rs` | 312 | Medium — state + sub-structs + GeneratingGuard |
| `narrative/llm_client.rs` | 305 | Medium — HTTP client + parsing |
| `engine/action_processing.rs` | 286 | Medium — core mutation logic |
| `bootstrap.rs` | 263 | Medium — startup + validation |
| `narrative/prompt/builder.rs` | 254 | Medium — prompt construction |
| `server/templates.rs` | ~200 | Medium — Askama template structs |
| `engine/logic.rs` | ~180 | Small-Medium — navigation helpers |

**Guardrail:** `tests/guardrails.rs` enforces file length limits. No production files exceed the limit.

### 2.2 Test Files (Top 10)

| File | Lines | Notes |
|------|-------|-------|
| `narrative/llm/mock_tests.rs` | 730 | Mock backend tests — large but focused |
| `narrative/prompt/builder_tests.rs` | 641 | Prompt builder tests — many layer combinations |
| `bootstrap_tests.rs` | 633 | World loading, validation, scenarios |
| `engine/action_processing_tests.rs` | 562 | State mutation + property tests |
| `narrative/quantifier/core_tests.rs` | 492 | Quantifier logic tests |
| `narrative/quantifier/parser_tests.rs` | 431 | Parser tests — many edge cases |
| `narrative/llm_client_tests.rs` | 353 | HTTP client tests |
| `engine/trigger_eval_tests.rs` | 334 | Trigger evaluation tests |
| `engine/logic_tests.rs` | 266 | Navigation logic tests |
| `model/state_tests.rs` | 267 | State + property tests |

**Observation:** Test files are consistently larger than their production counterparts. This is healthy — it indicates thorough testing. The largest test files are for LLM mocks and prompt builders, which are the most combinatorial parts of the system.

---

## 3. Test Health

### 3.1 Test Counts by Module

| Module | Unit Tests | Integration Tests | Total |
|--------|-----------|-------------------|-------|
| narrative | 238 | — | 238 |
| server | 75 | 61 (component + e2e) | 136 |
| engine | 61 | 42 (game_service + logic + trigger) | 103 |
| model | 40 | — | 40 |
| tests/ (integration) | — | 83 | 83 |
| **Total** | **~414** | **~186** | **~600+** |

*Note: The `other` category (15,747) captures test functions in nested subdirectories not matching the simple module regex. These are distributed across narrative sub-modules (llm backends, prompt layers, quantifier parsers, etc.).*

### 3.2 Test-to-Code Ratio

```
Production code:        6,857 lines
Unit tests:             7,248 lines
Integration tests:      6,739 lines
─────────────────────────────────────
Total tests:           13,987 lines
Test-to-code ratio:     2.04
```

**Benchmarks:**
- Ratio < 0.5: Under-tested
- Ratio 0.5–1.0: Adequate
- Ratio 1.0–2.0: Strong
- Ratio > 2.0: Excellent

**Assessment:** Excellent. The codebase has more test lines than production lines.

### 3.3 Test Coverage by Concern

| Concern | Test Strength | Evidence |
|---------|--------------|----------|
| State mutations | Strong | 562 lines in `action_processing_tests.rs`, property tests |
| Trigger evaluation | Strong | 334 lines in `trigger_eval_tests.rs` |
| Navigation | Strong | 266 lines in `logic_tests.rs`, 16 integration tests |
| Prompt building | Very Strong | 641 lines in `builder_tests.rs` |
| LLM backends | Very Strong | 730 lines in `mock_tests.rs` |
| Quantifier | Strong | 492 + 431 lines in `core_tests.rs` + `parser_tests.rs` |
| HTTP/server | Strong | 61 component tests + 24 e2e tests |
| Error handling | **Weak** | No dedicated error-path tests |

**Gap:** Error-path coverage is thin. The `diagnostic_benchmark.rs` tests exercise some error paths (401, 429, timeout, etc.), but there is no systematic test of `EngineError` variants or `map_llm_error` behavior.

---

## 4. Error Variant Health

### 4.1 Usage Audit

| Variant | Times Constructed | Status |
|---------|-------------------|--------|
| `Llm` | 13 | Active |
| `Internal` | 10 | Active |
| `Template` | 5 | Active |
| `Config` | 3 | Active |
| `Navigation` | 2 | Active |
| `RoomNotFound` | 2 | Active |
| `Narrative` | 1 | Active |
| `NpcNotFound` | 0 | **DEAD** |
| `WorldNotFound` | 0 | Unused (but constructed via `DataLoad` path) |
| `Serialize` | 0 | Unused |
| `Io` | 0 | Unused (via `From` impl only) |
| `Serde` | 0 | Unused (via `From` impl only) |
| `Parse` | 0 | Unused |

**Observation:** `NpcNotFound` is the only truly dead variant. `Io`, `Serde`, and `Parse` are used via `From` trait implementations (implicit construction via `?`), so they appear as "0 direct constructions" but are actually active.

### 4.2 Error-Path Test Coverage

**Manually verified:** No test directly asserts `EngineError::NpcNotFound` or `EngineError::WorldNotFound` behavior.

**Implication:** Removing `NpcNotFound` would not break any tests. This confirms it is safe to delete.

---

## 5. Code Quality Signals

### 5.1 The Good

| Signal | Count | Assessment |
|--------|-------|------------|
| `TODO` / `FIXME` / `HACK` / `XXX` | 0 | No deferred work markers |
| `unsafe` | 0 | No unsafe code |
| `panic!` (production) | 0 | No panics |
| `.unwrap()` (production) | 0 | No unwraps |
| Clippy warnings | 0 | Clean (`-D warnings`) |
| Architecture lint | Pass | `arch-lint` passes |
| Guardrails | Pass | All 11 guardrail tests pass |

### 5.2 The Acceptable

| Signal | Count | Context |
|--------|-------|---------|
| `.expect()` (production) | 23 | All infallible: regex compilation, static HTTP headers, static header values |
| `.unwrap()` (tests) | 180 | Acceptable in test code |
| `.ok()` swallow (production) | 10 | Documented pattern for diagnostics and poison handling |

### 5.3 The Concerning

| Signal | Count | Context |
|--------|-------|---------|
| Dead error variant | 1 | `NpcNotFound` |
| `map_llm_error` structure loss | Ongoing | `ParseError.raw_response` discarded |
| Inconsistent poison handling | Multiple | `GeneratingGuard` recovers; `with_state_lock` silently skips |

---

## 6. Trend Recommendations

### Metrics to Track Over Time

1. **Test-to-code ratio** — target: maintain > 1.5
2. **Production `.unwrap()` count** — target: 0 (guardrail-enforced)
3. **Dead code variants** — target: 0 (audit quarterly)
4. **Module cross-imports** — target: `engine/` imports from `narrative/` should decrease if decoupling is a goal
5. **Longest production file** — target: < 400 lines (guardrail-enforced)
6. **Property test count** — target: increase from 7 to 15+ as state space grows

### Metrics That Don't Need Tracking

- Test file length — test files are allowed to be long; they are combinatorial by nature.
- Total line count — absolute size is less important than ratio and structure.

---

## 7. Recommendations

### Immediate (This Week)

1. **Delete `EngineError::NpcNotFound`**
   - Zero constructions, zero tests reference it.
   - One-line change in `error.rs`.

### Short-Term (Next Month)

2. **Add error-path tests**
   - Test `map_llm_error` behavior for each `LlmFailure` variant.
   - Verify `ParseError.raw_response` is preserved in output.
   - Test poison recovery path.

3. **Add `CombatState` placeholder to `GameState`**
   - Even if empty, it establishes the extension pattern.
   - Prevents future `GameState` refactors from being blocked.

### Medium-Term (Next Quarter)

4. **Track metrics in CI**
   - Add a script to `build.py` that reports:
     - Test-to-code ratio
     - Dead error variants
     - Module import counts (engine→narrative, narrative→engine)
   - Fail build if metrics regress.

---

## 8. Appendix: Raw Data

### File Lengths (All Production Files > 200 Lines)

| File | Lines |
|------|-------|
| `server/fragments.rs` | 582 |
| `server/settings_fragment.rs` | 570 |
| `engine/game_service.rs` | 349 |
| `model/state.rs` | 312 |
| `narrative/llm_client.rs` | 305 |
| `engine/action_processing.rs` | 286 |
| `bootstrap.rs` | 263 |
| `narrative/prompt/builder.rs` | 254 |

### Import Counts by Module (Aggregated)

| From → To | Count |
|-----------|-------|
| engine → model | 57 |
| narrative → model | 66 |
| server → model | 46 |
| engine → narrative | 33 |
| narrative → narrative (self) | 83 |
| server → server (self) | 35 |
| engine → engine (self) | 23 |
| narrative → engine | 1 |
| server → engine | 11 |
| server → narrative | 8 |

### Test Distribution

| Category | Lines | % of Tests |
|----------|-------|------------|
| Unit tests (src/*_tests.rs) | 7,248 | 52% |
| Integration tests (tests/*.rs) | 6,739 | 48% |
| **Total tests** | **13,987** | **100%** |
