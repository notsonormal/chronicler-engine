---
diataxis: reference
title: Dashboard
---

## Overview

The dashboard is the player's single-page view of the engine, served at `/`. Play happens on the Game tab. The other tabs are management panels.

## Shell and Fragments

A static shell holds the page frame. The server owns all domain content: it renders each part of the screen as an HTML fragment from the current game state, and the shell loads the parts with one request.

Four statements hold for every part and every action:

1. **Data flows one way.** An action saves data and answers no screen content. The data reaches the screen only through a screen load: the revision moves, and the visible screen loads. The page asks for the revision at once after any action, so the result shows without waiting for the next ask.
2. **An action changes only the element it acts on.** A save answers nothing on success. On failure, it answers its error into the inline slot of its own form. A view change, such as opening a form, changes only the edit dialog. No action targets a part element, and no answer writes out of band.
3. **A screen load keeps what it does not change.** The page morphs each part into place. A kept node keeps its focus, selection, scroll position and view state. A load skips an element that holds unsaved input. A load that removes the focused node moves focus to the nearest row that survives, or else to the part.
4. **The browser owns view state, and the server owns data.**

Every action answer carries `X-Revision`, the stamp `<server>:<revision>` of the engine after the action. A dialog closes, a "Saved" line shows and the Action input clears only when the screen shows that stamp. A refusal answers `409`: the engine took nothing and wrote nothing. An invalid value answers `400`. `X-Refusal` names the kind of a `409`, a busy Game or a page that shows a Game that is no longer active. The page sends `X-Chronicler-Game` with each action, taken from the `X-Game` of its last screen load, and a route layer refuses a Game-bound action that names another Game than the active Game. The page drops the Game after a create, a switch or a Reset that it sent itself, until the next load.

The shell owns only client state:

- the active tab and the active Settings sub-tab, which `sessionStorage` keeps for a reload;
- the open state of an error popover or a Details disclosure, which a load keeps (`data-keep`);
- the stamp of the screen that the page shows (`data-revision` and `data-game` on `<html>`);
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

The action controls are two parts: `send-control` holds Send, and `autoplay-controls` holds Autoplay and Stop. The DOM order is the visual order, so Tab goes from the Action input to Send, the cycle budget, and Autoplay. The server renders Send and Autoplay as disabled while any Job or run holds the Game, and renders Stop only while an Autoplay run holds the Game. Send and Autoplay belong to the command form and the Autoplay form through their `form` attribute, and the forms stay in the shell. Autoplay takes clicks while the state is Checking or Preview, and the cycle budget is enabled while a plain Job runs. No script writes `disabled`: the server renders every control state, and a request disables its own controls through `hx-disabled-elt`. The Autoplay control holds a cycle budget and a start button. While an Autoplay run holds the Game, the action area is in the Autoplay state: it shows a Stop control in place of the Action input and the Autoplay control, and the status display shows the cycle of the run. The Game is view-only: the controls that change the story and the Game's config are disabled, and the Games list marks the Game. Reading, scrolling and the Game switch stay open.

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

The page asks `GET /changes` every `data-poll-ms` (1 s) while the browser shows the page, and asks at once when the page becomes visible and when an action ends. When the stamp of the answer differs from `<html data-revision>`, one load fetches the screen of the visible tab and a direct morph writes each part, followed by `htmx.process`. One load is in flight at a time. An answer for a tab that is no longer visible is dropped, and the loop loads the visible tab. A load that skipped a control in flight does not move the stamp, so the page asks again when the request ends.

A tab's parts exist only while the tab is open. A tab change empties each part of the tab that closes, so the story-log edit and its lock are gone, and loads the tab that opens. The loop fires `screen:beforeload`, `screen:beforemorph`, `screen:loaded` and `screen:cleared`; the scripts of the action area, the story log, the failure banner and the Settings sub-tab listen to them, because a direct morph fires no htmx swap event.

No part holds a draft. A form that waits for a Save is in the edit dialog, outside every part. A load skips the one element that holds unsaved input: the story-log entry in edit, and a control whose request is in flight. A save that fails keeps the form open in the dialog. A control that saves on change goes back to its rendered value, and its inline error says that the value was not saved.

An entry in edit carries `data-editing`, and the root of the story log carries `data-editing-entry` while any entry is in edit. A load morphs only the children of the root, so the attribute survives every load. The attribute locks every Edit control and every swipe control of the story log. After a saved edit, the lock ends when the screen shows the saved entry. A load that skipped the entry counts as current while the edit is open. When the edit ends, the page forgets its stamp and loads again, so the entry shows the engine's render. The Save control of the edit looks disabled while the status display shows a running Job or Autoplay run; a save by click or by keyboard in that state gets the busy refusal. The locked controls are inert for pointer and keyboard, and they keep their button role for assistive technology, because a script cannot keep an `aria-disabled` attribute through a load.

The edit dialog is one modal dialog of the shell (`#edit-dialog`), outside every part, so no load meets it. The forms that wait for a Save open there: the connection forms (Add and Edit), the Test result of a saved connection, the Game Rename and New Game forms, the preset Add, Edit and View forms, the World Create and Edit forms with their posture selects, and the record of an LLM message. A swap into `#edit-dialog-body` opens the dialog, and the root of the fragment carries `data-dialog-form` and a heading with the id `edit-dialog-title`. Cancel, Escape and a save that the screen shows close it. The dialog says "Saving…" between the answer and the screen, takes no second submit of its form in that time, and names a failed screen load in the same line. A click on the backdrop does not close it. An Escape that closes an open error detail closes only that detail. Closing the dialog discards the form and returns focus to the control that opened it. A save that was sent before Escape still applies, because closing the dialog does not cancel the request. An answer that comes from a control outside the dialog never replaces a form that is open in it. A refused save, with the busy refusal too, reports in the slot of its own form and keeps the dialog open.

### Markers

| Attribute | Meaning |
|---|---|
| `data-part` | On a part: its name. Only the screen load writes it. |
| `data-revision` | On `<html>`: the stamp of the screen that the page shows. It is absent while the page shows nothing. A load that skipped a control with a request in flight keeps the old stamp, so the next ask loads again. |
| `data-poll-ms` | On `<html>`: the interval of the ask. |
| `data-screen-busy` | On `<html>`: an ask or a load is in flight. |
| `data-keep` | On any element: the attributes that a load keeps. They hold view state, and the server never sets them. |
| `data-editing` | On the story-log entry in edit: a load skips it. |
| `data-editing-entry` | On the root of the story log while an entry is in edit: CSS and a click guard lock the controls of the edit. |
| `data-autosave` | On a form that saves on change: a failed save returns its fields to the rendered values. |
| `data-saved-flag` | On a saving control: the id of the element that shows "Saved" after a save that the screen shows. That element has the class `saved-flag`, which draws the text. |
| `data-saved` | On that element: the "Saved" state. A load keeps it. |
| `data-refusal-what` | On a saving form or a button that sends its own request: the words that finish the refusal line ("…, so …") when the engine refuses the request with `409`. |
| `data-game` | On `<html>`: the id of the Game that the screen shows, empty with no active Game. It is absent after a create, a switch or a Reset that this page sent, until the next load. |
| `data-dialog-form` | On the root of a fragment that opens in the edit dialog. |
| `data-shell-form` | On the root of a form that the shell renders outside every part, such as the text-check preview. |
| `data-close-dialog` | On a Cancel or Close control in the edit dialog: it closes the dialog with no request. |
| `data-error-slot` | On an inline error slot: its name. A slot that shows an error keeps it through a load. |
| `data-unreachable` | On an inline error slot that shows the Unreachable text: it clears with the banner. |
| `data-status` | On the status display answer: the machine state (`idle`, `thinking`, `autoplay`, `failed`). |
| `data-state` | On the action area: its state, which the page derives from the status display, the check in flight, the pending send and the open preview. The CSS reads it for the states that only the page knows. |
| `data-id` | On a story-log entry or a Games-list row: its record id. The focus rule takes it as a row id. |
| `data-raw-text` | On a story-log entry: its raw text, which the edit form opens with. |
| `data-tab` | On a tab button: the section it opens. |
| `data-cmd` | On a slash-palette item: its command. |

## Failures and Health

A failure shows where its scope is. A failed form or control shows the failure in its own inline slot. A generation failure shows in the status display as a short sentence that names only an action the dashboard offers. The status display draws that sentence in full, and the raw error text stays behind its Details control. A failure that affects the whole engine shows in the failure banner. A change ask or a screen load that the engine answers with a failure shows in `#screen-error`, and every part keeps its content. The engine answered, so it is reachable: only a network error is an engine failure that raises the banner. The next ask loads the screen again.

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
- **Games** — the active Game, the new-game form, and the saved Games of every World. A Game has a stable generated name and a renameable display name.
- **LLM Messages** — the record of each LLM call that a role makes, failed calls included.

## Supported Viewports

The dashboard is desktop-first. Desktop widths are fully supported, and tablet widths are best effort.

## Document References

- [`../../explanation/dashboard_design.md`](../../explanation/dashboard_design.md) — why the dashboard is built this way.
- [`../storage.md`](../storage.md) — the revision and the change counters that the screen reads.
