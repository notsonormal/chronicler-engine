//! Architecture guardrail tests using arch-lint. They fail the build on any violation defined in `arch-lint.toml`. Run them with `cargo nextest run --test architecture`.

arch_lint::check!();
