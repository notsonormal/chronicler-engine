//! [DOC: docs/architecture/guardrails.md]
//! Architecture guardrail tests using arch-lint.
//!
//! This test runs as part of `cargo test` and fails the build on any
//! architectural violations defined in `arch-lint.toml`.
//!
//! To run only architecture tests:
//!   cargo test --test architecture

arch_lint::check!();
