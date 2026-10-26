---
diataxis: explanation
title: Dashboard Design
---

## The SillyTavern lineage

The Chronicler Engine's dashboard inherits its interaction model from SillyTavern, the chat front end for LLM roleplay that set the conventions for this kind of interface. The borrowed pieces:

- **Story log, visual sidebar and action area.** The Game tab mirrors SillyTavern's chat layout: a centre column for the chat history, a side column for character portraits and scene imagery, and a bottom strip for input. The Chronicler Engine pins the action area to the bottom of the page rather than letting it float.
- **A/B swipes on the last message.** SillyTavern's swipe model, with several alternative generations on the last message and arrow buttons to move between them, maps directly to the `Swipe` aggregate.
- **Inline edit on any message.** The player opens an entry's edit control, changes the text, and saves. The Chronicler Engine applies the pattern to player inputs (re-edit what you said) and to AI responses (replace the LLM's generation with a hand edit).
- **Continue on an empty input.** SillyTavern's Continue extends the last narration from a separate button next to Send. The Chronicler Engine puts Continue on the Send button: the player sends an empty input.
- **Confirmation before destructive actions.** Deleting a message, resetting the game and deleting a game each ask the player to confirm first.

## How the dashboard shows server data

The server renders every region and panel, so domain content has one source. The HTMX runtime ships with the app's static assets, so a dashboard load reaches only the engine's own server.

### The problem of the first design

The first design let seven regions poll their own fragment on a timer of 2 to 5 seconds, and most swapped their whole content. Actions also wrote into regions by their own paths: whole-panel answers, outer swaps, out-of-band swaps, script writes and page reloads. In 26 days, 19 bugs came from this design. In 13, a swap destroyed client state: an open rename form, typed text, or focus. In 6, a region was stale or out of step with another region. The 19 bugs got 12 different fix mechanisms, each local to one region. The fixes also caused faults of their own. A poll pause that saved the wrong trigger to restore stopped a panel from polling again.

The present design replaces the local fixes with one rule for every region.

### One version poll

The page asks one small endpoint, `GET /changes`, once a second while the page is visible. The design chose polling over WebSocket and SSE. The first decision record on this choice (ADR-002, since removed) found the htmx WebSocket extension unreliable in production and polling simpler and more stable. The ask answers only numbers, so it is cheap. A push transport can carry the same numbers later.

### One number for each kind of data

The server keeps a number for each kind of data, not for each region. A region shows a fixed set of kinds, so one comparison tells the browser whether a region needs to load. The table from kind to regions sits in the page, next to each region, so one look at a region tells what loads it. A load is precise: a World edit does not load the LLM Messages panel, and a step change in a Job does not load the Games panel.

A number rises after its data is committed, and every save path raises its own kinds through the storage layer. A save path that names no kinds does not compile. So the dashboard needs no safety refresh that covers a forgotten path.

The numbers are counters, not dates. A counter cannot repeat within a millisecond, and it does not depend on the system clock.

### One path into a region

Data flows one way: an action saves data, and the regions that show the data load it. A region has one path in, its own load. So two answers cannot write the same region in different shapes, and a region cannot lag behind another region that an action wrote directly. An action answers nothing on success, and its error appears in its own form. This costs one extra round trip after each action: the ask that follows it.

### View state survives a load, and an edit holds its region

A change can arrive while the player works. A load must neither destroy what the player does nor wait forever.

- A load morphs the region, so a node that the new fragment keeps stays the same node, with its focus, selection and scroll position.
- View state, such as an open disclosure, an expanded LLM message or the active Settings sub-tab, lives in attributes that the server never sets. An element lists them in `data-keep`, and the load keeps them. The alternative was a region that waits while any disclosure is open. That region would show no new LLM message while a card is open, and an open error popover would hold the status display.
- An edit holds the region. A region with an open editor or a changed field waits, and loads once when the edit ends. A control that saves on change goes back to its rendered value when its save fails. If it kept the value, that value would be an edit, and one failed save would stop the whole panel from updating with nothing on the screen to say why. A disclosure that holds a form follows the same reasoning: a closed disclosure hides its fields, but the browser keeps their values, so a draft that the player closed would hold the panel with nothing on the screen to say so. Closing it discards the draft.

The tradeoff is that a held region shows older data while the edit lasts. The `data-live-state` of the region says so.

## Palette

The dashboard's palette is built for long prose. Quoted dialogue differs from narration by hue, and both stay at body weight.

## Document References

- [`../reference/frontend/dashboard.md`](../reference/frontend/dashboard.md) — the dashboard's parts and what owns what.
