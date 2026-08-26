# 10 — Document inherent impl locality rule

Type: task
Status: closed
Resolution: resolved-by-events
Blocked by: 09

## Question

Write a `## Inherent Impl Locality` section in `docs/diataxis/reference/architecture_system.md` documenting the module-per-type rule.

Content to cover (mirror the structure of the planned `## Free fn Doctrine` section from issue 11 in `.scratch/free-fn-scanner-rules/`):

- **Mental model**: module-per-type. A type's inherent impls live either in its single defining file (`foo.rs`) or in a folder dedicated to the type (`foo/`). Cross-folder or cross-layer splits are violations — they signal that the type is misplaced or that incidental refactor residue has accumulated.
- **Rule statement** (verbatim from the map):
  ```
  For every inherent impl `impl Foo` in production src/:
    impl_path must equal def_path, OR
    impl_path's parent dir must end with /snake_case(Foo)
  ```
- **Exclusions**: trait impls (separate policy), test files (`_tests.rs`), `#[cfg(test)] mod` inner blocks, `main.rs`.
- **Enforcement**: `guardrails_inherent_impl_locality` in `tests/infrastructure/guardrails/inherent_impl.rs`, riding `cargo test --test guardrails`, gated by `build.py`.
- **Anti-patterns**:
  - Splitting impls across sibling files in a folder not named after the type (e.g. `ActionPipeline` in `pipeline.rs` + `phases.rs` — fixed by consolidating or renaming folder).
  - Cross-layer impls (e.g. domain struct + application impl — fixed by relocating the type to the layer that owns its behavior, NOT by adding a trait).
  - "Mapper" pattern placing `impl DbX` in `backend/Xs.rs` away from `models/X.rs` — fixed by co-locating with the type definition and renaming files to match type names.
- **What this rule does NOT decide**:
  - Where trait impls live (separate trait-impl locality policy, out of scope for this effort).
  - Whether a type deserves its own folder vs. single file (judgment call — pick based on file_length guardrail's 2000-line cap).
- **Review policy vs hard rule**: this is a hard structural rule. Semantic questions ("is this folder really cohesive?") remain review policy, not guardrail matters — per the repo's standing constraint that guardrails must be deterministic, never LLM-judged at decision time.

Also:
- Anchor `[DOC: docs/diataxis/reference/architecture_system.md#inherent-impl-locality]` at the top of the rule file `inherent_impl.rs` so the rule and doc cross-reference.
- If `architecture_system.md` does not exist at the expected path, surface the discrepancy before creating the section (do not silently invent a new doc location).

Constraints:
- No code changes outside the doc and the `[DOC: ...]` anchor comment in `inherent_impl.rs`.
- Follow `chronicler-docs-hygiene` conventions — diataxis reference doc style.

Acceptance:
- Section exists in `architecture_system.md`.
- `inherent_impl.rs` has `[DOC: ...]` anchor pointing to the section.
- `build.py` green (docs validation step, if present).
- `chronicler-docs-hygiene` skill reports no violations introduced by this change.

## Resolution

Resolved by events — the auto-generation added in `scripts/generate_guardrails_doc.py` already documents the rule, and the ticket's specific instructions are stale on three points. No code or doc changes were made.

- **The rule is already documented.** `generate_guardrails_doc.py` reads the `///` doc comment on `check_inherent_impl_locality` (`tests/infrastructure/guardrails/inherent_impl.rs:12-15`) and emits a row in the auto-managed syn table in `docs/diataxis/reference/coding_standards/guardrails.md` (`inherent impl locality` row, line 60). That row stays in sync with the source automatically; there is nothing manual to maintain. No other guardrail rule (free fn location, mod purity, file length, etc.) has a dedicated prose section either — the one-row table is the established pattern for every rule in this suite.

- **The ticket's target path does not exist.** It names `docs/diataxi/reference/architecture_system.md`. The real folder is `docs/diataxis/` (with an "s"), and no `architecture_system.md` exists anywhere in `docs/`. The ticket's own constraint ("surface the discrepancy before creating the section; do not silently invent a new doc location") forbids inventing one.

- **The mirror reference is stale.** The ticket says to mirror "the planned `## Free fn Doctrine` section from issue 11 in `.scratch/free-fn-scanner-rules/`." That scratch directory no longer exists, and no `Free fn Doctrine` section exists in any doc. The structural template it wanted to copy is gone.

- **The `[DOC: ...]` anchor instruction would break a guardrail.** The ticket asks for a `[DOC: ...]` anchor on `inherent_impl.rs`, a test file. The `doc standards` guardrail (`tests/infrastructure/guardrails/structure.rs:69`) rejects DOC anchors in test files: "Test files must use a plain `//! <summary>` line only." `inherent_impl.rs` already has the correct plain `//!` summary on line 1; adding the anchor would fail `build.py`. No guardrail rule file carries a DOC anchor — consistent with that rule.

If a richer prose treatment is later wanted, it would be a fresh doc decision (e.g. a new `docs/diataxis/reference/coding_standards/inherent_impl_locality.md`, or a section under `docs/diataxis/explanation/architecture.md#architectural-commitments`), not the ticket as written.
