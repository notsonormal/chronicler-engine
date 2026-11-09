# Scan 2: driving adapter (src/adapters/driving/ and its front end assets/)

Scope note (known): the Rust side of the adapter holds no mutable state. `AppState` (`app_state.rs:28`) has only services (no Mutex, RwLock or Atomic). Every handler is a read or a call into `GameWriter` / catalogues. Almost all time-varying state of this adapter sits in the browser scripts `assets/dashboard-*.js` and `assets/index.html`, which `chat_window::index_handler` serves (`chat_window.rs:15`). I scanned them as part of the adapter. I did not read `.scratch/state-machines-*` or `tmp/state-machines/` (other than writing this file).

Line numbers: Rust lines come from my reads; JS lines are from rg. "unsure" is marked where it applies.

---

### 1. Screen sync protocol (ask loop, revision stamp, waiters)
- Where: `assets/dashboard-screen.js` (vars `askChain`:342, `asksPending`:343, `queuedAsk`:344, `loadsStarted`:190, `screenWaiters`:191, `dataset.revision`, `dataset.game`, `dataset.gamesPage`, `data-screen-busy`). Server side: `request_layers.rs:19` (`stamp_request_answers`), `screen.rs:60` (`screen_handler`), `layout/handlers/endpoints.rs:30` (`changes_handler`), `domain/model/change.rs` (`ChangeVersions`, `stamp()`).
- State variables: `screenRoot.dataset.revision` (present = "the page shows this stamp"; absent = "forgotten, stale"), `queuedAsk` (Promise or null), `asksPending` counter, `askChain` promise tail, `loadsStarted` counter, `screenWaiters[]` (stamp + `loadsStartedBefore` + resolve), `data-screen-busy` attribute.
- States (implied, never named): NO-STAMP (first load, or forgotten) -> SHOWING(stamp) -> STALE (server revision differs) -> LOADING(load n) -> SHOWING(new stamp); plus UNREACHABLE (see 8); plus WAITING (one or more waiters pending on a stamp the page does not show yet); plus RESTARTED (stamp has another `server` id).
- Transitions:
  - tick / visibilitychange / tab click / end of any request -> `askNow()` -> runAsk (`dashboard-screen.js:309,351,372`). `askNow` collapses callers into one `queuedAsk` that has not started yet.
  - ask answer value-equal to `dataset.revision` -> stay SHOWING, `clearScreenError()` (`:331`).
  - answer differs (any difference, also a lower number after a restart) -> `loadScreen()` (`:249`) -> SHOWING via `morphParts` (`:282`, writes `dataset.game` from `X-Game`, `dataset.revision` from `X-Revision`, then `resolveScreenWaiters`).
  - answer to a load is dropped when the visible tab or `gamesPage` changed during the fetch (`:277`).
  - end of every non-GET request -> `forgetScreenStamp()` (`:93`): SHOWING -> NO-STAMP. Also: Games pager click (`:384`), `leaveSection` (`:400`), `clearEditState` (story-log:101).
  - `afterScreenShows(stamp)` (`:193`): resolves at once when `screenShows`; else queues a waiter. A waiter resolves when same server id and `have >= want`, or when the server id differs and a load that started after the wait began has finished (`:201`).
- Guards / invariants: stamp is `<server>:<revision>`, revision only rises inside one server (`change.rs`); the screen stamp is read BEFORE the render (`screen.rs:~78`), so parts show at least that revision; the POST stamp is read AFTER the handler (`request_layers.rs:24`). Invalid: `dataset.revision` set while the DOM was cleared (`leaveSection` guards this by forgetting the stamp).
- Explicitness: FULLY IMPLICIT (several module-level variables plus data-attributes; no enum, no one transition place).
- Already modeled: no.
- Risks seen:
  - A waiter has no timeout and no cancel. If the engine is unreachable, or a load never succeeds, every waiter stays pending: `sendPending` (3), the "Saving…" dialog (5), the edit lock (6) and the Saved flag (13) all depend on it. (known: read; consequence: inferred.)
  - Parts of one load are rendered from separate registry reads (`busy_of_game`, `is_busy`, `autoplay_holds`, `status` are each called per part, `screen/handlers/parts.rs`), so one load can show a mixed busy state (e.g. Send enabled, status display busy). The pre-render stamp makes the next ask reload, so it self-heals within one poll interval. (known: code; effect inferred.)
  - `loadScreen` drops an answer after a tab change without recording anything. It relies on "that change has already asked again" (`:248`). If `askNow` returned the same queued ask, this holds; I did not prove it for all orders. (unsure.)
  - Two order-dependent handlers on `htmx:afterRequest`: element `hx-on` handlers vs the body listener `endRequest` (forget stamp). Today either order is safe. (inferred.)

### 2. Request-in-flight registry and the keep rules of a morph
- Where: `assets/dashboard-screen.js:72` (`requestsInFlight` Map), `:78-118` (hold/release, `loadKeeps`, `beforeNodeMorphed`, `beforeNodeRemoved`), `data-keep` attribute handling `:57-64`.
- State variables: `Map<node, count>`; `data-editing` on an entry; `data-keep` lists on nodes.
- States: node IDLE / IN-FLIGHT(n) / EDITED; implied "kept across load" for in-flight and edited nodes.
- Transitions: `htmx:beforeRequest` -> hold (`:80`); `htmx:afterRequest` or `htmx:onLoadError` -> release (`:93`). Morph skips and refuses to remove a node while held or edited.
- Guards: every hold needs one release.
- Explicitness: FULLY IMPLICIT.
- Already modeled: no.
- Risks seen:
  - `htmx:afterRequest` is dispatched on the element. If a script detached the element during the request (`leaveSection` does `part.replaceChildren()` at `:~403`; the dialog `close` handler does `editDialogBody.replaceChildren()`, `dashboard-dialog.js:87`), the event may not bubble to `body`. Then the entry is never released and the handlers of 8, 5 and 13 do not run for that request. (inferred; I did not test htmx behaviour.)
  - `htmx:sendError`, `htmx:timeout` and `htmx:responseError` are not listened to. This is safe only if htmx also fires `afterRequest` for them. (unsure.)

### 3. Action area (command form, preview, pending send)
- Where: `assets/dashboard-action-area.js`: `checkInFlight`:5, `sendPending`:10, `previewWasOpen`:89, DOM `#action-preview` content, `#status-display [data-status]`, `#action-area[data-state]`. Server: `action/handlers/actions.rs:28,50` (`dispatch_action`, `action_check_handler`), `layout/handlers/action_controls.rs`, `layout/handlers/endpoints.rs:70-110` (`data-status` values `idle|thinking|autoplay|failed`).
- State variables: `checkInFlight` (bool), `sendPending` (bool), preview open (DOM presence), status display `data-status` (from server), `data-state` (derived output).
- States (explicit as strings, `deriveActionState`:64): `checking` > `autoplay` > `generating` > `preview` > `idle`; the priority order is the only rule. Implied: PENDING-AFTER-ACCEPT (sendPending true, status display still idle), which the comment at `:7-10` names.
- Transitions:
  - command form before-request -> checking (`onCommandBeforeRequest`).
  - form after-request: `done` and preview closed -> clear input after screen shows, `beginPendingUntilScreenShows` (`onCommandAfterRequest`:270); `done` and preview open -> preview state; `refused` -> retake draft game (4).
  - preview send / autoplay start / retry / retrigger / new options: `beginPending...` and `endPendingAfterRequest`:306 (`done` -> wait for screen; otherwise end at once).
  - `endPending` clears `sendPending` and re-derives.
  - `screen:loaded` -> `syncActionState`.
- Guards / invariants: the server renders `disabled` for Send, Autoplay, Options buttons and posture selects from the busy state; the script never writes `disabled` (comment `:62`). CSS (`styles.css:693-697`) dims Send and Autoplay in state `generating` or while a form has `htmx-request`.
- Explicitness: EXPLICIT STATES (strings), derived in one function; the inputs (`checkInFlight`, `sendPending`) are set in ~10 scattered handlers.
- Already modeled: no.
- Risks seen:
  - The `/action/check` answer has two meanings under the same status 200: "a Job started" (empty body) or "here is a text-check preview" (HTML body). The script tells them apart by testing whether the preview element has a child after the swap (`onCommandAfterRequest`:270 uses `previewIsOpen()`). (known: read.)
  - `sendPending` can stay true: `endPendingAfterRequest` on `done` waits for `afterScreenShows` (see 1). The retry, retrigger and new-options controls have no `hx-on::on-load-error`; only `#command-form` has it (`index.html`, `onCommandLoadError`). A swap error there would skip after-request and leave `sendPending` true. (inferred.)
  - `useOption` refuses when `checkInFlight || actionIsGenerating()` but `editOption` only checks `checkInFlight` (`:~323`). Probably intended. (guessed.)

### 4. Draft Game pin (Game header protocol)
- Where: client `dashboard-screen.js:~405-428` (`htmx:configRequest` adds `X-Chronicler-Game`), `dashboard-action-area.js` (`syncCommandDraft`, `startNewCommandDraft`, `retakeDraftGameWhenScreenShows`), `dashboard-story-log.js:~92` (`data-draft-game` on an entry in edit). Server: `request_layers.rs:39` (`refuse_stale_game`), `utils/current_game.rs:17` (`CurrentGame`), `screen.rs` (`X-Game` answer header), `router.rs:20-54` (`game_bound_routes`).
- State variables: `screenRoot.dataset.game` (the Game the screen shows; "" when none), `data-draft-game` on `#command-form` or an entry (pin; absent = no draft), the server's active-Game pointer.
- States: NO-DRAFT, DRAFT(pinned game g); screen game = none or g'; implied: PIN-MATCHES / PIN-STALE.
- Transitions: first typed character -> pin to the screen's Game (`syncCommandDraft`); empty input -> unpin; Option click -> new draft; `409` refusal -> re-pin to the Game the screen shows once the screen shows the answer; edit opened -> pin entry; request sent -> header = pin, else the screen's Game, else no header. Server: header present and not the active Game -> `409 GameChanged` before the handler.
- Guards: the check is a route layer, not an extractor, because `Option<CurrentGame>` would turn the rejection into `None` (comment `request_layers.rs:32-36`).
- Explicitness: EXPLICIT (Rust: one layer) / FULLY IMPLICIT (client pin attribute). Mixed.
- Already modeled: no (the active-Game pointer itself is).
- Risks seen:
  - When `dataset.game` is "" (screen shows no Game) the header is omitted (`if (game)`), so the request acts on whatever Game is active by then. (known: read.)
  - Routes outside `game_bound_routes` (`/games/:id/mode`, `/posture`, `/presets`, `/games/:id/switch|delete|rename`) receive the header from the script but are not checked by the layer. (known.) They name the Game by path id, so this is probably intended.
  - The pin of an edit entry is set once and never retaken after a refusal, so a Save after the active Game changed stays refused until Cancel. (inferred.)

### 5. Edit dialog lifecycle (modal form session)
- Where: `assets/dashboard-dialog.js`: `dialogOpenerId`:29, `dialogSession`:60, `noticeSummary`:61, `unreachableText`:62, `<dialog open>`, `data-saving` attribute. Server fragments render into `#edit-dialog-body` (games/worlds/presets/connections/LLM record templates carry `data-dialog-form`, `data-close-dialog`).
- State variables: `editDialog.open`, `data-saving`, `dialogSession` (integer), `dialogOpenerId`, plus the two summaries for the status line.
- States: CLOSED, OPEN(form), SAVING (open + `data-saving`), implied: OPEN-WITH-NOTICE / OPEN-UNREACHABLE (status line only).
- Transitions: `htmx:afterSwap` into the body -> open (`:31`); `htmx:beforeSwap` into an open dialog from outside it -> swap refused (`:43`); form `afterRequest` with `done` -> SAVING and wait for the screen, then close if the session is unchanged (`:115`); `htmx:confirm` on a form while SAVING -> prevented (`:107`); Cancel / Escape / `close` event -> CLOSED, session+1, body emptied (`:87`).
- Guards: only a FORM element (not a button such as the connection Test) enters SAVING; session id stops a late save closing a newer dialog.
- Explicitness: FULLY IMPLICIT (attribute + counter, no named states).
- Already modeled: no.
- Risks seen:
  - SAVING has no exit but the screen showing the stamp, or the player closing the dialog (Cancel works). While SAVING the form is inert. (known: read.)
  - A form in the dialog that is submitted with `done` but whose handler is a validation-like `200` (e.g. the connection Test answers 200 with a failure disclosure) is a button, so not affected. A real FORM that answers 200 with an error body would enter SAVING. I found no such route. (inferred.)
  - Opening a second dialog while the first is SAVING: `afterSwap` does not reset `data-saving`; only `close` does. Blocked by the modal in practice. (inferred.)

### 6. Story-log entry edit mode and its lock
- Where: `assets/dashboard-story-log.js`: `editState`:17 (`null` or `{id, text}`), `data-editing` on the entry, `data-editing-entry` on `#story-log` root, `data-draft-game`, CSS `styles.css:1456,1558`, `storyLogSwapStart`:184. Server: `templates.rs:~20` `NarrationLogTemplate` (edit/delete/retrigger/swipe buttons, `view_only` -> `disabled`).
- State variables: `editState`, `data-editing`, `data-editing-entry`, server `view_only`.
- States: NOT-EDITING; EDITING(entry id); SAVED-AWAITING-SCREEN (editState null, entry attr removed, root lock still set); implied: STALE-EDIT (entry kept across loads while its Game changed).
- Transitions: Edit click -> EDITING (`showEditForm`:59, refused when `editState !== null`); Escape / Cancel -> `revertEdit`:113 -> NOT-EDITING (`clearEditState`:101, unlock); Save `done` -> `clearEditState`, lock released after `afterScreenShows` (`onEditSaved`:135); Save `refused|invalid` -> stay EDITING; Save other outcome (`failed`, `none`) -> `revertEdit` (text lost); tab leave -> `screen:cleared` handler resets (`:~245`).
- Guards: click guard in the capture phase blocks edit and swipe buttons while the root lock is set (`:~42`); a load skips the entry in edit (`beforeNodeMorphed`).
- Explicitness: FULLY IMPLICIT.
- Already modeled: no.
- Risks seen:
  - On a `failed` (500) or unreachable save, `revertEdit` discards the typed text, while on `refused` and `invalid` it keeps it. (known: read; may be intended.)
  - A `done` Save locks all edit and swipe controls until the screen shows the stamp (see 1). (inferred.)
  - The Save button is only styled as disabled (CSS `:has(status thinking|autoplay)`), and the click still reaches the engine, which refuses with 409. (known: comment in CSS.)

### 7. Answer outcome classification (status code <-> outcome)
- Where: Rust `utils/answer_words.rs:16` (`RefusalKind`), `:45` (`AnswerOutcome`), `:265` (`describe_answers`), `utils/error.rs:80` (`answer_for_error`), `utils/response.rs` (`refused`, `bad_request`, `internal_error`); JS `dashboard-errors.js:94` (`outcomeOfStatus`, `REFUSED_STATUS = 409`), `requestAnswer`.
- State variables: HTTP status; response extension `ErrorAnswer`; header `X-Revision`.
- States: done (<400), refused (409), invalid (other 4xx, incl. 404), failed (5xx), none (status 0).
- Transitions: Rust error -> status (`answer_for_error`: Validation -> 400, `StillThinking` and `LastMessageChanged` -> 409, `BadRequest` -> 400, `Failed` -> 500, `WorldHasGames` -> 400, others -> 500); `describe_answers` writes the first line from route words; JS maps status -> outcome and picks a slot (`invalid` -> inline form slot; others -> notice).
- Guards: a route with no entry in `ANSWER_WORDS` keeps its raw body (`describe_answers`: `let Some(words) ... else return response`).
- Explicitness: EXPLICIT on the Rust side (`AnswerOutcome`), mirrored by hand in JS. TWO PLACES THAT MUST AGREE.
- Already modeled: no.
- Risks seen:
  - Outcome is carried only by the status code across the wire. The mapping exists twice (Rust enum, JS function). A new status (e.g. a 422 or a 3xx) would map to `invalid` or `done` in JS without a Rust change. (inferred.)
  - `bad_request` and `internal_error` in `response.rs` build bodies with no `ErrorAnswer`; `describe_answers` rebuilds them through `ErrorAnswer::unbuilt` (known: read). `screen_handler` uses them, but `/screen` has no `ANSWER_WORDS` row, so its failure text is not rewritten. (known.)
  - `refuse_stale_game` returns 409 on `/debug/is_busy` (a GET); no JS reads that route. (known, harmless.)

### 8. Notice place, failure banner, unreachable flag, inline slots, details popovers
- Where: `assets/dashboard-errors.js`: `#notice[hidden][data-source]` (`showNotice`:131, `clearNotice`:141), `#failure-banner` with two children (`failure-banner-degraded` is a server part, `failure-banner-unreachable` is client text) (`refreshFailureBanner`:49, `markUnreachable`:64, `clearUnreachable`:78), inline slots `[data-error-slot][hidden]`, popovers `hidden` + `aria-expanded` kept by `data-keep`. Server: `builders/headers.rs:~150` (`failure_banner_html`), `app_state.rs:~92` (`render_failure_banner`).
- State variables: `notice.hidden`, `notice.dataset.source` (`request` or `screen`), `unreachable.innerHTML` (empty = down-flag clear), `degraded.innerHTML`, `banner.hidden`, `banner.classList.unreachable`.
- States: notice NONE / REQUEST-REPORT / SCREEN-REPORT; banner HIDDEN / DEGRADED / UNREACHABLE / BOTH (UNREACHABLE wins the class and the `role`).
- Transitions: request answer `none` -> `markUnreachable`; any other answer or successful fetch -> `clearUnreachable`; non-done answer -> inline slot (`invalid` with a slot) or `showNotice(request)`; any `htmx:beforeRequest` -> `clearNotice` (also clears a screen-source report); screen failure -> `showNotice(screen)`; screen success -> `clearScreenError` clears only `screen` source; close button -> `clearNotice`.
- Guards: `noticeShows` skips a rewrite of the same text, to keep an open Details and not re-announce.
- Explicitness: FULLY IMPLICIT (DOM emptiness and `data-source` encode the state).
- Already modeled: no.
- Risks seen:
  - "Unreachable" is set by an XHR with status 0. A request that the page aborts (element replaced, navigation) also has status 0, and would mark the engine as unreachable until the next success. (inferred; I did not test.)
  - Any new request (also a background auto-save) clears a report that the player has not read. (known: `dashboard-errors.js:~166`.)
  - Stale banner: the server part for Degraded is rewritten only on a load; `refreshFailureBanner` runs on `screen:loaded`, so it is consistent. none seen beyond that.

### 9. Busy state to controls (status display, Send, Autoplay, Stop, view-only lists)
- Where: `layout/handlers/action_controls.rs:18,31,46` (`send_control_answer`, `autoplay_controls_answer`, `busy_of_game`), `layout/handlers/endpoints.rs:50-110` (`render_status_display`, `busy_answer`, `failure_answer`), `application/games/view_query.rs:32` (`Status`: Busy > Failed > Idle), `app_state.rs:~85,~125` (`autoplay_holds`, `is_busy`), `games/handlers/games.rs:~253` (`posture_controls_html`, `view_only`), `templates.rs:~20` (`view_only` on story-log buttons), `view_models.rs:285` (`OptionsDockViewModel.is_busy`).
- State variables: `Option<Busy>` from the registry (`Busy::Job(step)` / `Busy::Autoplay{progress, step}`), `Game.failure` (Option) in storage.
- States: Idle, Failed(line), Busy::Job(step), Busy::Autoplay{cycle, step}. Controls: Send disabled when any busy; Autoplay disabled when any busy; Stop exists only for `Busy::Autoplay`; posture/preset selects and story buttons disable only for Autoplay (`view_only`), not for a plain Job; Options dock buttons disable for any busy.
- Transitions: none in this layer (it reads). The rule "busy first, then failure, then idle" sits in `Status` (`view_query.rs:200`).
- Guards: `Failed` and unreadable Game render as idle for the controls (`busy_of_game` `Err(_) => None`), while the status display shows the failure line.
- Explicitness: EXPLICIT (`Status`, `Busy`), consumed in 5+ render functions that each map it again.
- Already modeled: yes for the gate and `Busy`; the mapping to controls is not in the model.
- Risks seen:
  - Each part calls the registry separately; see risk in 1 (mixed views in one load). (known.)
  - Rules for "what is disabled when" are duplicated: Send/Autoplay in `action_controls.rs`, dock in `OptionsDockViewModel`, selects and story buttons in templates through `view_only`. The Job vs Autoplay difference (`is_busy` vs `autoplay_holds`) is easy to get wrong in a new part. (inferred.)
  - The config routes (`/games/:id/mode|posture|presets`) write with no busy check (comment in `dock.rs:6`), so the UI lock (`view_only`) is the only guard against a config change during a plain Job. (known: read comment; effect inferred.)

### 10. Options dock content
- Where: `domain/model/dock.rs:9` (`DockRecord { offered, failure }`), `templates.rs` (`OptionsDockTemplate`), `view_models.rs:285` (`OptionsDockViewModel`), `app_state.rs:~125` (`render_options_dock`); writers `application/generation/gate.rs:~838` (`save_options`, `put_dock`, `put_options_failure`), `adapters/driven/storage/transaction.rs:300-325`.
- State variables: `offered: Vec<String>` (empty = none), `failure: Option<OptionsFailure>`, missing row (= default), `is_busy` (live).
- States: EMPTY (no row, or empty offered and no failure), OFFERED, FAILED (failure only), implied: FAILED-WITH-STALE-OFFERED (failure set and offered non-empty).
- Transitions: Options success -> `put_dock(offered, failure None)`; Options failure -> `put_options_failure` (keeps `offered` in the row, `transaction.rs:307`); Narration text save empties the set (comment `gate.rs:~836`); `clear_dock` -> EMPTY.
- Guards: template precedence: failure wins, else a non-empty offered set, else empty body (`templates.rs` `{% if vm.failure.is_some() %}`).
- Explicitness: FULLY IMPLICIT (two fields; the template precedence is the rule).
- Already modeled: partly (the Job writes it; the combination rule is not).
- Risks seen:
  - The combination "failure set and offered non-empty" is reachable by design (`put_options_failure` keeps the set). The template hides the offered set. `get_debug_state` returns `current_options: dock.offered` regardless of the failure. (known: read.)

### 11. Role health (Failed / Succeeded / NoCalls)
- Where: `builders/headers.rs:55-70` (`RoleHealthState`, `classify`), `:111` (`role_health_cell`), `:136` (`connections_degraded`), `:159` (`failure_banner_html`), `application/games/view_query.rs:249` (`role_health`), `adapters/driven/storage/llm_messages.rs:11,41` (retention 50 + prune).
- State variables: derived per Role from the newest LLM message row of that agent: `error_message` Some/None, `backend_name+model_name` Some/None. Nothing stored.
- States: Failed(raw), Succeeded, NoCalls (no row).
- Transitions: a new LLM message row of the Role (error or ok) -> new state; prune of old rows -> may drop the newest row of a quiet Role.
- Guards: `connections_degraded` looks only at Narrator and Quantifier (`SETTINGS_ROLE_ROWS`); the banner lists all four Roles.
- Explicitness: EXPLICIT enum (`RoleHealthState`), derived from storage by one function; but the transition source (retention) is elsewhere.
- Already modeled: no.
- Risks seen:
  - Health is lost by retention, not by a success. `save_llm_message` keeps only 50 rows (`llm_messages.rs:11,41`). If an Options or Trigger call failed and then 50 later calls of other Roles ran, that failure row is deleted and the Role reads `NoCalls`. The banner then clears with no success. The doc comment on `role_failure_sentence` (`headers.rs:~37`) says the failure lasts "until the role's next call succeeds". (known: read both; consequence is a clear reading of the code.)
  - `role_health` picks the newest row by `(created_at, id)`, while the prune keeps by `created_at DESC` only. With equal timestamps the two orders can differ. (inferred, low impact.)

### 12. Saved-games pager and the games page
- Where: client `dashboard-screen.js:7,31-47,384` (`GAMES_PAGE_KEY`, `dataset.gamesPage`, sessionStorage); server `screen.rs:26` (`games_page_named`), `games/handlers/games.rs:28` (`render_games_panel`), `games/templates/games.rs:20-43` (`GamesPagerView`: `previous_disabled`, `next_disabled`, `aria-disabled`), `domain/model/saved_games_page.rs` (`clamp_page`), `application/games/catalogue.rs:105`.
- State variables: `dataset.gamesPage` (string, absent = page 1), sessionStorage copy, server-side clamped `page`, `page_count`.
- States: page n of N; implied: REQUESTED-PAGE-PAST-END (clamped).
- Transitions: Previous/Next click -> set `gamesPage`, remember, forget stamp, ask; delete/switch of a Game changes N; server clamps a page past the end.
- Guards: `aria-disabled` buttons ignore clicks; the answer of a load is dropped if `gamesPage` changed meanwhile.
- Explicitness: FULLY IMPLICIT on the client; server rule is one pure function.
- Already modeled: no.
- Risks seen:
  - The client never learns the clamped page: the answer has no page header (only `X-Game`, `X-Revision`). After the last Game of the last page is removed, `dataset.gamesPage` and sessionStorage keep the old number, so each load re-clamps; the pager buttons carry correct targets, so the user can recover. (known: read `morphParts`; effect inferred.)
  - The page is sent also on loads that do not include the Games part (`games_page=` is always appended when set). Harmless. (known.)

### 13. Autosave controls: rendered-value baseline and the Saved flag
- Where: `assets/dashboard-saves.js` (`settleField`, `restoreRenderedValue`, `flagSaved`, `settleSave`:~50), `data-saved` on `[data-saved-flag]` target, `data-autosave` on forms; templates `settings.rs` (`TextCheckCardTemplate`), `games.rs` (`GamePostureTemplate`), `worlds.rs` (posture selects, `world-posture-status`).
- State variables: `defaultValue` / `defaultChecked` / `defaultSelected` of each field (used as "last accepted value"), `data-saved` attribute on the flag.
- States: field CLEAN (value = default), DIRTY (changed, request in flight), SAVED (flag shown), REVERTED (after a failed save).
- Transitions: change -> request -> `done`: `settleSavedFields` then flag after the screen shows; any other outcome: flag removed, field restored to the default (a form restores only with `data-autosave`).
- Guards: only elements inside `[data-part]` or `#edit-dialog`.
- Explicitness: FULLY IMPLICIT.
- Already modeled: no.
- Risks seen:
  - A morph (load) that rewrites a field while a save is in flight is stopped by 2. After a later load the server value wins over `defaultValue` (morph writes the attribute, not `.defaultValue`?). unsure.
  - `clearPostureStatus` is a separate hand-made reset of the flag (`worlds` form `hx-on::before-request`), a second place that writes `data-saved`. (known.)

### 14. Form modes: World, Preset, Connection, Text check, Game posture
- Where & states:
  - World form `worlds/templates/worlds.rs:139-240` (`is_edit`, `is_readonly`, `auto_save_key`, `form_action`, `submit_text`, all derived from `world: Option<&WorldCard>`): modes CREATE / EDIT. EDIT: key readonly, posture selects autosave in a separate fieldset, "Cancel applies to details only" note. CREATE: posture inside the form, no autosave.
  - Preset forms `builders/presets.rs`: add form (by type), view (`is_default`), edit (not default), card buttons per `is_default` and active-slot (`mode_slot_actions`, `options_slot_actions`). `edit_preset_form_handler` calls `ensure_editable` (`prompt_presets.rs:~199`).
  - Preset allowed modes: two booleans `allowed_mode_novel`, `allowed_mode_if` (`PresetForm`, `prompt_presets.rs:115`). Both false means "not offered": CREATE -> default modes; UPDATE -> keep (`update_preset_handler`:~190). A player cannot clear both on update.
  - Text check `settings.rs:322-335` (`TextCheckSettings`): `(mode, enable_auto_check)`; `Disabled` + true is invalid and normalised on read and write (`normalised_auto_check`). The template disables the checkbox when mode is Disabled (`TextCheckCardTemplate`), and a disabled field posts nothing.
  - Connection form `ConnectionFormTemplate::new(Option<&LlmConnection>)`: `editing` derived from Option; `provider_order` (list) + `pin_provider_order` (bool): pin with an empty list is meaningless.
  - Game posture `GamePostureTemplate`: `preset_load_error: Option<String>` replaces the preset selects, `view_only` disables all; stored selection missing -> a "(missing) id" option is inserted (`view_models.rs:~100`).
- Transitions: all are "open dialog from a GET fragment" (Some/None of the record) then POST; the mode is fixed per dialog.
- Explicitness: derived from `Option` presence; no explicit mode type.
- Already modeled: no.
- Risks seen:
  - Text check: the select change posts the form, the checkbox is still `disabled` in the DOM until the next load, so switching Disabled -> Spell posts `enable_auto_check` absent = false, and auto check turns off silently. (inferred from the template and the handler; I did not run it.)
  - Both-false allowed-modes sentinel: on update the UI cannot express "no mode allowed" and a stored selection with no mode cannot be saved. (known: handler comment `prompt_presets.rs:~190`.)
  - `pin_provider_order` true with an empty `provider_order` is accepted by `into_config` (no check). Effect: `allow_fallbacks: false` with no order (`chat_completions.rs:~102`). (known: read; effect on the provider is unsure.)
  - World posture autosave does a read-modify-write of the whole World (`update_world_posture_handler`: get, patch, `update_world`); the world form and the posture selects can race. (inferred.)

### 15. Tabs and Settings sub-tabs
- Where: `assets/dashboard-panels.js:9-106`, `dashboard-screen.js:400` (`leaveSection`), `index.html` tab bar; sessionStorage keys `chronicle.activeTab`, `chronicle.settingsSubtab`; `settings.rs:~25` sub-tab buttons with `aria-selected` and panels with `hidden` (both in `data-keep`).
- State variables: `.tab.active` + `.tab-content.active` classes, `aria-selected`, `hidden` on sub-panels, sessionStorage.
- States: one of 6 tabs active; one of 2 Settings sub-tabs; implied: section CLEARED (parts empty after leaving) vs LOADED.
- Transitions: tab click -> classes switch, `leaveSection(old)` empties its parts and fires `screen:cleared`, `askNow()` loads the new tab; reload -> `restoreActiveTab`; Settings load -> sub-tab restore (`:~40`).
- Guards: only parts of the visible section are loaded (`visibleParts`).
- Explicitness: FULLY IMPLICIT (CSS classes and attributes).
- Already modeled: no.
- Risks seen:
  - Leaving the Game tab empties the story-log, so an open edit, an entry in flight or the focus is dropped (handled in 6 with `screen:cleared`). The announcement baselines in 17 are not reset there. (known.)
  - `DOMContentLoaded` runs `restoreActiveTab()` before the tab click handlers are attached; first load asks twice (`askNow` from `activateTab` and from the screen script) but `askNow` collapses them. (inferred, harmless.)

### 16. Slash menu
- Where: `assets/dashboard-slash-menu.js:8-10` (`slashMenu`, `slashMatches`, `slashActiveIndex`), `aria-expanded` and `aria-activedescendant` on the input.
- States: CLOSED / OPEN(matches, active index). Transitions: input event -> open/close by match; Arrow keys -> move; Enter -> select or close; Escape, submit, scroll, resize, mousedown outside, focusout (150 ms delay) -> close.
- Guards: the menu element exists iff OPEN (`slashMenu` null or node), mirrored in `slashActiveIndex = -1`.
- Explicitness: FULLY IMPLICIT (three variables; node presence = OPEN).
- Already modeled: no.
- Risks seen: none seen beyond a timer race: the `focusout` timeout (150 ms) can close a menu that the player reopened within that time. (inferred, low.)

### 17. Live-region announcement baselines
- Where: `assets/dashboard-action-area.js:110` (`announcedStatusLabel = "Ready"`), `:140` (`announcedEntryIds = null`), `:171` (`announcedOptions = null`), `dashboard-story-log.js:~85` (`restore-notice` timer via `scheduleNoticeClear`).
- States: first-time (null = take a baseline, announce nothing) vs later (diff against the baseline).
- Transitions: first `screen:loaded` containing the part -> baseline; later loads -> announce additions.
- Explicitness: FULLY IMPLICIT (null means "first load").
- Already modeled: no.
- Risks seen:
  - The baselines are not reset on `screen:cleared` or a Game change. After the Game is changed from the Games tab and the Game tab is opened again, the ids of the new Game differ from the old set, so every narration entry of the new Game (up to 50) is treated as new and read out. The same for options. (inferred from `announceNewNarration` and `leaveSection`; I did not run it.)

### 18. Story-log follow-scroll
- Where: `assets/dashboard-story-log.js:184` (`storyLogSwapStart`), `followStoryLogBottom` with a 1000 ms frame loop.
- States: NO-SWAP / SWAP-STARTED(atBottom, entries) / FOLLOWING (retry per frame until deadline).
- Transitions: `screen:beforemorph` -> record; `screen:loaded` -> consume, skip when editing, not at bottom, or no new entries; frame loop stops at deadline or on edit.
- Explicitness: FULLY IMPLICIT.
- Already modeled: no.
- Risks seen: `storyLogSwapStart` stays set if a morph throws between `beforemorph` and `loaded` (`morphParts` catch path); the next `beforemorph` overwrites it. none seen beyond that.

---

## Not state machines (checked)
- `AppState`: services only. `CurrentGame`: a per-request value, one read. `SCREEN_PARTS` (`parts.rs:28`): one table, a name with no renderer cannot exist. `RouteWords` / `ANSWER_WORDS`: a lookup table; a route missing from it only loses the rewritten first line (see 7).
- `GamesPagerView::new`: pure.
- Connection Test dialog and the Test button: stateless request/response (answers 200 with success or failure disclosure).

## Table of contents
name | file | explicitness
- 1 Screen sync protocol | assets/dashboard-screen.js, request_layers.rs | FULLY IMPLICIT
- 2 Request-in-flight registry and morph keep rules | assets/dashboard-screen.js | FULLY IMPLICIT
- 3 Action area (checking/autoplay/generating/preview/idle) | assets/dashboard-action-area.js | EXPLICIT STATES, SCATTERED INPUTS
- 4 Draft Game pin / Game header protocol | dashboard-screen.js, request_layers.rs, current_game.rs | MIXED (Rust explicit, client implicit)
- 5 Edit dialog lifecycle | assets/dashboard-dialog.js | FULLY IMPLICIT
- 6 Story-log entry edit mode and lock | assets/dashboard-story-log.js | FULLY IMPLICIT
- 7 Answer outcome classification | answer_words.rs, error.rs, dashboard-errors.js | EXPLICIT (Rust) + hand-mirrored (JS)
- 8 Notice, failure banner, unreachable flag | assets/dashboard-errors.js | FULLY IMPLICIT
- 9 Busy state to controls | action_controls.rs, endpoints.rs, view_query.rs | EXPLICIT (consumed in many places; gate already modeled)
- 10 Options dock content | dock.rs, templates.rs, gate.rs | FULLY IMPLICIT
- 11 Role health | headers.rs, view_query.rs, llm_messages.rs | EXPLICIT (derived; source scattered)
- 12 Saved-games pager | dashboard-screen.js, saved_games_page.rs | FULLY IMPLICIT (client)
- 13 Autosave baseline and Saved flag | assets/dashboard-saves.js | FULLY IMPLICIT
- 14 Form modes (World, Preset, Connection, Text check, Posture) | worlds.rs, presets.rs, settings.rs templates | FULLY IMPLICIT (Option presence, boolean pairs)
- 15 Tabs and Settings sub-tabs | assets/dashboard-panels.js | FULLY IMPLICIT
- 16 Slash menu | assets/dashboard-slash-menu.js | FULLY IMPLICIT
- 17 Live-region announcement baselines | assets/dashboard-action-area.js | FULLY IMPLICIT
- 18 Story-log follow-scroll | assets/dashboard-story-log.js | FULLY IMPLICIT
