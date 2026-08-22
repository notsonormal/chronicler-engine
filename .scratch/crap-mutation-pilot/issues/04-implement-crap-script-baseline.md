# Implement the CRAP report script and establish baseline

Type: task
Status: open
Assignee:
Blocked by: 01, 02

## Question

Implement the CRAP report script per the design in ticket 02 and the data sources in ticket 01, run it against the current codebase, and record the baseline.

## Context

This is the execution ticket for the CRAP half of the pilot. It may be executed in-session (this map overrides plan-only for task tickets) once 01 and 02 are resolved.

Work to do:

1. Implement `scripts/crap_report.py` (or chosen form) following the design spec from ticket 02. Include a module docstring (enforced by `scripts/check_python_docstrings.py`).
2. If ticket 01 requires a companion Rust binary, add it under an appropriate crate/bin and wire the script to invoke it.
3. Generate the coverage JSON (non-`--summary-only`) and run the script against `src/`.
4. Record the baseline: write the report to `tmp/crap/report.md` and capture the **top-N CRAP hotspots** in the resolution answer. Decide whether to commit a snapshot of the baseline or keep it `tmp/`-only (mirrors coverage artifacts).
5. Confirm the script passes `scripts/check_python_docstrings.py` and `python -m py_compile`.

## Expected output (resolution)

- The script committed under `scripts/`.
- The baseline top-N hotspots recorded in the answer, with the commit/keep-in-tmp decision stated.
- A note on any surprises (functions the join missed, macro noise, etc.) that should feed the "soft threshold" fog item on the map.
