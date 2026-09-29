# Decide whether failed LLM calls appear in LLM Messages

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The LLM Messages tab lists only successful calls. The two failed Quantifier requests from the review turn were missing (finding 1.2). Should failed calls be recorded and shown? If so, what does a failed entry hold: error text, duration, attempt number, fallback used?

## Context

- Screenshots 17, 18.
- The recorder is `src/application/llm_recorder.rs`. The record type is `src/domain/model/llm_message.rs`. Scout the save path before the grilling. Finding facts is the agent's job.
- Minor: the panel also has no duration or token counts. Decide whether they belong here or in the fog.
- Related to [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md), but separate: this is about recording, not alerting.

## Done when

- The decision is in the ticket answer. Implementation tickets are created.
