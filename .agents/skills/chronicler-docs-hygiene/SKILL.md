---
name: chronicler-docs-hygiene
description: "Use when auditing chronicler_engine docs for stale cross-references, ghost schemas, code-indexer drift (mechanic listings without a data-flow claim), tone rot, behavior mismatch vs source, or cross-doc drift on shared concepts. Read-only; reports findings, never edits."
---

# Documentation Hygiene

**This skill is read-only on docs. Do NOT edit, delete, or rewrite files. Report findings only.**

For the per-edit gate that prevents sediment during writing, see `chronicler_engine/docs/AGENTS.md` §"The Per-Edit Gate". This skill audits accumulated violations.

## Philosophy

Docs in this repo are a **Specification**, not a conversation. Predictable: a future reader who never saw the diff must reconstruct intent from the page alone. Sediment is the enemy; reasoning density is the goal.

A code reference (type, function, file, module, line) in prose is justified only if removing it loses contract info. If prose can state the contract without the reference, the reference is sediment — the code is the verification, the doc verifies the contract, not the implementation. Bulk listings of functions/types/fields without a Data-Flow Claim are sediment by definition.

## Leading Words

Operational definitions. The audit phases below operationalize these.

- **Specification** — declarative present-tense contract. The form `docs/system/*.md` should take.
- **Sediment** — conversational clutter: PR metrics, timeline logs, "removed on YYYY-MM-DD", forum links, "what used to exist", implementation summaries, bulk symbol/field listings.
- **Duplication** — the same meaning given more than one single source of truth. Multiple docs stating the same layer count, multiple sections restating the same contract, multiple cross-references for the same fact.
- **Time Capsule** — Plan files (`docs/plans/*.md`). The only place narrative context, debugging paths, and model quirks belong. Out of scope here.
- **Invariant** — RESERVED. Means `docs/architecture/invariants.md` machine-checked `INV-NNN` entries with contract tests. Do **not** use "invariant" for prose laws; that clashes with the registry term. Use "Specification" or name the law directly.
- **Data-Flow Claim** — sentence naming what flows through what (input → module → output). The sole justification for keeping a mechanic reference in prose. Bulk function/type lists without one = Sediment.
- **Code-Indexer** — section that reproduces source-tree shape (function/module/file listings) without a Data-Flow Claim or contract claim. Drift signal: prose contract replaced by structural enumeration. Phase 4 detects it.
- **Tone Rot** — voice drift toward conversational past/future tense, narrative walkthrough, or hedging. Phase 3 detects it.
- **Anchor** — a function/symbol/module/file reference in prose or a table cell. Each carries doc-rot tax (rename = audit finding). Pass the non-removable test or remove.

## Scope & Boundaries

Canonical taxonomy: `validate_docs.py` STANDARD_DIR_NAMES. Hygiene audits what the script treats as STANDARD; the script owns mechanical checks (broken links, ADR-NNN resolution, plan-link leakage), this skill owns semantic checks (Sediment, mechanics leakage, Tone Rot, ghost features). Run the script first and surface any overlap in `Findings`.

**In scope (STANDARD docs):**

- All of `chronicler_engine/docs/` excluding `plans/`, `adr/`, `architecture/invariants.md`, and the auto-generated index (`docs/AGENTS.md` AUTO-INDEX block).

**Out of scope:**

- `docs/plans/`, `old-docs/`, `CHANGELOG.md` — Time Capsule. Audit only cross-link leakage INTO STANDARD pages.
- `docs/adr/*.md` (prose) — decision records; date stamps, status changes, PR context are appropriate there. AUDIT only ADR cross-references that point INTO STANDARD pages.
- `docs/adr/README.md`, `docs/adr/adr-000-template.md` — ADR standards meta + placeholder. Skip.
- `docs/AGENTS.md` prose preamble — owned by `chronicler_engine/AGENTS.md` per-edit gate. Skip the AUTO-INDEX block too.
- `docs/architecture/invariants.md` — machine-checked `INV-NNN` registry. Owned by `validate_docs.py`.
- Rust `//!` / `///` and Python `#` comments — owned by `chronicler-comment-fixer`.
- Single-mention illustrative path (`./foo.rs`, `docs/x.md`) — allowed; only bulk listings are Sediment.
- `CONTEXT.md` glossary terms — owned by `domain-modeling`; do not redefine.
- Auto-creating ADRs from extracted sediment — flag candidate only; user or `documentation-and-adrs` skill decides.

## Execution Steps

Run all phases. **Zero findings in a phase = phase complete. Do not skip remaining phases because an earlier phase was clean.**

1. **Sediment audit.** Flag: rubric tests (mentioned with no assertion); past-tense deletion history ("was removed on DATE", "deleted in Phase X", "never wired"); external forum links (Reddit/StackOverflow/HN); PR-style validation metrics ("Reduced tokens from N to ~M"); implementation summaries (sections that could be deleted without losing contract info); bulk field/symbol bullets (`**foo**: desc. **bar**: desc. **baz**: desc. ...`).
   Cross-ref: past-tense deletion history also surfaces in Phase 3 (Tone Rot) — cite both phases in findings.
   Completion: every prose line in every in-scope file covered; each finding cites FILE:LINE and Sediment subtype.

2. **Mechanics eviction.** Flag private function names, type names, and module references embedded in body prose not justified by a Data-Flow Claim or contract claim. Apply the non-removable test (see Philosophy): if prose can state the contract without the reference, the reference is sediment.
   Carve-out: fenced code blocks (`` ```rust ``, `` ``` ``, `` ```mermaid ``) are NOT prose. Their presence alone is not a finding; flag only when fenced code contradicts or duplicates prose claims.
   Completion: every prose mechanic reference flagged unless it passes the non-removable test; citation FILE:LINE.

3. **Tone correction.** Declarative present tense. Flag future tense ("will"), past tense describing current behavior ("used to", "previously"), narrative walkthroughs ("First we do X, then we do Y"), hedged claims ("probably", "should", "may"). Flag past-tense deletion-history walkthroughs (e.g. "Older revisions of this document called this X. That framing was inaccurate:" — the past-tense framing itself is Sediment, even when explaining a fix).
   Completion: every tense/voice violation flagged with FILE:LINE; pass the Specification test (see Philosophy).

4. **Code-indexer pattern detection.** Flag sections whose purpose is enumerating functions/files/modules/fields without a data-flow or contract claim. Whole-file registries of functions are sediment. A "Components" table is implementation summary unless each row states a contract.
   Completion: every section listing 3+ functions/files OR any field-bullet enumeration OR any 16+ row symbol table flagged; citation FILE:LINE.

5. **Ghost Features.** A **capability claim** is an imperative or descriptive statement asserting the system supports / detects / enables / handles a named behaviour (e.g. "the engine detects double-submit and rejects", "triggers fire on every state transition"). Doc asserts capability; `src/` lacks it; no explicit `> **Status:** Planned / Stub / Proposed` marker. Treat unmarked claims as ghosts.
   **Schema claims** (subtype): doc asserts a struct has named fields, a JSON example lists field names, or a function signature is shown. Verify each named field / signature exists in the live `src/` struct definition. JSON examples listing fields NOT on the struct are ghost-schema claims. Field bullets (`Message { text, location_header, ... }`) where any field is not a direct field of the named struct = ghost-schema claim.
   Verification: search `src/` for the named behaviour — entry points, traits, dispatch tables, tests. Partial implementation counts as Ghost unless the doc downgrades the claim to "stub" / "planned" / "in progress".
   Completion: every capability claim AND every schema claim verified against `src/`; unmarked ghosts reported with FILE:LINE and missing evidence.

6. **Behavior Mismatch.** Doc claims behavior X; code does Y. Extract behavioral claims, verify against `src/`, report contradictions with file:line. This phase is semantic verification of behavioral claims (mechanical link / ADR / plan-link checks owned by `validate_docs.py` — see Scope).

7. **Cross-Doc Drift.** Conditional phase: when ≥2 STANDARD docs reference the same concept (layer count, enum name, struct field, phase number, status enum variant), verify all references agree. Common drift: layer counts (8 vs 7), enum variants, struct field names (direct vs accessor). Report drift with both file refs. Skip if no concept is referenced by ≥2 docs.

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
