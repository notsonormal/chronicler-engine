# Plan: Harden the chronicler-docs-hygiene skill

**Date:** 2026-08-12  
**Status:** Planned  
**Goal:** Update the `chronicler-docs-hygiene` skill so it triggers during doc authoring and audits more reliably, without changing AGENTS.md operational definitions (already slimmed).

## Background
`docs/AGENTS.md` already no longer defines `Sediment` / `Duplication` operationally; the skill is the canonical place for those definitions. However:

- The skill description is read-only audit only.
- There is no explicit "Writing Guide" or "Spec-vs-Summary" section with those names.
- There is no cross-doc drift phase.

## Scope

### Task 1: Update skill description and add Writing Guide
- In `.agents/skills/chronicler-docs-hygiene/SKILL.md`:
  - Change description to mention both writing and auditing.
  - Add a `## Writing Guide` section with positive recipes: declarative voice, lead-with-contract, anchor budget (max 3 per section), preferred anchor format (`Module::symbol`), and cross-reference rules.

### Task 2: Add Spec-vs-Summary rule
- Add a `### Spec-vs-Summary` subsection under Philosophy:
  - A spec states a contract.
  - An implementation summary describes code.
  - If removing a section loses no contract info, it is summary and should move to `//! [DOC: ...]` code comments or be rewritten.
  - Tables/bullets naming fields/symbols count as anchors and are subject to the anchor budget.

### Task 3: Add cross-doc drift phase
- Add a `Phase 7 — Cross-Doc Drift` phase:
  - When two or more docs reference the same concept (layer count, enum variant set, phase number, struct field, INV-NNN), verify they agree.
  - Report drift with both file refs.
  - Skip if no repeated concepts.

### Task 4: Audit pass with the updated skill
- Run the updated skill on all `docs/diataxis/**/*.md`.
- Capture findings in `tmp/docs-hygiene-findings.md`.
- Do not edit docs in this task; only collect findings for a follow-up cleanup plan.

## Out of scope
- Editing `docs/AGENTS.md` (already slimmed).
- Fixing findings from the audit (will spawn a follow-up plan).
- Adding new lints to `validate_docs.py`.

## Acceptance criteria

- Skill file has updated description, Writing Guide, Spec-vs-Summary rule, and Phase 7.
- Running the skill on `docs/diataxis/` completes without crashing.
- `tmp/docs-hygiene-findings.md` is produced.
- No `src/` code changes.

## Verification

- `python scripts/validate_docs.py` still passes.
- `python build.py` still passes.
