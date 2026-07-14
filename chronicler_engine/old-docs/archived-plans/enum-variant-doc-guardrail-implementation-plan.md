# Enum Variant Doc Guardrail Implementation Plan

## Summary
Mandate `///` rustdoc on every variant of every Rust enum in `chronicler_engine` (src/ + tests/), with a `/// [TRIVIAL_ENUM]` opt-out for self-documenting enums. Adds a new `syn`-based guardrail rule to the existing `tests/infrastructure/guardrails/` harness. Migrates all 28 existing enums. Updates downstream skills + `AGENTS.md` so variant docs become the single source of truth, killing enum re-explanation in system `.md` files.

## Key Changes
- **New guardrail rule**: `tests/infrastructure/guardrails/enums.rs` with `check_enum_variant_docs(path, content)` using `syn::visit::Visit`. Enforces both directions — missing-doc **and** trivial-with-doc conflict.
- **Harness wiring**: two new `#[test]` entries in `tests/infrastructure/guardrails/mod.rs` walking src/ and tests/.
- **Migration**: ~15 trivial enums marked `/// [TRIVIAL_ENUM]`; ~13 non-trivial enums get per-variant `///` prose; 3 already partial in `quantifier.rs` verified complete.
- **Comment-fixer skill**: variant `///` no longer flagged as slop.
- **Docs-hygiene skill**: rule banning enum-variant re-paraphrasing in system docs.
- **AGENTS.md**: DOCUMENTATION STRATEGY + CONVENTIONS updated.

## Architecture
Extends existing `tests/infrastructure/guardrails/` framework that already enforces doc-anchor, import-ordering, file-length, mod-purity, and layer-boundary rules. The new `enums.rs` module follows the established `check_*` function shape (`path, content -> Vec<Violation>`), parsed via `syn::parse_file` into AST, walked by a `Visit` impl that locates `ItemEnum` nodes. Per enum, the rule checks two directions:
1. **Missing-doc**: enum not marked `[TRIVIAL_ENUM]` AND variant lacks `doc` attr → ERROR.
2. **Trivial-conflict**: enum marked `[TRIVIAL_ENUM]` AND variant has `doc` attr → ERROR (marker means self-documenting, so variant docs contradict).

Severity = `Error` (mandatory). The signal: `///` either present on every variant, or the enum is marked trivial AND all variants are bare.

## Tech Stack
- Rust Edition 2024, `syn` + `syn::visit::Visit` (already dep).
- `walkdir` file discovery (already used in `mod.rs`).
- `python build.py` runs guardrails via `cargo nextest run guardrails`.

## Global Constraints
- AGENTS.md: no "What" comments — variant `///` must be semantic, not narration.
- AGENTS.md: frequency-of-edits rule — variant prose must be concise (1 line, <100 chars preferred).
- Rust 2024 edition, `Cargo.lock` pinned.
- `chronicler-comment-fixer/SKILL.md` allowlist must be updated alongside code migration.
- Guardrails run via `cargo nextest run guardrails` and `python build.py` final gate.

---

## Implementation

### Phase 1: Guardrail Rule + Harness Wiring

#### Task 1.1: Create `enums.rs` rule (missing-doc + trivial-conflict) + unit tests (3 SP)
- [ ] ##### SubTask 1.1.1: Create failing rule unit tests + implementation (3 SP)
  - Create file `chronicler_engine/tests/infrastructure/guardrails/enums.rs` with imports, rule logic, and five `#[test]` functions covering: (a) trivial-marker skip, (b) missing-doc flag, (c) documented pass, (d) empty enum, (e) trivial-with-variant-docs conflict. Compile fails until module declared in 1.1.2.

```rust
//! Enum variant doc guardrail: every enum variant must carry `///` doc, OR the enum must be marked `/// [TRIVIAL_ENUM]` with all variants bare.

use syn::visit::Visit;
use syn::{File, ItemEnum};

use crate::Violation;

const TRIVIAL_MARKER: &str = "[TRIVIAL_ENUM]";

fn has_trivial_marker(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("doc") {
            return false;
        }
        match attr.meta.require_name_value() {
            Ok(expr) => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s), ..
                }) = expr.value
                {
                    s.value().contains(TRIVIAL_MARKER)
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    })
}

fn has_doc(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("doc"))
}

struct EnumVisitor<'a> {
    file_path: &'a str,
    violations: &'a mut Vec<Violation>,
}

impl<'a> Visit<'a> for EnumVisitor<'a> {
    fn visit_item_enum(&mut self, node: &'a ItemEnum) {
        let trivial = has_trivial_marker(&node.attrs);
        for variant in &node.variants {
            let documented = has_doc(&variant.attrs);
            match (trivial, documented) {
                (false, false) => self.violations.push(Violation::error(
                    self.file_path,
                    variant.ident.span().start().line,
                    format!(
                        "Enum variant `{}::{}` lacks a `///` doc comment. \
                         Either document the variant or mark the enum with `/// [TRIVIAL_ENUM]` \
                         directly above the `enum` declaration if variants are self-documenting.",
                        node.ident, variant.ident
                    ),
                )),
                (true, true) => self.violations.push(Violation::error(
                    self.file_path,
                    variant.ident.span().start().line,
                    format!(
                        "Enum `{}` is marked `/// [TRIVIAL_ENUM]` but variant `{}` carries a `///` doc \
                         — remove either the marker or all variant docs.",
                        node.ident, variant.ident,
                    ),
                )),
                _ => {}
            }
        }
    }
}

pub fn check_enum_variant_docs(path: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let ast: File = match syn::parse_file(content) {
        Ok(ast) => ast,
        Err(_) => return violations,
    };
    let mut visitor = EnumVisitor {
        file_path: path,
        violations: &mut violations,
    };
    visitor.visit_file(&ast);
    violations
}

#[test]
fn check_enum_variant_docs_trivial_marker_skips_check() {
    let src = r#"
/// [TRIVIAL_ENUM]
enum Direction { North, South, East }
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert!(v.is_empty(), "expected no violations, got {v:?}");
}

#[test]
fn check_enum_variant_docs_flags_missing_variant_docs() {
    let src = r#"
enum PhaseError { Cancelled, Failed }
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert_eq!(v.len(), 2, "expected 2 violations, got {v:?}");
}

#[test]
fn check_enum_variant_docs_accepts_documented_variants() {
    let src = r#"
enum PhaseError {
    /// Generation cancelled by user.
    Cancelled,
    /// Narrator LLM call failed.
    Failed,
}
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert!(v.is_empty(), "expected no violations, got {v:?}");
}

#[test]
fn check_enum_variant_docs_skips_empty_enum() {
    let src = r#"
enum Never {}
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert!(v.is_empty());
}

#[test]
fn check_enum_variant_docs_flags_trivial_with_variant_docs() {
    let src = r#"
/// [TRIVIAL_ENUM]
enum Color {
    /// Red hue.
    Red,
    Blue,
}
"#;
    let v = check_enum_variant_docs("test.rs", src);
    assert_eq!(v.len(), 1, "expected 1 conflict violation, got {v:?}");
}
```

  - Verify failure: `cargo nextest run check_enum_variant_docs` → 5 FAIL (module not declared in mod.rs).

- [ ] ##### SubTask 1.1.2: Declare module in `mod.rs` (1 SP)
  - Modify `chronicler_engine/tests/infrastructure/guardrails/mod.rs`:
    - Add `pub mod enums;` next to existing `pub mod layers;` etc.
    - Add `pub use enums::*;` after existing re-exports.

```rust
pub mod enums;
// ... existing exports ...
pub use enums::*;
```

  - Rerun `cargo nextest run check_enum_variant_docs` → 5 PASS.

- [ ] ##### SubTask 1.1.3: Wire whole-codebase guardrail tests (1 SP)
  - Modify `chronicler_engine/tests/infrastructure/guardrails/mod.rs`:

```rust
#[test]
fn guardrails_enum_variant_docs() {
    check_src_files("enum variant docs", check_enum_variant_docs);
}

#[test]
fn guardrails_enum_variant_docs_tests() {
    check_tests_files("enum variant docs (tests)", check_enum_variant_docs);
}
```

  - Run `cargo nextest run guardrails_enum_variant_docs` → 2 FAIL (existing enums lack docs). This is the expected red state for Phase 2.

#### Task 1.2: Lock guardrail green-path with a synthetic trivial case (1 SP)
- [ ] ##### SubTask 1.2.1: Mark `Severity` in `mod.rs` trivial (1 SP)
  - `tests/infrastructure/guardrails/mod.rs:14` declares `pub enum Severity { Error, Warning }`. Mark it trivial to unblock the harness itself:

```rust
/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}
```

  - Rerun `cargo nextest run guardrails_enum_variant_docs_tests` → still fails for `tests/helpers/sqlite_test_app_builder.rs:33` `BackendSpec`. Hand off to Phase 2.

---

### Phase 2: Migrate Existing Enums

#### Task 2.1: Mark trivial enums with `/// [TRIVIAL_ENUM]` (3 SP)

For each file below, read it in full before editing, then insert `/// [TRIVIAL_ENUM]` on the line directly above each listed enum. No per-variant docs needed.

| File | Enum | Line |
|------|------|------|
| `src/adapters/driven/storage/backend/core.rs` | `Backend` | 61 |
| `src/adapters/driven/storage/backend/core.rs` | `BackendKind` | 67 |
| `src/adapters/driven/storage/backend/test_support.rs` | `ErrorKind` | 15 |
| `src/domain/engine/action.rs` | `Action` | 5 |
| `src/domain/model/agent.rs` | `ExecutionPhase` | 10 |
| `src/domain/model/agent.rs` | `BackendSelector` | 18 |
| `src/domain/model/agent.rs` | `Confidence` | 73 |
| `src/domain/model/agent.rs` | `AgentResult` | 87 |
| `src/domain/model/llm_backend.rs` | `LlmBackendType` | 7 |
| `src/domain/model/map.rs` | `Direction` | 53 |
| `src/domain/model/prompt_preset.rs` | `PresetType` | 7 |
| `src/domain/model/trigger.rs` | `ComparisonOperator` | 9 |
| `src/domain/model/state/message_types.rs` | `MessageType` | 8 |
| `tests/helpers/sqlite_test_app_builder.rs` | `BackendSpec` | 33 |

- [ ] ##### SubTask 2.1.1: Mark trivial enums in `domain/model/` (1 SP)
- [ ] ##### SubTask 2.1.2: Mark trivial enums in `adapters/`, `engine/`, `state/`, `tests/` (2 SP)

**Per-file validation** after each subtask: `cargo nextest run guardrails_enum_variant_docs 2>&1 | grep <file>` → that file no longer flagged for the marked enums. Final validation at end of Phase 2.

#### Task 2.2: Add per-variant `///` docs to non-trivial enums (split into clusters) (5 SP total)

For each non-trivial enum, read existing system docs (`docs/system/*.md`, `docs/adr/*`, `docs/diagnostics/error_catalog.md`) to extract per-variant semantics; write **one concise `///` line per variant**. Variant doc must be semantic (what the variant *means*, when emitted), not "What" narration.

| File | Enum | Variant count | Source docs to consult |
|------|------|---------------|------------------------|
| `src/error.rs:7` | `LlmFailure` | 5 | `docs/diagnostics/error_catalog.md`, ADR-032 |
| `src/error.rs:24` | `NarrativeFailure` | 2 | `docs/diagnostics/error_catalog.md` |
| `src/error.rs:68` | `EngineError` | 21 | `docs/diagnostics/error_catalog.md`, `docs/system/*.md` |
| `src/application/action_pipeline/phase_error.rs:7` | `PhaseError` | 5 | ADR-032, `docs/system/action_pipeline.md` |
| `src/application/errors.rs:6` | `ApplicationError` | 4 | `docs/system/action_pipeline.md` |
| `src/application/errors.rs:61` | `ProcessActionResult` | 3 | ADR-030, `docs/system/action_pipeline.md` |
| `src/application/generation_gate/slot.rs:9` | `GenerationSlot` | 2 | ADR-030 |
| `src/application/narrative_prompt/types.rs:11` | `PromptLayer` | 7 | ADR-022 |
| `src/application/ports/text_checker.rs:35` | `IssueKind` | 6 | `docs/system/text_check.md` |
| `src/domain/model/settings.rs:9` | `TextCheckMode` | 4 | `docs/system/text_check.md` |
| `src/domain/model/state/generation_status.rs:7` | `GenerationStatus` | 3 | ADR-030, `docs/system/game_flow.md` |
| `src/domain/model/state/generation_status.rs:28` | `GenerationPhase` | 3 | `docs/system/action_pipeline.md` |
| `src/domain/model/quantifier.rs:8` | `QuantifierConfidence` | 3 | Already documented — verify only |
| `src/domain/model/quantifier.rs:68` | `MovementType` | 3 | Already documented — verify only |
| `src/domain/model/quantifier.rs:99` | `NpcTransitionType` | 2 | Already documented — verify only |

Example shape (from `PhaseError`):

```rust
pub enum PhaseError {
    /// Generation cancelled by user; partial artifacts discarded, state rolled back.
    Cancelled,
    /// Narrator LLM call failed; inner carries forensics for retry decision.
    NarratorFailed(Issues),
    /// Persistence gate rejected the write; game state unchanged.
    PersistFailed(#[source] EngineError),
    /// Trigger referenced by id was not present in the scenario index.
    TriggerMissing(String),
    /// Snapshot expected at this phase was missing from storage.
    SnapshotMissing,
}
```

- [ ] ##### SubTask 2.2.1: Document error.rs enums (LlmFailure, NarrativeFailure, EngineError) (3 SP)
  - `EngineError` alone is 21 variants; consult `docs/diagnostics/error_catalog.md` for variant semantics. Each variant already has `#[error("...")]` — rephrase into a semantic one-liner, do not duplicate the user-facing message verbatim.
  - Validation: `cargo nextest run guardrails_enum_variant_docs` → no `error.rs` violations.
- [ ] ##### SubTask 2.2.2: Document application + state + ports enums (1 SP)
  - `PhaseError`, `ApplicationError`, `ProcessActionResult`, `GenerationSlot`, `PromptLayer`, `IssueKind`, `TextCheckMode`, `GenerationStatus`, `GenerationPhase`.
  - Validation: `cargo nextest run guardrails_enum_variant_docs` → PASS overall.
- [ ] ##### SubTask 2.2.3: Verify existing quantifier.rs docs are complete (1 SP)
  - Read `src/domain/model/quantifier.rs:1-130` in full. Confirm `QuantifierConfidence`, `MovementType`, `NpcTransitionType` variants all have `///`. Confirm none of these enums carry `[TRIVIAL_ENUM]` marker (would trigger conflict rule).
  - Validation: `cargo nextest run guardrails_enum_variant_docs` → PASS overall. `python build.py` → green.

---

### Phase 3: Skill + AGENTS.md Updates

#### Task 3.1: Update `chronicler-comment-fixer` skill (1 SP)
- [ ] ##### SubTask 3.1.1: Add variant-doc allowlist rule (1 SP)
  - Modify `.agents/skills/chronicler-comment-fixer/SKILL.md`: add a section "Enum Variant Docs" stating that `///` on enum variants is **allowed and required** for non-trivial enums; the `comment_finder.py` script must not flag them. Trivial enums use `/// [TRIVIAL_ENUM]` opt-out marker. A trivial-marked enum with variant docs is a violation (handled by guardrail, not this skill).
  - Add example to the detection table:
    ```
    | Enum variant narration prose | DELETE unless non-trivial |
    | Variant `///` semantic doc | KEEP — required by guardrail |
    ```

#### Task 3.2: Update `chronicler-docs-hygiene` skill (1 SP)
- [ ] ##### SubTask 3.2.1: Add anti-paraphrase rule (1 SP)
  - Modify `.agents/skills/chronicler-docs-hygiene/SKILL.md`: add rule under "Detection Targets" that system/reference docs must not re-paraphrase enum variant semantics. If a doc needs to mention a variant, link to the enum's source (or omit). Variant semantics live in code comments; docs cover behavior and cross-variant flows only.
  - Validation: visual inspection. No code change.

#### Task 3.3: Update `chronicler_engine/AGENTS.md` (1 SP)
- [ ] ##### SubTask 3.3.1: Document the new convention (1 SP)
  - Modify `chronicler_engine/AGENTS.md` DOCUMENTATION STRATEGY section: add point "6. Enum Variant Docs" — rule: every variant carries `///` rustdoc, OR enum is marked `/// [TRIVIAL_ENUM]` directly above `enum` keyword. Both at once = violation. Variant semantics live on the variant; do not duplicate in `.md`.
  - Modify CONVENTIONS section: add bullet "Every enum variant carries `///` rustdoc unless the enum is marked `/// [TRIVIAL_ENUM]` directly above the `enum` keyword. Never both."
  - Add to ANTI-PATTERNS: "Never re-paraphrase enum variant semantics in `docs/system/*.md` or `docs/reference/*.md`; the variant doc is the source of truth."
  - Validation: `python scripts/generate_docs_index.py` runs clean.

---

## Test Plan
- **Guardrail unit tests** (5): trivial-marker skip, missing-doc flag, documented pass, empty enum, trivial-with-variant-docs conflict. Live in `enums.rs`.
- **Whole-codebase guardrail tests** (2): `guardrails_enum_variant_docs`, `guardrails_enum_variant_docs_tests`. Must PASS after Phase 2.
- **Existing guardrail tests**: no regression by running `cargo nextest run guardrails` — all prior `guardrails_*` tests still PASS.
- **Build gate**: `python build.py` (fmt + clippy + guardrails + tests + coverage-summary) → EXIT 0.
- **Comment-fixer skill dry-run**: `python ../.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --all` → no false positives on enum variant docs.

## Per Task/Sub Task Validation Steps

| Task | Validation |
|------|------------|
| 1.1.1 | `cargo nextest run check_enum_variant_docs` → 5 FAIL before mod declaration, 5 PASS after 1.1.2 |
| 1.1.2 | `cargo nextest run check_enum_variant_docs` → 5 PASS |
| 1.1.3 | `cargo nextest run guardrails_enum_variant_docs` → FAIL (existing enums lack docs) — expected red |
| 1.2.1 | `cargo nextest run guardrails_enum_variant_docs_tests` → `Severity` PASS; still fails on `BackendSpec` (→ Phase 2) |
| 2.1.1, 2.1.2 | `cargo nextest run guardrails_enum_variant_docs` → no violations on marked files; also no trivial-conflict violations |
| 2.2.1, 2.2.2, 2.2.3 | `cargo nextest run guardrails_enum_variant_docs` → fully PASS after 2.2.3 (both missing-doc and trivial-conflict directions clean) |
| 3.1.1 | Manual review of `.agents/skills/chronicler-comment-fixer/SKILL.md` |
| 3.2.1 | Manual review of `.agents/skills/chronicler-docs-hygiene/SKILL.md` |
| 3.3.1 | `python scripts/generate_docs_index.py` → exit 0 |
| Final | `python build.py` → exit 0 |

## Assumptions
- `syn` version already in `tests/infrastructure/guardrails/` Cargo tree (confirmed: `structure.rs`, `style.rs` use `syn::visit::Visit` and parse files). No new dependency needed.
- Existing `Violation::error` API matches signature `error(file: &str, line: usize, message: impl Into<String>) -> Self`.
- `check_src_files` / `check_tests_files` helpers in `mod.rs` discover all `.rs` files under `src/` and `tests/` respectively — applies to `test_support/`, `tests/helpers/`, and any `#[cfg(test)]` module declared inside a src file (the syn parser sees the source text regardless of `cfg`).
- Triviality classification in the migration table is based on variant names; if implementation reveals non-obvious semantics (e.g. `AgentResult::NoOp`), implementer may flip enum from trivial to documented without plan amendment — note in commit message.
- `EngineError` variant list (21) may have grown since the audit; implementer re-reads `error.rs` in full before documenting and adjusts variant count.
- `[TRIVIAL_ENUM]` marker is plain rustdoc text — `cargo doc` will render it as enum-level prose. Acceptable trade-off. Future cleanup could move to `#[doc(hidden)]`-style inert attribute but that requires helper macro — out of scope.
- No backward compatibility concerns per AGENTS.md "Do not preserve backward compatibility unless asked".
- Plan does **not** include pruning existing `.md` re-paraphrasing — that is separate drift cleanup work to be done after this rule lands. Docs-hygiene skill gains the rule to *prevent* new re-paraphrasing; existing `.md` cleanup is a follow-up.
- Subagent story points: Tasks 1.1 (3 SP) and 2.2.1 (3 SP) appropriate for `general-purpose` subagent with primary-agent verification. Task 2.2.1 is highest-risk (21-variant EngineError) — primary agent spot-checks ≥3 variant docs for accuracy against `docs/diagnostics/error_catalog.md`.
