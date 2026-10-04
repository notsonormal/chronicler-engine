# Close the browser-test gap for stuck DOM states

Type: research (AFK)
Status: open

## Question

F25 says DOM stuck-state bugs, which stub fixtures don't model, went unverified while "full gate green" stood as proof. The user decided that UI correctness must come from automated tests, not from `/chronicler-ui-investigator`. Which stuck-state bugs lack an automated test, and what tests would close the gap?

## Context

- Comes from [Decide what you will do differently](01-decide-your-process-changes.md), Q3 and Q5.
- F25 is in [findings.md](../assets/findings.md), source row A19. Trace A19 through [synthesis.md](../assets/synthesis.md) and the lens files to the sessions behind it.
- The browser/Playwright tests run with `python build.py browser`. `tests/AGENTS.md` indexes the test files.
- F05 (`tdd` triggers) is a different problem and belongs to ticket 03.

## Done when

- Each stuck-state bug behind A19 is listed. For each one: what got stuck, whether a browser test covers it today (with test name), and why stub fixtures miss it.
- Each uncovered bug has a proposed test: the scenario, and what fixture or harness change it needs.
- The findings are written as a linked asset. The proposed tests are graduated into implementation tickets on this map, cut case by case.
