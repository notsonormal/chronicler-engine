# The full gate regenerates the two generated docs instead of failing on them

Type: task (implementation)
Status: open
Blocked by: —

## Question

Implement the decision of ticket 89: when the full gate runs `fmt`, it also regenerates `guardrails.md` and `http_routes.md` instead of failing on a stale copy.

## Decision (from 89, user, 2026-10-31)

- **Option A.** When `fmt` runs (the bare full gate and `--no-browser`), the gate runs `python scripts/generate_guardrails_doc.py` and `python scripts/extract_http_routes.py` without `--check`. With `--no-fmt`, the gate keeps the two `--check` steps. That is the same rule as `fmt`: the full gate rewrites tracked files, and `--no-fmt` does not.
- **Scope: only these two docs.** The gate does not run the other three generators (`AGENTS.md` Structure, `tests/AGENTS.md`, `docs/AGENTS.md`). The pre-commit hook owns those files. They have never failed a gate, and other efforts edit `AGENTS.md` often.
- **Rejected:** B (move the checks before `clippy`, so the run fails in about 3 s but still needs a rerun by hand), C (won't do), and D (delete the checks and let the hook own the docs).

## Why

- From 2026-10-28 to 2026-10-31, six full gates failed at "Checking guardrails.md freshness..." in six sessions: `logs/build_20261028_200011`, `_212011`, `_224445`, `build_20261029_020413`, `build_20261030_002004`, `build_20261031_004226`. Each run lost about 41 s (step 12 of 19) and one agent turn. After one failure, one lead ran the generators by hand before every gate.
- The check protects nothing at commit time. The gate installs the hook. The hook regenerates and stages `guardrails.md`, or it aborts the commit. No CI exists. A stale doc can reach a commit only through `--no-verify`, or when nothing has installed the hook yet.
- The hook does not regenerate `http_routes.md` (`GENERATORS` in `scripts/precommit_regenerate.py`). With this change, the gate becomes its regenerator.
- In its default mode, `generate_guardrails_doc.py` still exits 1 when a guardrail has no doc comment or a row has no reason. Only the staleness check goes away.

## Touch points (re-check before editing)

- `build.py`: the `http-routes-check` and `guardrails-doc-check` registry specs (about lines 377–386), `_GATE_CHECKS` (about line 1162) and `_plan_gate_steps` (the `if not args.no_fmt` branch, about line 1227). Each regenerating step must run before `validate-docs`.
- `scripts/tests/test_build_cli.py` (lines 44–45, 405–406) and `scripts/tests/test_build_cli_gate.py` (`test_no_fmt_prunes_fmt_only`, line 79). Add a test: with fmt, the plan regenerates both docs; with `--no-fmt`, the plan checks both docs.
- `AGENTS.md` Development Loop table, full-gate row: "fmt (rewrites files)" must also say the gate regenerates the two docs.
- `docs/diataxis/reference/coding_standards/guardrails.md:60` names `build.py guardrails-doc-check`. This text is outside the generated block. Fix it if the step name changes.
- `scripts/validate_docs.py:203`: the comment names "the `--check` freshness step". Update it.
- `scripts/tests/test_build_cli.py`: `EXPECTED_COMMANDS` (line ~395) pins the command of every registry spec, and `test_expected_subcommands_exist` (line ~34) lists the subcommands. A new spec needs a row in both.
- `docs/diataxis/reference/coding_standards/guardrails.md:60` also needs a fix when the step name stays: after this change, a row with no reason also fails the full gate at the regenerating step, not only at `guardrails-doc-check`.

## Plan review (2026-10-31)

Verified against the code: the line numbers above are correct. Both generators write in their default mode and exit non-zero on a parse failure (`generate_guardrails_doc.py`: exit 1 on a missing doc comment or reason; `extract_http_routes.py`: exit 2 on a missing router or zero routes). `validate_docs.py` reads `http_routes.md` (`ROUTES_DOC`, line 175) to check route strings, so the regenerating steps must run before `validate-docs`. The plan keeps that order.

The gate keeps its step count: 19 steps for the full gate, 18 for `--no-fmt` and for `--no-browser`. The regenerating steps replace the check steps at the same position in `_GATE_CHECKS`. The existing count tests stay valid.

### What already exists

- `REGISTRY` and `_spec_step` (`build.py`): one spec per step, label and command in one place.
- `_step_plan` (`build.py` ~line 1320): precedent for a step variant built in the plan (`_FORMAT_CHECK_CMD` before `clippy`).
- The default (write) mode of both generators. No new generator code is needed for the core change.

### Not in scope

- The three index generators that the pre-commit hook owns.
- Adding `extract_http_routes.py` to `GENERATORS` in `scripts/precommit_regenerate.py`.
- Moving the `--no-fmt` checks earlier in the plan (option B, rejected).

### Failure modes

| New path | Failure | Handling |
|---|---|---|
| Regenerating step, guardrails | A walker row has no reason, or a `check_*` fn has no doc | Generator exits 1. The step has `check=True`, so the gate stops. Same as today. |
| Regenerating step, routes | `router.rs` is missing or parses to zero routes | Generator exits 2. The gate stops. |
| Regenerated doc left in the tree | The agent does not see that the gate changed a tracked file | `guardrails.md`: the hook regenerates it at commit. `http_routes.md`: nothing catches it. See decision 2. |
| Concurrent edit of a generated doc | Another agent edits the doc while the gate runs | The gate overwrites the edit. Same risk as `fmt`; the generated blocks are not for hand edits. |

### Decisions (user, 2026-10-31)

1. **Two new registry specs.** Add `http-routes` (`python scripts/extract_http_routes.py`) and `guardrails-doc` (`python scripts/generate_guardrails_doc.py`) to `REGISTRY`, next to the two `-check` specs. Labels: "Regenerating http_routes.md..." and "Regenerating guardrails.md...". `_plan_gate_steps` picks the regenerating pair when fmt runs and the `-check` pair with `--no-fmt`, at the same position in the plan. Pin both commands in `EXPECTED_COMMANDS` and add both names to `test_expected_subcommands_exist`. Rejected: one spec whose command the plan rewrites.
2. **The generators write only on a change.** In default mode, each generator compares the rendered text with the file on disk. It writes and prints the "Updated"/"Wrote" line only when the text differs; otherwise it prints that the doc is unchanged. Add one test per script (`scripts/tests/test_extract_http_routes.py`, `scripts/tests/test_generate_guardrails_doc.py`): a second run reports unchanged and does not rewrite the file. The pre-commit hook also calls `generate_guardrails_doc.py`; its result stays the same.
