---
name: chronicler-comment-fixer
description: Remove AI-slop and restating comments from repo Rust, Python, HTML and CSS files.
---

# Run the pass

## 1. Find the comments

Pick the finder mode that matches the request. Its output is the scope.

```bash
# Mode 1: Uncommitted and untracked files (most common, after coding)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --uncommitted

# Mode 2: All Rust, HTML and CSS files (full codebase scan; skips Python)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --all

# Mode 3: Specific file or glob
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "src/foo.rs"
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "src/**/*.rs"
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --pattern "assets/*.css"
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --files src/foo.rs scripts/bar.py

# Mode 4: Files changed on this branch vs main (or a named base)
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --branch
python .agents/skills/chronicler-comment-fixer/scripts/comment_finder.py --branch develop
```

The script prints `path:line - comment_text`.

## 2. Classify every line, then act

Classify **every** comment line the finder returns, not a sample. A comment that passes both parts of the KEEP test below is kept or rewritten. Delete the rest.

Start with the densest files: a high comment-to-code ratio, or a ratio that jumped against the file's previous version. The KEEP test still decides each line.

Delete whole comment blocks. When you delete a head line, delete its continuation lines with it, or you leave an orphaned `///` that `cargo check` accepts and `clippy` rejects.

Report `classified N, kept K, rewritten R, deleted D`.

## 3. Verify

Re-run the finder with the same mode and confirm the count dropped by at least D. Then run `python build.py clippy`, which catches the orphaned-doc lint that `python build.py check` does not. That clippy run is the only gate this pass needs; leave the test suite alone.

# The KEEP test

Keep a comment only when **both** hold:

1. **Non-obvious.** It names a hidden constraint or a behaviour that would surprise a reader. Not a restatement of what the next line does. Not a reason a reader can derive.
2. **Not carried by the code.** The reason is not already recoverable from a symbol name, a type, a field, a function name, a test name, or an assertion message. If renaming the symbol would make the comment redundant, rename the symbol and delete the comment.

If either part fails, delete. If the reason is real but the code could carry it, delete and raise a reshape flag.

The default is delete. A comment that survives on a technicality is noise. The survivors are rare: an invariant with no symbol for it, a browser or OS behaviour that would surprise, an ordering constraint, a hidden coupling.

Worked examples, every one a delete:

| Comment | Why it fails |
|---|---|
| `/// The newest recorded attempt per agent, oldest-first.` on `newest_message_per_agent` | Carried by the name |
| `/// The display showed "Ready".` on `StatusOutcome::ready` | Carried by the field name |
| `// Lock the other entries' Edit controls so a second showEditForm cannot open a competing editor` on `lockOtherEditButtons()` | Carried by the name; the mechanism is visible in the body |
| `/// A cloneable handle to the scripted story-log shape.` on `StubStoryLogHandle` | Carried by the name |
| `// Options is mode-agnostic: its edit form carries no Allowed Modes flags.` above an `if preset_type == Options { String::new() }` branch | The branch shows it |
| `// Called from the save handler once the form posts.` | Narration of the call graph |
| `/// This function parses the request body.` | Restates the code |
| `//! This module provides helpers for X.` | "This module..." lead, no contract |
| `// Then we write the result back.` | Narration |
| `// === Helpers ===` | Separator with no reason |
| `/// Handles the request robustly.` | Praise, no contract |
| `# This function builds the report.` | Restates the code |
| `// TODO: fix this later` | Placeholder with no owner |
| `/// Render the header through the shipped HeaderTemplate, plus the out-of-band banner...` | "Render..." lead plus body restatement |

# Rewrite a survivor

Rewriting is for comments that pass the KEEP test but carry more than their reason. Run the test on the reason alone first: if nothing non-obvious is left once the restatement is cut, delete. Rewriting never rescues a comment that fails.

A rewrite keeps the reason and cuts everything else: restatement of the code, narration, a negative frame, a "This function..." lead. It is shorter than the original, usually one line.

```rust
// Here we sort the entries before saving them. We sort by timestamp
// because the snapshot loader assumes ascending order and will
// silently drop out-of-order rows.
```

Rewrite to `// The snapshot loader silently drops rows that arrive out of timestamp order.`

# Negative framing

Delete a comment that defines the code by what it is not, or disclaims what the code does not do. `// Not the same as the cache key` and `// This path does not validate` state the negative and add nothing. A negative statement that names a hidden constraint is judged by the KEEP test like any other comment; if it passes, rewrite it as the positive constraint.

# Carve-outs

Four comment classes are machine-enforced. Keep them:

- DOC anchors: `//! [DOC: ...]`.
- Module summaries: the `//!` line 2 of a file.
- Semantic enum-variant docs. The `check_enum_variant_docs` guardrail requires them, and `/// [TRIVIAL_ENUM]` above the enum opts out. `/// This variant represents the cancelled state.` is slop; `/// Generation cancelled by user; partial artifacts discarded.` is the semantic form.
- Feature-spec tags: `// [docs/specs/<spec>.md] SCENARIO: N.N`.

The anchor and module-summary rules live in `CODING_STANDARDS.md` (`## Code comments`) and `scripts/validate_docs.py --anchors`. Do not restate them here.

# Reshape flags

A comment written to defend a workaround or an unclear name shows the shape the code wants. Deleting the comment is half the fix. While judging each line, ask what made the comment necessary, and log the minimal code change that removes the reason.

```rust
// has to clone the whole roster because callers mutate the result; fine for now
pub fn roster(&self) -> Vec<Npc> { self.npcs.clone() }
```

Report: DELETE the comment. Reshape flag (OFFERED): `roster.rs:Session::roster - return &[Npc] and fix the two mutating callers`.

Reshape flags are proposals. An approved flag leaves this skill and runs as normal gated production work.

# Output format

```
Status: [PASS] or [FAIL]
Counts: classified N, kept K, rewritten R, deleted D

# Inconsistencies Found:
- (bullet points, each with a severity)

Severity levels:
- AI_SLOP: verbose "What" comments, generic praise, narration
- NOISE: a "why" comment the code already carries (fails KEEP part 2)
- STYLE: missing doc anchors, unanchored workarounds

# Actionable Fixes:
  FILE:LINES - Severity - Description
  Old: (snippet)
  New: (rewritten snippet, or empty for a delete)

# RESHAPE FLAGS: (OFFERED, FAIL reports only)
  FILE:SYMBOL - minimal shape change that removes the comment's reason
```
