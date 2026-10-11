---
diataxis: reference
title: Dashboard
---

## Overview

The dashboard is the player's single-page view of the engine, served at `/`. Play happens on the Game tab. The other tabs are management panels.

## Shell and Fragments

A static shell holds the page frame. The server owns all domain content: it renders each region and panel as an HTML fragment from the current game state, and the shell fetches each fragment with HTMX.

The shell owns only client state:

- the active tab and the active Settings sub-tab;
- the action-area state;
- a paused poll, while a refresh would overwrite input in progress (an open story-log edit, an expanded LLM Messages row);
- the Unreachable health state;
- the slash palette.

## Regions

The page has a header, a failure banner, a tab bar, and the body of the active tab. The header shows the display name of the active Game.

The Game tab has four regions: the story log, the visual sidebar, the options dock, and the action area.

- **Story log** — the Game's message history.
- **Visual sidebar** — the image of the current room and the portraits of the Characters in it.
- **Action area** — the command input, the text-check preview, and the status display.

Each live region polls its own fragment.

The action area is in one of four states: **Idle**, **Checking**, **Generating**, and **Preview**. When the text check is on, the text check sits between a typed send and the action pipeline. The status display shows the generation phase that the action pipeline reports.

## Failures and Health

A failure shows where its scope is. A failed form or control shows the failure in its own inline slot. A generation failure shows in the status display. A failure that affects the whole engine shows in the failure banner. A poll that gets no usable answer is an engine failure.

Four health states describe the engine's roles and the engine itself:

- **Healthy** — the role's newest LLM call succeeded.
- **Degraded** — the role's newest LLM call failed.
- **No calls yet** — the role has no recorded LLM call.
- **Unreachable** — the client got no usable answer from the engine.

The server owns the three role states. It reads them from the LLM Messages record. Role health is engine-wide: the newest call of a role covers all Games. Degraded and Unreachable raise the failure banner. The header refresh owns every role-health display: the banner, the Settings role rows, and the Connections sub-tab marker.

## Story Log and Swipes

Each story-log entry is one Message. An entry can carry the controls Edit, Delete, the swipe controls (previous, next, and Retry), and Retrigger.

A swipe switch restores the Snapshot of the target Swipe, so every region shows that state. Swipes exist only on the last Message. Retry adds a new swipe to it. Retrigger runs the previous turn's trigger narration again, as a new event Message.

## Management Panels

The management panels are Settings, Prompt Presets, Worlds, Games, and LLM Messages.

- **Settings** — the LLM connections and the role that each one serves (Narrator, Quantifier) on the Connections sub-tab, and the text-check settings on the Text Check sub-tab.
- **Games** — the active Game, the new-game form, and the saved Games of every World. A Game has a stable generated name and a renameable display name.
- **LLM Messages** — the record of each LLM call that a role makes, failed calls included.

## Supported Viewports

The dashboard is desktop-first. Desktop widths are fully supported, and tablet widths are best effort.

## Document References

- [`../../explanation/dashboard_design.md`](../../explanation/dashboard_design.md) — why the dashboard is built this way.
