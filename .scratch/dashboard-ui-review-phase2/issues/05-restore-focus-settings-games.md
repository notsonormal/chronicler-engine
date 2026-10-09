# Restore focus after Settings, Games and story-log delete swaps

Type: task (AFK)
Status: open
Blocked by: —

## Question

Keyboard focus falls to `<body>` after swaps that the focus restore from [Keep DOM state and focus through in-place swaps](../../dashboard-ui-review/issues/_resolved/65-keep-dom-state-through-swaps.md) does not cover. Extend the same mechanism to them.

## Items

From the [re-review](../../dashboard-ui-review/re-review-2026-10-09.md):

- **R10 (P2)** With real focus on the control, these leave focus on `<body>`: Settings connection Edit (→ form page), Cancel / back link (→ list), and Games saved-game Delete. `FOCUS_SWAP_TARGETS` in `assets/dashboard-panels.js` lists only `.worlds-panel, .prompt-presets-panel, .preset-card`. The Settings panel was rewritten by [Split Settings into Connections and Text Check sub-tabs](../../dashboard-ui-review/issues/68-split-settings-sub-tabs.md) after the restore shipped; the Games panel was never listed. Check every Settings and Games swap (role select, Test, Delete refusal, Add, rename, Switch).
- **R7 (P3)** Deleting the last story-log entry removes the focused Delete button and leaves focus on `<body>`. Move focus to a sensible target, such as the new last entry or the command input.

## Done when

- Each listed path keeps focus on a sensible control; stub-tier browser tests cover a Settings path and the Games delete.
- `python build.py` is green. Commit after user approval.
