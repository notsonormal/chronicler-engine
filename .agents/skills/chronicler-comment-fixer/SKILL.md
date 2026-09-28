---
name: chronicler-comment-fixer
description: Detect and report AI slop, "What" comments, missing doc anchors, and convention violations in the repo Rust, Python, HTML and CSS files.
---


# Script-Based Comment Discovery

Before manual searching, invoke the comment finder script to identify target comments:

```bash
# Mode 1: Uncommitted/new files (most common for review after coding)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --uncommitted

# Mode 2: All Rust, HTML and CSS files (full codebase scan)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --all

# Mode 3: Specific file pattern
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "src/foo.rs"
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "src/**/*.rs"
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "assets/*.css"

# Mode 4: Files changed in branch vs main (or custom base)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --branch
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --branch develop
```
The script outputs file paths, line numbers, and comment text in the format:
  path:line - comment_text

---

Read `CODING_STANDARDS.md`, section `## Code comments` — repo-level comment rules that the detection tables below extend.

# Remove the reason, not just the comment

A comment written to justify leaving the real problem unsolved shows the shape the code should have. Deleting the comment is half the fix. The reshape flag carries the other half.

`CODING_STANDARDS.md` holds the instance-level rule: if the code isn't clear, rename the symbols rather than comment. This section is the pass-level version. While judging each finding, ask what made the comment necessary. A workaround defended in prose, an unclear name explained in prose, a missing type described in prose: each one is a shape change the code wants. Log it as a reshape flag naming the minimal change that removes the reason, and offer it as the next action.

Three carve-outs remain, all guardrail-enforced: DOC anchors, module summaries, and semantic enum-variant docs. Every other comment earns its place by naming a reason the code cannot carry itself.

```rust
// has to clone the whole roster because callers mutate the result; fine for now
pub fn roster(&self) -> Vec<Npc> { self.npcs.clone() }
```

Report: DELETE the comment. Reshape flag (OFFERED): `roster.rs:Session::roster - return &[Npc] and fix the two mutating callers`.

# Detection Targets

## AI Slop Patterns (Rust)
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
| Justification comment defending a workaround or unclear code | `// has to clone the roster because callers mutate it; fine for now` | DELETE + reshape flag |

## Enum Variant Docs

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

## Python AI Slop

| Pattern | Action |
|---------|--------|
| "# This module/function..." leading | DELETE |
| Placeholder TODOs without owner | DELETE |
| AI slop phrases: "leverages", "utilizes", "robust", "seamless" | DELETE |

## Comment Density

### Increasing comment density

Check the comment density of the new file against the comment density of the old file. For example, 
if a 500-line file has 20 comments, then it would be strange for it to suddenly jump to 100 comments despite the size of the file only increasing by 300 lines. 

If the new comment density is much higher then you most certainly should be cutting them
more aggressively.

### Code to comment density

A 20 line function doesn't need 10 line comment. 

## Comments shouldn't explain obvious code

If you can understand the code by just reading the file then you don't need the comments. Comments should explain things that aren't immediately obvious.

## No negative explaining

Don't describe a thing by what it isn't, and don't editorialize about absences in body prose. A comment written with a negative frame is usually not written from a holistic perspective.

## Whether to trimming or remove

The value of a comment has to be consisted holistically. The natural inclination when you see a 10 line comment is to trim it, however, in some cases it might be better to just remove it entirely. 

## File/Module Comments

The first two lines of most production files will be a DOC module and a module comment e.g.

```rust
//! [DOC: docs/diataxis/reference/startup.md]
//! Command-line interface definitions
```

This is enforced by the guardrails. The second line is needed for auto-generating the STRUCTURE section in the AGENTS.md file.

Canonical anchor rules (mirrored by `scripts/validate_docs.py --anchors`):

- Anchor target must be a full repo path under `docs/diataxis/reference/` (e.g. `docs/diataxis/reference/storage.md`). `explanation/`, `how-to/`, `tutorials/` targets are rejected — source files associate with reference docs only.
- No section suffix. Path-only anchors.
- `src/test_support/*.rs` MUST NOT carry a `[DOC: ...]` line — shared test
  helpers are organised by fixture weight (ADR-028); a `//! <summary>` line
  on line 1 suffices.
- `tests/**/*.rs` MUST NOT carry a `[DOC: ...]` line.

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

# RESHAPE FLAGS: (OFFERED, FAIL reports only)
  FILE:SYMBOL - minimal shape change that removes the comment's reason
```

Reshape flags are proposals. An approved flag leaves this skill and runs as normal gated production work.

# Stay Focused On Fixing Comments

Do NOT build or run tests. Only run `python build.py check` after updating the comments to ensure the code still compiles. 