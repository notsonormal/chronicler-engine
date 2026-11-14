# Every writing and reviewing role runs on the Anthropic plan and stalls at its 5-hour limit

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Must the `implementer`, `strong-implementer` and `reviewer` roles have a model from another provider that is used when the 5-hour window is above pace?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass).

- **known** A comment pass and a document review started at 99 % of the window stood still from 22:53 to 23:51 on 2026-11-11. On 2026-11-12 the coordinator got a 429 at 20:50 and went on at 23:51.
- **known** `.pi/agents/implementer.md:3-5` names the same model as `model` and `spare_model`, so the spare-capacity rule has no effect; `reviewer` and `strong-implementer` name only Anthropic models.

## Options to decide

- **A.** Give each of these roles a non-Anthropic `spare_model`.
- **B.** The start message of a delegate gives the reset time, and the lead waits or picks another role.
- **C.** Nothing.
