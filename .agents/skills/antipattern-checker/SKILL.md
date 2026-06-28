---
name: antipattern-checker
description: Detect abstraction anti-patterns in Chronicler Engine Rust code. Explicit invocation only.
user-invocable: true
---

# Antipattern Checker

Detects **semantic** abstraction anti-patterns that static tools (clippy, arch-lint) cannot catch. Used for regression prevention during code review.

**Scope (user specifies at invocation):**
- **Uncommitted changes** (default if in dirty branch) — `git diff --name-only`
- **Branch diff** — vs main/develop
- **Specific file/module** — user provides path
- **Full codebase** — warn: expensive (~10-30 min)

If user doesn't specify scope, ask. Default to uncommitted if available, else branch diff.

---

## What to Detect

### 1. Coincidental Cohesion

Modules grouping items by "used together" rather than shared concept.

**Signals:**
- Module contains unrelated types spanning multiple subsystems
- Generic filenames: `misc.rs`, `util.rs`, `helpers.rs`, `common.rs`
- Module named for use ("fragment utilities") not concept ("text_check")
- 10+ types in one file with no shared behavior

**Examples from codebase:**
- `model/state.rs` — 11 unrelated types: `MessageType`, `GenerationStatus`, `MovementState`, `NarrativeState`, `SceneState`, `GameState`...
- `server/fragments/misc.rs` — text_check, retry, retrigger, switch_swipe handlers in one file

---

### 2. False Deduplication

Code looks similar; semantics differ. Merging couples unrelated callers.

**Signals:**
- Two types with same shape but different domain meaning, kept separate by module boundary
- Trait default applies provider-specific behavior to all impls
- "Generic" function with `Option<&str>` flags acting as hidden mode switches
- Identical impls across N backends because trait default is empty

**Examples from codebase:**
- `Confidence` vs `QuantifierConfidence` — identical enums, bidirectional `From` impls
- `sanitize_llm_output` — strips gemma-4 markers from ALL backends
- `configure_request` — carries OpenRouter-specific `X-Title`/`HTTP-Referer` headers

---

### 3. Refactor-be-damned

Extraction treating symptom, not root cause.

**Signals:**
- Constructor returns object violating documented invariant
- Function param prefixed `_` or never read
- New module re-implements pipeline steps by hand instead of parameterizing
- Single-caller helper extracted "for clarity"
- Type duplicates another type's fields with sync methods
- DTO exists solely to flatten another type for serialization
- Test-only enum threaded through production code

**Examples from codebase:**
- `Message::from_db` — produces invalid domain object (empty text, no swipes)
- `_player_name` param in `execute_action_impl` — never read
- `ArrivalTaskContext` — 13-field re-implementation of the pipeline
- `MessageEntry` DTO — mirrors `Message`+`Swipe`, no behavior

---

## Methodology

Every finding **must** include:

1. **File:line** — exact reference
2. **Evidence** — verbatim quote (1-5 lines)
3. **Why smell** — semantic reasoning, not restating pattern name
4. **Severity** — high / med / low
5. **Type** — fundamental (architectural) / mechanical (trivial)
6. **Proposed fix** — concrete direction

**No quote → no finding. No fix direction → no finding.**

---

## Severity Rubric

| Severity | Definition |
|----------|------------|
| **high** | Produces bugs, maintenance burden, or architectural drag |
| **med** | Increases cost of change, couples unrelated concerns |
| **low** | Cosmetic, mechanical cleanup |

---

## Output Format

```markdown
# Anti-Pattern Report: [Scope]

## Summary
- N findings across M files
- Severity: X high, Y med, Z low

## Findings

### 1. [Category] Short title

- **File:** `path/to/file.rs:line`
- **Evidence:**
  ```rust
  verbatim quote of offending code
  ```
- **Why smell:** (semantic reasoning, not restating pattern name)
- **Severity:** high/med/low
- **Type:** fundamental / mechanical
- **Proposed fix:** (concrete direction)

[... repeat for each finding ...]

## Cross-cutting notes
- Patterns recurring across multiple files
- Architecture-level observations

## Positive patterns (optional)
- X is OK because Y (prevents doomsday tone)
```

---

## Execution Flow

1. **Determine scope** — ask user if not specified
2. **List files** — describe files to be reviewed (don't grep, just list)
3. **Read files fully** — no grep-only findings. If a file is in scope, read it completely before reporting on it.
4. **Apply framework** — check each file against 3 categories
5. **Report all** — no cap, no filtering. Known findings from corpus included.
6. **Stop when scope exhausted** — do not audit adjacent files