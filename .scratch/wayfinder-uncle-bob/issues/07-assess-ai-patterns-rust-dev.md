# Assess AIR-J and speclj-structure-check patterns for AI-assisted Rust development

Type: research
Status: resolved
Assignee: pi
Blocked by: 04

## Question

What patterns from `AIR-J` (canonical IR, design-by-contract, structured diagnostics) and `speclj-structure-check` (Claude skill as a pre-flight structural check) could be applied to AI-assisted development of Chronicler Engine's Rust code? Are there opportunities for structured output schemas, validation steps, or development-time skills that check Rust code structure before compilation or tests?

## Answer

### AIR-J patterns relevant to Rust code development

AIR-J's core ideas are canonical IR, design-by-contract, and structured diagnostics. None of these are directly usable for Chronicler Engine's Rust code because AIR-J is a JVM language. The transferable patterns for AI-assisted development are:

| Pattern | How it applies to Rust development |
| --- | --- |
| Canonical representation / one meaning, one representation | Reduce output variance by giving AI assistants a single, checkable target format for each structural task: module headers, test registration, enum variant docs, import ordering. |
| Design-by-contract (`requires`, `ensures`, `invariants`) | Encode repo conventions as explicit, machine-checkable contracts rather than prose guidelines. The existing guardrail tests and scripts already do this for many rules. |
| Structured diagnostics | Return machine-readable failure codes with file/line/span so an agent can repair without re-reading the whole file. The `Violation` type in `tests/infrastructure/guardrails/mod.rs` already captures this pattern. |

### speclj-structure-check pattern relevant to Rust code development

`speclj-structure-check` is a Claude skill that runs a cheap structural check before expensive Speclj tests. The transferable idea for Chronicler Engine is a **pre-flight structural checker skill** for Rust code: run fast, deterministic validations before `cargo test`, `cargo clippy`, or `cargo nextest`.

### Comparison against Chronicler Engine's existing tooling

The repo already has substantial structural and convention checking that AI assistants must respect:

**Script-based checks**

- `scripts/check_test_structure.py` — forbids inline `#[cfg(test)]` blocks and checks every `*_tests.rs` file is registered as a `mod` declaration.
- `scripts/validate_docs.py` — validates markdown links, DOC anchors, diátaxis frontmatter, and test-support file conventions.
- `scripts/generate_guardrails_doc.py` — keeps `docs/diataxis/reference/coding_standards/guardrails.md` synchronized with `src/lib.rs`, `arch-lint.toml`, and `tests/infrastructure/guardrails/*.rs`.
- `scripts/generate_structure_index.py` and `scripts/generate_docs_index.py` — keep `AGENTS.md` indexes in sync with module summaries.
- `scripts/validate_feature_spec.py` — checks feature-spec scenario coverage by integration tests.

**syn-based guardrail tests** (`tests/infrastructure/guardrails/`)

- `structure.rs` — module doc anchors, mod.rs purity, file length, empty files, legacy test-context usage.
- `style.rs` — import ordering, separator comments, long comment runs, single-letter variable names.
- `location.rs` — test-file naming and pairing (`*_tests.rs`, matching source file).
- `free_fn.rs` — top-level free functions restricted to `mappers/`, `utils/`, `builders/`, `handlers/`, `bootstrap/`, `test_support/`.
- `inherent_impl.rs` — inherent impl locality relative to the type's defining file or a same-name folder.
- `layers.rs` — layer boundaries (`WiredApp` scope, HTTP/storage leak, handler return types).
- `enums.rs` — enum variant docs or `/// [TRIVIAL_ENUM]` marker.

**Agent skills that already fill overlapping niches**

- `chronicler-comment-fixer` — scans for AI slop, missing doc anchors, and convention violations in Rust and Python.
- `code-consistency-check` — reviews architectural consistency with codebase patterns.
- `antipattern-checker` — detects semantic abstraction anti-patterns beyond static tooling.
- `chronicler-after-plan-workflow` — post-implementation verification checklist that runs `build.py` and related checks.
- `code-review` — two-axis review of standards and spec compliance.

Chronicler Engine therefore already has a `speclj-structure-check`-like toolchain. What it lacks is a **single skill or workflow step that runs all cheap structural checks before the expensive test suite**, framed specifically for AI-generated changes.

### Concrete opportunities for AI-assisted Rust development

The following conventions are hard for AI assistants to get right and are good candidates for a pre-flight structural checker or a canonical checklist:

1. **Module-level two-line headers.** Every production file in `src/` must start with `//! [DOC: docs/diataxis/reference/<area>/<name>.md]` followed by `//! <human summary>`. Test files must not carry a `[DOC: ...]` anchor. Mistakes here are common and are caught by `guardrails_doc_standards` only at test time.
2. **Test file registration.** Adding `src/foo/bar_tests.rs` without `mod bar_tests;` in the parent `mod.rs` silently orphans tests. `check_test_structure.py` catches this.
3. **Test file pairing.** A `*_tests.rs` file in `src/` must have a matching source file or module directory. Caught by `guardrails_test_file_location`.
4. **mod.rs purity.** `mod.rs` files should only contain `pub mod`, `use`, `pub use`, and module docs. Caught by `guardrails_mod_purity`.
5. **Inherent impl locality.** Inherent impls must live with the type definition or in a folder named after the type. Caught by `guardrails_inherent_impl_locality`.
6. **Free fn location.** Top-level free functions are restricted to allowlisted category folders. Caught by `guardrails_free_fn_location`.
7. **Import ordering.** `std/core/alloc` → external crates → `crate/super/self`. Caught by `guardrails_import_ordering`.
8. **DOC anchor validity.** Anchors must resolve under `docs/diataxis/reference/` and must target existing files. Caught by `scripts/validate_docs.py`.
9. **AGENTS.md / docs index freshness.** New, renamed, or removed docs/modules require regenerating `docs/AGENTS.md` and `tests/AGENTS.md`. Caught by the pre-commit hook and `generate_*_index.py` scripts.
10. **Feature-to-test traceability.** New feature scenarios must be covered by integration tests and annotated with `SCENARIO:`. Caught by `scripts/validate_feature_spec.py`.

These are all deterministic, cheap to check, and currently spread across scripts and test binaries. A canonical checklist or unified pre-flight skill could reduce review/fix cycles by surfacing them earlier.

### Recommendations

**Do not build a new tool prototype from this research alone.** The repo already has the necessary checks; the improvement is in packaging and workflow, not new capability.

Recommended concrete actions:

1. **Create a `pre-flight` skill or script** that runs the cheap checks in one command before `cargo test`:
   - `python scripts/check_test_structure.py`
   - `python scripts/validate_docs.py`
   - `cargo fmt --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo nextest run --test guardrails --test architecture` (or a targeted subset)
   - This mirrors the spirit of `speclj-structure-check`: catch structural mistakes before expensive execution.
2. **Publish the canonical checklist** as a short document or as the skill's prompt, so AI assistants emit the right structural shape the first time. This applies AIR-J's "one meaning, one representation" idea to code edits without introducing a new IR.
3. **Extend existing guardrails before adding new skills.** If a recurring AI mistake is not covered by the current guardrails, add a `pub fn check_*` in `tests/infrastructure/guardrails/` rather than building a separate skill. The existing harness (`Violation`, `assert_violations`) already supports structured diagnostics.
4. **Use structured diagnostics in skill outputs.** The `Violation` shape (file, line, severity, message) should be the standard output format for any AI-facing structural check, making repairs mechanical for the agent.

No new ticket is warranted at this point. The existing skills, guardrails, and scripts cover the `speclj-structure-check` pattern, and the AIR-J patterns are abstract guidance rather than a concrete tool. If a `pre-flight` skill is prioritized, it should be specced separately with a clear command, target audience (AI assistants vs. humans), and performance budget.
