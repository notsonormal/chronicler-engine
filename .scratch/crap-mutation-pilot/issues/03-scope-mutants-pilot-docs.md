# Scope and document the cargo-mutants pilot

Type: grilling
Status: open
Assignee:
Blocked by:

## Question

What is the scope of the documented `cargo-mutants` pilot — which file(s) it targets, whether the invocation is wrapped in a script or just documented, and where the how-to documentation lives?

## Context

The prior research (`.scratch/wayfinder-uncle-bob/issues/10-research-crap-mutation-testing.md`) already ran `cargo-mutants` v27.1.0 against `src/application/agents/quantifier/utils/parser.rs` and found it too slow for a gate but useful as a targeted audit. This ticket fixes the *documented* pilot, not a new evaluation run.

Decisions to make:

- **Pilot target(s):** keep the quantifier parser as the canonical example, or pick a second high-risk file (e.g., pipeline core) to show the workflow on a larger module. One canonical example is enough for a pilot.
- **Wrapper vs. command:** document the bare `cargo mutants` invocation in a how-to, or add a thin `scripts/mutation_pilot.py` wrapper that fixes the target file and flags (`--test-tool nextest --baseline skip`). Prefer documenting the command unless a wrapper adds real value.
- **Documentation location:** a `docs/diataxis/how-to/` page (e.g., `how-to/run-mutation-testing.md`), and whether the dev-loop section of `AGENTS.md` should reference it.
- **On-demand framing:** make explicit in the doc that this is not a gate and not part of `build.py`.

## Expected output

A short scope decision: chosen pilot target(s), wrapper-or-document verdict, and the documentation path — enough for ticket 05 to write the doc.
