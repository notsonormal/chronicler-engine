# Review swipes, the options dock and the Thinking states

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

What UI findings do these three Game-tab states have?

- **Several swipes:** switching, new swipe, and whether the snapshot restore is visible.
- **Options dock with options present:** both after a turn with auto-generate on, and after `/options`.
- **Thinking / generating states** during a turn.

## Context

- The first review's turn finished too fast to capture the Thinking states. Slow the turn down, for example with a Mock connection, rather than changing the user's narrator connection.
- The options dock was empty in the first review because the world has "Auto-generate options after each turn" off. That is expected, not a finding.
- Use a throwaway game.

## Work

- Use `/chronicler-ui-investigator`. Use the same method and P1–P3 scale as the [first review](../review-2026-09-29.md).

## Done when

- The findings are in the ticket answer.
- Each finding is a new ticket in this map or is added to an existing one. New tickets are also added to the final re-review's `Blocked by:`.

## Answer

**Method.** Server `http://127.0.0.1:3322` (the isolated DB copy), headless Chrome, desktop viewport 1280x800. Throwaway world `t06-qa` (Redmist's map + scenarios copied, "Auto-generate options after each turn" on) and throwaway game `QA Throwaway_2026-10-03_1`. A local OpenAI-shaped stub (`tmp/ui-review/t06-llm-stub.py`) slept 6s per call and answered options prompts with `<suggestion>` tags, wired in as narrator through a temporary `ollama` connection so the Thinking states stayed on screen. The narrator was restored to `deepseek-v4-flash`, the stub connection deleted, and the throwaway game and world deleted; the active game was switched back to the user's `Redmist Estate_2026-09-29_1`, which I never drove (no turn, edit, or swipe on it). Shots are `tmp/ui-review/80..89-*.png`. Tags as in the first review.

### Findings

| # | P | Finding | Evidence |
|---|---|---|---|
| 6.1 | P1 | **Switching a swipe leaves the dashboard permanently stuck on "Generating narration...".** The snapshot a swipe restores is the one written during narration generation (`status = Generating`, `phase = Narrating`), so `/status/generating` keeps answering `narrating` after the switch; within 5s the status display flips to "Generating narration...", Send becomes a disabled "Stop", and a typed command cannot be sent — Enter is blocked by the disabled default button and a reload does not clear it. With the same switch emptying the dock (6.2) there is no in-page control left to start an action. | [known] `87-stuck-after-swipe-switch.png` (2/2 + "Generating narration..." + dimmed Stop), `86-swipe-switched-1of2.png`; `/status/generating` = `narrating` after the switch (was `idle` after a turn); `#submit-btn.disabled = true`; a real CDP Enter on the focused input produced no request; reload + 6s still stuck. Root cause: `MessageService::switch_swipe` re-saves the swipe's snapshot, which `save_message_and_snapshot` wrote while `input_buffer.status == Generating`. |
| 6.2 | P2 | **Switching a swipe silently empties the options dock.** The restored snapshot predates options generation, so `current_options` resets to `[]` and the dock (2s poll) goes blank. Same root cause as 6.1. | [known] dock `innerHTML` 1125 bytes → 0 after 2/2 → 1/2; `86-swipe-switched-1of2.png`. |
| 6.3 | P2 | **During generation the primary button says "Stop" but is disabled and has no handler.** `setButtonState(true)` relabels it and sets `disabled = true`; there is no cancel/abort route, so a generation cannot be stopped and the label is a false affordance. | [known] `81-turn-t1.png`, `83-turn-options.png`, `88b-options-thinking.png`; `assets/index.html:139-140`; no cancel route in `router.rs`; `#command-form button:disabled { opacity: .5 }`. |
| 6.4 | P3 | **The swipe row renders "1 / 1" for a single-swipe entry**, with a disabled ◀ and a ▶ titled "Retry"; the same ▶ glyph means "Next swipe" once a second swipe exists. (Overlaps finding 3.3.) | [known] `82-throwaway-arrival.png`, `84-options-dock-populated.png`; `src/adapters/driving/http/templates.rs:25`. |
| 6.5 | P3 | **The snapshot restore is not surfaced.** Switching a swipe gives no confirmation and no "restored swipe N" affordance; the only cues are the narration text and the counter, and the one state signal it does touch is the wrong one (6.1). | [known] `86-swipe-switched-1of2.png`; `switchSwipe()` replaces the log and refreshes the sidebar/header with no status or toast. |
| 6.6 | P3 | **The options dock keeps the previous turn's options live while the next turn generates** — they stay clickable and `useOption` submits, so a stale option can be picked mid-generation. May be intentional. | [known] `82-turn-thinking.png` (dock populated while "Quantifying scene..."); `assets/index.html` `useOption`. |

### Thinking / generating states

DOM sampled every 250ms through a stub-slowed turn: `Thinking...` (0.7–2.7s) → `Generating narration...` (2.7–8.7s) → `Quantifying scene...` (9.7–19.6s) → `Generating options...` → `Ready`. The labels match the pipeline order (narrate → quantify → options). No defect beyond 6.3; the phase label is refreshed by the 5s `/status/generating` poll, so it can lag the true phase, though the submit also swaps in a fresh status fragment. `81-turn-t1.png` and `83-turn-options.png` show the narration and quantify states, `88b-options-thinking.png` the options state. The options dock populates from the same pipeline (`84-options-dock-populated.png` after a turn with auto-generate on, `89-options-dock-after-slash.png` after `/options`); its `♻` and `✎` controls are the tofu icons of finding 3.3, and its label/option contrast passes AA (label `#888` on `#1a1a1a` ≈ 4.9:1).

### Proposed tickets

**Title:** Reset the generation status when a swipe switch restores a snapshot
**Type:** task
**Blocked by:** -
**Question:** Switching a swipe (`POST /message/:id/swipe/:index`) restores that swipe's snapshot, but the snapshot was written during narration generation and carries `input_buffer.status = Generating` / `phase = Narrating`. `/status/generating` then answers `narrating` indefinitely, the dashboard flips to "Generating narration..." with a disabled Stop, Enter cannot submit, and a reload does not clear it — the player is stuck until some other generation finalizes. Make a restored swipe leave the game Idle (for example persist the swipe's snapshot after the turn finalizes, or normalise the status on restore), and verify the dashboard returns to "Ready" with an enabled Send after a switch.

**Title:** Keep the options dock in step when switching swipes
**Type:** task
**Blocked by:** -
**Question:** Switching a swipe restores the pre-options snapshot, so the options dock silently goes blank even though the narration just shown had options. Decide whether a restored swipe should keep, drop, or regenerate its options and make the dock reflect that choice; if a turn's options should survive a swipe switch, the snapshot the switch restores must include them (shared root cause with the stuck-status ticket).

**Title:** Stop labelling the disabled Send button "Stop"
**Type:** task
**Blocked by:** -
**Question:** During a generation the primary button is relabelled "Stop" and set `disabled`, and there is no cancel route, so it looks like a control that can end the generation but cannot. Either wire it to a real cancellation path (a larger decision) or stop relabelling it and show a non-interactive "Generating..." indicator instead.

**Existing ticket:** 13 — the ▶ glyph meaning both "Next swipe" and "Retry", and icon-only buttons without labels, are the icon-buttons decision; finding 6.4 is the swipe-control instance of it.

Findings 6.5 and 6.6 fold into the two snapshot/dock tickets above rather than needing their own.

### Not checked

- A clean shot of the transient `Thinking...` label: the shot I aimed at it (`82-turn-thinking.png`) landed on `Quantifying scene...`; the label appears only in the DOM samples (it lasts under ~3s before the `/action/check` swap replaces the status fragment).
- `Generating event...` (the trigger state), and the retrigger button/path.
- Swipes that differ substantially in text: the stub returns near-identical narration, so switching was verified by the counter and the `[STUB-NARRATION-n]` tag, not by visibly different prose.
- A real narrator failure during a turn, and the engine-down path for a new swipe.
- 480px / narrow viewport for these states.
- The quantifier connection in this DB resolved to the working `deepseek-v4-flash`, not the down `ollama-gemma-4-e2b` the notes describe (engine log: `[Quantifier] Calling provider: OpenRouter deepseek/deepseek-v4-flash`), so I did not observe the quantifier fallback of finding 1.1.
