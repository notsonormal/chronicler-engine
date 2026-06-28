---
name: chronicler-comment-fixer
description: Detect and report AI slop, "What" comments, missing doc anchors, and convention violations in chronicler_engine/ Rust and Python files.
---


# Script-Based Comment Discovery

Before manual searching, invoke the comment finder script to identify target comments:

```bash
# Mode 1: Uncommitted/new files (most common for review after coding)
cd chronicler_engine && python ../.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --uncommitted

# Mode 2: All rust files (full codebase scan)
cd chronicler_engine && python ../.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --all

# Mode 3: Specific file pattern
cd chronicler_engine && python ../.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "src/foo.rs"
cd chronicler_engine && python ../.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "src/**/*.rs"

# Mode 4: Files changed in branch vs main (or custom base)
cd chronicler_engine && python ../.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --branch
cd chronicler_engine && python ../.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --branch develop
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

Do build or run tests. 