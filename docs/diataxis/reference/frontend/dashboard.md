---
diataxis: reference
title: Dashboard
---

## Overview

The dashboard is the player's single-page view of the engine, served at `/`. Play happens on the Game tab. The other tabs are management panels.

## Shell and Fragments

A static shell holds the page frame. The server owns all domain content: it renders each region and panel as an HTML fragment from the current game state, and the shell loads each fragment with HTMX.

Four statements hold for every region and every action:

1. **Data flows one way.** An action saves data and answers no region HTML. The data reaches the screen only through the change counters: a counter moves, and the regions that show that kind of data load. The page asks for the counters at once after any action, so the result shows without waiting for the next ask.
2. **An action changes only the element it acts on.** A save answers nothing on success. On failure, it answers its error into the inline slot of its own form. A view change, such as opening a form, changes only the edit dialog. No action targets a region element, and no answer writes out of band.
3. **A region load keeps what it does not change.** Every region loads its own fragment and morphs it into place. A kept node keeps its focus, selection, scroll position and view state. A load skips an element that holds unsaved input. A load that removes the focused node moves focus to the nearest row that survives, or else to the region.
4. **The browser owns view state, and the server owns data.**

The shell owns only client state:

- the active tab;
- the open state of an error popover or a Details disclosure and the active Settings sub-tab, which a load keeps (`data-keep`);
- the record of each live region: the counters of its kinds at its last good load, and its `data-live-state`;
- the action-area state, and the action area's own status element `#action-status`;
- the Unreachable health state;
- the slash palette.

## Regions

The page has a header, a failure banner, a tab bar, and the body of the active tab. The header shows the display name of the active Game.

The Game tab has four regions: the story log, the visual sidebar, the options dock, and the action area.

- **Story log** — the story window of the Game: its newest Messages, in order.
- **Visual sidebar** — the image of the current room and the portraits of the Characters in it.
- **Action area** — the Action input, the Autoplay control, the text-check preview, and the status display.

The action area is in one of five states: **Idle**, **Checking**, **Generating**, **Autoplay**, and **Preview**. When the text check is on, the text check sits between a typed send and the request to the engine. The status display shows the step of the running Job, or the failure line of the last Job. The action area reads the status display and writes nothing into it. The state of a send that no Job has shown yet (Generating before the status display moves) shows in `#action-status`.

The Autoplay control holds a cycle budget and a start button. While an Autoplay run holds the Game, the action area is in the Autoplay state: it shows a Stop control in place of the Action input and the Autoplay control, and the status display shows the cycle of the run. The Game is view-only: the controls that change the story and the Game's config are disabled, and the Games list marks the Game. Reading, scrolling and the Game switch stay open.

## Live Regions

Each live region names the kinds of data it shows (`data-live`), and loads when one of them changes. The kinds are the fields of the `GET /changes` answer. The server counts them (see the Storage reference).

| Region | Kinds |
|---|---|
| `#header` | games |
| `#failure-banner-degraded` | settings, LLM messages |
| `#story-log` | story, busy state, worlds |
| `.visual-sidebar` | story, worlds |
| `#options-dock` | Options, busy state |
| `#status-display` | busy state |
| `.games-panel` | games, worlds, prompt presets |
| `.llm-messages-panel` | LLM messages |
| `.settings-panel` | settings, LLM messages |
| `.prompt-presets-panel` | prompt presets, settings, games |
| `.worlds-panel` | worlds, games |

The page asks `GET /changes` every 1 s while the browser shows the page, and asks at once when the page becomes visible. Each region keeps a record: the counters of its kinds at its last good load. A region is due when one of its counters differs from its record. A new server id or a new active Game clears every record.

A region is in one of four states, shown in its `data-live-state`:

- **current** — the region shows the data of its record;
- **loading** — a load is in flight;
- **stale** — the region is due, and its section is hidden, so it loads when the player opens the section;
- **due** — the last load failed or sent no request, so the next ask tries again.

No region holds a draft. A form that waits for a Save is in the edit dialog, outside every region. A load skips the one element that holds unsaved input: the story-log entry in edit, and a control whose request is in flight. A save that fails keeps the form open in the dialog. A control that saves on change goes back to its rendered value, and its inline error says that the value was not saved.

An entry in edit carries `data-editing`, and the root of the story log carries `data-editing-entry` while any entry is in edit. A load morphs only the children of the root, so the attribute survives every load. The attribute locks every Edit control and every swipe control of the story log. After a saved edit, the lock ends with the load that shows the saved entry. A load that skipped the entry counts as current while the edit is open. When the edit ends, the region forgets its record and loads again, so the entry shows the engine's render. The Save control of the edit looks disabled while the status display shows a running Job or Autoplay run; a save by click or by keyboard in that state gets the busy refusal. The locked controls are inert for pointer and keyboard, and they keep their button role for assistive technology, because a script cannot keep an `aria-disabled` attribute through a load.

The edit dialog is one modal dialog of the shell (`#edit-dialog`), outside every region, so no load meets it. The forms that wait for a Save open there: the connection forms (Add and Edit), the Test result of a saved connection, the Game Rename and New Game forms, the preset Add, Edit and View forms, and the World Create and Edit forms with their posture selects. A swap into `#edit-dialog-body` opens the dialog, and the root of the fragment carries `data-dialog-form` and a heading with the id `edit-dialog-title`. Cancel, Escape and a save that the engine accepts close it. A click on the backdrop does not. An Escape that closes an open error detail closes only that detail. Closing the dialog discards the form and returns focus to the control that opened it. A save that was sent before Escape still applies, because closing the dialog does not cancel the request. An answer that comes from a control outside the dialog never replaces a form that is open in it. A refused save, with the busy refusal too, reports in the slot of its own form and keeps the dialog open. A load skips a control whose request is in flight, so the player's choice stays until the request ends. A load that skipped such a control does not count as current: its region stays due, and the page asks again when the request ends, so the control then gets the value that the engine renders.

### Markers

| Attribute | Meaning |
|---|---|
| `data-live` | On a region: the kinds of data it shows. |
| `data-live-state` | On a region: its state. The script sets it. |
| `data-keep` | On any element: the attributes that a load keeps. They hold view state, and the server never sets them. |
| `data-editing` | On the story-log entry in edit: a load skips it. |
| `data-editing-entry` | On the root of the story log while an entry is in edit: CSS and a click guard lock the controls of the edit. |
| `data-autosave` | On a form that saves on change: a failed save returns its fields to the rendered values. |
| `data-saved-flag` | On a saving control: the id of the element that shows "Saved" after a save that went through. That element has the class `saved-flag`, which draws the text. |
| `data-saved` | On that element: the "Saved" state. A load keeps it. |
| `data-refusal-what` | On a saving form: the words that finish "The Game is busy, so …" when the busy Game refuses the save. |
| `data-dialog-form` | On the root of a fragment that opens in the edit dialog. |
| `data-shell-form` | On the root of a form that the shell renders outside every region, such as the text-check preview. |
| `data-close-dialog` | On a Cancel or Close control in the edit dialog: it closes the dialog with no request. |
| `data-error-slot` | On an inline error slot: its name. A slot that shows an error keeps it through a load. |
| `data-unreachable` | On an inline error slot that shows the Unreachable text: it clears with the banner. |
| `data-status` | On the status display answer: the machine state (`idle`, `thinking`, `autoplay`, `failed`). |
| `data-state` | On the action area: its state, which the CSS reads. |
| `data-id` | On a story-log entry or a Games-list row: its record id. The focus rule takes it as a row id. |
| `data-raw-text` | On a story-log entry: its raw text, which the edit form opens with. |
| `data-tab` | On a tab button: the section it opens. |
| `data-cmd` | On a slash-palette item: its command. |

## Failures and Health

A failure shows where its scope is. A failed form or control shows the failure in its own inline slot. A generation failure shows in the status display as a short sentence that names only an action the dashboard offers. The status display draws that sentence in full, and the raw error text stays behind its Details control. A failure that affects the whole engine shows in the failure banner. A failed load of the Settings, Prompt Presets, Worlds or Games panel shows its error in the panel, and the panel keeps its content. A change ask, or a failed load of any other region, is an engine failure. In both cases the region stays due, and the next ask loads it again.

Four health states describe the engine's roles and the engine itself:

- **Healthy** — the role's newest LLM call succeeded.
- **Degraded** — the role's newest LLM call failed.
- **No calls yet** — the role has no recorded LLM call.
- **Unreachable** — the client got no usable answer from the engine.

The server owns the three role states. It reads them from the LLM Messages record. Role health is engine-wide: the newest call of a role covers all Games. Degraded and Unreachable raise the failure banner. Role health shows in three places: the failure banner, the Settings role rows, and the Connections sub-tab marker. The banner is the region `#failure-banner-degraded`, and the Settings panel renders its own rows and marker. Both load when the settings or the LLM messages change.

## Story Log and Swipes

Each story-log entry is one Message. Edit sits on every entry. The last entry alone carries Retrigger and Delete.

A swipe switch makes the target Swipe active, so every region shows the Swipe state of that Swipe. Only the last Message shows swipe controls. Retry adds a new swipe to it. Retrigger runs the stored event record's trigger narration again, as a new event Message.

## Management Panels

The management panels are Settings, Prompt Presets, Worlds, Games, and LLM Messages.

- **Settings** — the LLM connections and the role that each one serves (Narrator, Quantifier) on the Connections sub-tab, and the text-check settings on the Text Check sub-tab.
- **Games** — the active Game, the new-game form, and the saved Games of every World. A Game has a stable generated name and a renameable display name.
- **LLM Messages** — the record of each LLM call that a role makes, failed calls included.

## Supported Viewports

The dashboard is desktop-first. Desktop widths are fully supported, and tablet widths are best effort.

## Document References

- [`../../explanation/dashboard_design.md`](../../explanation/dashboard_design.md) — why the dashboard is built this way.
- [`../storage.md`](../storage.md) — the change counters that the live regions read.
