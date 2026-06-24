# chronicler-docs-consistency Skill

## Purpose

This skill validates documentation **content accuracy** - not just whether docs exist, but whether they correctly describe what the code does.

Checks include:
- Behavior claims match implementation
- Function/data signatures are accurate
- No missing critical details
- No "ghost features" (described but nonexistent)
- No outdated architectural patterns
- All cross-references valid

## How It Works

The skill validates documentation structure and cross-references:

1. **Extract references** - Parse module paths, function names, APIs from docs
2. **Verify against code** - Use LSP/search to confirm references exist
3. **Check internal consistency** - Validate links, ADR numbering, plan archives
4. **Report mismatches** - Specific file:line locations with fixes

## Example Usage

### Basic invocation

```
Check if the Chronicler Engine docs are up to date
```

### After a refactor

```
/docs audit - I just refactored the storage layer, make sure the architecture doc matches
```

### Pre-release verification

```
Run a documentation consistency check before we ship
```

## What It Checks

The skill compares **doc claims** against **code reality**:

### Behavior Validation
-Doc says "validates input" → code has no validation
- Doc says "async" → code is sync
- Doc describes workflow → code has different flow

### Signature Accuracy  
- Function signatures match (`fn foo(x: u32)` vs `fn foo(x: &str)`)
- Data schemas correct (required vs optional fields)
- API contracts accurate (headers, params, return types)

### Completeness
- Missing auth requirements
- Undocumented side effects
- Omitted error cases

### Ghost Features
- "Batch operations supported" → no batch API
- "Cached results" → no caching implemented
- "Retry logic" → no retries

### Architectural Drift
- Describes removed components
- References old patterns (e.g., sync vs async, monolith vs service)
- Wrong tier responsibilities

### Cross-Reference Integrity
- Internal `[]()` links resolve
- ADR numbers exist and match topic
- Auto-index in `README.md` current

## Output Format

The skill returns structured output:

```markdown
Status: [PASS] or [FAIL]

# Documentation Inconsistencies:
- ARCHITECTURE: CRITICAL - Module 'foo' documented but doesn't exist
  - docs/architecture/system.md:L45
  Current: "The `foo` module handles..."
  Expected: Remove this section or create src/engine/foo.rs

# Changelog Gaps:
- 2026-05-30: Commit 7ada90a not documented
  Files changed: src/application/*.rs
  Suggested entry: "- **Refactored application service** (details...)"

# Actionable Fixes:
  docs/architecture/system.md:L45 - ARCHITECTURE - Remove non-existent module
  docs/CHANGELOG.md:after L35 - CHANGELOG - Add entry for 2026-05-30 commits
```

## Severity Levels

| Level | Meaning | Action Required |
|-------|---------|-----------------|
| CRITICAL | Architecture describes non-existent code | Fix immediately |
| HIGH | Changelog missing significant changes | Add within 24h |
| MEDIUM | System docs inaccurate, broken links | Fix before next release |
| LOW | Style issues, minor outdated refs | Batch fix periodically |

## Troubleshooting

**Skill reports false positives:**

The skill uses git-based heuristics. If it flags something that's actually fine:
- Changelog may use different wording than commit messages
- Architecture may describe planned structure (should be marked as "planned")

**Skill misses real issues:**

If docs are clearly wrong but skill says PASS:
- Changes may be older than the git window (10 commits by default)
- Manual audit may still be needed for semantic accuracy

**Running in fresh context:**

The skill operates independently. If invoked in a new session, it will:
1. Read current documentation
2. Run git commands to detect changes
3. Cross-reference against code structure
4. Report findings

No prior context required.

## Related Skills

- **chronicler-comment-fixer**: Checks Rust code comments (not docs/) for AI slop
- **code-consistency-check**: Validates code patterns match conventions
- **documentation-and-adrs**: Guides creating new documentation (not checking consistency)

## Files Modified

This skill is **read-only**:
- Reads: `chronicler_engine/docs/**/*`, `chronicler_engine/src/**/*`, git history
- Writes: Nothing (report only)

To fix issues, use the skill's actionable fixes with Edit tool manually.
