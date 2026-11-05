# Confirm where each product fault goes

Type: grilling (HITL)
Status: open
Blocked by: 01

## Question

For each product fault that the audit found, which map owns the fix, and which faults does this map fix itself?

## Context

- **Candidates** of [the report](../assets/README.md):
  - a provider error longer than 500 bytes can panic the Job (R4)
  - each HTTP route is listed by hand in 6 tables (R5)
  - an event Narration sends the Narration before it twice (R7)
  - a failed Game create, rename or delete shows "Internal Server Error" (R8)
  - the stub server copies parts of the engine (R9)
  - the model workarounds have no measurement that says when they can go (R10)
  - some settings have two owners (R12)
  - two copies of the agent prompt (R13)
  - the player-text filter no longer reaches the typed Input (R16)
  - leftovers of the week's removals (R24)
- **Mostly routing.** Most rows have one sensible owner, so this is probably one round. R7 and R16 are in code that [Long-term memory and state](../../long-term-memory-and-state/map.md) changes next. The report proposes to fix the byte-500 panic (R4) here, with the lint `clippy::string_slice`.
- **The size of the R4 fix (plan review, 2026-11-04).** The crate lints are the attributes at the top of `src/lib.rs`. An `rg` finds 5 string slices in `src/` outside the tests, and 1 in `src/test_support/fixtures.rs:648` (recounted by ticket 01 and the coordinator). Two can panic: `transport/utils/response.rs:140` and `quantifier/utils/orchestration.rs:110`. Three slice at an index that `find` returned, so they are safe: `quantifier/utils/parser.rs:95` and `:120`, and `domain/model/action.rs:27`. The lint flags all of them (inferred: it flags every string slice). So the fix is one session or less.
- **The agenda** is the section for this ticket in `../assets/dispositions.md`, from [Propose a disposition for each candidate](01-research-propose-dispositions.md). Ask at most 3 questions in each round (map, Notes).

## Answer

Record the route of each fault. For each fault that this map fixes, create a task ticket. For each routed fault, record the ticket it became in the owning map.
