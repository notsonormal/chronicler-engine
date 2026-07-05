---
name: chronicler-docs-hygiene
description: "Read-only audit of chronicler_engine/docs/ Specification pages. Flags Sediment, mechanics leakage, code-indexer drift, tone rot. Reports findings, never edits."
---


# Documentation Hygiene

**This skill is read-only. Do NOT edit, delete, or rewrite files. Report findings only.**

## Philosophy

Docs in this repo are a **Specification**, not a conversation. Predictable: a future reader who never saw the diff must reconstruct intent from the page alone. Sediment is the enemy; reasoning density is the goal.

## Leading Words

- **Specification** — declarative present-tense contract. Reserved sense. The form `docs/system/*.md` should take.
- **Sediment** — conversational clutter: PR metrics, timeline logs, "removed on YYYY-MM-DD", forum links, "what used to exist".
- **Time Capsule** — Plan files (`docs/plans/*.md`). The only place narrative context, debugging paths, and model quirks belong. Out of scope here.
- **Invariant** — RESERVED. Means `docs/architecture/invariants.md` machine-checked `INV-NNN` entries with contract tests. Do **not** use "invariant" for prose laws; that clashes with the registry term. Use "Specification" or name the law directly.
- **Data-Flow Claim** — sentence naming what flows through what (input → module → output). The sole justification for keeping a mechanic reference in prose. Bulk function/type lists without one = Sediment.
- **Code-Indexer** — section that reproduces source-tree shape (function/module/file listings) without a Data-Flow Claim or contract claim. Drift signal: prose contract replaced by structural enumeration. Step 4 detects it.
- **Tone Rot** — voice drift toward conversational past/future tense, narrative walkthrough, or hedging. Step 3 detects it.

## Scope & Boundaries

Canonical taxonomy: `validate_docs.py` STANDARD_DIR_NAMES. Hygiene audits what the script treats as STANDARD; the script owns mechanical checks (broken links, ADR-NNN resolution, plan-link leakage), this skill owns semantic checks (Sediment, mechanics leakage, Tone Rot, ghost features). Run the script first and surface any overlap in `Findings`.

**In scope (STANDARD docs):**

- `chronicler_engine/docs/system/*.md`
- `chronicler_engine/docs/architecture/*.md` (except `invariants.md`)
- `chronicler_engine/docs/reference/*.md`
- `chronicler_engine/docs/diagnostics/*.md`
- `chronicler_engine/docs/external_applications/*.md` — Marinara-Engine + SillyTavern references; treat as Specification pages, not commentary.

**Out of scope:**

- `docs/plans/`, `old-docs/`, `CHANGELOG.md`, `ROADMAP.md` — Time Capsule. Audit only cross-link leakage INTO STANDARD pages.
- `docs/adr/*.md` (prose) — decision records; date stamps, status changes, PR context are appropriate there. AUDIT only ADR cross-references that point INTO STANDARD pages (the inverse — STANDARD pages referencing missing ADRs — is `validate_docs.py` BROKEN_ADR_REF territory).
- `docs/adr/README.md`, `docs/adr/adr-000-template.md` — ADR standards meta + placeholder. Skip.
- `docs/AGENTS.md` — auto-generated index (`<!-- AUTO-INDEX -->` markers). Skip; canonical definitions of Sediment + Duplication live there — cross-reference rather than redefine.
- `docs/architecture/invariants.md` — machine-checked `INV-NNN` registry. Owned by `validate_docs.py`.
- Rust `//!` / `///` and Python `#` comments — owned by `chronicler-comment-fixer`.
- Single-mention illustrative path (`./foo.rs`, `docs/x.md`) — allowed; only bulk listings are Sediment.
- `CONTEXT.md` glossary terms — owned by `domain-modeling`; do not redefine.
- Auto-creating ADRs from extracted sediment — flag candidate only; user or `documentation-and-adrs` skill decides.

## Execution Steps

Run all six phases.

1. **Sediment audit.** Flag: rubric tests (mentioned with no assertion); past-tense deletion history ("was removed on DATE", "deleted in Phase X", "never wired"); external forum links (Reddit/StackOverflow/HN); PR-style validation metrics ("Reduced tokens from N to ~M").
   Cross-ref: past-tense deletion history also surfaces in Step 3 (Tone Rot) — cite both phases in findings.
   Completion: every prose line in every in-scope file covered; each finding cites FILE:LINE and Sediment subtype.
2. **Mechanics eviction.** Flag private function names, type names, and module references embedded in body prose (any in-scope folder) that are not justified by a Data-Flow Claim. Bulk `src/*.rs` + `fn()` listings without Data-Flow Claim are also Sediment.
   Carve-out: fenced code blocks (`` ```rust ``, `` ``` ``, `` ```mermaid ``) are NOT prose. Their presence alone is not a finding; flag only when fenced code contradicts or duplicates prose claims.
   Completion: every prose mechanic reference flagged unless justified by Data-Flow Claim; citation FILE:LINE.
3. **Tone correction.** Declarative present tense. Flag future tense ("will"), past tense describing current behavior ("used to", "previously"), narrative walkthroughs ("First we do X, then we do Y"), hedged claims ("probably", "should").
   Completion: every tense/voice violation flagged with FILE:LINE; pass the Specification test (see Philosophy).
4. **Code-indexer pattern detection.** Flag sections whose purpose is enumerating functions/files/modules without a data-flow or contract claim. Whole-file registries of functions are sediment.
   Completion: every section listing 3+ functions/files without data-flow claim flagged; citation FILE:LINE.
5. **Ghost Features.** A **capability claim** is an imperative or descriptive statement asserting the system supports / detects / enables / handles a named behaviour (e.g. "the engine detects double-submit and rejects", "triggers fire on every state transition"). Doc asserts capability; `src/` lacks it; no explicit `<!-- Stub -->` / `Planned` / `Proposed` marker. Treat unmarked claims as ghosts.
   Verification: search `src/` for the named behaviour — entry points, traits, dispatch tables, tests. Partial implementation counts as Ghost unless the doc downgrades the claim to "stub" / "planned" / "in progress".
   Completion: every capability claim verified against `src/`; unmarked ghosts reported with FILE:LINE and missing evidence.
6. **Behavior Mismatch.** Doc claims behavior X; code does Y. Extract behavioral claims, verify against `src/`, report contradictions with file:line. This phase is semantic verification of behavioral claims (mechanical link / ADR / plan-link checks owned by `validate_docs.py` — see Scope).

## Output Format

```
Status: [PASS] or [FAIL]

# Findings:
- FILE:LINES — Severity — Phase
  Current: (snippet)
  Expected: (spec-shaped rewrite | "remove" | "add evidence" | "downgrade to Stub/Planned")

# Recommendations:
  FILE:LINES — Phase — one-line fix
  (1–3 recommendations per finding; never more)
```

Severity decision rule:
- `Error` — false claim or unmarked Ghost (reader-trust violation). Must fix before merge.
- `Warning` — contract-weakening or reader-blocking Sediment / mechanics leak / Tone Rot that hides current state. Should fix.
- `Info` — cosmetic / minor (lone outdated date, single PR metric in otherwise-clean doc, illustrative path drift). Consider when nearby.

## Related Skills

- **chronicler-comment-fixer** — detects AI slop in Rust `//!`/`///` and Python `#` comments. Sibling; same report-only model.
- **domain-modeling** — owns `CONTEXT.md` glossary and ADR authoring triggers. Cross-reference terms; do not redefine.
- **documentation-and-adrs** — guides creating and updating documentation and ADRs. Use when applying fixes this skill recommends.