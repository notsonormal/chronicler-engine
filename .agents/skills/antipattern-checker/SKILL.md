---
name: antipattern-checker
description: Detect abstraction anti-patterns in Chronicler Engine Rust code.
---

# Antipattern checker

Find abstraction anti-patterns in Rust code that clippy and arch-lint cannot see, because they depend on what the code means. Read `CODING_STANDARDS.md` first.

Scope: the one the caller gives. Without one, use the uncommitted changes, or the branch diff against `main` when the tree is clean. State the scope you used.

## What to look for

- **Coincidental cohesion**: a module that groups items because they are used together, not because they share one concept.
- **False deduplication**: one shared function, trait default or type serves callers whose meaning differs, so a change for one caller changes the others. Also the inverse: two types with the same shape and meaning, kept apart, with `From` impls between them.
- **Symptom refactor**: an extraction that moves a problem instead of removing it. Examples: a constructor that returns an object that breaks its own invariant, a parameter that no code reads, a DTO that only flattens another type, a test-only type threaded through production code.
- **Re-implementation**: a new symbol whose job an existing symbol already does. For each new public symbol in scope, search the codebase for one.

These are classes, not a closed list. Report other abstraction problems of the same kind.

## Done when

- You read each file in scope in full. Base each finding on the whole file, not on a grep match.
- Each finding gives `file:line`, a verbatim quote, why it is a problem in this code, a severity (high: bugs or architectural drag; med: costlier change; low: cosmetic) and a concrete fix. The caller verifies each finding against the quote.
- The report ends with "Checked and clean": each class and each new public symbol that you checked and found nothing for.
- You stayed inside the scope.
