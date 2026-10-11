# Render the story-log and LLM Messages stub fixtures from the real templates

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

The stub browser server hand-copies the story-log and LLM Messages fragments into `tests/test_utils/stub_fixtures/`, so a change to the shipped Askama template cannot change what a stub test reads. Can those two fixtures be rendered through the engine's own templates instead, the way the options dock and the text-check preview already are?

## Context

- The stub server (`tests/test_utils/stub_server.rs`) serves most fragments with `include_str!` (lines 30–38), but renders the options dock through `OptionsDockTemplate` (line 53) and the text-check preview through the engine template (line 72), "so the canned preview cannot drift from the shipped".
- This session added `id="edit-entry-{{ entry.id }}"` (`src/adapters/driving/http/templates.rs:25`) and `id="llm-msg-header-{{ msg.id }}"` (`:174`) for the focus work in [Restore keyboard focus after in-place htmx swaps](45-restore-focus-after-inline-swaps.md) and [Make LLM Messages rows keyboard-operable](44-keyboard-operable-llm-messages.md). Both fixtures had to be hand-edited to match.
- The drift is silent. [Pin the message-edit template hooks the edit JavaScript reads](../../_closed/ui-verification-redesign/issues/14-pin-message-edit-template-hooks.md) proved it: renaming `data-raw-text` in the template left the full gate green while Edit opened an empty textarea. A template change that drops a hook the tests rely on stays green today, because the fixture is a separate file and nothing compares them.
- The interim fix is to pin each new hook with a unit test on the real template, per ticket 14. That is one assert per hook and it never ends. This ticket removes the drift instead.

Changes:

- Render the story-log fragment from `NarrativeLogTemplate` and the LLM Messages fragment from the LLM template, building the view models in the stub server.
- Keep the canned **data** in the stub (the narrative text, the single LLM attempt); only the markup moves to the template.
- Delete `tests/test_utils/stub_fixtures/story_log.html` and `tests/test_utils/stub_fixtures/llm_messages.html`, or reduce them to data if the stub keeps a file per fixture.
- Scope: the story log and LLM Messages only — the two the accessibility tickets churned and the two with the most hooks the client JS addresses. Decide in this ticket whether `header`, `settings`, `prompt_presets`, `worlds`, `games` and `action_area` follow in a later ticket; do not convert them here.
- The engine templates are already `pub` in `adapters::driving::http::templates`, so the seam the options dock uses is available.

Tests:

- Every existing stub test must pass unchanged; they read the served fragment, not the file.
- Prove the drift is gone: change a hook in `NarrativeLogTemplate` (for example the edit button's `id`) and show the served stub fragment changes with no fixture edit. This is the mutation ticket 14 could only catch with a hand-written assert.

No new spec scenario: this changes where the stub gets its markup, not what the dashboard does.

## Done when

- The story-log and LLM Messages stub fragments are rendered through the real templates.
- Every existing stub test passes unchanged.
- A template hook change reaches the served stub fragment with no fixture edit, proven by a temporary mutation.
- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.

## Answer

**Both fixtures convert.** `/fragment/story-log` serves `NarrativeLogTemplate::new(&entries, true).render()` and `/fragment/llm-messages` serves `LlmMessagesTemplate::new(&messages).render()`. The canned narrative text and the single LLM attempt are now Rust data (`MessageEntry` / `LlmMessage`); the markup comes from the templates. `tests/test_utils/stub_fixtures/story_log.html` and `llm_messages.html` are deleted.

**Decision on the other fixtures: they follow in a later ticket.** `header`, `settings`, `prompt_presets`, `worlds`, `games` and `action_area` stay hand-copied for now — this ticket's scope is the two the accessibility tickets churned.

**The drift is gone, proven by mutation.** A temporary test asserted `id="edit-entry-MUTANT-1"` in the served fragment: it failed before any mutation, passed while `NarrativeLogTemplate`'s edit-button id was temporarily changed, and `templates.rs` was then reverted byte-identical to HEAD. The temporary test was deleted; no mutant string remains.

**Deviation the ticket did not allow for.** `stub/invariants.rs::check_forced_colors_focus_ring`'s Tab loop widened from `0..20` to `0..30`. Truthful rendering gives the last entry its delete control — the hand-copied fixture had drifted by omitting it — so `#command-input` moved from tab stop 19 to 20, exactly at the old bound. The bound is a search limit, not an assertion, and the focus-ring assertion is unchanged.

**Merge note.** The two scripted story-log fixtures added by [Reset the generation status and the options dock when a swipe switch restores a snapshot](43-reset-status-and-dock-on-swipe-switch.md) were dropped here for the same reason: the scripted two-Swipe and restored shapes now render through `NarrativeLogTemplate` from scripted `MessageEntry` data, so no hand-written log markup remains.

No new spec scenario: this changes where the stub gets its markup, not what the dashboard does. Every stub test passes. `python build.py` is green.
