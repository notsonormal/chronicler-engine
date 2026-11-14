# A passing Python test prints an error line into every gate log

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Must `test_validate_docs.py` silence the `argparse` error it provokes on purpose?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass), with the build session `a09ca82b`.

- **known** Every gate log holds "python -m unittest: error: unrecognized arguments: --no-anchors" from a passing test (`scripts/tests/test_validate_docs.py:414-415`). An agent's search of the log for a failure landed there first.

## Options to decide

- **A.** Wrap the call in `contextlib.redirect_stderr`.
- **B.** Nothing.
