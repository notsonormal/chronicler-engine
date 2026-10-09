# Keep the story log on the newest entry

Type: task (AFK)
Status: open
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
