# Plan: Documentation Hygiene Skill

**Date:** 2026-07-03
**Status:** Planned
**Goal:** Create a project-scoped agent skill (`chronicler-docs-hygiene`) that performs LLM semantic analysis on `docs/**/*.md` to preserve documentation as a specification of static target state, free of conversational sediment, code-indexer drift, and tone rot.

**Investigation source:** Gemini conversation on Matt Pocock's "document sediment" framework (https://gemini.google.com/share/fO2dZ4HRJPRG), cross-referenced with existing `.agents/skills/domain-modeling/SKILL.md`, `chronicler_engine/CONTEXT.md`, and sediment survey of `docs/system/*.md` (2026-07-03).

---

## Overview

Chronicler Engine documentation is rotting under AI-assisted edits. AI tools append PR commentary, validation metrics, historical obituaries of deleted code, and brittle function-name maps. Over time, spec files degrade into a hybrid of change log + conversation + spec. Root cause: no enforcement layer forces AI to be an aggressive editor rather than a polite adder.

This plan introduces two **decoupled** artifacts:

1. **Skill** (`.agents/skills/chronicler-docs-hygiene/SKILL.md`) — LLM semantic analysis of markdown prose. Report-only. Recommends 1-3 fixes per finding. Auto-triggered by pi when editing docs.
2. **Script** (`chronicler_engine/scripts/validate_docs.py`) — Deterministic checks (broken markdown links, broken ADR# references). Independent of skill, iterated separately. Hard-stop in CI / pre-commit.

Script and skill are **not** dependent on each other. Skill does not invoke script. Script does not invoke skill. This keeps token cost on doc edits minimal (skill loads) and keeps deterministic checks fast (no LLM call).

---

## Background

**Existing tooling precedent:**

- `chronicler_engine/scripts/validate_adrs.py` (406 lines) — enforces ADR structure (REQUIRED_SECTIONS, FORBIDDEN_SECTIONS, status/date format, inline version drift). Uses error/warning/grandfather pattern. Mirrors this skill's intended script approach.
- `.agents/skills/chronicler-comment-fixer/SKILL.md` — detects AI slop in Rust `//!`/`///` and Python `#` comments. Has `scripts/comment_finder.py`. Report-only. Direct sibling pattern.
- `.agents/skills/chronicler-docs-consistency/SKILL.md` — detect docs-vs-code drift. To be folded into the new skill (see Task 3).
- `.agents/skills/domain-modeling/SKILL.md` — owns `CONTEXT.md` glossary. New skill cross-references but does not redefine terms.

**Sediment survey findings (2026-07-03, `docs/system/*.md`):**

| Pattern | Example | Files hit |
|---------|---------|-----------|
| Past-tense deletion + date/phase | `"removed on 2026-07-03"`, `"deleted in Phase 2.1"` | `llm_processing.md:97, 165` |
| "Never wired / proposed in plan" | `"ForensicsCollector... never wired into the test harness"` | `llm_processing.md:165` |
| External forum links | Reddit/StackOverflow | `llm_processing.md:87` |
| PR-style validation metrics | `"Reduced completion tokens from 2048 to ~211"` | `llm_processing.md:85` |
| Code-indexer sections | "Instrumented Functions", "Module Location" with bulk `src/*.rs` + `fn()` listings | `llm_processing.md` §10, `invariants.md` (whole file) |
| Future-tense stub markers | `"Stub — not yet implemented"` | `llm_processing.md:31` (decided: NO skill rule, per Q4 in grilling) |

**Conceptual vocabulary ( Leading Words ):**

- **Specification** — declarative present-tense contract. The form `docs/system/*.md` should take. Reserved sense.
- **Invariant** — reserved for `docs/architecture/invariants.md` machine-checked INV-NNN entries with contract tests. New skill does NOT use this word for prose laws (avoid clash with existing precise term).
- **Sediment** — conversational clutter, PR commentary, timeline logs, validation metrics, tracking of "what used to exist" or "was removed".
- **Time Capsule** — ADR file. Only acceptable place for narrative context, debugging paths, model quirks. ADRs are out of scope for this skill — `validate_adrs.py` owns their hygiene.

---

## Scope

**Skill in-scope paths:**

- `chronicler_engine/docs/system/*.md`
- `chronicler_engine/docs/architecture/*.md` (except `invariants.md` — registry, exempt)
- `chronicler_engine/docs/reference/*.md`
- `chronicler_engine/docs/diagnostics/*.md`
- `chronicler_engine/docs/plans/*.md` (active only; `archived/` excluded via `old-docs/` move already done)

**Skill out-of-scope:**

- `docs/adr/` — ADRs are Time Capsules; `validate_adrs.py` owns structure. Past tense + history allowed there.
- `docs/external_applications/` — external conventions, not engine specs.
- `old-docs/` — already isolated historical archive.
- `CHANGELOG.md`, `ROADMAP.md` — inherently temporal by design.
- `invariants.md` — registry, not prose. Codified as exempt (decision: Q3 in grilling — `invariants.md` itself is sediment by the skill's logic; skill flags the file's existence as a code-indexer pattern candidate for deletion, but per-file deletion is a separate architectural decision).
- Rust `//!` / `///` and Python `#` comments — owned by `chronicler-comment-fixer`.

**Script scope** (independent, separate iteration):

- `chronicler_engine/docs/**/*.md` (excl. `old-docs/`, `external_applications/`)
- Rules: broken markdown link resolution, broken ADR# references. More rules added over time as sediment patterns crystallize in code form.

---

## Skill Responsibilities (folded + new)

| Responsibility | Source | Action |
|----------------|--------|--------|
| Sediment audit (no-op test, sediment keywords, PR metrics, forum links) | New | LLM judgment |
| Mechanics eviction (private fn names in `system/` body prose, bulk listings) | New | LLM judgment |
| Tone correction (present tense, declarative) | New | LLM judgment |
| Code-indexer pattern detection (sections enumerating fn/file lists without data-flow justification; flag `invariants.md` as candidate) | New | LLM judgment |
| Ghost Features (doc asserts feature as existing, code lacks it, no explicit stub/planned marker) | Folded from `chronicler-docs-consistency` | LLM judgment |
| Behavior Mismatch (doc claims behavior X, code does Y) | Folded from `chronicler-docs-consistency` | LLM judgment |

**Dropped from `chronicler-docs-consistency` (low value or redundant):**

- Missing Concepts — subjective, high false-positive rate.
- Outdated Patterns — redundant with Sediment / Mechanics Eviction.
- Wrong Signatures — brittle; if sigs appear in `docs/system/`, that itself is the finding (Mechanics Eviction). Code is source of truth.
- Broken References / ADR# resolution — moved to `validate_docs.py` (deterministic).

**Explicitly NOT in skill:**

- File-existence verification for `src/...` path mentions (script's job if done at all).
- File-path verification for individual illustrative single-mention paths (allowed; only bulk listings flagged).
- Auto-creating ADRs to capture extracted sediment. Skill **flags** candidate ADR extractions; user/`documentation-and-adrs` skill handles actual ADR creation. Per AGENTS.md plan-adherence rule.

---

## Execution Model

**Report-only.** Skill does not edit files. Matches `chronicler-comment-fixer` / `chronicler-docs-consistency` precedent. Honors AGENTS.md plan-adherence rule (61% drift failure rate).

**Per finding:**

- File:line reference
- Severity (Error / Warning / Info)
- Current vs Expected (snippet)
- 1-3 recommendations (no more than 3 — forces prioritization, prevents slop)

**Status line:** `Status: [PASS] or [FAIL]`

**Output format:** Markdown report, similar to `chronicler-comment-fixer` / `chronicler-docs-consistency`.

**Trigger:** `disable-model-invocation` omitted → pi auto-triggers when task semantically matches doc writing/updating. Not path-scoped (pi triggers on description match, not file path). Skill description must be explicit about doc-edit scope.

---

## Tasks

### Task 1: Draft `SKILL.md`

**File:** `.agents/skills/chronicler-docs-hygiene/SKILL.md` (new)

**Story points:** 3

**Content:**

- Frontmatter: `name`, `description` (explicit scope: `docs/system/`, `docs/architecture/` excl. `invariants.md`, `docs/reference/`, `docs/diagnostics/`, `docs/plans/`)
- Philosophy section (one paragraph: Specification, not conversation; predictability; reasoning density)
- Leading Words section (Specification, Sediment, Time Capsule; explicitly note **Invariant is reserved**, do not use)
- Information Hierarchy & Boundaries section (scope paths + exclusions)
- Execution Steps (6 phases per Responsibilities table)
- Output Format section (Status, Findings, Recommendations)
- Related Skills section (cross-ref `chronicler-comment-fixer`, `domain-modeling`, `documentation-and-adrs`; explicit "Replacing: chronicler-docs-consistency")
- "This skill is read-only. Do NOT edit, delete, or rewrite files. Report findings only." line near top.

**Target length:** ~100 lines.

**Acceptance criteria:**

- [ ] All 6 phases from Responsibilities table present.
- [ ] Scope paths explicit and match this plan.
- [ ] Read-only constraint stated.
- [ ] Cross-references to sibling skills present.
- [ ] Replacing-note for `chronicler-docs-consistency` present.
- [ ] Total length < 130 lines.

---

### Task 2: Draft `validate_docs.py` (skeleton)

**File:** `chronicler_engine/scripts/validate_docs.py` (new)

**Story points:** 3

**Content:**

Mirror `validate_adrs.py` structure:

- `Violation` NamedTuple `(severity, rule, message)`
- `Violation` severities: `error` (fails build), `warning` (does not fail)
- CLI: `--strict` (all errors), `--list` (violations, no pass/fail), `--path <file>`, default = errors fail
- Default scope: `chronicler_engine/docs/**/*.md`, excluding `old-docs/`, `external_applications/`, `adr/`
- Modes:
  - `--links` — broken markdown link resolution
  - `--adr-refs` — broken ADR# references (e.g., `ADR-099` where `adr-099-*.md` doesn't exist)
  - default (no flag) = all modes
- Exit code: 1 on errors, 0 on clean
- Output: per-file violation list, summary line at bottom

**Initial rules only (do not over-engineer before iteration):**

1. `BROKEN_MARKDOWN_LINK` — `[text](relative/path.md)` where target file doesn't exist. Skip http links.
2. `BROKEN_ADR_REF` — `ADR-NNN` mention where `docs/adr/adr-NNN-*.md` doesn't exist.

**Explicitly NOT in initial script (deferred, iterate later):**

- Sediment keyword detection (regex-based — too blunt, false positives).
- Density / sprawl metric.
- Code-indexer pattern detection (requires LLM).
- Deprecated term usage (per Q5 in grilling, not imperative yet).

**Acceptance criteria:**

- [ ] Script runs in <1 second on current `docs/`.
- [ ] Both rules implemented and tested against seeded broken links.
- [ ] Exit codes correct (1 on error, 0 on clean).
- [ ] Default scope excludes `adr/`, `old-docs/`, `external_applications/`.
- [ ] Mirrors `validate_adrs.py` code style and structure.

---

### Task 3: Fold + delete `chronicler-docs-consistency`

**File:** `.agents/skills/chronicler-docs-consistency/SKILL.md` (delete)

**Story points:** 2

**Pre-check:** `grep -rln "chronicler-docs-consistency" --include="*.md" --include="*.py" --include="*.json" --include="*.toml"` returns only the skill file itself (verified 2026-07-03). No external references to migrate.

**Actions:**

1. Verify Task 1's `chronicler-docs-hygiene/SKILL.md` explicitly covers the kept checks from `chronicler-docs-consistency` (Behavior Mismatch, Ghost Features).
2. Verify Task 2's `validate_docs.py` covers Broken References (moved to script).
3. Delete `.agents/skills/chronicler-docs-consistency/SKILL.md` and its directory if empty.
4. Re-grep to confirm no orphan references.

**Acceptance criteria:**

- [ ] Pre-deletion grep confirms no external references (re-run day-of).
- [ ] Post-deletion grep returns empty.
- [ ] Task 1 skill covers Behavior Mismatch + Ghost Features phases.
- [ ] Task 2 script covers broken markdown link + ADR ref checks.

---

### Task 4: Dry-run verification

**Story points:** 1

**Actions:**

1. Manually invoke skill on `chronicler_engine/docs/system/llm_processing.md` (known sediment-heavy file). Verify skill outputs report matching the survey's quotable-phrase anchors: the `**Fix**:` prefix in the Gemma 4 section; the Reddit forum link; the `**Validation**: Reduced completion tokens from 2048 to ~211` PR-metric line; the `ForensicsCollector … was removed on 2026-07-03` deletion-history sentence; the §10 "Instrumented Functions" and "Module Location" code-indexer sections.
2. Run `validate_docs.py` on current `docs/`. Verify clean exit (no seeded broken links yet) OR list of any existing broken links.
3. Sanity check: skill should NOT flag `docs/system/worlds.md` HTTP→handler→service→storage data-flow chain (legitimate data flow, not sediment). Verify skill distinguishes.

**Acceptance criteria:**

- [ ] Dry-run on `llm_processing.md` surfaces all the survey's quotable-phrase sediment anchors (PR-metric line, Reddit URL, `**Fix**:` register, deletion-history sentence, code-indexer sections).
- [ ] Dry-run on `worlds.md` does NOT flag legitimate data-flow chains.
- [ ] Script runs clean or lists real broken links.
- [ ] No false positives on ADR cross-references in `adr-005`, `adr-012` (exempt path).

---

## Dependencies

| Task | Depends on | Blocks |
|------|-----------|--------|
| Task 1 (SKILL.md) | None | Task 3, Task 4 |
| Task 2 (validate_docs.py) | None | Task 3, Task 4 |
| Task 3 (Fold + delete) | Task 1, Task 2 | Task 4 |
| Task 4 (Dry-run) | Task 1, Task 2 | — |

Tasks 1 + 2 are independent. Can execute in parallel.

**Recommended execution order:** Task 1 → Task 2 → Task 3 → Task 4. Skill first (primary artifact), script second (decoupled, can lag), fold third (after coverage verified), dry-run last (gate before close).

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Skill scope too aggressive → false positives on legitimate prose (data flows, illustrative paths) | Skill explicitly distinguishes bulk-listing (code-indexer) from single-mention illustrative path. Dry-run on `worlds.md` validates. |
| Auto-trigger fires too often → token tax on every doc edit | Skill description scoped to "creating/updating docs in `chronicler_engine/docs/`". Skill file kept <130 lines. Report-only (no destructive action) limits blast radius. |
| Skill drifts into auto-editing mid-task | Explicit "read-only, do NOT edit files" line in skill. Matches `chronicler-comment-fixer` precedent (report-only). |
| Script + skill rules diverge over time | Decoupled by design. Skill is LLM semantic; script is deterministic. Different teams of rules. Cross-reference in skill's "Related" section only. |
| Folded checks (Ghost Features, Behavior Mismatch) lose precision vs old skill | Kept checks re-stated in skill phases. Dropped checks (Missing Concepts, Outdated Patterns, Wrong Signatures) explicitly listed in plan as dropped + rationale. |
| `invariants.md` flag fires as false positive | Codified as single-file exemption + skill flags the file itself as code-indexer candidate (one-shot finding, not per-line). |

---

## Out of Scope

- **Auto-cleanup of existing sediment in `docs/system/llm_processing.md`.** Sediment exists; the skill will flag it. Applying fixes is a separate cleanup pass, not part of skill creation.
- **Deprecated-term auto-extraction from `CONTEXT.md`.** Deferred per Q5 in grilling. Skill handles term-conflicts via LLM against `CONTEXT.md` directly, not via script rule.
- **Migrating `validate_docs.py` into `build.py` or CI.** Script is created standalone; CI integration is a separate decision after rule set stabilizes.
- **Extending `validate_adrs.py` to enforce decision-rationale in ADR prose.** Separate concern; ADRs are out of scope for this skill per Q4 in grilling.
- **Auto-creating ADRs from extracted sediment.** Skill flags candidates only. Actual ADR creation deferred to user or `documentation-and-adrs` skill. Per AGENTS.md plan-adherence rule.

---

## Verification Log

- Sediment survey of `docs/system/*.md`: completed 2026-07-03.
- Grep for `chronicler-docs-consistency` references: completed 2026-07-03, only self-reference found.
- Cross-check with `chronicler-comment-fixer` scope (comments vs markdown bodies): confirmed boundary, no overlap.
- Cross-check with `domain-modeling` skill (`CONTEXT.md` ownership): confirmed, no overlap. New skill references but does not redefine.
