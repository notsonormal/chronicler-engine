# Scan 2: implicit state machines in the browser area

Scope: `assets/dashboard-*.js`, `assets/index.html`, the Askama templates in `src/adapters/driving/http/**/templates*.rs` and `templates.rs`, `layout/handlers/action_controls.rs`, `layout/handlers/endpoints.rs`, CSS selectors that read the state. Compared with `docs/diataxis/reference/frontend/dashboard.md`.
Epistemic labels: known = I read the code path. inferred = I reasoned from the code, I did not run it. unsure = I could not settle it.
htmx version in `assets/htmx.min.js`: 1.9.10 (known; `afterRequest` fires on error and on abort, `onLoadError` replaces it when a swap throws).

### The ask loop and the screen stamp
- Where: `assets/dashboard-screen.js` — `askChain` :342, `asksPending` :343, `queuedAsk` :344, `loadsStarted` :190, `screenWaiters` :191, `<html data-revision>`, `<html data-screen-busy>`, `<html data-game>`, `<html data-games-page>`.
- State variables: `data-revision` (absent or `server:rev`), `asksPending` counter (also mirrored to `data-screen-busy`), `queuedAsk` (null or a promise), `loadsStarted` counter, `screenWaiters` array, the visible `.tab-content.active`, `data-games-page`.
- States: NoScreen (no `data-revision`); Current (stamp equals the server answer); Stale-forgotten (stamp deleted by `forgetScreenStamp`); AskQueued (`queuedAsk` set, not started); AskRunning; LoadRunning; Dropped-load (answer discarded after a tab or page change); Hidden-page (poll skips the ask); Unreachable (see the banner machine). Implied: "waiting for a stamp to show" (`screenWaiters` not empty).
- Transitions:
  - page start -> NoScreen -> ask: `askNow()` on DOMContentLoaded or at once :379-383.
  - poll tick every `data-poll-ms` -> ask only if visible and `asksPending === 0`: `screenTick` :372-375. The tick never stops; it only skips.
  - `visibilitychange` visible -> `askNow` :385-387.
  - end of any request (`afterRequest`, `onLoadError`) -> `askNow`; a non-GET also `forgetScreenStamp`: `endRequest` :93-100.
  - tab change -> `leaveSection` forgets stamp, empties the parts of the old tab -> `askNow`: :400-409 and `dashboard-panels.js:9-26`.
  - Previous/Next click -> set `data-games-page`, forget stamp, ask: :392-399.
  - story-log edit ends -> `clearEditState` forgets stamp, asks: `dashboard-story-log.js:101-111`.
  - ask answer differs from `data-revision` -> `loadScreen` :309-340; equal -> `clearScreenError` and stop.
  - load answer arrives -> if tab or games page changed, drop (:272); else `morphParts` writes parts, sets `data-game`, `data-revision`, clears a `screen` notice, fires `screen:loaded`, runs `resolveScreenWaiters` :282-307.
  - `afterScreenShows(stamp)`: resolves at once if the stamp is shown or empty; else queues a waiter that resolves when the stamp shows, or when a load that started after the wait began carries another server id (restart): :193-212.
- Guards / invariants: one ask or load at a time (chain). Waiters resolve only on a load. A `server` change means the revision of the old server means nothing. `data-revision` absent means "page shows nothing". An equal-stamp ask does not load.
- Explicitness: FULLY IMPLICIT (counters, one attribute, two module arrays; no enum).
- Already modeled: no.
- Risks seen:
  - A waiter never resolves if every load fails, or the answer never reaches the stamp. The waiter holds `sendPending`, the dialog "Saving…" line and the edit lock. Known code path (`resolveScreenWaiters` runs only in `morphParts`). The dialog shows the failed-load text; the Thinking line and the edit lock have no exit but a later good load, a tab change, or a reload.
  - `forgetScreenStamp` has five callers (`endRequest`, games-page click, `leaveSection`, `clearEditState`, and via these `askNow`). The doc lists the stamp forget in two places only (end of a non-read request; absent until the next load). Known; see doc mismatch D2.
  - `loadScreen` calls `clearUnreachable()` after the headers, then `markUnreachable()` if the body read fails (:263-271). The banner can flicker. Known; low.
  - The games page value in `data-games-page` is never reconciled with the server clamp. A clamped page keeps sending the stale number. Known; harmless because the control `data-page` values come from the server.

### Failure banner (Unreachable and Degraded)
- Where: `assets/index.html:78-86` (`#failure-banner`, `#failure-banner-degraded`, `#failure-banner-unreachable`), `assets/dashboard-errors.js:49-84`.
- State variables: innerHTML of `#failure-banner-degraded` (server part), innerHTML of `#failure-banner-unreachable` (script), `banner.hidden`, class `unreachable`, attribute `role` (`status` or `alert`). The state is read back from the DOM text: `innerHTML.trim() !== ""`.
- States: Hidden; Degraded only; Unreachable only; Both (Unreachable wins for class and role). 
- Transitions:
  - no answer from `/changes`, `/screen` or a request -> Unreachable: `markUnreachable` called at `dashboard-screen.js:225,271,317,329` and `dashboard-errors.js:152-157`.
  - any answer with a status -> clear Unreachable: `clearUnreachable` (`dashboard-screen.js:264`, `:320`, `dashboard-errors.js:160`).
  - screen load writes `failure-banner` part -> Degraded appears or goes: `screen:loaded` -> `refreshFailureBanner` :184.
  - `refreshFailureBanner` also fires `banner:changed`, which the dialog copies.
- Guards / invariants: the banner is visible iff one of the two slots is non-empty. Nothing but `refreshFailureBanner` writes `hidden`/class/role, and it runs only after three events.
- Explicitness: FULLY IMPLICIT (the DOM content is the state; no variable).
- Already modeled: no.
- Risks seen:
  - Degraded content changes only through a screen load. When `/screen` fails, the Degraded part stays at its last render. Known (doc says the same: every part keeps its content).
  - Unreachable clears on any HTTP answer, including a 500. Doc says this on purpose. Known; matches the doc.

### The notice place (report with a source)
- Where: `assets/index.html:87-91` (`#notice`, `data-source`, `hidden`, `[data-report]`), `assets/dashboard-errors.js:112-150`.
- State variables: `#notice.hidden`, `#notice.dataset.source` (`request` | `screen`), the text of the report, the `data-keep` open state of a Details popover inside it.
- States: Hidden; ShowingRequestReport; ShowingScreenReport.
- Transitions:
  - failed answer (non-done, not "none", no form slot) -> ShowingRequestReport: `reportAnswer` `dashboard-errors.js:152-168`.
  - failed ask or load, or a failed morph -> ShowingScreenReport: `reportScreenError` `dashboard-screen.js:226`, `showScreenFailure`, `showScreenException`.
  - same text and same source again -> no write (`noticeShows` :117) so an open Details stays open.
  - good ask with equal stamp, or good load -> if source is `screen`, hide: `clearScreenError` :246 (called :321-324 and in `morphParts`).
  - any htmx request starts (including a dialog opener GET) -> hide: `htmx:beforeRequest` :171-174.
  - Close click -> hide and focus `#main-content` :176-183.
- Guards / invariants: `hidden` false implies `data-source` set; hidden true implies none. A screen load never clears a `request` report.
- Explicitness: FULLY IMPLICIT.
- Already modeled: no.
- Risks seen:
  - A `screen` report replaces a `request` report, and the later good load clears it, so the request report is lost. Doc says the latest report wins. Known; matches the doc.
  - The dialog copies the summary through `notice:changed` into module variable `noticeSummary`; the copy is a second store of the same fact (see Edit dialog).

### Inline slots and error popovers
- Where: `dashboard-errors.js:30-47` (`renderInlineError`, `clearInlineError`, `formSlotOf`), `:186-246` (popovers). Templates: `[data-error-slot]` in `index.html:139,160` and in form templates; `error_disclosure` in `src/adapters/driving/http/utils/error.rs:22-28`.
- State variables: slot `hidden`; popover `hidden`, `style.left/top/bottom`, toggle `aria-expanded`. The three open-state attributes are in `data-keep` so a morph keeps them.
- States: slot Empty/Shown; popover Closed/Open (positioned).
- Transitions: invalid answer from a form -> slot Shown (`reportAnswer`); next request from the same form -> slot Empty (`beforeRequest`); toggle click -> open or close; click outside, Escape, resize, any scroll -> close all.
- Guards / invariants: `aria-expanded` equals `!popover.hidden` (two values for one fact, kept equal by `closeErrorPopover` and the toggle handler only).
- Explicitness: FULLY IMPLICIT.
- Already modeled: no.
- Risks seen:
  - `data-keep="hidden style"` keeps a popover open through a load, but the popover position is not recomputed after a morph that moves the toggle. Inferred; the scroll/resize handlers close it.
  - An `/action/check` slot is `inline-error-slot` in `#command-form`; a refused (409) answer is `refused`, not `invalid`, so it goes to the notice, not the slot. Known; matches doc.

### Action area (Idle / Checking / Generating / Autoplay / Preview)
- Where: `assets/dashboard-action-area.js` — `checkInFlight` :5, `sendPending` :10, `previewWasOpen` :89, `previewSentText` :283; DOM `#action-area[data-state]`, `#action-status`, `#action-preview`, `#status-display [data-status]`; CSS `styles.css:673-696`, `:739`.
- State variables: `checkInFlight`, `sendPending`, "status display shows `data-status=thinking`", "status display shows `data-status=autoplay`", "`#action-preview` has a child element", `previewWasOpen` (edge memory for focus), `previewSentText`.
- States (as in the doc): Idle, Checking, Generating, Autoplay, Preview. Derived by one function with fixed priority checking > autoplay > generating > preview > idle: `deriveActionState` :64-70. Implied: Generating-by-pending (no status display yet) versus Generating-by-status.
- Transitions:
  - command form `before-request` -> Checking (`checkInFlight = true`): `onCommandBeforeRequest` :208 (`index.html:118-131`).
  - command form `after-request`: clears Checking; refused -> retake draft Game; done and no preview child -> clear input after screen shows and `beginPendingUntilScreenShows`: `onCommandAfterRequest` :270-281. done with preview child -> Preview.
  - `on-load-error` -> clears Checking: `onCommandLoadError` :265.
  - Retry, Retrigger, "New options": `before-request` -> `updateToThinking` (`sendPending = true`) :195; `after-request` -> `endPendingAfterRequest` (done: wait for screen; else end now) :306-310. Templates: `templates.rs:25` (log entry), `:64` (options dock).
  - Preview send -> `onPreviewSendAfterRequest` :290-297; Autoplay form -> `onAutoplayAfterRequest` :299-304; both call `beginPendingUntilScreenShows`.
  - `endPending` -> `sendPending = false`, re-derive :25-29.
  - Cancel in preview -> `closeActionPreview` :312-316 (template `templates.rs:105`).
  - `screen:loaded` -> re-derive (and `applyStatusDisplay` when the status display loaded): :183-189.
  - Option click -> `useOption` guard `!checkInFlight && !actionIsGenerating()` :318-324; `editOption` guard only `!checkInFlight` :326-333.
- Guards / invariants: the server renders `disabled` for Send, Autoplay, Option buttons from the busy state; no script writes `disabled` (known: grep finds no `disabled =` in any dashboard script). CSS reads only `data-state="generating"` (`styles.css:695-696`); the other four `data-state` values are written but no stylesheet reads them (known; tests may). Stop replaces the Action input by `:has(#stop-btn)` (`styles.css:673-676`), not by `data-state="autoplay"`: two separate signals for the Autoplay look.
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS. The state names exist as strings and one derive function, but the inputs (`checkInFlight`, `sendPending`) are set from about nine handlers across the file and from templates (`hx-on` attributes in `templates.rs`).
- Already modeled: no.
- Risks seen:
  - `sendPending` is a boolean, not a counter. Two overlapping sources (Retry has no `hx-sync`; a second click before the first answer) end on the first `endPending`. Inferred.
  - `sendPending` and `checkInFlight` are not reset by `screen:cleared` or by a Game switch. After a tab change the derive runs against an empty status display. Inferred: the hidden area reads Idle until the Game tab loads again; the Autoplay state flickers off for one load. Known code path, effect low.
  - If a screen never shows the stamp (see ask loop), `sendPending` stays true and the area shows "Thinking..." with `data-state=generating`. Known code path.
  - The preview keeps the Game it was written for through `#command-form[data-draft-game]`, but `syncCommandDraft` removes that attribute when the input becomes empty (:230-239). If the player empties the input while a preview is open, a preview send names the screen Game, not the draft Game. Known code path (`draftOfRequest` in `dashboard-screen.js:411-414` maps the preview to `#command-form`).
  - `editOption` does not check `actionIsGenerating()` while `useOption` does. Known; the server disables the Option buttons while busy, so the gap shows only in the window before a load.
  - `/action/check` is one route with two outcomes (a preview, or a dispatch with an empty body). The client tells them apart by "preview has a child element" after the swap (`onCommandAfterRequest` :274). A dispatch answer of `ok("")` (`actions.rs:36`) empties the preview. Known; works, but the route outcome is inferred from the DOM.

### Command draft pin (`data-draft-game`)
- Where: `dashboard-action-area.js:215-262`, `dashboard-screen.js:411-428` (`draftOfRequest`, `configRequest`), `dashboard-story-log.js:78-80`.
- State variables: attribute `data-draft-game` on `#command-form` and on the entry in edit; `data-game` on `<html>`; the value of `#command-input`.
- States: NoDraft (input empty, attribute absent); Pinned(Game id); Pinned-to-empty (screen had no Game); Retaken (after a 409, the pin moves to the screen Game).
- Transitions: `input` event with text and no attribute -> Pinned to `data-game`; input becomes empty -> NoDraft; send accepted and the input still holds the sent text -> `form.reset()` after the screen shows -> NoDraft; Option -> `startNewCommandDraft` -> re-pin; 409 -> `retakeDraftGameWhenScreenShows`.
- Guards / invariants: attribute present iff input non-empty, except the preview case above.
- Explicitness: FULLY IMPLICIT (an attribute with a free-form value).
- Already modeled: no.
- Risks seen: the preview gap above (known). A script-set `input.value` does not fire `input`; the code calls `syncCommandDraft` by hand in `startNewCommandDraft` and `clearCommandInput`. Any new script write needs the same call. Inferred.

### Edit dialog (`#edit-dialog`)
- Where: `assets/dashboard-dialog.js` — `dialogOpenerId` :29, `dialogSession` :60, `noticeSummary` :61, `unreachableText` :62; `<dialog id="edit-dialog">` in `index.html:218-221`; attribute `data-saving`; native `open`.
- State variables: `editDialog.open`, `data-saving`, `dialogSession`, `dialogOpenerId`, `noticeSummary`, `unreachableText`, the text of `#edit-dialog-status`.
- States: Closed; Open-editing; Open-saving (answer done, screen not showing yet). Implied: Open with a body replaced by a second swap.
- Transitions:
  - swap into `#edit-dialog-body` -> Open-editing (`showModal` if closed), remember opener id: `htmx:afterSwap` :31-39.
  - swap into the body from a control outside the open dialog -> blocked: `htmx:beforeSwap` :43-49.
  - form inside the dialog answers `done` -> Open-saving, then `afterScreenShows(stamp)` -> `close()` only if the session matches: `htmx:afterRequest` :115-127.
  - a form submit while `data-saving` -> cancelled: `htmx:confirm` :107-113.
  - Cancel (`data-close-dialog`), Escape (native), or the save above -> `close` event: clears the body and `data-saving`, increments the session, returns focus to the opener id: :87-93, :74-79 (focus), :95-100 (Cancel).
  - `notice:changed` and `banner:changed` -> copy text, re-render the status line :81-85.
- Guards / invariants: `data-saving` implies `open`. A saving dialog takes no second form submit. A button inside a form (the connection Test button) never starts Open-saving, because only `FORM` elements count.
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (no enum; `open` plus `data-saving` plus a session number, four listeners).
- Already modeled: no.
- Risks seen:
  - Before the first answer arrives, a dialog form has no guard against a second submit: `data-saving` is set only in `afterRequest`, and the forms have no `hx-sync` (known: `games.rs` rename/new, `settings.rs`, `worlds.rs`, presets forms). A double click can send the write twice (for example New Game). Known code path; effect depends on the route (inferred: a duplicate Game).
  - `data-saving` stays on if the screen never shows the stamp; the status line says "Saving…" plus the failed-load summary; only Escape or Cancel leaves. Known code path (doc says it names the failed load).
  - `dialogOpenerId` is overwritten by any later swap into the body. Today no dialog-internal control targets the body (known: grep of `hx-target`), so none seen; a future one would drop the focus return. Inferred.
  - `noticeSummary` and `unreachableText` are mirrors of the notice and banner. They are updated only by custom events; a missed event leaves a stale line. None seen.

### Story-log edit mode and its lock
- Where: `assets/dashboard-story-log.js` — `editState` :17; DOM: `#story-log[data-editing-entry]`, `.log-entry[data-editing]`, `[data-draft-game]`, `#edit-textarea`; templates `templates.rs:25` (`showEditForm`, `cancelEdit`, `onEditSaved`); CSS `styles.css:1456-1460`, `:1558-1559`, `:1568-1583`.
- State variables: `editState` (`null` or `{id, text}`), the lock attribute on the root, the `data-editing` attribute on one entry, the textarea node, `storyLogSwapStart`.
- States: Idle (`editState` null, no lock); Editing (editState set, entry `data-editing`, lock on); Saved-awaiting-screen (editState null, lock still on, entry no longer `data-editing`) — implied by the combination only; Reverted (back to Idle).
- Transitions:
  - Edit click -> Editing: `showEditForm` :59-98 (guard `editState !== null` returns).
  - Cancel or Escape -> `cancelEdit` -> `revertEdit` -> Idle: :113-133.
  - Save answers `done` -> `clearEditState` (editState null, drop `data-editing`, forget stamp, ask) -> Saved-awaiting-screen; then `afterScreenShows(stamp).then(unlockEditing)` -> Idle: `onEditSaved` :135-146.
  - Save answers refused or invalid -> stay Editing (draft and lock kept): :138.
  - Save answers failed or none -> `revertEdit` -> Idle (the draft text is lost): :139-142.
  - tab leaves -> `screen:cleared` -> editState null, unlock: :243-246.
  - A load skips the `data-editing` entry (`loadKeeps`, `dashboard-screen.js:114-116`).
- Guards / invariants: at most one entry in edit. Lock on whenever `editState` is set. While the lock is on, the click guard blocks `.edit-btn` and `.swipe-controls .action-btn` (`:46-57`, `styles.css:1558`). Delete and Retrigger are not in that guard; the edited entry hides them by CSS (`styles.css:1456`).
- Explicitness: FULLY IMPLICIT for the lock phases (a variable plus two DOM attributes, no enum). The third state exists only as a variable combination.
- Already modeled: no.
- Risks seen:
  - `showEditForm` sets `editState = {id, text: null}` before it checks that the entry exists (:60-62). If the entry is missing, `editState` stays set with no lock, no `data-editing` and no UI; all later Edit clicks return at the guard until `screen:cleared`. Known code path; the click comes from an entry, so it is hard to reach (inferred).
  - Saved-awaiting-screen: a tab change runs `screen:cleared` (unlock), then the old waiter can later run `unlockEditing` while a new edit is open. Inferred; the first load after the tab change normally resolves the waiter before a new edit can start.
  - Delete of the last entry while another entry is in edit: `beforeNodeRemoved` keeps nodes that are being edited (`loadKeeps`), and a deleted entry that is itself in edit would stay as a ghost. The CSS hides Delete on the entry in edit, so the ghost needs the deleted entry to differ from the edited one. None seen; stated for completeness. Inferred.
  - `editState.text` stores the entry HTML; if a load carried a changed render of the entry text, revert restores the stale HTML. The entry is skipped by loads, so it is the same text. None seen.

### Story-log follow-the-bottom scroll
- Where: `dashboard-story-log.js:177-241` — `storyLogSwapStart` :184.
- State variables: `storyLogSwapStart` (null or `{atBottom, entries}`), the scroll position of `#story-log`, `editState`.
- States: NotFollowing (null); Armed (captured at `screen:beforemorph`); Following (rAF retry loop up to 1000 ms until the box can scroll).
- Transitions: `beforemorph` with `story-log` -> Armed; `loaded` -> consume; scroll to bottom only if it was at bottom within 8 px, entries grew, and no edit open; retry per frame while the log cannot scroll yet; stop on an edit, on success, or on the deadline.
- Guards / invariants: "following" is not stored; it is the position at the moment of the load. An edit cancels the retry loop.
- Explicitness: FULLY IMPLICIT (transient; the mode is the scroll position).
- Already modeled: no.
- Risks seen: if `morphParts` throws between `beforemorph` and `loaded`, `storyLogSwapStart` stays set until the next `beforemorph` or `loaded` (:232-237 resets it). Known code path; harmless.

### Tabs, Settings sub-tab, Games page (client view state)
- Where: `assets/dashboard-panels.js:1-106`; `index.html:93-100` (tab buttons), `:101-216` (tab panels); `settings.rs:28-33,70` (`data-keep` on sub-tab buttons and panels); `dashboard-screen.js:6,24-41`.
- State variables: `.tab.active` plus `aria-selected` on the tab buttons; `.tab-content.active` on panels; `sessionStorage` keys `chronicle.activeTab`, `chronicle.settingsSubtab`, `chronicle.gamesPage`; `aria-selected` on `.settings-subtab`; `hidden` on `.settings-subtab-panel`; `data-games-page`.
- States: one active tab of six; one active Settings sub-tab of two; a Games page number.
- Transitions: tab click -> `activateTab` (swap classes, `leaveSection`, `askNow`) and remember; DOMContentLoaded -> `restoreActiveTab`; sub-tab click -> `selectSettingsSubtab` and remember; `screen:loaded` with `settings` -> restore the remembered sub-tab (the fresh render shows the first one); Previous/Next click -> `data-games-page` and storage.
- Guards / invariants: exactly one tab button active and one panel active (set in one function). One selected sub-tab button and one visible sub-tab panel (two attributes on four nodes; `data-keep` keeps them across loads). A tab's parts exist only while the tab is open.
- Explicitness: FULLY IMPLICIT (classes, ARIA attributes and storage; no variable).
- Already modeled: no.
- Risks seen:
  - `activateTab` removes all `active` classes first and returns if the panel is missing (:21), which leaves no tab shown. Known code path; the ids come from `data-tab` in the shell, so it needs a bad stored value. `restoreActiveTab` checks the button but not the panel. Inferred.
  - The sub-tab state has three copies (button attribute, panel attribute, storage). A fresh Settings render shows the first sub-tab for one task before the restore (comment says it never paints). Known.
  - `restoreActiveTab` runs in a DOMContentLoaded listener registered after the first `askNow` of `dashboard-screen.js:379-383`. The first ask is async, so `loadScreen` reads the restored tab. Inferred; works by ordering.

### Slash palette
- Where: `assets/dashboard-slash-menu.js:8-10` (`slashMenu`, `slashMatches`, `slashActiveIndex`), `#slash-menu` element, `aria-expanded` and `aria-activedescendant` on the input.
- State variables: `slashMenu` (null or element), `slashMatches` array, `slashActiveIndex` (-1 or index), `aria-expanded`, `aria-activedescendant`, class `active` and `aria-selected` on each item.
- States: Closed (menu null, matches empty, index -1); Open (menu set, matches non-empty, index 0..n-1).
- Transitions: `input` event -> `openSlashMenu` -> Open if there are matches else Closed :87-91; ArrowUp/Down -> move index; Enter with a match -> `selectSlash` (fills the command, closes) else close; Escape, form submit (capture), scroll (capture), resize, mousedown outside, focusout after 150 ms if the input lost focus -> `closeSlashMenu`.
- Guards / invariants: menu null iff matches empty and index -1 (kept by `closeSlashMenu`); `aria-expanded` mirrors menu existence; active item DOM state mirrors `slashActiveIndex`.
- Explicitness: EXPLICIT STATES, SCATTERED TRANSITIONS (three variables plus ARIA/DOM copies; one close function, six close callers).
- Already modeled: no.
- Risks seen:
  - `setTimeout(..., 150)` in `focusout` can fire after the player returned focus to the input; it re-checks `document.activeElement`, so it is safe. Known.
  - A script write to `input.value` (`useOption`, `clearCommandInput` via `form.reset()`) fires no `input` event, so a menu that is open is not refreshed. Submit closes it in the reset path; the Option path never opens it. Inferred; none seen.

### Request in-flight tracking and the keep rules of a load
- Where: `dashboard-screen.js:72-117` — `requestsInFlight` Map; `data-editing`; `beforeNodeMorphed`, `beforeNodeRemoved`, `beforeAttributeUpdated` callbacks; `data-keep`.
- State variables: `requestsInFlight` (element -> count), `data-editing` on an entry, `data-keep` lists.
- States per node: Free; InFlight (count ≥ 1); BeingEdited. A load skips a node in InFlight or BeingEdited.
- Transitions: `htmx:beforeRequest` -> count + 1; `afterRequest` or `onLoadError` -> count − 1 (`endRequest`).
- Guards / invariants: count never below zero (guard in `releaseInFlight`). Every `beforeRequest` has one later `afterRequest` or `onLoadError`: known for htmx 1.9.10 (`onerror`, `onabort`, `ontimeout` all fire `afterRequest`).
- Explicitness: FULLY IMPLICIT.
- Already modeled: no.
- Risks seen: a request that ends with neither event would keep its control out of every later load. Not seen in htmx 1.9.10. Inferred. Also: the keep rule protects only two kinds of node (see doc mismatch D1).

### Saving controls: "Saved" flag and rendered-value restore
- Where: `assets/dashboard-saves.js:1-78`; `data-saved-flag`, `data-saved`, `data-autosave`; templates `settings.rs:39,312,317`, `games.rs:199-237`, `worlds.rs:71-88`.
- State variables: the control `defaultValue`/`defaultChecked`/`defaultSelected` (used as the "rendered value" store), the flag element `data-saved`.
- States per control: Rendered (value equals default); Edited-unsaved; Saved-pending-screen; Saved (flag on); Failed-reverted.
- Transitions: `afterRequest` `done` -> `settleSavedFields` (the new value becomes the default), then flag on after the screen shows; not done -> flag off, restore default (a lone control, or a `data-autosave` form); `clearPostureStatus` clears the world flag before an Update World request.
- Guards / invariants: the default holds the last accepted value. A load keeps `data-saved` through `data-keep` on the flag element.
- Explicitness: FULLY IMPLICIT (it reuses the DOM default value as a store).
- Already modeled: no.
- Risks seen:
  - A `done` answer settles the default before the screen shows; a second change before the first save ends is `hx-sync`-free (`hx-trigger="change"` selects, no sync), so two saves can overlap and a later failure restores the first value, not the newest accepted one. Inferred.
  - `flagSaved` for a stale save can set `data-saved` after a newer failed save cleared it, because the waiter resolves later. Inferred.

### Live regions (announcements)
- Where: `dashboard-action-area.js` — `announcedStatusLabel` :110, `announcedEntryIds` :140, `announcedOptions` :171; `dashboard-story-log.js:3-15,148-157` (`restore-notice` with a 4 s clear timer, `noticeClearTimers`).
- State variables: three "last seen" memories (a string, a Set or null, an array or null) and a per-element timer.
- States: FirstLoad (memory null or "Ready": announce nothing) versus Later (announce differences). 
- Transitions: first `screen:loaded` with the part -> record; later loads -> diff and write a live region; restore notice -> text, then clear after 4 s (a re-announce replaces the timer).
- Guards / invariants: null means "never seen". `announcedStatusLabel` starts at "Ready" so an idle first load is silent.
- Explicitness: FULLY IMPLICIT (first-time versus later behaviour in nullable memories).
- Already modeled: no.
- Risks seen:
  - Not reset on a Game switch or a tab leave (known: no listener for `screen:cleared` or a change of `data-game`). After a switch to another Game, every narration entry has an unseen id and all of them are announced in one block. Known code path; the story window is a bounded set. Same for Options.
  - `announcedEntryIds` compares ids only; a retried narration with a new Swipe on the same Message is not announced. Known; may be intended.

### Pager of saved Games
- Where: `src/adapters/driving/http/games/templates/games.rs:103-107`; `dashboard-screen.js:392-399`; `games.css:256`.
- State variables: `data-games-page` on `<html>`, `aria-disabled` on Previous/Next, `data-page` on each control.
- States: no pager (one page); first page; middle; last page; (client page beyond the end, clamped by the server).
- Transitions: click on a control that is not `aria-disabled` -> set page, store, forget stamp, ask.
- Guards / invariants: a control at an end is `aria-disabled`, not `disabled`, so it keeps focus; the handler ignores it. The server owns the page rule.
- Explicitness: EXPLICIT (the server computes the pager; the client holds one number). Listed with the screen machine above.
- Already modeled: no.
- Risks seen: none beyond "page value not reconciled with the server clamp" (see ask loop).

### Generation gate, busy state, Job, Autoplay run (as shown in the browser)
- Where: `layout/handlers/action_controls.rs:11-60` (renders Send, Autoplay, Stop from `Option<Busy>`), `layout/handlers/endpoints.rs:46-110` (`status_element` with `data-status` = `idle` | `thinking` | `autoplay` | `failed`), `games.rs:44,57,64` (`run_held` badge).
- State variables: `Busy::Job(step)`, `Busy::Autoplay{progress, step}`, `Status::Failed`, `Status::Idle`, rendered as `disabled` and `data-status`.
- Explicitness: EXPLICIT (server enum) — the browser only reads it.
- Already modeled: yes (gate, Job, Autoplay run). The browser parts add: Stop exists only for an Autoplay run; Send shows "Generating…" while busy.
- Risks seen: none seen in the browser rendering. One local note: a `Status::Failed` and an unreadable Game both render as `data-status="failed"`; the action area treats `failed` like idle (known: no branch for it in `deriveActionState`).

## Doc versus code (`docs/diataxis/reference/frontend/dashboard.md`)

- D1. Statement 3 says "A load skips an element that holds unsaved input." The code skips two kinds only: an entry with `data-editing`, and a node with a request in flight (`loadKeeps`, `dashboard-screen.js:114-116`). The doc names "the two kinds" later, so the doc disagrees with itself. A part form with typed but unsent text (for example the text-check settings form, which saves on `change`) is not skipped by the keep rules. Whether Idiomorph keeps the typed value of the focused field: unsure.
- D2. The doc marker table says `data-revision` is absent "from the end of a request that is not a read until the next load". The code also forgets the stamp in `leaveSection`, in the Previous/Next click and in `clearEditState`. The doc says the stamp is forgotten at "the end of every request that is not a read" only. Known; low.
- D3. The doc says Stop shows "in place of the Action input and the Autoplay control" in the Autoplay state. The code does this with CSS `:has(#stop-btn)` (`styles.css:673-676`), not with `data-state="autoplay"`. The doc `data-state` row says the CSS reads it "for the states that only the page knows". Only `generating` is read by CSS (`styles.css:695-696`); `checking`, `autoplay`, `preview`, `idle` are written and not read by any stylesheet. Known.
- D4. The doc list of client state omits: `sendPending`/`checkInFlight` as variables (it names "the action-area state" only), `previewSentText`, `dialogSession`, the three live-region memories, `screenWaiters`, `requestsInFlight`. Known; the doc is selective on purpose, but the live-region memories and `screenWaiters` hold behaviour (first load silent, wait for stamp).
- D5. The doc says the Save control and swipe controls lock while an edit is open, and names Edit and swipe controls. The code also leaves Delete and Retrigger unlocked (hidden on the edited entry by CSS only). Known; the doc does not claim otherwise.
- D6. The doc says "The page asks `GET /changes` every `data-poll-ms` while the browser shows the page." Code: the timer never stops; a tick skips the ask when the page is hidden or an ask is pending (`dashboard-screen.js:372-375`). Matches in effect.
- D7. Everything else I checked agrees with the code: the five outcomes (`outcomeOfStatus`), `X-Chronicler-Game` header rule, no script writes `disabled`, dialog open/close rules and the backdrop (no handler), `data-source` clear rule, the focus plan after a load, the three drafts rules, the Unreachable text, `data-ask-timeout-ms` default of 10 s.

## Table of contents

| Name | File | Explicitness |
|---|---|---|
| Ask loop and screen stamp | assets/dashboard-screen.js | FULLY IMPLICIT |
| Failure banner (Unreachable, Degraded) | assets/dashboard-errors.js, assets/index.html | FULLY IMPLICIT |
| Notice place | assets/dashboard-errors.js | FULLY IMPLICIT |
| Inline slots and error popovers | assets/dashboard-errors.js | FULLY IMPLICIT |
| Action area (5 states) | assets/dashboard-action-area.js | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Command draft pin | assets/dashboard-action-area.js, dashboard-screen.js | FULLY IMPLICIT |
| Edit dialog | assets/dashboard-dialog.js | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Story-log edit mode and lock | assets/dashboard-story-log.js | FULLY IMPLICIT |
| Story-log follow scroll | assets/dashboard-story-log.js | FULLY IMPLICIT |
| Tabs, Settings sub-tab, Games page | assets/dashboard-panels.js | FULLY IMPLICIT |
| Slash palette | assets/dashboard-slash-menu.js | EXPLICIT STATES, SCATTERED TRANSITIONS |
| In-flight tracking and keep rules | assets/dashboard-screen.js | FULLY IMPLICIT |
| Saving controls and Saved flag | assets/dashboard-saves.js | FULLY IMPLICIT |
| Live-region memories | assets/dashboard-action-area.js, dashboard-story-log.js | FULLY IMPLICIT |
| Games pager | games.rs, dashboard-screen.js | EXPLICIT |
| Gate, Job, Autoplay run (browser view) | action_controls.rs, endpoints.rs | EXPLICIT (already modeled) |
