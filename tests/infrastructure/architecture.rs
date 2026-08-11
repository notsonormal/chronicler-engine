//! Architecture guardrail tests using arch-lint — fail the build on any violation defined in `arch-lint.toml`; run with `cargo nextest run --test architecture`.

arch_lint::check!();
