# T11: Documentation Hygiene Skill Hardening

## Summary

The `chronicler-docs-hygiene` skill is read-only and misses 5 categories of issues that have accumulated across 31 docs (stale cross-doc layer counts, ghost schema fields, code-indexer field bullets, implementation summaries masquerading as specs, anchor density). The skill also redefines canonical terms (`Sediment`, `Duplication`) that already live in `docs/AGENTS.md` — drift risk.

This plan hardens the skill to catch those 5 categories, adds a §Writing Guide section so it triggers on doc authoring (not just auditing), and slims `docs/AGENTS.md` so operational definitions live in one place (the skill). After skill lands, audit all 31 docs and apply findings.

## Key Changes

1. **Skill rewrite:** description triggers on write + audit; §Writing Guide added (positive recipe); §Spec-vs-Summary rule added; Phase 4 (Code-Indexer) tightened with enumeration types; Phase 5 (Ghost Features) tightened with schema-claim subtype; Phase 7 (Cross-Doc Drift) added; completion criteria tightened.
2. **AGENTS.md slim:** §"Keeping Documentation Clean" rewritten to drop Sediment + Duplication operational definitions. Single-sentence pointer to skill. Rest of AGENTS.md untouched.
3. **Audit pass:** run updated skill on all 31 docs; capture findings.
4. **Apply findings:** mechanical fixes + Category D rewrites (Two State Channels, Quantifier Forensics Gap, Post-History Is Not a Layer Variant) using new Spec-vs-Summary rule + other audit items surfaced by Phase 2.
5. **REFACTOR loop:** re-run skill, close loopholes.

## Decisions Locked

| # | Question | Decision | Rationale |
|---|----------|----------|-----------|
| D1 | Anchor budget per section | Max 3 anchors per section | Matches Spec-vs-Summary rewrite threshold. Permissive enough for genuinely-complex sections (e.g. cancellation sites). |
| D2 | Anchor format | `Module::symbol` preferred; `path/file.rs` allowed if Data-Flow Claim present; bare line refs forbidden | Quote-phrase anchor convention (`docs/AGENTS.md`) forbids line numbers. Module::symbol survives renames better than file:line. |
| D3 | Skill name | Keep `chronicler-docs-hygiene` | Less churn. Description change signals dual purpose. |
| D4 | AGENTS.md slim scope | Rewrite §"Keeping Documentation Clean" only; rest untouched | Minimal surgery. User explicitly said "remove entirely as suggested" for the operational defs. |

## Implementation

### Phase 1: Skill update + AGENTS.md slim (5 SP)

- [ ] #### Task 1.1: Skill description update (1 SP)
  - Old: "Read-only audit of chronicler_engine/docs/ Specification pages. Flags Sediment, mechanics leakage, code-indexer drift, tone rot. Reports findings, never edits."
  - New: "Use when writing or auditing chronicler_engine specs against code for stale references, ghost schemas, contract-vs-implementation-summary drift, and cross-doc layer/count mismatches. Read-only on docs; reports findings, never edits."
- [ ] #### Task 1.2: Add §Writing Guide section (1 SP)
  - **§Voice** — declarative present-tense; avoid future/past/hedge.
  - **§Shape** — lead with contract statement. ≤1 function/symbol anchor per claim. Max 3 anchors per section. Anchor format: `Module::symbol` preferred.
  - **§Spec-vs-Summary test** — if reader could delete section without losing any contract info, it's summary, not spec. Rewrite as contract + 1 anchor, OR move call-chain to code as `//! [DOC: ...]` comment.
  - **§Tables and bullets** — table cells with function/symbol names count as anchors. Field bullets (`**foo**: desc`) count as anchors regardless of count.
  - **§Cross-references** — when doc references layer/phase/struct count or enum name, link the canonical source. Don't restate the count.
- [ ] #### Task 1.3: §Spec-vs-Summary rule in Philosophy (1 SP)
  - Single paragraph added to existing Philosophy section. Text:
    > A spec states the contract — what the system guarantees. An implementation summary describes how the code works. Code references in prose are allowed only when justified by a Data-Flow Claim (input → module → output). A section whose substance is "here is the call chain for X" is implementation summary, not spec. A section with >3 anchors per claim, OR one that could be deleted without losing any contract info, is implementation summary. Rewrite or move to `//! [DOC: ...]` comments.
- [ ] #### Task 1.4: Tighten Phase 4 Code-Indexer (1 SP)
  - Add enumeration-type coverage: bulleted lists where bullets name modules/structs/fields; tables with function/symbol cells; field bullets in prose; mermaid diagrams with function-name node labels.
- [ ] #### Task 1.5: Tighten Phase 5 Ghost Features — schema-claim subtype (1 SP)
  - Add: schema claims = doc asserts struct has fields, JSON example lists field names, function signature shown. Verify each field/signature exists in live `src/` definition. JSON examples listing fields NOT on struct = ghost-schema claim.
- [ ] #### Task 1.6: Add Phase 7 Cross-Doc Drift (1 SP)
  - Conditional phase: when ≥2 STANDARD docs reference same concept (layer count, enum name, struct field, phase number), verify all agree. Report drift with both file refs. Skip otherwise.

Note: Tasks 1.1–1.6 are interdependent (skill is one file). Sequenced as one edit. SP per task = complexity, not file-write count.

- [ ] #### Task 1.7: Tighten phase completion criteria (≤1 SP, batch with 1.6)
  - Append to each phase's end: "Zero findings = phase complete. Do not skip remaining phases because this phase was clean."
- [ ] #### Task 1.8: Remove Sediment + Duplication definitions from docs/AGENTS.md (1 SP)
  - Current §"Keeping Documentation Clean" has full operational definitions. Replace with:
    ```
    ## Keeping Documentation Clean

    **Plan authoring convention:** reference doc issues by quotable phrase, never line numbers — line numbers rot. Use the exact sentence (or a quoted fragment of it) as the anchor.

    Operational criteria for "what counts as Sediment / Duplication" live in the [`chronicler-docs-hygiene`](../../.agents/skills/chronicler-docs-hygiene/SKILL.md) skill.
    ```
  - Removes ~10 lines of operational definition text from AGENTS.md.
- [ ] #### Task 1.9: Update skill scope section (≤1 SP, batch with 1.8)
  - Old scope lists `docs/system/*.md` etc. explicitly.
  - New: "STANDARD docs: all of `docs/` excluding `plans/`, `adr/`, `architecture/invariants.md`, and the auto-generated index. Canonical taxonomy: `validate_docs.py` STANDARD_DIR_NAMES."

### Phase 2: Run updated skill on all 31 docs (2 SP)

- [ ] #### Task 2.1: Audit pass (2 SP)
  - Run skill on all 31 docs (AGENTS.md, CHANGELOG.md, architecture/{guardrails,system}.md, diagnostics/{DEBUGGING,error_catalog}.md, external_applications/*.md, reference/*.md, system/*.md).
  - Exclude plans/ and adr/ per current scope.
  - Output: list of (file:line, phase, severity, current, expected).
  - No pre-emptive fixes. Capture findings only.
  - Deliverable: findings list embedded in commit message + saved to scratch for Phase 3 reference.

### Phase 3: Apply findings + Category D rewrites + other audit items (14 SP)

Findings-driven. Subtask structure locked; content fills from Phase 2 output.

- [ ] #### Task 3.1: Apply mechanical findings from Phase 2 (3 SP)
  - Mechanical edits: 1-anchor rewrites, cross-doc reference fixes, ghost-schema claim removals, tense/voice fixes.
  - Subtasks created as findings land (target: 5-10 individual edits).
  - Verification: validate_docs.py after each batch.
- [ ] #### Task 3.2: Category D rewrites using Spec-vs-Summary rule (5 SP)
  - Apply new rule to the 3 code-density-heavy sections added in t9-doc-quickwins pass:
  - [ ] ##### SubTask 3.2.1: docs/system/game_flow.md §"Two State Channels" (1 SP)
    - Reduce to contract prose. 1-anchor-per-claim. ≤3 anchors per section. Apply new anchor format.
  - [ ] ##### SubTask 3.2.2: docs/system/agent_system.md §"Quantifier Forensics Gap" (2 SP)
    - Same: contract prose. Move call-chain to `src/application/agents/quantifier/agent.rs` as `//! [DOC: ...]` comment if needed.
  - [ ] ##### SubTask 3.2.3: docs/system/prompt_system.md §"Post-History Is Not a Layer Variant" (2 SP)
    - Same. The assembler.rs LayerRenderer mechanics can move to `//! [DOC: ...]` if too dense.
- [ ] #### Task 3.3: Other audit items — apply Spec-vs-Summary rule to remaining dense sections (3 SP)
  - Search 31 docs for sections with >3 anchors per claim.
  - Rewrite each. Subtasks created as items land.
  - Includes (but not limited to): docs/system/action_pipeline.md §Components table (worker-written, ~16 rows), any other high-density sections surfaced by Phase 2.
- [ ] #### Task 3.4: REFACTOR — re-run skill, close loopholes (3 SP)
  - After Phase 3.1-3.3 land, re-run skill on all 31 docs.
  - Expect 0-3 new findings as the tightened skill catches things.
  - Apply fixes. Re-run until clean (max 3 iterations).

### Phase 4: Verify (1 SP)

- [ ] #### Task 4.1: Final verification (1 SP)
  - `python scripts/validate_docs.py` → 0 errors, 0 warnings.
  - `python build.py` → full build green (fmt, clippy, tests).
  - Spot-check 5 random docs with skill as regression test.

## Test Plan

| Test | Verification |
|------|--------------|
| Skill catches cross-doc drift | Phase 2 audit surfaces ≥1 stale layer-count reference (audit already found 5: llm_processing, narration_engine, triggers, quantifier_prompt, system_prompt) |
| Skill catches ghost-schema claims | Phase 2 surfaces data_schemas.md Message Schema direct-field claims |
| Skill catches code-indexer field bullets | Phase 2 surfaces ≥1 field-bullet enumeration (architecture/system.md §1 had 11 bullets) |
| Skill catches implementation summaries | Phase 2 surfaces ≥1 implementation summary (Category D sections) |
| Anchor budget rule works | Phase 3.2 rewrites drop each Category D section below 3 anchors per claim |
| AGENTS.md slim | AGENTS.md no longer contains operational definitions of Sediment/Duplication |
| Build green | `python build.py` exits 0 with all tests passing |
| validate_docs clean | 0 errors, 0 warnings across 72 docs |
| Regression | 5 random docs re-audited; no false positives introduced |

## Assumptions

- A1: User accepted Phase 1 sequencing (skill first, then audit, then fixes). Skill first lets Phase 2 reflect actual skill behavior, not predicted behavior.
- A2: User accepted that Category D rewrites (Phase 3.2) use the new Spec-vs-Summary rule from Phase 1.3, applied post-skill.
- A3: User accepted that other audit items (Phase 3.3) emerge from Phase 2 findings. The mermaid phase-numbering in game_flow.md and Trigger Schema `name` field are candidate items but not yet committed to specific fixes.
- A4: "Anchor" in D1/D2 means function/symbol reference in prose. Field bullets count as anchors. Plain text links to other docs do not.
- A5: `python scripts/validate_docs.py` is the canonical mechanical validator. Phase 4.1 relies on it.
- A6: Skills are loaded by name via description match. Description change in Task 1.1 makes the skill trigger on doc-writing contexts (not just auditing). No extension config changes needed.
- A7: Refactoring the skill body does not require breaking changes to its 6-phase structure. Existing phases get tightened; new content (Writing Guide, Spec-vs-Summary, Phase 7) is additive.
- A8: Implementation runs in current session (no separate context needed for skill edits). User indicated "Probably just do it without subagents until you want separate context" (Q9).
