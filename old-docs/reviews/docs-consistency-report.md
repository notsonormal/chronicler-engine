# Documentation Consistency Report

**Generated:** 2026-05-31  
**Scope:** `chronicler_engine/docs/`  
**Status:** FAIL

---

## Summary

| Category | Count |
|----------|-------|
| **ARCHITECTURE** | 2 |
| **SYSTEM** | 2 |
| **ADR** | 1 |
| **CROSS_REF** | 1 |
| **REFERENCE** | 1 |
| **GHOST_MODULE** | 2 |

**Total Issues:** 9

---

## Documentation Inconsistencies

### ARCHITECTURE: Module structure claims don't match code

- **FILE:** `docs/architecture/system.md:62-67`
  - **Current:** Claims `text_check/llm_client.rs` contains `call_chat_completions()` and HTTP client helpers
  - **Expected:** `llm_client.rs` is at `src/narrative/llm_client.rs` (single file, not in a directory module). The `llm/` directory contains backend implementations (`openrouter.rs`, `ollama.rs`, `deepseek.rs`, `mock.rs`), not client logic.
  - **Impact:** Misleads developers about where LLM HTTP logic lives

- **FILE:** `docs/architecture/system.md:174-185`
  - **Current:** Claims storage has `models` submodule with `DbGame`, `DbGameStateSnapshot`, etc., and `mappers` submodule
  - **Expected:** Storage tier uses `Storage` struct with `Backend` enum (`Sqlite`, `InMemory`, `Test`). No per-table repository structs or `models`/`mappers` submodules exist. All table-scoped methods live directly on `Storage`. Directory listing shows: `mod.rs`, `db.rs`, `backend_tests.rs`, `db_tests.rs`.
  - **Impact:** Describes a repository pattern that was removed during storage consolidation (ADR-020)

### SYSTEM: Behavioral descriptions inaccurate

- **FILE:** `docs/system/triggers.md:18-27`
  - **Current:** Describes "Second Quantifier (Post-Narration)" running before trigger evaluation, detecting NPCs in narration
  - **Expected:** Per `game_flow.md:23` and code in `application/action_pipeline/`, quantifier runs AFTER main narration (Phase 4.5), then triggers evaluate (Phase 5), then if trigger fires, post-event quantifier runs (Phase 5.5). Trigger evaluation reads state BEFORE `times_met` increments.
  - **Impact:** Order is confusing — suggests quantifier runs twice before triggers, when it's: main quantifier → triggers → post-event quantifier (only if trigger fires)

- **FILE:** `docs/system/llm_processing.md:140-141`
  - **Current:** Claims `call_chat_completions` in `src/narrative/llm_client.rs` is the "single chokepoint"
  - **Expected:** Function exists at that path (accurate), but doc doesn't mention that `LlmBackend::complete()` is the actual public interface callers use. The function is internal to backend implementations.
  - **Impact:** Minor — function location correct, but architectural emphasis misleading

### ADR: References verified

- **FILE:** `docs/adr/adr-021-state-patch-reducer.md:59-60`
  - **Current:** References "Code: `src/model/agent.rs` (`StatePatch::merge`)" and "Tests: `tests/components/state_patch_tests.rs`"
  - **Expected:** `StatePatch::merge` exists in `src/model/agent.rs` ✅, test file exists ✅
  - **Status:** ✅ **VERIFIED** — Code and test references are accurate

### GHOST_MODULE: Documents describe non-existent modules

- **FILE:** `docs/system/llm_processing.md:98-119`
  - **Current:** Claims `LlmMessageStorage` trait exists in `crate::storage::llm_message_storage` with SQLite + in-memory implementations
  - **Expected:** Storage module contains only: `mod.rs`, `db.rs`, `backend_tests.rs`, `db_tests.rs`. No `llm_message_storage` module or trait exists. LLM logging likely handled directly in `db.rs` or backends.
  - **Impact:** Describes an entire module that doesn't exist

- **FILE:** `docs/system/llm_processing.md:174-180`
  - **Current:** Lists `crate::storage::llm_message_storage` as module location
  - **Expected:** Same as above — no such module
  - **Status:** ❌ **FALSE CLAIM** — Describes non-existent module structure

### CROSS_REF: Path inconsistencies

- **FILE:** `docs/architecture/system.md:215-230`
  - **Current:** References `docs/system/*.md` files correctly, but doc itself uses singular path `docs/system.md` in some contexts
  - **Expected:** All references should use `docs/system/` (directory), not `docs/system.md` (file)
  - **Impact:** Minor — path format inconsistency

### REFERENCE: Schema naming confusion

- **FILE:** `docs/reference/data_schemas.md:25-48`
  - **Current:** Section titled "CharacterSheet Schema (Current)" documents unified structure for `PlayerCard` and `NpcCard`
  - **Expected:** `CharacterSheet` type doesn't exist — separate `PlayerCard` and `NpcCard` types do. Heading implies a unified schema that the code doesn't have.
  - **Impact:** Confuses developers about whether `CharacterSheet` type exists

---

## Actionable Fixes

### Priority 1 (High - Architectural Misinformation)

**FILE:** `docs/architecture/system.md:62-67`  
**Type:** ARCHITECTURE  

Current:
```markdown
  - **`llm_client`**: HTTP client helpers refactored into composable pure functions...
```

Expected:
```markdown
  - **`llm_client`**: Single file `src/narrative/llm_client.rs` with `call_chat_completions()`. Backend implementations in `llm/` directory.
```

---

**FILE:** `docs/architecture/system.md:174-185`  
**Type:** ARCHITECTURE  

Current:
```markdown
- **`models`**: Database row structs...
- **`mappers`**: Conversion logic...
- **`backend`**: Unified `Storage` struct...
```

Expected:
```markdown
- **`Storage`**: Unified struct with `Backend` enum. No `models` or `mappers` submodules. All table operations on `Storage` directly.
```

---

### Priority 2 (High - Ghost Modules)

**FILE:** `docs/system/llm_processing.md:98-119, 174-180`  
**Type:** GHOST_MODULE  

Current:
```markdown
- **`LlmMessageStorage` trait** (`crate::storage::llm_message_storage`) abstracts persistence...
```

Expected:
```markdown
- **LLM Logging**: Handled directly in `crate::storage::db.rs`. No separate trait or module.
```

**Action:** Investigate where LLM message logging actually lives and update accordingly.

---

### Priority 3 (Medium - Behavioral Clarity)

**FILE:** `docs/system/triggers.md:18-27`  
**Type:** SYSTEM  

Current:
```markdown
### 4. Second Quantifier (Post-Narration)
```

Expected:
```markdown
### 4. Quantifier (Post-Narration)
### 5. Trigger Evaluation
### 5.5. Post-Event Quantifier (Conditional)
```

---

### Priority 4 (Low - Schema Naming)

**FILE:** `docs/reference/data_schemas.md:25`  
**Type:** REFERENCE  

Current:
```markdown
## CharacterSheet Schema (Current)
```

Expected:
```markdown
## Character Schema (PlayerCard and NpcCard)
```

---

## Notes

- **DeepSeek Backend:** Verified exists (`src/narrative/llm/deepseek.rs`) and module-exported. ADR-002 claim of "stub — not yet implemented" may be outdated.
- **StatePatch::merge:** Verified exists in `src/model/agent.rs` with documented merge semantics ✅.
- **PromptAssembler:** Verified exists with `assemble()` method and `LayeredPromptAssembler` implementation ✅.
- **tmp/diagnostics:** Does not exist at rest. The `ForensicsCollector` infrastructure that would have created it was removed on 2026-07-03 (never wired in — see `docs/plans/observability-and-forensics-plan.md` Task 2.3). LLM call forensics live in the `llm_messages` SQLite table per ADR-012.
- **Storage structure:** `src/storage/` contains only 4 files — no submodule pattern as described in docs.

---

## Verification Methods Used

1. **`ctx_search`** — Locate code patterns, module imports, trait definitions
2. **`ctx_read`** — Read documentation and source files
3. **`ctx_shell`** — List directory contents, verify file existence
4. **Manual cross-reference** — Compare doc claims against discovered code reality

---

## Recommendations

1. **Immediate:** Fix Priority 1 and 2 issues (architectural lies and ghost modules)
2. **Short-term:** Add markdown link checker to CI (`markdown-link-check`)
3. **Medium-term:** Create "doc owner" assignments for each `.md` file
4. **Long-term:** Add doc tests that `cargo test` can validate (compile-checked examples)
