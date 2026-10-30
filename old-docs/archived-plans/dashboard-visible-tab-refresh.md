# Plan: The dashboard loads the visible tab

**Date:** 2026-10-26
**Status:** Done (2026-10-27). Stages 1–8 and 7b committed. The Final gate ran after the last stage.
**Goal:** The page shows one screen: the header, the failure banner and the visible tab. One request loads the whole screen whenever anything changes. A tab's content exists only while the tab is open. A form that waits for a Save is in a dialog, outside the screen, so no load ever meets a draft. The server renders every control state. The browser keeps only view state.

This plan replaces the browser half of `old-docs/archived-plans/change-driven-dashboard-refresh.md` (the "base plan"). It also replaces the edits draft ("Edits outside the live regions", never committed, deleted 2026-10-26), and takes the dialog design from it. The server half of the base plan stays: the change numbers, `GET /changes`, actions that answer no screen content, and morph. Branch: `wf/change-driven-dashboard-refresh`.

## Origin

The base plan made each screen part a live region. Each region names the kinds of data it shows (`data-live`), keeps its own record of numbers, and loads on its own. Each edit form sat inside a region, so each draft needed protection from the loads. The protection grew in every stage: the hold rule, the dirty check, the end-of-edit rules, five draft markers, editor slots, and the discard on close. The edits draft removed the draft protection with a dialog. It did not question the rest.

The owner asked for a clean-room review of the UI. Two walled models designed the UI from a brief of requirements only, with no access to the repo (`tmp/ui-clean-room/`: `brief.md`, `design-opus.md`, `design-sonnet.md`, `comparison.md`). Both are Anthropic models, because the second family was not available, so their agreement is weaker evidence than two families would give. The comparison found:

- **The server half holds.** Both designs keep server rendering, actions that answer no screen content, and "a change marks the screen old, then the screen fetches and morphs".
- **History, not requirements:** the browser table of kinds and regions, the busy refusal answered as `200`, and the timer-control test API `window.chroniclerLive`.
- **Statement 5 of the edits draft is forced** (a load never writes into a draft). **The single dialog is free.** It stays, because it needs no dirty check.
- **Six gaps:** one snapshot for each update; "Saved" only after the screen shows the save; the Game id on Game-bound actions; `disabled` rendered only by the server; "unreachable" only on a network error; and a test that the screen equals a fresh server render.

The code facts come from `tmp/clean-room-plan/inventory.md` and `tmp/statement5/inventory.md` (both git-ignored), and from reads of the code.

## Decisions

### Owner decisions

| # | Decision |
|---|---|
| O1 | Every form that waits for a Save opens in one transient modal `<dialog>` in the shell. Closing the dialog discards the form. |
| O2 | The edit of a story-log message stays inline. The player can open it and type at any time, also while a Job runs. |
| O3 | A save of a message edit while the Game is busy stays refused. Save shows as disabled while the Game is busy, with "Wait for the narration to finish", and the draft stays. |
| O4 | Option B of the comparison: one load of the visible screen. The browser has no table of kinds and regions. |
| O5 | The list rule: a list sends short rows, the full item loads on open, and a list with no natural limit gets pages. |
| O6 | Clear on leave, load on open, for every tab, the Game tab included. Leaving the Game tab loses the story's scroll position and an open message edit. The Action input keeps its text, because it is in the shell. |
| O7 | Plan review D1, option B: the browser asks for the parts of the visible tab by name (`GET /screen?parts=…`). The shell is the one source of the layout, and the server knows renderers, not tabs. |
| O8 | Plan review D2, option A: when one part fails to render, the whole screen answers `500`, `#screen-error` shows the error, and every part keeps its content. |
| O9 | Drift audit D1 (2026-10-27): a stage 7b "simplify" runs before stage 8. It makes S1, S2, S4 and S5 (Drift audit decisions), and brings the plan body in line with the code. |
| O10 | Drift audit D2 (2026-10-27), proposal C3: the page does not forget its Game after its own create, switch or Reset. A send before the next load gets the `409` of P14, and the typed text stays. Scenarios 16.56 and 16.57 change (S3). |
| O11 | Stage 8 decisions (2026-10-27). (1) The page number travels as `<html data-games-page>`, set by a click on a Previous or Next control (it carries the page it goes to as `data-page`), sent by `loadScreen` as `games_page=N` on `GET /screen`, and deleted by the tab change. (2) The saved-Games query excludes the active Game in SQL (`WHERE id IS NOT ?`, `ORDER BY updated_at DESC, id DESC`, `LIMIT 50 OFFSET n`), so a page always holds 50 saved Games. (3) The server clamps a page past the end to the last page, and renders Previous and Next as `disabled` at the ends. |

### Plan decisions

The owner accepted P1–P22 as written (2026-10-26), with P12 changed by O7. P1–P9 come from the edits draft.

| # | Decision | Reason |
|---|---|---|
| P1 | A screen load skips an element that holds unsaved input: the story-log entry in edit (`data-editing`), and a control whose request is in flight. The page counts requests itself, from `htmx:beforeRequest` and `htmx:afterRequest`, because the htmx class `htmx-request` can stay on an element after a load swapped it (found in stage 2). A control whose request is in flight includes a dialog opener during its `GET`: the dialog reads the opener's request when its answer swaps (stage 3, U1), so a load must not rewrite the opener meanwhile. The end of a `GET` does not forget the stamp (S1 forgets it for an action), so a skipped opener can differ from the server render until the next revision move. The player cannot see this: each opener renders the same markup for as long as its row exists. | The skip changes only that element. The rest of the screen stays current. The kept inline error slot already uses the same Idiomorph callbacks. A load records the stamp of what it rendered, also when it skipped an element. The end of every action request forgets the stamp and asks (S1), so the next load gives the skipped control the server value. A load that skipped the entry in edit counts as current while the edit is open; the end of the edit (save or Cancel) forgets the stamp. (History: stage 2 found that a load during a save left the form stale; stages 2–7 kept the old stamp after a skip and made the asks wait for the request; stage 7b replaced that with S1.) |
| P2 | The World posture selects move with the World editor into the dialog, and keep their save on change. | No load reaches the dialog. |
| P3 | The preset card View opens in the dialog, the same as Edit. | Without a hold, the next load would put the plain card back. |
| P4 | The Test button of a saved connection opens the dialog with the result. | The result is the answer of an action, not stored data. In a part, the next load erases it (review finding H3). |
| P5 | The marker `data-editing` replaces `data-work` on the story-log entry. `data-work` is deleted. | `data-work` means "held" today. |
| P6 | The Edit button of a message is disabled (server-rendered) while an Autoplay run holds the Game, and enabled during a plain Job. | Autoplay makes the Game view-only. During a Job, O2 allows the edit. |
| P7 | The dialog closes on Cancel, on Escape, and when the screen shows a save that went through (P13). A click on the backdrop does not close it. Escape first closes an open error detail, and only an Escape with no open detail closes the dialog, so a draft is not lost while the player reads its error. | A click outside by mistake must not discard typed text. |
| P8 | Two checks enforce "no draft in the screen". A guardrail: a source literal that holds a draft control must itself carry `data-dialog-form` (a dialog form) or `data-shell-form` (a form in the shell, such as the text-check preview). The only permanent allow-list rows are field-builder files whose literals hold no `<form`; the sweep checks where their fields render. A screen sweep: the parts of every tab contain no draft control. | The guardrail catches a new form at build time. The sweep sees the HTML that the server really renders. |
| P9 | The lock of the other entries' Edit buttons during an edit is an attribute on the story-log root (`data-editing-entry`), which CSS and one click guard read. After a saved edit, the lock stays until the screen shows the saved entry, so a second editor cannot open on stale text. | A load morphs only the children of the root, so the lock survives every load. No script sets `disabled`. |
| P10 | The page keeps asking `GET /changes` every 1 s. The answer gets one new field, `revision`. When the page's stamp differs from `server:revision`, the page loads `GET /screen` for the parts of the visible tab (P12). | The ask loop, the server id and the unreachable handling exist and are tested. One extra request for each change costs little on a local server. Rejected: `GET /screen` answering 204 when nothing changed, which leaves `GET /changes` used only by tests. |
| P11 | `revision` is one engine-wide number. `Numbers::raise_game` and `Numbers::raise_engine` raise it (`changes.rs:28, 38`). Every raise passes through one of them (known). A screen load reads the revision first and renders after, so the screen holds at least that revision. | One place to raise, no new raise path. The per-kind numbers stay as the tested record of which kind each write raises. |
| P12 | `GET /screen?parts=<names>` answers one `<template data-part="…">` for each part it names, in that order, with the headers `X-Revision` and `X-Game`. The browser names the parts: the `data-part` elements of the shell and of the visible tab (O7). The server holds one renderer for each part name, in one list (`SCREEN_PARTS`), and answers `400` for an unknown, repeated or missing name. One script morphs each template into the element with the same `data-part`, then calls `htmx.process` on it. | One request, one stamp, one writer of the screen. The layout has one source, the shell. A part with no renderer fails at once, not only in a rare state. |
| P13 | An action answers `2xx` with `X-Revision: <server>:<revision>` (the revision after its commit), `409` for a busy refusal (the text stays "Still thinking..."), `400` for an invalid value and `500` for a failure. The page closes a dialog, clears the Action input or shows "Saved" only when its stamp has the same server id and at least that revision. The Action input clears only if it still holds the text that was sent. While the dialog shows "Saving…", a second submit of its form is not sent (`data-saving`), and a late close closes only the dialog session that saved (stage 6 review M1, M2). Both `409` refusals carry `X-Refusal: busy` or `X-Refusal: game-changed`, and the page builds the message of the form's slot from it (stage 5). A `409` to a button outside every container, such as Stop, reports in the command form's slot (Scenario 16.58). | The confirmation and the screen agree. The client finds a refusal by its status, not by a class string. The player can type the next action before the screen shows the last one, and that text must not be lost. |
| P14 | Every non-GET request from the page carries `X-Chronicler-Game` with the Game of the screen. A route layer on the 11 Game-bound POST routes (inventory F) refuses a request whose header names another Game, with `409` and "The active Game changed". The check is not in `CurrentGame`: axum turns a rejected `Option<CurrentGame>` into `None` (known: `axum-core-0.4.5/src/extract/mod.rs:103`), and `reset_busy_handler` takes `Option<CurrentGame>`, so a stale Stop would act on no Game and still answer `2xx`. A request without the header acts on the active Game, as today (tier-1 tests, curl). The page takes its Game from `X-Game` of each screen load. It does not forget the Game after its own create, switch or Reset (O10): a send before the next load gets the `409`, and the text stays. | A tab cannot send to a Game that another tab switched to a moment before. This also prepares per-tab Games (ticket 01). |
| P15 | "Unreachable" means a network error only. A non-2xx answer of `GET /changes` or `GET /screen` shows in one shell slot, `#screen-error`, and the screen keeps its content. `dashboard-errors.js` makes its slot in the nearest container of `INLINE_ERROR_CONTAINERS` that has none: a part-level opener, a row that the server renders with no slot (a saved Game row), or a form in a part (the Options regen form). It removes that slot when the slot clears, so the part again equals a fresh render. | The engine answered, so it is reachable (Scenario 16.25). The `HX-Reswap: none` answers of the four panels go away. `INLINE_ERROR_CONTAINERS` stays, keyed on `[data-part]` and excluding `#send-control` and `#autoplay-controls`: a failed GET of an opener at part level has no slot of its own, so `dashboard-errors.js` makes one in the part (section 3, Open). |
| P16 | Tests wait for `<html data-revision>` to equal the server's `server:revision`, and for `data-screen-busy` to be absent. The ask interval comes from `<html data-poll-ms>`. The stub server's shell sets a short interval. `window.chroniclerLive` is deleted. | A passive stamp in place of a test API in production. |
| P17 | Send, Autoplay and Stop move into two server-rendered parts that render `disabled` from the busy state: `#send-control` (Send) and `#autoplay-controls` (Autoplay and Stop). The two parts keep the DOM order equal to the visual order: Action input, `#send-control`, cycles field, `#autoplay-controls`. The Action input and the cycles field stay in the shell. The owner chose two parts over one part with CSS `order` after the stage 7 review (M1: Tab and reading order). `hx-disabled-elt` names only the shell fields, `#command-input` and `#autoplay-cycles`. Send and Autoplay show the in-flight state of their form through CSS on its `htmx-request` class (S2). htmx clears `disabled` on load, error, abort and timeout, but not after a swap that throws (stage 7 review L5). The command form swaps into `#action-preview`. After such a throw, htmx also keeps the form's `htmx-request` class and its sync request (inferred from the vendored source), so the Action input stays disabled, Send looks busy, and the form drops every submit until a reload. A throw needs a defect in the preview markup or a handler; Scenario 16.68 states this outcome. A guardrail bans script writes of `disabled`. The action area still leaves Checking on `htmx:onLoadError` of the check request, because htmx skips `htmx:afterRequest` there. | The seven script writes of `disabled` (inventory D1) have no server rendering today. A load now corrects every control state. |
| P18 | The Save of a message edit reads the busy state through CSS (`:has()` on the status display), and shows as disabled with the O3 text while `data-status` is `thinking` or `autoplay`. A keyboard save during a Job gets the `409` refusal, and the draft stays. | No script changes its state. Alternative: no disabled look, only the refusal text after a click. |
| P19 | The active Settings sub-tab is kept in `sessionStorage`, as the active tab is. | Clear on leave would reset it every time. |
| P20 | An LLM Messages row is a header only: Role, backend and model, time, and the error mark; a failed row also shows a one-line error summary (Gate log 4). Opening a row opens the full record in the dialog, through `GET /fragment/llm-messages/{id}`. A Worlds row shows a shortened description, as a preset card does: `truncated_preview` (`builders/presets.rs:208`) changes to take a text, and both use it. | The 50 newest records hold 652 KB in the probe database, and each list load sends all of it today. A record never changes, so the dialog needs no load. |
| P21 | The saved Games show 50 for each page, with Previous and Next. The order stays `updated_at DESC` (`storage/games.rs:18`), with `id DESC` added so a page is stable. The active Game stays above the pages. The list is not grouped by World (known: each row carries a World badge). The page number is view state of the open tab, and leaving the tab resets it (O11). It is kept in `<html data-games-page>`, next to `data-revision` and `data-game`, so no part root gets a script writer. A server-rendered Previous or Next control carries the page that it goes to as `data-page`; one click listener in `dashboard-screen.js` sets the attribute, forgets the stamp and asks. `loadScreen` adds `games_page=N` to `GET /screen` while the attribute is set. A page past the end is clamped to the last page by the server, and the controls are `disabled` at the ends (server-rendered, P17). A load carries the page that was asked for when it started, and an answer that arrives after a click on Previous or Next is dropped, as an answer for a tab that is no longer visible is (§1): the click forgets the stamp, so a stale load that ends later could otherwise record it and hide the new page. CSS dims a disabled pager control (`.games-pager button:disabled`, look only). A `games_page` that is not a whole number of at least 1 answers `400`. | `list_games` has no limit (inventory H). The attribute is the one mechanism that P21 adds to the browser. |
| P22 | The part routes (`/fragment/story-log` and the others) stay. `GET /screen` calls the same renderers. The Settings, Prompt Presets, Worlds and Games handlers get renderers that return `Result<String>`, like the Game-tab renderers. | About 270 test lines read the part routes as the tier-1 read seam. One renderer, two routes. |

### Plan review decisions (2026-10-26)

The owner chose D1 = B (O7) and D2 = A (O8).

| # | Question | Options | Recommendation |
|---|---|---|---|
| D1 | Who holds the list of the parts of a tab? | A: the server, `GET /screen/{tab}` (P12 as written). B: the browser asks for the parts of the visible tab and the shell, `GET /screen?parts=header,failure-banner,story-log,…`. The server has one renderer for each part name, and answers `400` for an unknown name. | B. The shell already lists the parts as elements, so the layout has one source. The server knows renderers, not tabs, as owner decision Q2 intended. |
| D2 | What happens when one part fails to render? | A: the whole screen answers `500`, `#screen-error` shows it, and every part keeps its content (as written). B: the screen answers `200`; the failed part keeps its content and shows its error in its own slot, and the other parts load. | A. One failure path and one slot. A failure stays visible until the next good load. The cost: a part that fails every time stops the whole tab from updating, and Scenario 38.18 changes from "the panel shows its error" to "the screen shows its error". |

### Drift audit decisions (2026-10-27)

The stage reviews of stages 6 and 7 found real races, and each one got a local fix. The client grew from 1,838 lines of dashboard JS (`ebae627e`) to 2,070 (`a4c0f7dd`), and the new loop was the size of the one it replaced. A read-only audit of the code against the approved plan (`tmp/drift-audit/audit.md`) traced most of the growth to three gaps in the design: P1 did not say what the stamp means after a skip, P17 gave `disabled` two writers, and P13 opened a window while the dialog waits. The owner chose S1–S5 (O9, O10). The P13 mechanisms stay, because they are the cost of an owner decision. P1, P13, P14, P15, P17 and sections 1, 2, 6 and 8 below describe the state after stage 7b.

| # | Decision | Removes |
|---|---|---|
| S1 (C1) | The end of every action request forgets the stamp and asks. The end is `htmx:afterRequest` of a non-GET htmx request, `htmx:onLoadError` (htmx skips `afterRequest` when a swap throws), and the `.finally` of `actionFetch`. A load always records the stamp of what it rendered, also after a skip. The restart rule of `afterScreenShows` uses a load number: each load takes the next number when it starts, each wait records the number of the last load started, and only a load with a higher number can end the wait when the server id differs (section 2). | The kept stamp after a skip, `skippedInFlightInLoad`, `loadWaitsForRequests`, `aRequestOfThePageIsInFlight`, `serversMet`, `meetServer`, and the stage 7 forget for a control that the server rendered disabled (`renderedDisabled`). The load number replaces `serversMet`. |
| S2 (C2) | `hx-disabled-elt` names only shell fields: `#command-input` and `#autoplay-cycles`. Send and Autoplay show the in-flight state through CSS on the `htmx-request` class of their shell form, and are not `disabled` during that request. `hx-sync="this:drop"` already drops a second submit. | The stage 7 hold of the `hx-disabled-elt` controls: `controlsDisabledBy`, `disabledByRequest` and `renderedDisabled`. The P1 request counter (`holdInFlight`, `releaseInFlight`) stays. `disabled` on a part has one writer, the server. |
| S3 (C3) | O10. `data-game` comes from `X-Game` of every screen load, with no exception. | `GAME_CHANGING_ROUTES`, `forgetShownGameAfterGameChange`, `gameChanges`. |
| S4 (C4) | The look of the status display comes from CSS on its content (`#status-display:has([data-status="…"])`). No script writes the part root. | `syncStatusClass`, `STATUS_STATE_CLASSES`, `STATUS_CLASS_OF_STATE`, and the status-display rule of the convergence check. |
| S5 (C5) | Every path that starts the pending state ends it on its own answer: on success when `afterScreenShows` of the answer's `X-Revision` resolves, on a refusal or a failure at once. The five paths are the send (`onCommandAfterRequest`), the preview send (`onPreviewSendAfterRequest`), the Autoplay start (`onAutoplayAfterRequest`), retry and retrigger (`submitGenerationRequest` in `dashboard-story-log.js`, which starts the state before its request), and the Options regen button (`hx-on::before-request="updateToThinking()"`, `templates.rs:81`, which needs an after-request end). htmx fires its request events on the form, not on the button, so the handler on the button never ran and the regen showed no pending state (stage 7b red log); both handlers sit on the regen form. The send, the preview send and the Autoplay start begin the state only when the engine took the request, so they have no refusal end; retry, retrigger and the Options regen begin it before the request and end it at once on a refusal or a failure. The regen form reports its own `409` in `onNewOptionsAfterRequest`, as the Autoplay form does in `onAutoplayAfterRequest`, and is no saving form for `settleSave`. | `pendingLoadStarted` and its `screen:beforeload` listener. If no other listener or test uses `screen:beforeload`, the event goes too. |

Not chosen now, recorded for later: C6 (native focus return through stable opener ids; it needs a probe check first), C7 (the server renders an error slot next to every opener, so no script makes one), C8 (a `409` reports with the server's own sentence, so `X-Refusal` and `data-refusal-what` go away).

## The architecture after this plan

1. **Data flows one way.** An action saves data and answers only its outcome: a status code, and on success the revision after its commit. A failure shows in the inline slot of the form that sent it. Data reaches the screen only through a screen load.
2. **The page shows one screen, loaded in one request.** A screen is the header, the failure banner and the parts of the visible tab. Any change moves the revision, and the visible screen loads again. A tab's parts exist only while the tab is open.
3. **A screen load keeps what it does not change.** It morphs each part, keeps the view state that `data-keep` names and a showing inline error slot, and skips an element that holds unsaved input.
4. **A load never writes into a draft.** A form that waits for a Save is in the dialog, outside the screen. The one draft in a part is the story-log message in edit.
5. **The server renders data and control states. The browser owns view state.** No script sets `disabled`. The shell keeps the active tab, the Settings sub-tab, the open state of disclosures, and the action-area view state.
6. **A list sends short rows.** The full item loads when the player opens it. A list with no natural limit has pages.

## Design

### 1. The screen

- **Parts.** Each target element in `assets/index.html` has `data-part="<name>"`. The shell parts are `header` and `failure-banner`. The tab parts are:

  | Tab | Parts |
  |---|---|
  | Game | `story-log`, `visual-sidebar`, `options-dock`, `send-control`, `autoplay-controls`, `status-display` (stage 7) |
  | Settings | `settings` |
  | Prompt Presets | `prompt-presets` |
  | Worlds | `worlds` |
  | Games | `games` |
  | LLM Messages | `llm-messages` |

  A part element has no `hx-get`, no `hx-trigger` and no `hx-swap`. `data-live` and `data-live-state` are deleted.
- **The route.** `GET /screen?parts=<names>` checks each name against `SCREEN_PARTS`, reads the revision, then renders each named part with its renderer (P22). It answers `200` with one `<template data-part>` for each part, in the order of the request, and the headers `X-Revision: <server>:<revision>` and `X-Game: <id>` (empty with no active Game). An unknown, repeated or missing name answers `400`. If one renderer fails, the route answers `500` with the error text, and no part (O8).
- **The request.** The loop builds `parts` from the `data-part` elements outside every `.tab-content` (the header and the failure banner) and inside the visible `.tab-content`, in document order. While `<html data-games-page>` is set, the request also carries `games_page=<n>` (P21).
- **The loop** (`assets/dashboard-screen.js`, in place of `dashboard-live.js`). While the page is visible, it asks `GET /changes` every `data-poll-ms` (1,000 in the shell). An action asks at once. When `server:revision` differs from `<html data-revision>`, the loop loads the screen of the visible tab. One load is in flight at a time, and asks during a load queue one more load. On `200`, the loop morphs each template into its part in one step, calls `htmx.process` on each part, sets `data-revision` and `data-game` from the answer's headers with no exception (S1, S3), and fires `screen:loaded`. The end of every action request forgets `data-revision` and asks (S1). Each load carries the tab it was asked for. An answer for a tab that is no longer visible is dropped, and the loop loads the visible tab.
- **Events.** A direct morph fires no htmx swap event, so every listener that today watches a region's swap moves to an event of the loop. The loop fires `screen:beforemorph` after the answer and before the morph, `screen:loaded` after the morph, `screen:cleared` after a tab change empties its parts, and `screen:error` when `#screen-error` shows or clears a summary (the dialog status line reads it, P13).

  | Listener today | Moves to |
  |---|---|
  | `readStatusDisplay` (`hx-on::after-swap` on `#status-display`, `index.html:201`) | `screen:loaded` |
  | `pendingLoadStarted` on `htmx:beforeRequest` of `#status-display` (`dashboard-action-area.js:248`) | Deleted in stage 7b: the pending send ends through `afterScreenShows` (S5) |
  | `syncActionState`, `announceNewNarration`, `announceOptions` on `htmx:afterSwap` (`dashboard-action-area.js:267`) | `screen:loaded` |
  | `syncStatusClass` on `htmx:afterSwap` | Deleted in stage 7b: CSS on the content (S4) |
  | The story-log scroll follow: `storyLogSwapStart` on `htmx:beforeSwap` and `followStoryLogBottom` on `htmx:afterSwap` (`dashboard-story-log.js:329, 338`) | `screen:beforemorph` and `screen:loaded` |
  | `refreshFailureBanner` on `htmx:afterSettle` (`dashboard-errors.js:193`) | `screen:loaded`, and after `markUnreachable` and `clearUnreachable` |
  | `planFocusAfterLoad` and `applyFocusPlan` (`dashboard-live.js:454, 474`) | `screen:beforemorph` and `screen:loaded`, for each part |
  | `clearUnreachable` on a successful region load (`dashboard-errors.js:184`) | Every answer of `GET /changes` and `GET /screen` |

  The listeners on action requests (`htmx:afterRequest`, `htmx:responseError`, `htmx:sendError`) stay, because actions stay htmx requests.
- **What moves from `dashboard-live.js`.** To `dashboard-screen.js`: the Idiomorph callbacks (`data-keep`, the kept error slot, the skip rule), the focus plan, `askNow` and `actionFetch`. To the controls that save on change: `settleField`, `restoreRenderedValue`, `flagSaved` and the control branch of `settleSave`. Deleted: the records, the region states, `fieldIsDirty`, `regionHoldsEdit`, `endEdit`, the hidden-section rule, the `htmx.ajax` branch, the editor-slot click rule, the `toggle` listener and `window.chroniclerLive`.
- **A tab change.** `activateTab` empties each part of the tab that closes (`replaceChildren()`), clears `data-revision` and `data-games-page` (P21), and loads the screen of the tab that opens. The header and the banner load with it. `replaceChildren()` keeps the attributes of the part root, so the tab change also removes `data-editing-entry` from the story-log root, and the story-log script resets its edit state (`editState`, `dashboard-story-log.js:25`) on `screen:cleared`. The announced Narrations and Options stay as they were, so a return to the Game tab announces only what was added while it was closed, as today. A modal dialog makes the tab bar inert, so no tab changes while the dialog is open.
- **No active Game.** Boot always resolves a Game (`resolve_boot_game`, `bootstrap/run.rs:97`), and the active Game cannot be deleted (`catalogue.rs:88`). So only a test server has no active Game. Then the header part renders "No active game", the story log, the visual sidebar and the Options dock render empty, Send and Autoplay render enabled, the status display renders "Ready", and the other tabs render as usual, so the Games tab can still create a Game.
- **A reload.** The shell restores the active tab and the Settings sub-tab from `sessionStorage` (P19), then the first ask loads that tab.
- **Revision.** `Numbers` gets one field, `revision`, which each of the two inner raise functions adds to. `ChangeVersions` and the `GET /changes` answer get the field. The per-kind numbers and their tests stay.

### 2. Actions and their answers

- **Success.** Every action handler answers `2xx` with `X-Revision`, read after its write. An action that wrote nothing answers the current revision. A Job's send answers after `Job::start` has stored the Input.
- **Refusals.** `Refusal::StillThinking` answers `409` (`src/adapters/driving/http/error.rs:34`), with the same text. `BUSY_STATUS` and the client's string test of `class="status wait"` are deleted. `refusalOf` and `answeredRefused` test the status `409` and read `X-Refusal`.
- **The Game header.** `htmx:configRequest` and `actionFetch` add `X-Chronicler-Game` from `<html data-game>` to every non-GET request. A route layer on the 11 Game-bound POST routes (inventory F) compares it with the active Game (P14), not `CurrentGame`. A refused send keeps the typed text.
- **After the answer.** One helper, `afterScreenShows(revision)`, resolves when `<html data-revision>` has the same server id and at least the revision. If the server id differs (the server restarted), it resolves at the first good load that starts after the wait began; a load that started earlier may still carry the old server and does not count (stage 6 review L2). The load number of S1 marks this. The pending state of the action area ends the same way (S5). The dialog shows "Saving…" until then and then closes. The Action input clears then, if it still holds the sent text. The "Saved" line shows then.
- **Stages 2–4.** `X-Revision` comes in stage 5. Until then, a `2xx` closes the dialog and shows "Saved" at once, as today.

### 3. The dialog

This section is the edits draft's design, unchanged except for P7 and P13.

- **Element.** The shell has one `<dialog id="edit-dialog" aria-labelledby="edit-dialog-title">` with two children, `<div id="edit-dialog-body">` and the status line `#edit-dialog-status`, outside every tab.
- **Open.** An opener has `hx-get="<form route>"`, `hx-target="#edit-dialog-body"` and `hx-swap="innerHTML"`. On `htmx:afterSwap` into the body, `assets/dashboard-dialog.js` calls `showModal()` and focuses the first field. If the GET fails, the opener's inline slot shows the error. While the dialog is open, an answer into its body from a control outside the dialog does not swap: that request was sent before the dialog opened, and the form holds a draft (statement 4).
- **Root marker.** Each dialog fragment has a root with `data-dialog-form` and an `<h2 id="edit-dialog-title">`.
- **Save.** `hx-post` with `hx-swap="none"`. A `2xx` closes the dialog when the screen shows the revision (P13; at once before stage 5). A `400`, `409` or `500` shows in the form's inline slot, and the dialog stays open with the typed values.
- **Close.** Cancel (`data-close-dialog`), Escape and a save that went through call `dialog.close()`. The `close` event empties the body, so the next open loads a fresh form. A click on the backdrop does nothing.
- **Focus return.** `showModal()` returns focus to the opener on close (inferred from the HTML spec; no separate check in Chrome, Gate log 2, review L11). If the opener is gone, focus goes to its part. A load can rewrite the opener node into another control, so the dialog records the opener's request (`hx-get` or `hx-post`) and returns focus to the control of its part that sends that request, else to the part (Milestone A, U1). Ticket C6 would replace this.
- **A snapshot.** The form shows the data as it was when it opened. The last save wins, as today.

**The forms that move into the dialog:**

| Form | Today | Opener and GET route | Save route (unchanged) |
|---|---|---|---|
| Game Rename | `details.game-rename` (`games.rs:28`) | Rename button. New `GET /fragment/games/{id}/rename` | `POST /games/{id}/rename` |
| New Game | `details.new-game-add` (`games.rs:61`) | New Game button. New `GET /fragment/games/new` | `POST /games` |
| Preset Add (4 types) | `details.preset-add` (`prompt_presets.rs:21, 60, 95, 134`) | Add button for each type. New `GET /fragment/prompt-presets/new?type=<type>` | `POST /prompt-presets` |
| Preset Edit and View (P3) | Card swaps (`builders/presets.rs:26, 106`) | Existing `/fragment/prompt-presets/{id}/edit` and `/view` | `POST /prompt-presets/{id}` |
| World New and Edit, with posture (P2) | `#world-editor` slot (`worlds.rs:23, 74`) | Existing `/fragment/worlds/new` and `/worlds/{key}/edit` | `POST /worlds`, `/worlds/{key}`, `/worlds/{key}/posture` |
| Connection New and Edit, with Test | `#connection-editor` slot (`settings.rs:29, 145`) | Existing `/fragment/connections/new` and `/{id}/edit` | `POST /connections/add`, `/{id}/edit`, `/test` |
| Row Test result (P4) | `.connection-test-slot` (`settings.rs:65`) | The row's Test button posts into the dialog body | none |
| LLM record (P20) | Hidden `<details>` in each card (`templates.rs:145–192`) | Row button. New `GET /fragment/llm-messages/{id}` | none |

### 4. The controls that stay in a part

The Game mode, posture and preset selects (`games.rs:131–175`), the Settings role selects (`settings.rs:40–47`) and the text-check form (`settings.rs:272`) save on change. A save that goes through sets the field's rendered value, and "Saved" shows when the screen shows the revision. A failed or refused save puts the control back to its rendered value and shows the error in its slot. While its request is in flight, a load skips the control (P1).

### 5. The story-log message edit

- **Open.** The Edit button opens the inline editor (`showEditForm`). The entry gets `data-editing`, and the story-log root gets `data-editing-entry` (P9).
- **Loads.** `beforeNodeMorphed` and `beforeNodeRemoved` return `false` for a node with `data-editing`. New narration shows while the player types. Entries carry `id="entry-{id}"`, so Idiomorph matches by id.
- **Save while busy (O3, P18).** CSS shows the Save as disabled while the status display's `data-status` is `thinking` or `autoplay`. A `failed` status does not disable it, because the Game is not busy. The server refusal (Scenario 8.7, now `409`) stays as the second line of defence.
- **Leaving the Game tab** empties the story-log part, so the edit is discarded (O6).
- **The message is deleted from another tab.** The skipped entry stays until the edit ends. A Save gets the refusal of Scenario 8.9 and ends the edit.

### 6. Control states (P17)

| Control | Today (inventory D1) | After |
|---|---|---|
| Send | Script disables it while generating or checking | `#send-control` renders it, with `form="command-form"`, disabled while the Game is busy. During the request of the command form, CSS on `#command-form.htmx-request` shows it as busy and takes no click; `hx-sync="this:drop"` drops a second submit (S2). |
| Action input | Script disables it while checking | Stays in the shell. `hx-disabled-elt` on the command form. |
| Autoplay button and cycles | Script disables them unless idle | The button is in `#autoplay-controls`, disabled unless the Game is idle; during the request of its form, CSS shows it as busy (S2). The cycles field stays in the shell, keeps its value, and is disabled by `hx-disabled-elt` during that request. |
| Stop | Shown by the action-area state | Rendered in `#autoplay-controls` while a run holds the Game. |
| Other entries' Edit buttons during an edit | Script sets `disabled` | Root attribute, CSS and one click guard (P9) |
| Swipe controls of the edited entry | Script sets `disabled` | The same root attribute |

The action-area states Checking and Preview stay browser view state, as an attribute on `#action-area`. Idle, Generating and Autoplay come from the rendered parts. While the action area's own pending state shows in `#action-status`, CSS hides `#status-display`, so the two do not show together. The status display part is empty until its first load, and the status announcer starts at the label `Ready`, so a page that opens during a Job announces its step at the first load and an idle page announces nothing.

### 7. Failures

- A failed action shows in its own form's inline slot, as today.
- A network error on any request marks the engine unreachable. Any HTTP answer clears it (P15).
- A non-2xx `GET /changes` or `GET /screen` shows its summary in `#screen-error`, with the detail behind a Details control. The screen keeps its content, and the next ask tries again. An answer that the page cannot read, or a morph that throws, shows in `#screen-error` in the same way (Scenario 44.41). The next good ask or load clears the slot.
- A new server id makes the stamp differ, so the screen loads again.

### 8. Checks

- **Convergence check** (stage 1). A test helper waits until the screen is quiet, then compares each part with a fresh server render in the page. The comparison drops the attributes named in `data-keep`, the htmx classes, a showing inline error slot, and the story-log entry in edit. A control whose request is in flight is not dropped. From stage 6 it also checks that each part of a closed tab is empty, and compares part-root attributes (without `tabindex`, and with `data-editing-entry` exactly while an entry is in edit). It has one per-part rule after stage 7b: it asks the fresh render of the Games part for the page that `<html data-games-page>` names (O11). `StubRunner::run` calls it after each check, before it closes the page (`tests/browser/stub/support.rs:32–80`). `StubRunner::finish` is not the place, because it runs once after all checks. The helper waits until the page's stamp equals the server's and no load is in flight, then compares. While a part differs, it waits for the next load, up to a timeout, so a write that lands during the check does not fail it. The 14 tier-3 checks run it through their shared end step, `with_test_page` (`tests/test_utils/browser.rs`). A check that leaves the page unreachable on purpose opts out, with its reason.
- **Guardrails** (`tests/infrastructure/guardrails/live_regions.rs`, renamed `screen.rs`):
  - every `data-part` in the shell names a part in `SCREEN_PARTS`, every name in `SCREEN_PARTS` appears once in the shell, and each `.tab-content` holds at least one part (this replaces the region table check and the kind tokens);
  - a part element has no `hx-get`, `hx-trigger` or `hx-swap`;
  - only `dashboard-screen.js` writes into a part, with these named writers: the inline editor and the root lock of the story log (`dashboard-story-log.js`, O2, P9), the saved value of a control that saves on change (`dashboard-saves.js`, section 4), the Settings sub-tab attributes (`dashboard-panels.js`, P19), the inline error slots (`renderInlineError` fills them; `dashboard-errors.js` makes and removes generated slots, and writes `data-unreachable` on a slot, P15), the open state of an error detail (`dashboard-errors.js`, kept by `data-keep`), the Saved flag of a control that saves on change (`dashboard-saves.js`), and `tabindex` on a part root for the focus fallback. The guardrail matches selectors, so it sees the exception row of `dashboard-errors.js` but not writes through parameters; the review checks the rest;
  - no script listens for `htmx:beforeSwap`, `htmx:afterSwap` or `htmx:afterSettle` of a part (section 1, Events);
  - no script writes `disabled` (P17);
  - a draft control appears only in a `data-dialog-form` template (P8);
  - the existing checks stay, with "part" in place of "region": no `HX-Refresh`, no retarget or `hx-target` into a part, no out-of-band swap, scripts use `actionFetch`.
- **Screen sweep** (P8). A stub check loads every tab and finds no draft control in any part.

### 9. Docs

- `docs/diataxis/reference/frontend/dashboard.md`: the six statements, the parts table, the markers, the dialog, the failures.
- `docs/diataxis/explanation/dashboard_design.md`: why one screen load, why the dialog, why clear on leave; the clean room as the source.
- `docs/diataxis/reference/storage.md`: the revision in the Change Counters section.
- `docs/specs/browser_live_regions.md` becomes `docs/specs/browser_screen.md` (chapter 44 kept). The record, hold and hidden-section scenarios are replaced.
- New `docs/specs/screen.md` (chapter 45, tier 1): `GET /screen`, the revision, `X-Revision`, the `409` refusal and the Game header.

## What already exists

- The change numbers, the server id and `GET /changes` (`changes.rs`, `endpoints.rs:80`). Every raise passes through `Numbers::raise_game` or `Numbers::raise_engine` (known).
- The renderers of the Game tab and LLM Messages: `AppState` methods that return `Result<String>` (`app_state.rs:84–172`). The status display uses plain functions (`busy_answer`, `idle_answer`, `failure_answer`).
- The Idiomorph callbacks for `data-keep` and the kept error slot (`dashboard-live.js:414–439`). The skip rule adds conditions to them.
- `actionFetch`, `markUnreachable`, `clearUnreachable`, `renderInlineError` and the inline slots.
- htmx 1.9.10 adds `htmx-request` to the element that sends, and supports `hx-disabled-elt`, which it clears on load, error, abort and timeout (known, from the vendored source).
- The editor GET routes for Worlds, connections and presets.
- No `<dialog>` exists in `assets/`.
- The controls that save on change carry their own `hx-post` (`games.rs:133–171`, `settings.rs:40`), so htmx puts `htmx-request` on the control itself, and the skip rule can find it (known).
- htmx finds the form of a submit button through its `form` attribute (known, from the vendored source), so a Send button outside `#command-form` with `form="command-form"` submits it (P17).
- `truncated_preview` (`builders/presets.rs:208`) for the Worlds rows.
- `activateTab` and the active-tab key in `sessionStorage` (`dashboard-panels.js:94–112`).

## Blast radius

- **Server:** `revision` in `Numbers` and `ChangeVersions`; `GET /screen?parts=` and `SCREEN_PARTS`; `String` renderers for four panels; four new GET routes (Rename, New Game, preset Add, LLM record); `X-Revision` on every action answer; `409` for the busy refusal; the Game-header route layer on the 11 Game-bound routes; `#send-control` and `#autoplay-controls` renderers; short rows for LLM Messages and Worlds; Games pages.
- **Assets:** `index.html` (parts, dialog, `#screen-error`, `#send-control`, `#autoplay-controls`, `data-poll-ms`, `?v=` bumps); new `dashboard-screen.js` and `dashboard-dialog.js`; `dashboard-live.js` deleted; `dashboard-errors.js` (status classifier, unreachable, `INLINE_ERROR_CONTAINERS` on `[data-part]`, P15); `dashboard-action-area.js` (no `disabled` writes, `afterScreenShows`); `dashboard-story-log.js` (`data-editing`, root lock); `dashboard-panels.js` (tab change, sub-tab storage, deletions); `styles.css`.
- **The busy refusal:** 7 lines in `src/` (3 files), 19 lines in `assets/` (4 files), 20 lines in `docs/specs/` (8 files), 30 lines in `tests/` (8 files).
- **Specs:**
  - stages 2–3: the scenarios that the edits draft listed (`browser_games.md` 27.4–27.7; `browser_prompt_presets.md` 28.1, 28.2, 28.4, 28.5; `browser_worlds.md` 29.2, 29.3; `browser_story_log.md` 30.3, 30.4, 30.9, 30.15–30.17; `browser_settings.md` 40.2, 40.3; `browser_dashboard.md` 16.29–16.31, 16.35; `settings.md` 20.24; `prompt_presets.md` 21.28; `browser_autoplay.md` 42.5; `browser_live_regions.md` 44.9–44.11, 44.17, 44.18, 44.20–44.23);
  - stage 4: `browser_llm_messages.md` 35.x, `llm_messages.md` 33.x, `browser_worlds.md`;
  - stage 5: the 20 refusal lines; new chapter 45;
  - stage 6: `browser_live_regions.md` rewritten as `browser_screen.md`; `browser_dashboard.md` 16.25 and 16.38; `failure_display.md` 38.18;
  - stage 7: the action-area scenarios that check a disabled Send or Autoplay (`browser_dashboard.md`, `browser_autoplay.md`);
  - stage 7b: `browser_dashboard.md` 16.56 (kept; text checked), 16.57 (O10 outcome), 16.60 (deleted with `gameChanges`), 16.62 (Send and Autoplay look busy, not `disabled`), 16.64, 16.65, 16.68 (the dead form after a throw), 16.41 (the colour comes from the content, so the class lines go, S4), 16.69 and 16.70 (the pending paths end, S5); `browser_screen.md` 44.24 (same stamp line as 44.27) and 44.15 (a script action that wrote nothing still loads once); `browser_screen.md` 44.27 (its stamp line), 44.39 (each revision move loads once), 44.40 (restated for the load number; it is the only check of stage 6 review L2);
  - stage 8: `browser_games.md`, `games.md`.
- **Tests:** the tagged tests of those scenarios; `tests/browser/stub/live_regions.rs` (rewritten and split, review finding M7); the 19 `chroniclerLive` uses in 4 files; `tests/test_utils/stub_server.rs` (it serves `GET /screen?parts=` and `revision`); the 36 guardrail unit tests in `live_regions_tests.rs`; the 41 `load_now` calls in the stub checks, which use `htmx.ajax` on a region; the 23 `SCENARIO` tags that name `browser_live_regions.md` and change with its rename; the listeners in the Events table (section 1); the three behaviour tests in `settings_tests.rs` (review finding M6) move to scenarios in stage 2.
- **Not touched:** storage, the per-kind numbers and their tests, the Job and the writer rule, the save routes (apart from their answer headers).

## Failure modes

| Failure | Effect | Answer |
|---|---|---|
| A write commits while `GET /screen` renders its parts | One part shows the write, another does not | The screen holds the revision read before the render. The next ask sees a newer revision and loads again, within one ask. |
| A new form is added inside a part | A load could erase its draft | The guardrail fails the build. The screen sweep fails the stub tier. |
| A part is in the shell but has no renderer, or a renderer has no part in the shell | The screen answers `400`, or a renderer is never used | The parts guardrail fails the build. A `400` on `GET /screen` fails every browser check of that tab. |
| A save goes through, then the screen cannot load | The dialog stays at "Saving…" | `#screen-error` shows the reason. The dialog closes at the next good load. |
| A double click on Send after the answer, before the screen shows the Job | A second send | The server refuses it with `409`. The text stays. |
| A tab sends after another tab switched the Game | The send would go to the other Game | `409` "The active Game changed" (P14). The text stays, and the next load shows the new Game. |
| This tab creates, switches or resets a Game, then sends before the next load | The page still names the old Game | The same `409`, and the text stays (O10). Create, switch and Reset are on the Games tab, so a send needs a tab change, which loads first. |
| The edited message is deleted from another tab | A skipped entry stays until the edit ends | Save gets the refusal of Scenario 8.9 and ends the edit. |
| `showModal()` does not return focus in Chrome | Focus goes to `<body>` | Stage 2 checks it first. If it fails, the dialog script records the opener. |
| The player changes tab while a screen load is in flight | The old tab's answer would fill hidden, emptied parts and set the stamp, so the new tab would not load | The loop drops an answer for a tab that is not visible, and loads the visible tab (section 1). |
| A Stop from a tab whose Game is no longer active | It would act on no Game, or on the wrong one | The route layer answers `409` before the handler runs (P14). |
| The server restarts between an action and the next load | The answer's revision belongs to the old server | `afterScreenShows` resolves at the first good load that started after the wait began (section 2, S1). |
| A listener still watches a region's htmx swap after stage 6 | It never runs: no announcement, no scroll follow | The Events table lists every listener. A guardrail forbids `htmx:beforeSwap`, `htmx:afterSwap` and `htmx:afterSettle` listeners that test a part's id. |

## Stages

### How a stage runs

1. A subagent builds the stage (the role is in the table below). It writes the failing tests of each new or changed scenario first, cleans its own comments, and runs `python build.py --no-browser` once before it reports.
2. The gate of the stage runs. If the gate passes, the stage is committed on `wf/change-driven-dashboard-refresh` and the next stage starts. The owner is not asked between stages. The owner's approval of this plan covers the commit at each gate that passes.
3. The gate adds one row to the Gate log (end of this section): the commit, the checks, and each difference from the plan with its reason.
4. If a stop condition holds, the work stops, and the owner decides.
5. From stage 7b on: a fix (from a review or a failing test) that adds a counter, a flag, a list, a listener or a CSS state rule edits the plan body in the same pass, with its reason. If the plan's design does not explain why it is needed, it goes to the owner under stop condition 4. (Retro ticket 86: per-stage fixes moved the code away from the plan.)

### Gates

| Gate | Steps |
|---|---|
| **Light** | 1. The exit checks of the stage. 2. `python build.py` is green. 3. Commit. |
| **Review** | 1. `/simple-after-plan` in slice mode: one `reviewer` checks the spec, the standards, the comments and the docs of the stage, from one bundle (`scripts/prepare_review_bundle.py --uncommitted`). 2. Each finding is verified; the fixes go back to the implementer's resumed session. 3. The Light steps. |
| **Milestone** | 1. The Review steps, up to the green full gate. 2. A UI spot check with `/chronicler-ui-investigator` on a probe server (port 3001, `--target-dir target/ui-investigator`, real connections): each surface in the milestone list, with a shot in `tmp/ui-gates/<milestone>/` that the orchestrator has looked at. 3. Commit. 4. A short report to the owner. The work goes on unless a stop condition holds. |
| **Final** | After stage 8. See Final gate. |

| Stage | Builder | Gate | Exit checks |
|---|---|---|---|
| 1. Convergence check | `strong-implementer` | Review | The check runs after every stub check and every tier-3 check. A mutation (one region that skips its load) makes it fail. Each allow-list row has a reason and names the stage that removes it. |
| 2. Dialog, skip rule, connection forms | `strong-implementer` | Review | The four first checks pass. The Test result of a saved connection survives a Settings load (H3). The template guardrail fails on a test form inside a region. |
| 3. Other forms, story-log edit, end of the hold | `strong-implementer` | **Milestone A** | `run_live_regions_checks` passes 10 runs in a row (open flake F1 in the Gate log). `regionHoldsEdit`, `fieldIsDirty`, `held`, `data-work`, `data-close-on-save`, `data-reset-on-save`, `data-opens-editor`, `data-close-editor` and `editor-slot` are gone from `assets/` and `src/`. The sweep allow list is empty. The guardrail allow list holds only field-builder rows (P8). |
| 4. Short rows | `implementer` | Light | The size of an LLM Messages list load, before and after, is in the Gate log. The full record opens in the dialog. |
| 5. Action answers | `strong-implementer` | Review | One tier-1 table test sends each action route and finds `X-Revision`. No code tests the string `class="status wait"`. A stale Game header answers `409` on each of the 11 routes. |
| 6. Screen load, clear on leave | `strong-implementer` | **Milestone B** | The first checks and their numbers are in the Gate log. `data-live`, `data-live-state`, `window.chroniclerLive` and `dashboard-live.js` are gone. The convergence check passes with an empty allow list. |
| 7. Control states | `implementer` | Review | The guardrail finds no script write of `disabled`. The action-area scenarios pass. |
| 7b. Simplify | `strong-implementer` | Review, with a whole-client check | The names that S1–S5 remove are gone from `assets/` and `tests/` (`rg`). The convergence check has no per-part rule. Each changed scenario has a red log. The dashboard JS line count before (`a4c0f7dd`) and after is in the Gate log. The reviewer reads every first-party script in `assets/` and the rules of `styles.css` that carry behaviour (`:has()`, `pointer-events`, state attributes) against this plan, not only the diff, and lists each mechanism that this plan does not name. Before the commit, each listed item points to a sentence of the plan body, or goes to the owner (stop condition 4). |
| 8. Games pages, docs | `implementer` | Light, then Final | A tier-1 test with 51 saved Games finds two pages. `validate-docs` passes. |

**Milestone A (drafts outside the screen), surfaces:**
- each dialog: open, save, a refused save (an invalid value), Cancel, Escape, and focus back on the opener;
- the Test result of a saved connection;
- a message edit while a Job runs: new narration arrives around it, and Save looks disabled until the status is Ready;
- the Games and Prompt Presets panels with buttons in place of disclosures.

**Milestone B (one screen load), surfaces:**
- tab changes during an Autoplay run: the closed tab is empty in the DOM, and the opened tab loads once;
- one `GET /screen` for each change on the Game tab, and its size and time against the stage 6 limits;
- a second browser tab switches the Game: the first tab shows the new Game;
- the server stops, then starts: the Unreachable banner shows and clears, and the screen recovers with no reload.

### Stop conditions

The work stops, and the owner decides, when:

1. a "checks first" step fails (stage 2 item 1, stage 6 item 1);
2. stage 1 finds a mismatch whose fix needs a design choice that this plan does not make;
3. a stage must change a behaviour that a spec states, and this plan does not name that change;
4. a review finding or a UI spot-check finding needs a design change, not a fix;
5. the full gate fails, and the fix needs a design choice. A failing test whose fix is clear is fixed, also when it seems unrelated (`AGENTS.md`);
6. a stage needs a step that this plan puts in a later stage, or the order of the stages no longer works.

### Stage 1 — The convergence check

1. The helper (section 8), against today's design: each visible `data-live` region whose state is `current`, compared with a fresh GET of its `hx-get` route.
2. Call it from `StubRunner::run` and from `with_test_page`, which ends each of the 14 tier-3 checks. In this stage, "quiet" means that each visible region is `current` and its record equals the numbers of `GET /changes`.
3. Report each mismatch it finds. A mismatch whose cause a later stage removes goes on an allow list with its reason, and the list only shrinks. Any other mismatch gets a failing scenario and a fix in this stage.

### Stage 2 — The dialog, the skip rule and the connection forms

1. Browser checks first (stub tier): an htmx swap into `#edit-dialog-body` opens the dialog; `showModal()` returns focus to the opener; a load skips a select whose request is in flight; after the save, the next load sets the select to the server value. If a check fails, stop and report.
2. The dialog, `dashboard-dialog.js`, the CSS and `data-dialog-form`.
3. The skip rule (P1).
4. The connection New and Edit forms and the row Test (P4) move into the dialog. `#connection-editor` is deleted. This closes H3.
5. The template guardrail and the screen sweep, with an allow list of the forms that stage 3 moves.
6. Specs and tests: 40.2, 40.3, 16.29, 16.35, 20.24, 44.22, the dialog scenarios, and the three `settings_tests.rs` tests.

### Stage 3 — The other forms, the story-log edit, the end of the hold

1. Rename, New Game, the preset Add forms, preset Edit and View, World New and Edit.
2. The story-log edit uses `data-editing` and the root lock (P9).
3. Delete the hold (`regionHoldsEdit`, `fieldIsDirty`, `held`), `data-work`, `data-close-on-save`, `data-reset-on-save`, `data-opens-editor`, `data-close-editor`, `#world-editor`, the disclosures, the preset card swaps, the `toggle` listener and the editor-slot rules. Empty the allow lists.
4. Specs and tests: the stage 2–3 list in the Blast radius section.

### Stage 4 — Short rows

1. LLM Messages rows show the header only, and the record opens in the dialog (P20). Measure the list load before and after on the probe database.
2. Worlds rows show a shortened description.

### Stage 5 — Action answers

1. `revision` in `Numbers`, `ChangeVersions` and `GET /changes`, with tier-1 scenarios.
2. `X-Revision` on every action answer.
3. The busy refusal answers `409`. The 20 spec lines and 30 test lines change. The client tests the status.
4. The Game header (P14): the client sends it, and the route layer on the 11 Game-bound routes checks it.

### Stage 6 — The screen load and clear on leave

1. Checks first, on the probe server and in the stub tier:
   - a direct `Idiomorph.morph` followed by `htmx.process` makes a new row's `hx-post` work, and a changed `hx-post` on a kept node use its new value;
   - a direct `Idiomorph.morph` applies the `Idiomorph.defaults` callbacks (`data-keep`, the kept error slot, the skip rule);
   - the size and the server time of `GET /screen` for the Game-tab parts with a full window of 50 Messages, during an Autoplay run. If one load takes more than 100 ms on the server or sends more than 200 KB, stop and report.
2. `GET /screen?parts=` with `SCREEN_PARTS`, and the part renderers (P22).
3. `dashboard-screen.js`: the loop, the tab change, the stamp, `afterScreenShows`, `#screen-error`, and unreachable only on a network error.
   - The client half of P13, deferred from stage 5: the dialog closes, "Saved" shows and the Action input clears (only if it still holds the sent text) when `afterScreenShows(X-Revision)` resolves, not on a 2xx.
   - `X-Game` from the screen answer replaces the interim `<html data-game>` and `recordShownGame` of stage 5; the rule that a create, switch or Reset from this tab clears the Game until the next load stays (removed in stage 7b, O10).
   - `submitEdit` stops calling `chroniclerLive.settle()`.
   - `tests/test_utils/stub_server.rs` is at 1,980 of 2,000 lines: split it before it serves `GET /screen`.
4. The shell: `data-part`, no region attributes, `data-poll-ms`, the sub-tab in `sessionStorage`.
5. Change the focus fallback of `dashboard-dialog.js` from `[data-live]` to `[data-part]`. Delete `dashboard-live.js` (records, states, the hidden-section rule, the `htmx.ajax` branch, `chroniclerLive`) and the `HX-Reswap: none` answers of the four panels. `INLINE_ERROR_CONTAINERS` stays, keyed on `[data-part]` (P15; this step said "delete" until 2026-10-27).
6. The guardrails and the convergence check move to parts. The stub server serves `GET /screen?parts=`.
7. Specs and tests: `browser_screen.md`, 16.25, 16.38, 38.18, and the 19 `chroniclerLive` uses.

### Stage 7 — Control states

1. `#send-control` (Send) and `#autoplay-controls` (Autoplay, Stop), rendered from the busy state.
2. `hx-disabled-elt` on the command form and the Autoplay form.
3. The Checking and Preview view states as an attribute on `#action-area`.
4. The guardrail that bans script writes of `disabled`.
5. Specs and tests for the action area.

### Stage 7b — Simplify (O9, O10)

The source is `tmp/drift-audit/audit.md`, proposals C1–C5. Each step removes a mechanism; none adds one. If a step needs a new mechanism, stop and report (stop condition 4).

1. **S1.** On the end of every action request (`htmx:afterRequest` of a non-GET request, `htmx:onLoadError`, and the `.finally` of `actionFetch`), forget the stamp and ask. A load sets the stamp from its answer, also after a skip. Delete `skippedInFlightInLoad`, `loadWaitsForRequests`, `aRequestOfThePageIsInFlight`, `serversMet`, `meetServer` and `renderedDisabled`. Add the load number of S1 for the restart rule of `afterScreenShows` (section 2); it is the one mechanism this stage adds. The P1 request counter (`holdInFlight`, `releaseInFlight`) stays, because a load still skips a control in flight. The edit end of the story-log entry still forgets the stamp.
2. **S2.** `hx-disabled-elt="#command-input"` on the command form and `hx-disabled-elt="#autoplay-cycles"` on the Autoplay form. CSS on the form's `htmx-request` class shows Send and Autoplay as busy and takes no click during the request of their form. Delete `controlsDisabledBy` and `disabledByRequest`.
3. **S3.** Delete `GAME_CHANGING_ROUTES`, `forgetShownGameAfterGameChange` and `gameChanges`.
4. **S4.** The status display look in CSS. Delete `syncStatusClass`, `STATUS_STATE_CLASSES`, `STATUS_CLASS_OF_STATE`, any helper that only they use, and the convergence check's status-display rule.
5. **S5.** Delete `pendingLoadStarted` and its `screen:beforeload` listener. Each of the five paths of S5 ends the pending state on its own answer: on success through `afterScreenShows` of the answer's `X-Revision`, on a refusal or a failure at once. The Options regen button gets an after-request end. The server side holds: `GameWriter::accept` and `reserve_for_autoplay` raise the busy state inside the hold, and the `X-Revision` layer reads the stamp after the handler. Delete the `screen:beforeload` event if nothing else uses it.
6. **Specs and tests.** Failing tests first for each changed scenario (the Blast radius, stage 7b row). 16.57 states O10: the `409`, the kept text, and the next load showing the Game; 16.60 is deleted. 44.27 loses its stamp line. 44.39 becomes "each revision move loads once". 44.40 is restated for the load number and stays the check of stage 6 review L2. 16.62, 16.64 and 16.65 follow S2. 16.68 states the dead form after a throw (P17). Each of the five pending paths has a check that its pending state ends. A check that a mutation used as red evidence must still fail at its own assertion when its rule is removed.
7. **Docs.** No doc names the deleted symbols, but `docs/diataxis/reference/frontend/dashboard.md` states the old rules: line 45 (a request disables its own controls through `hx-disabled-elt`), lines 61 and 76 (a load that skipped a control in flight keeps the stamp), and line 21 (check that it still holds). Rewrite them to S1 and S2. Stage 8 writes the rest of section 9.

### Stage 8 — Games pages

1. `list_games` with a limit and an offset, and the Previous and Next controls (P21).
2. Specs and tests in `games.md` and `browser_games.md`.
3. Docs (section 9).

### Final gate

The review scope is the branch from the commit that adds this plan to its last stage commit. The earlier commits on the branch were reviewed with the base plan.

1. `/chronicler-after-plan-workflow-plus-review`, steps 1–11, with that fixed point: verify the build against this plan, archive the plan, tests, coverage, slop and duplicates, docs, comments, and the full gate. The verify step lists each client and server mechanism that this plan does not name, as the drift audit did.
2. The UI pass with `/chronicler-ui-investigator` on a probe server with real connections. Each item needs a shot in `tmp/ui-gates/final/` that the orchestrator has looked at:
   1. every tab loads when it opens and is empty in the DOM after it closes;
   2. each dialog: open, save, refused save, Cancel, Escape, and focus back on the opener;
   3. a message edit during a Job: new narration arrives, Save looks disabled, the edit saves after Ready;
   4. the story follows new narration at its bottom, and stays where it is when scrolled up;
   5. an Autoplay run: Send and Autoplay are disabled, Stop shows, and a tab change to LLM Messages and back works;
   6. LLM Messages: short rows, the record dialog, and the size of a list load;
   7. two browser tabs: tab B switches the Game; a send from tab A is refused with "The active Game changed", keeps its text, and tab A then shows the new Game;
   8. a busy refusal (`409`) shows in the slot of its form;
   9. the server stops and starts: Unreachable shows, then clears, with no reload;
   10. a page reload restores the active tab and the Settings sub-tab;
   11. the Games list with 51 saved Games: two pages;
   12. keyboard only: each dialog keeps focus inside, Escape closes it, and focus returns.
3. The four reviews of `/chronicler-after-plan-workflow-plus-review`, in parallel with the UI pass.
4. One report to the owner: the findings of the reviews and of the UI pass, each verified, grouped by priority. No fix pass and no merge to `main` start before the owner decides.

### Gate log

| Stage | Commit | Gate | Checks | Differences from the plan |
|---|---|---|---|---|
| 1 | `67a60bf5` | Review | Full gate `logs/build_20261026_160102.log` (244 guardrail, 1460 integration, 34 browser). Mutation runs fail as they should: a region that skips its load (`logs/build_20261026_150753.log`), a stub fragment that changes without its number (`logs/build_20261026_150836.log`), a dirty select outside an allowed editor (`logs/build_20261026_154357.log`). All six allow-list rows fire. Review `tmp/review/stage1/review.md`: M1, L3, L7, L8 fixed; L1, L4, L5 rejected. | The tier-3 call is in `with_test_page`, not in 14 places (the plan was wrong: a shared end step exists). `StubServer::lent()` keeps the stub server alive for the check. No spec scenario: the check is test infrastructure. Opt-outs also cover two layout checks that write their own markup (review L2). `convergence.rs` names this plan and the stage of each allow-list row until stage 6 empties the list (review L6). The browser tier takes about 157 s, not about 92 s. Two stub settings routes now keep the saved value and move the `settings` number. Flake fixed: Scenario 26.5 (`options.rs`) read the status display before its load; it now waits for it. **Open flake F1:** Scenario 44.20 (`check_a_card_in_edit_survives_the_activate_of_another_card`) fails 3 of 8 runs on `ebae627e` without stage 1 (`logs/build_20261026_155655.log`): after Cancel of a preset edit, no panel load happens before `settle()` returns. Cause unknown. Stage 3 deletes the mechanism; its exit check needs 10 green runs. |
| 2 | `9f45cdde` | Review | Full gate `logs/build_20261026_170846.log` (259 guardrail, 1460 integration, 36 browser). First checks red then green (`tmp/stage2/red-*.log`, `green-*.log`). H3 closed: 40.5. Guardrail detection tests for a form inside a region, an unconditional `disabled`, a dialog form without its title, and a permanent row with a `<form`. Review `tmp/review/stage2/review.md`: M1–M6, L1–L3, L5, L6 (first half), L7–L9, L12, L13 fixed. A full gate failed once in 44.19 (`logs/build_20261026_164930.log`); the convergence check found a defect in P1: a load that skipped a form in flight left it stale until the next change. Fixed with 44.27 (red `tmp/stage2/red3-run_draft_checks.log`), 5/5 repeats of the draft and live-region runs. | The page counts requests itself, because the class `htmx-request` can stay after a swap (P1 updated). A skipped load leaves its region due (P1 updated). The P8 guardrail rule is per source literal, with `data-shell-form` for the text-check preview and one permanent field-builder row, `builders/forms.rs` (P8 updated). The `data-editing` half of P1 waits for stage 3. The saved-connection Test answers a dialog fragment; 20.19 and 20.20 reworded. 39.2 lost its icon with the deleted back link. 44.26 states the final rule while its test carries the stage 3 allow list. 44.24 and 44.27 switch off `window.regionHoldsEdit` in the page, because the hold blocks every load while a save is in flight; stage 3 deletes these lines with the hold. The dialog's focus fallback uses `[data-live]` until stage 6 (item 5). Not fixed: review L4 (a skipped select whose save fails after another tab changed the role can show a stale value until the next settings change; P1's due rule now loads it again), L11 (no separate check that Chrome returns focus after `showModal()`; the fallback covers it). |
| 3 | `fbc0feb5` | **Milestone A** | Full gate `logs/build_20261026_181828.log` (259 guardrail, 1459 integration, 36 browser). Exit checks: the hold names and markers are gone from `assets/` and `src/`; the sweep and convergence allow lists and their code are deleted; the guardrail list holds only `builders/forms.rs`; `run_live_regions_checks` 10/10 before the fixes and 5/5 after (F1 is gone with the card swap; 44.20 rewritten). Review `tmp/review/stage3/review.md`: M1, M2, L1–L5, L9–L11 fixed. UI check `tmp/ui-gates/milestone-a/findings.md` (shots 01–07, each looked at): 11 surfaces pass; U1 (focus returned to an opener node that a load rewrote into Set Active or Delete) and U2 (dialog pinned top-left by `* { margin: 0 }`) fixed with 16.51 and 16.52 and checked again on the probe (shot 07). Red logs in `tmp/stage3/`; no red logs for the stub dialog-form scenarios written with the code (27.4–27.7, 44.17–44.21, 16.30, 16.31, 28.5, 28.6, 29.x, 28.1, 28.2); the review judged each and M2 strengthened the weak ones. | A load that skipped the entry in edit clears the region record when the edit ends (P1 updated). Rename and New Game are buttons that open the dialog, so 27.3 focus lands on Rename. The GET refusals 21.5, 21.6 and 21.8 answer 400. New routes `GET /fragment/games/new`, `/fragment/games/:id/rename`, `/fragment/prompt-presets/new?type=`; the single-card route is deleted (21.2, 21.3 deleted; 44.23 deleted with the disclosures). 44.13 scope cut. World posture selects stay inside the World dialog (P2). A script-made inline error slot is removed, not hidden, after a successful save (found by the convergence check). Locked Edit and swipe buttons have no `aria-disabled`, because no script may write it and a load would drop it (review L7). Not fixed: review L8 (no scenario for a missing `type=` or the 500 answers of the new form routes), L12 (`submitEdit` calls `chroniclerLive.settle()`; stage 6). Three retargeted legacy unit tests deleted (covered by 21.24–21.27). |
| 4 | `7adf77a7` | Light | Full gate `logs/build_20261026_184109.log` (259 guardrail, 1463 integration, 36 browser). `GET /fragment/llm-messages` on a copy of the probe database (50 records): 1,374,004 bytes before, 36,269 bytes after; one record answers 9,768–64,847 bytes. The record opens in the dialog (35.1, 35.2). Red logs in `tmp/stage4/logs/`. | New route `GET /fragment/llm-messages/:id` (404 for a missing id, into the row's slot; 33.5, 35.6). A failed row shows the `ERROR` mark and a one-line summary. `truncated_preview` moved to `utils/template_helpers.rs`. The short Worlds row is tier 1 only (25.13). One legacy unit test lost its assertion on response text in the list. 35.x rewritten for Open and Close; the expand and collapse scenarios are deleted. Nobody looked at the new rows in a real browser yet: final gate item 6. |
| 5 | `0ca0e3d0` | Review | Full gate `logs/build_20261026_210039.log` (259 guardrail, 1471 integration, 36 browser). Exit checks: 45.1 sends every POST route and finds `X-Revision` (and fails on a route missing from its table); `rg 'status wait'` finds nothing in `src`, `assets`, `tests`, `docs`; 45.4 refuses a stale Game header on each of the 11 routes, and its rows and the stub's list equal `game_bound_routes()`; 45.5 shows a stale Stop does not stop a Job (`Option<CurrentGame>`). Review `tmp/review/stage5/review.md`: M1–M4, L1–L6, L8, L9, L11, L12 fixed. Red logs in `tmp/stage5/logs/`. | The client half of P13 moved to stage 6 (no screen stamp before it). New header `X-Refusal: busy \| game-changed`, because both refusals answer `409`. Interim `<html data-game>`: recorded when no visible region is due or loading; cleared after a create, switch or Reset from this tab (16.56, 16.57). A header that names no Game answers `409`. A `409` to a plain button is reported in its container's slot, or in the command form's slot (16.58 Stop, 16.59 Reset). The scenario chapter is `docs/specs/screen.md` (45.x) from this stage. The review ran about two hours after the build, because of the model quota. |
| 6 | `577c358a` | **Milestone B** | Full gate `logs/build_20261026_231825.log` (273 guardrail, 1478 integration, 39 browser); browser tier green twice more (`_232216`, `_232451`). First checks: a direct morph plus `htmx.process` wires new and changed `hx-post` (44.30); the `Idiomorph.defaults` callbacks apply (44.31, 44.32); `GET /screen` for the Game tab with 50 Messages during an Autoplay run: 114,888 bytes, 21.7–25.2 ms (`tmp/stage6/first-check-c-proxy.txt`). `data-live`, `data-live-state`, `chroniclerLive` and `dashboard-live.js` are gone from `assets/` and `src/`. Review `tmp/review/stage6/review.md`: M1–M6, L1–L12, D1–D4, D6, D7 fixed; each targeted stub check fails at its own assertion under its mutation (`tmp/stage6/review-fixes/mutations/`). UI check `tmp/ui-gates/milestone-b/findings.md` (shots 01–05, each looked at): 9 surfaces pass. | The stub checks were written after the code; their red evidence is the mutation runs. `INLINE_ERROR_CONTAINERS` stays, keyed on `[data-part]`. `dashboard-errors.js` is the one named exception to "only the screen script writes into a part" (it appends a generated error slot). A load that skips a control in flight keeps the stamp, and asks do not reload until that request ends; a load that skips the entry in edit moves it, and the edit end forgets the stamp. A dialog session counter and a `data-saving` guard stop a late close and a second submit. A `gameChanges` counter stops an old load from writing back the Game. The screen error shows in the dialog status line while the dialog is open. The Settings sub-tab is in `sessionStorage`. `docs/specs/browser_live_regions.md` is now `browser_screen.md`; 44.23 and 44.33 are unused (review L13). `dashboard_design.md` stays false until stage 8 (review D5). The stub shell asks every 100 ms. **Open flake F2:** `check_every_failure_sentence_reads_in_full` failed once in the convergence wait ("an ask or a load is in flight"); cause not proven (guess: the 100 ms check pause in phase with the 100 ms stub ask); the pause is now random 60–140 ms; not seen in 6 later runs. Pre-existing, for the owner: the LLM Messages list is oldest-first within the newest 50 (`list_latest_llm_messages` reverses its query). |
| 7 | `a4c0f7dd` | Review | Full gate `logs/build_20261026_235737.log` before the review fixes; after them `logs/build_20261027_002926.log` (286 guardrail, 1484 integration, 40 browser), browser tier again `logs/build_20261027_003417.log`. Exit checks: `rg` finds no script write of `disabled` in `assets/*.js` (the guardrail `guardrails_scripts_write_no_disabled` found the 4 real writes red, `tmp/stage7/logs/red-guardrail-repo.log`); detection tests cover property and bracket writes, calls split over lines, and a constant that holds `"disabled"`; the convergence check passes with the two new parts. Red logs in `tmp/stage7/logs/`, mutation runs in `tmp/stage7/mutations/` and `tmp/stage7/mutations2/`. Review `tmp/review/stage7/review.md`. | **Two parts, not one (M1, owner decision A).** `#send-control` and `#autoplay-controls` replace `#action-controls`, so the DOM order is the visual order and the CSS `order` rules are gone; 16.67 checks Tab and the left-to-right order. The cycles field is enabled during a Job, and Autoplay takes clicks during Checking and Preview (L2, in the specs). A pending send looks disabled through CSS and is not `disabled`: Enter can send again and gets `409` (L6, accepted by the owner). The screen script holds the `hx-disabled-elt` controls of a request as in flight and forgets the stamp when a request began with a control the server rendered disabled, because htmx clears `disabled` at the end of every request. A swap that throws releases the hold on `htmx:onLoadError` (L5, 16.68); htmx does not clear `disabled` on that path, so the Action input stays disabled until a reload. The guardrail does not cover `Object.assign`, `setAttributeNS`, names built at run time, markup strings or inline handlers (L1). |
| 7b | `5f0703fb` | Review, with a whole-client check | Full gate `logs/build_20261027_022053.log` before the review fixes; after them `logs/build_20261027_025314.log` (286 guardrail, 1484 integration, 40 browser), browser tier twice more (`_025718`, `_025957`). Exit checks: `rg` finds none of the 16 names that S1–S5 remove in `assets/`, `tests/`, `src/` or `docs/`, apart from this plan's history text; the convergence check has no per-part rule. Dashboard JS 2,070 lines at `a4c0f7dd`, 1,962 after (`dashboard-screen.js` 502 → 410). Red logs in `tmp/stage7b/logs/`; 44.40 and 16.70 passed on the old code, so their red evidence is mutation runs (`tmp/stage7b/mutations/`: m1–m9b, f8), each failing at its own assertion; the new 44.15 line fails on `d0065a53`'s scripts (`f9-head.log`). Review `tmp/review/stage7b/review.md`: F1 (option A, a P1 sentence), F2–F10, F12 fixed. Whole-client check: 26 unnamed mechanisms, 11 class (a), 11 class (b) now named by plan sentences (P1, P7, P9, P13, P15, P17, S5, §3, §6, §7, §8), 1 class (c) (C8, deferred), 3 class (d) deleted. | The Options regen's `hx-on` sat on the button, but htmx fires request events on the form, so the regen never showed a pending state; it now does (S5). The regen form reports its own `409` and is excluded from saving forms; that exclusion has no check that fails without it, because the regen report lands last. The status-display part is empty until its first load, and the announcer starts at `Ready`, so a page that opens during a Job announces its step; no scenario states this. 16.60 deleted; 16.69 and 16.70 added; 44.24 changed with 44.27; 16.41 and 16.42 lost their class lines (S4). Stub flag `generation_requests_accepted`. `stub_server.rs` is at 1,934 of 2,000 lines. Review F11 (info, inferred): if a load starts before a wait and a restarted server answers it, the wait lasts until the next revision move; a load in flight fails at the network level when the server stops, so this probably cannot occur; not fixed. A skipped opener can differ from the server render until the next revision move (P1; option B, one extra load per dialog open, not chosen). |
| 8 | `f3eb700e` | Light | Full gate `logs/build_20261027_193554.log` (286 guardrail, 1494 integration, 40 browser), browser tier twice more (`_194048`, `_194346`). Exit checks: `tests/http/games_pages.rs` (20.41–20.45, 45.15) with 1 active and 51 saved Games finds 50 rows on page 1 and 1 row on page 2; `validate-docs` passes. Red logs in `tmp/stage8/logs/`; storage cases written with the code; mutations m1 (no delete in `leaveSection`, 27.9) and m2 (no dropped answer after a page click, 27.10) fail at their own assertions. Docs of section 9: `dashboard.md` checked line by line against the code after 7b, `dashboard_design.md` rewritten, the revision in `storage.md`. Dashboard JS 1,977 lines. | The stage stopped once before any edit: the plan named no carrier for the page number (stop condition 4); the owner chose A and answered the two small points (O11). A load that started before a page click drops its answer, as an answer for a hidden tab does (P21). `games_page` that is not a whole number of at least 1 answers `400`, as an unknown part name does. The 13 part renderers take `&ScreenView` in place of `Option<GameId>`. The convergence check passes the page number. Not checked: focus after a click on Next on the last page (final gate item 12). `data-games-page` stays at 2 after a delete clamps the list to page 1 (harmless). `stub_server.rs` is at 1,947 of 2,000 lines. |

## Done when

- The browser has no table of kinds and regions. `data-live`, `data-live-state`, `window.chroniclerLive` and `dashboard-live.js` are gone.
- Any change loads the visible screen in one request, and a closed tab holds no content.
- The convergence check runs after every browser check and passes.
- No script writes `disabled`, and the guardrail enforces it.
- The names that S1–S5 remove are gone, and each client mechanism is named in this plan (stage 7b gate, Final gate step 1).
- A busy refusal answers `409`, and no code tests the string `class="status wait"`.
- A Game-bound action from a tab whose Game is no longer active is refused, and its text stays.
- "Unreachable" shows only after a network error.
- No draft control is in any part, and every form that waits for a Save opens in the dialog.
- An LLM Messages list load sends headers only. The Games list has pages.
- Every scenario in the Blast radius section passes at its tier, and the full gate is green.
- Every stage has a row in the Gate log, and the final gate's report is with the owner.

## Risks

- **The cost of a screen load.** Any change re-renders the visible tab. During Autoplay on the Game tab, that is about one load each second (inferred). Stage 6 measures it first, with a stop condition.
- **Changes the screen does not show.** Autoplay in another Game moves the revision, so the visible tab loads with no visible change. The morph keeps every node, so the player sees nothing. Stage 6 measures the cost.
- **One render, not one database snapshot.** The parts render one after another. A write during the render shows in the next load (see Failure modes).
- **Slower tier-3 tests.** The real shell asks every 1 s, so a tier-3 wait can take up to 1 s.
- **The CSS-only disabled Save (P18)** shows the state to the eye, but not to assistive technology. A keyboard save gets the refusal text.
- **False mismatches in the convergence check.** The normalisation list is short and named. A new client-owned attribute needs a row in it.
- **CSS-only busy states (S2, L6).** During its own request, and in the pending window, Send and Autoplay look busy but are not `disabled`. Assistive technology gets no signal; `hx-sync="this:drop"` and the `409` are the defence.
- **One extra load after an action that wrote nothing (S1),** such as a connection Test. The stamp is forgotten, so the next ask loads the screen once.
- **Modal dialogs change how the player works** (O1). The UI pass in stage 8 checks the layout.

## Not in scope

- Server-Sent Events. The loop can move to a push later, on the same revision.
- One URL for each screen.
- A "changed elsewhere" check for drafts.
- Collapsing the per-kind numbers into the one revision, and the per-kind fields of `GET /changes`. After stage 6 no browser code reads the per-kind fields; only the tier-1 tests do. Per-tab Games (ticket 01) decides their future, because a tab that shows one Game may need per-Game numbers.
- Pages for Worlds, presets and connections. The player writes each one by hand, so they stay short (2 Worlds and 8 presets in the probe database).
- Moving the tier-1 tests from the part routes to `GET /screen`. With O7, `GET /screen?parts=story-log` answers the same part as `/fragment/story-log`, so the part routes can go later.
- Per-tab Games (ticket 01). P14 prepares it.
- A save of a message edit while a Job runs (O3).
- The three different 4xx/5xx classifiers (review finding M4), the remaining size of `stub_server.rs`, the `Tx::conn` gap, and the low review findings.
