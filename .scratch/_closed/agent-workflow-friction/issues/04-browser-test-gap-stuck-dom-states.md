# Close the browser-test gap for stuck DOM states

Type: research (AFK)
Status: resolved

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

## Answer

**None of the stuck-state bugs A19 names lacks an automated test today.** All three — the
stranded action area, the frozen second-edit poll, and the swipe restore that forgot
"Generating…" — plus the two adjacent ticket-27 bugs now carry a browser test. Four are tier 2
over the shipped `assets/index.html`; the strand bug also has a tier-3 proof. The gap was real
when A19 was written; it closed through the dashboard-ui-review tickets 27, 42, 43 and 65,
each of which put "a browser test covers X" in its Done-when. `python build.py browser` is green
on the committed tree (62 passed, 0 failed).

The findings, the per-bug test names, and the stub-fixture analysis are in
[dom-stuck-state-test-gap.md](../assets/dom-stuck-state-test-gap.md).

**Why the stub fixtures no longer miss it.** The tier-2 stub serves the shipped shell verbatim,
so client-JS stuck states are reproducible at tier 2. The fixtures that stayed hand-copied
(`action_area`, `header`, `settings`, `prompt_presets`, `worlds`, `games`, `visual_sidebar`) can
drift, but ticket 50 already owns that conversion, and the strand bug's shipped markup is also
covered at tier 3. No new ticket for fixture fidelity — it would fragment ticket 50's decision.

**One residual same-class gap, uncovered.** A persisted `Generating` status with no live
slot — what a mid-flight panic leaves — has no integration test, and the page may not be able
to send the action that heals it. `heal_stale` runs only on the action path and bootstrap; the
status poll reports the stale phase without healing; the client disables the default submit
button while generating, and Enter does not submit through it. Graduated as
[Recover a dashboard stuck on Generating with no live generation](09-recover-stuck-generating.md)
(grilling) because the fix is a design choice — heal in the poll, allow submit while
generating, or add a recovery control. Its tier-1 test is proposed in the asset.

Also recorded in the asset as not graduated: a direct test of the `pausePolling` double-pause
guard, which is unreachable behind the UI lock that 30.15 already pins.
