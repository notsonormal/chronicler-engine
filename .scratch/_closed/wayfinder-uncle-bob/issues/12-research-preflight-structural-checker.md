# Research pre-flight structural checker skill for Chronicler Engine

Type: research
Status: closed
Assignee: assistant
Blocked by: 09

## Question

What cheap structural and convention checks should run before Chronicler Engine's expensive test suite, and how should they be packaged for AI assistants?

## Context

The Uncle Bob 2026 applicability report found that `speclj-structure-check`'s pre-flight structural-check pattern is applicable to Chronicler, but Chronicler already has the necessary checks scattered across `scripts/` and `tests/infrastructure/guardrails/`:

- `scripts/check_test_structure.py`
- `scripts/validate_docs.py`
- `scripts/validate_feature_spec.py`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- Guardrail/architecture test binaries

The gap is packaging, not capability.

## Expected output

1. A ranked list of which checks are cheap enough to run before the full test suite and catch the most common AI-generated mistakes.
2. A proposal for a single `pre-flight` command or skill (script name, CLI, output format, integration with `Violation` diagnostics).
3. A recommendation on whether this should be a human tool, an AI skill, or both, and where it fits relative to `build.py`.

## Resolution

### 1. The premise does not hold

After reviewing the actual workflow, the supposed gap is not real:

- An AI assistant editing Rust files already runs `cargo fmt` and `cargo clippy` directly when those files change.
- An AI assistant editing docs or specs already runs `scripts/validate_docs.py` or `scripts/validate_feature_spec.py` when those files change.
- An AI assistant editing JSON data already runs `scripts/validate_data.py` when data files change.
- `build.py` already runs the full gate, so anything missed by targeted checks is caught there.

Running `cargo fmt --check` as a separate pre-flight step is especially pointless: either the assistant runs `cargo fmt` to fix formatting, or formatting is already correct. A `--check` adds no value.

Running all validation scripts on every Rust edit is also wasteful. They are cheap, but they produce irrelevant output when the files they validate have not changed.

### 2. Reject a unified pre-flight command

A `build.py --check` mode that runs fmt, clippy, docs, specs, data, and freshness checks is the wrong abstraction. It treats checks as a uniform bundle instead of matching the check to the change.

The correct pattern is already in use: **run the check that corresponds to the file you changed.**

| Changed files | Relevant check |
|---|---|
| `src/**/*.rs` | `cargo fmt`, `cargo clippy --all-targets -- -D warnings` |
| `docs/**/*.md` | `python scripts/validate_docs.py` |
| `docs/specs/*.md`, `tests/http/**/*.rs`, `tests/browser/**/*.rs` | `python scripts/validate_feature_spec.py` |
| `data/**/*.json` | `python scripts/validate_data.py` |
| `src/adapters/driving/http/**/*.rs` | `python scripts/extract_http_routes.py --check` |
| `tests/infrastructure/guardrails/**/*.rs`, `tests/infrastructure/architecture.rs` | `cargo nextest run --test architecture --test guardrails` |

### 3. What, if anything, should change

No new skill or script is warranted for pre-flight checks.

The only related improvement that was raised is making `build.py` the single entry point for cargo commands so their output goes into the same `build.log` instead of being scattered across terminal sessions. That is a logging/convenience concern, not a pre-flight structural-checker concern. It should be tracked separately if desired.

### Recommendation

**Abandon the pre-flight structural-checker idea.** The existing targeted-check workflow is correct. Close this ticket with no implementation.

If future pain appears, revisit the specific pain rather than adding a generic `--check` mode.
