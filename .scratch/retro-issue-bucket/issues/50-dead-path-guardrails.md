# Make the two path-limited guardrails check real files

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Two guardrail walkers check no file, because the path that they are limited to no longer exists. Their unit tests pass, because the tests use fake paths. What do we do with them, and how do we stop a path-limited walker from going dead again?

- **Option A: re-point both walkers.** Point them at `adapters/driving/http/`, the folder that took the place of `server/`, and change their unit tests to real-shaped paths.
- **Option B: delete both walkers.** Do this if the rules no longer earn their place (for example, if another rule or the compiler already covers them).
- **Option C: A or B, plus a meta rule.** A walker that is limited to a path fails when that path matches no file under `src/`.

Recommendation: C with A. Today, neither rule would find a violation in `adapters/driving/http/` (known: 0 non-test lines there name `GameState` without `GameStateSnapshot`, and 0 name `(StatusCode, String)`). So a re-point costs only the path and the tests.

## Context

- **Source.** Not a `/retro` or `/reflect` run. The finding came up in the grilling of turn-state-model ticket 25, [Sketch the new design next to today's code](../../turn-state-model/issues/25-sketch-new-design-next-to-code.md) (2026-10-14, Q9). The owner asked for an issue outside that map. The fix is a guardrail, so this bucket's Notes put it in scope.
- **Evidence** (re-checked on 2026-10-14):
  - `tests/infrastructure/guardrails/mod.rs:114` `check_src_files` passes each path relative to `src/` (`relative_path`).
  - `tests/infrastructure/guardrails/layers.rs:94` `check_handler_return_type` returns early unless the path starts with `src/server/`. A path relative to `src/` never starts with `src/`, so this walker would check nothing even if `src/server/` existed.
  - `tests/infrastructure/guardrails/layers.rs:203` `check_server_layer_boundaries` returns early unless the path starts with `server/`. `src/server/` does not exist.
  - The unit tests at `layers.rs:267–305` and `:508–520` use paths such as `src/server/test_handler.rs` and `server/renderers.rs`, so they pass.
  - `check_handler_return_type` also has a `TODO` (`layers.rs:91`) that questions its list of exceptions.
- **Why it matters.** The turn-state-model sketch (`assets/sketch-25-new-design.md` §6) makes guardrails part of the design. A walker that silently checks nothing gives false confidence.
- **Skills:** `/grilling`.
