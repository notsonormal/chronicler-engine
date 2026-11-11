# Weekly audit 2026-11-04: dashboard-http

## Summary

The client side is healthy: one reader (`reportAnswer`) writes every request report, and the part-route removal left almost nothing behind. The server has one main failure mechanism (`ErrorAnswer` and `describe_answers`), but two older paths and three text policies remain beside it. The largest cost is in the tests. The facts of each route are in six hand-kept tables, held in step by source-parsing checks. The stub server is a partial second engine that each dashboard ticket rewrites. The large `tests/http` clone pairs are not from this week.

## Findings

### 1. High: the facts of each route are in six hand-kept tables, held together by source parsing

Evidence (known):
- `src/adapters/driving/http/builders/router.rs:28` and `:66`: the router.
- `src/adapters/driving/http/utils/answer_words.rs:153`: `pub const ANSWER_WORDS: &[RouteWords]` (46 rows, route strings again, and no method).
- `tests/http/answer_words.rs:427`: `const PINNED_WORDS` (46 rows, the same routes and sentences). `:616` `fn takes_get(route)` writes down the method of each row a second time.
- `tests/http/screen.rs:108`: `let idle_rows` (one row for each `POST` route). `:375`: `game_bound_rows`.
- `tests/test_utils/stub_request_layers.rs:15`: `pub const GAME_BOUND_ROUTES: [&str; 10]`, "as that function writes them".
- `tests/test_utils/stub_server.rs:907`: the stub router (44 `.route(` calls).
- A tier-1 test reads the router source: `tests/http/screen.rs:71` `include_str!("../../src/adapters/driving/http/builders/router.rs")` and `.split(".route(")`. The guardrails do the same (`tests/infrastructure/guardrails/answer_words.rs:16`).

Cost: a new `POST` route needs 4 or 5 hand edits, a Game-bound route 6. An assertion on source text is not a domain outcome (`tests/STRATEGY.md`).

Why review missed it: `10e4f236` added `ANSWER_WORDS` and `PINNED_WORDS`. Later stages added `game_bound_rows` and `GAME_BOUND_ROUTES`. Each diff added one table that looked local.

Fix class: **structure**. Put one route table in `src/`. Each entry holds the path, the method, the words and the Game-bound flag. `build_router` registers from it. `ANSWER_WORDS`, `GAME_BOUND_ROUTES`, `takes_get` and the two `include_str!` scans then read that table. Only the sample request of each route (URI and body) stays hand-written in tests. Move the shared rows out of `tests/http/screen.rs` into `tests/http/support/`, because `answer_words.rs` imports `crate::screen::{game_bound_rows, …}` from another test file.

### 2. Medium: the stub server reimplements parts of the engine, and each ticket rewrites it

Evidence (known):
- `tests/test_utils/stub_server.rs:1481` `screen_answer` copies the screen envelope and the `x-revision` and `x-game` headers of `screen/handlers/screen.rs:59`. `:1456` `screen_part_html` copies the part list of `SCREEN_PARTS`.
- `tests/test_utils/stub_request_layers.rs:73` reimplements `refuse_stale_game` and `stamp_request_answers` (`utils/request_layers.rs:19`, `:40`).
- The stub copies the order of the engine's refusals: `stub_server.rs:1178` "The busy Game refuses first, and a request that names no Message is invalid before both."
- Two checks exist only to keep the copy in step: `check_stub_answers_the_parts_of_the_server` (`guardrails/screen.rs:191`, which parses the stub source) and the `GAME_BOUND_ROUTES` assertion (`tests/http/screen.rs:426`).
- Cost: 12 commits changed 612 lines of `stub_server.rs` and `handles.rs` for a net growth of 24 (1955 → 1979). The `src/.../http` and `assets` code changed about 2950 lines, so the stub added about 20 % churn. A new domain field also breaks it (`1359b375`, two `PromptPreset` literals).

Why review missed it: each ticket changed the engine and the stub in one diff, so the stub edit looked like a normal test update.

Fix class: **structure**.

| Option | What it removes | Cost |
|---|---|---|
| Make the two request layers and the screen envelope generic over a small trait (stamp, active Game, part render), so the stub calls the engine functions | `stub_request_layers.rs`, the copied envelope, the parts guardrail | Small. The handles stay as they are |
| Run tier 2 against the real router with `ScriptedModel` and `Storage::with_failure` | The stub router, the copied refusal order, most handles | Large. A change to `tests/STRATEGY.md` "Faked" for tier 2. Some states, such as a held `/changes`, need a new hook |

Recommended: the first option now. Measure the churn again after two tickets.

### 3. Medium: three text policies for an engine failure, and the default hides the engine text

Evidence (known): `src/adapters/driving/http/utils/error.rs:57` `enum AnswerPolicy { EngineText, SettingsSave, GenericText }`. `error.rs:112` `AnswerPolicy::GenericText => GENERIC_FAILURE_TEXT` ("Internal Server Error"). `http/error.rs:13`: every `?` and every `CurrentGame` rejection uses `GenericText`. In `games/handlers/games.rs`, `switch_game_handler` uses `EngineText` with the Game name. `delete_game_handler` (`:174`) uses `GenericText` with the name. `create_game_handler` (`:123`) and `rename_game_handler` (`:159`) use `?`, which gives `GenericText` and no name. Scenario 45.22 says "the second line … is the text of the engine error", but it tests other routes. `error_tests.rs:37` still pins the old leak guard (`engine_non_displayable_into_response_returns_500_with_generic_body`).

Effect: a storage failure on create, rename or delete of a Game shows "Internal Server Error" under Details, so the tuner cannot see the cause (`PRODUCT.md`).

Why review missed it: `GenericText` is the older leak guard of `IntoResponse`. Stage 2 added `EngineText` beside it, and each handler chose locally.

Fix class: **one-off fix** and **rule**. Remove `GenericText`. Make `IntoResponse` use the engine text, or remove `IntoResponse` so that each handler calls `answer_for_error`. Add 45.22 rows for the three Game routes.

### 4. Medium: two older failure paths remain beside `ErrorAnswer`

Evidence (known):
- `utils/fragment.rs:11` `render_or_error_fragment` returns an `error-message` div inside a `200` (14 callers, for example `builders/presets.rs:40` and `settings/templates/settings.rs:95`). The page reader treats a `200` as "done", so a failed nested render shows as content inside a part. `tests/http/failure_display.rs:262` states the opposite rule: "a failed load must answer non-2xx, not 200 with an error body".
- `screen/handlers/screen.rs:66`, `:70` and `:82` use `bad_request(error_fragment(..))` and `internal_error(..)` from `utils/response.rs:20` and `:29`. After `d4e67887`, the only production caller of these two helpers is `screen.rs`. The stub answers an unknown part as plain text instead (`stub_server.rs:1508`: `format!("unknown part {name}")`).

So: one way for each action route (inferred). Reads and nested renders still use the old helpers.

Why review missed it: stage 2 changed the action routes, and stage 3 of the part routes deleted `failed_fragment_response`. Neither ticket owned the renders that are nested in a part.

Fix class: **one-off fix**. Make `render_or_error_fragment` return `Result` and let the part renderer fail. Use `invalid_answer` and `failed_answer` in `screen.rs`, and delete `bad_request` and `internal_error`.

### 5. Medium: the new guardrails pin source text where a type or a sweep would check behaviour

Evidence (known):
- `guardrails/answer_words.rs` has four checks. All four read `router.rs` or handler files with regex. `check_post_handlers_can_answer_an_error` lists its own holes (`:127`, "Not covered: a handler whose parameter list holds a `{` … a return type written with a path …").
- `guardrails/screen.rs:313` `resolves_to_part` follows JS `const` declarations "up to three declarations deep". `:342` says "An element that reaches a helper as a parameter is not resolved." The file has 823 lines and its tests have 860.
- Behaviour checks that do the same job exist already. `convergence.rs` and `convergence.js` (273 lines) compare each part with a fresh `GET /screen` after every stub check. `slot_sweep.rs` sweeps the rendered DOM.

Assessment (inferred): the two rows-against-routes checks go away with finding 1. The `Html<String>` check can become a type: a route helper that accepts only a `Response` or `Result<Response, ApplicationError>` output. The `hx-get`/`hx-post` path check can be a `slot_sweep`-style DOM sweep. Keep the script-write checks of `screen.rs`, but do not extend their regex dataflow: the convergence check sees the writes they miss.

Why review missed it: each ticket added a check for its own rule.

Fix class: **structure** (route helper type), **check** (DOM sweep), **rule** in `CODING_STANDARDS.md`: "prefer a type or a rendered-DOM sweep to a regex over source".

### 6. Low: `PINNED_WORDS` copies `ANSWER_WORDS` row for row

Evidence (known): `tests/http/answer_words.rs:427` contains 46 rows such as `("/prompt-presets/:id/delete", "/prompt-presets/p1/delete", "The preset was not deleted.")`, the sentence of `answer_words.rs:193`. Scenario 45.21 itself gives only three examples. Other tests in the file build the expected text from the production table (`:141` `fn words_of(route)` reads `ANSWER_WORDS`), so those tests cannot catch a wrong word.

Cost: a change to a word edits two tables. The test only finds a difference between two hand-written copies.

Why review missed it: a 1:1 table looks like coverage.

Fix class: **one-off fix**. Keep the 405 sweep (it proves that every row reaches its route), but assert the composed sentence only for the three spec examples, plus a shape check on every row (it starts with "The " and ends with ".").

### 7. Low: leftovers of the part-route removal

Evidence (known):
- The stub part renderers keep the form of the deleted route handlers: `stub_server.rs:1139` `async fn story_log_fragment(State(state): State<Arc<StubState>>) -> Html<String>`, called as `story_log_fragment(state()).await.0` (`:1461`). Seven such functions remain.
- `work/issues/pending/23-story-log-earlier-messages-investigation-plan.md:13` says "`#story-log` polls `GET /fragment/story-log` every 2 s". `d4e67887` deleted that route.
- Test names keep "fragment", for example `tests/http/failure_display.rs:243` `test_failed_panel_loads_answer_non_2xx_with_an_error_fragment`.

A search for the removed routes and `X-Refusal` found no JS caller, spec, reference doc or guardrail exemption (known). Only synthetic test fixtures name them.

Why review missed it: the stub functions still compiled after stage 3.

Fix class: **one-off fix**.

### 8. Low: the large `tests/http` clone pairs are old; one small new clone

Evidence (known): `git blame` of the clone ranges in `duplicates.txt` shows that the pairs are not from this week. `actions.rs`↔`swipe_new.rs` has 0 of 48 lines from the window, `retrigger.rs`↔`swipe_new.rs` has 2 of 63, and `actions.rs`↔`options.rs` has 0 of 48. The one clone that is mostly new is `tests/browser/stub/action_controls.rs`↔`game_header.rs` (23 of 46 lines): "fill `#command-input`, press Enter, wait". A `submit_command` helper exists, but it is private to `tests/browser/stub/failure_display.rs:20`. Ten stub files submit the command form by hand.

Why review missed it: each new stub file wrote its own copy.

Fix class: **one-off fix**. Move `submit_command` to `tests/browser/stub/support.rs`. The old tier-1 clones are a separate drain task.

## Bitter lesson

1. This area holds almost no workaround for model weakness. The failure display shows the raw engine and model text under Details (`error_disclosure`, `raw_error_detail` in `utils/error.rs`) and does not classify failures by hand. `GenericText` (finding 3) works against that.
2. The convergence check (273 lines, no knowledge of each part) runs after every stub and full-stack check, and each new test makes it stronger. The screen guardrail (823 + 860 lines) encodes JS idioms by hand, with listed holes. Put new effort into DOM sweeps and convergence.
3. The six route tables (finding 1) are human upkeep that a derivation could do. Each new route costs 4 to 6 hand edits. One table removes that work.
4. The stub handles (`set_rename_refused`, `story_writes_refused_as_busy`, `StubActionOutcome`) model engine behaviour by hand. `ScriptedModel::hold` and `Storage::with_failure` already produce such states through the real engine (inferred).

## What went well

1. One reader on the client. `assets/dashboard-errors.js:152` `reportAnswer` is the only writer of request reports. The other scripts read only `requestAnswer(event).outcome` (`dashboard-action-area.js:272`, `dashboard-story-log.js:137`, `dashboard-saves.js:60`, `dashboard-dialog.js:117`). A guardrail keeps `fetch(` to `/screen` and `/changes`.
2. The part-route removal was staged well: `ddaea736` and `ff4703ca` moved the tests to `/screen`, then `d4e67887` deleted 14 routes (396 lines) with no caller left. The old `X-Refusal` header is also gone without a trace.
3. The stub shares the words instead of copying them: `stub_server.rs:1003` installs the real `describe_answers` layer and answers through the real `refused`, `invalid_answer` and `failed_answer`. This is the model for the fix in finding 2.
