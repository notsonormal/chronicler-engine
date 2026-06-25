# Plan: Doc Drift Linter (Stale Identifier Detection in docs/)

**Date:** 2026-06-25
**Status:** Planned
**Goal:** Catch stale code identifiers (function names, struct fields, CLI flags, paths) in `docs/` before they mislead agents or humans.

---

## Overview

Doc drift is a recurring pain. Handlers get renamed, struct fields removed, CLI flags
restructured — and docs/ markdown keeps referencing the old names. This causes:

- Agents following docs that point at nonexistent symbols → wrong-file investigations.
- After-plan-workflow Step 3 (docs consistency) becomes manual grep cleanup, error-prone.
- ADRs referencing historical (pre-decision) names intentionally, which is fine but noise.

This plan adds a Python linter that extracts code identifiers from `docs/` markdown
and verifies they still exist in `src/`.

---

## Background

**Problem source:** Pain 5 from the docs-consistency retro. Identifiers in docs drift
from source. Manual grep cleanup is error-prone and was the entire content of
after-plan-workflow Step 3.

**Existing related work:**

- `chronicler-docs-consistency` skill — agent-driven semantic drift review, slow, broad.
- This plan: narrow, mechanical, fast, automated. Complements the skill, does not replace it.

---

## Scope

### In scope (Phase 1)

- Fenced code blocks only (```` ``` ````), not inline backtick spans. Inline backticks
  have too much prose-noise (false positive risk HIGH).
- Identifiers extracted from fenced code:
  - `snake_case_fn_names`
  - `struct_field_names` (in `struct Foo { field: ... }` shape)
  - CLI flags (lines starting with `--` or `-x`)
  - `src/` path references (`src/engine/foo.rs`)
- Check existence against `src/` tree (grep for identifier, check path exists).
- Report unknowns as warnings (not errors).

### Out of scope (Phase 1)

- Inline backtick spans.
- External crates' identifiers (need allowlist or detect via `use`/`extern`).
- Prose mentions ("the handler does X" without code fence).
- Type names (CamelCase) — too many collide with external crates.

### Special handling

- `docs/adr/*.md` excluded from Phase 1 scan. ADRs intentionally reference historical
  names pre-decision. Separate "current state" pass possible later.
- Allowlist file: `docs/plans/doc-drift-allowlist.txt` for intentional references
  (e.g., planned-but-not-yet-implemented identifiers with `// TODO` or "will add"
  context in the doc itself). Lower maintenance burden if allowlist stays small.

---

## Architecture Decisions

1. **Python, not Rust.** Runs in `scripts/`, lives next to `vault_to_single_file.py` and
   `issue_tracker/`. No compile, easy to extend. Matches project convention (Python for
   automation).
2. **Fenced code blocks only in Phase 1.** False positive risk is HIGH otherwise.
   Narrow scope first, expand if value proves out.
3. **Warning, not error.** Does not break `build.py`. Agent reviews output.
4. **Allowlist over suppression.** Maintain a single allowlist file rather than inline
   `<!-- lint-ignore -->` markers — keeps docs readable and allowlist greppable.
5. **Grep src/, don't parse Rust.** No syn / rust-analyzer dependency. Substring grep
   gives ~5% false negatives (identifier used only in macro expansion) but zero false
   positives from parsing quirks. Acceptable for warning-level lint.
6. **ADRs excluded by default.** Historical references are intentional. Re-scan
   separately later if "current state" vs "history" split is needed.

---

## Phase 1: Implementation

### Task 1.1: Identifier Extractor

- Markdown parser (use `mistune` or `markdown-it-py`, already common in Python).
- Walk AST, collect:
  - Code block content blocks
  - Parse each code block with lightweight regex for identifiers above
- **Deliverable:** `scripts/doc_drift_linter/extract.py` → list of `(file, line, identifier, kind)`.

### Task 1.2: Existence Checker

- For each identifier:
  - `snake_case_fn` → `rg "\bfn {id}\b"` in `src/`
  - `struct field` → `rg "\b{id}:"` in `src/` (heuristic)
  - `CLI flag` → check `clap` derive struct fields or arg strings in `src/`
  - `src/ path` → `os.path.exists`
- Apply allowlist filter before reporting.
- **Deliverable:** `scripts/doc_drift_linter/check.py` → list of unknowns with location.

### Task 1.3: CLI + Report Format

- `python -m scripts.doc_drift_linter` → exits 0 always (warning-only Phase 1).
- Output format:

  ```
  WARNING: docs/architecture/foo.md:42
    identifier `render_narration` (snake_case_fn) not found in src/
    context: ```rust\n  render_narration(ctx);\n```
  ```

- `--allowlist path` flag (default `docs/plans/doc-drift-allowlist.txt`).
- `--exclude-adr` flag (default True).
- **Deliverable:** `scripts/doc_drift_linter/__main__.py`.

### Task 1.4: Allowlist Bootstrap

- Run lint, collect current unknowns.
- Triage each: real drift (fix doc) vs intentional (add to allowlist with one-line reason).
- **Deliverable:** `docs/plans/doc-drift-allowlist.txt` seeded with intentional entries.

### Task 1.5: Skill + Workflow Integration

- Add step to `chronicler-after-plan-workflow` skill: "Run doc-drift linter, fix real
  drift, allowlist intentional references."
- Document in `docs/agents/` if applicable.
- **Deliverable:** Updated skill file.

---

## Phase 2 (Deferred — Wait for Phase 1 Value Proof)

- Inline backtick spans (with allowlist grow).
- External crate identifier detection (via `use`/`extern` parsing).
- Type-name (CamelCase) checks.
- ADR "current state" re-scan (extract terminal-decision identifiers only).
- CI integration (hook into pre-commit).

---

## Success Criteria

1. Linter runs on full `docs/` tree in < 5 seconds.
2. Zero unexplained unknowns after Phase 1.4 bootstrap (real drift fixed, intentional
   allowlisted).
3. Warning-level only does not break `python build.py`.
4. After-plan-workflow Step 3 ("update docs") references the linter by name.

---

## Risks

- **False positive noise** → falls back to allowlist. Mitigated by fenced-code-only scope.
- **Allowlist rot** → allowlist grows stale. Mitigation: review allowlist quarterly, each
  entry must have one-line `# reason:` comment.
- **Grep substring collisions** → `render` finds `render_foo`. Mitigated by word-boundary
  regex `\b{id}\b`.
- **Macro-hidden identifiers** → false negatives. Acceptable at warning level.

---

## Open Questions

1. Should the linter also scan `README.md` at repo root and `AGENTS.md`? (Lean: yes, same
   drift risk. Confirm before Phase 1.4.)
2. Allowlist format: one identifier per line, or grouped by file?
3. Should `# reason:` comments be required for every allowlist entry? (Lean: yes, prevents
   silent allowlist rot.)
