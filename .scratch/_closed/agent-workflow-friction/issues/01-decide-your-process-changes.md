# Decide what you will do differently

Type: grilling (HITL)
Status: resolved

## Question

Some frictions only change if you change what you ask for, approve, or invoke. Writing it into a skill doesn't change your behaviour. Which of these will you take on?

1. **Approving ticket lists (F16).** Each ticket costs a claim, a build slot, a ~11-minute gate, and a two-axis review. When an agent proposes tickets, do you check the count against that cost before approving? What's your rule of thumb for "too many"? The decided rule also sets how tickets 02 and 03 cut their graduated work.
2. **Closing at merge (F17).** When a delegate's work merges, do you ask for the ticket's `## Answer` in that same turn? Or do you want ticket 02 to build a check (F18) so you don't have to remember?
3. **Running the UI check (F25).** After a UI-touching ticket, do you invoke `/chronicler-ui-investigator` yourself? Or do you skip it and accept that stub-fixture greens are the proof?

Then the default-drop findings marked `[01]` in [findings.md](../assets/findings.md#default-drop--judgement-advice): F19, F22, F31, F32, F33 and F34. For each one, would you carry it as your own habit? For example, F19 could mean listing prior-art links when you start a map. If not, it is dropped.

## Context

- Findings F16, F17, F25 and the `[01]` default-drop rows are in [findings.md](../assets/findings.md).
- Only answer what you will actually do. "No" is a valid answer and makes the finding a drop.
- Facts already known:
  - A full gate on an idle box took 662.96 s.
  - The window saw two retroactive consolidations, 12→6 and 13→5.
  - Tickets 44 and 46 in `dashboard-ui-review` were closed only through transcript archaeology.
  - The UI investigator ran once in three days, when invoked by hand.

## Done when

- Each of F16, F17 and F25 has an answer: a habit you adopt, a mechanism handed to ticket 02, or a drop.
- Each `[01]` default-drop finding is marked adopted-as-habit or dropped.
- Adopted habits are recorded in the `## Answer`. Nothing is written into a skill unless the answer says so.

## Answer

Resolved 2026-10-04 by grilling. You adopt no new habits, and nothing is written into a skill.

| Finding | Disposition | Reason |
|---|---|---|
| F16 ticket sizing | **Dropped.** No rule; you judge each ticket list case by case. | User choice (Q1=C). The lead disagreed and recommended a "gate floor" rule: merge any ticket whose work is shorter than one gate (~11 min). Tickets 02 and 03 now size their graduated work case by case. |
| F17 close at merge | **Dropped** | User choice (Q2=D). The re-check on 2026-10-04 found 0 stranded tickets in `.scratch/`. |
| F18 stranded-ticket check | **Dropped**, and removed from ticket 02 | Follows from Q2=D, confirmed in Q4=A. Cost accepted: if tickets get stranded again, nothing catches them. |
| F25 UI investigator | **Reframed as a test gap** → [Close the browser-test gap for stuck DOM states](04-browser-test-gap-stuck-dom-states.md) | UI correctness must come from automated tests. `/chronicler-ui-investigator` is not part of proving correctness. Wherever automated tests miss a gap, the gap gets fixed (Q3, Q5=A). |
| F19, F22, F31, F32, F33, F34 | **Dropped** | User choice (Q6). The lead recommended adopting F19, F32 and F34 as habits. |
