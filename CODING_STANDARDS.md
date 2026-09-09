# Coding Standards

## Implementation

Do not preserve backward compatibility unless the user asks for it.

Symbols (functions, types, variables) must use verbose, domain-aligned names that map 1-to-1 with concepts in the `docs/` (where such symbols are present).

HTTP form structs (`axum::extract::Form` payloads) must deserialize from partial urlencoded bodies: mark fields a legacy poster may omit with `#[serde(default)]`, and encode checkbox groups as one boolean field per value.

## Code comments

Every file in `src/` (excluding `*_tests.rs` unit tests and `src/test_support/*.rs`) has:
 - Line 1: `//! [DOC: docs/diataxis/reference/<area>/<name>.md]` (links to reference documentation; `reference/` only — no `explanation/`, `how-to/`, or `tutorials/` targets). `src/test_support/*.rs` files must NOT carry a DOC anchor — the structure guardrail exempts them.
 - Line 2: `//! Human-readable summary` (used for auto-generating Structure section; required for `src/test_support/*.rs` too)
   Function-level anchors removed.

Never write comments that paraphrase what the code does. If the code isn't clear, rename the symbols rather than comment. Comments explain the WHY only when non-obvious: a hidden constraint, behavior that would surprise a reader.

Never reference the task in code comments ("used by X flow", "added for issue Y", "TODO from review"). Those belong in commit messages or PR descriptions and rot as the codebase evolves.

## Code Reviews

Do not try to compile, build or test the code when doing code reviews. Unless the review explicitly calls for it (e.g. the `test-police` review). 

During code reviews, avoid vague architectural critiques and focus on actionable changes.

## Testing

Read `tests/AGENTS.md` and `tests/STRATEGY.md` when writing or reviewing tests.

## Documentation

Read `docs/AGENTS.md` when writing or reviewing documentation.