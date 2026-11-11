# Unenforced invariants — theme: http (routes, inline templates, browser JS contract)

Scope: `src/adapters/driving/http/**` (non-test), `assets/index.html`, `assets/dashboard-*.js`.
Read-only investigation. Terms follow `CONTEXT.md` (Game, Message, Swipe, Job, busy state, Autoplay run).

---

## http-01 — The Game a request is *checked* against and the Game it *writes* come from two separate reads of the active-Game pointer.

**Invariant.** One request acts on exactly one Game: the Game its `X-Chronicler-Game` header names, or the active Game when the header is absent. The Game the layer checked and the Game the handler writes are the same value.

**Relied on at.**
- `src/adapters/driving/http/utils/request_layers.rs:45` — `refuse_stale_game` reads `state.game_catalogue.active_game_id()` and compares it with the header.
- `src/adapters/driving/http/utils/current_game.rs:34` — `CurrentGame::from_request_parts` reads `active_game_id()` **again**, after the middleware's await, and that value is what every handler passes to `GameWriter`.
- Nothing later compares the handler's `GameId` with the pointer: `Job::start(&state.game_writer, …, game_id, request)` (`chat_window.rs:57`) opens the Game by id only.
- The module doc claims the opposite: "`CurrentGame` reads through the catalogue once per request" (`src/application/games/active.rs:11`), and the layer doc says the check covers "every handler" (`request_layers.rs:35-38`).

**Write paths that could violate it.** Every route in `game_bound_routes` (`src/adapters/driving/http/builders/router.rs:28-62`): `/action`, `/action/check`, `/autoplay`, `/swipe/new`, `/retrigger`, `/reset`, `/message/:id/swipe/:index`, `/history/:id`, `/history/delete`, `/status/reset-busy`. A switch (`/games/:id/switch`) or a create (`/games`) from another tab moves the pointer between the two reads.

**Enforcement.** PARTIAL. The layer checks the header (`request_layers.rs:44-52`), then the extractor re-reads the pointer. Two `Mutex` acquisitions (`active.rs:37-39`), with an await point between them.

**Consequence.** A Job starts on the Game that just became active, not the Game the page named. The refusal that protects the player ("The active Game changed") is bypassed for that request: the wrong Game is written, that Game's `updated_at` and change counters move, and the page that sent the request shows Game A while Game B was played.

**Likelihood.** No player action alone. Requires a switch or create in another tab inside a window of a few microseconds.

**Epistemic status.** VERIFIED (both reads and the writer call read).

**Suggested enforcement point.** Read the pointer once per request into a request extension (in `refuse_stale_game`) and have `CurrentGame::from_request_parts` take `GameId` from that extension instead of re-reading.

---

## http-02 — Every route that acts on the active Game sits behind `refuse_stale_game`; the list is hand-written and nothing notices a missing route.

**Invariant.** A route whose handler acts on the active Game (or reads its busy state) is in `game_bound_routes`, so a request from a page that shows another Game is refused.

**Relied on at.** `src/adapters/driving/http/utils/request_layers.rs:35-52` — the whole stale-Game protection exists only for the routes that carry the `route_layer` at `router.rs:62`.

**Write paths that could violate it.** `src/adapters/driving/http/builders/router.rs:28-62` is the only list. A new Game-bound route added to `build_router` (`router.rs:66-201`) instead of `game_bound_routes` skips the layer with no error.

**Enforcement.** TEST-ONLY, and partial. `tests/http/screen.rs:85-99` extracts the routes from the *source text* of `game_bound_routes`, and `tests/http/screen.rs:403-430` asserts that `game_bound_rows()` plus `GAME_BOUND_READS` equals that extraction. Both sides derive from the same hand-written list, so a Game-bound route that is *not* in the function is invisible to the test.

**Consequence.** A stale tab's request acts on the new active Game (same outcome as http-01). The refusal that is supposed to catch the stale page never runs for the new route.

**Likelihood.** Code change only. The list is complete today (I checked every POST route and its handler's extractors).

**Epistemic status.** VERIFIED (list, layer, test source all read).

**Suggested enforcement point.** A guardrail row over `router.rs`: every `post(...)` handler whose signature contains `CurrentGame` must be registered inside `game_bound_routes`.

---

## http-03 — The slash-command names are written twice: once in the browser palette, once in `Action::parse`.

**Invariant.** Every command the palette offers (`/impersonate`, `/guide`, `/options`) is a command the engine still parses.

**Relied on at.**
- `assets/dashboard-slash-menu.js:3-7` — `SLASH_COMMANDS` offers the three commands and writes the chosen one into `#command-input` (`dashboard-slash-menu.js:98-99`).
- `src/domain/model/action.rs:35-46` — the same three literals decide the `Action`. Any other value falls through to `Self::FreeAction(input)`, which the pipeline sends to the narrator as player text.

**Write paths that could violate it.** The JS constant (`dashboard-slash-menu.js:3`) and the `match` arms (`action.rs:35-46`); no shared constant, no generated table, no `data-` attribute carries the list from the server.

**Enforcement.** NONE. The slash-menu test (`tests/browser/stub/slash_menu.rs`) drives the shipped shell against a stub and pins the client only; no test compares the palette with `Action::parse`.

**Consequence.** A renamed or added command silently becomes free text: the player picks "Guided Generation", the narrator receives it as an in-story action, and the story changes instead of the generation being steered. Nothing reports it.

**Likelihood.** Code change only (a rename on either side). No player or LLM action triggers it.

**Epistemic status.** VERIFIED.

**Suggested enforcement point.** One list of commands in the domain (`Action::COMMANDS`) rendered into the shell as `data-cmd` items, so the palette reads the server's list instead of holding its own.

---

## http-04 — HTML form field names are duplicated between browser-built markup and the server's form structs.

**Invariant.** Every field the browser sends is a field name the handler's `Form<…>` struct declares, with the same spelling.

**Relied on at.**
- Story-log edit: `src/adapters/driving/http/history/handlers/history.rs:16-17` requires `text: String`; the field is created only by the browser at `assets/dashboard-story-log.js:73` (`textarea.name = "text"`), and the Save control includes it by element id (`src/adapters/driving/http/templates.rs:25`, `hx-include="#edit-textarea"`).
- Action input: `src/adapters/driving/http/action/handlers/actions.rs:16-18` requires `command: String`; the name lives in the shell (`assets/index.html:131`) and in two builder inputs (`templates.rs:91`, `templates.rs:99`).

**Write paths that could violate it.** The JS-built textarea (the only writer of `text`), the shell input, and the preview fragment (`templates.rs:91,99`).

**Enforcement.** NONE. The Rust side is covered by the HTTP tests that post `text=…` by hand; nothing links the name in `history.rs:17` to the name set at `dashboard-story-log.js:73`. The server cannot see the mismatch: an absent field is an axum extractor `422`, not a compile error.

**Consequence.** A rename on either side turns every story edit into a `422` whose body is the extractor's raw text; the JS keeps the draft and the lock (`dashboard-story-log.js:114-117`) and shows a message that names no cause. For the Action input the same rename sends an empty `command`, which becomes an empty Action.

**Likelihood.** Code change only. Both names match today.

**Epistemic status.** VERIFIED.

**Suggested enforcement point.** Render the field name into the entry (`templates.rs`) and have the JS read it from a `data-` attribute, or a guardrail row asserting the literal in the JS equals the struct field name.

---

## http-05 — The `?v=` on every asset URL is the only cache bust, and nothing checks that a change bumps it.

**Invariant.** A changed asset under `assets/` is served under a URL that no browser already holds.

**Relied on at.** `assets/index.html:7-11` and `assets/index.html:224-231` — each asset URL ends in `?v=<n>` (e.g. `styles.css?v=25`, `dashboard-story-log.js?v=15`). The server sets no cache header: `ServeDir` at `src/adapters/driving/http/builders/router.rs:203` is used with its defaults.

**Write paths that could violate it.** Any edit to a file under `assets/` without editing the matching line of `assets/index.html`. `build.py` copies `data` and `assets` for deployment (`build.py:1306-1310`) and never reads the version. No test, no script, no guardrail row mentions `?v=`.

**Enforcement.** NONE.

**Consequence.** A returning browser keeps the old script against the new server markup. Every JS lookup is guarded (`if (!element) return`), so the failure is silent: the screen loop keeps asking, one part never updates, and no report appears. The `data-` contract is exactly where a stale script and a new server disagree.

**Likelihood.** Any deploy where a version is forgotten. Not triggerable by a player, an LLM, or a World author.

**Epistemic status.** VERIFIED for the missing check (`grep '?v='` finds no reader in `scripts/`, `tests/` or `build.py`). INFERRED for the browser's actual reuse of the old response: I did not observe a cache header from the running server.

**Suggested enforcement point.** Serve the asset URL with a content hash, or add a `build.py` step that fails when the `?v=` of a file does not change with its content hash.

---

## http-06 — `index.html` is compiled into the binary while the scripts and styles are read from `assets/` at run time.

**Invariant.** The shell the server sends and the JS/CSS it serves are one version.

**Relied on at.** `src/adapters/driving/http/chat_window/handlers/chat_window.rs:19-20` — `Html(include_str!("…/assets/index.html"))`; `src/adapters/driving/http/builders/router.rs:203` — `ServeDir::new("assets")`. Every element id the JS needs is in the compiled shell, and every `data-part` name the JS sends is in `SCREEN_PARTS`.

**Write paths that could violate it.** A deployment that copies the binary and `assets/` separately (`build.py:1306-1310`), or a checkout where `assets/` is newer than the binary. Nothing compares the two.

**Enforcement.** NONE. The screen guardrail (`tests/infrastructure/guardrails/screen.rs:145-187`) compares `SCREEN_PARTS` with the *source* `assets/index.html`, and the browser tests serve both from one checkout, so the drift of a deployed pair is untested.

**Consequence.** The browser's requests name a part the binary does not render (a `400` on every load, the screen freezes) or the shell holds an id the served JS no longer reads (a control that does nothing). Both are silent: `showScreenFailure` reports into the notice, then the next load fails the same way.

**Likelihood.** A deployment mistake, not a player action.

**Epistemic status.** VERIFIED for the two sources. INFERRED for the deployment risk: I read `_copy_deployment_assets` but did not trace every deploy path.

**Suggested enforcement point.** Serve the shell from `assets/index.html` too, or embed the whole `assets/` directory with `include_dir!` so one version cannot split.

---

## http-07 — An empty Autoplay cycle budget silently becomes 10 cycles instead of being refused.

**Invariant.** The budget of an Autoplay run is the number the player gave: a whole number from 1 to 100.

**Relied on at.** `src/adapters/driving/http/autoplay/handlers/autoplay.rs:15-18` — `#[serde(default)] pub cycles: String`, so a missing field is `""`; `src/application/autoplay/budget.rs:17-20` — an empty string returns `Ok(Self::DEFAULT)` (10).

**Write paths that could violate it.** The `cycles` input of `#autoplay-form` (`assets/index.html:143-158`) has `min`/`max`/`step` but no `required`; a player who clears the field sends `cycles=`. A hand-made request may omit the field entirely.

**Enforcement.** NONE for the empty value; the range `1..=100` at `budget.rs:26-32` is enforced for a value that parses. The three browser/Autoplay tests post an explicit number.

**Consequence.** An Autoplay run of 10 Jobs where the player gave no number. The run holds the Game for ten Cycles, and the status display reports `1/10`, which the player did not ask for.

**Likelihood.** A player clears the number box and clicks Autoplay. Reachable today.

**Epistemic status.** VERIFIED.

**Suggested enforcement point.** Refuse an empty `cycles` in `Budget::parse` (the field is required in the form) or make the form field `required` and keep the default only for an absent field at the call site.

---

## http-08 — A World key is used as a URL path segment and as an element id, and nothing restricts its characters.

**Invariant.** A World key is safe in a URL path and in an HTML `id`: no `/`, `?`, `#`, `%`, space or quote.

**Relied on at.**
- `src/adapters/driving/http/builders/router.rs:114-134` — `/worlds/:key`, `/worlds/:key/posture`, `/worlds/:key/edit`, `/worlds/:key/delete` are single-segment routes.
- `src/adapters/driving/http/worlds/templates/worlds.rs:31-32` — the Edit opener is `hx-get="/worlds/{{ world.key }}/edit"` with `id="world-edit-open-{{ world.key }}"`; `worlds.rs:71,76,81` build the auto-save URL `hx-post="/worlds/{{ key }}/posture"`; the form action is `/worlds/{key}` (`worlds.rs:213`).
- The dialog returns focus by that id: `assets/dashboard-dialog.js:53` (`document.getElementById(dialogOpenerId)`).

**Write paths that could violate it.** `src/adapters/driving/http/worlds/handlers/worlds.rs:30` (`pub key: String`, no check) → `into_world_card` (`worlds.rs:75`) → `Storage::create_world` (`src/adapters/driven/storage/worlds.rs:155`), whose only refusal is the `worlds.key UNIQUE` constraint (`storage/worlds.rs:189`). Also a hand-edited World JSON seeded by `scripts/validate_data.py`, which does not check key characters for URL safety.

**Enforcement.** NONE (DB `UNIQUE` only).

**Consequence.** A World keyed `a/b` or `a?b` or `a#b` is stored, and every control that names it builds a URL that is a different route or a truncated one: the Edit dialog opens on nothing (`404` → notice), Delete answers an unrelated `400`, and the posture auto-save writes nowhere. The World can never be edited or deleted from the dashboard again.

**Likelihood.** A World author typing a key. `my world` still works (the browser percent-encodes the space); `/`, `?`, `#`, `%` break it. No client-side and no server-side rule today.

**Epistemic status.** VERIFIED for the URLs and the missing check. INFERRED for the full list of affected controls (I did not open every World URL in a browser).

**Suggested enforcement point.** A `WorldKey` newtype with a validating constructor (`[A-Za-z0-9_-]+`) used by `WorldForm`, the storage insert and the data loader.

---

## http-09 — A narrative posture value from a form is validated on three routes and silently defaulted on two.

**Invariant.** A stored `narrator_mode` / `narrative_perspective` / `narrative_tense` is a value someone chose, never a fallback.

**Relied on at.**
- Strict: `src/adapters/driving/http/games/handlers/games.rs:203-216` — `NarratorMode::from_str(...)` / `NarrativePerspective::from_str(...)` return `Err`, answered as a `400`.
- Silent: `src/adapters/driving/http/worlds/handlers/worlds.rs:84,89,94` (create) and `worlds.rs:200-207` (update) use `parse_or_default`, which stores the enum's default for any unrecognized string.

**Write paths that could violate it.** `POST /worlds` (`router.rs:113`), `POST /worlds/:key` (`router.rs:114`) — the World form's three selects. The HTML select offers only valid values, so only a hand-made request reaches the default; a renamed/mistyped option value in a template would too.

**Enforcement.** PARTIAL: `/games/:id/mode` and `/games/:id/posture` refuse a bad value; both World write paths accept it.

**Consequence.** A World silently stores Novel / second person / past tense, and every later Game of that World inherits it. No report is shown, and the form's next render shows the default as if it had been chosen.

**Likelihood.** A hand-made request, or a template edit that changes an `option` value. Not reachable through the rendered form today.

**Epistemic status.** VERIFIED (both write paths read).

**Suggested enforcement point.** `parse_or_default` → `from_str` plus `answer_for_error` in `WorldForm::into_world_card`, so both paths refuse like `/games/:id/mode` does.

---

## Checked and enforced

| Invariant | Where it is enforced |
|---|---|
| Every part name the shell sends has a renderer, and every renderer has one shell element, each once | Guardrail `tests/infrastructure/guardrails/screen.rs:145-187` (build gate) + `tests/test_utils/screen_part_table.rs` |
| A part element has no `hx-get`/`hx-trigger`/`hx-swap`, so only the screen load writes a part | Guardrail `screen.rs:46-90` |
| Every `POST` route has an `ANSWER_WORDS` row, and every row is a router route | Guardrail `tests/infrastructure/guardrails/answer_words.rs:19-66` |
| Every fragment and shell request path matches a words row | Guardrail `answer_words.rs` (table + `tests/http/answer_words.rs`) |
| A delete, retry, retrigger or swipe switch names the last Message, and the engine compares it in the same step as the busy check | `src/application/generation/gate.rs:539-546,580-582` |
| A swipe index is inside the Message's Swipe list | `gate.rs:542-546` (the `Path<usize>` is unvalidated at the HTTP layer, refused in the gate) |
| The last Message of a story is never deleted | `gate.rs:583-587` (`can_delete`), rendered off at `templates.rs:25` (`entries.len() > 1`) |
| A saved Games page past the end becomes the last page, and a page below 1 is a `400` | `src/domain/model/saved_games_page.rs:22`, `screen/handlers/screen.rs:25-34`; `GamesPagerView` saturates (`games/templates/games.rs:34-35`) |
| The stamp on a `POST` answer and the `revision` of `GET /changes` are the same numbering | one engine-wide `revision` raised by every kind (`src/domain/model/change.rs:146`, `ChangeVersions::stamp`), and `stamp_request_answers` reads it after the handler (`request_layers.rs:22-33`) |
| Every `POST` answer carries `X-Revision`, a refusal included | layer order in `router.rs:200-211`: `describe_answers` → stamp → trace → the Game layer |
| A form field the browser omits answers a refusal, not a silent write | `LastMessageForm` (`handler_helpers.rs:11-24`) and `AutoplayForm` are the only `serde(default)` HTTP forms; both are checked at the handler |
| Every `[data-error-slot]` is a direct child of the `form` or `.edit-actions` that `formSlotOf` searches | `:scope >` lookup at `dashboard-errors.js:45-46`; each slot is a direct child: `templates.rs:25`, `worlds.rs:122`, `settings.rs:196`, `games.rs:129,176`, `presets.rs:83`, `prompt_presets.rs:121` |
| An attribute the browser writes on a node inside a part is listed in `data-keep` | `hidden`/`style` and `aria-expanded` (`utils/error.rs:28`), settings sub-tab `aria-selected`/`hidden` (`settings/templates/settings.rs:30-70`), text-check `data-saved` (`settings.rs:312`); the story-log entry in edit is skipped instead (`dashboard-screen.js:114-125`) |
| A `data-saved-flag` names an element rendered in the same document | `world-posture-status` and `text-check-status` pairs; the World pair shares one `is_edit` (`worlds/templates/worlds.rs:131-132,211`), and `flagSaved` returns quietly when the element is absent (`dashboard-saves.js:44-48`) |
| A dialog opener renders an `id`, so focus can return after the dialog closes | browser guardrail `tests/browser/stub/slot_sweep.rs` |
| Every dialog fragment carries `data-dialog-form` and the heading `#edit-dialog-title` | all eight fragments: `prompt_presets.rs:95-96`, `games.rs:125-126,147-148`, `worlds.rs:85-86`, `settings.rs:141-142,270-271`, `templates.rs:163-164`, `presets.rs:15-16,64-65` |
| `hx-target`/`hx-include` selectors resolve | `#edit-dialog-body` and `#action-preview` are in the shell (`index.html:116,219`); `closest .posture-group`, `closest .posture-row`, `closest .preset-picker-row`, `closest form` and `next .connection-test-slot` are rendered in the same template body as the control that names them |
| The shell's tab ids match `data-tab` values | `index.html:94-99` (`<id>`) + `data-tab` vs `index.html:120-206` (`<tab>-tab`); `dashboard-panels.js:11` |
| `#story-log`'s edit lookup `[data-id]` cannot hit a Games row | the two parts never hold content at once: `leaveSection` empties the tab that closes (`dashboard-screen.js:400-408`); `data-id` appears only on story entries (`templates.rs:25`) and Games rows (`games/templates/games.rs:89`) |
| A request from a page with no active Game names no Game | `draftOfRequest` sends no header for an empty `data-game` (`dashboard-screen.js:415-428`), and the layer treats an absent header as the active Game |
| Every row of the `screen` part table is a part the server renders | guardrail `screen.rs:188-230` (stub `screen_part_html`) |
