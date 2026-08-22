# Document CRAP and mutation workflows and update indexes

Type: task
Status: open
Assignee:
Blocked by: 02, 03, 04

## Question

Write the how-to documentation for the CRAP report and the `cargo-mutants` pilot, and update the repo indexes so the new tooling is discoverable.

## Context

This is the execution ticket for the documentation half of the pilot. It may be executed in-session once 02, 03, and 04 are resolved.

Work to do:

1. Write a `docs/diataxis/how-to/` page for the CRAP report: how to generate the coverage JSON, run `scripts/crap_report.py`, and read the output. Follow the repo's diataxis conventions (consult `docs/AGENTS.md` and the validate_docs checks).
2. Write a `docs/diataxis/how-to/` page for the `cargo-mutants` pilot per the scope in ticket 03.
3. Update `AGENTS.md` Structure/scripts index if a new script is added (the pre-commit hook regenerates docs index via `scripts/generate_docs_index.py`).
4. Update `docs/AGENTS.md` if a new how-to page is added under `docs/diataxi`/`docs/diataxis`.
5. Run `python scripts/validate_docs.py` to confirm docs pass validation.

## Expected output (resolution)

- Both how-to pages committed.
- Indexes updated; `validate_docs.py` passes.
- Links to the new docs recorded in the answer.
