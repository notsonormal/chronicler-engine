---
name: code-consistency-check
description: Review code for architectural consistency with codebase patterns (not comment style). Use when user asks to verify code follows patterns, matches existing architecture, or checks for anti-patterns.
---

# Core Directives

- **Consistency over Optimization**: Match existing patterns, not "clever" code
- **Intra-Language Harmony**: Python follows Python patterns, Rust follows Rust patterns

# Output Format

```
Status: [PASS] or [FAIL]

# Inconsistencies Found:
- (List as bullet points with severity)

# Actionable Fixes:
  FILE:LINES - Severity - Description
  Old: (snippet)
  New: (snippet)
```

# Note

For comment style checking in chronicler_engine/, use **`chronicler-comment-fixer`** instead.