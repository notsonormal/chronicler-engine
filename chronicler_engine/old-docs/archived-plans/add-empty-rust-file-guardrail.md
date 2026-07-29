# Add empty-rust-file guardrail (archived)

**Status:** Shipped. Implementation landed as `tests/infrastructure/guardrails/structure.rs::check_empty_rust_file` + the two `#[test]` entries in `tests/infrastructure/guardrails/mod.rs` (`guardrails_empty_rust_files_src`, `guardrails_empty_rust_files_tests`).

## Final rule

A `.rs` file fails the guardrail when **both** hold:

1. **Text scan:** every line in the file is either blank or starts with `//`, `//!`, `/*`, `*`, or `*/` after trimming.
2. **Parse scan:** `syn::parse_file(content)` returns `Ok` with `ast.items.is_empty() && ast.attrs.is_empty()`.

Either condition alone is not enough: the text scan rejects `//!`-only files (treating inner doc as comments), while the parse scan rejects files whose only content is a stray `#![cfg(...)]` attribute (attrs are real cfg). A file with code lines plus a parse error is left to other guardrails to surface — this rule does not flag it.

## Rationale for the belt-and-suspenders shape

Two false-positive classes motivated the dual scan:

- `syn::parse_file` returns `Ok` with `attrs` populated for files that contain only inner doc (`//!` lines parse to inner attrs). A pure `items.is_empty()` check would silently pass a doc-only file. The text scan classifies `//!` as comment content, catching this.
- A file that opens a block comment and never closes it can fail to parse (text is consumed as `None` tokens, returning `Err`). The text scan alone would flag such a file as comment-only, which is misleading — it's a parse error. The parse scan catches this.

## Wiring

- Source files: `check_src_files("empty rust file", check_empty_rust_file)`
- Test files: `check_tests_files("empty rust file (tests)", check_empty_rust_file)`
- Both registered in `tests/infrastructure/guardrails/mod.rs`. No exemption list — `lib.rs`/`main.rs` and every other entry file has real code.

## Outcome

- `tests/infrastructure/guardrails/structure.rs::check_empty_rust_file` — walker.
- `tests/infrastructure/guardrails/mod.rs` — two test entries wired in.
- Dead file `src/adapters/driven/text_check/types.rs` deleted (caught by the rule pre-deletion).
- Rule count in `guardrails.md §3` defers to the registry; no volatile count restated.
