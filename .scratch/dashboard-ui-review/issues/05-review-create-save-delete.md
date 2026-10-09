# Review create, save and delete flows

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

What happens, on success and on failure, when the user creates, saves, or deletes a connection, a prompt preset, a world, a game, and a message? The main question: is every failure shown to the user?

## Context

- The first review did not exercise these flows. It found silent failures by reading code: `submitEdit`, `submitNewSwipe`, and `submitRetrigger` check neither `response.ok` nor `.catch` (finding 1.7).
- Provoke failures on purpose: invalid input, duplicate keys, a stopped engine mid-request.
- Use throwaway entities only. Do not change or delete the user's existing games, worlds, presets, or connections.

## Work

- Use `/chronicler-ui-investigator`. Use the same method and P1–P3 scale as the [first review](../review-2026-09-29.md).

## Done when

- The ticket answer has a table: flow × outcome (success / failure) → what the user sees.
- Each finding is a new ticket in this map or is added to an existing one. New tickets are also added to the final re-review's `Blocked by:`.

## Answer

Method: engine on port 3210, shared Chrome tab via `scripts/cdp.mjs`, throwaway entities only; one deliberate engine stop for the network-failure batch. Full report with all screenshots: `tmp/ui-review/t05-report.md` (screenshots 50–69; also reviewed the earlier partial run's 21–49). Tags: **[known]** = observed in this session (DOM and screenshot), **[inferred]** = from code, not exercised.

### Flow × outcome

| Flow | Success → what the user sees | Failure → what the user sees |
|---|---|---|
| Connection add | New card appears in the list; the Add form clears. [known] 57 | Unknown provider → **whole Settings panel replaced by a bare error** (all cards, Add form and Text Check gone); no toast; reload needed. [known] 54 |
| Connection edit | Card replaced in place with the new name/model; no toast. [known] 57 | Error replaces the **card** — the connection disappears from the list, everything else stays. [known] 56 |
| Connection delete | Confirm → card removed; panel intact. [known] DOM | Engine down → nothing: confirm closes, card stays, no toast. [known] 65 |
| Preset add | New card at the top; form clears. [known] 50 | Invalid type → **whole Prompt Presets panel replaced by the text "Invalid preset type"** (card count 0). [known] 51. Missing field → 422, raw serde text in the top toast, panel intact. [known] prior 33 |
| Preset edit (save) | Card replaced in place with the new name; no toast. [known] DOM | Not reachable from the UI; `Update failed` would replace the card with a bare error. [inferred] |
| Preset delete | Confirm → card removed; panel stays. [known] DOM | Active (mode-default) preset: refused, but the refusal **replaces the card** with bare error text; the card disappears. [known] 52. Engine down → silent. [inferred] |
| World create | Panel re-renders; new world in the list; form gone. [known] 53 | Invalid map/scenarios JSON → red toast with the parse message; every entered value is kept. [known] prior 39 + live |
| World update (save) | Card/list updates; posture auto-save shows green "Saved". [known] prior 41 | Invalid scenarios JSON → toast; form stays; the previous autosave's green "Saved" remains. [known] prior 42 |
| World delete | Confirm → item removed. [known] DOM | Engine down → nothing: confirm closes, item stays, no toast. [known] 66 |
| Game create | Page reloads, new game active, arrival narration appears. [known] 59 | World not found → red toast "Error: World not found"; the invalid selection stays. [known] 58 |
| Game delete | Confirm → item removed; the active game is unaffected. [known] 69 | Engine down → nothing: confirm closes, all games stay, no toast. [known] 67 |
| Message edit (save) | Text replaced in the log; polling continues. [known] 60 | Engine unreachable → **silent**; textarea stays; ✓/✗ dead; `#story-log` keeps `hx-trigger="none"` forever (reload to recover). [known] 63 |
| Message delete | Confirm → last entry removed. [known] 62 | Engine down → toast "Failed to delete message". [known] 68 |
| New swipe / Retry | Status advances, narration appears (ticket 06's area). | Engine down → stuck "Thinking...", disabled "■ Stop", no error. [known] 64 |

**Answer to the main question:** no — not every failure is shown. Three failure modes are silent or destructive: (a) network failures on htmx requests (`htmx:sendError` is unhandled) — connection/world/game deletes do nothing; (b) a failed message save shows nothing and permanently pauses polling; (c) server validation errors are shown *by replacing the panel or card they were aimed at*, which reads as data loss and needs a reload. Failures that do report (world create/update parse errors, game create, message delete) report raw technical text in most cases.

### Findings

1. **P1 — Error responses wipe whole panels** (connections add, presets add). 200-status `render_error` bodies are swapped into `.settings-panel` / `.prompt-presets-panel`; the panel is left holding only the error. 54, 51. → ticket 25.
2. **P1 — Failed edit / refused delete destroys the entity card** (connection edit, active-preset delete). 56, 52. → ticket 26.
3. **P1 — Failed message save is silent and permanently pauses the story-log poll**; `resumePolling()` lives only in `.then()`, so the log never refreshes again; ✓/✗ become dead. 63, 64. → ticket 27.
4. **P1 — Creating a world with an existing key silently overwrites the existing world** (`INSERT OR REPLACE`); reproduced and verified in the SQLite DB. 53; prior 40. → ticket 29.
5. **P2 — Duplicate connection and preset names are accepted silently.** 57, 50. → ticket 28.
6. **P2 — Network failures on htmx deletes are silent** (connection/world/game). 65, 66, 67; `deleteMessage()` is the contrast, 68. → context added to ticket 08.
7. **P3 — Raw server error strings reach the user** (`Failed to deserialize form body: ...`, `Unknown LLM backend ...`). Prior 33, 35; 54, 56. → context added to ticket 08.
8. **P3 — Stale green "Saved" autosave indicator after a failed world update.** Prior 42. → context added to ticket 16.

New tickets 25–29 are added to the final re-review's `Blocked by:`. Merges into existing tickets: 08 (findings 6, 7) and 16 (finding 8).

### What works

Add/edit/delete success paths are coherent; world create/update failures keep all form values and name the problem; delete success removes the item with no stray toast; `deleteMessage()` is the one mutation that handles both HTTP and network errors.

### Cleanup

All throwaways deleted through the UI (2 connections, 2 presets, 1 world + the one it overwrote, 2 Test Realm games from the earlier partial run and this session). Final DB state verified: 7 user connections, 4 visible presets, 2 worlds, 1 game (the user's `Redmist Estate_2026-09-29_1`, active); active preset registry back to defaults; narration/quantifier unchanged. The user's game was not touched.
