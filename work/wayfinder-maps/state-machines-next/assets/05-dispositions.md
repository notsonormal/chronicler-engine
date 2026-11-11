# Ticket 05: the rule applied to the dashboard in the browser

Date: 2026-11-09. Read in the working tree at `8eb0b057` plus other agents' uncommitted work. No file of `src/`, `tests/`, `docs/` or `assets/` changed.

Labels: **known** means I read the code path end to end, or I ran it. **inferred** means I read part of it. **unsure** means I did not check.

I ran one check. It is in `/tmp/g4check/` and is not part of the repo. It uses `assets/htmx.min.js` (htmx 1.9.10) and headless Chrome. It answers the question of the detached element (row G4, below).

## 1. How I read the rule

Ticket 01 gives the tests in words. Three words need a reading for browser JS. I give the reading so ticket 06 can change it.

| Word of the rule | My reading | Why |
|---|---|---|
| "place" (model-based test: "two or more places make its moves") | A function or a listener that assigns the state of the machine (a module variable or a DOM attribute). I count each one. | The catalogue counts "six `gate.rs` methods" of B1 as scattered. I use the same unit. |
| "parts that run at the same time" (TLA+ test) | Separate actors whose events interleave: the page, the engine, a request in flight, a timer. Two event handlers of one page in one task are not parts. | The slash menu and the popover have no second actor. |
| "a client can see a different result in some order" | Two orders of the same events give two results, and the spec needs the right one (a race). A register where the last writer wins by design is not a race. | The notice place (G9) is last-writer-wins by design in `dashboard.md:158`. |

**Effect of the first reading.** It gives the model-based test to 14 of the 15 machines. Only G3 has one writer. I report this as a result and not as a defect of the reading. The first harness costs the most. Each later reference machine is a table of states and events in the same harness. See section 7, point 1.

## 2. Counts per disposition

15 machines (G1 to G15). A machine can earn more than one.

| Disposition | Machines | Count |
|---|---|---|
| Explicit in the code | G1, G4 | 2 |
| Model-based test | G1, G2, G4 to G15 (all but G3) | 14 |
| TLA+ model and verify skill | G1, G2, G3, G4, G6, G8, G12 in a new `verify-screen` skill; G5 in `verify-gate` | 8 |
| Diagram | G1, G2, G3, G4, G5, G6, G8, G12 (each has a TLA+ model or a transition table) | 8 |
| Leave it | none | 0 |

Both "explicit" machines have a **known** path to a state that is not valid. Their code sites are in section 4.

## 3. The disposition table

Each cell is short. The evidence of each row is in section 4.

| Row, machine | State that is not valid, and its path | Places that make the moves (count, file:line) | Concurrent parts, client-visible order difference | Prose in `docs/diataxis/` | Dispositions and form | Behaviour question |
|---|---|---|---|---|---|---|
| **G1** action area | "Idle" or no "Thinking..." while an accepted send still waits for its screen. **Known** path: Retry or Retrigger, then a second Retry, Retrigger or New options that gets `409`. `endPending` (`dashboard-action-area.js:25-29`) runs from `endPendingAfterRequest` (`:306-310`) on the refused answer. The one flag `sendPending` (`:10`) holds the pending state of every sender. Window: until the screen shows the stamp of the first send. Plain defect, filed as 31. | 5 writers: `beginPending :20`, `endPending :25`, `onCommandBeforeRequest :208`, `onCommandAfterRequest :270`, `onCommandLoadError :265`. One derivation writes the attribute: `applyActionState :74`. 8 hx-on sites call them: `index.html:123-125,:147`, `templates.rs:25` (Retry, Retrigger), `templates.rs:64` (New options). | The engine's Job, the status display load, and the page's pending send run at the same time. Order: the pending end and the status display "thinking" must overlap. Known orders: Scenario 16.69 and 16.70 (`browser_dashboard.md:683-701`). | `dashboard.md:61-67` (five states, no priority). | Explicit: one state object `{checking, pending: count, preview}` and one function that makes the moves and alone writes `data-state` and `#action-status`. Model-based test (explicit). TLA+ in `verify-screen` (pending end rule). Diagram from the table of the derivation (`:64-70`) and from the TLA+ model; it replaces `:61-67`. | none |
| **G2** screen stamp and waiters | none known. A shown stamp can be ahead of a skipped control, but `endRequest` forgets the stamp (`dashboard-screen.js:97`) to cover it. Known and by design (`browser_screen.md:304`, Scenario 44.27). | 5 writers: `morphParts :303`, `forgetScreenStamp :170-172` (called from `:97`, `:396`, `:407`, `dashboard-story-log.js:109`), `afterScreenShows :193-199`, `resolveScreenWaiters :201-212`, `loadScreen :254` (`loadsStarted`). 8 consumers of the waiter: `action-area :32,:216,:251`, `story-log :144,:161,:177`, `saves :63`, `dialog :123`. | Engine revision, an ask, a load and a request end run at the same time. **Known** order differences: a load in flight when a wait for another server starts (Scenario 44.40, `browser_screen.md:430-440`; `loadsStartedBefore` at `dashboard-screen.js:197,:207`); a request end before or after a load that skipped its control (Scenario 44.27). | `dashboard.md:31,:85-97`, `:125` (marker). | Model-based test. TLA+ in `verify-screen`: the waiter rule is one property for all 8 consumers. Diagram from the TLA+ model; it replaces `:85-97`. | none |
| **G3** ask and load loop | none known. | 1 writer: `askNow :351-370` (`askChain`, `queuedAsk`, `asksPending`). `markScreenBusy :346` writes `data-screen-busy`, which only tests read (`tests/test_utils/screen.rs:18,:32`, `convergence.js:19`, `browser/stub/screen.rs:903`). 8 callers of `askNow`. | The ask, the load, the poll timer (`:372-377`) and the end of a request. **Known** order: `forgetScreenStamp` (`:97`) must come before `askNow` (`:98`). With the reverse order the ask finds the same stamp and skips the load. | `dashboard.md:85-89`. | TLA+ in `verify-screen` (the model needs the ask chain to state the rule of the stamp). Diagram from the model. No model-based test: one writer. | none |
| **G4** what a load keeps | `requestsInFlight` (`dashboard-screen.js:72`) keeps a detached node whose request ended. **Known** path (run, htmx 1.9.10): the player presses Save in the edit dialog and then Escape. The `close` handler empties the body (`dashboard-dialog.js:87-93`). The request ends, and `htmx:afterRequest` reaches the node but not `document.body`. `endRequest` (`dashboard-screen.js:93-99`) never runs: no release, no `forgetScreenStamp`, no `askNow`. `reportAnswer`, `settleSave` and the dialog's answer handler (`dashboard-errors.js:169`, `dashboard-saves.js:71`, `dashboard-dialog.js:115`) never run either. `leaveSection` (`:400-404`) has the same effect on a part control (**inferred**: same htmx path, not run). | 2 writers: `holdInFlight :74-76`, `releaseInFlight :78-82`. Readers: `loadKeeps :114-116`, `beforeNodeMorphed :118`, `beforeNodeRemoved :122`. | A load answer and a request end. **Known** orders: Scenario 44.24 (`browser_screen.md:272`), 44.27 (`:304`), 44.32 (`:356`). | `dashboard.md:18,:97` (the sentence at `:18` is wider than the rule), `:118` (a save before Escape still applies). | Explicit: hold and release by the xhr of the request, so a detached node cannot lose the release (form idea; the work ticket decides). Model-based test (explicit). TLA+ in `verify-screen` (keep rule). Diagram from the table. | Question 4: must a failed save after a close show a notice? |
| **G5** the Game a request names | none known. Bug 07 (preview send, filed) is the only defect near it. The three writers of the command input value all end in a sync (section 5, item 3). | 6 writers: `syncCommandDraft :230-238`, `startNewCommandDraft :241-245`, `retakeDraftGameWhenScreenShows :250-257`, `showEditForm :95`, `clearEditState :106`, `morphParts :301` (`data-game`). One reader: `htmx:configRequest :420-428`. Server side: `refuse_stale_game` (`request_layers.rs:39-56`). 3 files. | The engine's active pointer, the page's shown Game, and the draft pin. **Known** order: the engine switches while a load that shows the new Game is in flight (Scenario 16.56, `browser_dashboard.md:550`). The page names the old Game until the load ends. The server answers `409` (16.55, `:539`; 16.57, `:562`). | `dashboard.md:33-40` (drafts name their Game). | Model-based test. TLA+: extend `verify-gate` (it shares `active` with `VerifyGate.tla`). Diagram from the TLA+ model. | none |
| **G6** story-log edit | none known. Scout claim "`showEditForm` sets `editState` before the entry lookup" (`dashboard-story-log.js:61-62`): no path. `data-id` exists only on story entries and Games rows (`templates.rs:25`, `games.rs`), and the Games parts are empty while the Game tab is open. | 5 writers: `showEditForm :59`, `clearEditState :101`, `lockEditing :34`, `unlockEditing :39`, the `screen:cleared` handler `:243`. Attributes: `data-editing` (`:94`, `:105`), `data-editing-entry` (`:36`, `:41`). | The load, the save and the lock. **Known** order: the lock ends only after the screen shows the saved entry (`:133-145`). Otherwise a second editor opens on a stale textarea. Scenario 44.10 (`browser_screen.md:122`). | `dashboard.md:97-103`, `:130-131`. | Model-based test. TLA+ in `verify-screen` (lock end rule). Diagram from the TLA+ model; it replaces `:99-103`. | none |
| **G7** story-log scroll follow | none known. | 2 writers of `storyLogSwapStart`: the `screen:beforemorph` handler `:223-230` and the `screen:loaded` handler `:232-241`. The retry loop `followStoryLogBottom :207-221` reads `editState`. | The animation frame loop and a load. Order effect is only the scroll position. | none | Model-based test. | none |
| **G8** edit dialog session | none known apart from the G4 loss. A done save sets `data-saving` (`dashboard-dialog.js:121`), and only `close` clears it (`:90`). This holds "Saving…" until the screen shows the stamp, as Scenario 44.43 requires. | 7 writers: the `afterSwap` handler `:31` (`dialogOpenerId`), `close` handler `:87` (`data-saving`, `dialogSession`), `afterRequest` handler `:115` (`data-saving`), `renderDialogStatus :66`, `notice:changed :77`, `banner:changed :82`, the click handler `:95` (`close()`). | The save request, the screen load and Escape. **Known** order: a save that ends after Escape reaches no handler (G4, run). A done save closes the dialog only after the screen shows its stamp (`:123-125`). | `dashboard.md:105-118`. | Model-based test. TLA+ in `verify-screen` (session and close rule). Diagram from the TLA+ model; it replaces `:105-118`. | Question 4 (shared with G4). |
| **G9** notice place | none known. Scout claim "a second failure with the same first line keeps the old details" is **refuted**: `noticeShows` (`dashboard-errors.js:117-126`) compares the summary and the detail. | 2 writers: `showNotice :131-139`, `clearNotice :141-148`. | Request reports and screen reports write one place. The order is "the latest report wins" by design (`dashboard.md:158`). It is not a race. | `dashboard.md:156-162`. | Model-based test. | none |
| **G10** failure banner | none known. `refreshFailureBanner :49-62` derives `hidden`, the class and the `role` from the two slots, and each writer ends with a call to it. | 3 writers: `markUnreachable :64`, `clearUnreachable :78`, the engine's morph of the degraded slot (followed by `screen:loaded` at `:184`). One function writes the attributes. | none. The two slots are independent flags. | `dashboard.md:156-162`. | Model-based test. | none |
| **G11** Saved flag | none known. A later failed save restores the saved value (`dashboard-saves.js:66-68`), so the flag stays true to the control. | 2 writers: `flagSaved :43-49`, `clearPostureStatus :75-78`. The server keeps `data-saved` through a load (`settings.rs:312`). | The save and the screen load. This is the generic waiter rule of G2 (`:63`). | `dashboard.md:31`, `:133-134`. | Model-based test. | Question 2: when does "Saved" go away? |
| **G12** tab, sub-tab, saved-Games page | none known. A page number past the last page is clamped by the engine (`dashboard.md:95`, `games.md` Scenario 20.44). This is a state that the design allows. `data-games-page` keeps the asked number; the pager control carries the clamped neighbour. | 7 writers: `activateTab dashboard-panels.js:9`, `selectSettingsSubtab :35`, the `screen:loaded` handler `:47`, `rememberActiveTab :65`, `rememberSettingsSubtab :27`, `rememberGamesPage dashboard-screen.js:24`, the pager click `:391-398`. 2 files. | The tab click and a load in flight. **Known** order: the old answer is dropped (`dashboard-screen.js:274`; Scenario 27.10, `browser_games.md:116`). | `dashboard.md:43`, `:89-95`. | Model-based test. TLA+ in `verify-screen` (the drop rule). Diagram from the TLA+ model. | none |
| **G13** slash menu | none known. | 5 writers: `closeSlashMenu :16`, `renderSlashMenu :43`, `openSlashMenu :87`, `selectSlash :93`, `setSlashActive :101`. | none. Timers and events of one page. | `dashboard.md:50` (one line). | Model-based test. | none |
| **G14** error detail popover | none known. | 4 writers: the toggle click `dashboard-errors.js:216-235`, `closeErrorPopover :188`, `closeAllErrorPopovers :197`, `positionErrorPopover :203`. | none. | `dashboard.md:44`. | Model-based test. | none |
| **G15** live-region announcements | none known. See question 5 for the Game switch. | 3 writers, one memory each: `announceStatusDisplay :112` (`:110`), `announceNewNarration :142` (`:140`), `announceOptions :173` (`:171`). | none. | none in `dashboard.md` (no live-region prose). | Model-based test. If the owner decides that a Game switch must reset the memories (question 5), the path to a state that is not valid becomes known and the machine also earns "explicit". | Questions 5 and 6. |

## 4. Evidence by machine

### G1 action area

- **Known.** `deriveActionState` picks one state in this order: checking, autoplay, generating, preview, idle (`dashboard-action-area.js:64-70`). The inputs are `checkInFlight` (`:5`), `sendPending` (`:10`), the status display (`:40-53`) and the preview (`:59-62`).
- **Known.** `checkInFlight` is sound. The command form is in the shell (`index.html:117-125`), so no tab change detaches it. A swap error ends in `onCommandLoadError` (`:265`). A timeout or an abort fires `htmx:afterRequest` (htmx source: `b.onerror`, `b.onabort`, `b.ontimeout`).
- **Known.** `sendPending` is one boolean with several writers. Retry and Retrigger have no `hx-sync` (`templates.rs:25`; the only `hx-sync` in that template is on Save). New options has `hx-sync="this:drop"` (`templates.rs:64`), which only stops a second click on the same control.
- **Known.** The 409 comes from the busy registry for any second accept while a Job holds the Game (`verify-gate`).
- **Inferred.** The window is short: one round trip plus one ask. The server renders Send as enabled until the next load, and a click on it gets `409`. This is why I filed the defect as low (31).
- **Known (read).** The scout claim `afterScreenShows(null)` is harmless: every `POST` answer carries `X-Revision` (`request_layers.rs:19-33`, `screen.md` Scenario 45.1).

### G2, G3 screen stamp, waiters, ask loop

- **Known.** `screenShows` is true when the server id is equal and the revision is greater or equal (`dashboard-screen.js:179-185`). Only a good `morphParts` resolves waiters (`:306`).
- **Known.** `morphParts` sets the stamp after any load, also a load that writes no part (`:301-306`). So a waiter ends after a tab change as well.
- **Known.** The load of an old tab is dropped at `:274` and does not resolve waiters. The change that dropped it already asked again (`:23`, `dashboard-panels.js:24`).
- **Known.** The stamp the answer carries is read before the render (`screen.rs`, S.1 in `storage.md:62`). A busy change during the render shows at most as a newer part under an older stamp. The next ask loads again.
- **Known.** The three screen parts of the action area read the busy registry apart (`action_controls.rs:54,:58`, `view_query.rs:198`). One load can show Send enabled next to a busy status display. The next ask corrects it, because every busy change raises the stamp (`changes.rs:34`, `gate.rs:188-200`). No disposition for this read. It stays with ticket 04 (server).

### G4 what a load keeps

- **Known (run).** In htmx 1.9.10, `htmx:afterRequest` reaches the element itself. It reaches an ancestor only when the element is still in the page (`b.onload`, `Ir`, `se` in `assets/htmx.min.js`). The check page showed this log for a form that was emptied away while its request was in flight:

  ```
  body beforeRequest f
  detached
  own afterRequest
  ```

  There was no "body afterRequest".
- **Known (read).** No `cancel` handler exists on the dialog (`dashboard-dialog.js`, whole file). Escape closes it, also while `data-saving` is on or a request is in flight. The `close` handler detaches the form (`:87-93`).
- **Known.** `htmx:confirm` runs before `htmx:beforeRequest` (`dashboard-dialog.js:107-113`), so a refused second submit never holds a node.
- **Inferred.** `leaveSection` (`:400-404`) loses the same events for a control in a part. A tab change while a Settings select saves would not report a failure. Not run.

### G5 the Game a request names

- **Known.** All 11 game-bound routes sit behind one layer (`router.rs:25-62`, `request_layers.rs:39-56`). A dialog form posts to Games, Settings, Worlds or Presets routes, and none of them is game-bound. So a dialog form needs no `data-draft-game`.
- **Known.** The draft pin is set in `syncCommandDraft` on the first keystroke (`:234-236`) and cleared on an empty input (`:234`).

### G6, G7, G8 view state of the story log and the dialog

- **Known.** `showEditForm` takes `document.querySelector('[data-id=…]')` without a scope (`dashboard-story-log.js:62`). `data-id` exists only on story entries and Games rows, and the Games part is empty while the Game tab shows. No collision.
- **Known (by design).** A load keeps an entry in edit even when the Message is gone or the Game changed (`isBeingEdited`, `dashboard-screen.js:110-112`). The Save then fails with a report, and Cancel removes the editor. Scenario 16.55 (`browser_dashboard.md:539`) states the Game case. The delete case by another client has no scenario but the same code path. This is the "ghost" of the ticket. It is the wanted behaviour: the draft is the player's text (`dashboard.md:101`).
- **Known.** Delete and Retrigger are hidden on the entry in edit (`styles.css:1456`) and visible on the others. Only Edit and the swipe controls are locked (`styles.css:1558-1559`). `dashboard.md:99` says exactly this.

### G9 to G15

- **Known.** The poll and the screen load use `fetch` (`dashboard-screen.js:260,:312`), not htmx. They do not clear the notice at `htmx:beforeRequest` (`dashboard-errors.js:171-174`).
- **Known.** No `hx-trigger` of the app is `load`, `every` or `revealed`. The only value is `change` (12 uses: `games.rs` 7, `settings.rs` 2, `worlds.rs` 3). So only a player action starts an htmx request.
- **Known.** `selectSlash` writes `input.value` (`dashboard-slash-menu.js:94`) and does not call `syncCommandDraft`. The menu opens only from an input event (`:113-118`), so the draft is already pinned.
- **Known.** The slash menu is built in `document.body` (`:49-54`), and scroll, resize, submit and focus loss close it (`:164-179`).

## 5. "To check" results

| Item | Result |
|---|---|
| G1: `sendPending` and `checkInFlight` are not reset by `screen:cleared` or a Game switch. | **Known.** `checkInFlight` lives in the shell and ends with its own request. `sendPending` ends at the next good load of any tab (`morphParts :303-306`). The area is hidden while another tab shows. `screen:loaded` calls `syncActionState` when the Game tab loads (`dashboard-action-area.js:200-204`). Nowhere. |
| G1: the screen parts read the busy registry one by one. | **Known.** One load can show Send enabled and a busy status display. The next ask corrects it. No disposition. It stays with ticket 04. |
| G5: a script write to `input.value` fires no `input` event. | **Known.** Three writers: `useOption :321` and `editOption :330` call `startNewCommandDraft`. `selectSlash :94` needs no sync, because the menu opens only after an input event. `clearCommandInput :222-226` syncs. No path to a draft pin that is wrong. Nowhere. A model-based test of G5 would catch a new writer that forgets the sync. |
| G6: a deleted entry in edit stays as a ghost. | **Known**, and by design (see section 4). Nowhere. |
| G9: a background request clears a report that the player has not read. | **Known.** No background htmx request exists (section 4). The clear is reached only by a request that the player started. `dashboard.md:158` says the same. Nowhere. |

## 6. Doc checks

Files: `docs/diataxis/reference/frontend/dashboard.md`.

| Claim | Result | Where it goes |
|---|---|---|
| D1. A load skips an element with unsaved input (`:18`). | Imprecise. `:97` gives the true rule: two kinds, the entry in edit and a control in flight. A part form with typed text does not exist (`:19`). | Wording fix in defect 30. |
| D2. `data-revision` is forgotten at the end of a request that is not a read (`:125`). | Incomplete. The code also forgets it in `leaveSection` (`dashboard-screen.js:407`), in the pager click (`:396`) and in `clearEditState` (`dashboard-story-log.js:109`). | Defect 30. |
| D3. Stop in the Autoplay state through `data-state`. | The doc does not say so. `:67` says only that the action area is in Autoplay and shows Stop. The CSS shows Stop with `:has(#stop-btn)` (`styles.css:673-676`) and reads one `data-state` value, `generating` (`:695-696`). The values `checking`, `autoplay` and `preview` are read by tests only. | Nowhere. |
| D4. The client-state list leaves out several variables (`:41-50`). | True for `sendPending`, `checkInFlight`, `previewSentText`, `dialogSession`, the live-region memories, `screenWaiters`, `requestsInFlight`, `editState`, the slash palette state. The list names no variable. | Defect 30. If the TLA+ model and the diagrams replace the prose, the fix is no longer needed. |
| D5. Edit and swipe controls are locked, Delete and Retrigger only hidden. | The doc is right. `:99` names the locked controls. Delete and Retrigger are hidden on the entry in edit and not locked on the others. | Nowhere. |
| Scout: "forgets the stamp **and asks**" for every request. | The doc says the page asks when any request ends (`:85`). No conflict. | Nowhere. |
| Scout: "the CSS reads `data-state`" (`:148`). | The doc says "for the states that only the page knows". The one value is `generating`. Correct. | Nowhere. |
| Scout: the five states have no priority. | True. The priority is `dashboard-action-area.js:64-70`. | Defect 30. |
| Scout: `data-screen-busy` is listed as page state (`:128`). | The marker exists and the doc says what it means. Tests read it. No claim is wrong. | Nowhere. |

## 7. Forms and draft work tickets for the costly dispositions

Costs: S is under one day. M is one to three days. L is more. These are guesses (**unsure**). I did not run any work.

**Work ticket A: make G1 and G4 explicit (M).**

- G1: replace `checkInFlight` and `sendPending` with one state object `{checking, pending, preview}`. `pending` is a count of own accepted requests. One function `moveActionArea(event)` makes every move and alone writes `data-state` and `#action-status`. The hx-on sites call it with an event name. This also fixes defect 31.
- G4: hold and release the in-flight entry by the xhr of the request (`htmx:beforeSend` gives the xhr; its own `loadend` releases). A detached node can then not lose the release. One module function holds and releases. **Unsure:** the form needs a check against htmx 1.9.10. Done when a browser test closes the dialog during a save and sees the entry released.
- Question 4 must be answered first for the report. The release does not depend on it.

**Work ticket B: model-based tests of the browser machines (L for the first, S each after).**

- Reference machines live in the tests only. Production code does not change, except for A.
- Group the 14 machines into five reference machines: refresh (G1, G2, G4, G5, G12), view state of the log and dialog (G6, G7, G8, G11), reports (G9, G10, G14), palette (G13), announcements (G15).
- The harness runs generated event sequences against the real page in the stub browser, and compares the reference state with the page state after each step. **Unsure:** the platform choice (research 01 of the map) names the tool.

**Work ticket C: `verify-screen` skill with a TLA+ model (L).**

- Variables: `serverRev`, `serverId`, `shownStamp`, `waiters`, `askState` (idle, running, queued), `loadState`, `inFlight` (set of controls), `editing`, `lock`, `dialog` (open, saving, session), `tab`, `gamesPage`, `pending`.
- Environment actions: the engine raises its revision, the engine restarts (new `serverId`), a request starts or ends (done, refused, failed, none), a tab change, Escape.
- Rules (each quotes its scenario): a waiter resolves only when the screen shows its stamp, or after a load that started after a restart; the old tab answer is dropped; a load never writes a node in edit or in flight; a request end forgets the stamp before the ask; the lock ends after the screen shows the saved entry; a request that ends after a close still releases its hold (this one fails today, and so the TLA+ model finds the G4 loss).
- The busy state is a free input here (the status display shows "thinking" or "idle"). So the model shares no variable with `VerifyGate.tla`, and the area gets its own skill (see section 8, point 2).
- `scripts/verify_coverage.py` lists Rust functions only (docstring, lines 1-12). A coverage record for JS functions needs a hasher for JS (`function name(...) {...}`), and that is part of this ticket. **Known** for the docstring; **inferred** that it has no JS support.
- Cost evidence: the dashboard scripts changed 100 times since 2026-09-01 (`dashboard-errors.js` 21, `action-area` 20, `story-log` 19, `panels` 12, `dialog` 10, `screen` 10, `saves` 6, `slash-menu` 2). The refresh files (`screen`, `action-area`, `dialog`) changed 40 times. Each change needs a maintain pass.

**Work ticket D: extend `verify-gate` for G5 (M).**

- Add the page variables `shownGame` and `draftGame`, and the header check of `refuse_stale_game`. The model shares `active` and `pointerBy`.
- Scenarios: 16.55, 16.56, 16.57 (`browser_dashboard.md:539-575`).

**Work ticket E: diagrams (S each, after A and C).** Generated from the TLA+ model and from the table of G1. They replace `dashboard.md:61-67`, `:85-97`, `:99-103` and `:105-118`.

## 8. Points where I could not decide

1. **The "place" count.** The literal reading gives 14 model-based tests. Ticket 06 can ask the owner for a threshold (for example three writers). I did not invent one. A threshold of three writers keeps 10: G1, G2, G5, G6, G8, G10, G12, G13, G14, G15 (G4 stays by being explicit).
2. **The home of the refresh model.** The refresh rules and G5 share the busy registry and the active pointer with `VerifyGate.tla` only if the model reads them. I split the area: G1 to G4, G6, G8 and G12 take the engine as free inputs, so they get `verify-screen`; G5 reads `active` and extends `verify-gate`. If the owner wants one model, then the rule says it extends `verify-gate`, and the page variables join that file. This is a judgement about the abstraction, not a fact.
3. **Question 2 and the start of a new save.** "Saved" stays during the next request of the same control (`dashboard-saves.js:43-49`, flag cleared only at the end of a failed save). No spec says when it must leave. This is part of question 2.

## 9. Behaviour questions

| # | Machine | Question | Status |
|---|---|---|---|
| 1 | G2 | "Thinking…", "Saving…" and the edit lock stay until a load works. Keep this? | **Dropped.** `browser_screen.md` Scenario 44.43 (`:468`) and `browser_dashboard.md` Scenarios 16.45 (`:421`) and 16.69 (`:683`) state that they last until the next good load. |
| 2 | G11 | "Saved" has no expiry. Must it go away after a time, or when the next save starts? Specs only assert that the text shows (`browser_worlds.md:13,:57`, `browser_settings.md:54`). | **Kept.** Add the second half: "or when the next save of the same flag starts". |
| 3 | G8 | The Test button becomes the focus target. | **Dropped.** Refuted by ticket 02 (`settings.rs:199-202`, `:60`). |
| 4 | G4, G8 | A save that fails after the player closed the dialog shows no report, and its hold is never released. Must a failed save after a close show a notice? | **Kept, now evidenced.** I ran the htmx 1.9.10 check: the body listener does not see the end of the request of a detached element. `dashboard.md:118` says the save still applies. The doc's "one reader of every answer" (`:16`) does not hold here. |
| 5 | G15 | After a Game switch, `announceNewNarration` (`dashboard-action-area.js:142-160`) announces every Narration of the other Game in one block, and `announceOptions` (`:173-193`) announces its Options. What must a screen reader hear? | **Kept.** |
| 6 | G15 | After a failure line, a return to "Ready" is not announced (`dashboard-action-area.js:112-122`: the early return at `:114-116` leaves the memory at "Ready"). Is the silence wanted? | **New, low.** |

Count: 3 kept (2, 4, 5), 2 dropped (1, 3), 1 new (6).

## 10. Defects filed

| No. | Title |
|---|---|
| 30 | `dashboard.md` omits three forget points, the action-area priority and most of the client state |
| 31 | A refused request ends the pending state of an accepted send |

## 11. Draft `## Answer`

Resolved <date>. I applied the rule to G1 to G15. Two machines earn "explicit": G1 (the one `sendPending` flag, defect 31) and G4 (a request that ends after its node is detached never releases its hold; run with htmx 1.9.10). Fourteen earn a model-based test under the literal reading of "two or more places"; G3 has one writer. Eight earn a TLA+ model: seven in a new `verify-screen` skill (G1, G2, G3, G4, G6, G8, G12) and G5 as an extension of `verify-gate`. Eight earn a generated diagram. No machine is "leave it". The refresh rules earn a TLA+ model: they have parts that run at the same time and a client-visible order difference (Scenarios 44.27, 16.56; the restart wait, Scenario 44.40), and the model replaces `dashboard.md:85-97`. Kept questions: 2, 4, 5; new: 6. Two defects filed: 30 and 31. Open for ticket 06: the threshold for "place" and the home of the refresh model.
