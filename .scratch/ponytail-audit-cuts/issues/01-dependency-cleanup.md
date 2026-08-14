# 01: Dependency cleanup

Type: task
Status: open

## Question

Remove unused direct dependencies and replace `once_cell` with the stdlib equivalent.

## Work

1. Verify and remove from `Cargo.toml`: `image`, `pulldown-cmark`, `parking_lot`, `futures-util`, `async-stream`, `tokio-stream`, `tracing-serde`.
2. Replace `once_cell::sync::Lazy` with `std::sync::LazyLock` in `src/application/prompting/sanitize.rs` and drop `once_cell`.

## Acceptance

- `rg` shows zero direct references to each removed crate in `src/` and `tests/`
- `cargo check --all-targets --all-features` passes after removal
- `python build.py` passes after removal

## Notes

Use `cargo tree` to rule out a feature-only requirement before deleting each crate. `LazyLock` is stable in the project's Rust version (1.88).
