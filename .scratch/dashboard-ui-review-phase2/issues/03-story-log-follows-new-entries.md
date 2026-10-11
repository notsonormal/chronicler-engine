# Keep the story log on the newest entry

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

After a turn, the story log keeps its scroll position, so the new narration lands below the fold on every turn (finding R3, P2, in the [re-review](../../dashboard-ui-review/re-review-2026-10-09.md); shot `tmp/t24/10-move-turn.png`). Make the log follow new entries without taking the scroll away from a player who is reading further up.

## Context

- No script scrolls `#story-log`. The poll morphs the log (`morph:innerHTML`, [Keep DOM state and focus through in-place swaps](../../dashboard-ui-review/issues/_resolved/65-keep-dom-state-through-swaps.md)), which keeps the scroll position by design, so a new entry never comes into view.
- Proposed rule: if the log was at (or within a few pixels of) the bottom before a swap that adds an entry, scroll to the bottom after it; otherwise leave the position alone. A player's own send could also force a scroll to the bottom.
- Keep that ticket's guarantees: an idle poll touches no nodes, and selection, focus and scroll survive it.
- Edit mode pauses the poll; the rule must not fight an open edit.
- Delete, Swipe switch and new Swipe replace the log by other paths; check each.

## Done when

- The rule is in `dashboard.md` and a stub-tier browser scenario pins both halves (follows at bottom, stays put when scrolled up).
- `python build.py` is green. Commit after user approval.

## Answer

Implemented. The log follows an appended entry, but only for a player who was already at the bottom.

**`assets/dashboard-story-log.js`** — the rule. `htmx:beforeSwap` on `document.body` records, for a swap whose target is `#story-log`, whether the log was at (or within 8px of) its bottom and how many `.log-entry` nodes it held. `htmx:afterSwap` re-reads the entry count and, when the swap added at least one entry and the log was at the bottom, sets `scrollTop = scrollHeight`; every other case returns without touching the position. The listener pair follows the convention already in `dashboard-action-area.js` (`evt.detail.target || evt.detail.elt`, target id `story-log`), and the rule reads geometry only — it writes no DOM.

Guarantees from [Keep DOM state and focus through in-place swaps](../../dashboard-ui-review/issues/65-keep-dom-state-through-swaps.md) hold: an idle poll adds no entry, so the handler never moves the log, and selection, focus and scroll survive as before. An open edit returns early: the poll is paused for it, and a swap that lands during one (a delete) does not move the view.

The other three paths, checked:

- **Delete** — `deleteMessage` goes through `htmx.ajax` into `#story-log`, so the rule sees the swap, but the entry count drops: no scroll, and the position is whatever the browser makes of an `innerHTML` replace, as before.
- **Swipe switch** — `switchSwipe` writes `innerHTML` directly (no htmx swap) and adds no entry, so the rule never runs for it.
- **New swipe** — `/swipe/new` returns through the poll, which morphs the *last entry's text* in place. No entry is added, so the rule does not fire. See follow-ups.

**`docs/diataxis/reference/frontend/dashboard.md`** — the rule in the Story Log section (when it fires, the 8px slack, the open-edit case, and that a swap adding no entry leaves the position alone), plus one sentence in the Polling Cadences morph paragraph.

**`docs/specs/browser_story_log.md`** — scenarios 30.20 (follows the bottom) and 30.21 (leaves a scrolled-up log alone).

**Tests — tier 2 (stub browser)**, `tests/browser/stub/story_log.rs`:

- `test_poll_that_appends_an_entry_follows_the_bottom` (30.20)
- `test_poll_that_appends_an_entry_leaves_a_scrolled_up_log_alone` (30.21)

Both drive the shipped shell against the stub server and add the entry through the existing `StubStoryLogHandle::append_narration`, so the poll, the morph and the fragment template are the real ones. Red check: with the `scrollTop` write commented out, 30.20 fails ("gap 81 px"); 30.21 passes either way, because it pins the guard half that the pre-fix client already satisfied.

`python build.py` is green on the full gate (browser tier included): `nextest: 71 passed, 0 failed (Running browser tests...)`, integration 1595 passed / 2 skipped, guardrails 162, architecture 1.

Follow-ups (non-blocking):

- A **new swipe** regenerates the last entry's text in place, so the rule leaves the position alone: a player at the bottom ends up mid-narration with the new tail below the fold. Widening the rule to "the log grew while at the bottom" would cover it, but that goes past this ticket's stated rule ("otherwise leave the position alone").
- **Another tab open.** An inactive tab is `display: none`, so the hidden log reports zero geometry and `atBottom` reads true; the handler then writes `scrollTop = 0`. Measured in the stub: the write does not clobber the retained offset (a player who was at the bottom while hidden returned to `scrollTop 30`), so no guard was added — but a player who leaves the tab mid-generation comes back with the appended entry's height (81px in the fixture) still below the fold.
- The ticket's optional "a player's own send could also force a scroll to the bottom" is **not** implemented. It needs a client signal for "this send was mine" (the send is a form post, not a swap) and would conflict with the "stays put while reading further up" half. Not in Done when.

Uncommitted in the `wf/p2-03` worktree, as the phase-2 brief requires; the lead integrates it.
