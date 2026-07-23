# 05: Rewrite the scanner with module-allowlist rules + rewrite tests

Status: open
Type: task
Assignee: (unassigned)
Blocked by: 01, 02, 03, 04

## Question

Implement the new scanner (renamed or replaced) that applies the module-allowlist rules decided in 02, paired (or not) with signature constraints decided in 03, running in the mode decided in 04. Rewrite the regression tests and wire (or unwire from `build.py`) per 04.

## Context

This ticket cannot start until 01/02/03/04 resolve. Its shape depends on their answers:

- **01** removes the three parked-orchestrator suppressions — the residual list is 18 honest free fns only.
- **02** decides file-level vs folder-level allowlist. Determining the allowlist entries depends on this.
- **03** decides whether to pair constraints. Determines whether the scanner has a signature-classifier component at all.
- **04** decides gate vs advisory. Determines whether the scanner ships in `build.py` or in `healthcheck.py`, and whether the output is "FAIL: smell" or "REVIEW CANDIDATE: shape".

## Scope (to be sharpened after blockers resolve)

- Replace or rename `scripts/find_free_fn_smells.py`. Candidate name: `find_free_fn_candidates.py` (advisory mode) or `check_free_fn_doctrine.py` (if gate). Name depends on 04.
- Replace `SUPPRESSED_FREE_FNS` with module-allowlist data structure (file patterns or folder patterns per 02; constraint pairs per 03).
- Fix the parser bugs surfaced by the current UNKNOWN findings (slices `&[T]`, qualified types like `crate::domain::...`) — only if the new rules still need a signature parser (per 03). If 03 rejects constraints, the parser may not be needed at all.
- Rewrite `scripts/tests/test_find_free_fn_smells.py` to cover the new allowlist logic.
- Update `build.py` (remove scanner from gate if 04 = advisory) or add to `healthcheck.py` (if 04 = advisory).
- Update the Free fn Doctrine section in `system.md` to reflect the new allowlist shape decided in 02.

## Verification

- `python scripts/<new_scanner>.py` — runs, produces report with 0 unsuppressed flags on the current codebase (the 18 honest free fns all pass via allowlist).
- `python scripts/tests/test_<new_scanner>.py` — passes.
- If gate: `python build.py` green. If advisory: `python scripts/healthcheck.py free_fn_candidates` runs.
- Introduce a deliberate smell-shaped free fn in a non-allowlisted module — verify the scanner flags it (or surfaces it as REVIEW CANDIDATE if advisory).
- Introduce a deliberate smell-shaped free fn inside an allowlisted module — verify whether the scanner catches it (depends on 03; if constraints adopted, it should flag; if pure allowlist, it will miss — that's the accepted tradeoff).

## Story points

3 SP (pre-blockers). Reassess after 02/03/04 land — if 03 = no constraints and 04 = advisory, the scanner becomes mostly a module-allowlist filter and drops to 1-2 SP.

## Notes

- This is the terminal ticket of the effort. Resolving it closes the map.
- If the implementation surfaces a rule that cannot be expressed in the chosen shape (e.g. 03 wants a constraint that the parser can't compute), STOP and report back — do not silently widen the allowlist to accommodate. That would recreate the suppression-list problem.
