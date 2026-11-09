### Action-area mode (checking / generating / autoplay / preview / idle)
- Where: `assets/dashboard-action-area.js:5` (`checkInFlight`), `:10` (`sendPending`), `:59` (`previewIsOpen()` reads `#action-preview`), `:40` (`statusShows()` reads `#status-display [data-status=…]`), `:89` (`previewWasOpen`), `:283` (`previewSentText`)
- State variables: `checkInFlight: bool`, `sendPending: bool`, presence of a child in `#action-preview`, the server-rendered `data-status` value (`idle|thinking|autoplay|failed`), plus `#action-area`'s `data-state` as the derived output
- States: `idle`, `checking`, `generating`, `autoplay`, `preview` — all derived, none stored as one value
- Transitions: `deriveActionState()` at `:64` is the only derivation; setters: `onCommandBeforeRequest` sets `checkInFlight` `:201`; `onCommandAfterRequest` `:270` and `onCommandLoadError` `:265` clear it; `beginPending` `:20` from `updateToThinking` `:195`, `onPreviewSendAfterRequest` `:299`, `onAutoplayAfterRequest` `:306`; `endPending` `:25` from `endPendingWhenScreenShows` `:31` and `endPendingAfterRequest` `:306`; preview open/closed by the server swap of `/action/check` into `#action-preview` and `closeActionPreview` `:310`
- Guards / invariants: `checking` and `generating` are meant to be exclusive (check happens before send); `autoplay` outranks `generating` in the derivation; `failed` is never named by the derivation, so a failed status display collapses to `idle`/`preview`
- Explicitness: FULLY IMPLICIT (5 sources, one pure derivation function, setters spread over 9 functions)
- Already modeled: no
- Risks seen: the guard of `useOption` `:319` tests `checkInFlight || actionIsGenerating()`, but `actionIsGenerating()` `:55` is `sendPending || statusShows("thinking")`, which is false while the display shows `autoplay` — so a click on an Option during an Autoplay run is only stopped by the server-rendered `disabled` on the button (known, mitigated). `endPendingAfterRequest` `:306` calls `endPending()` on any non-`done` answer, so a refused retrigger clears the pending state that an unrelated in-flight send set, because one boolean serves every sender (known).

### Screen freshness protocol (the revision stamp)
- Where: `assets/dashboard-screen.js:170` (`forgetScreenStamp`), `:174` (`splitStamp`), `:179` (`screenShows`), `:190` (`loadsStarted`), `:191` (`screenWaiters`), `:193` (`afterScreenShows`), `:201` (`resolveScreenWaiters`), `:282` (`morphParts` writes `dataset.revision` and `dataset.game`); server side `src/adapters/driving/http/utils/request_layers.rs:19` (`stamp_request_answers`), `:13` (`REVISION_HEADER`), `src/adapters/driving/http/screen/handlers/screen.rs:78` (stamp read before render)
- State variables: `document.documentElement.dataset.revision` (`"<server>:<revision>"` or absent), `dataset.game`, the array `screenWaiters` of `{stamp, loadsStartedBefore, resolve}`, the counter `loadsStarted`
- States: screen current (dataset.revision ≥ every awaited stamp), screen stale (no stamp at all after `forgetScreenStamp`), waiting (a promise parked in `screenWaiters`), and the restart case (stamp names another server id)
- Transitions: `morphParts` `:314` sets the stamp after a load; `endRequest` `:99`-`:100` and `loadKeeps` paths call `forgetScreenStamp` on the end of every non-`GET` request; the pager click `:394` and `leaveSection` `:400` also forget it; `askNow` `:351` polls `GET /changes` and loads when `answer.server + ":" + answer.revision` differs from the stamp
- Guards / invariants: a load may only claim a revision it rendered at or after — the server reads the stamp **before** the render (`screen.rs:78`), so a part can be newer than the stamp, never older. A waiter must be resolved by a load that *started* after it was queued, or by a server-id change
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (the two states are named only inside `screenShows`, the transitions live in 5 modules)
- Already modeled: no (the change counters themselves are in `domain/model/change.rs`; this client protocol is not)
- Risks seen: a waiter is only ever resolved from `resolveScreenWaiters` `:201`, which runs inside `morphParts`. While the engine is unreachable, `runAsk` `:363` returns early after `markUnreachable`, so a `sendPending` that waits on a stamp stays pending until reachability returns and a load lands (known — no timeout on the waiter, and `askTimeoutSignal` bounds only the fetch). `screenShows` `:179` compares with `>=`, so a load whose stamp was raised by a write that the load did not render still counts as "shows" for any older stamp (inferred; harmless for content, but it is why `forgetScreenStamp` exists).

### Ask queue and `data-screen-busy`
- Where: `assets/dashboard-screen.js:342`-`:344` (`askChain`, `asksPending`, `queuedAsk`), `:346` (`markScreenBusy`), `:351` (`askNow`), `:375` (`screenTick`)
- State variables: `askChain: Promise`, `asksPending: int`, `queuedAsk: Promise|null`, `<html data-screen-busy>`
- States: idle, one ask queued (deduped by `queuedAsk`), ask in flight, ask chained behind a previous ask
- Transitions: `askNow` `:351` returns `queuedAsk` when set; `.finally` `:361` decrements; `screenTick` `:375` starts one only when `asksPending === 0`; `visibilitychange` `:382` and the pager click force one
- Guards / invariants: at most one ask resolving at a time; a queued ask serves every caller that arrives before it starts
- Explicitness: FULLY IMPLICIT (counter + set of two promise slots)
- Already modeled: no
- Risks seen: `queuedAsk` is cleared only inside the `.then` `:355`; a caller that arrives after `askChain` resolved but before the `finally` runs can still receive a settled promise, and then sees an already-finished ask (inferred, low impact).

### The Game a request names (`X-Chronicler-Game` / `data-draft-game`)
- Where: `assets/dashboard-screen.js:415` (`draftOfRequest`), `:421`-`:426` (`htmx:configRequest`); `assets/dashboard-action-area.js:229`-`:256` (`syncCommandDraft`, `startNewCommandDraft`, `retakeDraftGameWhenScreenShows`); `assets/dashboard-story-log.js:92` (entry gets `data-draft-game`); server `src/adapters/driving/http/utils/request_layers.rs:39` (`refuse_stale_game`), `src/adapters/driving/http/utils/current_game.rs:16`
- State variables: `#command-form[data-draft-game]`, an entry's `data-draft-game`, `document.documentElement.dataset.game`, the request header `X-Chronicler-Game`, and the server's active pointer read per request
- States: draft pinned to the Game the screen showed when typing began; draft pinned to an older Game (stale); no draft (request acts on the active Game); request header matches / does not match the active Game
- Transitions: typing sets the pin `:236`; an empty input drops it `:234`; an Option starts a new draft `:243`; a `409` re-pins it `:254`; a load rewrites `dataset.game` `:313`
- Guards / invariants: a request from a page that shows an old Game must be refused, not applied. The record check (`src/adapters/driving/storage/utils/records.rs`) is the backstop for a Message id.
- Explicitness: FULLY IMPLICIT (one attribute per element plus one root attribute plus one header)
- Already modeled: the active pointer is modeled (`application/games/active.rs`); the per-request draft protocol is not
- Risks seen: any POST from a control with no `[data-draft-game]` ancestor (every dialog form: rename, preset edit, world edit, connection edit) falls back to `screenRoot.dataset.game` (`:424`), i.e. whatever Game the last load showed. A dialog opened before a switch in another tab saves against the new Game; only the record-ownership check prevents a wrong write (known from code, backstop present).

### Failure surfaces: notice place, failure banner, popovers
- Where: `assets/dashboard-errors.js:49` (`refreshFailureBanner`), `:64` (`markUnreachable`), `:78` (`clearUnreachable`), `:117` (`noticeShows`), `:131` (`showNotice`), `:141` (`clearNotice`), `:188` (`closeErrorPopover`), `:197` (`closeAllErrorPopovers`); server `src/adapters/driving/http/builders/headers.rs:159` (`failure_banner_html`), `src/adapters/driving/http/utils/error.rs:24` (`error_disclosure`)
- State variables: `#notice`'s `hidden` plus `dataset.source` (`"request"` / `"screen"`), the two banner slots `#failure-banner-degraded` and `#failure-banner-unreachable` (emptiness), `banner.hidden`, `banner.classList["unreachable"]`, `banner[role]` (`alert`/`status`), each popover's `hidden` plus its toggle's `aria-expanded`
- States: notice empty, notice from a request, notice from a screen load; banner hidden, degraded-only (role `status`), unreachable (role `alert`, also shows the degraded rows); popover closed / open
- Transitions: `showNotice` `:131` (from `reportAnswer` `:152` with `"request"`, from `reportScreenError` `:216` with `"screen"`); cleared by a new request's `htmx:beforeRequest` `:172`, by the close button `:177`, by `clearScreenError` `:236` only when `source === "screen"`; `markUnreachable` `:64` / `clearUnreachable` `:78` from every fetch's catch and success; `clearPostureStatus` `assets/dashboard-saves.js:75`
- Guards / invariants: a screen load never clears the report of a request; `unreachable` implies `hidden = false` and `role = alert`; a popover's `hidden` and its toggle's `aria-expanded` must agree — `data-keep` on both preserves them through a morph
- Explicitness: FULLY IMPLICIT (two independent content slots compose one visual mode; two attributes per popover)
- Already modeled: no
- Risks seen: `markUnreachable` `:64` and `clearUnreachable` `:78` are called from four fetch sites (load, ask, and both failure paths). A single successful `GET /changes` clears the banner while a `GET /screen` is still failing, so the "unreachable" claim can flip per request rather than per engine state (inferred). `refreshFailureBanner` is only re-run on `screen:loaded` `:184` and from those two functions, so a directly written slot would leave the banner stale (known shape, no other writer today).

### Role health (NoCalls → Succeeded / Failed)
- Where: `src/adapters/driving/http/builders/headers.rs:55` (`RoleHealthState`), `:66` (`classify`), `:111` (`role_health_cell`), `:136` (`connections_degraded`), `:159` (`failure_banner_html`); data `src/application/games/view_query.rs:25` (`RoleHealth { backend_model: Option<String>, last_error: Option<String> }`)
- State variables: the pair (`last_error.is_some()`, `backend_model.is_some()`), classified into three states
- States: `Failed(raw)` (last attempt failed), `Succeeded` (last attempt succeeded), `NoCalls` (nothing recorded — explicitly *not* a success), and the banner-only state "degraded" for `Options`/`Trigger`, which have no Settings row
- Transitions: only in the recorder — a new `LlmMessage` for the Role replaces `last_error`/`backend_model`; from the adapter's point of view the state is read-only
- Guards / invariants: `Succeeded` requires `last_error == None` **and** `backend_model.is_some()`; the `(Some(_), None)` combination is unreachable through `classify` and falls into `NoCalls`
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (the three states exist only as a private rendering enum; the transitions live in the LLM recorder)
- Already modeled: partially — the health *data* is modeled, the state machine is not written down anywhere
- Risks seen: `connections_degraded` `:136` only considers `SETTINGS_ROLE_ROWS = [Narrator, Quantifier]`, so the Settings sub-tab marker and the failure banner disagree when only `Options` or `Trigger` is failing (known, intentional per the comment at `:14`). Role health outlives the Game that produced it, so a banner row can name a failure that the current Game never caused (known, stated at `:44`).

### Status display (`data-status` values)
- Where: `src/adapters/driving/http/layout/handlers/endpoints.rs:50` (`render_status_display`), `:65` (`idle_answer`), `:70` (`status_element`), `:86` (`busy_answer`), `:100` (`failure_answer`); reader `assets/dashboard-action-area.js:40`
- State variables: the rendered `data-status` attribute (`idle`, `thinking`, `autoplay`, `failed`) plus the CSS class string
- States: `idle` ("Ready"), `thinking` (Job step label), `autoplay` ("Autoplay: cycle n/budget [step]"), `failed` (failure line in a disclosure). `autoplay` with no step is the between-Jobs state.
- Transitions: derived per load from `GameViewQuery::status` `src/application/games/view_query.rs:198` (busy first, then the failure line, then idle); an unreadable Game renders `failed` with `GenerationFailure::unreadable` `:58`
- Guards / invariants: busy outranks failed, so a Game can hold both a failure line and a running Job; the status display is written only by the server, never by the client
- Explicitness: EXPLICIT STATES (one precedence function), transitions follow the busy registry
- Already modeled: yes for the underlying busy/failed registry; the UI mapping is one function
- Risks seen: `busy_of_game` `src/adapters/driving/http/layout/handlers/action_controls.rs:46` maps `Failed` **and** an unreadable Game to "not busy", so the Send control stays enabled while the status display says `failed` — deliberate (documented at `:43`), but the two parts therefore show different modes for the same Game (known).

### Story-log edit mode (three sources of truth)
- Where: `assets/dashboard-story-log.js:17` (`editState = {id, text} | null`), `:34` (`lockEditing` sets `#story-log[data-editing-entry]`), `:39` (`unlockEditing`), `:59` (`showEditForm`), `:101` (`clearEditState`), `:113` (`revertEdit`), `:135` (`onEditSaved`), `:243` (`screen:cleared`); reader `assets/dashboard-screen.js:114` (`loadKeeps`) and the capture-phase click guard at `:43`-`:57`
- State variables: `editState`, an entry's `data-editing` attribute, the root's `data-editing-entry` attribute, the injected `#edit-textarea`
- States: no edit; edit open (entry locked, textarea holds raw markdown); save in flight; save accepted but waiting for the screen; edit cancelled
- Transitions: `showEditForm` `:59` sets all three; `clearEditState` `:101` clears `data-editing` on entries, drops `editState`, forgets the stamp and asks; `revertEdit` `:113` also unlocks the root; `onEditSaved` `:135` releases the lock only in `afterScreenShows(...).then(unlockEditing)`; a refused or invalid answer returns without unlocking `:139`
- Guards / invariants: `showEditForm` returns early when `editState !== null`; the root lock must outlive the save until the screen shows the saved text; a load must not morph the edited entry
- Explicitness: FULLY IMPLICIT (three representations, released in three places)
- Already modeled: no
- Risks seen: `clearEditState` `:101` runs on a `done` answer *before* the lock is released, so between those two the entry is unlocked for morphing but the root lock still blocks the edit buttons; a second `showEditForm` for another entry cannot open meanwhile (known). A refused save `:139` leaves the edit open with the lock, which is intended but leaves `editState` and `data-editing` alive if the answer never arrives (inferred).

### Edit-dialog lifecycle
- Where: `assets/dashboard-dialog.js:29` (`dialogOpenerId`), `:60`-`:62` (`dialogSession`, `noticeSummary`, `unreachableText`), `:70` (`renderDialogStatus`), `:107` (`htmx:confirm` guard), `:117` (`htmx:afterRequest` sets `data-saving`), `:88` (`close` handler)
- State variables: the native `dialog.open`, `data-saving` on `#edit-dialog`, `dialogSession: int`, the two mirrored strings, `dialogOpenerId`
- States: closed; open with a form; open with `Saving…` (a submit is blocked by `htmx:confirm` `:107`); open after a refused or failed save (values kept)
- Transitions: a swap into `#edit-dialog-body` `:31` opens it; `data-saving` set at `:121` and cleared only in the `close` handler `:90`; a `done` answer closes it after `afterScreenShows` and only for the same `dialogSession`
- Guards / invariants: no second submit while `data-saving`; an answer from a control outside the dialog must not replace its body (`htmx:beforeSwap` `:43`); a save may only close the session that sent it
- Explicitness: FULLY IMPLICIT (open flag + one marker attribute + a session counter)
- Already modeled: no
- Risks seen: `data-saving` is set for every accepted form in the dialog, including the `Update World` form whose posture selects auto-save separately — `clearPostureStatus` `assets/dashboard-saves.js:75` is the only cross-check, so a failed world update under a green posture banner is handled ad hoc (known from the comment). If the awaited stamp never arrives, `data-saving` stays set and the dialog takes no further submit (inferred).

### The "pending until the screen shows it" family
- Where: `assets/dashboard-action-area.js:31` (`endPendingWhenScreenShows`), `:35` (`beginPendingUntilScreenShows`), `:214` (`clearCommandInputAfterScreenShows`), `:250` (`retakeDraftGameWhenScreenShows`); `assets/dashboard-saves.js:66` (`flagSaved` after `afterScreenShows`); `assets/dashboard-dialog.js:123`; `assets/dashboard-story-log.js:135`, `:154`, `:168`
- State variables: the promises inside `screenWaiters`, plus the per-feature flags (`sendPending`, `data-saving`, `data-saved`, `data-editing-entry`)
- States: request accepted, effect not yet on screen (waiting), effect shown (released)
- Transitions: every `htmx:afterRequest` with outcome `done` looks up the answer's `X-Revision` and defers its own end to `afterScreenShows(stamp)`; a non-`done` outcome ends immediately
- Guards / invariants: A must (write) → B (stamp read on the answer) → C (screen load with ≥ that stamp) → D (release the UI state). Releasing at B is the bug the ordering prevents.
- Explicitness: FULLY IMPLICIT (the ordering is a convention repeated in six call sites)
- Already modeled: no
- Risks seen: `clearCommandInputAfterScreenShows` `:214` compares the input against `event.detail.requestConfig.parameters.command` at release time, so a request whose parameters are absent throws there and the pending never ends (inferred, `endPendingAfterRequest` path). Every consumer repeats the same three lines, so a new sender can silently omit the wait (known pattern).

### Prompt-preset Allowed Modes and the default-preset mode
- Where: `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs:123`-`:125` (`allowed_mode_novel`, `allowed_mode_if`), `:173` (create), `:207` (update), `:194` (`edit_preset_form_handler` → `ensure_editable`); renderer `src/adapters/driving/http/builders/presets.rs:79`-`:80`, `:143` (`shows_allowed_modes`), and the card at `:161` (View for `is_default`, Edit/Delete otherwise)
- State variables: two booleans from the form, `PromptPreset.allowed_modes: Vec<NarratorMode>`, `is_default`, `preset_type.is_mode_tagged()`
- States: both modes, novel only, IF only, none (unreachable), and the dual rendering mode View (default preset, read-only) vs Edit
- Transitions: create maps both-`false` to `settings_defaults::default_allowed_modes()` `:176`; update maps both-`false` to `None` = "preserve" `:207`; the checkbox fields exist only when `shows_allowed_modes`
- Guards / invariants: urlencoded checkboxes cannot express "unchecked all", so both-`false` is overloaded as "not offered"; `ensure_editable` blocks an edit of the default preset even if a client posts to the route
- Explicitness: FULLY IMPLICIT (two booleans, one overloaded value, two different readings of it on create vs update)
- Already modeled: no
- Risks seen: a player who unchecks both boxes in the edit form gets "no change" silently instead of an error or an empty set — the same gesture means "both modes" on the create path (known, commented at `:158` and `:204`). An Options preset is mode-agnostic, so `activate_preset_handler` `:283` skips the allow check while its card renders no checkboxes: correct, but the rule lives in two places (known).

### Text-check card (mode + auto-check flag)
- Where: `src/adapters/driving/http/settings/templates/settings.rs:328`-`:329` (checkbox `disabled` when `text_check_disabled`), `:346` (`text_check_disabled`), `:355` (`effective_enable_auto_check`), `:340`-`:341` (`enable_auto_check` is the *effective* value, not the stored one); domain normaliser `src/domain/model/settings.rs:322`, `:331`
- States: Disabled (auto-check forced off, checkbox disabled), Spell / Grammar / Spell+Grammar with auto-check on or off, and the stored-but-inert combination "mode Disabled + stored flag true"
- Transitions: `save_text_check_handler` `src/adapters/driving/http/settings/handlers/settings.rs:216` → `set_mode_and_auto_check` `settings.rs:326`, which normalises `:331`
- Guards / invariants: auto-check must be false while the mode is Disabled; the renderer shows the effective value so the invalid combination is never displayed as checked
- Explicitness: EXPLICIT (one normaliser), only the inert stored value is implicit
- Already modeled: yes
- Risks seen: switching the mode to Disabled discards a previously enabled auto-check, so switching back to Spell leaves it off (known from `normalised_auto_check`). The renderer's `disabled` means the player cannot restore it without re-enabling the mode first (known).

### World form mode (create / edit)
- Where: `src/adapters/driving/http/worlds/templates/worlds.rs:300` (`WorldFormTemplate::from_world_data`), fields `is_edit` `:139`, `is_readonly` `:143`, `auto_save_key: Option<String>` `:141`, `form_action` `:144`, `submit_text` `:149`, `map_placeholder`/`scenarios_placeholder`; template `:57` (`{% if is_edit %}` scope note), `:67` (posture fieldset only when creating), `:124` (posture fieldset with auto-save when editing); handler `src/adapters/driving/http/worlds/handlers/worlds.rs:151` (edit form), `:165` (`update_world_handler`), `:236` (`update_world_posture_handler`)
- States: create form (key editable, posture in the form, POST `/worlds`), edit form (key readonly, posture auto-saves on change to `/worlds/:key/posture`, POST `/worlds/:key`)
- Transitions: one constructor decides all six fields from `world.is_some()`; the server has two writers for the same World record
- Guards / invariants: the create path must post the whole card; the edit path must patch only the posture in `/posture` and the rest in `/worlds/:key`. `default_scenario_id` is dropped when its scenario disappears `:190`.
- Explicitness: FULLY IMPLICIT (six fields, no form-mode enum; two POST routes write one record)
- Already modeled: no
- Risks seen: the posture test in the edit form is a *second* write path (`update_world_posture_handler`) that re-reads and rewrites the whole card; a concurrent detail save (which also rewrites the card from its own read `:176`) can lose the posture change (inferred, last-writer-wins on the same record). The template hides the posture fieldset from the edit *form* `:67` and shows it outside the form `:124`, so a player who changes posture and then presses Cancel keeps the posture change — stated in the note at `:125`, but the rule lives in prose only (known).

### Connection form mode and the test surface
- Where: `src/adapters/driving/http/settings/templates/settings.rs:211` (`editing: bool`), `:271` (`form_action`), `src/adapters/driving/http/settings/handlers/settings.rs:281` (`ConnectionTestSurface`), `:283`-`:290` (`popover_id`)
- States: add form (`/connections/add`, no stored id) vs edit form (`/connections/:id/edit`); test surface `SavedConnection(id)` (answer goes into the dialog body) vs `Form` (answer goes into the form's result slot)
- Transitions: `ConnectionFormTemplate::new(None|Some)`; `into_config(id)` `:146` takes an empty id for an unsaved test `:365`
- Guards / invariants: a test of unsaved values must not write; `max_tokens_ceiling` is preserved by the edit handler `:272`-`:277` because the form does not own it
- Explicitness: EXPLICIT for the test surface (an enum with one `popover_id`), FULLY IMPLICIT for the form mode (`editing` + `form_action` derived independently)
- Already modeled: no
- Risks seen: `into_config` `:146` sets `max_tokens_ceiling: None`, and the add path does not restore it, so the value the comment says the form does not own is silently dropped on add (known from code — probably `None` for a new row anyway, unsure if it can ever be set).

### Saved-Games pager and page view state
- Where: `src/adapters/driving/http/games/templates/games.rs:29` (`GamesPagerView::new`, `previous_disabled`/`next_disabled`, `previous_page`/`next_page` clamped), `:104`-`:106` (`aria-disabled`); client `assets/dashboard-screen.js:24`-`:36` (`rememberGamesPage` / `restoreGamesPage`, `sessionStorage["chronicle.gamesPage"]`), `:390`-`:397` (the click handler), `src/adapters/driving/http/screen/handlers/screen.rs:29` (`games_page_named`)
- State variables: `document.documentElement.dataset.gamesPage`, the sessionStorage copy, the rendered page number, `aria-disabled`
- States: page 1 / middle / last, and out-of-range (a stored page larger than the list now has)
- Transitions: click sets both copies, forgets the stamp and asks; a tab change or reload restores from sessionStorage
- Guards / invariants: `aria-disabled` (not `disabled`) so a morphed control keeps focus; a page number of 0 is a `400`
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS
- Risks seen: the stored page is never clamped on restore, so a page beyond `page_count` renders an empty list with the pager saying "Page 7 of 1" until the player pages back (inferred; the server accepts any page ≥ 1).

### Tabs and the Settings sub-tab
- Where: `assets/dashboard-panels.js:6`-`:7` (`ACTIVE_TAB_KEY`, `SETTINGS_SUBTAB_KEY`), `:9` (`activateTab`), `:35` (`selectSettingsSubtab`), `:73` (`restoreActiveTab`); server side `src/adapters/driving/http/settings/templates/settings.rs:30`-`:33`, `:70` (`aria-selected`, `hidden`, `data-keep`)
- State variables: `.tab.active` (class), `.tab-content.active` (class), the sub-tab's `aria-selected`, the panel's `hidden`, the two sessionStorage strings
- States: one visible tab; within Settings, Connections or Text Check
- Transitions: click → class swap + `leaveSection` for the tab that closes + `askNow`; a `screen:loaded` of the `settings` part restores the remembered sub-tab inside the same task as the morph
- Guards / invariants: view state must survive a morph, so each attribute is listed in `data-keep` (`assets/dashboard-screen.js:60`-`:68`) and the sub-tab is re-applied right after the morph so it never flashes
- Explicitness: FULLY IMPLICIT (a class plus an attribute plus storage for one mode)
- Already modeled: no
- Risks seen: `data-keep` is the only link between a JS view-state attribute and the morph callback; a new attribute added without a `data-keep` entry loses its value silently on the next load (known from the design, no compiler check).

### Unreachable / reachable tracking
- Where: `assets/dashboard-errors.js:64` (`markUnreachable`), `:78` (`clearUnreachable`), `:94` (`outcomeOfStatus`: status `0` → `"none"`)
- State variables: emptiness of `#failure-banner-unreachable` (the state itself is stored as banner content), plus `banner[role]`
- States: reachable (slot empty, banner may be degraded-only), unreachable (slot filled, role `alert`)
- Transitions: `reportAnswer` `:152` on status 0 marks it; `runAsk` `assets/dashboard-screen.js:329` and `:355` and `loadScreen` `:266`-`:275` mark on a fetch throw and clear on a response
- Guards / invariants: at most one banner shows, and an unreachable engine outranks a degraded role
- Explicitness: FULLY IMPLICIT
- Already modeled: no
- Risks seen: the same per-request clearing race as the notice: one answered `GET /changes` clears the banner set by a thrown `GET /screen` (inferred).

### Error-answer classification (two places)
- Where: `src/adapters/driving/http/utils/answer_words.rs:41` (`AnswerOutcome`), `:112` (`unbuilt`), `:277` (`describe_answers`), `:157` (`ANSWER_WORDS`); `src/adapters/driving/http/utils/error.rs:80` (`answer_for_error`)
- State variables: the `ErrorAnswer` response extension (`outcome`, `message`, `detail`, `item`) or its absence
- States: `Refused(Busy | GameChanged | LastMessageChanged)` → `409`, `Invalid` → `400`/`404`, `Failed` → `500`, and the *unbuilt* fallback where the outcome is inferred from the status code at `:112`
- Transitions: handlers build the extension through `answer_for_error`/`refused`/`failed_answer`; `describe_answers` rewrites the body afterwards, or derives an outcome when no extension is present
- Guards / invariants: the status code and the outcome must agree (two places can set it: the builder, and the status-derived fallback)
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (one enum, two producers)
- Already modeled: no
- Risks seen: a route missing from `ANSWER_WORDS` `:157` keeps its handler's raw `error_fragment` body, so the "first line of every failed answer" guarantee is a convention over a hand-maintained table with no compile-time link to the router (`builders/router.rs:48`); a `404` on a route listed as a dialog opener is `Invalid`, on another route `describe_answers` would read it as `Invalid` too because it derives from `status.is_server_error()` (known).

### Route-word table registry
- Where: `src/adapters/driving/http/utils/answer_words.rs:157`-`:246` (`ANSWER_WORDS`), consumed at `:277`
- State variables: none over time — a static table, but it is the registry that a state-like rule ("every POST has words") depends on
- Explicitness: EXPLICIT (a table)
- Already modeled: no
- Risks seen: nothing at run time; the risk is drift with `builders/router.rs`, which no test shown here enforces (unsure whether another test does). I include it only because the failure-body state machine above depends on it.

### Busy state surfaced in the adapter (already modeled elsewhere)
- Where: `src/adapters/driving/http/layout/handlers/action_controls.rs:18` (`send_control_answer`), `:31` (`autoplay_controls_answer`), `:46` (`busy_of_game`); `src/adapters/driving/http/app_state.rs:110` (`view_only = autoplay_holds`), `:145` (`is_busy`); `src/application/games/view_query.rs:219`, `:224`
- State variables: the `Busy` enum from the registry (`src/application/generation/gate.rs:267`), read fresh on every render
- States: idle, busy with a Job at a step, busy with an Autoplay run at a step or between Jobs; "view only" is the Autoplay sub-state
- Transitions: owned by `GameWriter`/`BusyRegistry`; the adapter only reads
- Explicitness: EXPLICIT (owned by `gate.rs`)
- Already modeled: yes
- Risks seen: the adapter derives four views of one state (`send` disabled, `autoplay` disabled, `Stop` present, `view_only`), each with its own rule — a Game busy with a plain Job offers no Stop, which is deliberate (`:37`) but means the disable rule exists in three rendering functions. None seen beyond that.

| name | file | explicitness |
|---|---|---|
| Action-area mode | `assets/dashboard-action-area.js` | FULLY IMPLICIT |
| Screen freshness (revision stamp) | `assets/dashboard-screen.js` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Ask queue / `data-screen-busy` | `assets/dashboard-screen.js` | FULLY IMPLICIT |
| Request Game identity | `assets/dashboard-screen.js` + `utils/request_layers.rs` | FULLY IMPLICIT |
| Failure surfaces | `assets/dashboard-errors.js` | FULLY IMPLICIT |
| Role health | `builders/headers.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Status display `data-status` | `layout/handlers/endpoints.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Story-log edit mode | `assets/dashboard-story-log.js` | FULLY IMPLICIT |
| Edit-dialog lifecycle | `assets/dashboard-dialog.js` | FULLY IMPLICIT |
| Pending-until-screen-shows | `assets/dashboard-action-area.js` and 4 more | FULLY IMPLICIT |
| Preset Allowed Modes / default preset | `prompt_presets/handlers/prompt_presets.rs` | FULLY IMPLICIT |
| Text-check mode + flag | `settings/templates/settings.rs` | EXPLICIT (one normaliser) |
| World form mode | `worlds/templates/worlds.rs` | FULLY IMPLICIT |
| Connection form mode / test surface | `settings/templates/settings.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Saved-Games pager | `games/templates/games.rs` + `dashboard-screen.js` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Tabs / Settings sub-tab | `assets/dashboard-panels.js` | FULLY IMPLICIT |
| Unreachable tracking | `assets/dashboard-errors.js` | FULLY IMPLICIT |
| Error-answer classification | `utils/answer_words.rs` + `utils/error.rs` | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Route-word table | `utils/answer_words.rs` | EXPLICIT |
| Busy state in the adapter | `layout/handlers/action_controls.rs` | EXPLICIT (already modeled) |