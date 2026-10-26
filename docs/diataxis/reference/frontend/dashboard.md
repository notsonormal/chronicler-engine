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
2. **An action changes only the element it acts on.** A save answers nothing on success. On failure, it answers its error into the inline slot of its own form. A view change, such as opening an editor, changes only its own row or the editor slot of its region. No action targets a region element, and no answer writes out of band.
3. **A region load keeps what it does not change.** Every region loads its own fragment and morphs it into place. A kept node keeps its focus, selection, scroll position and view state. A load waits while the player edits in the region. A load that removes the focused node moves focus to the nearest row that survives, or else to the region.
4. **The browser owns view state, and the server owns data.**

The shell owns only client state:

- the active tab;
- the open state of disclosures, the active Settings sub-tab and an open error popover, which a load keeps (`data-keep`);
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

A region is in one of five states, shown in its `data-live-state`:

- **current** — the region shows the data of its record;
- **loading** — a load is in flight;
- **held** — the player edits in the region, so the load waits;
- **stale** — the region is due, and its section is hidden, so it loads when the player opens the section;
- **due** — the last load failed or sent no request, so the next ask tries again.

A region is held while it holds an edit: an element with `data-work` (the root of an open editor), or a field whose value differs from the rendered value. A held region does not swap, also when a load was already in flight. The **end-of-edit rule** frees it. A save that goes through makes the current value of each of its fields the rendered value, removes `data-work` from the form's editor root, and closes a disclosure that marks itself `data-close-on-save`. The held region then loads once. A disclosure that holds a form is an edit too. If it closes without a save, its edit is discarded: each form in it goes back to its rendered values, and the page asks at once, so the held region loads what it waited for. A close after a save discards nothing, because the save made the saved values the rendered ones. The New Game form and the Add forms of the Prompt Presets panel follow this rule, as does the Rename form of a Game. An editor with `data-work` is not a disclosure: its Cancel discards its edit. A save that fails keeps the edit of a form. A control that saves on change goes back to its rendered value, and its inline error says that the value was not saved.

The Worlds editor and the connection editor open into an editor slot of their region (`#world-editor`, `#connection-editor`). While the slot holds an editor, the list of the region is hidden. Cancel empties the slot in the browser with no request.

### Markers

| Attribute | Meaning |
|---|---|
| `data-live` | On a region: the kinds of data it shows. |
| `data-live-state` | On a region: its state. The script sets it. |
| `data-keep` | On any element: the attributes that a load keeps. They hold view state, and the server never sets them. |
| `data-work` | On the root of an open editor: the region is held. |
| `data-close-on-save` | On a `<details>`: a save of its form closes it. A close without a save is a discard (see the hold rule above), with or without this marker. |
| `data-reset-on-save` | On a form that creates something: a save returns it to the empty rendered form. |
| `data-autosave` | On a form that saves on change: a failed save returns its fields to the rendered values. |
| `data-saved-flag` | On a saving control: the id of the element that shows "Saved" after a save that went through. That element has the class `saved-flag`, which draws the text. |
| `data-saved` | On that element: the "Saved" state. A load keeps it. |
| `data-refusal-what` | On a saving form: the words that finish "The Game is busy, so …" when the busy Game refuses the save. |
| `data-opens-editor` | On a control that opens an editor: Cancel returns focus to it. |
| `data-close-editor` | On a Cancel control in an editor slot: it empties the slot with no request. |
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
