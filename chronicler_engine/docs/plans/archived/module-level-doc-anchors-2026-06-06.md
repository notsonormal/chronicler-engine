# Module-Level DOC Anchor Migration

**Created:** 2026-06-06  
**Status:** ✅ Complete  
**Type:** Code Quality / Documentation Infrastructure

---

## Problem Statement

The Chronicler Engine had ~67 function-level DOC anchors (`/// [DOC: ...]` and `// [DOC: ...]`) scattered throughout the codebase. Approximately 75% pointed to the generic `docs/architecture/system.md` instead of domain-specific documentation. This created:

1. **Noise**: Per-function anchors cluttered code without proportional value
2. **Maintenance burden**: every new function required manual anchor placement decisions
3. **Weak linkage**: anchors pointed to generic overview rather than specific domain docs
4. **Guardrail complexity**: `DocAnchorVisitor` AST visitor with ~120 lines of control flow analysis

## Solution

Replace function-level DOC anchors with module-level anchors:

- **One `//! [DOC: ...]` per file** on line 1
- **Domain-specific targets** (e.g., `game_flow.md`, `navigation.md`, `triggers.md`)
- **Remove spawn-site guardrail** (INV-004 tests already verify cancellation)
- **Keep long-comment-run guardrail** (prevents documentation re-inline as wall-of-text)

## Files Changed Summary

### Guardrail Infrastructure
- ✅ `tests/infrastructure/guardrails/structure.rs`
  - Added `check_module_doc_anchors()` (~150 lines)
  - Added `MODULE_DOC_EXEMPTIONS`, `SYSTEM_MD_EXEMPT` constants
  - Added `expected_doc_target()` mapping function
  - Removed `DocAnchorVisitor`, `stmt_count`, `contains_control_flow`, `expr_contains_control_flow`
  - Removed `check_doc_anchors()` and `check_spawn_site_docs()`
  - Fixed path separator normalization for cross-platform matching

- ✅ `tests/infrastructure/guardrails/mod.rs`
  - Added `guardrails_module_doc_anchors` test
  - Removed `guardrails_doc_anchors` test
  - Removed `guardrails_spawn_site_docs` test

### Source Files (102 files with module-level anchors)

#### Application Tier (`application/` - 11 files)
All → `docs/system/game_flow.md`
- `mod.rs`, `application_service.rs`, `context.rs`, `game_service.rs`, `game_lifecycle.rs`, `message_editing.rs`, `query_handlers.rs`
- `action_pipeline/mod.rs`, `actions.rs`, `pipeline.rs`, `retry.rs`

#### Engine Tier (`engine/` - 6 files)
- `mod.rs`, `logic.rs` → `docs/system/navigation.md`
- `trigger_eval.rs` → `docs/system/triggers.md`
- `state_diagnostics.rs` → `docs/architecture/invariants.md`
- `action.rs`, `parser.rs`, `action_processing.rs` → `docs/system/game_flow.md`

#### Model Tier (`model/` - exempt from system.md ban)
- `character.rs` → `docs/system/character_state.md`
- `trigger.rs` → `docs/system/triggers.md`
- `agent.rs` → `docs/system/agent_system.md`
- `llm_backend.rs`, `llm_message.rs` → `docs/system/llm_processing.md`
- `game.rs`, `message.rs`, `quantifier.rs`, `state.rs`, `world.rs`, `mod.rs`, `map.rs`, `scenario.rs`, `state_snapshot.rs` → `docs/system/game_flow.md` or exempt

#### Narrative Tier (`narrative/` - 32 files)
- `mod.rs` → `docs/system/narration_engine.md`
- `agents/*` (10 files) → `docs/system/agent_system.md`
- `llm/*`, `llm_client/*` (11 files) → `docs/system/llm_processing.md`
- `prompt/*` (6 files) → `docs/system/prompt_system.md`
- `text_check/*` (4 files) → `docs/system/text_check.md`

#### Server Tier (`server/` - 25 files)
All → `docs/system/dashboard.md`
- Including fragments, settings_fragment, prompt_presets_fragment

#### Bootstrap (`bootstrap/` - 7 files)
All → `docs/system/startup.md`

#### Storage Tier (`storage/` - exempt, storage schema IS architecture)
- `backend/messages.rs`, `backend/swipes.rs`, `backend/games.rs` → `docs/system/game_flow.md`
- `backend/llm_messages.rs` → `docs/system/llm_processing.md`
- All other storage files exempt

### Documentation Updates
- ✅ `docs/architecture/guardrails.md` - Replaced section 3.2 with module-level anchor requirements
- ✅ `chronicler_engine/AGENTS.md` - Updated lines 111 and 191
- ✅ `chronicler_engine/TODO.md` - Marked DOC anchor item complete
- ✅ `docs/CHANGELOG.md` - (to be updated)

## Domain Doc Mapping Table

| Module | Target Domain Doc | Rationale |
|--------|------------------|-----------|
| `application/*` | `system/game_flow.md` | Action pipeline, message handling |
| `engine/mod.rs`, `logic.rs` | `system/navigation.md` | Movement logic, room finding |
| `engine/trigger_eval.rs` | `system/triggers.md` | Trigger evaluation |
| `engine/state_diagnostics.rs` | `architecture/invariants.md` | State consistency checks |
| `model/character.rs` | `system/character_state.md` | NPC card model |
| `model/trigger.rs` | `system/triggers.md` | Trigger model |
| `model/agent.rs` | `system/agent_system.md` | Agent trait, execution |
| `model/llm*` | `system/llm_processing.md` | LLM client, backends |
| `narrative/agents/*` | `system/agent_system.md` | Agent implementations |
| `narrative/prompt/*` | `system/prompt_system.md` | Prompt assembly, budget |
| `narrative/llm/*` | `system/llm_processing.md` | LLM backends, sanitization |
| `narrative/text_check/*` | `system/text_check.md` | Harper text checking |
| `narrative/mod.rs` | `system/narration_engine.md` | Narrative orchestration |
| `server/*` | `system/dashboard.md` | HTMX frontend, fragments |
| `bootstrap/*` | `system/startup.md` | Load, run, validate |
| `model/*` (other) | exempt | Model tier IS architecture |
| `storage/*` | exempt | Storage schema IS architecture |
| `cli.rs`, `error.rs`, `lib.rs`, `main.rs`, `settings.rs` | exempt | Cross-cutting infrastructure |
| `test_support/*` | exempt | Internal test infrastructure |

## Verification

### Anchor Counts
```
Module-level //! [DOC: anchors: 102
Function-level /// [DOC: anchors: 5 (all in exempt files: cli.rs, settings.rs, test_support/)
Body-level // [DOC: anchors: 7 (all in exempt files)
```

### Test Results
- ✅ `cargo nextest run --test guardrails` - 15 tests pass
- ✅ `cargo nextest run --test architecture` - 1 test passes
- ✅ `python build.py` - All 9 steps pass
  - Formatting (cargo fmt)
  - Clippy (all-targets, all-features)
  - Architecture tests
  - Guardrail tests
  - Test structure validation
  - Build (debug)
  - Data/assets copy
  - Full test suite (884 tests pass, 1 skipped, 3 LLM tests skipped)
  - Coverage (disabled by default)

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Removing anchor guardrail breaks CI | Compile error | Removed all call sites in same commit |
| Files without clear domain doc | Points to `system.md` | Exemption list for cross-cutting/storage/model files |
| Large diff (~100 files) | Merge conflicts | Mechanical changes, batched by module |

## Architecture Decisions

### ADR: Module-Level Over Function-Level
**Decision:** One `//! [DOC: ...]` per file, not per function  
**Rationale:** Reduces noise while maintaining code-to-doc linkage. Complex blocks belong in extracted methods, not inline comments.

### ADR: Remove Spawn-Site Guardrail
**Decision:** Removed `check_spawn_site_docs()`  
**Rationale:** INV-004 contract tests already verify cancellation behavior. Proximity-based comments add no test value.

### ADR: Keep Long-Comment-Run Guardrail
**Decision:** Retained `guardrails_long_comment_runs`  
**Rationale:** Prevents re-inlining documentation as wall-of-text comments. Encourages external docs over inline walls.



## 2026-06-06 Post-Implementation Update: storage.md

**Decision:** Created  as dedicated domain documentation for the storage tier.

**Rationale:**
- Storage tier IS architecture, but deserves its own spec doc (not system.md or game_flow.md)
- Reduces exemptions - storage files now have proper domain docs
- Follows principle: every tier should link to domain documentation

**Updated Mapping:**

| Storage Module | Target Doc | Rationale |
|---------------|-----------|-----------|
| ,  |  | Storage tier infrastructure |
|  (core, worlds, personas, etc.) |  | CRUD implementations |
|  |  | DB row structs |
|  |  | Domain↔DB mappings |
| , ,  |  | Game flow logic |
|  |  | LLM call logging |

**Result:**
- 24/28 storage files have  anchors
- 4 files have specific domain anchors (game_flow.md, llm_processing.md) - correct
-  removed from MODULE_DOC_EXEMPTIONS
- Only 3 files remain exempt: , , 
- Added  to SYSTEM_MD_EXEMPT (error taxonomy IS architecture)

**Final Exemption List (Minimal):**



**Total Module DOC Anchors Added: 130 files

## Follow-Up Work

- [ ] Consider adding extraction suggestions to guardrail violations ("Extract this block to a separate method")
- [ ] Add automated doc link validation (ensure referenced docs exist)
- [ ] Consider adding module-level anchors to test files linking to spec docs (optional)

---

## Change Log

**2026-06-06:**
- Implemented module-level DOC anchor system
- 102 files updated with `//! [DOC: ...]` on line 1
- Removed ~120 lines of AST visitor code
- Updated guardrails.md, AGENTS.md, TODO.md
- All 884 tests pass
