# Make the trial script

Type: task (AFK)
Status: open
Blocked by: —

## Question

Turn the prototype trial script into the kept script `scripts/trial.py`. It runs a test setup file through Autoplay on a probe server. Also make the failure text of an empty answer name its `finish_reason`, so a trial and a player can see why a call failed. [Shape the test setup and the trial](08-test-setup-and-trial.md) decided both.

The other engine tickets check their work with this script, so it comes first. The two parts are independent: the script reads `finish_reason` from the stored raw answer, not from the failure text.

Done when `python scripts/trial.py run scripts/trial_setups/redmist_julian.json --cycles 5` plays a short trial to its end, the prototype is gone, and `python build.py` passes.

## Decisions

From [Shape the test setup and the trial](08-test-setup-and-trial.md):

- The script applies the setup through the HTTP routes. No engine command, and no DB write.
- A filtered story call (`content_filter`) ends an Autoplay run. The script starts Autoplay again on the same Game, at most 5 times for each trial, and the pack marks the gap.
- A trial has a token limit, default 6,000,000. `repeats` defaults to 1.
- The reading pack stays as the prototype makes it.
- The trial connection is OpenCode Go.

Choices made when the work was ticketed ([Write the build plan for the engine changes](09-build-plan.md)), corrected by the plan review of 2026-10-31:

- **The setup file describes the test. Flags describe the machine.**
  - The file holds only fields that a route sets today: `world`, `persona`, `mode`, `perspective`, `tense`, `presets` (`system`, `impersonate`, `quantifier`, `options`), `connections` (`narration`, `quantifier`), `cycles` (the Budget), `token_limit`, `restart_limit`, `repeats` and `out`.
  - Flags: `--base` (default the probe server, `http://127.0.0.1:3001`), `--db` (default the probe DB) and `--cycles` (overrides the file, so a short check can use the same file).
  - `response_length` is not in the file. It has no route, and [Add the connection slots and remove the global response length](21-connection-slots-and-response-length.md) moves the length words into the presets. That ticket also adds the Impersonate and Options slots to the file, and removes the `response_length` row from the pack.
  - Options always on is not in the file. A Game copies it from its World.
- **`connections` names a Connection id that the probe DB already holds.** The script does not create Connections, because a Connection holds an API key and the setup file is in git.
- **Setup files live in `scripts/trial_setups/`.** `data/` holds schema-checked engine data, and `tmp/` is not in git. One example, `redmist_julian.json`, holds the setup of trials A and B (100 Cycles) on the "Go check" Connection.
- **The order of the setup:** the two Connection routes, `POST /games`, then mode, then posture, then presets. Mode must come before presets, because a mode change rewrites the preset bundle of the Game. The pack holds the values that the script applied.
- **The new Game id.** `POST /games` answers an empty body. The script reads `max(id)` of `games` before and after, as the prototype does, and refuses when it did not rise.
- **The Game header.** Every Game-bound request sends `X-Chronicler-Game: <id>`. Then a client that switches the active Game during a trial gets a `409`, and the trial's writes do not go to another Game.
- **A refused route stops the run.** Any setup route that does not answer `200` stops the script, which prints the method, the path, the status and the body.
- **The Cycles played** are the number of Input entries of the Game, less 1 when the newest entry is an Input. A Cycle ends only on an answered Input (`docs/specs/autoplay.md`). The prototype's plain Input count is one too high after a failed call.
- **Restart.** When a run ends before its Budget, the script reads the newest failed call of the Game.
  - Its `finish_reason` (from the copied `raw_response_json`) is `content_filter`, the token limit is not spent, and fewer than `restart_limit` restarts have run: wait until `GET /debug/is_busy` answers `false`, then start Autoplay again with the Cycles that remain. The pack marks the gap with the `finish_reason`.
  - Any other end stops the trial. The pack says why. A provider outage or an engine fault must show, not hide behind a restart.
- **The token limit is for each trial.** Each repeat is one trial and has its own limit and its own restarts (ticket 08). The script prints the tokens of each trial and the total at the end.
- **The failure line.** `POST /status/reset-busy` clears the failure line of the Game. The script reads the failure line before any stop. It sends `reset-busy` only to stop a run at the token limit, never after a run that ended by itself.
- **Repeats:** each repeat is a new Game. The packs go to `trial-1` … `trial-N` under `out`. Then `compare` runs over them.
- **The failure text:** an empty answer names its reason in both failure strings, for example "LLM returned an empty response (finish_reason: content_filter)". The short status line ("The language model's answer could not be read.") does not change. With no reason known, the text stays as it is today.
- **The Python tests** use a real `sqlite3.connect(":memory:")` with the tables that the script reads, so a later schema change fails a test. Only the HTTP function is a fake.

## Facts

Known on 2026-10-31.

The script and the probe:

- `.scratch/prompt-flexibility/prototype/PROTOTYPE_trial.py` (409 lines) has `run`, `export` and `compare`. It copies new `llm_messages` rows every 2 s (the record keeps 50 rows). It stops at a token limit (default 5,000,000) with `POST /status/reset-busy`. It does not apply a setup, restart a run or repeat a trial. It also calls `POST /games/:id/switch`, which is not needed, because `POST /games` already makes the new Game active. [The trial prototype](../assets/trial-prototype.md) describes it and the packs of trials A and B.
- The pack's setup table prints `settings.response_length` and the `agents` setting.
- The routes, with their form fields (the form structs are in the handler modules, not in `router.rs`):

  | Route | Fields | Answer |
  |---|---|---|
  | `POST /games` | `world_key`, `persona_key` | `200`, empty body |
  | `POST /games/:id/mode` | `narrator_mode`: `novel` or `interactive_fiction` | `200` |
  | `POST /games/:id/posture` | `narrative_perspective`: `second` or `third`; `narrative_tense`: `present` or `past` | `200` |
  | `POST /games/:id/presets` | `system_preset_id`, `quantifier_preset_id`, `impersonate_preset_id`, `options_preset_id` (all four) | `200`; `400` for a missing preset or a preset that the mode forbids |
  | `POST /connections/set-narrator`, `POST /connections/set-quantifier` | `connection_id` | `200`; `400` "Connection not found" |
  | `POST /autoplay` | `cycles` (1 to 100, default 10) | `200`; `400` bad Budget; `409` busy |
  | `POST /status/reset-busy` | — | `200`, body `reset` |
  | `GET /debug/is_busy` | — | `true` or `false` |

- A request with no `X-Chronicler-Game` header acts on the active Game. A request whose header names another Game than the active one is refused with `409` (`utils/request_layers.rs`).
- `POST /status/reset-busy` removes the holder and writes the dock record with no failure (`GameWriter::stop`, `application/generation/gate.rs`).
- A refused route answers an HTML fragment (`utils/error.rs`): `400` validation, `409` busy, `500` engine.
- The opening of a new Game comes from the World, not from a model call (`domain/model/utils/opening.rs`).
- The probe server: `/chronicler-ui-investigator`, target dir `target/ui-investigator`, port 3001, DB `target/ui-investigator/debug/chronicler_3001.db` (at version 31). In that DB, "Go check" is `conn-1793449507061-3a1cb15f` (OpenCode Go, `deepseek-v4.1-flash`, `reasoning_effort: none`).
- `sqlite3.connect(path)` creates an empty file when the path is missing.
- `tmp/db-backup-20261031/` is deleted by the full gate after 30 days (`build.py`, `clean_tmp_dirs`). Do not depend on it.
- Python scripts start with a module docstring and have no shebang (`scripts/check_python_docstrings.py`). Their tests are `unittest` files in `scripts/tests/test_<module>.py`, run by the `py-tests` step. `build.py` runs no Python linter. `generate_structure_index.py` lists each top-level `scripts/*.py`.

The failure text:

- "LLM returned an empty response" is the `Display` text of `LlmFailure::EmptyResponse { raw_response }` (`src/error.rs`, a `thiserror` enum with no `Serialize`).
- `parse_chat_response` (`transport/utils/response.rs`) reads `finish_reason` only for `"length"`. Every other reason with empty content becomes `EmptyResponse`, at its one production construct site.
- Two strings reach the player: the `Display` text (the status disclosure, through `GenerationFailure::from_engine_error`), and `EngineError::llm_error_string` ("LLM Error: empty response": `llm_messages.error_message`, the LLM Messages panel and the banner disclosure).
- `LlmFailure::token_counts_suffix` already adds a suffix to a variant's text from its `#[error(...)]` attribute.
- `src/error.rs` is not a folder that allows a top-level free function (`guardrails/free_fn.rs`).
- Sites of `EmptyResponse`: `src/error.rs` (declaration, `raw_response_body`, `llm_error_string`), `response.rs` (construct), `error_tests.rs` (3), `llm_recorder_tests.rs` (2), `response_tests.rs` (2), `generation_status_tests.rs` (1). Patterns with `{ .. }` need no edit.
- Only `error_tests.rs` and `llm_recorder_tests.rs` pin the text. `response_tests.rs` pins the variant and the raw body. No spec scenario, integration test or doc pins the text.
- `ChatCompletionsMock` (`src/test_support/chat_completions_mock.rs`) serves one request with one fixed body (`"content":"stub answer"`, `"finish_reason":"stop"`). `start()` has 22 call sites.

## What already exists

| Need | Reuse |
|---|---|
| The HTTP helper, the row copy, the usage parse, `export`, `compare`, the pack | The prototype script |
| A suffix on a failure text | `LlmFailure::token_counts_suffix` and its use in an `#[error(...)]` attribute |
| A mock with another answer | `serve_one_request` in `ChatCompletionsMock`. `start()` delegates to the new `answering(body)` |
| The failure text that the pack prints | `llm_messages.error_message` |
| The end of a run | `GET /debug/is_busy` |

## Steps

1. **The failure text, test first.**
   - Add `ChatCompletionsMock::answering(body)`, which takes a whole JSON body. Make `start()` call it with the current body, so its 22 call sites stay.
   - Add the case `chat-completions-empty-answer-names-its-finish-reason` to `chat_completions_tests.rs`, with its `// CASE` line. The mock answers empty content with `finish_reason: content_filter`. Assert the variant, and that `llm_error_string()` holds `finish_reason: content_filter`.
   - Give `LlmFailure::EmptyResponse` a field `finish_reason: Option<String>`. `parse_chat_response` reads `finish_reason` once, before the `"length"` test, and fills the field.
   - Add the suffix as a method of `impl LlmFailure`, next to `token_counts_suffix`, and use it in the `#[error(...)]` attribute and in the `llm_error_string` arm. No free function in `src/error.rs`.
   - Update the sites listed in the Facts. The two exact strings in the tests stay, because they have no reason.
2. **Move the script.** Move the prototype to `scripts/trial.py`. Give it a module docstring and no shebang. Keep `run`, `export`, `compare` and the reading pack. Drop the `/games/:id/switch` call.
3. **The setup file and the flags.** `run <setup.json>` reads the fields and the flags above, with the defaults above. It refuses a `--db` path that does not exist, and a Connection id that the DB does not hold. Add `scripts/trial_setups/redmist_julian.json`.
4. **Apply the setup** in the order above, with the Game header, and stop on a refused route. Write the applied values into the pack.
5. **The run loop.** Count the Cycles played, restart, stop at the token limit and repeat, as decided above. Read the failure line before any stop.
6. **Tests.** Add `scripts/tests/test_trial.py`. The command functions take the HTTP function and the DB connection as arguments (for example `cmd_run(args, http=http, connect=sqlite3.connect)`). Cover:
   - the request list of a full setup: method, path, form and header, in order;
   - the setup defaults, the `--cycles` override, and a refused setup (a missing Connection id, a missing DB path);
   - the Cycles played, with and without an unanswered Input at the end;
   - the restart decision: `content_filter` with restarts left, `content_filter` at the restart limit, another reason, the token limit;
   - the usage parse of a canned `raw_response_json`;
   - the pack line of a failed call.
7. **Clean up.**
   - Delete `.scratch/prompt-flexibility/prototype/`.
   - Point the links in `assets/trial-prototype.md` and in [Shape the test setup and the trial](08-test-setup-and-trial.md) at `scripts/trial.py`.
   - Regenerate the structure index (`scripts/generate_structure_index.py`).
8. **Check.** Start the probe server from this branch (`/chronicler-ui-investigator`) on its own DB. Run `python scripts/trial.py run scripts/trial_setups/redmist_julian.json --cycles 5`. The pack has the setup table with the applied values, and the story plays 5 Cycles.
9. Run `python build.py`.

## Failure modes

| Code path | Failure | Result | Handled by |
|---|---|---|---|
| Setup | A route answers `400`, `409` or `500` (a preset that the mode forbids, a posture value other than `second`/`third`) | The script stops and prints the request and the answer | "A refused route stops the run" |
| Setup | Another client creates a Game at the same moment | `max(id)` can name the other Game. The Game header then makes the next request fail with `409` | The Game header |
| Run | Another client switches the active Game during the trial | The next request fails with `409`, and the trial stops | The Game header |
| Run | A story call is filtered again at the same point | The trial stops after `restart_limit` restarts, and the pack says so | `restart_limit` |
| Run | A call fails for another reason (HTTP error, outage) | The trial stops, and the pack names the reason | The restart rule |
| Run | The token limit is passed between two polls | The limit overshoots by up to one poll. `reset-busy` in the middle of a Job drops that Job's late writes | Accepted. The pack marks the last Cycle as cut |
| Restart | Autoplay is started while the old holder still exists | `409 StillThinking` | Wait for `is_busy` to answer `false` |
| Pack | `reset-busy` erases the failure line | The pack would say "no failure" | Read the failure line before any stop |
| DB | `--db` names a missing file | `sqlite3` would create an empty file | The script refuses the path |
| Failure text | A provider sends no `finish_reason` | The text stays as today | The suffix is added only when a reason is known |

## Not in scope

- No engine command, no DB write, no new route, and no new `llm_messages` column. The 50-row record does not change.
- No criteria and no judge for comparing trials.
- Options always on as a setup field.
- The Impersonate and Options slots in the setup file ([Add the connection slots and remove the global response length](21-connection-slots-and-response-length.md) adds them).
- A short status line for a filtered answer.

## Output

Resolved when the check passes and `python build.py` passes. The answer records the setup file fields, the path of the check pack, and any route that did not act as the Facts say.
