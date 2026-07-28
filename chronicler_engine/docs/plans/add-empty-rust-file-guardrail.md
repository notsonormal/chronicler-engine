# Add empty-rust-file guardrail

## Summary
New guardrail fails any `.rs` file that parses to zero items and zero attrs — i.e. only comments/whitespace. Uses `syn::parse_file` already in use in the same file. Mirrors `check_file_length` shape. Wired into src and tests walks.

## Key Changes
- New rule `check_empty_rust_file` in `chronicler_engine/tests/infrastructure/guardrails/structure.rs`.
- Two `#[test]` entries in `mod.rs`: `guardrails_empty_rust_files_src`, `guardrails_empty_rust_files_tests`.
- Delete dead `chronicler_engine/src/adapters/driven/text_check/types.rs`; drop any stale `mod types;` / `pub use types::*;` in `text_check/mod.rs`.
- Bump count in `guardrails.md` §3 from "21 rules" to "22 rules".

## Implementation

### Phase 1: Guardrail + cleanup

- [ ] #### Task 1.1: Implement `check_empty_rust_file` (2 SP)
  - In `structure.rs`, add:
    ```rust
    pub fn check_empty_rust_file(path: &str, content: &str) -> Vec<Violation> {
        let mut violations = Vec::new();
        let Ok(ast) = syn::parse_file(content) else { return violations; };
        if ast.items.is_empty() && ast.attrs.is_empty() {
            violations.push(Violation::error(
                path, 1,
                "File is empty: only comments and blank lines. Delete it or add real code.".to_string(),
            ));
        }
        violations
    }
    ```
  - Reuses `Violation`, `syn`; no new deps. Handles `//!`-only files (attrs non-empty → no false pass — wait, types.rs IS `//!`-only and must fail).
  - NOTE: `//!` inner doc on a file parses into `ast.attrs`. types.rs has only `//!` lines → `attrs` non-empty → would WRONGLY pass. Fix: strip leading `//!`/inner attrs OR check raw non-comment content. Use: after parse, also require that content has at least one non-comment, non-blank line. Simpler: count `ast.items.is_empty()` AND no outer `mod`/`use`/`fn`/`struct` etc. — equivalent to `items.is_empty()`. Accept that file with only `#![...]` inner attrs flags as non-empty (attrs are real cfg). For `//!` docs: treat as comments → use text-level check for `//!` lines.
  - REVISED impl: text-level scan. A line is "code" if, after trimming, it is non-empty and not starting with `//`, `//!`, `/*`, `*`, `*/`. If zero code lines AND `syn::parse_file` returns Ok with empty items → fail. Belt-and-suspenders avoids both syn false-pass (attrs) and parser false-negative.
  - ##### SubTask 1.1.1: Verify against types.rs (1 SP) — rule must flag types.rs before deletion. Run `cargo nextest run --test guardrails guardrails_empty_rust_files_src` post-impl, pre-deletion; expect FAIL naming types.rs.

- [ ] #### Task 1.2: Wire tests in `mod.rs` (1 SP)
  - `#[test] fn guardrails_empty_rust_files_src() { check_src_files("empty rust file", check_empty_rust_file); }`
  - `#[test] fn guardrails_empty_rust_files_tests() { check_tests_files("empty rust file (tests)", check_empty_rust_file); }`
  - Re-export already glob (`pub use structure::*;`) — verify no manual list to update.

- [ ] #### Task 1.3: Delete dead file (1 SP)
  - Remove `chronicler_engine/src/adapters/driven/text_check/types.rs`.
  - Grep `text_check/mod.rs` for `mod types` / `use types` / `types::` — remove stale refs.
  - `cargo check -p chronicler_engine` green.

- [ ] #### Task 1.4: Update guardrails.md (1 SP)
  - §3 "21 rules" → "22 rules". Verify exact phrase via `grep -n "21 rules" guardrails.md` first.

## Test Plan
- `cargo nextest run --test guardrails` green after all tasks.
- Pre-deletion (Task 1.1 done, 1.3 not): new src test FAILS naming `types.rs` — proves rule catches the target case.
- Post-deletion: same test green.
- `cargo check` after types.rs deletion — compiles.
- `python build.py` — green (optional, primary agent verifies at end).

## Per Task/Sub Task Validation Steps
- Task 1.1: Temp run `cargo nextest run --test guardrails guardrails_empty_rust_files_src` with types.rs present → fails on types.rs. Then proceed.
- Task 1.2: `cargo nextest list --test guardrails | grep empty_rust` shows both tests.
- Task 1.3: `cargo check -p chronicler_engine` green; `grep -rn "types" chronicler_engine/src/adapters/driven/text_check/` returns nothing.
- Task 1.4: `grep -n "21 rules" guardrails.md` empty; `grep -n "22 rules" guardrails.md` hits.

## Assumptions
- "Empty" = zero lines that are neither blank, `//`, `//!`, `/* */`, nor continuation `*`. `//!` docs count as comments (matches user example).
- File with only `#![cfg(...)]`/inner attrs = non-empty (attrs are code). Acceptable; rare in src/.
- Rule applies to `src/` and `tests/`. No exemption list — `lib.rs`/`main.rs` have real code.
- Deleting types.rs in-scope. Comment body: "DELETE THIS EMPTY FILE".
- syn parse error → rule returns no violation (parse errors surfaced by other guardrails).
