---
diataxis: reference
title: Dashboard
---

## Overview

The dashboard is the player's single-page view of the engine, served at `/`. Play happens on the Game tab. The other tabs are management panels.

## Shell and Fragments

A static shell holds the page frame. The server owns all domain content: it renders each part of the screen as an HTML fragment from the current game state, and the shell loads the parts with one request.

Six statements hold for every part and every request:

1. **Data flows one way.** A `POST` saves data and answers only its outcome: a status code and the revision after its commit. A dialog-opener `GET` saves nothing, answers the form, and gets no revision. An invalid answer to a form with typed input shows in the slot of that form. Every other report shows in the notice place of the shell. The data reaches the screen only through a screen load. The page asks for the revision at once after any request, so the result shows without waiting for the next ask.
2. **The page shows one screen, loaded in one request.** The screen is the header, the failure banner and the parts of the visible tab. Any change moves the revision, and the visible screen loads. No request targets a part element, and no answer writes out of band.
3. **A screen load keeps what it does not change.** The page morphs each part into place. A kept node keeps its focus, selection, scroll position and view state. A load skips an element that holds unsaved input. A load that removes the focused node moves focus to the nearest row that survives, or else to the part.
4. **A load never writes into a draft.** A form that waits for a Save is in the edit dialog, outside every part. The one draft in a part is the story-log message in edit, which a load skips.
5. **The server renders data and control states. The browser owns view state.** No script writes `disabled`.
6. **A list sends short rows.** The full item loads when the player opens it. A list with no natural limit has pages.

Every `POST` answer carries `X-Revision`, the stamp `<server>:<revision>` of the engine after the request. A dialog closes, a "Saved" line shows and the Action input clears only when the screen shows that stamp. A refusal answers `409`. It means a busy Game, a page that shows a Game that is no longer active, or a Message that is no longer the last one. The engine took nothing and wrote nothing. An invalid value answers `400`, and a failure `500`. The status chooses the outcome, and the server writes the first line of every failed answer, which the page shows as the summary of the report. Delete, Retry and Retrigger send the id of the Message that their page rendered them on, and the engine compares it with the last Message in the same step as the busy check, so no write lands between the check and the act. The page sends `X-Chronicler-Game` with each request that is not a read, taken from the `X-Game` of its last screen load, and a route layer refuses a Game-bound request that names another Game than the active Game. The page keeps the Game of its last load after a create, a switch or a Reset that it sent itself. A send before the next load gets the `409` of a changed Game, and its text stays.

The shell owns only client state:

- the active tab and the active Settings sub-tab, which `sessionStorage` keeps for a reload;
- the open state of an error popover or a Details disclosure, which a load keeps (`data-keep`);
- the stamp of the screen that the page shows (`data-revision` and `data-game` on `<html>`);
- the report that the notice place shows, and its source (`data-source` on `#notice`);
- the id of the control that opened the edit dialog;
- the action-area state, and the action area's own status element `#action-status`;
- the Unreachable health state;
- the slash palette.

## Parts of the Game Tab

The page has a header, a failure banner, a tab bar, and the body of the active tab. The header shows the display name of the active Game.

The Game tab has six parts of the screen: the story log, the visual sidebar, the options dock, the send control, the autoplay controls, and the status display. The action area holds the last three and the controls that are no part: the Action input and the cycle budget.

- **Story log** — the story window of the Game: its newest Messages, in order.
- **Visual sidebar** — the image of the current room and the portraits of the Characters in it.
- **Options dock** — the Options that the player can pick from, or the Options failure of the Game.
- **Action area** — the Action input, the send control, the cycle budget, the autoplay controls, the text-check preview, and the status display.

The action area is in one of five states: **Idle**, **Checking**, **Generating**, **Autoplay**, and **Preview**. When the text check is on, the text check sits between a typed send and the request to the engine. The status display shows the step of the running Job, or the failure line of the last Job. The action area reads the status display and writes nothing into it. The state of a send that no Job has shown yet (Generating before the status display moves) shows in `#action-status`.

The action controls are two parts: `send-control` holds Send, and `autoplay-controls` holds Autoplay and Stop. The DOM order is the visual order, so Tab goes from the Action input to Send, the cycle budget, and Autoplay. The server renders Send and Autoplay as disabled while any Job or run holds the Game, and renders Stop only while an Autoplay run holds the Game. Send and Autoplay belong to the command form and the Autoplay form through their `form` attribute, and the forms stay in the shell. Autoplay takes clicks while the state is Checking or Preview, and the cycle budget is enabled while a plain Job runs. No script writes `disabled`: the server renders every control state. A request disables only the shell field of its form, the Action input or the cycle budget, through `hx-disabled-elt`. While the request of its form runs, CSS shows Send or Autoplay as busy and takes no click. While an Autoplay run holds the Game, the action area is in the Autoplay state: it shows a Stop control in place of the Action input and the Autoplay control, and the status display shows the cycle of the run. The Game is view-only: the controls that change the story and the Game's config are disabled, and the Games list marks the Game. Reading, scrolling and the Game switch stay open.

## The Screen

The screen is the header, the failure banner and the parts of the visible tab. Each part is an element with `data-part`. The server holds one renderer for each part name (`SCREEN_PARTS`), and the browser names the parts it wants: `GET /screen?parts=<names>` answers one `<template data-part>` for each name, with the headers `X-Revision` and `X-Game`.

| Section | Parts |
|---|---|
| none | `header`, `failure-banner` |
| Game | `story-log`, `visual-sidebar`, `options-dock`, `send-control`, `autoplay-controls`, `status-display` |
| Settings | `settings` |
| Prompt Presets | `prompt-presets` |
| Worlds | `worlds` |
| Games | `games` |
| LLM Messages | `llm-messages` |

The page asks `GET /changes` every `data-poll-ms` (1 s) while the browser shows the page, and asks at once when the page becomes visible and when a request ends. When the stamp of the answer differs from `<html data-revision>`, one load fetches the screen of the visible tab and a direct morph writes each part, followed by `htmx.process`. One load is in flight at a time. An answer for a tab that is no longer visible is dropped, and the loop loads the visible tab. The end of every request that is not a read forgets the stamp and asks, so a control that a load skipped shows the server value in the next load.

A tab's parts exist only while the tab is open. A tab change empties each part of the tab that closes, so the story-log edit and its lock are gone, and loads the tab that opens. The loop fires `screen:beforemorph`, `screen:loaded` and `screen:cleared`; the scripts of the action area, the story log, the failure banner and the Settings sub-tab listen to them, because a direct morph fires no htmx swap event.

The saved Games have pages of 50, newest change first. The active Game shows above the pages and is not in them. A Previous or Next control carries the page that it goes to; a click sets `<html data-games-page>`, and each load of the Games part asks for that page. The server clamps a page past the end to the last page, and renders a control at an end of the list with `aria-disabled="true"` and not `disabled`, so that the control keeps keyboard focus through the load. The click handler ignores such a control. A list of one page has no pager. Leaving the Games tab deletes the attribute, so the tab opens on page 1.

No part holds a draft. A form that waits for a Save is in the edit dialog, outside every part. A load skips the two kinds of element that hold unsaved input: the story-log entry in edit, and a control whose request is in flight. A save that fails keeps the form open in the dialog. A control that saves on change goes back to its rendered value, and a report says that the value was not saved: in the notice, or in the Unreachable banner when no answer came.

An entry in edit carries `data-editing`, and the root of the story log carries `data-editing-entry` while any entry is in edit. A load morphs only the children of the root, so the attribute survives every load. The attribute locks every Edit control and every swipe control of the story log. After a saved edit, the lock ends when the screen shows the saved entry. A load that skipped the entry counts as current while the edit is open. When the edit ends, the page forgets its stamp and loads again, so the entry shows the engine's render. Each entry renders its own Save and Cancel, which show only while the entry is in edit. The Save, Retry, Retrigger, swipe and Delete controls are htmx controls like the controls of every other part, so their answers reach the one reader. A refused or invalid Save keeps the draft and the lock, and any other failed Save reverts the entry. The Save control of the edit looks disabled while the status display shows a running Job or Autoplay run; a save by click or by keyboard in that state gets the busy refusal. The locked controls are inert for pointer and keyboard, and they keep their button role for assistive technology, because a script cannot keep an `aria-disabled` attribute through a load.

The edit dialog is one modal dialog of the shell (`#edit-dialog`), outside every part, so no load meets it. The forms that wait for a Save open there: the connection forms (Add and Edit), the Test result of a saved connection, the Game Rename and New Game forms, the preset Add, Edit and View forms, the World Create and Edit forms with their posture selects, and the record of an LLM message. A swap into `#edit-dialog-body` opens the dialog, and the root of the fragment carries `data-dialog-form` and a heading with the id `edit-dialog-title`. Cancel, Escape and a save that the screen shows close it. The dialog says "Saving…" between the answer and the screen, takes no second submit of its form in that time, and names a failed screen load in the same line. A click on the backdrop does not close it. An Escape that closes an open error detail closes only that detail. Closing the dialog discards the form and returns focus to the control that opened it. The server renders an `id` on each control that opens the dialog, and the dialog focuses the control with that `id`. A save that was sent before Escape still applies, because closing the dialog does not cancel the request. An answer that comes from a control outside the dialog never replaces a form that is open in it. A refused or failed save keeps the dialog open with its draft, and the notice reports it. The dialog repeats the summary of the notice and the Unreachable text in its status line, because the modal hides the page.

### Markers

| Attribute | Meaning |
|---|---|
| `data-part` | On a part: its name. Only the screen load writes it. |
| `data-revision` | On `<html>`: the stamp of the screen that the page shows. It is absent while the page shows nothing, and from the end of a request that is not a read until the next load. |
| `data-poll-ms` | On `<html>`: the interval of the ask. |
| `data-screen-busy` | On `<html>`: an ask or a load is in flight. |
| `data-keep` | On any element: the attributes that a load keeps. They hold view state, and the server never sets them. |
| `data-editing` | On the story-log entry in edit: a load skips it. |
| `data-editing-entry` | On the root of the story log while an entry is in edit: CSS and a click guard lock the controls of the edit. |
| `data-autosave` | On a form that saves on change: a failed save returns its fields to the rendered values. |
| `data-saved-flag` | On a saving control: the id of the element that shows "Saved" after a save that the screen shows. That element has the class `saved-flag`, which draws the text. |
| `data-saved` | On that element: the "Saved" state. A load keeps it. |
| `data-game` | On `<html>`: the id of the Game that the screen shows, empty with no active Game. Each load sets it from `X-Game`. |
| `data-games-page` | On `<html>`: the page of the saved Games that the Games tab asks for. It is absent until a click on Previous or Next, and a tab change deletes it. |
| `data-page` | On a Previous or Next control of the Games part: the page that the control goes to. |
| `data-dialog-form` | On the root of a fragment that opens in the edit dialog. |
| `data-shell-form` | On the root of a form that the shell renders outside every part, such as the text-check preview. |
| `data-saving` | On the edit dialog: a save was sent and the screen does not show it yet. |
| `data-close-dialog` | On a Cancel or Close control in the edit dialog: it closes the dialog with no request. |
| `data-error-slot` | On the slot that a form with typed input renders as a direct child, on the slot that the message edit renders in its Save and Cancel group, and on the report container of the notice place. No other element holds one. |
| `data-report` | On the report container of the notice place `#notice`: the element that holds the report. The page also writes `hidden` and `data-source` on `#notice`. |
| `data-source` | On `#notice`: `request` or `screen`, the kind of the report that it shows. A good screen load clears only a `screen` report. |
| `data-close-notice` | On the Close control of the notice place: it clears the notice and moves focus to `#main-content`. |
| `data-status` | On the status display answer: the machine state (`idle`, `thinking`, `autoplay`, `failed`). |
| `data-state` | On the action area: its state, which the page derives from the status display, the check in flight, the pending send and the open preview. The CSS reads it for the states that only the page knows. |
| `data-id` | On a story-log entry or a Games-list row: its record id. The focus rule takes it as a row id. |
| `data-raw-text` | On a story-log entry: its raw text, which the edit form opens with. |
| `data-tab` | On a tab button: the section it opens. |
| `data-cmd` | On a slash-palette item: its command. |

## Failures and Health

A failure shows where its scope is. An invalid answer to a form with typed input shows in the slot of that form, and the typed text stays. Every other report, a refusal, a failure and a failed screen load included, shows in the notice place `#notice` under the header, on every tab, with a Close control. A notice stays until the next request, until a failing screen load works again, or until Close, and a dialog opener counts as a request. The latest report wins: a failed screen load replaces the notice of a request that shows, and the next good load clears it. A request that gets no answer raises the failure banner, and no notice or slot holds a report of it. A generation failure shows in the status display as a short sentence that names only a remedy the dashboard offers. The status display draws that sentence in full, and the raw error text stays behind its Details control. A failure that affects the whole engine shows in the failure banner. A change ask or a screen load that the engine answers with a failure shows in the notice place, and every part keeps its content. The engine answered, so it is reachable: only a network error is an engine failure that raises the banner. The next ask loads the screen again.

Four health states describe the engine's roles and the engine itself:

- **Healthy** — the role's newest LLM call succeeded.
- **Degraded** — the role's newest LLM call failed.
- **No calls yet** — the role has no recorded LLM call.
- **Unreachable** — the client got no usable answer from the engine.

The server owns the three role states. It reads them from the LLM Messages record. Role health is engine-wide: the newest call of a role covers all Games. Degraded and Unreachable raise the failure banner. Role health shows in three places: the failure banner, the Settings role rows, and the Connections sub-tab marker. The banner is the part `failure-banner`, and the Settings panel renders its own rows and marker. The banner loads with every screen load. The Settings rows load while the Settings tab is open.

## Story Log and Swipes

Each story-log entry is one Message. Edit sits on every entry. The last entry alone carries Retrigger and Delete.

A swipe switch makes the target Swipe active, so every part shows the Swipe state of that Swipe. Only the last Message shows swipe controls. Retry adds a new swipe to it. Retrigger runs the stored event record's trigger narration again, as a new event Message.

## Management Panels

The management panels are Settings, Prompt Presets, Worlds, Games, and LLM Messages.

- **Settings** — the LLM connections and the role that each one serves (Narrator, Quantifier) on the Connections sub-tab, and the text-check settings on the Text Check sub-tab.
- **Prompt Presets** — the prompt presets of each type. A card shows a short preview, and the full preset opens in the edit dialog.
- **Worlds** — the Worlds with a shortened description. The full World opens in the edit dialog.
- **Games** — the active Game, the New Game control, and the saved Games of every World, in pages. A Game has a stable generated name and a renameable display name.
- **LLM Messages** — the record of each LLM call that a role makes, failed calls included. A row shows the header of the call, and the full record opens in the edit dialog.

## Supported Viewports

The dashboard is desktop-first. Desktop widths are fully supported, and tablet widths are best effort.

## Document References

- [`../../explanation/dashboard_design.md`](../../explanation/dashboard_design.md) — why the dashboard is built this way.
- [`../storage.md`](../storage.md) — the revision and the change counters that the screen reads.
