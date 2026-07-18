---
name: chronicler-comment-fixer
description: Detect and report AI slop, "What" comments, missing doc anchors, and convention violations in chronicler_engine/ Rust and Python files.
---


# Script-Based Comment Discovery

Before manual searching, invoke the comment finder script to identify target comments:

```bash
# Mode 1: Uncommitted/new files (most common for review after coding)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --uncommitted

# Mode 2: All rust files (full codebase scan)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --all

# Mode 3: Specific file pattern
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "src/foo.rs"
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "src/**/*.rs"

# Mode 4: Files changed in branch vs main (or custom base)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --branch
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --branch develop
```
The script outputs file paths, line numbers, and comment text in the format:
  path:line - comment_text

---

Review the file `chronicler_engine\AGENTS.md` to understand the coding standards around comments.

# Detection Targets

### AI Slop Patterns (Rust)
| Pattern | Example | Action |
|---------|---------|--------|
| Verbose module docs (3+ `//!`) | `//! This module handles...` | DELETE |
| "This [module/function]..." leading | `//! This module provides...` | DELETE |
| "Inspired by..." what comment | `//! Inspired by X` | DELETE |
| "What" doc | `/// This function parses...` | DELETE |
| Generic praise | "well-designed", "robust", "efficient" | DELETE |
| Narration comments | `// This does X`, `// Then we do Y` | DELETE |
| Separator comments | `// === Section ===` | DELETE |
| Enum variant narration prose | `/// This variant represents...` | DELETE (rephrase as semantic, see below) |

# Enum Variant Docs

Enum variant `///` comments are **allowed and required** for non-trivial enums. The
`check_enum_variant_docs` guardrail enforces this. The opt-out marker
`/// [TRIVIAL_ENUM]` directly above the `enum` declaration signals that variants
are self-documenting; no variant `///` may appear on a trivial-marked enum.

A variant doc must be **semantic** — what the variant *means* or *when it is
emitted* — not "What" narration.

| Variant doc form | Verdict |
|-----------------|---------|
| `/// Generation cancelled by user; partial artifacts discarded.` | KEEP — semantic |
| `/// This variant represents the cancelled state.` ("This variant...") | DELETE — slop |
| `/// Red hue.` on `Color::Red` | DELETE — trivial, use `[TRIVIAL_ENUM]` |

### Python AI Slop

| Pattern | Action |
|---------|--------|
| "# This module/function..." leading | DELETE |
| Placeholder TODOs without owner | DELETE |
| AI slop phrases: "leverages", "utilizes", "robust", "seamless" | DELETE |

# File/Module Comments

The first two lines of most production files will be a DOC module and a module comment e.g.

```rust
//! [DOC: docs/system/startup.md]
//! Command-line interface definitions
```

This is enforced by the guardrails. The second line is needed for auto-generating the STRUCTURE section in the AGENTS.md file.

# Output Format

```
Status: [PASS] or [FAIL]

# Inconsistencies Found:
- (List as bullet points with severity)

Severity levels:
- AI_SLOP: Verbose "What" comments, generic praise
- STYLE: Missing doc anchors, unanchored workarounds

# Actionable Fixes:
  FILE:LINES - Severity - Description
  Old: (snippet)
  New: (snippet)
```

# Stay Focused On Fixing Comments

Do NOT build or run tests. Only run a simple `cargo check` after updating the comments to ensure the code still compiles. 