# Review the text-check-enabled flow

Type: task (AFK)
Status: resolved
Blocked by: 03

## Question

What UI findings does the flow have when text check is enabled? This covers the ✓ on the player input, the ✓ on log entries, "check before sending to LLM", how results display, and what happens when the check fails.P

## Context

- The first review ran with text check disabled.
- Blocked by 03 so the review runs on a working action area.
- Enable text check through Settings, then restore the previous setting before the session ends.

## Work

- Use `/chronicler-ui-investigator`. Use the same method and P1–P3 scale as the [first review](../review-2026-09-29.md).
- Save screenshots under `tmp/ui-review/`, continuing the numbering.

## Done when

- The findings are in the ticket answer.
- Each finding is a new ticket in this map or is added to an existing one. New tickets are also added to the final re-review's `Blocked by:`.

## Answer

### Method

Server `http://127.0.0.1:3311` on an isolated copy of the user's DB (`target/debug/chronicler_3311.db`), desktop viewport 1280x800, shared headless Chrome in a tab I created. Every shot is one I looked at. Shots `tmp/ui-review/70`–`79`; the flow was driven through the real DOM (`assets/index.html` handlers and the real `/action/check`, `/action/confirm`, `/check-text` endpoints). `[known]` = observed or measured; `[inferred]` = read from code, not exercised.

Settings: opened with **Mode `Disabled` + "Check before sending to LLM" ticked** (`71`), changed to **`Spell + Grammar`** through the Settings UI (`72`), and restored to `Disabled` + ticked at the end (verified from the raw `/fragment/settings` response, not the cached DOM). The `Disabled`+ticked combination was the pre-existing state and is left as found — finding 4.3 already owns it. Data touched: the copy DB gained four turns while driving the flow; nothing outside the copy.

### Findings

| # | P | Finding | Evidence |
|---|---|---|---|
| C1 | P1 | **A text-check confirm leaves the command input permanently disabled.** Sending a flagged command through the preview ("Send" or "Send Original") swaps `#action-area` with the `/action/confirm` response. That response renders the input `disabled` because generation has just started, and nothing re-renders the action area when generation ends, so the input stays disabled while the status reads "Ready". Typing does nothing; only a reload recovers. Reproduced from both confirm paths. Root cause: `action_confirm_handler` returns `render_action_area()` without the `HX-Retarget: #status-display` swap (`builders/headers.rs`, used only by `dispatch_with_status_headers`), and the sole `action-area-refresh` listener in `assets/index.html` has no dispatcher. | [known] `76-after-confirm-input-stuck-disabled.png` (greyed input, bright Send, "Ready"); DOM `inputDisabled: true, btnDisabled: false` at status Ready; real CDP typing produced an empty value; [inferred] mechanism from `action/handlers/actions.rs`, `builders/headers.rs`, `assets/index.html` |
| C2 | P2 | **The read-only ✓ on a log entry renders the submit preview.** When the checked text has issues, `/check-text` returns the whole "Did you mean?" template — Send / Send Original / Cancel — into `#text-check-result`, under the still-live command form. A check on a historical entry therefore offers to re-submit it: clicking "Send" started a generation and created a new input entry from the corrected text. There is no indication of which entry was checked. | [known] `79-log-check-with-issues-preview.png`; DOM result buttons `["Send","Send Original","Cancel"]`; clicking Send → status "Generating narration..." and new input entry "I relieve the pron from Carla." |
| C3 | P2 | **The send preview replaces the action area, taking the generation status and Stop with it.** Submitting a flagged command while a turn is generating (Enter in the input submits even though the Send button is disabled) opens the preview and removes `#status-display` and its control from the DOM. The in-flight turn then runs with no phase text and no Stop; the preview is the only thing on screen. | [known] `77-submit-during-generation-preview-hides-status.png`; DOM `statusEl: false` while `/status/generating` answered `thinking` |
| C4 | P2 | **Opening or dismissing the preview drops keyboard focus to `<body>`.** The preview replaces the element holding focus, so nothing is focused, the correction textarea is not focused, and Cancel restores the form without focusing the input. Ticket 07 deferred this path here. | [known] with the input focused: after the preview opened, `document.activeElement === document.body`; after Cancel, `document.body` again while the input value was restored |
| C5 | P2 | **A plain check result has no dismiss control and no owner.** A clean check renders as a bare green "No issues found" at the bottom-left, with no label tying it to the entry, no close button and no expiry; it survives every story-log poll. While text check is disabled the ✓ is still shown and dead-ends on "Text check is disabled". No ✓ exists on the command input (`checkCurrentInput()` has no caller — corroborates ticket 15). | [known] `74-log-entry-check-clean-result.png`; DOM with mode `disabled`: `#text-check-result` = "Text check is disabled", `.check-btn` still present; `assets/index.html` `checkCurrentInput` has no caller |
| C6 | P3 | **Cancel restores a stale check result.** `saveActionArea()` snapshots the entire action-area HTML, including whatever is in `#text-check-result`, and `restoreActionArea()` puts it back. Cancelling a send preview brought back an earlier "No issues found" that no longer relates to anything. | [known] DOM after Cancel: `#text-check-result` = "No issues found" with the misspelled command restored |
| C7 | P3 | **The preview's ✍ (U+270D) renders as an empty box** in this environment, before "Did you mean?". Same glyph class as finding 3.3. | [known] `75-check-before-send-preview.png`, `79-log-check-with-issues-preview.png` |
| C8 | P3 | **Every correction is applied at once and silently changes the player's words.** The preview labels the box "Corrected (edit if needed)" and makes Send the primary action; on this day Harper changed "recieve"→"relieve", "phon"→"pron", "arround"→"aground" (meaning-changing suggestions), with no per-issue accept/reject. The check service's output quality is not the UI's to fix, but the surface presents it as an authoritative "Did you mean?". | [known] `75`, `79` corrected-textarea values |

### What works (so the negative findings are scoped)

- With text check enabled, a ✓ result no longer replaces the command form — finding 2.1 stays fixed (`74`).
- The auto-check preview itself is a clear compare-and-confirm panel: Original struck through, editable Corrected box, issue tags, Cancel (`75`).
- A command with no issues skips the preview and starts the turn with the form intact (`77-clean-submit-generating.png`).
- The ✓ works while a turn is generating and does not disturb the action area (`78-check-during-generation.png`).
- With "Check before sending to LLM" unticked, no check runs and the command goes straight through (DOM: `hasPreview:false`, status "Thinking...").
- Settings save and restore both work, with a "Text check settings saved!" status (`72`).

### Proposed tickets

**Title:** Fix the command input left disabled after a text-check confirm
**Type:** task
**Blocked by:** —
**Question:** With text check enabled, sending a flagged command through the preview ("Send" or "Send Original") swaps `#action-area` with `/action/confirm`'s response, which renders `#command-form input[name=command]` as `disabled` because generation has just started. Nothing re-renders the action area when generation finishes — the only `action-area-refresh` listener in `assets/index.html` has no dispatcher, and the 5s status poll touches only the status span and the Send button — so the input stays disabled while the status reads "Ready" and only a reload recovers. Reproduce: enable Spell + Grammar with "Check before sending to LLM", submit a misspelled command, click Send, wait for Ready; `tmp/ui-review/76-after-confirm-input-stuck-disabled.png`. Make the confirm path reconcile the input (or refresh the action area when generation ends) so the form is usable again, and add a browser test for submit → preview → confirm → Ready → type.

**Title:** Rework the ✓ text-check result so a check cannot submit and can be dismissed
**Type:** task
**Blocked by:** —
**Question:** The ✓ on a player-input log entry is a read-only check, but when it finds issues `/check-text` returns the full send-preview template into `#text-check-result`, under the still-live command form (`tmp/ui-review/79-log-check-with-issues-preview.png`); clicking its "Send"/"Send Original" re-submits the historical entry text as a new command through `/action/confirm` (verified: a new turn was generated). A clean result renders as a bare green "No issues found" at the bottom-left with no reference to the entry it checked and no dismiss control (`74`), and cancelling a later send preview restores that stale result because `saveActionArea()` snapshots the whole action-area HTML including `#text-check-result`. Decide and implement the check result as its own read-only surface: identify the entry, offer a close control, never offer Send, clear it rather than restoring a stale copy on Cancel, and do not present an auto-apply of every Harper suggestion as an authoritative "Did you mean?" (it produced "relieve"/"pron"/"aground" for "recieve"/"phon"/"arround").

**Title:** Keep the generation status and Stop reachable while the send preview is open
**Type:** task
**Blocked by:** —
**Question:** The send preview replaces the whole `#action-area`, so while it is open `#status-display` and the Send/Stop control do not exist in the DOM. Submitting a flagged command while a turn is generating (Enter in the command input submits even though the button is disabled) hides the in-flight turn's status entirely: the engine keeps generating with no phase text and no Stop (`tmp/ui-review/77-submit-during-generation-preview-hides-status.png`, `statusEl:false` while `/status/generating` answered `thinking`). Decide where the preview renders relative to the status display — a sibling inside the action area, or its own element like `#text-check-result` — and keep the status poll alive while the preview is up. Finding 6.3 (the Stop button is disabled and has no handler) means the same as found, but do not claim a working Stop that does not exist. Related: ticket 08 owns where status and health belong.

**Title:** Restore focus when the text-check preview opens or is dismissed
**Type:** task
**Blocked by:** —
**Question:** With the command input focused and a flagged command submitted, the preview replaces the element holding focus, so `document.activeElement` becomes `<body>`, the correction textarea is not focused, and there is no focus management; Cancel likewise leaves focus on `<body>` and the restored input unfocused. Ticket 07 deferred this path here. Move focus to the corrected textarea when the preview opens and back to the command input on Cancel/confirm (or give the swapped controls stable ids so htmx's focus restore works), and cover the open → Cancel and open → Send paths in a browser test.

**Existing ticket: 15 — why:** the ✓ in the story log is offered while text check is disabled and dead-ends on "Text check is disabled"; the command input has no ✓ at all (`checkCurrentInput()` has no caller). 15 already asks whether the ✓ should show when disabled and whether the button should exist.

**Existing ticket: 13 — why:** the preview's "Did you mean?" ✍ (U+270D) renders as an empty box; 13 owns the glyph-icon decision.

### Not checked

- **A genuine text-check service failure.** Harper runs in-process and did not fail; I could not force `/check-text`'s 500 (`Check failed: …`) or its 400s (`Enter text to check`, `Settings unavailable`). [inferred] `checkText()` ignores `response.ok`, so an error body is injected into `#text-check-result` as-is (the 500 body is plain text, the 400s are HTML spans).
- **Grammar-only mode and `ignored_words`.** Exercised only `Spell + Grammar` with an empty ignore list.
- **The `/action/check` fail-open path.** [inferred] from code: if `check_player_input` errors, the handler logs and dispatches the action unchecked rather than showing the failure.
- **Slash commands with text check on.** Engine commands (`/options`, `/impersonate`, `/guide`) bypass the check by design; not driven.
- **Mobile 480px.** Desktop 1280x800 only, matching the first review's primary target.
- **Screen-reader output for the result or preview.** Ticket 07's remit; its K4 already notes text-check results have no live region.

