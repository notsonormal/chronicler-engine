# Structured Error Taxonomy

## Problem

`EngineError` uses plain `String` payloads for most variants (`Io`, `Parse`, `Narrative`, `Llm`, `Internal`, `Config`, `Template`). This forces code like `game_service.rs` to perform **string content matching** (`msg.contains("timed out")`) to categorize failures. When something breaks, you must grep for the exact string to find the source.

## Approach

**Add structured variants alongside existing ones.** Do NOT change the shape of existing tuple variants in this plan — that creates a cascade of breakage across ~25 files. Instead:

1. Introduce `LlmFailure` and `NarrativeFailure` enums
2. Introduce `InternalError` struct
3. Change `EngineError::Llm`, `EngineError::Narrative`, `EngineError::Internal` to wrap the new types
4. Keep `Io`, `Parse`, `Config`, `Template` as tuple variants; just improve the strings passed to them
5. Eliminate all `msg.contains(...)` string matching

This is a smaller, safer change than a full struct-variant migration.

---

## Phase 1: Design Types

### Task 1: Design `LlmFailure`, `NarrativeFailure`, `InternalError`

**Description:** Add new error types to `src/error.rs`.

**Design:**
```rust
#[derive(Error, Debug)]
pub enum LlmFailure {
    #[error("LLM returned an empty response")]
    EmptyResponse,
    #[error("LLM API returned HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("LLM network error contacting {url}: {detail}")]
    Network { url: String, detail: String },
    #[error("Failed to parse LLM response as {expected_format}")]
    ParseError { raw_response: String, expected_format: &'static str },
    #[error("LLM request timed out")]
    Timeout,
}

#[derive(Error, Debug)]
pub enum NarrativeFailure {
    #[error("Prompt build failed at stage '{stage}': {reason}")]
    PromptBuild { stage: &'static str, reason: &'static str },
    #[error("Narration generation failed at stage '{stage}': {reason}")]
    Generation { stage: &'static str, reason: &'static str },
}

#[derive(Error, Debug)]
#[error("Invariant violated: {invariant}")]
pub struct InternalError {
    pub invariant: String,
}

pub fn internal_error(invariant: impl Into<String>) -> InternalError {
    InternalError { invariant: invariant.into() }
}

impl From<InternalError> for EngineError {
    fn from(e: InternalError) -> Self {
        EngineError::Internal(e)
    }
}
```

**Acceptance criteria:**
- [ ] New types compile
- [ ] `Display` impls produce human-readable messages
- [ ] `From<InternalError> for EngineError` works with `?` operator

**Files touched:** `src/error.rs`
**Estimated scope:** Small

---

### Task 2: Update `EngineError` Variants

**Description:** Replace `Llm(String)`, `Narrative(String)`, `Internal(String)` with structured wrappers.

**Before:**
```rust
Llm(String),
Narrative(String),
Internal(String),
LlmEmptyResponse,
```

**After:**
```rust
#[error("LLM error: {0}")]
Llm(#[source] LlmFailure),
#[error("Narrative generation error: {0}")]
Narrative(#[source] NarrativeFailure),
#[error("Internal invariant violated: {0}")]
Internal(#[source] InternalError),
// LlmEmptyResponse removed — use Llm(LlmFailure::EmptyResponse)
```

**Acceptance criteria:**
- [ ] `cargo check` passes with new types
- [ ] All existing `EngineError::LlmEmptyResponse` usages flagged for migration

**Files touched:** `src/error.rs`
**Estimated scope:** Small

---

## Checkpoint 1

- [ ] New error types compile
- [ ] `python build.py` passes (temporary `From` shim if needed for old call sites)

---

## Phase 2: Migrate Call Sites

### Task 3: Migrate `llm_client.rs`

**Description:** Replace string-based error constructions with `LlmFailure` variants.

**Target changes:**
- `Err(format!("LLM API error: {error_msg}"))` → `Err(EngineError::Llm(LlmFailure::Http { status: 200, body: error_msg }))`
- `Err("The world seems to hold its breath...".to_string())` → `Err(EngineError::Llm(LlmFailure::ParseError { raw_response, expected_format: "content or reasoning" }))`
- `Err(format!("Failed to parse LLM response: {e}"))` → `Err(EngineError::Llm(LlmFailure::ParseError { raw_response, expected_format: "valid JSON" }))`
- Network errors → `LlmFailure::Network { url, detail }`

**Acceptance criteria:**
- [ ] Zero string-based `Llm` error constructions
- [ ] All `llm_client.rs` tests pass

**Files touched:** `src/narrative/llm_client.rs`
**Estimated scope:** Medium

---

### Task 4: Migrate `llm.rs` Backends

**Description:** Update backend implementations to use structured errors.

**Target changes:**
- `EngineError::LlmEmptyResponse` → `EngineError::Llm(LlmFailure::EmptyResponse)`
- `EngineError::Narrative("Mock failure".to_string())` → `EngineError::Narrative(NarrativeFailure::Generation { stage: "mock", reason: "configured_failure" })`
- `EngineError::Config("...".into())` → `EngineError::Config { key: "llm_backend", reason: "deepseek_not_implemented" }` (if Config becomes struct variant)

**Acceptance criteria:**
- [ ] All backends use structured `LlmFailure` and `NarrativeFailure`
- [ ] Tests updated for new error messages

**Files touched:** `src/narrative/llm/mod.rs`, `src/narrative/llm/backends.rs`
**Estimated scope:** Medium

---

### Task 5: Migrate `game_service.rs`

**Description:** Replace string-match error handling with `match` on `LlmFailure` variants.

**Before:**
```rust
EngineError::Narrative(msg) if msg.contains("timed out") => "LLM Error: request timed out"
```

**After:**
```rust
EngineError::Llm(LlmFailure::Timeout { .. }) => "LLM Error: request timed out"
EngineError::Llm(LlmFailure::Network { .. }) => "LLM Error: response incomplete"
EngineError::Llm(LlmFailure::ParseError { .. }) => "LLM Error: unexpected response format"
EngineError::Llm(LlmFailure::EmptyResponse) => "LLM Error: empty response"
```

**Acceptance criteria:**
- [ ] Zero `msg.contains(...)` string matching
- [ ] All match arms use structured variants
- [ ] User-facing error messages unchanged

**Files touched:** `src/engine/game_service.rs`
**Estimated scope:** Medium

---

### Task 6: Migrate `model/state.rs`

**Description:** Replace `EngineError::Internal(String)` with `internal_error()` helper.

**Target changes:**
- `EngineError::Internal("log_entry_exists".to_string())` → `crate::error::internal_error("log_entry_exists").into()`

**Acceptance criteria:**
- [ ] All `Internal` constructions use `internal_error()`

**Files touched:** `src/model/state.rs`
**Estimated scope:** Small

---

### Task 7: Migrate Remaining Sites

**Description:** Update remaining `EngineError` constructions across the codebase.

**Files to check:**
- `src/engine/action_processing.rs`
- `src/server/fragments.rs`
- `src/server/mod.rs`
- `src/settings.rs`
- `src/bootstrap.rs`
- `src/engine/logic.rs`

**Acceptance criteria:**
- [ ] `grep -n 'EngineError::Llm("' src/**/*.rs` returns nothing
- [ ] `grep -n 'EngineError::Narrative("' src/**/*.rs` returns nothing
- [ ] `grep -n 'EngineError::Internal("' src/**/*.rs` returns nothing

**Files touched:** All `src/**/*.rs`
**Estimated scope:** Medium

---

## Checkpoint 2

- [ ] Zero string-based `Llm`/`Narrative`/`Internal` constructions
- [ ] `game_service.rs` has no `msg.contains(...)` matching
- [ ] `python build.py` passes

---

## Phase 3: Documentation

### Task 8: Generate `docs/diagnostics/error_catalog.md`

**Description:** Write a catalog mapping each `EngineError` variant to diagnosis steps.

**Acceptance criteria:**
- [ ] One section per `EngineError` variant
- [ ] Each section: "First Check", "Common Causes", "Related Invariants"
- [ ] Cross-references `invariants.md`

**Files touched:** `docs/diagnostics/error_catalog.md`
**Estimated scope:** Small

---

### Task 9: Update `DEBUGGING.md`

**Description:** Point to `error_catalog.md` as primary reference.

**Acceptance criteria:**
- [ ] `DEBUGGING.md` error taxonomy section references `error_catalog.md`
- [ ] Agent rules updated to prefer `error_catalog.md` over manual string grepping

**Files touched:** `docs/agents/rules/DEBUGGING.md`
**Estimated scope:** Small

---

## Checkpoint 3

- [ ] `error_catalog.md` complete
- [ ] `DEBUGGING.md` updated
- [ ] `python build.py` passes

---

## Dependency Graph

```
Phase 1: Design Types
├── Task 1: Design LlmFailure, NarrativeFailure, InternalError
└── Task 2: Update EngineError variants
    │
    ▼
Phase 2: Migrate Call Sites
├── Task 3: llm_client.rs
├── Task 4: llm.rs backends
├── Task 5: game_service.rs
├── Task 6: model/state.rs
└── Task 7: Remaining sites
    │
    ▼
Phase 3: Documentation
├── Task 8: error_catalog.md
└── Task 9: DEBUGGING.md
```

---

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Test assertions check exact error strings | Medium | Update test assertions to match new `Display` output; keep messages semantically identical |
| `#[source]` changes `Display` format | Low | Verify `to_string()` output in tests; adjust if needed |
| Some error sites missed | Low | Use `grep` verification in Task 7 |

---

*Plan created: 2026-05-04*
*Scope: Structured errors only — async concurrency is in separate plan*
