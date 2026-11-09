# `test-pattern` cannot select the tests of a spec scenario

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

An agent wants to run the tests of scenario 41.26. The `SCENARIO` tags are comments, so the test names do not hold the number. Must `build.py` select tests by scenario?

## Context

Source: `/retro` 2026-11-08, the coordinator session of statechart-models tickets 08, 10, 11, 12, 13 and 16 (`01a1bd22`), and its subagent sessions (t12 `549a889a`).

- **known** t12 ran `python build.py test-pattern "41.2" --test-target http` and got "error: no tests to run".
- **known** `scripts/validate_feature_spec.py` already maps each scenario to its tagged tests.

## Options to decide

- **A.** A `--scenario 41.26` option of `build.py`, which finds the tests through the mapping of `validate_feature_spec.py`.
- **B.** One line in the `test-pattern` help text: test names hold no scenario numbers.
- **C.** Nothing.
