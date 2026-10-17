# Stop the comment pass from deleting code inside string literals

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The comment pass deleted two `// MEASURE_CLAMPED_TEXT` lines in `tests/browser/stub/failure_display.rs`. They look like comments, but they are code: each sits inside a JavaScript probe in a Rust raw string, and `.replace("// MEASURE_CLAMPED_TEXT", MEASURE_CLAMPED_TEXT)` swaps the line for the `clampedText` helper. With the lines gone, the probe threw `ReferenceError: clampedText is not defined`. Clippy passed. Only the browser tier caught the problem, one step later, in another agent's gate run.

Three things let it through:

- **The finder lists string contents as comments.** `comment_finder.py` reads lines, not tokens, so a `//` line inside `r#"…"#` counts as a Rust comment.
- **The skill tells the pass to skip tests.** `chronicler-comment-fixer/SKILL.md:44`: "That clippy run is the only gate this pass needs; leave the test suite alone." The premise that a comment cannot change behaviour is false for text inside a string literal.
- **The marker looks like a comment.** A substitution target written as `// NAME` invites deletion, and nothing next to it says it is load-bearing.

Where should the fix go?

- **Make it unrepresentable.** Write substitution targets so that they are not comments (for example `__MEASURE_CLAMPED_TEXT__`, or build the script with `format!`/`concat!`). The finder then has nothing to list. A guardrail can forbid `.replace("//` in `tests/browser/`.
- **Teach the finder about string literals.** Skip lines inside Rust raw strings (`r#"` … `"#`) and ordinary strings. This is more robust, but a line-based scanner needs a small tokenizer.
- **Widen the verify step.** The pass runs the test tiers of the files it touched (`test-pattern`, or `browser` for `tests/browser/`). This catches the error but does not prevent it, and it makes the pass slower.

## Context

- Source: `/retro` 2026-10-11 on the `/simple-after-plan` pass over tickets 15 and 16 of the dashboard UI review phase 2. Lead session `01a12cf3-c68e-7568-beff-572aec227289`; comment-fixer session `01a12d26-ae8a-7758-8695-8ed76a413d32`; fix session `01a12d34-5d65-71e2-9f41-20525be037e4`.
- Finder output in the comment-fixer session: `tests/browser/stub/failure_display.rs:778 - // MEASURE_CLAMPED_TEXT` and `:943`. Its classification: "markers labeling the JS const in a raw string … It's a separator/label → DELETE".
- Its "code-line invariance" check compared the non-comment lines of `git diff -U0` before and after the pass. It classed the deleted lines as comments, so the check passed.
- The red run: `logs/build_20261011_230606.log`, browser 28/29, `run_failure_display_slot_checks`. The fix restored both lines and added a sentence to the `MEASURE_CLAMPED_TEXT` doc saying that the marker is load-bearing (`tests/browser/stub/failure_display.rs:802`).
- The lead's brief also asked for clippy only, as the skill says.
- Related: [Scope the comment pass to the lines the diff adds](12-comment-pass-scope-to-the-diff.md). Both markers were older than the change, but a narrower scope is not the fix: the user ruled on 2026-10-11 that the pass must also clean older comments in touched files. The fix belongs on the string-literal side. [Cover JavaScript in the comment finder](37-comment-finder-javascript-coverage.md): a JS-aware finder must also tell JavaScript inside a Rust string from Rust.
