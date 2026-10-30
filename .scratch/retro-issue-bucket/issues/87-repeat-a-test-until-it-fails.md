# Repeat one test until it fails, without a hand-written loop

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A browser stub check that passes once at its stage gate can still be flaky. The only way to measure a flake rate, or to confirm a flake fix, is to run the check many times. `build.py` has no option for this, so the lead wrote shell loops around `python build.py test-pattern … --test-target browser` and parsed each new log with `grep`. One loop ran 12 times with no stop condition. Each run hit the 180 s nextest timeout, and the loop used the full 1200 s bash timeout before it ended.

How should a repeat run work, and when must a gate use it?

## Context

- Source: `/retro` 2026-10-29, lead session `01a1743c-7303-77ca-953f-4e72116e9d9f` (the action-answers Final gate and its fix pass, branch `wf/change-driven-dashboard-refresh`).
- **The flake.** Scenario 30.26 (`check_a_delete_of_a_message_that_is_no_longer_last_reports_in_the_notice`, `tests/browser/stub/story_log_edit.rs`) was added in stage 5 (`70bf8dbc`). It passed the stage 5 full gate once. In the Final gate it failed 4 of 6 runs (`logs/build_20261029_021208.log` to `…021324.log`). Cause: `stop_page_timer` (`tests/test_utils/screen.rs:84`) changed `data-poll-ms`, but the tick that the old interval had already scheduled still ran, so a load after the stub append showed the new Message. The helper has 8 callers. Fixed in `a68ad764`; 12 of 12 runs then passed.
- **The loops.** Runs of 6, 8, 12 (hung: `logs/build_20261029_022918.log`, `…023220.log`, `…023522.log` each `TIMEOUT [180s]`, caused by the lead's own broken helper edit) and 12. Each loop iteration also rebuilt nothing but paid the `build.py` prelude.
- **Earlier need.** The visible-tab refresh plan required `run_live_regions_checks` to pass 10 runs in a row as a stage 3 exit check (flake F1); that was also done by hand.
- **Not ticket 84.** Won't do 84 is about re-running an unchanged tree by mistake. This finding is a deliberate repeat to measure a flake.
- **Possible shape (a proposal, not a decision).** `python build.py test-pattern "<p>" --test-target <t> --repeat N`: stop at the first failure or timeout, print "k of N passed" and the failing log. A gate line: a stage that adds or changes a stub browser check runs it with `--repeat 10` before the commit. Open: does nextest's own retry or `--stress` option already give this?

## Further evidence (2026-10-30, design-review fix pass, builder session `d0ca6bc4-7364-47ab-98b4-b354d422cbf8`)

- The brief required each added or changed stub check to pass 10 runs in a row. The builder wrote its own loop script (`tmp/dd-fixpass/loop10.sh`) and ran all 27 `*_checks` runners 10 times (`logs/loop10-run1.log` … `run10.log`), then a second hand loop for the review fixes (`tmp/dd-fixpass/rev-loop.result`). A third hand-written repeat loop in the same effort.
