# Follow up on small issues found in the final re-review

Type: task (AFK)
Status: open
Blocked by: —

## Question

The [re-review](../../dashboard-ui-review/re-review-2026-10-09.md) found P3 issues. Which still hold, and what fixes them? Same rules as [Follow up on small issues found during review](../../dashboard-ui-review/issues/_resolved/36-follow-up-small-review-issues.md) and its [round 2](../../dashboard-ui-review/issues/_resolved/37-follow-up-small-review-issues-2.md): check each item first and drop what no longer holds with a reason.

## Items

- **R4** The command input narrows 899 → 855px while the button reads "Generating…" (layout shift). Shot `tmp/t24/08-options.png`.
- **R5** The failure banner pushes the tab bar and the page down 31px when it appears.
- **R11** Text-check preview: "Send Original" is the large primary, "Send with edits" and "Cancel" are small and stacked. Connection form: Test and Cancel are smaller than Save.
- **R12** Panels go stale across tabs (after a game delete, Worlds still says "(1 game)"). Decide whether a cross-panel refresh is worth it or rule it out.
- **R13** Deleting the last saved game leaves an empty "Saved Games" heading with no empty-state line.
- **R14** Rename reloads onto the Game tab, not Games; the rename disclosure is cramped and misaligned.
- **R15** Create World with an empty Map JSON shows only "That action failed."; the grey placeholder JSON looks like a value; the "Auto-generate options" checkbox is centred with its label below.
- **R17** Original finding 4.8: world cards use two layouts by description length.
- **R18** Preset previews cut mid-word with no ellipsis; "Set Active (…)" is a large primary beside small buttons.
- **R23** Original finding 3.5: the large empty sidebar area below the portraits.
- **R24** `.agents/skills/chronicler-ui-investigator/SKILL.md` still describes `#connection-status` ("always reads Connected") and `/fragment/action-area`; both are gone.
- **R25** `stub::options::test_options_dock_edit_fills_without_submitting` is reported LEAKY by nextest.

## Done when

- Every item is fixed or dropped with a reason.
- `python build.py` is green. Commit after user approval.
