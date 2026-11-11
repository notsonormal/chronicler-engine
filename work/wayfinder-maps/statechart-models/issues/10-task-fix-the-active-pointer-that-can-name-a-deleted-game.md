# Fix the active pointer that can name a deleted Game

Type: task
Status: resolved
Blocked by: —

## Question

Close the orders (three known, one inferred) that leave the active pointer (`ActiveGame`) on a Game that no longer exists. Decide the design, change the code, bring the TLA+ model along, and add a test where a seam can force an order.

The harm is low (owner, ticket 09, Q4). But ticket 14 waits for the model fix of this ticket, because the diagrams must show a model that follows the code.

## The orders (1 to 3 known: TLC on a scratch variant of the model)

1. A delete of Game 2 passes its check, a switch moves the pointer to Game 2, and the delete removes Game 2. Run 1 found this order (`DeleteNotActive`).
2. A switch reads Game 2, a delete of Game 2 passes its check and removes it, and the switch sets the pointer to Game 2.
3. A reset creates Game 3 in its hold. A delete of Game 3 passes its check, because the pointer still names the old Game, and removes it. Then `ActiveGame::set_new` moves the pointer to Game 3.
4. The same order through `GameCatalogue::create_game` (`GameWriter::create`, then `set_new`, `catalogue.rs:59-62`). Found by the plan review of 2026-11-08 (inferred). The TLA+ model has no create action.

Detail: the Comments of [ticket 09](09-grilling-decide-the-gaps-and-the-verify-skill.md). The scratch variant (a switch in two steps, the rule `ActiveExists`) and its configs: [assets/09-switch-in-two-steps/](../assets/09-switch-in-two-steps/).

## Context

- **Two locks (known).** `ActiveGame` has its own mutex. `BusyRegistry::hold` locks only the `busy` mutex. `GameCatalogue::switch_game` calls `storage.get_game` and then `ActiveGame::set`, outside every hold. `GameCatalogue::delete_game` reads `ActiveGame::get` outside the hold of `GameWriter::delete`. `GameCatalogue::reset` calls `GameWriter::replace` and then `ActiveGame::set_new`.
- **A silent set (known).** `ActiveGame::set` calls `Storage::touch_game_updated_at`, an `UPDATE` that answers `Ok` when no row matches.
- **The harm (inferred).** Each later request names a Game that does not exist, until the person switches Games or creates a Game. A restart repairs the pointer (`resolve_game_id` picks the newest Game of the World). Two requests on two tokio worker threads (`Runtime::new()` in `run.rs:122`), or a thread preemption, reach each order. The window has no `await`.
- **The fix mutant of run 1 closes order 1 only.** A check inside the hold of the delete is not enough.
- If the fix needs a refactor beyond `ActiveGame`, `GameCatalogue` and `GameWriter`, stop and report it (`AGENTS.md`).

## The decisions (plan review, 2026-11-08)

- **The design: the `ActiveGame` mutex is the outer lock** (owner, decision 3, Option C). Each of these runs while that mutex is held: the switch (the Game read, the touch, the pointer set), the delete (the pointer check, then `GameWriter::delete`), the reset (`GameWriter::replace`, then the pointer set) and the create (`GameWriter::create`, then the pointer set). The change stays in `active.rs` and `catalogue.rs`. `GameWriter` keeps its API. The rule of `storage.md:23` stays true: `GameCatalogue` is the only reader and the only writer of `ActiveGame`. The lock order becomes `ActiveGame`, then the guard (`busy`), then the backend, then the connection. No code takes `busy` and then `ActiveGame` today (known: `gate.rs`, `job.rs` and `driver.rs` do not name `ActiveGame`). The cost (inferred: milliseconds): `CurrentGame`, the changes answer (`request_layers.rs:45`) and `/screen` (`screen.rs:73`) wait while a reset holds the mutex.
- Rejected: the pointer check and set inside a `BusyRegistry` hold (it crosses `generation/` and breaks the `storage.md` rule); one mutex for the holders and the pointer (a refactor); a storage touch that fails when no row matches (it closes order 2 only).
- **The invariant sentence goes into `storage.md` now** (owner, decision 4, Option A): "The active Game always names a Game that exists."

## The work

- The code fix, with the lock order in the doc comments of `ActiveGame` and `BusyRegistry::hold` (`gate.rs:187-188`).
- The invariant sentence in `storage.md`.
- The TLA+ model: the switch in two steps, a create action (order 4; plan review, owner, decision 14, Option A), the rule `ActiveExists` (games.md 18.2 and 19.2), and the fix. The create action takes the outer lock as the reset does, and a config that holds it shows that order 4 is closed. `delete-race` then passes. Update `coverage.toml`: add `src/application/games/active.rs` with its functions, and every new or changed function of `catalogue.rs`. `sync_commit` and `sync_date` move only when the owner commits the fix (`SKILL.md`, step 6). If ticket 08 has landed, record the hashes of the listed functions with its record command. Remove `delete-race = ["DeleteNotActive"]` from `known_breaks`. Replace the fix mutant `delete-check-inside-the-hold` with mutants that each remove one new check, so `ActiveExists` and `DeleteNotActive` each fail once. Add `ActiveExists` to `games`, `games-run` and `delete-race`, and fix the comment in `delete-race.cfg`. In `SKILL.md`: the description (add `active.rs`), the Action table (the switch in two steps), the Rule table, the Scope table, the line example `delete-race: breaks`, and *Known findings*.
- A tier-1 test only if a seam can force an order. Today `ActiveGame` and `GameCatalogue` have no pause seam. Do not add a seam only for this test without a reason in the Answer.
- One row in [the trial log](../assets/trial-log.md).

## Checks

- `python3 .agents/skills/verify-gate/check.py` and `python3 .agents/skills/verify-gate/check.py --mutants` pass.
- `python build.py` passes.
- Do not commit.

## Not in scope

- A `reset` whose `CurrentGame` id is no longer the active Game. The pointer then moves to the new Game, and an earlier switch is lost. No pointer names a deleted Game in that case.
- The read-once gap of `CurrentGame` in other handlers.
- A storage guard on `touch_game_updated_at`.
- A test that hits the race many times on a multi-thread runtime. Such a test cannot force the order, and it gives random results. The platform-choice rulings (ticket 12) chose no scheduler tool.
- New spec scenarios. With the fix, every order gives an answer that games.md 18.2 or 19.2 already promises (ticket 09, Q5).

## What already exists

- The scratch TLA+ model variant (a switch in two steps, `ActiveExists`) and its configs: `assets/09-switch-in-two-steps/switch-in-two-steps.diff`, `switch-race.cfg`, `noreset.cfg`.
- The pattern for poisoned locks: `unwrap_or_else(|e| e.into_inner())` (`active.rs:28`).
- The tests that pin the answers: `tests/http/games_switch.rs` (18.1, 18.2, 18.5) and `tests/http/games_delete.rs` (19.1 to 19.4). They must stay green.
- The comment on the lock order in `BusyRegistry::hold` (`gate.rs:187-188`). Add `ActiveGame` to the front of the order it names.
- The fix-mutant format and `known_breaks` handling: `mutants.toml:214-230`, `check.py:140-185`.

## Failure modes

| New code path | When it fails | Handled? |
|---|---|---|
| Switch under the lock: `get_game` fails | `500`. The pointer stays, and the lock is released when the guard drops | yes, same as today |
| Switch under the lock: the touch fails | `500`. The pointer stays (18.5) | yes |
| Delete under the lock: `GameWriter::delete` fails | Error answer. The pointer stays, and the lock is released | yes |
| Reset or create under the lock: `replace` or `create` fails | The pointer stays on the old Game | yes, same as the comments in `catalogue.rs` today |
| A panic while the lock is held | The mutex is poisoned. The next lock recovers it through `into_inner`. The pointer keeps its old value | yes (inferred) |
| A later change takes the two locks in the other order | Deadlock | Write the lock order in the doc comments of `ActiveGame` and `BusyRegistry::hold` |
| A long reset blocks pointer reads on a tokio worker | Polls are slow for a moment | accepted (inferred). It is the same as the busy hold today |

## Answer

The mutex of `ActiveGame` is now the outer lock of the switch, the delete, the create and the reset (decision 3, Option C). The TLA+ model follows the code, every config holds, and `known_breaks` is empty. Not committed.

### The code (known: read and built)

- `src/application/games/active.rs`: `ActiveGame::lock` (`pub(super)`) gives a `PointerHold`, which holds the `MutexGuard`. `PointerHold` has `get`, `set` (the touch, then the move) and `set_new`. `ActiveGame::set` and `ActiveGame::set_new` are gone, and `ActiveGame::get` stays for `CurrentGame`. The doc comment names the invariant and the lock order.
- `src/application/games/catalogue.rs`: `switch_game` takes the lock before `get_game` and keeps it through `PointerHold::set`. `delete_game` takes the lock before the check and keeps it through `GameWriter::delete`. `create_game` and `reset` take the lock just before `GameWriter::create` and `GameWriter::replace`, and keep it through `set_new`. `GameCatalogue::new` uses `lock().set(..)`. `GameWriter` keeps its API.
- `src/application/generation/gate.rs`: the doc comment of `BusyRegistry::hold` puts the `ActiveGame` mutex first in the lock order.
- Two small changes outside the files that the ticket names, both needed by the API change (known):
  - `tests/infrastructure/guardrails/walker_rows.rs`, rule 2: the callee `ActiveGame::set` became `ActiveGame::lock`. Without it, rule 2 no longer guards the write side. `guardrails.md` was regenerated by `build.py`.
  - `src/adapters/driven/storage/games.rs:138`: a doc comment named `ActiveGame::set`; it now names `PointerHold::set`.
- `docs/diataxis/reference/storage.md`: the invariant sentence "The active Game always names a Game that exists.", the four requests that hold the mutex, and the lock order.

### The TLA+ model (known: TLC)

- `VerifyGate.tla`: the variable `pointerBy` (who holds the outer lock), the helpers `TakePointer` and `DropPointer`. The switch is two actions (`YouSwitchTheGame`, `TheSwitchSetsThePointer`). A new create (`YouCreateAGame`, `TheCreateMovesThePointer`). The delete and the reset take the lock in their first action and drop it in their second. The new rule `ActiveExists` (storage.md; games.md 18.2 and 19.2). The two-step form stays although the lock makes each pair atomic, so that a mutant can remove the lock of one request.
- Configs: `ActiveExists` added to `games`, `games-run` and `delete-race`. `delete-race` now also allows `"create"`, with `NewGames = {3, 4}`, and its comment says what it shows.
- `mutants.toml`: `delete-check-inside-the-hold` removed. Four new mutants, each removes the lock from one request: `switch-without-the-pointer-lock` (breaks `DeleteNotActive` or `ActiveExists`), `delete-without-the-pointer-lock` (`DeleteNotActive`, with `ActiveExists` out of the config), `reset-without-the-pointer-lock` (order 3) and `create-without-the-pointer-lock` (order 4). The reset and the create mutants take `DeleteNotActive` out of the config, because their shortest path breaks both rules in one state, and TLC names `DeleteNotActive` first. The two stop mutants (`stop-keeps-failure-line`, `stop-keeps-options-failure`) were written again for the new variable in their `UNCHANGED` text.
- `VerifyGate.view.mjs`: labels for the three new actions, and the words for a switch in flight.
- `coverage.toml`: `GameCatalogue::new` and `GameCatalogue::create_game` added, and a new `[[files]]` entry for `active.rs` (`ActiveGame::get`, `ActiveGame::lock`, `PointerHold::get`, `PointerHold::set`, `PointerHold::set_new`). They are plain strings with no hash: the coordinator records the hashes. `delete-race = ["DeleteNotActive"]` removed from `known_breaks`. `sync_commit` not moved.
- `SKILL.md`: the description (the active pointer, `active.rs`), the line example (a generic `<config>`), the Action table, a paragraph on the outer lock, the Rule table (`DeleteNotActive`, `ActiveExists`), the Scope row of `delete-race`, and *Known findings* (none now).
- Trial log: one row for 2026-11-08 and a short note.

### Tier-1 test

None added (known: no seam). `ActiveGame` and `GameCatalogue` have no pause point, and the window has no `await`. A pause seam only for this test would add test code to the lock path. The TLA+ model and its four mutants carry the orders. The existing tests `tests/http/games_switch.rs` and `tests/http/games_delete.rs` pass.

### Checks

| Check | Result |
|---|---|
| `check.py` | every config holds, exit 0 (known) |
| `check.py --mutants` | 21 of 21 caught, exit 0 (known) |
| `build.py clippy` | OK (known) |
| `build.py integration` | 1653 passed, 0 failed, 3 skipped (known) |
| `build.py browser` | 44 passed (known) |
| `build.py guardrails` | `guardrails_long_comment_runs` failed once on my doc comment, fixed. `arch_lint_check` fails only on `work/wayfinder-maps/weekly-retro-2026-11-04/assets/local-only/tmp-evidence/` (git-ignored files of 2026-11-04, not mine). It has 0 violations outside that folder (known) |
| `build.py` (full gate) | stops at Vale: `AGENTS.md:336` has a sentence of more than 25 words. That line comes from commit `25db6ee5` (owner, 2026-11-08), not from this ticket (known). My own Vale errors in `storage.md` are fixed |

### Left open

- The two failures above (Vale on `AGENTS.md:336`, arch-lint on the local-only `.scratch` folder) are outside this ticket. I did not fix them.
- The hashes in `coverage.toml` for the new and changed functions (coordinator). The `build.py` warning shows until then.
- The cost of the lock (inferred: milliseconds): `CurrentGame`, `/changes` and `/screen` read through `ActiveGame::get`, which waits while a reset or a create holds the mutex.

### Coordinator check (2026-11-08)

- `check.py`: every config holds, exit 0. `check.py --mutants`: exit 0; the four pointer-lock mutants each break once (known: a run by the coordinator).
- The hashes of the listed functions are recorded (`scripts/verify_coverage.py record`, ticket 08).
- The full gate passes (`logs/build_20261108_224117.log`), after the Vale fix of `AGENTS.md:336` and the `**/.scratch/**` exclude in `arch-lint.toml` (see ticket 08).
