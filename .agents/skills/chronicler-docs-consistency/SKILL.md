---
name: chronicler-docs-consistency
description: Detect and report documentation drift, inconsistencies, and staleness in Chronicler Engine docs compared to actual codebase implementation.
---

# Core Directives

**Docs Must Describe Reality**: Documentation should accurately explain what the code does, not just name existing modules.

**Specificity Over Vagueness**: Good docs use concrete terms (function names, data flows, invariants) not generic descriptions.

**Cross-Reference Integrity**: All internal doc links, ADR references, and module paths must resolve.

**No Build Required**: This skill updates documentation files only. Do NOT run `cargo build`, `cargo test`, `python build.py`, or any compilation/test commands — updating `.md` files does not require build verification.
# What to Check

The skill validates documentation **content accuracy**:

| Check Type | What to Verify | Red Flags |
|------------|---------------|-----------|
| **Behavior Mismatch** | Does described behavior match code? | "Validates input" but function has no validation |
| **Wrong Signatures** | Are function/data signatures accurate? | Doc says `fn foo(x: u32)` but code is `fn foo(x: &str)` |
| **Missing Concepts** | Does doc skip important details? | Describes API but omits required auth header |
| **Ghost Features** | Does doc describe nonexistent features? | "Supports batch operations" but no batch API exists |
| **Outdated Patterns** | Does doc reference old architecture? | Describes monolithic service after microservice refactor |
| **Broken References** | Do links and cross-refs work? | `[]()` links to missing section, ADR# not found |

The skill reads documentation and compares **claims** against code:

1. **Extract Claims** - Parse behavioral descriptions, signatures, data flows from docs
2. **Verify Implementation** - Check if described behavior/signatures match actual code
3. **Flag Contradictions** - Report specific mismatches between doc claims and code reality
4. **Check References** - Validate links, ADR numbers, cross-doc citations

## Examples

**Doc claims:** "Movement validation prevents invalid room transitions"
**Skill checks:** Does movement code actually validate? Are there bypasses?
**→ FLAG if:** Validation claimed but not implemented

**Doc claims:** "`player.health` is required (u32)"
**Skill checks:** Is field `Option<u32>` or `u32`?
**→ FLAG if:** Doc says required but code has `Option`

**Doc claims:** "Uses SSE for real-time updates (ADR-002)"
**Skill checks:** Does ADR-002 exist? Does code use SSE?
**→ PASS if:** Both ref and implementation valid

# Output Format

```markdown
Status: [PASS] or [FAIL]

# Documentation Inconsistencies:
- DOC_TYPE: Description
  - FILE:LINES - Current vs Expected
  - Example: "Architecture mentions module X which doesn't exist"

# Actionable Fixes:
  FILE:LINES - Doc Type - Description
  Current: (snippet from docs)
  Expected: (what it should say / code truth)
```

Document types (report all that apply):
- **ARCHITECTURE**: `docs/architecture/system.md` doesn't match `src/`
- **SYSTEM**: `docs/system/*.md` describes old behavior
- **ADR**: `docs/adr/*.md` references removed code
- **REFERENCE**: `docs/reference/*.md` has wrong schemas/APIs
- **CROSS_REF**: Broken links or stale references

- **chronicler-comment-fixer**: Detects AI slop and missing doc anchors in Rust code comments (not docs/)
- **code-consistency-check**: Validates code patterns match architectural conventions
- **documentation-and-adrs**: Guides creating and updating documentation (not checking consistency)
