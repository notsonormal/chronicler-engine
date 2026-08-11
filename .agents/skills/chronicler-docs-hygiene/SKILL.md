---
name: chronicler-docs-hygiene
description: "Use when auditing docs/diataxis/ docs against AGENTS.md and src/ for what validate_docs.py can't catch: rule violations, mode drift, stale-source drift. Read-only; reports findings, never edits."
---

# Documentation Hygiene

## Philosophy

Docs are a **Specification**, not a conversation. Sediment is the enemy; reasoning density is the goal. A code reference in prose is justified only if removing it loses contract info; if prose can state the contract without the reference, the reference is sediment — the code is the verification, the doc verifies the contract, not the implementation.

- **Sediment** — conversational clutter: PR metrics, timeline logs, "removed on YYYY-MM-DD", forum links, "what used to exist", implementation summaries, bulk symbol/field listings.
- **Ghost Features** — doc asserts a capability or schema the live `src/` doesn't carry, with no explicit `> **Status:** Planned / Stub / Proposed` marker. Subtype: **schema claims** (doc asserts a struct has named fields, a JSON example lists field names, a function signature is shown — verify each against the live `src/`).



## Execution

Run all phases regardless of earlier outcomes. A clean phase does not end the audit; the audit ends when every phase's completion criterion is met across every in-scope file.

## Phase 1 — Validator

Run the validator first (`python scripts/validate_docs.py --strict`). Surface any overlap in `Findings` rather than re-litigating machine-layer warnings.

## Phase 2 — Conventions compliance

Per-doc check against `docs/AGENTS.md` `Writing Conventions` rules the validator can't enforce. **The phase points at AGENTS.md anchors; it never restates the rule content.**

Rules (check body prose against each anchor in `docs/AGENTS.md` `## Writing Conventions`):

- **`No negative explaining`** — flag body-prose negation, tautological negative definitions, defensive scope disclaiming. Out-of-scope lists and Diagrams are the canonical home for scope statements (not findings); see the rule's carve-out.
- **`Explanation unfolds; it does not justify`** — flag section titles phrased as `Why X?` or `Why X instead of Y?`; flag justification-tail framing in body prose (`the design pays that cost in exchange for X`, `the design holds this cost for Y`).
- **`No code-indexer docs`** — mechanics leaks in body prose: bare impl-detail function names, `Type::method()` form when instance form names it, variant payload syntax, struct field dumps, Rust-type leaks, code syntax. Apply the seam-identifier grep test ("would a reader grep for the name to find the contract the prose is describing?") — full Keep/Drop list at this anchor in AGENTS.md.
- **`Reference defers to source`** — flag column-level DDL, struct field lists, function signatures, migration version numbers, constants, caps restated in `reference/` prose when those are the authoritative source in code.
- **`Register`** (nested under `#### Explanation`) — flag narrated reader experience, dramatic contrast framing, editorializing perspective, speculative color in `explanation/` prose.

Completion: every in-scope file audited against each anchor; each finding carries the AGENTS.md §anchor.

## Phase 3 — Sediment

Flag: rubric tests (mentioned with no assertion); past-tense deletion history ("was removed on DATE", "deleted in Phase X", "never wired"); external forum links (Reddit / StackOverflow / HN); PR-style validation metrics ("Reduced tokens from N to ~M"); implementation summaries (sections that could be deleted without losing contract info); bulk field/symbol bullets (`**foo**: desc. **bar**: desc. **baz**: desc. ...`).

Past-tense deletion history often co-occurs with Phase 1's no-negative-explaining or justification-tail findings — flag the prose once, citing both phases.

Completion: every prose line in every in-scope file covered; each finding cites `FILE:LINE` and Sediment subtype.

## Phase 4 — Ghost Features

Doc asserts a capability or schema the live `src/` doesn't carry, with no explicit `> **Status:** Planned / Stub / Proposed` marker. Treat unmarked claims as ghosts.

**Capability claims** — imperative or descriptive statements asserting the system supports / detects / enables / handles a named behaviour (e.g. "the engine detects double-submit and rejects", "triggers fire on every state transition"). Verify against `src/`: entry points, traits, dispatch tables, tests. Partial implementation counts as Ghost unless the doc downgrades the claim to "stub" / "planned" / "in progress".

**Schema claims** (subtype) — doc asserts a struct has named fields, a JSON example lists field names, a function signature is shown. Verify each named field / signature exists in the live `src/` struct definition. JSON examples listing fields NOT on the struct are ghost-schema claims. Field bullets (`Message { text, location_header, ... }`) where any field is not a direct field of the named struct = ghost-schema claim.

Completion: every capability claim AND every schema claim verified against `src/`; unmarked ghosts reported with `FILE:LINE` and missing evidence.

## Phase 5 — Behavior Mismatch

Doc claims behavior X; code does Y. Extract behavioral claims, verify against `src/`, report contradictions with `file:line`. This phase is semantic verification of behavioral claims (mechanical link / ADR / plan-link checks owned by `validate_docs.py`).

Completion: every behavioral claim verified against `src/`; contradictions reported with `FILE:LINE` (doc) and `src/file:line` (code).

## Phase 6 — Cross-Doc Drift

Conditional phase: when ≥2 in-scope docs reference the same concept (layer count, enum name, struct field, phase number, status enum variant, INV-NNN identifier), verify all references agree. Common drift: layer counts (8 vs 7), enum variants, struct field names (direct vs accessor), INV-NNN guarantees. Report drift with both file refs.

Skip if no concept is referenced by ≥2 docs AND no cross-tree citations exist.

Completion: every concept referenced by ≥2 in-scope docs verified to agree across all references; cross-tree citations flagged.

## Phase 7 — Enum variant paraphrase

Enum variant semantics live on the variant in `src/` as `///` rustdoc (or are self-documenting via `/// [TRIVIAL_ENUM]`). Flag any in-scope doc section that re-paraphrases variant semantics already stated on the enum, unless the doc adds a Data-Flow Claim or behavior-specific contract (e.g. "`PhaseError::Cancelled` is surfaced to the client as HTTP 499, with state rolled back" — the HTTP rollback is the contract, the variant definition is not). Variant listings in docs must be a pointer to source ("see `PhaseError` in `src/application/action_pipeline/phase_error.rs`") or omitted.

Completion: every enum variant re-paraphrase flagged with `FILE:LINE`; cite the source enum path.

## Output Format

```
Status: [PASS] or [FAIL]

# Findings:
- FILE:LINES — Severity — Phase
  Current: (snippet)
  Expected: (spec-shaped rewrite | "remove" | "add evidence" | "downgrade to Stub/Planned" | "see AGENTS.md §<anchor>")

# Recommendations:
  FILE:LINES — Phase — one-line fix
  (1–3 recommendations per finding; never more)
```

Severity decision rule:

- `Error` — false claim or unmarked Ghost (reader-trust violation). Must fix before merge.
- `Warning` — contract-weakening or reader-blocking Sediment / mechanics leak / mode drift / AGENTS.md rule violation that hides current state. Should fix.
- `Info` — cosmetic / minor (lone outdated date, single PR metric in otherwise-clean doc, illustrative path drift). Consider when nearby.

## Related Skills

- **chronicler-comment-fixer** — detects AI slop in Rust `//!` / `///` and Python `#` comments. Sibling; audits code comments, this skill audits docs.
- **domain-modeling** — owns `CONTEXT.md` glossary and ADR authoring triggers. Cross-reference terms; do not redefine.
