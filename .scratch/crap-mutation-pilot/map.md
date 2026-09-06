# CRAP reporting and mutation-testing pilot

## Destination

A working, on-demand CRAP report script that joins per-function cyclomatic complexity with `cargo-llvm-cov` function coverage, plus a documented `cargo-mutants` on-demand pilot workflow, with a baseline run recorded. Neither tool is gated in CI.

## Notes

- Domain: Chronicler Engine Rust codebase, quality/dev-loop tooling.
- Origin: the Uncle Bob 2026 applicability study (`.scratch/wayfinder-uncle-bob/`) — see ticket [10-research-crap-mutation-testing](../wayfinder-uncle-bob/issues/10-research-crap-mutation-testing.md) and the final report's recommendation #2. That research already piloted `cargo-mutants` v27.1.0 on the quantifier parser and proposed a `scripts/crap_report.py` shape; this map turns the proposal into shipped tooling.
- Skills to consult: research, grilling, code-consistency-check, code-simplification.
- Standing preferences:
  - CRAP and mutation testing are **on-demand reports, never CI gates** (settled by the prior research ticket).
  - New scripts live in `scripts/` and follow existing Python conventions (module docstrings enforced by `scripts/check_python_docstrings.py`).
  - Existing coverage scripts (`scripts/parse_coverage.py`, `scripts/coverage_summary.py`) are file-level only; per-function coverage must come from a non-`--summary-only` `cargo llvm-cov` JSON.
  - This effort **overrides plan-only mode for AFK task tickets**: once the design decisions (01–03) are closed, the implementation/documentation task tickets (04–05) may be executed in the same session they are claimed, limited to one ticket per session.

## Decisions so far

- [01 — Research per-function CC and coverage sources](./issues/01-research-cc-coverage-sources.md) — coverage is the existing `build.py --coverage` full JSON (`data[0].functions[]`); CC is a Python heuristic for the pilot; join on `(file, bare_name)` via v0-tail parse + closure filter + monomorphization aggregation + project-`src/` filter; `rust-code-analysis` rejected (won't build on Rust 1.88); `syn` binary is the documented upgrade if the baseline shows the heuristic's match-arm undercount hides risk.

## Not yet specified

- Whether a soft CRAP threshold should surface as a non-gating warning in `build.py`. Graduates after the baseline run (ticket 04) shows whether a threshold is meaningful.
- Whether the CRAP script should emit a machine-readable JSON sidecar for later tooling. Not needed for the pilot; revisit if a downstream consumer appears.
- Whether the match-arm CC undercount warrants the `syn`-binary upgrade. Graduates after the baseline run (ticket 04) shows whether the heuristic's lower-bound CC hides real risk in match-heavy modules.

## Out of scope

- Making CRAP or mutation testing a CI gate or per-PR gate (settled by prior research).
- Porting Uncle Bob's language-specific tools (`crap4java`, `crap4go`, `crap4clj`, `mutate4java`, `mutate4go`, `clj-mutate`).
- Embedding differential mutation manifests or source-file hash footers in the codebase.

## Open tickets

- [02 — Design the CRAP report script interface and output](./issues/02-design-crap-script-interface.md) — unblocked (01 resolved)
- [03 — Scope and document the cargo-mutants pilot](./issues/03-scope-mutants-pilot-docs.md)
- [04 — Implement the CRAP report script and establish baseline](./issues/04-implement-crap-script-baseline.md) (blocked by 02)
- [05 — Document CRAP and mutation workflows and update indexes](./issues/05-document-crap-mutation-workflows.md) (blocked by 02, 03, 04)

## Closed tickets

- [01 — Research per-function CC and coverage sources](./issues/01-research-cc-coverage-sources.md) — resolved


## Assets

_None yet._
