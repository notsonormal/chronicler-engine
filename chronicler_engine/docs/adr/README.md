# ADR Standards

Architecture Decision Records for the Chronicler Engine. Each ADR captures *why* a significant decision was made — not *what was changed*.

## Template and rules

The canonical template and all rules live in [`adr-000-template.md`](./adr-000-template.md). Copy that file to start a new ADR. The inline comments in the template document every requirement, forbidden section, and lifecycle rule — they are the single source of truth.

The rules are machine-enforced by [`scripts/validate_adrs.py`](../../scripts/validate_adrs.py). Run it locally:

```bash
python scripts/validate_adrs.py
python scripts/validate_adrs.py --path docs/adr/adr-NNN-foo.md
```

Exits non-zero on any violation. The standard is enforced uniformly on all ADRs.

## When to write an ADR

Write one only when **all three** are true:

1. **Hard to reverse** — changing your mind later has meaningful cost
2. **Surprising without context** — a future reader will wonder "why this way?"
3. **Result of a real trade-off** — genuine alternatives existed and one was picked for specific reasons

If any is missing, skip the ADR. Use inline comments, PR descriptions, or plan docs instead.

## When NOT to write an ADR

- Bug fixes (use a plan or commit message)
- Pure refactors with no design choice (use a plan)
- File moves and renames
- Adding a new test
- Implementation detail (belongs in a plan under `docs/plans/`)

## File conventions

- File name: `adr-NNN-kebab-case-title.md` (zero-padded 3-digit number)
- Title line: `# ADR-NNN: Title`
- Numbering: sequential, never reused, never renumbered
- Start a new ADR by copying `adr-000-template.md`

## Vocabulary

ADR terms must match [`chronicler_engine/CONTEXT.md`](../../CONTEXT.md). ADRs may not redefine these terms — they may only use them. If an ADR reveals a glossary gap or contradiction, fix CONTEXT.md first, then write the ADR.

## Scope

One decision per ADR. If a decision covers multiple independent features, split it — one ADR per feature, cross-referenced under `Related ADRs`.

## Indexing

ADR files are auto-indexed in [`docs/README.md`](../README.md) by the doc-index generator. New ADRs appear after the next index regeneration — do not hand-edit the auto-index block.

## Existing ADRs

All ADRs conform to the current standard (enforced by `validate_adrs.py`). Known structural debts that the validator permits but are not yet resolved:

- **ADR-006** — covers 5 features + dual-LLM architecture in one ADR; violates one-decision-per-ADR rule. Splitting requires writing new ADRs.
- **ADR-012** — number reused (deleted Turn+Swipe ADR-012 vs current LLM Logging ADR-012); historical references may be ambiguous. Do not repeat this mistake.

Fixing these is a separate cleanup task, not part of writing new ADRs.

## Deletions

ADRs may be deleted when they are no longer relevant or never qualified as ADRs in the first place (e.g., implementation patterns, superseded intermediate decisions whose content is fully captured by a later ADR). Deletion is preferred over leaving stale content that future readers must triage.

Deleted ADRs:
- **ADR-013** (Message Domain Model) — superseded by ADR-017; intermediate wrong decision
- **ADR-019** (One Table Per Storage Module) — superseded by ADR-020; pure historical
- **ADR-021** (State Patch Reducer) — not architectural; code-level utility
- **ADR-023** (Immediate Message Persistence) — implementation invariant, not architecture
- **ADR-018** (Application Service Layer) — referenced by ADR-027 but file never existed; reference removed from ADR-027

Numbers are not reused. Gaps in numbering are expected.
