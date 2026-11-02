# A spec scenario that the ticket's rule changes stays as it was, and the Spec review passes it

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A ticket can decide a rule that makes an existing spec scenario wrong. The implementer can keep the scenario and bend the code to fit it, and the Spec review reads only the diff. How does the review find a scenario that the diff leaves as it was, but that the ticket's rule changes?

## Context

Source: `/retro` 2026-11-02, the coordinator session of layout-trials ticket 15 ([Try the history in the ME format](../../layout-trials/issues/15-history-in-me-format.md)). Build session `f9547a3c`, Spec review session `30effee5`, Standards review session `2eafa64c`.

- **known** The ticket's owner rule D4 names three cases in which an LLM message does not record what was sent. One case is "A failed call stores an empty `raw_request_json`". D8 puts the repair in the ticket.
- **known** Spec Scenario 33.1 (`docs/specs/llm_messages.md`) said: "the record of that entry carries the failure text and no Raw JSON section".
- **known** The build made the real transport store the payload of a failed call. But `ScriptedModel` kept an empty payload on a scripted failure. The build report says "A scripted failure keeps an empty payload, so spec 33.1 is unchanged". So the code kept the old rule at the seam that the scenario's test uses.
- **known** The Spec review listed the D4 cases under "Checked and correct". Its brief (`.agents/skills/code-review/SKILL.md`, step 4) asks for (a) missing requirements, (b) scope creep and (c) wrong implementations. All three come from the diff. Neither reviewer read a spec file that the diff did not touch.
- **known** The coordinator found it while it checked the review findings against the D4 rule. The fix: a scripted failure carries the payload, and Scenario 33.1 and its test changed.
- **Won't do check.** "34 — A change to a port's contract does not list every caller" and the D3 group do not match: there the review caught each case, and here it did not.

## Options to decide

- **A.** Add a point (d) to the Spec sub-agent brief in `code-review/SKILL.md`: "existing spec scenarios and tests whose behaviour the spec changes, but which the diff leaves as they were. Search `docs/specs/` for the behaviour that the spec changes."
- **B.** The ticket or plan lists the scenarios that its rules change, and the implementer brief carries that list.
- **C.** No change. The coordinator's check of each finding caught it.
