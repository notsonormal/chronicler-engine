# Plan: Rename docs/README.md → docs/AGENTS.md + Code-Indexer Sweep (Final)

**Date:** 2026-07-04
**Status:** Approved (Open Questions resolved)
**Scope:** `chronicler_engine/docs/`, `chronicler_engine/scripts/`
**Story points:** ~13 (split into 2 execution sessions — see Phase Ordering)
**Approach:** Docs-only. No production code, no tests.

## Summary

Rename `docs/README.md` → `docs/AGENTS.md` to align with repo convention (root + engine use AGENTS.md, which agents auto-load). Update `generate_docs_index.py` output path. Collapse duplicate "Key Principles"/"Workflow"/"Quick Reference" content in `docs/AGENTS.md` (covered by `chronicler_engine/AGENTS.md` + AUTO-INDEX block). Then clean sediment across ~15 doc files: delete "Key Files" tables, CSS class enumerations, SQL DDL duplication, Code Mapping trees, file-path bullets, `Future Work`/`Backlog` speculation, `(NEW)`/`(Updated)` markers, phase tracker markers, demo-specific content (Redmist Estate, Julian Redmist, Aethelgard), historical PR commentary.

Agents already resolve "where is X?" via `chronicler_engine/AGENTS.md` STRUCTURE block. Duplicating file paths in spec docs is sediment that rots on every rename.

Anchored on writing-great-skills: documentation = specification of static target state. No conversational sediment. One authoritative source per concept.

## Locked Decisions (from Open Questions)

1. **Quick Reference table** → DELETE (B). Redundant with AUTO-INDEX.
2. **Demo content** (Redmist topology, Julian persona, Aethelgard mock) → DELETE entirely from docs (A). Data files are source of truth; engine spec doesn't bake in examples.
3. **Migration Policy** in `data_layer.md` → DELETE entirely (A). Agents read `src/adapters/driven/storage/db.rs` source for migration sequence invariants.
4. **`data_schemas.md` vs `data_layer.md`** → Keep both (A). Drop SQL DDL from `data_schemas.md`, keep JSON examples only.
5. **Execution tempo** → Two sessions (B). Session 1: Phases 1+2+3a (~5 SP). Session 2: Phase 3b+4+5 (~6 SP).
6. **Future Work / Backlog sections** → DELETE on sight (A). `ROADMAP.md` covers long-term vision.
7. **Scope creep** → Latitude to fix similar sediment inline. Surface uncertain cases at end of session for user review.
8. **`architecture/system.md` §7 Storage subsection** → Collapse to one-line pointer to `system/storage.md` (or remove entirely if redundant with `AGENTS.md` STRUCTURE block — verify during implementation; prefer removal if `AGENTS.md` already lists `system/storage.md`).
9. **Cross-doc links** → Fix links broken by the sweep during the sweep. Do NOT touch `docs/plans/` link fixes (out of scope).
10. **H1 sweep** → IN SCOPE. `generate_docs_index.py` is an auto-generation script anyway; H1s must be clean.

## Session 1 — Quick Win + Sediment Sweep (~5 SP)

### Phase 1 — Rename + Generator Update (1 SP)

1. `git mv docs/README.md docs/AGENTS.md`
2. Update `scripts/generate_docs_index.py`:
   - Output target: `README.md` → `AGENTS.md`
   - AUTO-INDEX markers unchanged (`<!-- AUTO-INDEX START -->` / `<!-- AUTO-INDEX END -->`)
3. Update `scripts/install_git_hooks.py` if it greps for `README.md` filename
4. Grep repo for `docs/README.md` references — if found, update to `docs/AGENTS.md` (but NOT in `docs/plans/` per Decision 9)
5. Run `python scripts/generate_docs_index.py` — verify AUTO-INDEX regenerates with all `.md` files including self
6. **Verify**: `grep -rn "docs/README" . --exclude-dir=docs/plans --exclude-dir=docs/plans/archived` → 0 hits

### Phase 2 — Content Collapse in docs/AGENTS.md (1 SP)

- **DELETE** `## Key Principles` section — duplicates `chronicler_engine/AGENTS.md` "Spec-Driven Implementation" + "Documentation Strategy"
- **DELETE** `## Workflow` section — duplicates engine AGENTS.md "Planning Requirements"
- **DELETE** "Quick Reference" table (per Decision 1) — redundant with AUTO-INDEX
- **KEEP** AUTO-INDEX block
- **KEEP** "Plan authoring convention" paragraph (docs-specific: quotable phrase, never line numbers)
- **ADD** top-line pointer above AUTO-INDEX: "For general engine principles, workflow, and conventions, see [`../AGENTS.md`](../AGENTS.md)."
- **H1**: keep `# Chronicler Engine Documentation` (or trim if has sediment — verify)

### Phase 3a — Sediment Sweep (3 SP)

**3.1 `reference/data_schemas.md` (1 SP)**
- DELETE `## Implementation Requirements` (historical PR task — "Modify main.rs... deprecating hardcoded Aethelgard mock data")
- DELETE `## Data Normalization Rules` (old character JSON format guidance)
- DELETE `## Redmist Estate Map Topology` (demo-specific, per Decision 2)
- STRIP `(NEW)` / `(Updated)` markers from section titles — 9 instances
- DROP SQL column-level DDL listings; KEEP JSON examples (per Decision 4)
- H1 sweep: verify no sediment in H1

**3.2 `reference/data_layer.md` (1 SP)**
- DELETE `## Migration Policy` entirely (per Decision 3) — replace with one-line summary: "Migrations run on first access via `run_migrations`; see `src/adapters/driven/storage/db.rs`."
- DELETE `### v14: starting_room_id Relocation` subsection
- DELETE `## Future Work` (per Decision 6)
- DELETE `## Code Mapping` (file-path tree — agents have AGENTS.md STRUCTURE block)
- H1 sweep

**3.3 `reference/persona_system.md` (0.5 SP)**
- DELETE `## Role of "Julian Redmist"` (per Decision 2)
- DELETE `## LLM Integration Changes` (aspirational register)
- DELETE `## Boundaries` (generic)
- H1 sweep

**3.4 `system/worlds.md` (0.3 SP)**
- DELETE `## Future Enhancements` + `## Backlog` (Decision 6)
- DELETE `## Performance Considerations` (tautology)
- DELETE `## Testing Strategy` generic sub-sections
- DELETE `## Error Handling` three identical sub-sections
- H1 sweep

**3.5 `system/game_flow.md` (0.2 SP)**
- STRIP "Streaming Narration Optimization: ... 73% improvement" metric; KEEP "Narration is saved immediately after Phase 4" as present-tense spec
- STRIP generic "Design Principles" bullets
- H1 sweep

**Session 1 End Verification**
- `python scripts/generate_docs_index.py` exits 0
- `python build.py` passes (architecture + guardrails tests)
- `grep -rn "docs/README" .` → 0 hits outside `docs/plans/`
- H1 spot-check on swept files

**Session 1 Archive**: move plan to `docs/plans/archived/` after Session 2 completes. For now, keep plan active.

---

## Session 2 — Code-Indexer Sweep + Final Verification (~6 SP)

### Phase 3b — Code-Indexer Sweep (5 SP)

**3.6 `system/agent_system.md` (0.5 SP)**
- DELETE `## Key Files` table — also fixes duplicate-row bug (`src/application/agents/quantifier/agent.rs` listed twice, lines 73 + 78)
- DELETE `## Adding a New Agent` section — duplicates ADR-009
- STRIP phase anchors: "Current agents (Phase 2)" → "Current agents"; "Future agents (out of scope for Phase 2)" → "Future agents"
- H1 sweep

**3.7 `architecture/system.md` (2 SP)** — biggest payoff
- §11 "Test Binaries" table: drop "Count" column (~221, ~60, ~32 rot every test commit). Keep qualitative purpose only.
- §1 model/state description: trim implementation detail ("MessageHistory which encapsulates Vec<Message> where each Message is an independent narrative unit..."). Keep type-level summary.
- §3 "Driven Adapters" subsection: collapse to one-line pointers to `system/storage.md` / `system/llm_processing.md` / `system/text_check.md`
- DELETE "NPC Event Layer" subsection — duplicates `system/triggers.md` "Times Met Semantics"
- DELETE "Sub-system References" table at end — covered by AUTO-INDEX in `docs/AGENTS.md`
- §7 Storage subsection: collapse to one-line pointer to `system/storage.md` (per Decision 8). If `AGENTS.md` STRUCTURE block already lists `system/storage.md`, prefer full removal of §7.
- H1 sweep

**3.8 `system/dashboard.md` (1 SP)**
- DELETE `### CSS Classes` section (~80 lines of `.foo - bar baz`) — agents have `assets/styles.css`
- DELETE `### Frontend Implementation` subsection — duplicates `system/ui_design.md`
- Worlds Management Tab subsection: collapse to one-line pointer to `system/worlds.md`
- H1 sweep

**3.9 `system/storage.md` (0.3 SP)**
- `## Schema`: drop file-path reference ("Schema is defined in `src/adapters/driven/storage/db.rs`"). Keep table-category summary.
- `## Testing Strategy`: drop file-path references. Keep `InMemory` / `with_test_failures` description.
- H1 sweep

**3.10 `system/text_check.md` (0.3 SP)**
- DELETE `## Module Structure` table (file paths)
- H1 sweep

**3.11 `system/prompt_system.md` (0.5 SP)**
- DELETE `## Implementation` / `### Key Files` subsection — code-indexer pattern
- DELETE `## Differences from SillyTavern` comparison table — permanent context, not actionable
- COLLAPSE `## Other Prompt Systems` / `### Quantifier Prompt (Separate)` to one-line pointer to `reference/quantifier_prompt.md`
- H1 sweep

**3.12 `system/ui_design.md` (0.5 SP)**
- KEEP `## Design Tokens` tables
- DELETE `## Implementation` / `### CSS Custom Properties` — duplicates `assets/styles.css`
- DELETE `## JavaScript Features` — duplicates `system/dashboard.md` Button Logic
- H1 sweep

**3.13 `system/character_state.md` (0.2 SP)**
- DELETE `## Rationale: Why Track Persistence?` (single Example block, no real rationale)
- H1 sweep

**3.14 `system/navigation.md` (0.2 SP)**
- COLLAPSE `### Example Flows` three bullets to one (same pattern repeated)
- H1 sweep

**3.15 `diagnostics/DEBUGGING.md` (0.3 SP)**
- DELETE `## Diagnosis Workflow` Steps 1-4 — generic debugging advice
- KEEP `## LLM Call Forensics (SQLite)` + `## Using Tracing`
- H1 sweep

**Cross-doc link fixes during Phase 3b**: Fix any links broken by deletions/collapses (e.g., if `system/storage.md` was linked from `architecture/system.md` §7). Do NOT fix links in `docs/plans/` per Decision 9.

### Phase 4 — Final Verification (0.5 SP)

1. `python scripts/generate_docs_index.py` — exits 0, AUTO-INDEX valid in `docs/AGENTS.md`, includes renamed AGENTS.md self
2. `python build.py` — verify no doc-anchor-dependent tests break
3. `grep -rn "docs/README" .` → 0 hits (excluding `docs/plans/archived/`)
4. Verify all internal markdown links still resolve in swept files (excluding `docs/plans/`)
5. Open `docs/AGENTS.md` — verify AUTO-INDEX renders correctly
6. H1 spot-check on 5 swept files

### Phase 5 — Archive + Changelog (0.5 SP)

1. Move this plan to `docs/plans/archived/`
2. Add `CHANGELOG.md` entry under Unreleased:
   ```
   ### Changed
   - Renamed `docs/README.md` → `docs/AGENTS.md` to align with AGENTS.md repo convention (agents auto-load AGENTS.md, not README.md).
   - Updated `scripts/generate_docs_index.py` output target to `docs/AGENTS.md`.
   
   ### Removed
   - Code-indexer sediment from ~15 doc files: "Key Files" tables, CSS class enumerations, SQL DDL duplication, `## Code Mapping` trees, file-path bullets. Agents resolve paths via `chronicler_engine/AGENTS.md` STRUCTURE block.
   - Speculation sections: `Future Work`, `Backlog`, `Future Enhancements` across docs (covered by `ROADMAP.md`).
   - Historical PR commentary: `(NEW)`/`(Updated)` section markers, phase tracker `[x]`/`[~]` markers, v14 BREAKING notices, PR performance metric sediment (~73% improvement).
   - Demo-specific content (Redmist Estate topology, Julian Redmist persona backstory, Aethelgard mock reference). Data files are source of truth.
   - "Quick Reference" table + "Key Principles" + "Workflow" sections from `docs/AGENTS.md` (duplicated by `chronicler_engine/AGENTS.md` + AUTO-INDEX).
   - `architecture/system.md` §11 "Count" column from Test Binaries table — rots on every test commit.
   ```

## Test Plan

Docs-only; no production code touched.

- **Generator sanity**: `python scripts/generate_docs_index.py` exits 0, produces valid AUTO-INDEX in `docs/AGENTS.md`
- **Build integrity**: `python build.py` passes (architecture + guardrails tests confirm no doc-anchor breakage)
- **Link integrity**: `grep -rn "docs/README" .` → 0 hits (excluding `docs/plans/archived/`)
- **Anchor integrity**: Module-level `//! [DOC: docs/...]` anchors in Rust source still resolve. No Rust files reference `docs/README.md` directly (verify in Phase 1).
- **Spot-check**: 5 swept sections confirmed sediment (no unique info lost). SKIP `## Migration Policy` check — Decision 3 is explicit deletion; agents read `db.rs`.
- **H1 sweep**: H1s on all swept files confirmed clean post-sweep.

## Delegation Recommendation

- **Phase 1+2** (Session 1): primary agent (orchestrator). Sensitive rename + content judgment.
- **Phase 3a** (Session 1): `worker` subagent (mid-tier). Clear deletion rules. Primary reviews output + runs `build.py` + verifies H1s.
- **Phase 3b** (Session 2): `worker` subagent (mid-tier). Careful review of `architecture/system.md` §11 + Driven Adapters collapse. Primary verifies.
- **Phase 4+5** (Session 2): `delegate` subagent (low-tier). Script execution + mechanical verification + changelog entry.

## Assumptions

1. `docs/README.md` rename is safe. GitHub auto-renders `README.md` only at repo root, not subfolders. No external system depends on subfolder `README.md` filename. Will verify with repo-wide grep in Phase 1.
2. `chronicler_engine/AGENTS.md` is NOT touched. Content from `docs/README.md` that duplicates engine AGENTS.md gets deleted, not migrated.
3. AUTO-INDEX markers (`<!-- AUTO-INDEX START -->` / `<!-- AUTO-INDEX END -->`) are filename-agnostic. Only output path in `generate_docs_index.py` changes.
4. Demo-specific content (Redmist topology, Julian backstory, Aethelgard reference) gets DELETED, not migrated. Data files in `data/worlds/redmist_estate/` + `data/personas/julian.json` are the source of truth.
5. JSON examples in `data_schemas.md` are KEPT (data-author-facing). SQL DDL is dropped.
6. `## Migration Policy` in `data_layer.md` is pure sediment per Decision 3. Migration sequence invariants live in `src/adapters/driven/storage/db.rs` source. Deletion is final; do not second-guess.
7. Referenced file paths in docs (e.g., "see `src/application/agents/quantifier/agent.rs`") may be wrong. Sweep DELETES these references, doesn't update them. If path is broken, deletion removes broken reference (acceptable).
8. No CSP / CI doc-lint rule greps for `docs/README.md` specifically. Verify in Phase 1.
9. `chronicler_engine/AGENTS.md` STRUCTURE block (auto-generated by `generate_structure_index.py`) is NOT touched. Regenerated separately.
10. Cross-doc link fixes happen during Phase 3b for swept files only. `docs/plans/` link fixes are out of scope.
11. Latitude to fix similar sediment inline during sweep (Decision 7). Uncertain cases surfaced at end of session.

## Execution Rules

- **Plan Adherence**: STOP-and-report per `AGENTS.md` if scope grows beyond listed files OR if a deletion unexpectedly contains unique load-bearing information.
- **Caveman communication**: All commit messages, changelog entries, summaries stay technical-prose. No fluff.
- **Concurrency**: Primary agent uses `target/` (or `target/agent2` if parallel worker needs isolated build).
