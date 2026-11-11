# P0: the test rule and the guardrail machinery

Type: task
Status: resolved

## Question

Build §3 of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) on branch `wf/rebuild-p0`, from the current `main`. Read §1, §2 and §3 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §3 pass. The owner has approved the last commit, and the branch has merged to `main`.

## Comments

**2026-10-17, implementer (P0 built, not committed).** Branch `wf/rebuild-p0`, from `main`
(bc7d6cd4). Every Content item of plan §3 landed; `python build.py --no-browser` is green
(`logs/build_20261017_132959.log`: guardrails 182 passed, integration 1634 passed + 2 skipped).

### What each Content item became

| Item | Where |
|---|---|
| 1. `build.py` target selection | `build.py`: `LIB_TEST_TARGET`, `--test-target <name>` on the `test-pattern` subparser, `_test_target_flags`, `_step_command(.., test_target)`. `lib` → `--lib`, any other name → `--test <name>`, no target → today's all-binaries command |
| 2. The one rule | `tests/STRATEGY.md` "The one rule": the three places, the scenario rule, and a `Check:` per place. The "a test for each branch" sentence is gone; the unit row of the tier table points at the new section |
| 3. `validate_seam_cases.py` | `scripts/validate_seam_cases.py` (+ `scripts/tests/test_validate_seam_cases.py`, 14 tests): the declaration form, the tag form, the four failure kinds plus an unattached tag. Registered as the `seam-cases` step in `build.py` next to `spec-coverage` and in `GATE_ORDER`. Scans `src/**/*_tests.rs` and `tests/storage/**/*.rs`; passes on today's empty case set |
| 4. Legacy list + allowlist | `tests/infrastructure/guardrails/unit_test_allowlist.rs`: `LEGACY_UNIT_TEST_FILES`, today's 106 `src/**/*_tests.rs` files as a table literal, and `check_unit_test_allowlist` (a unit test lives in a seam contract, a `domain/model/` file, or the list; a stale entry fails). Detection tests in `unit_test_allowlist_tests.rs` |
| 5. Four table-driven walkers | `tests/infrastructure/guardrails/walkers.rs`: `BannedName`/`BannedNameRow`, `CallSiteRow`, `TypeCheckPosition`/`TypeCheckRow`, `CallCountRow`, and `check_banned_name_rows` / `check_call_site_rows` / `check_type_check_rows` / `check_call_count_rows`. The row tables are `walker_rows.rs` (four tables, empty: no production row yet) |
| 6. Dead-path assertion | Inside each walker: a row whose scope (or `file`) matches no file under `src/` is an error that names the rule and the scope. Covers only the new rows, as the plan orders; today's two dead walkers stay until C4 |
| 7. `generate_guardrails_doc.py` | `parse_walker_rows` reads the one documented form (struct literal + comment block above it + one-line values), `format_walker_rows_table` renders the new table inside the `syn` sentinel, and a row without a reason or without a `rule:` field exits non-zero, so `--check` fails. `guardrails.md` regenerated; its §3 prose states the form |
| 8. `registry_tests.rs` rework | Added `every_walker_table_is_run_by_a_registered_test`, `every_walker_row_has_a_detection_case`, `every_walker_rule_number_is_used_once`; `TestBodies` now also collects expression-path references inside `#[test]` bodies. The per-kind detection tests are the four table-driven tests in `walker_tests.rs` (a fixture case table per kind: detection, clean, out-of-position and dead-path cases) |

### Measurements

`test-pattern "test_delete_last_between_actions_http"` after a one-line marker edit in
`src/domain/model/state/game_state.rs` (file restored and `cmp`-verified after each run):

| Run | `--test-target http` | all binaries |
|---|---|---|
| 1 | 7.12 s | 10.47 s |
| 2 | 5.51 s | 11.62 s |

So the target shape is 5.5 to 7.1 s against 10.5 to 11.6 s for all binaries (ticket 07
measured 5.0 to 5.6 s against 13.2 s). The targeted run executes 1 test and skips 270; the
all-binaries run skips 1846.

Line counts of the new guardrail code (guessed 350 in ticket 13 §2): `walkers.rs` 473,
`walker_tests.rs` 315, `walker_rows.rs` 19 — 807 with their tests, plus the legacy-list
guardrail and its tests at 253. The walkers carry four row types with their doc comments and
a four-case fixture table per kind, which is where the difference from the guess sits.

### Deviations, with reasons

- **`unit_test_standards.md` no longer shows `storage.set_game_id(1)`.** The rewrite as the
  readable surface of the one rule replaces the eight patterns; the storage example is now a
  `tests/storage/` case, so the `set_game_id` examples go in P0 rather than P1. P1's Done-when
  search (`set_game_id`, `current_game_id`) therefore has one doc fewer to change.
- **One extra doc line:** `docs/diataxis/reference/coding_standards/testing.md` described
  `unit_test_standards.md` as the "canonical eight-pattern form", which the rewrite makes
  false, so the reference line changed with it.
- **`every_walker_row_has_a_detection_case` keys on the rule number** (a production row's rule
  needs a fixture case of the same kind). With no production row yet the check is vacuous,
  which is the same state as the seam-case checker.
- **`tests_snapshot()`** was added so `check_tests_files` and the walkers read one file
  snapshot; the legacy list and the four walkers read `src/` only.
- **`--test-target` is not validated** against the test-binary list: cargo reports an
  unknown target itself, which keeps `build.py` free of a second list of binaries.

### Validation

- `python build.py --no-browser` — green (`logs/build_20261017_132959.log`), 19 gate steps.
- `python build.py guardrails` — 182 passed (`logs/build_20261017_132433.log`).
- `python build.py clippy` — green (`logs/build_20261017_132709.log`).
- `python -m unittest discover scripts/tests` — 388 passed.
- `python scripts/generate_guardrails_doc.py --check`, `scripts/validate_docs.py`,
  `scripts/check_test_structure.py`, `scripts/check_python_docstrings.py` — all pass.
- Browser tier not run: P0 changes no browser behaviour.

Nothing is unresolved; the three lead decisions of the fix pass are recorded below.

**2026-10-17, fix pass after the review (lead decisions).**

- The churn in `build.py` and `scripts/` since the first report (docstring and comment
  rewording, the deleted ticket/plan references) comes from the commit gate's comment cleanup.
  The lead accepts it.
- **The `Narrator` stays out of the box lists until P4.** The lead's rule: docs state only what
  exists. P4 adds the `Narrator` to `tests/STRATEGY.md` "The one rule" and to
  `unit_test_standards.md` "The boxes and their seam contracts", with its seam contract.
- **The call-site method-call match relies on the receiver naming convention** (inferred design,
  lead). `Type::method` matches `Type::method(..)` and a method call whose receiver ends in an
  identifier or field named the snake_case of `Type` (`active_game.get()`,
  `self.active_game.set(..)`, `state.narrator.narrate(..)`). The walker has no type information,
  so a value stored under another name is not matched; the limit is stated on `CallSiteRow`.

### Validation of the fix pass

- `python build.py --no-browser` — green (`logs/build_20261017_135100.log`), 18 steps,
  guardrails 187 passed, integration 1634 passed + 2 skipped.
- `python build.py clippy` (`logs/build_20261017_135013.log`), `python build.py guardrails`
  (`logs/build_20261017_135014.log`, 187 tests, +5 over the first pass),
  `python -m unittest discover scripts/tests` (393 tests, +5),
  `python scripts/generate_guardrails_doc.py --check` and `python build.py validate-docs`
  (`logs/build_20261017_135002.log`) — all pass.
- One repair outside the review findings: `guardrails.md` §3 prose carried the lead's new
  `§6.2` citation on the same line as its `.scratch/...` path, which `validate-docs`
  (`STANDARD_DOC_BODY_REFERENCE`) rejects before the `## Document References` section. The
  citation moved to `## Document References` and the sentence now reads "The Rule column holds
  the design rule's number." The provenance stays; the lead can reword it.

**2026-10-17, owner override of plan §3 Docs (ticket 26 R11).** The owner found that the rewritten `unit_test_standards.md` was walkthrough prose with no reader. The lead deleted it. Its unique rules moved: the shape of a case and the legacy break rule to `tests/STRATEGY.md` "The one rule", and the XSS rule to `testing.md` "XSS Regression". The links in `CODING_STANDARDS.md`, `testing.md`, `integration_test_standards.md` and two skills changed. Later slices: where the plan names `unit_test_standards.md` (P1 Docs), read `tests/STRATEGY.md`.

## Answer

P0 is built and merged to `main`. `build.py test-pattern --test-target <name>`; the one rule in `tests/STRATEGY.md`; `scripts/validate_seam_cases.py` in the guardrails family; the legacy list with its allowlist guardrail and an empty seam-contract table; the four table-driven walkers with empty row tables, the dead-path assertion and one table-driven detection test per kind; `generate_guardrails_doc.py` parses the rows in one strict form. `unit_test_standards.md` is deleted (owner). Full gate green: `logs/build_20261017_151119.log`.
