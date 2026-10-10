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

## Server-rendered fragments and polling

The server renders every region and panel, so domain content has one source. The HTMX runtime ships with the app's static assets, so a dashboard load reaches only the engine's own server.

Polling makes a pause cheap. Each request is independent, so a pause is "stop sending requests for a while". The dashboard pauses a poll in two places, where a refresh would race the player's work:

- **An open story-log edit.** The next refresh would overwrite the text the player is editing.
- **An expanded LLM Messages row.** The next refresh would collapse the raw request or response the player is reading.

A pause lives only in the browser. A WebSocket design would need pause and resume messages over the socket, and SSE would need a custom pause mechanism.

## Palette

The dashboard's palette is built for long prose. Quoted dialogue differs from narration by hue, and both stay at body weight. One accent hue marks every OK and active state, and a second accent hue marks every error state.

## Text-check preview emphasis

The text-check preview offers three controls at one size: Send with edits, Send Original, and Cancel. The corrections are suggestions that the player opts into, so Send Original keeps the primary colour and Send with edits takes the secondary one.

## Document References

- [`../reference/frontend/dashboard.md`](../reference/frontend/dashboard.md) — the dashboard's parts and what owns what.
