# Review keyboard use and screen-reader output

Type: task (AFK)
Status: open
Blocked by: —

## Question

Can every dashboard task be done with the keyboard alone? What does the accessibility tree expose for each tab?

## Context

- The first review found icon buttons with only `title` and no `aria-label` (finding 3.3), and New Game selects with no labels (finding 5.5). Do not re-report these. Look for what is still unknown: tab order, focus visibility, focus after swaps, and live regions for status changes.
- `chrome_devtools_*` has no accessibility snapshot. `node scripts/cdp.mjs snap <target>` gives the accessibility tree.

## Work

- Use `/chronicler-ui-investigator`. Use the same method and P1–P3 scale as the [first review](../review-2026-09-29.md).

## Done when

- The findings are in the ticket answer.
- Each finding is a new ticket in this map or is added to an existing one. New tickets are also added to the final re-review's `Blocked by:`.
