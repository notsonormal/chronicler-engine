# P2: delete the arrival opening

Type: task
Status: resolved
Blocked by: 02

## Question

Build §5 of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) on branch `wf/rebuild-p2`, from the current `main`. Read §1, §2 and §5 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §5 pass. The owner has approved the last commit, and the branch has merged to `main`.

## Comments

Done on branch `wf/rebuild-p2` from `9bec8705` (main after P1). Not committed; the owner commits and
approves the last commit. Code is green: `python build.py` (log `logs/build_20261017_184434.log`)
— fmt, data, clippy, test-structure, spec-coverage, seam-case, Python docstring, Python tests,
http_routes freshness, guardrails.md freshness, doc validation, guardrails 187/187, integration
1625 passed / 0 failed / 2 skipped, browser 29/29. Integration is -3 tests against `main`: the
tree deleted the three `arrival_service_tests.rs` cases. One comment reword in `run.rs` came after
that run; `python build.py clippy` (`logs/build_20261017_184756.log`) and `cargo fmt --check` are
green on the final tree.

### What the slice did

| Plan item | Became |
|---|---|
| Delete `application/arrival_service.rs` | File deleted (201 lines). `src/application/mod.rs` drops `pub mod arrival_service;` and `mod arrival_service_tests;`. |
| Delete `application/arrival_service_tests.rs` | File deleted (164 lines, 3 tests). Its `application/arrival_service_tests.rs` entry goes from the legacy list in `tests/infrastructure/guardrails/unit_test_allowlist.rs` (the guardrail fails a stale entry). |
| Delete `ArrivalSpawnRequest` and `spawn_arrival_task_if_needed` | Both deleted from `src/bootstrap/init_game.rs` (67 lines). Its module summary becomes "Game state initialization and game resolution" (the generator reads line 2 of the file). |
| Delete their call in `bootstrap/run.rs` | The call and the three values it read from the loaded state (`nearby_npcs`, `all_npcs`, `room_id`) go. |
| Delete `PromptContext::build_narration_prompt` | Deleted from `src/application/prompting/assembler.rs` (15 lines). Checked first: the only caller was `arrival_service.rs:161`, and no test calls it (`rg -n build_narration_prompt src/ tests/` → the callee and that one call site). |
| Docs | The generated structure index in `AGENTS.md` drops the `arrival_service.rs` row and takes the new `init_game.rs` summary; `guardrails.md` renumbers one row (line shift in the allowlist). Both regenerated with the four `scripts/generate_*` generators, as the pre-commit hook does. `startup.md` and `narration_system.md` (the two docs ticket 13 §4 names): neither mentions the arrival opening or the spawn, so neither changed. |

**Done-when search.** `rg -n "arrival_service|ArrivalService|spawn_arrival" src/ tests/ docs/diataxis/`
→ no matches. `rg -ni "arrival" src/ tests/ docs/diataxis/ CONTEXT.md docs/specs/` → one unrelated
hit, `docs/diataxis/explanation/rust_idioms.md:36` ("request-arrival").

### Deviations the plan did not state

1. **The boot-time `load_game_state` call stays, without binding its result.** After the spawn call
goes, `prepare_state` is the only caller of `load_game_state`, and its return value had no reader.
The plan's Content row deletes only the spawn; the core step's Deletions row deletes
`load_game_state` and `try_persist_initial_message` later. Deleting the call now would also delete
the boot-time seed of the initial Snapshot and the scenario Message — a second player-visible
change the plan's P2 row does not list — and keeping the function with no caller fails clippy
(`dead_code`). So the call stays for its side effect, with a comment saying the loaded state is not
kept. Flag for the core step: `prepare_state` will drop it there.
2. **`PreparedData.db_pool` is removed.** It was read only by the deleted spawn call, so clippy
(`-D warnings`, `field is never read`) failed. `prepare_data` still builds the `DbPool` and still
passes it to `init_game::resolve_game_id`.

### Checks with no finding

- No `docs/specs/` scenario and no tier-1 or browser test relies on an LLM opening for a World with
  no scenario text, as §5 says. `rg -ni arrival docs/specs/ tests/browser/` → no matches; both
  seeded worlds (`data/worlds/test`, `data/worlds/redmist_estate`) carry non-empty scenario text, so
  the spawn returned early in every real-server run. `tests/http/games_create.rs` and
  `tests/helpers/storage_ext.rs` build their own world with scenario text.
- The arrival defect that `.scratch/agent-workflow-friction/issues/12` records as out of scope (no
  gate slot at boot, so an Action can run beside it) goes with the code. That scratch issue is a
  resolved historical record, so it is left untouched.
- No test file, guardrail rule or doc line outside those listed names the deleted code.

**2026-10-17, lead (code review).** Two items lost their only production caller and stay on purpose: `PromptAssembler::with_max_tokens` (P4 removes `with_storage` and passes the token budget as a value, so P4 reuses or replaces it) and `ActionPipeline::recorder()` (a test seam of `bootstrap/wiring_tests.rs`; the core step deletes `ActionPipeline`). Old plans under `docs/plans/` that name `ArrivalTaskContext` are history and stay as they are. Retro: no new finding; the spec review and the standards review found no defect.

## Answer

P2 is built and merged to `main`. The arrival opening is gone: `arrival_service.rs` and its tests, `ArrivalSpawnRequest`, `spawn_arrival_task_if_needed` and `PromptContext::build_narration_prompt`. A World with no scenario text starts with no LLM opening. The boot-time `load_game_state` call stays until the core step. Full gate green: `logs/build_20261017_185010.log`.
