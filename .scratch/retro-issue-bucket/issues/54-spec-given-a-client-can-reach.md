# State that a scenario's Given must be a state a client can reach

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

In C4 and C5 the lead deleted six scenarios whose Given no client request can produce: `dashboard.md` 39.1 case 3, `games.md` 18.3, `swipe_new.md` 11.1, 11.3, 11.4 and `retrigger.md` 14.2. Each one was found late, by a review, after a test had written records directly to fake the state. The refusal in the code stays as a guard. One exception was kept: a World data change between two boots can put a room off the map.

Should `tests/STRATEGY.md` ("Spec scenarios") state the rule?

- **Yes, one sentence.** A scenario's Given is a state that client requests, or a data change between boots, can produce. A test that writes records to make an unreachable state does not cover a scenario.
- **Review rule only.** `CODING_STANDARDS.md` gets it as a judgement call for the spec-axis review.
- **Leave it.**

## Context

- Source: `/retro` 2026-10-19 on the core step of the Rust rebuild (ticket 07, the C4 and C5 sections; reflection "a spec scenario whose precondition no client request can produce is deleted").
