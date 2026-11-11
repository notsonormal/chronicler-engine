# Pass 3 — Behaviours vs. Tests

Read-only audit. Class of each behaviour, with proof. `rg` was used; no repo-root `grep`.

Class meanings:

| Class | Meaning |
|---|---|
| PINNED-RIGHT | A test asserts the behaviour the sentence calls correct, so a fix keeps it green. |
| PINNED-WRONG | A test asserts the faulty behaviour as the expected result, so a fix must change the test. |
| TOUCHED | The code path runs, but the asserts would still pass if the behaviour changed. |
| NO TEST | No test asserts the behaviour. The words searched are listed. |

## Table

| ID | Behaviour (short) | Class | Proof |
|---|---|---|---|
| B1 | Leaving + destination must not move the player | NO TEST | Only `Entering` is expressible above the parser: `QuantifierResult` holds no movement type (`src/domain/model/quantifier.rs:8-12`), and the shared test helper always writes `"Entering"` (`src/test_support/scripted_model.rs:470-473`). |
| B2 | Non-JSON answer keeps Room + Characters, tells the player nothing | TOUCHED | `tests/http/actions.rs:519` (helper used by `:506` too), asserts `:550-554` (status `Ready`) and `:560-563` (Characters kept). No assert of the Room; no assert of the dock or the failure banner. |
| B3 | Trigger fires with its Character absent | PINNED-RIGHT | `src/domain/model/utils/game_rules_tests.rs:411`, assert `:424-427` (`expect("a global Trigger fires for a Character not in the scene")`, then `character_id == "gabriella"`). Integration-level: `tests/http/actions.rs:310` (shopkeeper absent from the scene, its Trigger still fires). |
| B4 | Two qualifying Triggers: which one fires | NO TEST | No test holds two qualifying Triggers. `fired_trigger` takes the first of a `HashMap` walk (`src/domain/model/utils/game_rules.rs:87-90`, `evaluate_triggers` at `:166`). Nearest test: two Triggers on one Character, one pre-marked fired (`game_rules_tests.rs:491`). |
| B5 | Delete the event Narration, then a new Action: one-shot Trigger fires again | NO TEST | Searched `delete`, `refire`, `fires_again`, `one_shot`, `history/delete`, `retrigger`. The delete-then-generate tests are `/swipe/new` (`tests/http/swipe_new.rs:662`) and Retrigger (`tests/http/retrigger.rs`), never a following `/action`. |
| B6 | Retry of the opening Narration | NO TEST | Only the plan is pinned: `src/domain/model/generation_tests.rs:115` (row "the opening" at `:118`, assert `:164-169`) and `:192`. No test asserts that the analysis runs, a Trigger fires, or the Room changes. Searched `opening`, `swipe/new`, `retry`, `analysis`, `is_analysed`. |
| B7 | Options failure: status display text and failure banner together | NO TEST | The Options tests fetch only `options-dock` (`tests/http/options.rs:224`, `:501`). The one Options status-display fetch (`:1004`, assert `:1022-1029`) is a *settings* read failure, not an Options call failure. No test fetches `failure-banner` after an Options call; the Options failure effect `"showing no options"` (`src/adapters/driving/http/builders/headers.rs:31`) is asserted nowhere. |
| B8 | Role health after a failed call + more than 50 other calls (retention) | NO TEST | Pruning is pinned (`tests/storage/llm_message_storage.rs:70`, assert `:89-91`, 55 rows, one role). Banner clearing is pinned only by a later success of the same role (`tests/http/failure_display.rs:34`, assert `:95-97`), never by retention dropping the failed row. |
| B9 | `/mode` or `/rename` while a Job runs | NO TEST | The busy-row table `tests/http/answer_words.rs:184-225` omits both routes. `/mode` is pinned refused under the Autoplay holder (`tests/http/answer_words.rs:236`, assert `:290-294`). `/rename` has no test while anything is busy. |
| B10 | `/switch` while a Job runs: what the running Job does | PINNED-RIGHT | `tests/http/races.rs:419`, asserts `:455-457` (the switch back accepts and Game A is active) and `:459-477` (each story holds only its own Narration). |
| B11a | `POST /worlds` with zero Rooms and `scenarios_json` `[]` | PINNED-RIGHT | `tests/http/requires_migration/worlds_fragment_handlers.rs:104`: map `regions: []`, `scenarios_json` `[]`, assert `:127-143` (`is_success`, empty body, the World in the list). |
| B11b | `/screen` with the active Game's Room missing | PINNED-WRONG | `tests/http/failure_display.rs:217`, missing Room set at `:222`, assert `:230-234`: `StatusCode::INTERNAL_SERVER_ERROR`, "a failed load must answer non-2xx". |
| B12 | `POST /settings/text-check` with an unknown `check_mode` | NO TEST | Searched `check_mode`, `text-check`, `no_such_mode`, `unknown`, `invalid`, `bogus`. Values tested: `spell` (`tests/http/settings.rs:222`) and `disabled` (`:248`). `src/adapters/driving/http/settings/handlers/settings.rs:186-191` maps any other value to `Disabled`. |
| B13 | Typed Input holding `</PlayerInput>` or `{{user}}` | NO TEST | Searched `PlayerInput`, `{{user}}`, `FILTERED`, `escape`, `sanitize_for_prompt`. Cousin tests use `{{system}}` and `{{char}}` only (`src/application/narration/narrator_tests.rs:1562`, `:1582`). `</PlayerInput>` is escaped nowhere (`src/application/prompting/assembler.rs:472`). |
| B14 | Narration answer with `finish_reason: "length"` | TOUCHED | Transport: `src/adapters/driven/llm/transport/utils/response_tests.rs:74`. Kind mapping: `src/domain/model/state/generation_status_tests.rs:84`. Screen line: `tests/http/failure_display.rs:332` asserts the player's line, but *injects* `TokenBudgetSpent` (`:335-339`), so it would pass even if a `length` answer stopped producing that kind. No test drives a length answer through a Job. |
| B15 | Process restart in the middle of a Job | NO TEST | Searched `restart`, `recover`, `boot`, `startup`, `Input with no`, `last Message is an Input`, `stale`. `src/bootstrap/run_tests.rs:101` restarts only to check that the opening is not duplicated. |
| B16 | `X-Chronicler-Game` sent by the real page | PINNED-RIGHT | `tests/browser/stub/game_header.rs:29` (registered `:558`), assert `:53-57` on `game_headers("/action/check")`; the stub reads that header by name (`tests/test_utils/stub_request_layers.rs:85-88`), so a different name in `assets/dashboard-screen.js:6` fails it. The stale refusal through the page: `:83`, assert `:100-102`. |

## Count per class

| Class | Count |
|---|---|
| PINNED-RIGHT | 4 (B3, B10, B11a, B16) |
| PINNED-WRONG | 1 (B11b) |
| TOUCHED | 2 (B2, B14) |
| NO TEST | 10 (B1, B4, B5, B6, B7, B8, B9, B12, B13, B15) |

## Notes that change a fix's shape

- **B1 (corrects the brief's suspicion).** The Game-rules tests cannot pin the move, because `QuantifierResult` carries no movement type. But `MovementType::Leaving` *is* pinned as keeping its destination at the reading layer: `src/application/agents/quantifier/utils/parser_tests.rs:142` (assert `:147-148`) and `src/application/agents/quantifier/utils/orchestration_tests.rs:477` (assert `:497`). A fix at `QuantifierReading::into_result` (`src/application/agents/quantifier/types.rs:56-62`) leaves both green. A fix inside the parser breaks `parser_tests.rs:142`.
- **B11a.** The accepting test sits in `tests/http/requires_migration/` ("Quarantined HTTP tests pending specs"), which is still compiled and run by the `http` binary through `tests/http/mod.rs:39`.
- **B9 vs B10.** `/switch` is not a Game-bound write that a Job refuses; it moves the active pointer. So it has no busy refusal to assert, and B10's question is answered by what the running Job writes.
