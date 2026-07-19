---
diataxis: explanation
title: Dashboard Design
---

> **Diátaxis mode:** Explanation. The reader problem solved here is *understanding*: the design behind the player-facing dashboard — the chat-app aesthetic inherited from SillyTavern, the polling cadences, the polling-pause pattern, the snapshot-restoration cascade, the empty-input continuation flow, and the server-rendered fragments. Companion to `../reference/dashboard.md`, which describes the dashboard as it is.

## The SillyTavern lineage

The Chronicler Engine's dashboard inherits its interaction model from SillyTavern, the chat-front-end for LLM roleplay that defined the UX vocabulary this dashboard speaks. The borrowed pieces are visible in the static shell and the per-message template:

- **Story log + visual sidebar + action area layout.** The three-region Game tab mirrors SillyTavern's center column for chat history, side column for character portraits and scene imagery, and bottom strip for input. The Chronicler Engine's specific layout (story log 80% / visual sidebar 20% / action area 64px) is the SillyTavern chat layout with the action area pinned to a fixed-height bottom strip rather than floating.
- **A/B swipes on the last message.** SillyTavern's swipe model — multiple alternative generations on the last message, navigation by arrow buttons — maps directly to the `Swipe` aggregate and the swipe controls. The Chronicler Engine extends the model: each swipe carries a `snapshot_id` (see "Snapshot restoration cascade" below) so switching swipes rewinds world state alongside the text.
- **Inline edit on any message.** SillyTavern's inline-edit pattern — click an entry's edit button, modify the text, save — maps to the edit flow: textarea replaces the text span, story-log polling pauses for the duration of the edit, server commits the new raw text on save. The Chronicler Engine applies the same pattern to player inputs (re-edit what you said) and AI responses (replace the LLM's generation with a hand-edit).
- **Continue button on empty input.** SillyTavern's "Continue" — pressing send with no text extends the last narration — maps to `continue_narration`: the empty command routes through `process_action(String::new())`, the same pipeline a typed action uses, with no player input.
- **Confirm dialogs for destructive actions.** Delete-last-message, reset-game, and delete-game each carry a JavaScript `confirm()` dialog before the action proceeds. The dialog text is per-template (the engine doesn't enforce it).

The data-model half of the borrowed pieces lives in the companion `./reference/message_model.md`.

## Polling cadences

Five endpoints carry their own cadence: four poll the server and one fetches once on load. The cadence values are stated in `./reference/dashboard.md`'s Polling Cadences. The shape of the values reflects what each endpoint carries:

- **Story log at 2s.** The story log is the primary feedback channel — narrative text is what the player watches, and new generations should appear promptly.
- **Status display at 5s.** The status pill ("Ready" / "Thinking..." / phase name) changes only on phase transitions, which are themselves paced by LLM round-trip time. A 5s cadence lands once per status transition.
- **Visual sidebar at 5s.** The sidebar carries the location image and NPC portraits — image data is large compared to text. A 5s cadence matches the rate at which sidebar content can actually change within a turn; a tighter cadence would re-fetch unchanged imagery.
- **LLM messages at 4s.** The LLM Messages tab is a forensics surface for inspecting prompts and responses. A 4s cadence sits between the story log and the sidebar; the player inspects messages deliberately rather than watching them arrive.
- **Header fetches once on load.** Game title, current game name, and connection status are stable for the duration of a session; the header fetches once and refreshes when the JS in `assets/index.html` triggers a refresh (e.g., after a swipe switch).

Per-tab management panels (Settings / Prompt Presets / Worlds / Games) fetch on tab activation and stay still while inactive. They're management surfaces; once a player has loaded the list they care about, the list is stable until the player triggers a reload.

The polling mechanism described above is what makes the polling-pause pattern (next section) possible: each request is independent, so pausing is "stop sending requests for a while".

## Polling-pause pattern

Polling a DOM region with HTMX is a one-line `hx-trigger` attribute. Pausing the poll is the same attribute set to `none`. The dashboard uses this for two flows where polling would race the user's in-progress interaction:

- **Edit mode pauses the story log.** While the user has a textarea open in place of an entry's text, the server's view of that entry's `text` (rendered into the polling fragment) could overwrite the user's edit. The static shell writes `hx-trigger="none"` on `#story-log` when edit mode starts, calls `htmx.process()` to re-read the attribute, and restores the original `hx-trigger` on save or cancel.
- **Expand mode pauses the LLM messages list.** When the user expands a `<details>` block inside an LLM message card to inspect a raw request/response JSON, the next polling refresh would re-render the card with `<details>` collapsed. The static shell tracks which cards are expanded, pauses the polling on expansion, and restores the expanded state after the next refresh when the user collapses.

Both pauses are local to the dashboard; the server side has no concept of a paused poll. The simplicity is a consequence of polling-as-fetch: each request is independent, so pausing is just "stop sending requests for a while". A WebSocket-based design would need to send pause/resume messages over the socket and have the server stop pushing; SSE would need a custom pause mechanism. Polling makes the pause a client-side attribute change.

## Snapshot restoration cascade

Switching swipes is a `snapshot_id` change: the target swipe was generated from a specific `GameStateSnapshot`, and switching to it rewinds the world to that snapshot's state. Three dashboard regions depend on world state and must re-render in step:

- **Story log.** The message history is part of the snapshot. A swipe switch that rewinds to a previous snapshot rewinds the message history too — earlier messages may differ (the snapshot was taken before they were generated).
- **Visual sidebar.** The location and NPC portraits depend on the player's current room and the NPCs present, both part of game state. A swipe switch that changes location or NPC presence must refresh the sidebar.
- **Header.** The header carries the current game name and connection status. The game name is stable across swipes within a single game, but the header refresh is part of the cascade because it costs nothing and the static shell wires it explicitly.

The cascade is implemented in the swipe-switch JavaScript: replace `#story-log` innerHTML with the response, then `htmx.trigger` on `.visual-sidebar` and `#header`. The Rust side serves the story-log fragment from the new snapshot's `MessageHistory`; the sidebar fragment from the new snapshot's location + NPC data; the header from the unchanged game metadata.

The split between "what changes per swipe" (story log + sidebar) and "what doesn't" (header game name) is incidental — the cascade hits three regions because three regions are state-dependent, not because three regions must always update together.

## Empty-input continuation

Submitting with an empty input routes to `continue_narration`, which calls `process_action(String::new())`. The empty string passes through the same action pipeline a typed command uses; the pipeline distinguishes "no player input" from "typed input" and produces a continuation message instead of a fresh narration turn.

The flow is named after SillyTavern's "Continue" button. SillyTavern exposes Continue as a separate UI affordance (a button next to Send); the Chronicler Engine exposes it through the existing Send button's empty-input path. The UI is identical to a typed send — the player types nothing and presses Send, and the engine extends the last narration. The button-state transition (Ready → Stop → Ready) is the same as a typed action.

The continuation produces a new swipe on the last message. The player can then navigate the new swipe with the existing swipe controls; comparing the new continuation to the prior one is the same comparison as comparing two swipes from a typed action's retry.

## Server-rendered fragments

The dashboard is a single `index.html` shell plus a fixed set of fragment endpoints the static shell fetches and polls. Every per-message update, every status poll, every panel content is a server-rendered HTML fragment that HTMX swaps into a target element. The client-side JavaScript is limited to:

- Tab switching (toggle `.active` class)
- Button-state transitions (Ready ↔ Stop ↔ Error)
- Polling-pause for edit / expand modes
- Swipe, retrigger, edit, delete submissions
- Text-check preflight orchestration

The inline JavaScript block in `assets/index.html` is several hundred lines of imperative DOM manipulation, scoped to button state and polling pause. The HTMX runtime is loaded from a CDN; everything else is plain ES.

The rendering pipeline is Askama with compile-time template validation; templates are checked at build time against their context structs. The companion `./reference/ui_design.md` carries the visual specs that this rendering path produces.

## Document References

- [`../reference/frontend/dashboard.md`](../reference/frontend/dashboard.md) — dashboard as it is: layout, tabs, polling cadences, flows, game management.
- [`../reference/storage.md#messages`](../reference/storage.md#messages) — the `Message` / `Swipe` aggregate that the swipe and snapshot-restoration flows operate on.
- [`../reference/frontend/ui_design.md`](../reference/frontend/ui_design.md) — design tokens and component specs that the server-rendered fragments consume.
- [`../reference/frontend/http_routes.md`](../reference/frontend/http_routes.md) — full HTTP route topology (52 routes, machine-generated).
- [ADR-001: HTMX Web Dashboard Architecture](../../docs/adr/adr-001-htmx-web-dashboard.md) — historical decision record for the HTMX + server-rendered HTML architecture.
- [ADR-002: HTTP Polling for Real-Time Updates](../../docs/adr/adr-002-http-polling.md) — historical decision record for polling over WebSocket/SSE.
- [ADR-003: Askama Template Engine](../../docs/adr/adr-003-askama-templates.md) — historical decision record for the Askama template engine.
