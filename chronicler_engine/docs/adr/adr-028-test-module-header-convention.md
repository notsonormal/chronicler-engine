# ADR-028: Test Module Header Convention

**Date:** 2026-07-04
**Status:** Accepted

## Context

Chronicler Engine enforces a two-line module header on every production file in `src/`:

```rust
//! [DOC: docs/path/to/domain-doc.md]
//! Human-readable summary
```

The first line anchors the file to a domain doc; the second line is the summary that drives the auto-generated `chronicler_engine/AGENTS.md` structure index (`scripts/generate_structure_index.py`). A guardrail rule (`check_doc_standards` in `tests/infrastructure/guardrails/structure.rs`) fails the build on any `src/` file that lacks both lines.

Test files in `tests/` currently have no equivalent rule. Some have full `[DOC: ...]` headers, some have bare `//!` summaries, and 32 of ~49 files have no module header at all. The structure of the test tree is invisible — there is no auto-generated index of what each test file covers.

We want:
1. An auto-generated `tests/AGENTS.md` structure index mirroring the `src/` index.
2. A guardrail rule that fails the build when a test file lacks a `//!` summary.

The question is which header shape the rule enforces. Production files use `[DOC: ...]` + summary; tests have a different organisational basis (fixture weight, not domain), so a literal copy of the production rule is wrong.

## Decision

**Test files use a single-line `//!` summary; no DOC anchor is required.**

```rust
//! Browser tests for editing — DOM mutation + persistence

use foo::bar;
// ... rest of file
```

Optional multi-line summaries are allowed:

```rust
//! Integration tests for game lifecycle operations.
//!
//! RATIONALE: This file is cross-cutting over `src/application/` rather than a
//! mirror of a single source module.
```

The header is the first non-blank line of the file. Subsequent lines may be `#![...]` inner attributes or `use` statements.

Two artefacts enforce the convention:

1. **`scripts/generate_tests_structure_index.py`** — mirrors `generate_structure_index.py`. Walks `tests/**/*.rs`, extracts the first `//!` summary, writes a bullet tree to `tests/AGENTS.md` between `<!-- AUTO-STRUCTURE-TESTS START -->` / `END -->` markers.
2. **`check_test_module_header`** guardrail (in `tests/infrastructure/guardrails/structure.rs`) — fails the build on any `tests/**/*.rs` file whose first non-blank line is not a non-empty `//!` comment.

The script and rule cover the entire `tests/` tree, including `tests/helpers/` and `tests/test_utils/`.

### Why summary-only (no DOC anchor)

- **Test files are organised by fixture weight, not domain.** The test mirror convention (chronicler_engine/AGENTS.md, 2026-07-02) groups tests by binary (integration / http / browser / llm / infrastructure) and mirrors `src/` paths inside each binary. A test for `ApplicationService::with_storage` lives under `tests/integration/application/`, not under any single domain doc. Forcing a `[DOC: docs/system/X.md]` anchor would either be copy-paste noise pointing at `docs/reference/testing.md` for 36 files, or arbitrary / wrong anchor choices.
- **A single-line summary is sufficient to drive the structure index.** The auto-generator only needs the summary text; it does not consume the anchor. Adding an anchor requirement multiplies the surface for bikeshedding without buying anything.
- **DOC anchors remain available when meaningful.** A test file that is the canonical example of a domain concept (e.g. `state_patch.rs` exercising `StatePatch` merge semantics) may anchor to `state-management.md`. The rule does not forbid this — it only requires the summary.

### Why cover `tests/helpers/` and `tests/test_utils/`

- The structure index lists every file under `tests/`, so the rule that drives the index must cover the same set. Otherwise `tests/AGENTS.md` promises structure the guardrail does not enforce.
- These directories hold shared plumbing imported by other test binaries. A `//!` summary that says "Shared X for Y tests" is honest about the file's purpose and aids readers landing on the structure index.

### Why `Violation::warn` not `Violation::error`

The existing `check_doc_standards` rule for `src/` files uses `Violation::warn`. `assert_violations` panics on any violation regardless of severity, so WARN still fails the build. Keeping WARN matches the existing convention and avoids inconsistency between the two rules.

## Consequences

### Positive

- Every test file is discoverable via `tests/AGENTS.md` with a one-line description of what it covers.
- The build fails when a new test file is added without a `//!` summary, preventing future drift.
- Authors are not forced to invent or copy-paste DOC anchors for files that do not have a natural domain-doc home.
- Existing `//!` headers in tests (e.g. `//! Integration tests for DefaultApplicationService`) remain valid; no churn beyond adding headers to files that lack them.

### Negative

- Test files no longer carry an explicit pointer to a domain doc. Readers looking for the "why" of a test pattern must follow the production code being tested, not a `[DOC: ...]` link.
- Two header conventions coexist in the repo (production: `[DOC: ...]` + summary; tests: summary only). Contributors must learn the distinction.
- The new rule fires on ~32 files; all are fixed in the same change to keep the rule green from day one.

### Trade-offs

- **Summary-only over DOC-anchor-mandatory:** chose summary-only because tests are organised by fixture weight, not domain. Cost: weaker link from test to spec. Benefit: no fake anchors, single rule shape.
- **All-of-`tests/` over `tests/integration/`-only:** chose all because `tests/AGENTS.md` lists the whole tree. Cost: helpers and test_utils also need headers. Benefit: index and guardrail match scope.
- **WARN over ERROR severity:** chose WARN to match `check_doc_standards`. Cost: minor inconsistency between rule "importance" labels. Benefit: consistent severity style across doc-related rules.

## Related ADRs

- [ADR-027: Hexagonal Architecture Migration](./adr-027-hexagonal-architecture-migration.md) — defines the layer dependency rules enforced by `arch-lint.toml`.

## References

- `scripts/generate_structure_index.py` — original structure index generator (src/).
- `tests/infrastructure/guardrails/structure.rs::check_doc_standards` — production doc-standards rule this ADR parallels.
- `chronicler_engine/AGENTS.md` — test mirror convention (2026-07-02).