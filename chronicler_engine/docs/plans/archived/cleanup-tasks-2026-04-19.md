# Chronicler Engine Cleanup Tasks

## TL;DR

> **Quick Summary**: Remove unused dependencies, fix 4 production .unwrap() violations, rename tui_state to generation_state, and enhance prompt injection filtering.
>
> **Deliverables**:
> - Cleaned Cargo.toml (removed unused tokio-tungstenite)
> - Cleaned index.html (removed unused SSE.js)
> - Safe openrouter_client.rs (no .unwrap() in production)
> - Renamed tui_state → generation_state across codebase
> - Enhanced sanitize_for_prompt() with instruction-pattern filtering
>
> **Estimated Effort**: Small (< 2 hours)
> **Parallel Execution**: NO - sequential file edits with LSP rename for tui_state
> **Critical Path**: Task 4 (LSP rename) → Tasks 1,2,3,5 (independent)

---

## Context

### Original Request
User requested cleanup based on critique review:
1. Remove unused `tokio-tungstenite` from Cargo.toml
2. Remove unused SSE.js script from index.html
3. Fix 4 production `.unwrap()` violations in `openrouter_client.rs`
4. Rename `tui_state` → `generation_state`
5. Add instruction-pattern filtering to `sanitize_for_prompt()`

### Verification Strategy
- Run `cargo build` after each task to verify no breakage
- Run `cargo test` at end to verify all tests pass
- Verify SSE removal by checking no `hx-sse` attributes exist anywhere

---

## Execution Strategy

### Sequential + Parallel Mix

> LSP rename must go FIRST (multi-file atomic rename).
> After Task 4 completes, Tasks 1, 2, 3, 5 can run in parallel (independent).

```
Step 1: Task 4 (LSP rename tui_state → generation_state)
         ↓
Step 2 (parallel): Task 1, Task 2, Task 3, Task 5
```

---

## TODOs

- [x] 1. Remove unused `tokio-tungstenite` from Cargo.toml

  **What to do**:
  - Read `Cargo.toml` and remove the `tokio-tungstenite = "0.21"` line from dependencies
  - Run `cargo build` to verify no compilation errors (the dependency is unused but removing it should not break anything)

  **Must NOT do**:
  - Remove any other dependencies

  **References**:
  - `Cargo.toml` - Dependency list (line contains tokio-tungstenite)
  - Research confirmed: tokio-tungstenite is planned but never implemented, no `/ws` route exists in `src/server/mod.rs`

  **Acceptance Criteria**:
  - [ ] `grep -n "tungstenite" Cargo.toml` returns no matches
  - [ ] `cargo build` succeeds without errors

  **Commit**: YES
  - Message: `chore(deps): remove unused tokio-tungstenite dependency`
  - Files: `Cargo.toml`

- [x] 2. Remove unused SSE.js script from index.html

  **What to do**:
  - Read `assets/index.html`
  - Remove `<script src="https://unpkg.com/htmx.org@1.9.10/dist/ext/sse.js"></script>` line
  - Verify no `hx-sse` attributes exist anywhere in the codebase (use grep)

  **Must NOT do**:
  - Remove any other scripts or HTMX imports

  **References**:
  - `assets/index.html` line 8 - SSE.js script tag (unused, no hx-sse usage found)
  - Research confirmed: SSE.js loaded but never used, no server-side SSE endpoint exists

  **Acceptance Criteria**:
  - [ ] SSE.js script tag removed from index.html
  - [ ] `grep -rn "hx-sse" .` returns no matches (no SSE usage anywhere)
  - [ ] `cargo build` succeeds (no Rust code depends on SSE)

  **Commit**: YES
  - Message: `chore(html): remove unused SSE.js script`
  - Files: `assets/index.html`

- [x] 3. Fix production `.unwrap()` violations in openrouter_client.rs

  **What to do**:
  - Read `src/narrative/openrouter_client.rs`
  - At lines 129, 140, 151, replace `.unwrap()` with proper `and_then()` or `map()`:
    ```rust
    // Line 129: c.unwrap() → c.and_then(|s| Some(s.to_string()))
    // Line 140: r.unwrap() → r.and_then(|s| Some(s.to_string()))
    // Line 151: rc.unwrap() → rc.and_then(|s| Some(s.to_string()))
    ```
  - The values come from `is_non_empty()` guards, so after the guard they should be `Some(v)` not `None`

  **Must NOT do**:
  - Change the logic, only change the error handling

  **References**:
  - `src/narrative/openrouter_client.rs:129-151` - .unwrap() calls after is_non_empty() guards
  - `src/narrative/openrouter_client.rs:123-152` - Full context of extraction logic

  **Acceptance Criteria**:
  - [ ] No `.unwrap()` calls in production code in this file
  - [ ] `cargo build` succeeds
  - [ ] `cargo test` succeeds

  **Commit**: YES
  - Message: `refactor(narrative): replace unwrap() with and_then() in openrouter_client.rs`
  - Files: `src/narrative/openrouter_client.rs`

- [x] 4. Rename `tui_state` → `generation_state` (LSP rename)

  **What to do**:
  - Use `lsp_rename` tool to rename `tui_state` to `generation_state` across the entire workspace
  - This is a multi-file rename spanning:
    - `src/model/state.rs` - field definition
    - `src/server/fragments.rs` - usage
    - Any other files using `tui_state`
  - Run `cargo build` after rename to verify compilation

  **Must NOT do**:
  - Rename any other fields or types that happen to have similar names
  - Partial renames (must rename all occurrences of `tui_state` as a field/variable)

  **References**:
  - `src/model/state.rs` line 36: `pub tui_state: TuiState`
  - `src/server/fragments.rs` - uses `state.tui_state.is_generating`
  - Research confirmed: Used for web "generating" status, NOT terminal TUI

  **Acceptance Criteria**:
  - [ ] `grep -rn "tui_state" src/` returns no matches
  - [ ] `grep -rn "generation_state" src/` returns matches
  - [ ] `cargo build` succeeds
  - [ ] All tests pass

  **Commit**: YES
  - Message: `refactor(core): rename tui_state to generation_state`
  - Files: `src/model/state.rs`, `src/server/fragments.rs`, any others

- [x] 5. Enhance `sanitize_for_prompt()` with instruction-pattern filtering (REVERTED per user request)

  **What to do**:
  - Read `src/narrative/prompt.rs` lines 15-26 (current sanitize_for_prompt function)
  - Add a second regex pattern to filter instruction override attempts:
    ```rust
    static INSTRUCTION_PATTERNS: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?i)(ignore (all )?(previous|system)|disregard|new instruction|your (new )?prompt|you (are now|have been|must)").expect("valid regex")
    });
    ```
  - Apply this pattern in `sanitize_for_prompt()` to replace matches with `[FILTERED]`
  - Keep the existing `INJECTION_PATTERN` for `{{...}}` filtering
  - Run tests to verify no regressions

  **Must NOT do**:
  - Remove existing `{{...}}` filtering - keep both patterns
  - Change function signature or return type

  **References**:
  - `src/narrative/prompt.rs:15-26` - Current sanitize_for_prompt() with only `{{...}}` filtering
  - `src/narrative/prompt.rs:434-440` - render_user_layer() calls sanitize_for_prompt()
  - Test file: `src/narrative/prompt.rs` has test functions at lines 692-978

  **Acceptance Criteria**:
  - [ ] `sanitize_for_prompt()` filters both `{{...}}` AND instruction patterns
  - [ ] `cargo test` passes (existing tests should still work)
  - [ ] Verify with manual test: sanitized output replaces "ignore previous" etc with `[FILTERED]`

  **Commit**: YES
  - Message: `security(narrative): add instruction-pattern filtering to sanitize_for_prompt`
  - Files: `src/narrative/prompt.rs`

---

## Final Verification Wave

- [x] F1. **Build Check** — Run `cargo build` to verify all changes compile
- [x] F2. **Test Check** — Run `cargo test` to verify all tests pass
- [x] F3. **SSE Audit** — Confirm no `hx-sse` attributes exist anywhere
- [x] F4. **Unwrap Audit** — Confirm no `.unwrap()` in production code in openrouter_client.rs

---

## Success Criteria

- [ ] tokio-tungstenite removed from Cargo.toml
- [ ] SSE.js script removed from index.html
- [ ] No .unwrap() in production code in openrouter_client.rs
- [ ] tui_state renamed to generation_state (across model/state.rs, server/fragments.rs, etc.)
- [ ] sanitize_for_prompt() filters instruction override patterns
- [ ] All tests pass

