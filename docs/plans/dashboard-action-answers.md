# Plan: One answer, one reader, and a fixed place for every report

**Status:** Approved by the owner on 2026-10-28, not started. The owner approved the design in `docs/diataxis/explanation/dashboard_design.md` (sections "An action answers one of five outcomes", "A report shows where the player can act on it", "One reader for every answer", and the focus sentence of the dialog bullet). Those sections are ahead of the code until this plan is built, and are committed with this plan. The owner authorized a commit at each gate that passes. No owner decision is open.
**Status:** Draft. The owner approved the design in `docs/diataxis/explanation/dashboard_design.md` (sections "An action answers one of five outcomes", "A report shows where the player can act on it", "One reader for every answer", and the focus sentence of the dialog bullet). Those sections are ahead of the code until this plan is built. They are committed together with this plan, once the owner approves it. No owner decision is open.
**Goal:** The server writes every sentence of an action answer, in one body format and one status rule. One piece of browser code reads every answer and shows every report. An invalid answer to a form with typed input shows inside that form. Every other report shows in the notice place `#screen-error`, which has a Close control. No answer shows in the Unreachable banner, and an open dialog repeats it in its status line. The server renders every control that sends a request and every place that holds a report.
**Branch:** `wf/change-driven-dashboard-refresh` (HEAD `82531b0b` when this draft was written).

This plan follows `old-docs/archived-plans/dashboard-visible-tab-refresh.md` (the "visible-tab plan"). That plan is done. Its final gate left a list of client findings. The rules of the visible-tab plan stand unless this plan changes one by name: P13, P14, P15, S1–S5 and the rule that only the screen script writes a part (section 8 of that plan). The section "Rules of the visible-tab plan that change" names each change.

## Changes the player will see

This list names every change of what the player sees. Each line cites the decisions that make it. The final report to the owner checks this list against the build.

| # | What the player sees | Decision |
|---|---|---|
| 1 | A failed Action reads one sentence that says what was not done, for example "The Game is busy, so the Action was not sent." The server writes it. The lead "That action failed." and the prefix "Error:" are gone. A failed screen load and a failed part lose the prefix too. | A1, A3, A4, A5, O6 |
| 2 | The sentence names the item when the server knows it. A row button names its Game, World, preset or Connection, for example `The Game "Name" was not deleted.` A Message action and an Action name none. | A4, A24 |
| 3 | The engine's own text is behind Details. A refusal shows its second line ("Still thinking...") there. | A1, O6 |
| 4 | The words use the terms of the product. "the command was not sent" becomes "the Action was not sent", "the run did not start" becomes "the Autoplay run did not start", and Game, World and Connection take a capital letter. The new-Options control reads "the Action was not sent". | A28 |
| 5 | A refusal, a failure, an invalid answer to a control that is not a form with typed input, and a failed screen load show in the notice place under the header, on every tab. Today a report shows beside its control, or in a strip at the bottom of the Game tab. The reports that move are those of the row buttons, the Settings rows and cards, the World rows, the preset cards, the LLM message cards, Reset, Stop and, in stage 4, the story-log controls. | A8, A10, A20, O4 |
| 6 | An invalid answer to a form with typed input shows inside that form, and the typed text stays. A message edit that the engine answers `400` now keeps its draft and shows the report in the entry. Today it reverts the edit. | A9, A15 |
| 7 | The notice has a Close control. Close hides the notice and moves focus to the main content. | A10, A27 |
| 8 | A notice stays until the player sends the next action, until a failing screen load works again, or until Close. A dialog opener counts as an action, so opening a dialog clears the notice. A good load after an action no longer erases the report of that action. | A11, A25 |
| 9 | An open dialog repeats the summary of every notice in its status line. It also repeats the Unreachable text, because the modal hides the banner. A refused or failed dialog save leaves the dialog open with its draft. | A12, A26 |
| 10 | No answer shows in the Unreachable banner, and not in a place beside the control. | A8, O9 |
| 11 | The preview of the text check closes only after a done answer. After a refused, invalid or failed confirm, or no answer, the preview stays open with the corrected text, and the report shows in the notice or the banner. Today the preview closes after every answer, and the corrected text is lost. | A23 |
| 12 | A dialog opener that fails reads "The form was not opened: <the engine's message>." in the notice. | A5, A2 |
| 13 | Closing a dialog returns focus to the control that opened it, also after a load rewrote the list. | A17 |

## Origin

The final review of the visible-tab plan found one weak point. An action answer is simple on the wire: `2xx` with `X-Revision`, `409` with `X-Refusal`, and another status for a failure. The client has no single reader for it. It reads the answer again at about eleven sites in five scripts, and each site picks its own slot and its own words (`tmp/final-review/thermo.md`, T1). The UI pass found two small defects of the same family: a story-log failure shows in a strip at the bottom of the Game tab (U3), and a refused Rename reads "That action failed. Error: …" (U4) (`tmp/ui-gates/final/findings.md`).

The owner then approved a design that settles where a report shows, how long it stays, and who reads it. This plan builds that design.

Known facts from the code at this HEAD (each was read, not guessed):

- A refusal is read in four ways. `htmx:responseError` reports it for a `BUTTON` only (`assets/dashboard-errors.js:195-212`). `settleSave` reports it for a form or field in a part or the dialog, and skips an element with `hx-on::after-request` (`assets/dashboard-saves.js:54`, `:79-82`). Four handlers in `assets/dashboard-action-area.js` report their own (`:254`, `:280`, `:295`, `:312`). Four `fetch` chains in `assets/dashboard-story-log.js` report their own (`:151-277`).
- The words of a report come from three places: the literal in a script, `data-refusal-what` in seven templates, and a default (`"the action was not done"`, `"the change was not saved"`). `REFUSAL_REPORTS` holds the reason (`assets/dashboard-errors.js:154-160`).
- The client makes slots and finds them by a list of containers. `inlineErrorSlotFor(elt, true)` appends a `generated-` slot into the nearest container (`assets/dashboard-errors.js:31-49`). The server renders 17 `data-error-slot` elements (`assets/index.html` and `src/`). A report of a transport failure also writes into a slot (`reportSlotFailure`, `:134-147`).
- The body of an error has two shapes. `render_error` adds `Error: ` (`src/adapters/driving/http/utils/error.rs:20`). `error_fragment` does not (`:13`). A `400` reads "That action failed. Error: msg" or "That action failed. msg", by the route.
- Three classifiers map errors to statuses: `settings_save_failure`, `delete_world_handler` and `action_error_response`. A displayable `ApplicationError::Engine` answers `500` through `IntoResponse` (`src/adapters/driving/http/error.rs:18-20`) and `400` through `delete_world_handler` (`src/adapters/driving/http/worlds/handlers/worlds.rs:208-217`).
- The Game mode, posture and preset saves are not Game-bound, but they can answer `409`: `save_config` refuses while an Autoplay run holds the Game (`src/application/generation/gate.rs:501-506`).
- `#screen-error` is in the shell under the header and the failure banner (`assets/index.html:86-92`). A successful load clears it (`clearScreenError`, `assets/dashboard-screen.js:255-260`, called from `morphParts`). The dialog already repeats its summary in `#edit-dialog-status` (`assets/dashboard-dialog.js:118-131`).
- htmx 1.9.10 fires `htmx:afterRequest` for every request that it sends and whose swap does not throw, also for a network error, an abort and a timeout. A swap that throws fires `htmx:onLoadError` instead. A request that `htmx:confirm`, validation, `htmx:beforeRequest` or `hx-sync` drops fires neither. `successful` is `status < 400` (vendored `assets/htmx.min.js`, `b.onload`). If htmx finds the element detached after the first `afterRequest` of a load, it fires the event a second time on the nearest ancestor that is in the body (known, from the same source).
- htmx dispatches each event twice: first the camelCase name (`htmx:afterRequest`), which bubbles to the body, then the kebab-case name (`htmx:after-request`), which `hx-on::after-request` listens to (known, `ce` and `jt` in the vendored source). So the body listeners run before the site handler. A site handler that detaches its element hides every later event of that request from the body: a `sendError` on a detached element does not reach `document.body` (known, from the same source).

## Design statements

The design document states these. Every decision below cites the statement that it serves. A rule that no statement supports is not in the plan.

| # | Statement | Where in `dashboard_design.md` |
|---|---|---|
| DS1 | An action answers one of five outcomes. **Done**: `2xx` with the revision. **Refused**: `409` (the Game is busy, or another Game is active now). **Invalid**: `400`, or `404` for a dialog that opens a record that is gone. **Failed**: `500`. **No answer**: the engine is unreachable. | "An action answers one of five outcomes" |
| DS2 | The status chooses the outcome. The server writes the sentence of every action answer. A sentence says what was not done and names the item where one exists, so it reads correctly away from its control. The engine's full text is behind Details. The words have one source, and a new action brings its sentence with it. | same section |
| DS3 | An invalid answer to a form with typed input shows inside that form, and the typed text stays. The forms with typed input are the dialog forms, the Action input, the Autoplay cycles and the message edit. Every other report (a refusal, a failure, a failed screen load, an invalid answer to a control that is not such a form) shows in the notice place of the shell, under the header. An open dialog repeats the summary in its status line. A refused or failed dialog stays open with its draft. No answer shows in the Unreachable banner. | "A report shows where the player can act on it" |
| DS4 | A form report stays until that form sends again or closes. A notice stays until the player sends the next action, until a failing screen load works again, or until the player closes it. The notice place is in the shell, so a tab change keeps it. | same section |
| DS5 | One piece of browser code reads every action answer and shows every report. A control adds only its own work after a done answer: it closes the dialog, clears the input or moves focus. A control that saves on change also returns to its rendered value when its save does not go through. | "One reader for every answer" |
| DS6 | The server renders every control that sends a request and every place that holds a report. | same section |
| DS7 | Closing the dialog returns focus to the control that opened it. | "View state survives a load…", dialog bullet |
| DS8 | The page shows "Saved", closes the dialog and clears the Action input only when the screen shows the revision of the answer. | "An action answers one of five outcomes" |
| DS9 | Each action carries the Game of the screen. The engine refuses an action from a page whose Game is no longer active. The refused text stays in the input. | "An action answers one of five outcomes" |

DS8 and DS9 hold in the code today. This plan keeps them and changes their words only.

## Decisions

### Owner decisions

The design settles these. They are owner decisions, not open ones.

| # | Decision | Statement |
|---|---|---|
| O1 | Scope: one reader (T1), one error-text family (E), the story-log actions through htmx (S), listeners that do not read request paths (T8), the places of a report, and dialog focus by opener id (C6). | DS1–DS7 |
| O2 | The gates are the four gates of the visible-tab plan: Light, Review, Milestone, Final. Step 5 of "How a stage runs" keeps its rule. | none (process) |
| O3 | The dashboard JS line count (`wc -l assets/dashboard-*.js`, 1,995 at this HEAD, checked) goes in the Gate log at every gate. It shows whether the work simplifies. It is not a limit. A stage that adds lines says why. | none (process) |
| O4 | An invalid answer to a form with typed input (dialog forms, Action input, Autoplay cycles, message edit) shows inside that form. Every other report (refused, failed, invalid for a control that is not such a form, such as a row button or a save-on-change select, and a failed screen load) shows in the notice place `#screen-error`. An open dialog repeats the summary in its status line and stays open with its draft. No answer shows only in the Unreachable banner. | DS3 |
| O5 | The notice place has a Close control. A notice clears when the player sends the next action, when a failing screen load works again, or when the player closes it. A form report clears when the form sends again or closes. | DS4 |
| O6 | The summary is the server's sentence alone. There is no "That action failed." lead. Details always shows, with the engine's full text. | DS2 |
| O7 | The server renders every control that sends a request. The message editor renders Save and Cancel in every entry, and the script builds only the textarea (A15). | DS6 |
| O8 | Focus returns to the opener by its `id`. The dialog records the `id` and focuses it on close (A17). A probe check runs first. | DS7 |
| O9 | A network error on any control, the preview confirm included, is "no answer". The one reader reads status 0 on `htmx:afterRequest`. | DS1, DS3 |
| O10 | Scenario 40.6 is deleted with its stub flag. Scenario 16.29 covers a failed dialog form. | DS1 |
| O11 | DS2 reads "The server writes the sentence of every action answer." The Unreachable banner, the summary of a failed screen load and the fallback for an empty answer stay in the script (old D1, option a). | DS2 |

### Plan decisions

The code and the owner decisions settle these. Each one is a rule of this plan.

| # | Decision | Reason | Statement |
|---|---|---|---|
| A1 | **A refusal is a `409` and nothing else.** Its body has two lines: the sentence `"<reason>, so <words>."`, then the engine's own line. The reasons are "The Game is busy" and "The active Game changed". The engine's lines are "Still thinking..." and "The page showed a Game that is no longer active." `Refusal::BadRequest` and `Refusal::Failed` are not refusals in this plan: they answer `400` and `500`. | The tier-1 specs say that a busy answer "carries the 'Still thinking...' line" (about 20 lines in `docs/specs/`). The second line keeps that true. | DS1, DS2, DS9 |
| A2 | **The status rule.** `400`: an invalid value, a POST that names a record that is gone, or a change that the record's state does not allow (a default preset, a World that still has Games). `404`: a GET that opens a dialog for a record that is gone. `409`: a refusal (A1). `500`: an engine failure. A displayable `ApplicationError::Engine` (`WorldHasGames`) is `400`, not `500`. No route reaches that error through `IntoResponse`: only `delete_world_handler` reaches it, and it answers `400` today. So the rule changes a unit test, not a route. Two openers change from `400` to `404`: the connection Edit form (Scenario 20.24) and the World Edit form. | This is what most handlers do today (the preset, Game and LLM openers answer `404`; the POSTs answer `400`). The rule fixes the three exceptions and writes the rest down. | DS1 |
| A3 | **One body.** Every failed action answers `<div class="error-message">` + text + `</div>`. The first line is the sentence. Any later line is the engine's text. There is no `Error: ` prefix. An answer that no constructor built (an extractor rejection, a bare status) gets the same body from the layer (A5). `render_error` is deleted. | `Error:` has no pin in any spec or test (grep of `docs/specs/` and `tests/`: 0 matches). | DS2 |
| A4 | **The first line, by outcome.** Refused: `"<reason>, so <words>."` Invalid: `"<Words>: <the engine's message>."` The engine's message tells the player what to fix, so the summary keeps it. Failed: `"<Words>."`, and the engine's text is line 2. `<words>` has a capital letter at the start of a sentence. Where an item exists, the sentence names it: `The preset "Name" was not deleted.` A handler that holds the item passes its name. A handler that holds only an id reads the display name on its failure path, and on no other path (A24). A Message action and an Action have no item name, so their sentences have none. | The engine knows what it refused and which item it read. A summary must read correctly away from its control. A failure text may be long (a storage error), so it stays behind Details. | DS2 |
| A5 | **The words come from one table, keyed by route, for every action route.** A row has the route, a noun and a predicate (`the edit was not saved`). The table has a row for each of the 35 `post` routes of the router. It also has a row for each of the ten `GET` routes that a dialog opener names: `/fragment/games/new`, `/fragment/games/:id/rename`, `/fragment/worlds/new`, `/worlds/:key/edit`, `/fragment/connections/new`, `/fragment/connections/:id/edit`, `/fragment/prompt-presets/new`, `/fragment/prompt-presets/:id/edit`, `/fragment/prompt-presets/:id/view` and `/fragment/llm-messages/:id`. Each of the ten can fail. Six answer `404` for a gone record. `/fragment/games/new` answers `500` (`src/adapters/driving/http/games/handlers/games.rs:124`). `/fragment/prompt-presets/new` answers `400` (`src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs:96`). Each opener that renders a template answers `500` when the template fails. A guardrail test reads the router source and fails for a `post` route with no row. It also reads each template source and `assets/index.html`, and it fails for an `hx-get` or `hx-post` path that matches no row (a `{{ … }}` in a path counts as a parameter, and a query string is dropped). One layer, `describe_answers`, reads the matched route and builds the first line of every answer of status 400 or higher on a route with a row. It takes the outcome from the status (A7), so an answer that no constructor built also gets its sentence: an extractor rejection (`Path` or `Form`, for example `/games/abc/rename`) or a bare status. The text that such an answer carried becomes line 2, as escaped text. A handler and the Game-header layer produce an answer with an outcome (and an item, A4) and no text. A route has one sentence, whoever sends it: `POST /action/check` serves the Action form and the new-Options form, and both read "the Action was not sent". The words use the terms of `CONTEXT.md` (A28). A new action adds its row in the same commit. | The handler cannot know the route. The route is the unit of "what was not done": one route is one action. A free string in four places (`tmp/final-review/thermo.md`, T6) becomes one table. | DS2 |
| A6 | **One classifier, `answer_for_error`,** replaces `settings_save_failure`, `delete_world_handler`'s branch and `action_error_response`. It builds a `400`, a `409` or a `500` from the table in section 1. The openers build their `404` with `action_not_found_response`: a gone record is a `None`, not an error value. | Three classifiers are three copies of one rule. | DS1 |
| A7 | **One decoder: `actionAnswer(event)`.** It returns `{ outcome, status, stamp, text }`. `outcome` is `done` (status below 400), `refused` (`409`), `invalid` (another `4xx`), `failed` (`5xx`) or `none` (status 0). `stamp` is the `X-Revision` header. The `4xx` fallback also reads a status that the app does not send today, such as the `422` of an axum form rejection, as invalid. | It replaces `requestSucceeded`, `answeredRefused`, `refusalOf`, `refusalOfResponse` and eight reads of `X-Revision` by name. | DS1, DS5 |
| A8 | **One reporter: `reportAnswer(event)`.** It listens to `htmx:afterRequest` only. It does not listen to `htmx:responseError` or to `htmx:sendError`. (1) `none`: call `markUnreachable()` and stop. No slot and no notice gets the report. (2) Any other answer calls `clearUnreachable()`. (3) `done`: stop. (4) `invalid`, and the sender has a form slot (A9): fill that slot. (5) Every other report: show the notice (A10). The summary is the first line of the body, and Details holds the whole text. | htmx dispatches the camelCase `afterRequest` to the body before the site handler runs, so the reporter sees a network error while the element is still in the page. `sendError` comes after a site handler has detached its element, and it does not reach the body, so a listener for it cannot work. The reporter needs no flag against a second report: a second `afterRequest` comes only when the site handler detached its element, and that happens only after a done answer (A23), which the reporter does not report. The cost of O9: an abort or a timeout would also read as "unreachable". The app has no abort or timeout source (stage 3 check 1 pins it). | DS3, DS5, O6, O9 |
| A9 | **The form slot.** The slot of a sender is the `[data-error-slot]` that is a direct child of `sender.closest("form, .edit-actions")`. A sender with no such container, or a container with no slot, has no form slot, and its report goes to the notice (A8 step 6). There is no list of containers, no walk up the tree, no slot made by a script and no `data-error-into`. | The server renders a slot in each form of the DS3 list and nowhere else (A14). The rule then needs no knowledge of the control kind or of the field types. | DS3, DS6 |
| A10 | **The notice place.** `#screen-error` renders a report container `[data-report]` and a Close button `[data-close-notice]`. The script writes only the report container. The place holds one report, and the latest report wins. It carries `data-source="action"` or `data-source="screen"`. A load that fails writes `screen`, and an action report writes `action`. A click on Close hides the place, empties the report and moves focus to `#main-content` (A27). The report container holds `data-error-slot="screen"`, and the place holds none, so the writer and the clearer of a slot never touch the Close button. The place stays a direct child of `<body>` in the shell, so no load morphs it. | One fixed place on every tab (DS3). The source decides which clear applies (A11). | DS3, DS4, DS6 |
| A11 | **Clearing.** (1) The body listener of `htmx:beforeRequest` clears the notice, and the form slot of the sender, for every htmx request. A dialog opener (`GET`) counts (A25). The screen load and the ask use `fetch`, so they clear nothing. A request that a confirm, a validation or `hx-sync` drops fires no `beforeRequest`, so it clears nothing. (2) `clearScreenError` clears the notice only when it has `data-source="screen"`. A good load therefore never erases the report of an action. (3) Close clears the notice. (4) A dialog close drops its body, and its form slot with it. | The page asks for the screen after each action, and that load works. Without the source, the load would erase the report at once. | DS4 |
| A12 | **The dialog repeats the summary.** The event `screen:error` is renamed `notice:changed`. The notice fires it with the summary, and with an empty summary when it clears. `refreshFailureBanner` fires a second event, `banner:changed`, with the Unreachable text, and with an empty text when the banner has none (A26). `renderDialogStatus` shows the notice summary and the Unreachable text while the dialog is open. A refused or failed save leaves the dialog open with its form. | The status line and its listener exist (`assets/dashboard-dialog.js:118-131`). Only the sources of its text widen: from a screen failure to every notice, and to the Unreachable text. | DS3 |
| A13 | **What stays at a site.** Only the success work and the restore: the pending state, the input clear, the dialog close, focus, the announce, the restore of a save-on-change control, and the revert of a story-log edit. No site decodes the answer by itself, and none writes a slot. | This is T1. | DS5 |
| A14 | **The slot guardrail.** The server renders one `[data-error-slot]` as a direct child of each form of the DS3 list: every `form` in the edit dialog, `#command-form` and `#autoplay-form`. The `.edit-actions` group of an entry renders one too. The report container of `#screen-error` (`[data-report]`, A10) is the one other slot. No other element holds `data-error-slot`. The list is by kind of form, as DS3 gives it. The New Game form holds selects only, and it is a dialog form, so it keeps its slot. A stub check (the sweep, Scenario 44.45) loads each tab, opens each dialog opener, puts one entry in edit (stage 4), and tests every `form`, every `.edit-actions` and every `[data-error-slot]` against this rule. | The rule is the server's half of A9. One check covers every control (DS6). | DS6 |
| A15 | **The message editor (option c: the server renders Save and Cancel).** Each entry renders a group `.edit-actions` with Save (`hx-post="/history/{id}"`, `hx-include="#edit-textarea"`, `hx-swap="none"`, `hx-sync="this:drop"`), Cancel (`onclick="cancelEdit()"`) and the form slot of A14. CSS shows the group and hides the other `.message-actions` controls while the entry has `data-editing`. `showEditForm` builds the textarea only (with `name="text"`). `editState.actions` and the `innerHTML` swap of the buttons go. Save keeps the class `save-btn` and Cancel keeps `cancel-btn`, so the CSS of P18 (`assets/styles.css:1519-1534`) still applies. Ctrl+Enter and Cmd+Enter in the textarea click the Save of the entry in edit (`entry.querySelector(".save-btn").click()`), so the click guard and `hx-sync` still apply. This replaces the `submitEdit()` call of the keydown handler (`assets/dashboard-story-log.js:96-98`). Every entry holds a Save, so a test selects `[data-editing] .save-btn`, not `.log-entry.narration .save-btn`. The P9 root lock, the skip of an entry in edit and the O3 refusal (Save is refused during a Job, the draft stays) stay as they are. | The textarea is a draft and must stay out of the server's render (P8). The edit slot lives in the entry in edit, which a load skips, so it needs no rule to survive a load. | DS3, DS6 |
| A16 | **The story-log controls are htmx controls.** Retry, Retrigger, the two swipe controls and Delete render `hx-post` with `hx-swap="none"`. Delete also renders `hx-confirm="Delete this message?"`. Retry and Retrigger render `hx-on::before-request="updateToThinking()"` and `hx-on::after-request="endPendingAfterRequest(event)"`, the handler of the new-Options form (renamed). The Edit control stays `onclick="showEditForm(id)"`: it sends no request. `actionFetch` is deleted. Their failures show in the notice place, not under the entry. | S5 gives the three pending paths one end. S1 already ends every htmx action with a stamp reset and an ask. | DS5, DS6 |
| A17 | **Focus by opener id.** Each dialog opener renders a stable `id` (section 5). The dialog records `evt.detail.requestConfig.elt.id` when a swap opens it. On close it calls `document.getElementById(id)?.focus()`. If the opener is gone, focus stays where the browser puts it, on `<body>`. The request-matching code goes. | An id survives a load that replaces the node, and the call does not depend on node identity. It also works in a browser that does not focus a button on click (Safari, Firefox on macOS; inferred, not checked here). | DS7 |
| A18 | **No body listener reads what another wrote. The site handler runs after them.** The reporter writes the form slot and the notice. `settleSave` writes controls and the Saved flag. The dialog listener writes the dialog. `endRequest` writes the stamp. The four body listeners of `htmx:afterRequest` are in `assets/dashboard-dialog.js:162`, `assets/dashboard-errors.js:218`, `assets/dashboard-saves.js:85` and `assets/dashboard-screen.js:87`. htmx dispatches `htmx:afterRequest` to them first, then `htmx:after-request` to the site handler (`hx-on::after-request`). So the order of the body listeners does not matter, and the plan fixes none. `endRequest` forgets the stamp before a site handler calls `afterScreenShows` (as today). | There is no order to keep among the body listeners. The order between the body and the site handler is htmx's, and it decides O9. | DS5 |
| A19 | **`isSavingControl` loses its `hx-on::after-request` clause.** `settleSave` no longer reports. On a form with no `data-autosave` and no `data-saved-flag` it does nothing that matters. The `AFTER_REQUEST_FORMS` list, `check_after_request_forms_are_listed`, `check_after_request_form_rows_are_used` and their detection tests go. | The list exists only to stop a second report. | DS5 |
| A20 | **No report lives in a part.** The slots of the Settings role rows, connection rows and text-check card, the World rows and posture group, the preset cards and the LLM message cards go. So do the generated slots and `#story-log-error`. The keep rule `showsInlineError` goes with them (`assets/dashboard-screen.js:43-49`, `:101-103`), and so do its twin in `tests/test_utils/convergence.js:39-78` and the words of `tests/STRATEGY.md` (section "Convergence check", line 234: "a showing inline error slot"). The convergence check still skips an entry in edit. | Each of these slots is a place beside a control, and DS3 has no such place. Six of the seven sit in a part. `world-posture` sits in the dialog, but its group is not a form of the DS3 list, so its report goes to the notice, and the open dialog repeats it. | DS3 |
| A21 | **The stub mounts the engine's words layer.** The engine exports the words table (`REFUSAL_WORDS`) and the layer `describe_answers`. The stub server mounts the same layer, added after `action_answers_middleware`, so that the layer also wraps the stale-Game refusal of the stub (`tests/test_utils/stub_server.rs:981-984`). The stub's stale-Game check and its busy refusal call the engine's `refused(kind)` and give no text. The stub keeps its own list of Game-bound routes, `GAME_BOUND_ROUTES` and `is_game_bound` in `tests/test_utils/stub_action_layers.rs`, for its Game-header check. The comparison of that list with `game_bound_routes` in Scenario 45.4 stays. The words table has no `game_bound` field. The stub builds its error answers through the engine's constructors, which stage 2 makes `pub`. The layer rule of A5 covers only an answer that no constructor built. | The stub then writes its sentences through the engine's own layer, so a sentence cannot drift. One shared Game-bound list (thermo T7) is outside O1, so this plan does not build it. | DS2, O1 |
| A22 | **Listeners find their forms by attribute or id, not by request path (T8).** The `htmx:onLoadError` listener of the check request (`assets/dashboard-action-area.js:247-251`) becomes `hx-on::on-load-error="onCommandLoadError()"` on `#command-form`. The `htmx:beforeRequest` listener of the confirm path (`:273-278`) becomes `hx-on::before-request="onPreviewBeforeRequest()"` on the two preview forms. No script reads `requestPath`. Stage 3 checks that htmx 1.9.10 runs `hx-on::on-load-error` first. | A load error on the new-Options form also posts `/action/check`, so the path test ends a check in flight by mistake. | DS5 |
| A23 | **The preview closes only after a done answer.** `onPreviewSendAfterRequest` (`assets/dashboard-action-area.js:280-291`) calls `closeActionPreview()` after every answer today. After this plan it calls it only after a done answer. After a refused, invalid or failed confirm, or no answer, the preview stays open with the corrected text, and the report shows in the notice or the Unreachable banner. This changes the behaviour that Scenarios 16.40 and 16.44 state (`docs/specs/browser_dashboard.md:364`, `:411`: "Then the preview closes"). The plan names the change (stop condition 3), and section 6 changes both scenarios. No other site handler detaches its element after an answer that the reporter reports (known for the preview; inferred for the others from the review `tmp/followup-plan/review2.md`, H1). | A control adds only its own work after a done answer (DS5), and a refused or failed answer leaves the form open with its draft (DS3). The close after a failure also dropped the text that the player corrected. | DS5, DS3 |
| A24 | **A handler that holds only an id reads the display name on its failure path.** These handlers hold an id: Game Switch and Delete (`src/adapters/driving/http/games/handlers/games.rs:148`, `:169`), World Delete (`src/adapters/driving/http/worlds/handlers/worlds.rs:208`), preset Delete (`src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs:229`) and Connection Delete (`src/adapters/driving/http/settings/handlers/settings.rs:345`). After a failed answer, each reads the display name of its item once, and the sentence names it. If the read fails or finds nothing, the sentence has no name. No handler reads a name on the success path. A handler that already holds the item passes its name with no new read: preset Duplicate and Activate (`require_preset_action!`, `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs:240`, `:266`) and the form handlers. | The reports that DS3 moves away from their control are the reports of row buttons, and a row button handler holds an id only. Without the name, the sentence reads wrong away from its row. The read is one local SQLite read (inferred, not measured), and only on a failed answer. | DS2 |
| A25 | **A dialog opener clears the notice like any other action.** The clear of A11 runs for each htmx request, an opener (`GET`) included. | DS1 counts the answer of an opener as an action outcome. Without the clear, a refusal of an earlier Send would show in the status line of a Rename dialog that the player opens later (A12). The only htmx `GET` requests of the app are the 13 `hx-get` openers (grep). The screen load and the ask use `fetch`. | DS4 |
| A26 | **The dialog status line repeats the Unreachable text while the dialog is open.** `refreshFailureBanner` (`assets/dashboard-errors.js:70-80`) fires `banner:changed`. `renderDialogStatus` adds the text of the Unreachable banner to the status line while the dialog is open and the banner shows it. The next answer clears the banner and the line. | The reason of DS3 holds for the banner: the modal hides the page, so the banner is behind the backdrop, and assistive technology does not read the `role="alert"` of an inert page (inferred, not checked). The status line is `role="status"` (`assets/index.html:231`). | DS3 |
| A27 | **Close moves focus to `#main-content`.** The click handler of Close calls `clearNotice()`, then focuses `#main-content`. | Close hides the control that holds focus, so focus would fall to `<body>`. `#main-content` has `tabindex="-1"` and is the target of the skip link (`assets/index.html:74`, `:102`), so the next Tab goes into the tab content. | DS4 |
| A28 | **The words use the terms of `CONTEXT.md`.** A sentence says "Action", not "command", and "Autoplay run", not "run". It writes Game, World, Connection, Swipe, Message, Retry and Narrator with a capital letter. The prose of this plan calls the new-Options form the "new-Options form". A plain word that `CONTEXT.md` has no entry for ("preset", "posture", "retrigger") stays plain. | `CONTEXT.md` lists "Command (alone)", "Run (alone)" and "regen" as words to avoid. The player reads these sentences. | DS2 |

### Open owner decisions

No question is open. The owner decided D1 as O11 (2026-10-28).

### Rules of the visible-tab plan that change

| Rule | Change |
|---|---|
| P13, "the text stays 'Still thinking...'" and the last two sentences | The text "Still thinking..." is line 2 of the `409` body (A1). The page no longer builds a refusal message from `X-Refusal`. It reads the first line of the body (A8). `X-Refusal` is deleted. |
| P14, "with `409` and 'The active Game changed'" | The first line is "The active Game changed, so <words>." The engine's line is line 2 (A1). |
| P15, `INLINE_ERROR_CONTAINERS`, and "a load keeps a slot that shows an error" | The list is deleted. The slot rule is A9. A load has no slot to keep (A20). The script no longer makes a slot. |
| S1, the end `.finally` of `actionFetch` | Stage 4 deletes `actionFetch`, so that end goes. The ends `htmx:afterRequest` and `htmx:onLoadError` stay. |
| S5, the new-Options form | The new-Options form no longer reports its own `409`. The one reader does. Its after-request handler only ends the pending state (A16). |
| S5, retry and retrigger through `submitGenerationRequest` | The path goes (A16). Retry and Retrigger end the pending state through `endPendingAfterRequest`. |
| P18 | Stays. The new Save keeps the class `save-btn` (A15). |
| Section 3, focus return | "The dialog records the opener's request" becomes "the dialog records the opener's id" (A17). |
| Section 8, guardrails | The `PART_WRITE_EXCEPTIONS` row for `inlineErrorSlotFor` goes. The guardrail "dashboard scripts send actions through actionFetch" becomes "no script calls `fetch(` except the two reads of `dashboard-screen.js`" (stage 4). |

## The architecture after this plan

1. **An action answers once, in one shape.** `2xx` with `X-Revision`; `409` with a sentence and the engine's line; `400`, `404` or `500` with a sentence and the engine's text. The server writes the first line from one table. The body has no prefix.
2. **The page reads the answer in one place.** `actionAnswer` decodes it. `reportAnswer` shows it. A site keeps only its success work.
3. **A report has two places.** Inside a form of the DS3 list, an invalid answer. In the notice place of the shell, every other report. No answer shows in the Unreachable banner.
4. **The server renders the places.** Each form of the DS3 list renders its slot. The shell renders the notice place with its Close control. The script fills them and empties them. It never makes one.
5. **The story log is a part like the others.** Its controls are server-rendered htmx controls.
6. **Focus returns by id.** The dialog records the id of its opener.

## Design

### 1. The answer of an action (server)

- **The outcomes (DS1).** Done: `2xx` with `X-Revision` (P13 stands). Refused: `409`. Invalid: `400`, or `404` for an opener. Failed: `500`.
- **The status rule (A2).**

  | Kind | Status | Examples today |
  |---|---|---|
  | Invalid value | `400` | an empty Game name; `Refusal::BadRequest`; `EngineError::Validation` and `Config` from a settings save |
  | A POST names a gone record | `400` | `Message not found` (Scenario 8.9); `Preset not found` on update or delete; `World 'key' not found` on update |
  | The record's state does not allow it | `400` | a default preset; `WorldHasGames` (`400` through `delete_world_handler` today; the `IntoResponse` path answers `500`, and no route reaches it) |
  | A GET opens a dialog for a gone record | `404` | the Rename form, the preset Edit and View forms, the LLM record (all `404` today); the connection Edit form and the World Edit form (`400` today, `src/adapters/driving/http/settings/handlers/settings.rs:211`, `src/adapters/driving/http/worlds/handlers/worlds.rs:148`) |
  | Refusal | `409` | a busy Game; a stale Game header |
  | Engine failure | `500` | a storage error; `Refusal::Failed`; a render failure of `GET /screen` |

- **The body and the first line (A3, A4).** A handler answers a status and an outcome, and the layer `describe_answers` writes the body. `error_fragment(text)` is the only builder of `<div class="error-message">`. `render_error` is deleted, and its callers use `error_fragment`: the two `IntoResponse` impls in `src/adapters/driving/http/error.rs`, `action_layers.rs`, `utils/fragment.rs:29,37`, `src/adapters/driving/http/action/handlers/actions.rs:99` and `src/adapters/driving/http/screen/handlers/screen.rs:66-82`. A `GET /screen` failure loses its `Error:` prefix too. So does a part failure text that the try-render wrapper writes (`fragment.rs`). It shows in `#screen-error`, whose summary is its own line. The stub calls `render_error` too (`tests/test_utils/stub_server.rs:1079, 1236, 1247, 1576, 1647, 1943, 1952`; `tests/test_utils/stub_action_layers.rs:99`). `error_fragment` becomes `pub` for it. Every route that a control names has a row (A5). A route that no control names keeps the body that its handler wrote.

  | Outcome | Line 1 | Line 2 |
  |---|---|---|
  | Refused | `The Game is busy, so the edit was not saved.` | `Still thinking...` |
  | Refused (stale Game) | `The active Game changed, so the edit was not saved.` | `The page showed a Game that is no longer active.` |
  | Invalid | `The Game was not renamed: Display name cannot be empty.` | none (the message is in line 1) |
  | Failed | `The preset was not deleted.` | the engine's text |

  The words and the layer names are working names. Stage 2 pins the exact form with Scenario 45.19. A message that already ends with a period gets no second one.
- **The classifier (A6).** One function, `answer_for_error(error, how)`, in `utils/error.rs`. The input `how` holds the text policy and whether the route is a settings route. The text policy is `Detail` (the engine's text goes to line 2) or `Generic` (the generic text that `IntoResponse` uses today goes to line 2). It returns an answer with an outcome, the engine's text and, where the handler holds it, the item:

  | Error | Answer |
  |---|---|
  | `ApplicationError::Validation(m)` | invalid (`400`), `m` |
  | `EngineError::Validation(m)`, `EngineError::Config(m)`, on a settings route only | invalid (`400`), `m` |
  | `ApplicationError::Engine(WorldHasGames)` | invalid (`400`), its message |
  | `ApplicationError::Refused(StillThinking)` | refused (`409`), kind `Busy` |
  | `ApplicationError::Refused(BadRequest(m))` | invalid (`400`), `m` |
  | `ApplicationError::Refused(Failed(f))` | failed (`500`), `f.line()` |
  | any other engine error, and `EngineError::Validation` or `Config` on a route that is not a settings route | failed (`500`), the text that `how` names |

  Today only `settings_save_failure` maps `EngineError::Validation` and `Config` to `400`. Every other route answers `500` for them, and this plan keeps that. The policy of which errors show their text stays as it is. `ApplicationError::Engine` that is not displayable keeps its generic text on the `IntoResponse` path (`src/adapters/driving/http/error_tests.rs:36-47` pins that). This plan changes the format and the first line, not the policy.
- **The refusal (A1, A5).** `refused(kind)` builds a `409` with an empty body and the kind as a response extension. `refuse_stale_game` and `Refusal::StillThinking` both call it. The layer reads the kind, reads `MatchedPath`, looks the route up in `REFUSAL_WORDS` and writes the body. The layer sets `X-Refusal` until stage 4, so the story-log chains keep working. The names are working names.

  These routes can answer `409`. Their words:

  | Route | Words |
  |---|---|
  | `/action`, `/action/check`, `/action/confirm` | the Action was not sent |
  | `/autoplay` | the Autoplay run did not start |
  | `/swipe/new` | the Retry was not started |
  | `/retrigger` | the retrigger was not started |
  | `/reset` | the Game was not reset |
  | `/message/:id/swipe/:index` | the Swipe was not switched |
  | `/history/:id` | the edit was not saved |
  | `/history/delete` | the Message was not deleted |
  | `/status/reset-busy` | the Autoplay run was not stopped |
  | `/games/:id/mode`, `/games/:id/posture`, `/games/:id/presets` | the change was not saved |

  The other 21 `post` routes and the ten openers get a row of the same form. The words follow the verb of the route: `/games` "the Game was not started", `/games/:id/switch` "the Game was not switched", `/games/:id/rename` "the Game was not renamed", `/games/:id/delete` "the Game was not deleted", `/worlds` "the World was not created", `/worlds/:key` "the World was not saved", `/worlds/:key/posture` "the posture was not saved", `/worlds/:key/delete` "the World was not deleted", `/connections/add` and `/connections/:id/edit` "the Connection was not saved", `/connections/test` and `/connections/:id/test` "the Connection was not tested", `/connections/:id/delete` "the Connection was not deleted", `/connections/set-narrator` "the Narrator was not changed", `/connections/set-quantifier` "the Quantifier was not changed", `/settings/text-check` "the text-check setting was not saved", `/prompt-presets` and `/prompt-presets/:id` "the preset was not saved", `/prompt-presets/:id/delete` "the preset was not deleted", `/prompt-presets/:id/duplicate` "the preset was not duplicated", `/prompt-presets/:id/activate` "the preset was not activated". An opener reads "the form was not opened". The words of the first group are the words that the client builds today, with the terms of `CONTEXT.md` (A28), apart from the new-Options form (A5) and the three config saves, which got the default "the change was not saved" from `settleSave`. The words of `data-refusal-what` keep their meaning and take the terms of `CONTEXT.md`: "the game was not reset", "the game was not renamed" and "the game was not started" take a capital G, "the run was not stopped" becomes "the Autoplay run was not stopped", and "the connection was not saved" becomes "the Connection was not saved". The two preset words stay.

  The Game row buttons Switch and Delete cannot answer `409` (`GameCatalogue::switch_game` calls `ActiveGame::set`, and `GameCatalogue::delete_game` calls `GameWriter::delete`; neither checks the busy state; known from `src/application/games/catalogue.rs:67-74` and `:88-96`, `src/application/games/active.rs:39-47` and `src/application/generation/gate.rs:609-616`). They get a row for their `400` and `500`.
- **`MatchedPath`.** axum 0.7.9 puts `MatchedPath` in the request extensions for a middleware that runs after routing (known from its source and tests, `extract/matched_path.rs`). The engine router uses `.route` and `.merge`, not `nest_service`, for these routes (`nest_service` serves only `/assets` and `/data`). The `409` of `refuse_stale_game` comes from a `route_layer` inside the endpoint, so a layer outside it sees that response. Condition: `describe_answers` must be added after every `.route` call, because `Router::layer` wraps only the routes that exist at that call (known from `axum-0.7.9/src/routing/mod.rs` and `path_router.rs`). It need not be the last `.layer`. In the engine router it goes next to `stamp_action_answers` (`router.rs:248`), and `TraceLayer` follows it (`router.rs:252`). The stub router also uses `.route` and `.layer`. There the layer is added after `action_answers_middleware` (`stub_server.rs:981-984`), so that it wraps the stale-Game refusal that the stub produces. Stage 2 checks it first. If the layer does not see the route, it falls back to the segment matcher that the stub has today (`tests/test_utils/stub_action_layers.rs:28-41`), moved into the engine.

### 2. The reader (client)

- **`actionAnswer(event)` (A7)** lives in `dashboard-errors.js`. It reads `event.detail.xhr`. `text` is the plain text of the body. The story-log and dialog sites read `stamp` and `outcome` from it.
- **`reportAnswer(event)` (A8).** The steps are in A8. The camelCase `afterRequest` of a network error reaches the body before the site handler runs, so the sender is still in the page. The reporter has no `sendError` listener.
- **The report.** The summary is the first line of the body. Details holds the whole text. The two parts come from the one body (A4), so a `409` reads "The Game is busy, so the edit was not saved." with both lines behind Details. There is no client lead and no client words. An answer with an empty body (a panic, a proxy) reads "The server answered <status>." This is the one sentence of an action answer that the client writes (D1).

  | Answer | Where | Summary | Details |
  |---|---|---|---|
  | `409` | notice | the first line of the body | both lines |
  | `400`/`404`, sender in a form with a slot | that form's slot | the first line of the body | the whole text |
  | `400`/`404`, any other sender | notice | the first line of the body | the whole text |
  | `5xx` | notice | the first line of the body | the whole text |
  | no answer (status 0) | the Unreachable banner, and the dialog status line while a dialog is open (A26) | the banner text | none |
- **The notice (A10, A11).** `showNotice(summary, detail, source)` fills `[data-report]` of `#screen-error`, sets `data-source`, un-hides the place and fires `notice:changed`. `clearNotice()` empties it, hides it and fires `notice:changed` with an empty summary. `reportScreenError` and `clearScreenError` call them (with source `screen`). `refreshFailureBanner` fires `banner:changed` (A12). `screenErrorShows` stays: a failure that lasts writes nothing the second time, so an open Details stays open.
- **The form slot (A9).** `formSlotOf(elt)` returns a slot or `null`. `clearInlineError` empties and hides a slot. It never removes one.
- **What the sites keep (A13).**

  | Site | Success work | Other work |
  |---|---|---|
  | Action form (`#command-form`, `onCommandAfterRequest`) | clear the input after the screen shows the stamp; begin the pending state | `checkInFlight = false`; sync the action state |
  | Preview forms (`onPreviewSendAfterRequest`) | as the Action form; close the preview, only after a done answer (A23) | none; the preview stays open after any other answer, and the reporter has already reported it (O9) |
  | Autoplay form (`onAutoplayAfterRequest`) | begin the pending state | none |
  | Regen, Retry, Retrigger (`endPendingAfterRequest`) | end the pending state when the screen shows the stamp | end it at once when not done |
  | Save-on-change control and dialog-less forms (`settleSave`) | settle the fields; flag Saved when the screen shows the stamp | on a failure: restore the rendered value |
  | Dialog form (`dashboard-dialog.js`) | `data-saving`, then close when the screen shows the stamp | none |
  | Story-log Save | clear the edit state; unlock when the screen shows the stamp | refused or invalid: keep the draft and the lock. Failed or no answer: revert the edit and release the lock (as today, Scenarios 30.4 and 30.16) |
  | Swipe switch, Delete | announce the restore, or move focus, when the screen shows the stamp | none |
- **T8 (A22).** The two listeners become `hx-on::` attributes. Stage 3 checks the event order.

### 3. The places

**The notice place (DS3, DS4, A10).** `assets/index.html` renders it under the failure banner (`:86-92`):

```html
<div id="screen-error" class="screen-error" role="alert" hidden>
  <div data-error-slot="screen" data-report></div>
  <button type="button" class="notice-close" data-close-notice aria-label="Close">…</button>
</div>
```

Attribute and class names are working names. The Close control has an icon (`#i-x` exists in the sprite) and an accessible name. A click handler calls `clearNotice()` and moves focus to `#main-content` (A27).

**The form slots (A14).** The forms of the DS3 list keep the slot that the server already renders as a direct child of the form:

| Form | Slot | Template |
|---|---|---|
| Action input (`#command-form`) | `action` | `assets/index.html:140` |
| Autoplay cycles (`#autoplay-form`) | `autoplay` | `assets/index.html:163` |
| Game Rename | `game-rename` | `src/adapters/driving/http/games/templates/games.rs:129` |
| New Game | `new-game` | `src/adapters/driving/http/games/templates/games.rs:176` |
| Preset Add (each type) | `preset-add-{type}` | `src/adapters/driving/http/prompt_presets/templates/prompt_presets.rs:121` |
| Preset Edit | `preset-edit-{id}` | `src/adapters/driving/http/builders/presets.rs:83` |
| Connection form (Add and Edit) | `connection-form` | `src/adapters/driving/http/settings/templates/settings.rs:184` |
| World form (Create and Edit) | `world-form` | `src/adapters/driving/http/worlds/templates/worlds.rs:123` |
| Message edit (new) | `edit-{id}`, inside `.edit-actions` | `src/adapters/driving/http/templates.rs:25` (stage 4) |

**The slots that go (A20).** Each only placed a report beside its control:

| Slot | Template |
|---|---|
| `role-{role}` | `src/adapters/driving/http/settings/templates/settings.rs:43` |
| `connection-{id}` | `src/adapters/driving/http/settings/templates/settings.rs:64` |
| `text-check-card` | `src/adapters/driving/http/settings/templates/settings.rs:315` |
| `world-{key}` | `src/adapters/driving/http/worlds/templates/worlds.rs:35` |
| `world-posture` | `src/adapters/driving/http/worlds/templates/worlds.rs:134` |
| `preset-{id}` (inside the card string) | `src/adapters/driving/http/builders/presets.rs:154` |
| `llm-msg-{id}` | `src/adapters/driving/http/templates.rs:158` |
| `story-log` (`#story-log-error`) | `assets/index.html:183-185` (stage 4) |

**Other senders.** The preview forms, Stop, the row buttons, the saved-Game and active-Game buttons, the save-on-change selects and the new-Options form are not forms of the DS3 list. Their reports go to the notice. The preview has a textarea for the corrected text, but DS3 does not name it, so this plan treats the preview confirm like any other control: its reports go to the notice, and the preview stays open with its text after any answer but a done one (A23). See Risks.

**The sweep (A14).** It is Scenario 44.45, a stub check in the shape of Scenario 44.26. Its stub data needs: the six dialog forms (Rename, New Game, preset Add and Edit, connection Add and Edit, World Create and Edit), a Game row of each state, a World, a preset of each type, a connection, the Settings tab, and a story log of three entries with one entry in edit (stage 4). The stub renders the real templates (`tests/test_utils/stub_server.rs:97-106`, `:935-980`), so the sweep sees the real markup. Stage 3 checks that the stub renders each of these, and adds what is missing. Until stage 4, the sweep allows the `story-log` slot.

### 4. The story log (S)

- **Controls (A15, A16).** The server renders `hx-post` for Retry, Retrigger, the two swipe controls, Delete and Save. The `view_only` rule (`disabled`) stays server-rendered.
- **Success work.** Retry and Retrigger end the pending state through `endPendingAfterRequest`. A swipe switch announces the restore when the screen shows its stamp. Delete moves focus when the screen shows its stamp. Save clears the edit state at once, then unlocks the root when the screen shows its stamp (P9, DS8).
- **Failure work.** The reporter shows a refusal and a failure in the notice place and an invalid answer to Save in the edit slot (A8). A Save that is refused (`409`) or invalid (`400`) keeps the draft and the lock (O3, DS3). A Save that fails in another way, or gets no answer, reverts the entry and releases the lock, as today (Scenarios 30.4 and 30.16). The revert is the one story-log failure path that stays in the script.
- **The click guard.** The capture-phase click guard (`assets/dashboard-story-log.js:55-68`) stops a click on a locked control before htmx sees it. It works the same for an htmx control.
- **The edit slot.** The slot is in the entry in edit, and a load skips that entry (`data-editing`). So a report in it needs no keep rule. It clears when the form sends again (A11) and when Cancel ends the edit: the next load renders the entry again with a hidden slot.
- **Deleted.** `actionFetch`, `throwServerDetail`, `storyLogErrorSlot`, `showStoryLogError`, `reportedRefusal`, `submitEdit` (the keydown of the textarea clicks Save, A15), `submitGenerationRequest`, `submitNewSwipe`, `submitRetrigger`, `switchSwipe`, `deleteMessage`, the `#story-log-error` element and its CSS (`assets/styles.css:1353-1360`), `editState.actions`, `X-Refusal` and its header constant, `REFUSAL_REPORTS`, `reportRefusal`, `refusalOf` and `refusalOfResponse`.

### 5. The focus of a dialog (C6, A17)

- **Openers get ids (13).** `id` on each of 13 openers: the four preset Add buttons (`preset-add-open-{type}`), the Rename button (`game-rename-open-{id}`), New Game (`game-new-open`), Create New World (`world-new-open`), World Edit (`world-edit-open-{key}`), Connection Edit (`connection-edit-open-{id}`), the row Test (`connection-test-open-{id}`), Add Connection (`connection-add-open`), preset View (`preset-view-open-{id}`) and preset Edit (`preset-edit-open-{id}`). The LLM record opener, the 14th that targets `#edit-dialog-body`, has its id already (`llm-msg-open-{id}`, `src/adapters/driving/http/templates.rs:153`). The server renders every id (DS6).
- **The dialog.** It records `dialogOpenerId` in the `htmx:afterSwap` handler, and on close it focuses the element with that id. A new load that replaces the node keeps the id, so the call finds the new node.
- **Deleted.** `requestOf`, `hasRequest`, `partHolding`, `openerNow`, `OPENER_REQUEST_ATTRIBUTES`, `dialogOpenerRequest`, `dialogOpenerPart`, `dialogOpener`, and the `focusIsFree` rule (`assets/dashboard-dialog.js:27-109`).
- **The probe checks first.** P1: in Chromium, after a dialog closes by Cancel and by Escape, `document.getElementById(id).focus()` leaves focus on an opener that the player opened by click, and on one opened by keyboard. P2: the vendored `Idiomorph.morph`, with the callbacks of `Idiomorph.defaults`, keeps an opener with an `id` through a load that removes a row above it, and `getElementById` then returns an element with the same text. A control run with the id removed must fail: it shows that the check can fail. If P1 or P2 fails, C6 drops out (O8). The request-matching code stays, and the Gate log records why.

### 6. Checks

The oracle stays green through every stage that does not change it: the refusal and failure checks that exist today. They are:

- Tier 2 (`tests/browser/stub/`): `form_failures.rs` (16.29–16.32, 16.35); `dashboard.rs` (16.42, 16.44, 16.45); `failure_display.rs` (16.20–16.22, 16.25, 16.27, 16.33, 16.34, 16.37, 16.38, 16.40, 16.41, 16.43); `game_header.rs` (16.53–16.59, 16.61); `action_controls.rs` (16.62–16.71); `story_log_edit.rs`; `autoplay.rs`.
- Tier 1 (`tests/http/`): `story_log.rs` (8.9), `races.rs` (8.7), `swipe_new.rs` (12.1), `retrigger.rs` (15.1), `autoplay/run.rs` (41.8), `screen.rs` (45.1–45.7), `screen_load.rs`, `games_pages.rs`.

A scenario whose text changes keeps its test. The test changes only what the new text says. A test that must change more says so in the Gate log.

**New and changed scenarios.** The next free numbers are 16.72, 30.25, 44.45, 45.17, 8.16 and 25.14 (checked: the highest numbers are 16.71, 30.24, 44.44, 45.16, 8.15 and 25.13; 44.44 is the highest of 44, so 44.45 is free). The new scenarios of 16 run from 16.72 to 16.78, and 45.18 and 45.19 are free too.

| Scenario | Spec | Tier | Stage | What | Evidence |
|---|---|---|---|---|---|
| 45.18 (new) | `docs/specs/screen.md` | 1 | 1 | One table: an invalid value, a gone record by POST, a gone record by GET opener, a state refusal (`WorldHasGames` through the delete of a World), a refusal and an engine failure answer the status of A2 and a body with no `Error:` prefix. | Red first: the prefix, and the two `400` openers (20.24 and 25.14 have their own rows). The `500` of `WorldHasGames` on the `IntoResponse` path cannot be red in an HTTP test, because no route reaches it. |
| 20.24 (changed) | `docs/specs/settings.md` | 1 | 1 | The connection Edit form of an unknown id answers `404`. | Red first. |
| 25.14 (new) | `docs/specs/worlds.md` | 1 | 1 | The World Edit form of an unknown key answers `404`. | Red first (no test exists today; checked by grep). |
| 45.17 (new) | `docs/specs/screen.md` | 1 | 2 | Each route of the router's Game-bound list answers a stale-Game `409` with "The active Game changed, so <its words>." and the engine's line. The test loops over `game_bound_routes_of_router` (`tests/http/screen.rs:83-98`) and reads the words from `REFUSAL_WORDS`. The busy half names its routes in the test: the Game-bound routes that the writer rule refuses while a Job holds the Game, and the three config saves while an Autoplay run holds the Game (`src/application/generation/gate.rs:501-506`). Each answers "The Game is busy, so <its words>." and "Still thinking...". The three config saves (`src/adapters/driving/http/builders/router.rs:122-133`) are not behind `refuse_stale_game`, which only `game_bound_routes` applies (`:27-63`), so they answer no stale-Game `409`. | Red first: today the body is "Error: The active Game changed". Mutation: delete one row of the table, expect the failure. |
| 45.19 (new) | `docs/specs/screen.md` | 1 | 2 | The first line of each outcome has the form of A4: invalid "<Words>: <message>.", failed "<Words>." with the engine's text in line 2. One route with an item (a row button that reads its name on the failure path, A24) and one without. One answer that no constructor built: `POST /games/abc/rename` (a `Path` rejection) answers `400` with the words of its route, and its own text is line 2 (A5). | Red first: the bodies hold no words today. |
| 45.3, 45.4 (changed) | `docs/specs/screen.md` | 1 | 2, 4 | Stage 2: the sentence is added. Stage 4: the `X-Refusal` line goes. `docs/specs/screen.md:52,63,196,201` and `tests/http/screen.rs:342,425` change. | The tests change with the text. |
| 44.45 (new) | `docs/specs/browser_screen.md` | 2 | 3, 4 | The sweep of A14: every `form` of the dialog, `#command-form`, `#autoplay-form` and every `.edit-actions` has one direct-child slot, and no other element holds a slot but the report container of `#screen-error`. | Red first: the slots of A20. Mutation: remove one form slot, and add one slot to a card; each fails. |
| 16.72 (new) | `docs/specs/browser_dashboard.md` | 2 | 3 | A refused or failed answer of a row button shows in `#screen-error`. The place has a Close control, and Close hides it and moves focus to `#main-content` (A27). | Red first: today the report shows in a slot of the row. |
| 16.73 (new) | `docs/specs/browser_dashboard.md` | 2 | 3 | One table: a failed or refused request of each control kind (row button, save-on-change select, dialog form `500` and `409`, preview form that stays open, Stop) reports once, in the notice, with the first line of the body. An invalid answer of the Action input, of the Autoplay cycles and of a dialog form reports in the slot of its form, and the typed text stays. | Red first: the client builds its own words until this stage. |
| 16.74 (new) | `docs/specs/browser_dashboard.md` | 2 | 3 | The clearing rules of A11: the next action clears a notice; a notice of an action survives the load that follows it; a notice of a failed screen load clears when a load works; a form report clears when its form sends again; a dialog opener clears a notice (A25); a tab change keeps the notice. | Red first: today a good load clears `#screen-error`. |
| 16.75 (new) | `docs/specs/browser_dashboard.md` | 2 | 3 | A `409` shows the first line of its body, and Details holds both lines. A `500` shows the server's first line with no "That action failed." lead, and Details holds the engine's text. | Red first: today the client ignores the `409` body and leads a `500` with its own words. |
| 16.76 (new) | `docs/specs/browser_dashboard.md` | 2 | 3 | No answer on a row button, a save-on-change select, a dialog form and a preview confirm raises the Unreachable banner, and no slot and no notice holds a report. Any later answer clears the banner. A preview confirm with no answer leaves the preview open with its text (A23). With a dialog open, no answer shows in the dialog status line, and the next answer clears it (A26). | Red first: today the slot of the control shows the text. |
| 16.77 (new) | `docs/specs/browser_dashboard.md` | 2 | 3 | A load error on the new-Options form does not end a check that is in flight. A load error on `#command-form` ends it. | The test sends the event with `htmx.trigger(elt, "htmx:onLoadError", …)`. That call dispatches the camelCase name and then the kebab-case name that `hx-on::on-load-error` listens to (`ce` in `assets/htmx.min.js`). A plain `dispatchEvent` of the camelCase name does not reach the attribute. Mutation: remove the attribute from `#command-form`, expect the positive half to fail. Restore the path test, expect the negative half to fail. |
| 16.19, 16.20, 16.29–16.32, 16.34, 16.35, 16.37, 16.38, 16.40, 16.42, 16.44, 16.54, 16.55, 16.57–16.59 (changed) | `docs/specs/browser_dashboard.md` | 2 | 3, 4 | The words come from the server, and the place follows O4: a `409` and a `500` show in the notice, an invalid dialog form in its slot, and no answer in the banner. 16.40 (engine stopped at a preview confirm) asserts the banner, and its line "Then the preview closes" (`docs/specs/browser_dashboard.md:364`) becomes "Then the preview stays open with the corrected text". 16.44 (a busy refusal at a preview confirm) asserts the notice, and its line "Then the preview closes" (`:411`) changes the same way (A23). 16.55 moves its report from the entry to the notice in stage 4. | The tests change with the text. |
| 16.71 (changed) | `docs/specs/browser_dashboard.md` | 2 | 3 | The refusal of the new-Options form shows in the notice with "the Action was not sent". | The test changes with the text (`tests/browser/stub/action_controls.rs:512` names the slot). |
| 28.5, 29.5, 35.6 (changed) | `docs/specs/browser_prompt_presets.md`, `browser_worlds.md`, `browser_llm_messages.md` | 2 | 3 | The refusal of a card, a row or a record opener shows in the notice, and focus stays where the scenario says. | The tests change with the text. |
| 42.5–42.7 (changed) | `docs/specs/browser_autoplay.md` | 2 | 3, 4 | "The story log's error slot" becomes the notice. 42.6 keeps the Autoplay form's slot only for an invalid cycle count; the busy refusal shows in the notice. | The tests change with the text. |
| 44.17, 44.18 (changed) | `docs/specs/browser_screen.md` | 2 | 3 | 44.17: a `500` of a dialog save shows in the notice and in the dialog status line, and the dialog keeps its form. 44.18: the error shows in the notice. | The tests change with the text. |
| 44.19, 44.31 (changed) | `docs/specs/browser_screen.md` | 2 | 3 | 44.19 becomes "A load keeps the notice of an action". 44.31 loses the slot that shows an error. | The keep rule goes (A20). |
| 30.4, 30.5, 30.10, 30.16, 30.18, 30.19 (changed) | `docs/specs/browser_story_log.md` | 2 | 4 | The failure shows in the notice. The revert of 30.4 and 30.16 stays. | The tests change with the text. |
| 30.25 (new) | `docs/specs/browser_story_log.md` | 2 | 4 | An invalid answer to Save (`400`) shows in the slot of the entry in edit, the draft and the lock stay, and Cancel clears the slot. | Red first: today a `400` reverts the edit. |
| 30.26 (new) | `docs/specs/browser_story_log.md` | 2 | 4 | A refused Save keeps the draft and the lock, and reports in the notice. | Red first: the place (`#story-log-error` today). The test asserts that the draft stays, so a `revertEdit` on a refusal fails it. |
| 8.16 (new) | `docs/specs/story_log.md` | 1 | 4 | The story-log fragment sends its requests through htmx, asks before a Delete, sends the textarea with a Save, and each entry renders Save, Cancel and one edit slot. | Red first: today the controls are `onclick` calls. The test asserts `hx-post`, `hx-confirm` and `hx-include`; the spec names ids and classes only (`tests/STRATEGY.md`, Markup). |
| 44.15 (changed) | `docs/specs/browser_screen.md` | 2 | 4 | The vehicle is an htmx request in place of `actionFetch` (`tests/browser/stub/screen.rs:150`). | The test changes with `actionFetch`. |
| 16.47, 16.51 (changed), 16.78 (new) | `docs/specs/browser_dashboard.md` | 2 | 5 | 16.51: closing the dialog focuses the opener by its id, also after a load rewrote the panel. 16.78: a load that removes a row above an opener keeps the id, and an opener that is gone leaves focus on `<body>`. | The P1 and P2 probes. Mutation: remove an id, expect the failure. |

The tests that name a deleted symbol change in stage 4: `tests/browser/stub/story_log_edit.rs:492` stubs `window.submitNewSwipe` to count clicks, and `tests/http/swipe_switch.rs:20` reads `switchSwipe(` from the HTML. They count requests and read `hx-post` instead. `tests/browser/stub/story_log_edit.rs:792` stubs `window.showEditForm`, which stays (A16), so it needs no change. `tests/browser/stub/dialog.rs:668` also calls `actionFetch` and changes with it.

**Other specs and tests that change.** Each changes at the stage that makes it false.

| Stage | File | Change |
|---|---|---|
| 1 | `src/adapters/driving/http/utils/response_tests.rs:61-78` | The three `render_error` tests call `error_fragment`. The `Error:` assert goes. |
| 1 | `src/adapters/driving/http/utils/fragment.rs:29,37` | The try-render wrapper calls `error_fragment`. A part failure text loses its `Error:` prefix. This is a visible change. |
| 1 | `tests/test_utils/stub_server.rs:1079, 1236, 1247, 1576, 1647, 1943, 1952`; `tests/test_utils/stub_action_layers.rs:99` | The stub calls `error_fragment` (made `pub`) in place of `render_error`. Stage 2 moves its error answers to the engine's constructors (A21). |
| 1 | `src/adapters/driving/http/error_tests.rs` | The displayable-engine test changes its status in place. No route reaches it. |
| 2 | Scenario 40.6 (`docs/specs/browser_settings.md:70-78`), `tests/browser/stub/settings.rs:244-280`, the stub flag `connection_saves_refused_as_busy` (`tests/test_utils/stub_server.rs:521,686,1325`, `tests/test_utils/stub_server/handles.rs:268`) | Deleted (O10). |
| 3 | `tests/browser/stub/action_controls.rs:512` | The slot selector of the new-Options form, with 16.71. |
| 3 | `tests/STRATEGY.md:222-240` (section "Convergence check") | "a showing inline error slot" goes from the list of what the comparison drops (A20). |
| 3 | Scenario 20.36 (`tests/http/settings.rs:210`) | The test asserts `data-refusal-what=` on the connection forms. The assert goes with the attribute. |
| 3 | Scenario 20.34 (`docs/specs/settings.md:52-58`), `docs/specs/worlds.md:97`, `docs/specs/prompt_presets.md:20-24`, `docs/specs/worlds.md:17`, `docs/specs/settings.md:32`; `tests/http/settings.rs`, `tests/http/worlds.rs`, `tests/http/prompt_presets.rs`, `tests/http/games_fragment.rs` | The markup scenarios say that a role row, a connection row, the text-check card and the World posture group hold an error slot. They hold none after A20. The tests that assert those slots go or change. |
| 3 | `tests/browser/stub/` `dashboard.rs`, `dialog.rs`, `failure_display.rs`, `form_failures.rs`, `layout.rs`, `llm_messages.rs`, `prompt_presets.rs`, `screen_keeps.rs`, `screen_morph.rs`, `settings.rs`, `worlds.rs`; `tests/test_utils/convergence.js:39-78` | They name a slot that goes, or the keep rule of a showing slot (A20). They change with the place. |
| 4 | `docs/specs/browser_autoplay.md` 42.5 (lines 73, 78, 82) and 42.7 (lines 97, 102); `tests/browser/stub/autoplay.rs:110,160,200,265` | "The story log's error slot" becomes the notice. The selector `#story-log-error` changes. |
| 4 | Scenario 16.55 (`docs/specs/browser_dashboard.md:539`); `tests/browser/stub/game_header.rs:149-153` | The report moves to the notice. |
| 4 | `tests/browser/stub/action_controls.rs:511` | `Retry` and `Retrigger` name `#story-log-error`. The selector moves to the notice. |
| 4 | `tests/browser/stub/story_log_edit.rs:180,559,566,845,908`; `tests/browser/stub/game_header.rs:141`; `tests/browser/stub/autoplay.rs:102` | The Save selector `.log-entry.narration .save-btn` becomes `[data-editing] .save-btn` (A15). |

### 7. Docs

- `docs/diataxis/reference/frontend/dashboard.md`: lines 16, 18, 23, 69, 73, 89, 97, 98 and 108 name rules that change. They say that a failure shows in the inline slot of its own control, that a load skips an inline error that shows, that `X-Refusal` names the kind of a `409`, that a refused save reports in the slot of its own form, and the markers `data-refusal-what`, `data-error-slot` and `data-unreachable`. The doc gets the status rule, the two places, the notice and its Close control, and the marker `data-source`.
- `docs/diataxis/explanation/dashboard_design.md` holds the design that this plan builds. The owner approved it. Stage 5 reads it against the code, sentence by sentence (section "Done when"). The owner amends DS2 under D1 (a).
- `docs/specs/`: the scenarios of section 6 and `docs/specs/screen.md` lines 52, 63, 196 and 201.
- `tests/STRATEGY.md`, section "Convergence check" (line 234): the words "a showing inline error slot" go (A20).
- `docs/diataxis/reference/frontend/http_routes.md` does not change: this plan adds no route.

Each stage fixes the docs that it makes false. Stage 5 reads both docs once, top to bottom.

## What already exists

- The change numbers, the revision and the two action layers: `stamp_action_answers` and `refuse_stale_game` (`src/adapters/driving/http/utils/action_layers.rs:19-53`). The route list `game_bound_routes` (`src/adapters/driving/http/builders/router.rs:27-63`). `tests/http/screen.rs:83-98` reads it from the router source, and the test of Scenario 45.4 compares it with its rows (`:399-412`).
- The Game header on every htmx non-GET request: `htmx:configRequest` (`assets/dashboard-screen.js:428`). `actionFetch` adds it a second time for `fetch` (`:435-443`); after stage 4 only the first remains.
- The end of every action request: `endRequest` forgets the stamp and asks (`assets/dashboard-screen.js:79-88`, S1). It already ends a request that is an htmx request, so an htmx story-log action needs no extra ask.
- The notice place `#screen-error`, its `renderInlineError` writer, `screenErrorShows` and the dialog status line. This plan widens their input and adds Close and the source.
- `hx-sync="this:drop"` on the two shell forms (`assets/index.html:123,148`).
- The stub server renders the real Askama templates for the Games, Worlds, Settings, Prompt Presets and story-log parts and for the dialog forms (`tests/test_utils/stub_server.rs:97-106`, `:935-980`). The sweep sees the real markup.
- The stub's busy refusal and stale-Game refusal call the engine's own `refused` and `render_error` (`tests/test_utils/stub_server.rs:1317`, `tests/test_utils/stub_action_layers.rs:99`). They already share code with the engine.
- A second `htmx:afterRequest` dispatch exists today, and nothing breaks on it: the four body listeners (`assets/dashboard-dialog.js:162`, `assets/dashboard-errors.js:218`, `assets/dashboard-saves.js:85`, `assets/dashboard-screen.js:87`) are idempotent (inferred). The new reporter stops on a done answer, and the preview detaches only after a done answer (A23), so it never writes twice.
- htmx dispatches `htmx:afterRequest` to the body listeners first, then `htmx:after-request` to the site handler (A18). A site handler that detaches its element hides every later event of that request from the body (O9).

## Blast radius

- **Server:** `src/adapters/driving/http/error.rs`; `utils/error.rs`, `utils/response.rs`, `utils/action_layers.rs`; the router (one layer, no new route); the handlers that build an error body (`worlds`, `settings`, `games`, `prompt_presets`, `chat_window`, `action/handlers/actions.rs:99`, `screen/handlers/screen.rs:66-82`, `layout/handlers/endpoints.rs:95`); the templates of A15, A16 and section 3 (the eight slots that go, the 13 opener ids, the story-log template); `action_controls.rs`; `utils/fragment.rs` (it calls `render_error`). A new file for the words table carries a DOC anchor and a summary line (`CODING_STANDARDS.md`).
- **Assets:** `index.html` (the notice place gets its markup, `#story-log-error` goes, `?v=` bumps); `dashboard-errors.js`, `dashboard-saves.js`, `dashboard-action-area.js`, `dashboard-story-log.js`, `dashboard-dialog.js`, `dashboard-screen.js` (`actionFetch` and `showsInlineError` go); `styles.css` (the Close control, the `.edit-actions` state rule, `.story-log-error` goes).
- **Specs:** section 6, with its table of other specs and tests that change. **Docs:** section 7.
- **Tests:** the stub server and `stub_action_layers.rs` (A21); `tests/http/screen.rs`; the stub checks of section 6 (about 15 files under `tests/browser/stub/`); `tests/test_utils/convergence.js`; `tests/infrastructure/guardrails/` (`PART_WRITE_EXCEPTIONS` and its machinery, `AFTER_REQUEST_FORMS` and its two checks, the `actionFetch` rule, with their detection tests in `screen_tests.rs`; a new guardrail for the words table); the legacy unit tests `src/adapters/driving/http/error_tests.rs` (the displayable-engine test changes status in place) and `src/adapters/driving/http/utils/response_tests.rs` (the three `render_error` tests); the Save selectors of the story-log tests (section 6).
- **Not touched:** the change numbers, the Job and the writer rule, the screen load, the dialog (apart from focus and the status source), the part routes (T2), the part renderers' error types (T3), and the policy of which errors show their text.

## Failure modes

| Failure | Effect | Answer |
|---|---|---|
| The words layer does not see the route | A `409` says the reason only | Stage 2 checks `MatchedPath` first, with a control run that adds the layer before the last `.route` (it must fail). The layer is added after every `.route` call (section 1). The fallback is the segment matcher (section 1). The guardrail fails for a `post` route with no row. |
| A new route has no row | The first line has no "so …" or no words | The guardrail of A5 fails the build. |
| htmx fires `afterRequest` twice | `clearUnreachable()` runs twice | It is idempotent. The second dispatch comes only when a site handler detached its element after a done answer (A23), and the reporter stops on a done answer. |
| A preview confirm gets a network error | The preview stays open with its text (A23) | O9: the reporter reads status 0 on `afterRequest`, which reaches the body before any site handler runs. Scenario 16.76 checks it. |
| An abort or a timeout reads as "unreachable" | The banner says "unreachable" for a request that the engine did not fail | O9 cost. The app has no abort or timeout source. Stage 3 check 1 (d) pins that. |
| A good screen load erases the report of an action | The player never sees the report | The source of A10 and A11. Scenario 16.74 checks it. |
| A form of the list has no slot | An invalid answer shows in the notice, away from the text | The sweep fails the build (A14). |
| A story-log request fails with no answer | The revert runs, and the banner says "unreachable" | Same as today for the revert (`.catch` of the `fetch` chain). |
| A Close click hides the control that holds focus | Focus would fall to `<body>` | A27 moves focus to `#main-content`. |
| No answer while a dialog is open | The banner is behind the modal | A26: the dialog status line repeats the Unreachable text. Scenario 16.76 checks it. |
| A control names a path with no row | Its failure has no words | The guardrail of A5 reads the templates and `assets/index.html`. |
| The opener is gone when the dialog closes | Focus stays on `<body>` | A17. The server renders no id for a row that no longer exists. |
| `showModal()` focus return and the explicit call differ | None: the call wins | P1 checks it. |
| A Save is clicked while its request is in flight | A second request | `hx-sync="this:drop"` drops it (A15). |

## Stages

### How a stage runs

1. A subagent builds the stage (the role is in the table below). It writes the failing tests of each new or changed scenario first, cleans its own comments, and runs `python build.py --no-browser` once before it reports.
2. The gate of the stage runs. If the gate passes, the stage is committed on `wf/change-driven-dashboard-refresh` and the next stage starts. The owner is not asked between stages. The owner's approval of this plan covers the commit at each gate that passes.
3. The gate adds one row to the Gate log (end of this section): the commit, the checks, the dashboard JS line count (O3), and each difference from the plan with its reason.
4. If a stop condition holds, the work stops, and the owner decides.
5. A fix (from a review or a failing test) that adds a counter, a flag, a list, a listener or a CSS state rule edits the plan body in the same pass, with its reason. If the plan's design does not explain why it is needed, it goes to the owner under stop condition 4. This plan names these additions: the answer outcome and the words table (A4, A5), the `data-source` of the notice, its Close control, its click listener and the `data-report` container (A10, A27), the `htmx:beforeRequest` clear listener (A11), the events `notice:changed` and `banner:changed` with the dialog listener of each (A12, A26), the `.edit-actions` CSS state rule (A15), the opener ids (section 5), and the test that pins the lack of an abort or timeout source (stage 3 check 1 (d)).

### Gates

| Gate | Steps |
|---|---|
| **Light** | 1. The exit checks of the stage. 2. `python build.py` is green. 3. Commit. |
| **Review** | 1. One `reviewer` checks the spec, the standards, the comments and the docs of the stage, from one bundle (`scripts/prepare_review_bundle.py --uncommitted`). 2. Each finding is verified; the fixes go back to the implementer's resumed session. 3. The Light steps. |
| **Milestone** | 1. The Review steps, up to the green full gate. 2. A UI spot check with `/chronicler-ui-investigator` on a probe server (port 3001, `--target-dir target/ui-investigator`, real connections): each surface in the milestone list, with a shot in `tmp/ui-gates/<milestone>/` that the orchestrator has looked at. 3. Commit. 4. A short report to the owner. The work goes on unless a stop condition holds. |
| **Final** | After stage 5. See Final gate. |

| Stage | Builder | Gate | Exit checks |
|---|---|---|---|
| 1. Status and body on the server | `strong-implementer` | Review | 45.18, 20.24 and 25.14 pass, each red first. `rg 'render_error' src tests` finds nothing. `rg '"Error: ' src` finds no producer of the prefix, and `rg 'action_error_response' src` finds nothing (the three classifiers are gone, and `answer_for_error` takes no prefix). One classifier, `answer_for_error`, replaces the three. The browser tier is green with no client edit. |
| 2. The words | `strong-implementer` | Review | The words layer finds each row of the table. 45.17 and 45.19 pass, each red first. The guardrail of A5 passes, and fails when a row is removed. Scenario 40.6 and its stub flag are gone (O10). The stub mounts the engine's layer, and its error answers get their words from it. The guardrail also fails for an `hx-get` or `hx-post` path with no row. The browser tier is green with no client edit. |
| 3. The one reader and the places | `strong-implementer` | **Milestone A** | `rg -g '!assets/htmx.min.js' 'data-refusal-what\|requestPath\|answeredRefused\|requestSucceeded\|INLINE_ERROR_CONTAINERS\|GENERATED_SLOT\|generated-\|PART_WRITE_EXCEPTIONS\|showsInlineError\|data-unreachable\|reportConfirmFailure\|reportSlotFailure' src assets tests docs` finds only this plan's history (the vendored `assets/htmx.min.js` holds `requestPath` twice, so the grep skips it). The `AFTER_REQUEST_FORMS` list and checks are gone. 44.45 passes on all six tabs and on every dialog opener (the `story-log` slot is allowed until stage 4). 16.72–16.77 pass (16.72–16.76 red first; 16.77 by mutation). 16.76 passes with `reportConfirmFailure` deleted (O9). |
| 4. The story log | `strong-implementer` | **Milestone B** | `rg 'actionFetch\|story-log-error\|storyLogErrorSlot\|showStoryLogError\|reportedRefusal\|X-Refusal\|x-refusal\|REFUSAL_REPORTS\|refusalOf' src assets tests docs` finds only this plan's history. 30.25, 30.26 and 8.16 pass. 44.45 no longer allows the `story-log` slot. |
| 5. Focus of the dialog, docs | `implementer` | Light, then Final | The P1 and P2 probes, and their control run, are in the Gate log (or the reason that C6 dropped out). `rg 'requestOf\|openerNow\|dialogOpenerRequest' assets tests` finds nothing if C6 stayed. `dashboard_design.md` matches the code (Done when). `validate-docs` passes. |

**Milestone A (reports), surfaces:**
- a refused Rename: the notice under the header shows the sentence of the server with no `Error:` and no "That action failed." lead (U4), and its Close control hides it and moves focus to the main content;
- a Send during a Job: "The Game is busy, so the Action was not sent." in the notice, and the typed text stays;
- a Send, a Stop and a Reset from a tab whose Game changed (two browser tabs): each reports in the notice;
- the new-Options control during a Job: the notice reads "the Action was not sent";
- a row button that the engine fails (Delete of a Game, for example): the sentence names the item;
- an invalid Rename (empty name): the report is inside the dialog form, and the typed text stays;
- a dialog save that the engine fails (stub or a stopped storage): the dialog stays open with its draft, the notice shows the report, and the dialog status line repeats the summary;
- a notice, then a new action: the notice clears; a notice, then a dialog opener: the notice clears; a notice, then a load: the notice stays;
- a failed screen load, then a good load: the notice clears;
- the engine stops and starts: the Unreachable banner shows and clears, and no slot and no notice shows the text;
- a preview confirm with the engine stopped: the banner shows, and the preview stays open with the corrected text (A23);
- a preview confirm that the engine refuses: the notice shows the report, and the preview stays open with the corrected text (A23);
- a dialog open and the engine stopped: the status line shows the Unreachable text, and the next answer clears it (A26).

**Milestone B (story log), surfaces:**
- a Save during a Job: the notice shows the report, and the draft and the lock stay;
- an invalid Save: the report is in the edit slot of the entry, and Cancel clears it;
- a Retry, a Retrigger, a swipe switch and a Delete that the engine refuses or fails: each reports in the notice;
- the other Edit controls and the swipe controls stay locked during an edit, and unlock after Save and after Cancel;
- the log still follows its bottom, and the entry layout holds with the hidden edit slot.

### Stop conditions

The work stops, and the owner decides, when:

1. a "checks first" step fails: stage 3 item 1 (a). A failure of the stage 2 `MatchedPath` check is not a stop (it has a fallback). A failure of the C6 probe is not a stop: C6 drops out;
2. a stage finds a mismatch whose fix needs a design choice that this plan does not make;
3. a stage must change a behaviour that a spec states, and this plan does not name that change;
4. a review finding or a UI spot-check finding needs a design change, not a fix;
5. the full gate fails, and the fix needs a design choice. A failing test whose fix is clear is fixed, also when it seems unrelated (`AGENTS.md`);
6. a stage needs a step that this plan puts in a later stage, or the order of the stages no longer works.

### Stage 1 — Status and body on the server

The client does not change in this stage. It ignores the body of a `409`, and no script reads the `Error:` prefix.

1. The status rule (A2): the connection Edit form and the World Edit form answer `404`; `WorldHasGames` answers `400` on every path.
2. `error_fragment` is the one builder, and it becomes `pub`. Delete `render_error`, and change its callers in `utils/fragment.rs:29,37`, the other callers of section 1 and the stub callers that section 1 names (`tests/test_utils/stub_server.rs:1079, 1236, 1247, 1576, 1647, 1943, 1952`; `tests/test_utils/stub_action_layers.rs:99`).
3. Write `answer_for_error`. Replace the three classifiers and their callers.
4. Specs and tests: section 6, stage 1 rows, and the table of other specs and tests that change. `src/adapters/driving/http/error_tests.rs`: the displayable-engine test changes its status in place.

### Stage 2 — The words

The client does not change in this stage. It builds its own words, and it ignores the body of a `409`.

1. Checks first (tier 1): the words layer finds the matched route of `/history/:id` and `/message/:id/swipe/:index` through `MatchedPath`, for a refusal from `refuse_stale_game` and for a busy refusal from a handler. A control run adds the layer before the last `.route` call, and it must fail to find a route. If the check fails, use the fallback of section 1.
2. The answer outcome, `refused(kind)`, `REFUSAL_WORDS` (one row for each of the 35 `post` routes and the ten openers, A5) and `describe_answers`. Keep `X-Refusal` (the layer sets it until stage 4). Add the layer after every `.route` call: `Router::layer` wraps only the routes that exist at that call. In the engine it goes next to `stamp_action_answers` (`router.rs:248`). In the stub it goes after `action_answers_middleware` (`stub_server.rs:981-984`).
3. Make `answer_for_error`, `refused` and `action_not_found_response` produce an answer with an outcome and the engine's text, and let the layer write the body (A3, A4). Pass the item name where the handler holds it. In the five handlers that hold only an id (A24), read the display name on the failure path only.
4. The guardrail of A5: every `post` route of the router source has a row, and every `hx-get` or `hx-post` path of a template source and of `assets/index.html` matches a row.
5. The stub: mount the engine's layer (A21). Make the engine's constructors `pub`, and build the stub's error answers (`client_failure`, `client_refusal`, `tests/test_utils/stub_server.rs:1930-1955`) through them. Replace the stale-Game refusal (`tests/test_utils/stub_action_layers.rs:99`) and the busy refusal (`tests/test_utils/stub_server.rs:1317-1320`) with `refused(kind)`. Keep `GAME_BOUND_ROUTES`, `is_game_bound` and the comparison of Scenario 45.4 (`tests/http/screen.rs:399-412`).
6. Delete Scenario 40.6 and the stub flag `connection_saves_refused_as_busy` (O10).
7. Specs and tests: section 6, stage 2 rows.

### Stage 3 — The one reader and the places

1. Checks first (stub tier, on the vendored htmx): (a) `hx-on::on-load-error` runs for a swap that throws; (b) `afterRequest` fires with status 0 for a network error, an abort and a timeout, and `sendError` fires for a network error only; (c) a `sendError` on an element that the site handler detached does not reach the body, and the `afterRequest` of the same network error reaches it first (O9); (d) the app has no abort or timeout source: no `htmx:abort`, no `hx-sync` with `replace` or `abort`, no `htmx.config.timeout` (known by grep at this HEAD: none), and a test pins it (O9). If (a) fails, stop. (b), (c) and (d) set the status-0 rule.
2. The shell: the notice place gets its report container (with `data-error-slot="screen"`) and its Close control (A10), with CSS and the click listener (A27). Rename the event `screen:error` to `notice:changed` in the two dispatchers of `dashboard-screen.js` and the listener of `dashboard-dialog.js`. Fire `banner:changed` from `refreshFailureBanner`, and let `renderDialogStatus` show its text (A12, A26).
3. Write `actionAnswer`, `reportAnswer`, `showNotice`, `clearNotice` and `formSlotOf`. Add the body listener of `htmx:beforeRequest` that clears for every htmx request (A11, A25). Make `clearScreenError` check the source. Run Scenario 16.76 against the new reporter before the next deletion (O9).
4. Delete `answeredRefused`, `reportRefusalOfButton`, `requestSucceeded`, `reportSlotFailure`, `refusalSummary`, `commandFormSlot`, `reportConfirmFailure`, the generated-slot code, `INLINE_ERROR_CONTAINERS`, `clearUnreachableSlots`, `data-unreachable`, and the `htmx:responseError` and `htmx:sendError` listeners. These stay until stage 4, because the story-log chains use them: `REFUSAL_REPORTS`, `reportRefusal`, `refusalOf`, `refusalOfResponse`, `throwServerDetail`, `showStoryLogError`. `plainResponseText` stays: the reader and `dashboard-screen.js` use it.
5. Cut the four action-area handlers and the dialog handler to their success work (A13). The preview handler closes the preview only after a done answer (A23). Rename `onNewOptionsAfterRequest` to `endPendingAfterRequest`. Add the T8 attributes and delete the two path tests.
6. `settleSave` stops reporting. `isSavingControl` loses its clause (A19). Delete `data-refusal-what` from the seven templates, and the `AFTER_REQUEST_FORMS` list, its two checks and their detection tests.
7. Delete the seven slots of section 3 and `showsInlineError` (A20), and change `convergence.js` and the words of `tests/STRATEGY.md` (section "Convergence check"). Delete the `PART_WRITE_EXCEPTIONS` row, `PartWriteException`, `without_exempt_functions`, `check_part_write_exceptions_are_used` and their detection tests (`tests/infrastructure/guardrails/mod.rs:538-542`, `tests/infrastructure/guardrails/screen.rs:395-455`, `tests/infrastructure/guardrails/screen_tests.rs:259-322`).
8. The stub must render each form of section 3. Write the sweep (44.45), allowing the `story-log` slot.
9. Specs and tests: section 6, stage 3 rows. Docs: `dashboard.md` lines 16, 18, 23, 69, 73, 89, 97, 98 and 108, and `tests/STRATEGY.md`.

### Stage 4 — The story log

1. Templates (A15, A16): the `hx-post` controls, `.edit-actions` with its slot, and the CSS state rule.
2. Script: `showEditForm` builds the textarea only. `onEditSaved`, the swipe and Delete after-request handlers, and `endPendingAfterRequest` for Retry and Retrigger. Delete what section 4 lists. The keydown of the textarea clicks the Save of the entry in edit (A15). Delete `actionFetch` and the Game-header write in it. Change the guardrail "dashboard scripts send actions through actionFetch" to "no script calls `fetch(` except the two reads of `dashboard-screen.js`" (`tests/infrastructure/guardrails/screen.rs:483-500`, `tests/infrastructure/guardrails/mod.rs:614`).
3. Delete `X-Refusal` and `REFUSAL_HEADER` in the engine and the stub, `#story-log-error` from `assets/index.html` and its CSS. The sweep stops allowing the `story-log` slot.
4. Specs and tests: section 6, stage 4 rows. Docs: `dashboard.md` lines 16, 71 and 108.

### Stage 5 — Focus of the dialog, docs

1. Probes first (section 5): P1 and P2, and the control run. If P1 or P2 fails, C6 drops out: skip steps 2 and 3, and write the reason in the Gate log.
2. Add the 13 opener ids.
3. Record `dialogOpenerId`, focus it on close, and delete the code that section 5 lists. Scenario 16.51 changes; 16.78 is new.
4. Docs: read `dashboard.md` and `dashboard_design.md` top to bottom against the code, as the Done-when item says. Run `python build.py validate-docs`.

### Final gate

The review scope is the branch from the commit that adds this plan to its last stage commit.

1. `/chronicler-after-plan-workflow-plus-review`, steps 1–11, with that fixed point. The verify step lists each client and server mechanism that this plan does not name, as the drift audit did. It also runs the three greps of the exit checks.
2. The UI pass with `/chronicler-ui-investigator` on a probe server with real connections. Each item needs a shot in `tmp/ui-gates/action-answers-final/` that the orchestrator has looked at:
   1. every item of Milestone A and Milestone B, again, on the final build;
   2. each of the 14 openers (the 13 with new ids, and the LLM record opener): open with the mouse, close with Cancel, and check where focus lands; then with the keyboard and Escape;
   3. a row whose opener is gone when the dialog closes (delete the row from a second tab): where focus lands;
   4. the 4xx and 5xx texts of one form each, and the Details content;
   5. the notice place on each of the six tabs, with Close.
3. The four reviews of `/chronicler-after-plan-workflow-plus-review`, in parallel with the UI pass.
4. One report to the owner. No fix pass and no merge to `main` start before the owner decides.

### Gate log

| Stage | Commit | Gate | Dashboard JS lines | Checks | Differences from the plan |
|---|---|---|---|---|---|
| Baseline | `82531b0b` | none | 1,995 (`wc -l assets/dashboard-*.js`, checked 2026-10-27) | none | none |
| 1 | | Review | | | |
| 2 | | Review | | | |
| 3 | | **Milestone A** | | | |
| 4 | | **Milestone B** | | | |
| 5 | | Light, then Final | | | |

Expected line effect (a guess, not checked): stage 3 about −60 (the reporters, the container list, the generated slots and the keep rule go, and the reader and the notice come). Stage 4 about −80 (four `fetch` chains and `actionFetch`) and +15 (the handlers). Stage 5 about −30 (opener tracking). The sum is about −155. A stage that adds lines says why.

## Done when

- A refusal answers `409` with a sentence of the form "<reason>, so <words>." and the engine's line. Every other failed action answers a first line from the words table (A4). No code builds a sentence of an action answer in the browser, and `X-Refusal`, `REFUSAL_REPORTS` and `data-refusal-what` are gone.
- Every failed action answers one body shape with no `Error:` prefix, and one status for each kind (A2). The sentences use the terms of `CONTEXT.md` (A28).
- One function decodes an action answer, and one function reports it. `isSavingControl` has no `hx-on::after-request` clause, and the `AFTER_REQUEST_FORMS` list is gone.
- An invalid answer to a form of the DS3 list shows in that form. Every other report shows in `#screen-error`, which has a Close control. No answer shows in the Unreachable banner, and in the dialog status line while a dialog is open. The preview closes only after a done answer. The clearing rules of DS4 hold.
- No script makes an error slot. The sweep finds a slot for every form of the DS3 list, and for no other element but the report container of `#screen-error`.
- No script sends an action with `fetch`. The story-log controls are htmx controls.
- No script reads `requestPath`.
- Dialog focus returns by the id of the opener, or the Gate log says why C6 dropped out.
- `docs/diataxis/explanation/dashboard_design.md` matches the code. Stage 5 reads it against the code, and each sentence is true. The design sections are ahead of the code until this plan is built, and they are committed together with the approved plan.
- Every scenario of section 6 passes at its tier, and the full gate is green.
- Every stage has a row in the Gate log with its line count, and the final gate's report is with the owner.

## Risks

- **The preview textarea (O4).** The text-check preview has a textarea for the corrected text. DS3 names the dialog forms, the Action input, the Autoplay cycles and the message edit, and not the preview. So an invalid answer to a preview confirm shows in the notice, away from the textarea. The preview stays open with its text (A23), so the player can read the report and correct the text. The route accepts text that the check already took, so the case is rare (inferred, not checked). If the owner wants the preview in the list, it needs a slot in the preview and one more row in A14.
- **The name read of a row button (A24).** A row button handler reads the display name only on a failed answer. If the read also fails, the sentence has no name, and it reads away from its row without one. A Message action has no item name. The read is local (inferred, not measured).
- **A new route with no row.** The guardrail of A5 fails for a `post` route with no row, and for an `hx-get` or `hx-post` path of a template with no row. A route that no template names is covered only by review.
- **One sentence for the new-Options form and the Action form (A5).** A player who clicks the new-Options control reads "the Action was not sent". `/options` is an Action in `CONTEXT.md`, so the sentence is true. It does not say which Action.
- **No answer while a dialog is open (A26).** The dialog status line repeats the Unreachable text. The status line is `role="status"`, and the banner is behind the modal. Whether assistive technology reads the status line is inferred, not checked.
- **Close and focus (A27).** Focus moves to `#main-content`. The next Tab goes into the tab content (inferred, not checked).
- **An abort or a timeout reads as "unreachable" (O9).** The reporter reads status 0 on `afterRequest` as "no answer". A later `hx-sync` with `replace` or `abort`, or a request timeout, would show the banner for a request that the engine did not fail. Stage 3 check 1 (d) pins that the app has no such source.
- **The latest report wins.** The notice holds one report. A refusal that arrives while a failed screen load shows replaces it. The next good load then does not clear the refusal (source `action`). The other direction also holds: an action report, then a failed screen load replaces it, and the next good load clears it. The player can lose the action report before reading it. DS4 does not decide this case, and the plan keeps the rule.
- **Retry and Retrigger have no `hx-sync` (A16).** A double click sends a second request, and the engine answers a busy `409`. The same holds today with `fetch`, so this plan does not change it.
- **Explicit focus (A17).** In a browser that does not focus a button on click, the id still finds the opener, so the call works. Chromium is the only browser the stub tier runs. This is inferred, not checked.
- **CSS state rule `.edit-actions` (A15).** It shows two buttons by `[data-editing]`. A load that skips the entry keeps the markup, so the state holds. The convergence check compares the part with a fresh render, which has the same hidden buttons.
- **Sweep reach (A14).** It sees the forms that the stub renders. A form that the stub lacks is invisible to it. Stage 3 checks the stub data for each form in section 3.
- **`stub_server.rs` size.** It has 1,955 lines (checked). The guardrail counts 2,000 non-blank lines. Stage 2 deletes the flag `connection_saves_refused_as_busy` and changes the error answers in place; a stage that nears the limit splits the file first.

## Not in scope

- The two unused part routes and the part routes in general (thermo T2). Not changed here.
- The error types of the part renderers (T3), the whole-file wording of the part-write exemption (T4, stale: the row now names one function, and stage 3 deletes it), the size of two test files (T5), the three status reads of one screen load (T9), the wrappers in `parts.rs` (T10), the nine dialog wrappers (T11) and the two preset forms (T12).
- One shared list of the Game-bound routes for the engine and the stub (thermo T7, outside O1). The stub keeps its own list, and Scenario 45.4 compares it with the router (A21).
- The policy of which errors show their text. A non-displayable `ApplicationError::Engine` keeps its generic text on the `IntoResponse` path.
- The three sentences that the client writes about the page (O11).
- A story-log Save that fails with a `500`, or gets no answer, keeps no draft (Scenarios 30.4 and 30.16 stay). Keeping the draft on any failure would match the dialogs, but no design statement asks for it and it changes a stated behaviour.
- Per-tab Games, Server-Sent Events, and collapsing the per-kind change numbers (the visible-tab plan's list stays).
