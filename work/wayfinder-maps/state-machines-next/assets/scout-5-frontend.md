# Implicit state machines in the browser code and the server-rendered templates

### Action-area view state
- **Where**: `assets/dashboard-action-area.js:5` (`checkInFlight`), `:10` (`sendPending`), `:89` (`previewWasOpen`); the mode source is the server part `#status-display` (`src/adapters/driving/http/layout/handlers/endpoints.rs:69-107`) and the presence of a child in `#action-preview` (`assets/index.html:119-123`)
- **State variables**: `checkInFlight: bool`, `sendPending: bool`, `#status-display [data-status]` ∈ `idle | thinking | autoplay | failed`, `#action-preview.firstElementChild` (presence), `previewWasOpen` (the remembered previous value of that presence), and the derived string written to `#action-area[data-state]`
- **States**: `idle`, `checking`, `generating`, `autoplay`, `preview` (the five the doc names). Implied states: "sending with no Job shown yet" (`sendPending` true while `data-status` still says idle), and "failed display" which derives to `idle`
- **Transitions**:
  - any → `checking`: command form request starts → `onCommandBeforeRequest` `dashboard-action-area.js:208`
  - `checking` → previous: `onCommandAfterRequest` `:270-281`, `onCommandLoadError` `:265`
  - any → `generating`: `updateToThinking` `:195` (Retry, Retrigger, Options regen), accepted command `:277`, accepted preview send `:294`, accepted Autoplay start `:301`
  - `generating` → previous: `endPendingWhenScreenShows` `:31`, `endPendingAfterRequest` `:306`
  - any → `preview`: htmx swaps the check answer into `#action-preview` (`index.html:120-121`); no JS writes this transition
  - `preview` → `generating`: `onPreviewSendAfterRequest` `:290-297`; `preview` → idle: `closeActionPreview` `:312`
  - any → `autoplay`: a load writes `data-status="autoplay"`; `applyStatusDisplay` `:132-138`
- **Guards / invariants**: no script writes `disabled` (`:71-77` comment). `deriveActionState` is the only reader, so the order is the whole priority: `checking` > `autoplay` > `generating` > `preview` > `idle`
- **Explicitness**: FULLY IMPLICIT (two booleans, one DOM-presence flag, one server string, one remembered flag; the enum exists only as a derived string)
- **Already modeled**: no (the `Busy` enum of the gate is modeled; its *rendering* into `data-status` and the browser's own states are not)
- **Risks seen**:
  - known: `preview` is unreachable while a run holds the Game, because `autoplay` wins (`:64-69`), yet spec 16.9 keeps Autoplay clickable in the Preview state. The state name then no longer describes the screen.
  - known: `useOption`/`editOption` guard only on `checkInFlight || actionIsGenerating()` (`:318-332`); `actionIsGenerating` is false while `data-status="autoplay"` (`:47-57`), so an Option button rendered while idle is only protected by the server's `disabled` render (`src/adapters/driving/http/templates.rs:64`).
  - known: a done answer with no `X-Revision` header (`dashboard-errors.js:101-107`) lands as `stamp === null`, and `afterScreenShows(null)` resolves at once (`dashboard-screen.js:193-195`) — `sendPending` ends before the screen shows anything.
  - known: `data-state` is not re-derived on a tab change (`leaveSection` `dashboard-screen.js:400`), so the area can keep `generating` while its parts are empty.

### The screen stamp and the waiters that end a pending state
- **Where**: `assets/dashboard-screen.js:7` (`screenRoot`), `:170` (`forgetScreenStamp`), `:174-187` (`splitStamp`/`screenShows`), `:190-191` (`loadsStarted`, `screenWaiters`), `:193-211` (`afterScreenShows`/`resolveScreenWaiters`)
- **State variables**: `<html data-revision>` present/absent, the `<server>:<revision>` value it holds, `loadsStarted` counter, `screenWaiters[]` entries `{stamp, loadsStartedBefore, resolve}`
- **States**: stamp shown and current; stamp shown and behind; stamp absent ("forgotten" after a non-read request); zero or more waiters queued; a waiter of an earlier server generation (engine restart)
- **Transitions**: set ← `morphParts` `:305`; forgotten ← `endRequest` `:96` (non-get) and `leaveSection` `:404`; waiter pushed ← `afterScreenShows` `:197`; waiters resolved ← `resolveScreenWaiters` `:201-211` (called from `morphParts` `:307`)
- **Guards / invariants**: a waiter resolves only on `have.revision >= want.revision` with the same server id, or on a load started after the wait that shows a different server id (`:203-209`). A load that is *not* started before the wait began does not count
- **Explicitness**: FULLY IMPLICIT (presence of an attribute plus a queue of closures)
- **Already modeled**: no
- **Risks seen**:
  - known: only a *successful* morph resolves waiters. A load that fails, or that is dropped at `:274` because the tab or page changed, leaves the waiters queued. Every consumer (`sendPending`, the story-log lock, the dialog close, the "Saved" flag) then keeps its state until some later load succeeds. With the engine down, "Thinking…", the edit lock and "Saving…" are all stuck at once.
  - inferred: `screenWaiters` is never cleared on `screen:cleared`, so a waiter for a Game that no longer exists survives a Game switch or reset.

### The ask / load loop
- **Where**: `assets/dashboard-screen.js:342-344` (`askChain`, `asksPending`, `queuedAsk`), `:346-349` (`markScreenBusy`), `:351-370` (`askNow`), `:372-380` (`screenTick`)
- **State variables**: `asksPending: int` (count, and the "busy" flag it derives), `queuedAsk: Promise|null` (in-flight ask), `askChain` (serialisation), `document.visibilityState`, `<html data-screen-busy>` presence
- **States**: quiet, asking, loading, hidden-page (no ask at all), failed ask (banner up, loop continues)
- **Transitions**: start ← `DOMContentLoaded` `:376-380`, `visibilitychange` visible `:383`, every `screenTick` `:372`, every htmx request end `endRequest` `:97`, tab change (`dashboard-panels.js:22`), Games pager click `:395`, end of an edit (`dashboard-story-log.js:105`)
- **Guards / invariants**: one ask at a time (`queuedAsk` dedupe, `asksPending === 0` in the tick); a load whose section or `data-games-page` changed is dropped (`:274`)
- **Explicitness**: FULLY IMPLICIT (a counter plus a nullable promise)
- **Already modeled**: no
- **Risks seen**:
  - known: `data-screen-busy` is written `:347-348` but no shipped code reads it. Only the test harness does (`tests/test_utils/convergence.js:19`, `tests/test_utils/screen.rs:18`). The doc's Markers table presents it as page state.
  - known: `askChain` is extended on every request end and never reset; it holds one promise per ask for the life of the page.

### What a load keeps: in-flight requests and edited entries
- **Where**: `assets/dashboard-screen.js:72` (`requestsInFlight`), `:74-86` (`holdInFlight`/`releaseInFlight`), `:104-119` (`requestIsInFlight`, `isBeingEdited`, `loadKeeps`), `:121-129` (Idiomorph callbacks)
- **State variables**: `Map<Node, int>` request count per element; `data-editing` on a node; `data-keep` attribute lists
- **States**: a node is loadable, or held by a request, or held by an edit (both → held); `htmx-request` class (explicitly *not* trusted, `:66-71`)
- **Transitions**: count +1 ← `htmx:beforeRequest` `:84-87`; count −1 ← `htmx:afterRequest`/`htmx:onLoadError` `:91-92`; `data-editing` set/cleared by the story log (`dashboard-story-log.js:59`, `:101`)
- **Guards / invariants**: a held node is neither morphed nor removed (`:115-128`). `beforeAttributeUpdated` returns false for every attribute the element's `data-keep` names (`:59-67`), which is how view state survives
- **Explicitness**: FULLY IMPLICIT
- **Already modeled**: no
- **Risks seen**:
  - known: the count is keyed by the request element. An element that a load replaces is a detached key until its request ends, so the *replacement* node is not held and can be morphed by a later load while the same logical control still waits. Inferred: the guard covers the node, not the intent.

### Story-log edit lifecycle
- **Where**: `assets/dashboard-story-log.js:17` (`editState`), `:34-52` (root lock), `:59-99` (`showEditForm`), `:101-133` (`clearEditState`/`revertEdit`/`cancelEdit`), `:135-146` (`onEditSaved`), `:244-247` (`screen:cleared`)
- **State variables**: `editState: null | {id, text}` (`text` starts `null`), `#story-log[data-editing-entry]` (root lock), entry `data-editing`, entry `data-draft-game`
- **States**: idle; editing (with the three flags); saving (entry still in edit, lock held); released-after-save (root locked, `editState === null`, entry no longer in edit — the deliberate window the CSS comment names at `assets/styles.css:1554-1557`)
- **Transitions**: idle → editing ← `showEditForm` `:59`; editing → released-after-save ← `onEditSaved` done `:135-146` (unlock on `afterScreenShows` `:146`); editing → idle ← other failure `:139-142` → `revertEdit` `:113`, Escape `:124`, tab change `:245`
- **Guards / invariants**: refused and invalid answers keep the draft and the lock (`:136-137`); one edit at a time (`:60`); the lock may outlive `editState` but not the other way round
- **Explicitness**: FULLY IMPLICIT (a nullable object, two DOM flags and a promise-held release)
- **Already modeled**: no
- **Risks seen**:
  - known: `unlockEditing` depends on `afterScreenShows`, which never resolves without a successful load. The log then stays fully locked with no entry in edit; only a tab change (`screen:cleared` `:245`) frees it.
  - known: `showEditForm` sets `editState` before it looks the entry up (`:61-62`). If the entry is missing, `editState` is non-null with `text === null` and the root is never locked; the next Edit click is refused by `:60` until a `screen:cleared`.
  - known: `revertEdit` never clears `editState.text` before `clearEditState`, so a revert then a re-open is fine only because `showEditForm` rebuilds the object.

### Story-log scroll follow
- **Where**: `assets/dashboard-story-log.js:184` (`storyLogSwapStart`), `:186-200` (measurements), `:202-222` (`followStoryLogBottom`), `:224-242` (listeners)
- **State variables**: `storyLogSwapStart: null | {atBottom, entries}` (valid only between `screen:beforemorph` and `screen:loaded`), the per-frame retry loop bounded by `STORY_LOG_FOLLOW_TIMEOUT_MS` (`:205`), `editState` as a veto
- **States**: following (was at the bottom), holding (player scrolled up), retrying (box cannot scroll yet), stopped (deadline reached or an edit opened)
- **Transitions**: capture ← `screen:beforemorph` `:229-236`; decide ← `screen:loaded` `:238-248`; retry ← `requestAnimationFrame` `:214`; stop ← deadline or `editState !== null` `:212-213`
- **Guards / invariants**: the follow happens only when the log was at the bottom *and* the entry count grew
- **Explicitness**: FULLY IMPLICIT
- **Already modeled**: no
- **Risks seen**: known: `screen:beforemorph` fires for the whole screen, and a throw inside `morphParts` after that point skips `screen:loaded`, so `storyLogSwapStart` stays set until the next `beforemorph` overwrites it. No harm seen, since that is the next consumer.

### Edit-dialog session
- **Where**: `assets/dashboard-dialog.js:29` (`dialogOpenerId`), `:60` (`dialogSession`), `:61-62` (`noticeSummary`, `unreachableText`); dialog state lives on `#edit-dialog` (`assets/index.html:220-223`) in `open` and `data-saving`
- **State variables**: `editDialog.open`, `data-saving` presence, `dialogSession: int` (a generation token), `dialogOpenerId: string`, `noticeSummary` and `unreachableText` (mirrored into `#edit-dialog-status`)
- **States**: closed; open (draft); saving (draft, no second submit); closed-by-session-match; closed-by-user while saving (the pending close is skipped)
- **Transitions**: closed → open ← `htmx:afterSwap` into `#edit-dialog-body` `:31-40`; open → saving ← a done answer of a form inside `:115-126`; saving → closed ← `afterScreenShows` + session match `:124-127`; saving → open ← `[data-close-dialog]` `:95-100` or Escape (`close` event `:87-93`); a swap from outside is dropped while open `:43-49`
- **Guards / invariants**: `data-saving` ⇒ a form of the dialog is in flight; closing always bumps `dialogSession`; a save that waits never closes a later session
- **Explicitness**: EXPLICIT STATES, SCATTERED TRANSITIONS (two attributes, one counter, five listeners)
- **Already modeled**: no
- **Risks seen**:
  - known: `data-saving` is cleared only on `close` (`:90`). If the screen never shows the stamp, the dialog says "Saving…" forever and `htmx:confirm` refuses every later submit of that form (`:107-113`). Escape is the only exit and it discards the draft.
  - known: `dialogOpenerId` is overwritten by *any* swap into the body (`:33`). A Test button inside the dialog makes itself the opener, so focus no longer returns to the control that opened the dialog.

### Notice place (report with a source)
- **Where**: `assets/dashboard-errors.js:113` (`noticeElement`), `:117-129` (`noticeShows`), `:131-139` (`showNotice`), `:141-150` (`clearNotice`); the element is `#notice` (`assets/index.html:96-103`)
- **State variables**: `#notice.hidden`, `#notice.dataset.source` ∈ absent | `request` | `screen`, and the rendered summary/detail text
- **States**: no report; a request report; a screen report
- **Transitions**: show ← `reportAnswer` `:152-167` and `showNotice(..., "screen")` `:131`; clear ← `htmx:beforeRequest` `:171-174` (every htmx request, reads included), `data-close-notice` click `:176-182`, `clearScreenError` (`dashboard-screen.js:238-241`) on a good load or a matching ask
- **Guards / invariants**: one report at a time; the same report is not re-rendered (`noticeShows`, which keeps an open Details popover)
- **Explicitness**: FULLY IMPLICIT (`hidden` plus a string discriminator)
- **Already modeled**: no
- **Risks seen**: known: identity is the summary text, so a second failure whose first line equals the shown one keeps the *first* detail lines and raises no `notice:changed`, which leaves the dialog status line stale.

### Unreachable health (failure banner)
- **Where**: `assets/dashboard-errors.js:49-62` (`refreshFailureBanner`), `:64-76` (`markUnreachable`), `:78-85` (`clearUnreachable`); the elements are `#failure-banner`, `#failure-banner-degraded` (a screen part) and `#failure-banner-unreachable` (client-only)
- **State variables**: `#failure-banner.hidden`, `.unreachable` class, `role` ∈ `status | alert`, the two children's `innerHTML` emptiness, `unreachableText` in the dialog
- **States**: quiet; degraded only (server role health, from the part); unreachable only; both; recovered
- **Transitions**: unreachable ← any fetch failure (`dashboard-screen.js:279`, `:315`) or a `0` status answer (`:154-157`); cleared ← any successful response (`dashboard-screen.js:268`, `:317`; `dashboard-errors.js:158`)
- **Guards / invariants**: the degraded half is server-owned and only reloads on `screen:loaded` (`:184`)
- **Explicitness**: FULLY IMPLICIT
- **Already modeled**: no (role health is server-side; this client half is not)
- **Risks seen**: known: one successful response clears the banner for every request kind, so the 1 s ask can clear it while a POST is still failing. The tracked state is per response, not per connection.

### Saved-on-change control (rendered value vs current value)
- **Where**: `assets/dashboard-saves.js:10-35` (`settleField`/`restoreRenderedValue`), `:43-49` (`flagSaved`), `:55-72` (`settleSave`), `:75-78` (`clearPostureStatus`)
- **State variables**: each control's live value (`value`/`checked`/`selectedIndex`) against its rendered baseline (`defaultValue`/`defaultChecked`/`defaultSelected`); `data-saved-flag` → the id of the "Saved" element; `data-saved` on that element (`src/adapters/driving/http/settings/templates/settings.rs:312`, `src/adapters/driving/http/worlds/templates/worlds.rs:132`)
- **States**: saved (live == rendered, no flag); dirty (live != rendered); saving; saved-and-flagged
- **Transitions**: done → baseline promoted + flag after the screen shows ← `:61-63`; not done → baseline restored (a select/input) or the whole form reverted (`data-autosave`) + flag cleared ← `:65-69`; flag cleared by a form submit inside the World dialog ← `:75-78` (`worlds.rs:88`)
- **Guards / invariants**: the flag is set only when the screen shows the stamp; a failed save must never leave a green "Saved"
- **Explicitness**: FULLY IMPLICIT (a hidden second value per control plus one attribute)
- **Already modeled**: no
- **Risks seen**:
  - known: `data-saved` has no expiry and is kept through loads (`data-keep`). "Saved" stays on screen until a later failed save of the same control.
  - known: the flag waits on `afterScreenShows`, so no successful load means no flag at all, with no timeout and no error.
  - known: `#world-posture-status` (`worlds.rs:132`) has no `data-keep="data-saved"`, unlike the text-check flag. It survives because it is inside the dialog, not a part — an invariant of the markup, not of the flag.

### Tab, Settings sub-tab and saved-Games page
- **Where**: `assets/dashboard-panels.js:9-25` (`activateTab`), `:35-45` (`selectSettingsSubtab`), `:47-63` (restore), `:73-83` (`restoreActiveTab`); `assets/dashboard-screen.js:31-46` (`rememberGamesPage`/`restoreGamesPage`), `:388-397` (pager click)
- **State variables**: `.tab.active` / `.tab-content.active` / `aria-selected`; `.settings-subtab[aria-selected]` and `.settings-subtab-panel[hidden]`; `<html data-games-page>` (absent = page 1); the three `sessionStorage` keys
- **States**: which tab is open, whether that tab's parts hold content, which sub-tab, which Games page. Implied state: a closed tab's parts are *empty*, which is how "not loaded" is encoded (`leaveSection` `:400-408`)
- **Transitions**: tab ← click `:90-93`/restore `:78`; sub-tab ← click `:98-105`/a load `:47-63`; page ← click `:395`, restore `:35`
- **Guards / invariants**: a load for a section that is no longer visible is dropped (`:274`)
- **Explicitness**: EXPLICIT STATES, SCATTERED TRANSITIONS
- **Already modeled**: no
- **Risks seen**:
  - known: `data-games-page` is never reconciled with the server's clamped page (`src/domain/model/saved_games_page.rs:clamp_page`). After deletions move the list to fewer pages, the client keeps asking for the missing page and `sessionStorage` restores it after a reload. The panel shows `Page 1 of 1` while `<html data-games-page>` still says `3`.
  - known: `activateTab` calls `leaveSection` on the *leaving* tab before the new tab loads, so a save in flight in the leaving tab loses its host element while `requestsInFlight` still holds it.

### Slash menu
- **Where**: `assets/dashboard-slash-menu.js:8-10` (`slashMenu`, `slashMatches`, `slashActiveIndex`), `:16-28` (`closeSlashMenu`), `:87-99` (`openSlashMenu`/`selectSlash`), `:101-111` (`setSlashActive`)
- **State variables**: menu node presence, `slashMatches: []`, `slashActiveIndex: -1 | 0..n-1`, `aria-expanded`, `aria-activedescendant`
- **States**: closed; open with no match (closes); open with matches and index 0; open with a moved index
- **Transitions**: open ← `input` event `:113-118`; index moves ← ArrowDown/ArrowUp `:122-134`; select ← Enter `:135-141` or `mousedown` `:148-157`; close ← Escape, submit `:175`, outside `mousedown`, `focusout` after 150 ms `:164-173`, `scroll`/`resize` `:178-179`
- **Guards / invariants**: `slashActiveIndex ∈ [-1, slashMatches.length - 1]`; `openSlashMenu` recomputes both together (`:87-91`)
- **Explicitness**: FULLY IMPLICIT
- **Already modeled**: no
- **Risks seen**: known: `selectSlash` writes the input value programmatically (`:93-99`), so no `input` event fires and `slashMatches` keeps a non-empty list with `slashMenu === null` until the next keystroke. Every reader guards on `slashMenu` (`:120-123`), so the invalid pair is currently inert.

### Error detail popover (Details)
- **Where**: `assets/dashboard-errors.js:16-29` (markup with `data-keep`), `:188-201` (`closeErrorPopover`/`closeAllErrorPopovers`), `:216-237` (toggle click), `:239-247` (Escape, resize, scroll)
- **State variables**: `popover.hidden`, `toggle[aria-expanded]`, both in `data-keep`
- **States**: closed; open and positioned
- **Transitions**: open ← toggle click `:221-229`; close ← same toggle, outside click, Escape `:239-245`, resize, scroll
- **Guards / invariants**: one toggle per popover id (`aria-controls`); a load keeps both attributes, so the popover survives a morph
- **Explicitness**: FULLY IMPLICIT (`hidden` plus a mirrored ARIA attribute)
- **Already modeled**: no
- **Risks seen**: known: two attributes encode one state and are written in two places (`renderInlineError` renders `aria-expanded="false"` with `hidden`, the click handler writes both). A load that replaces the toggle but keeps the popover (different `data-keep`) splits them. Inferred; not seen.

### Live-region announcements (first time vs later)
- **Where**: `assets/dashboard-action-area.js:110` (`announcedStatusLabel`), `:140` (`announcedEntryIds`), `:171` (`announcedOptions`), `:112-136`, `:142-169`, `:173-192`
- **State variables**: three module-level memories; two are `null` (unprimed) and one is the string `"Ready"`
- **States**: unprimed (the first `screen:loaded` primes and announces nothing); primed and unchanged (no announcement); primed and changed (announce)
- **Transitions**: prime ← the first `screen:loaded` after page load; announce ← a later `screen:loaded` that includes the part
- **Guards / invariants**: `announcedStatusLabel` is *not* updated when the display holds an error disclosure (`:113-115`)
- **Explicitness**: FULLY IMPLICIT
- **Already modeled**: no
- **Risks seen**: known: after a failure line, a return to the same label that preceded it (usually "Ready") is not announced, because the label was not recorded during the failure.

## Doc vs code mismatches (`docs/diataxis/reference/frontend/dashboard.md`)

| Doc sentence | Code | Verdict |
|---|---|---|
| "The end of every request that is not a read forgets the stamp **and asks**." | `dashboard-screen.js:93-99` forgets the stamp for non-reads but calls `askNow()` for every request, reads included | mismatch (the ask is not limited to non-reads) |
| "`data-state` … The CSS reads it for the states that only the page knows." | `assets/styles.css:695-696` reads only `[data-state="generating"]`; `checking`, `preview`, `autoplay`, `idle` are read by no shipped code (tests read them) | over-stated |
| "The action area is in one of five states: Idle, Checking, Generating, Autoplay, and Preview." | `dashboard-action-area.js:64-69` sets an unstated priority, so `autoplay` hides both an open preview and `generating`, and `checking` hides everything | doc omits the priority |
| "`data-screen-busy` — On `<html>`: an ask or a load is in flight." | written at `dashboard-screen.js:347-348`; no shipped script or CSS reads it, only the test harness | marker is test-facing only |
| "`data-game` … It is empty with no active Game." | the shell has no `data-game` attribute at all (`assets/index.html:2`); it becomes `""` after the first load | cosmetic (absent ≈ empty) |
| "The server renders data and control states. The browser owns view state. No script writes `disabled`." | holds: no JS writes `disabled`; the dim-and-block behaviour is CSS (`styles.css:693-698`, `1568-1576`) | matches |
| "The action area … reads the status display and writes nothing into it." | holds (`#action-status` is a separate element, `index.html:163`) | matches |

## Table of contents

| name | file | explicitness |
|---|---|---|
| Action-area view state | assets/dashboard-action-area.js:5,10,89 | FULLY IMPLICIT |
| Screen stamp and waiters | assets/dashboard-screen.js:170-211 | FULLY IMPLICIT |
| Ask / load loop | assets/dashboard-screen.js:342-380 | FULLY IMPLICIT |
| What a load keeps | assets/dashboard-screen.js:72-119 | FULLY IMPLICIT |
| Story-log edit lifecycle | assets/dashboard-story-log.js:17-146 | FULLY IMPLICIT |
| Story-log scroll follow | assets/dashboard-story-log.js:184-242 | FULLY IMPLICIT |
| Edit-dialog session | assets/dashboard-dialog.js:29-127 | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Notice place source | assets/dashboard-errors.js:113-150 | FULLY IMPLICIT |
| Unreachable health | assets/dashboard-errors.js:49-85 | FULLY IMPLICIT |
| Saved-on-change control | assets/dashboard-saves.js:10-78 | FULLY IMPLICIT |
| Tab / sub-tab / Games page | assets/dashboard-panels.js:9-83, assets/dashboard-screen.js:31-46 | EXPLICIT STATES, SCATTERED TRANSITIONS |
| Slash menu | assets/dashboard-slash-menu.js:8-111 | FULLY IMPLICIT |
| Error detail popover | assets/dashboard-errors.js:188-247 | FULLY IMPLICIT |
| Live-region announcements | assets/dashboard-action-area.js:110-192 | FULLY IMPLICIT |

Note: the server side of the action controls (`Busy` → `disabled`, Stop only during a run: `src/adapters/driving/http/layout/handlers/action_controls.rs:20-42`, `endpoints.rs:69-107`) is a *second, independent* encoding of the already-modeled busy registry, re-derived per part render. It is not its own state machine, but every part must read it in the same request to stay consistent, and nothing checks that.