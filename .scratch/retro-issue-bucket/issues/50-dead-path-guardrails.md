# Make the two path-limited guardrails check real files

Type: grilling (HITL)
Status: resolved
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

**2026-10-19, lead (evidence for the owner's status change).** The core step's C4 built option C with A. Both walkers now take the HTTP adapter's folder: `check_handler_return_type` uses `HTTP_ADAPTER_SCOPE`, and `check_server_layer_boundaries` checks `adapters/driving/http/` with `SwipeState` as its banned type (`tests/infrastructure/guardrails/layers.rs`). The meta rule is in place: a path-limited check whose scope matches no file reports `dead_scope_violation` (`tests/infrastructure/guardrails/mod.rs`), and a walker row reports `dead_path_violation` (`walkers.rs`, tested in `walker_tests.rs::a_dead_path_names_the_rule_and_the_scope`). The branch is `wf/rebuild-core`, not yet on `main`. Recommendation: resolve this ticket when the core step merges to `main`.

## Answer

2026-10-21. **Resolved: option C with A, built in core step C4.** Triage pass 2 (`../triage.md`), HEAD `511e28dd`; the owner approved the close.

- The fix is on `main`: `51efaafc` (C4), merged in `22a8f090`. Both walkers use the HTTP adapter's folder (`HTTP_ADAPTER_SCOPE` in `tests/infrastructure/guardrails/layers.rs`).
- The meta rule is in place: `dead_scope_violation` (`tests/infrastructure/guardrails/mod.rs`) and `dead_path_violation` (`walkers.rs`, tested in `walker_tests.rs`).
- Residual, not part of this answer: the meta rule checks scope paths, not exemption paths. `structure.rs:135` (`server/mod.rs`) and `free_fn.rs:22-28` (`application/generation/slot.rs`, `application/pipeline/spawn.rs`) name no file, and `layers.rs:280` passes whatever the `mod.rs` skip does. Candidate for a new ticket.
