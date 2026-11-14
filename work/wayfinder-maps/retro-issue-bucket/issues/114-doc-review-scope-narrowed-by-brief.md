# `/simple-after-plan` lets a lead narrow the document review to the changed lines

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`simple-after-plan` says to run `/document-review` "over the changed `.md` files in scope". Must it say that the review also covers each doc that describes the changed code?

## Context

Source: `/retro` 2026-11-13, the long-term-memory-and-state session `01a1cc56` (base story memory, ticket 14, and its `/simple-after-plan` pass), with the document reviews `df29da40` (round 1) and `c027c6d4` (round 2). This finding names **Won't do 75** and brings new evidence against its reason: no delegate report caught the narrowing.

- **known** The round-1 brief said "for the changed ones only the changed parts" and "Not in scope: … the code". `document-review/SKILL.md:17` says "the diff is where the review starts, not where it stops".
- **known** `prompt_system.md` already said a call asks "never more than the ceiling"; the new writer call broke it. Round 1 did not read that sentence. Round 2, briefed to check each statement against the source on disk, found it, and the fix became scenario 46.11.
- **inferred** Leads read `simple-after-plan/SKILL.md:10` as "changed lines only".

## Options to decide

- **A.** Reword `:10`: "the changed `.md` files and every doc that describes the changed code; the brief keeps the skill's Scope section".
- **B.** Nothing: Won't do 75 stands.
