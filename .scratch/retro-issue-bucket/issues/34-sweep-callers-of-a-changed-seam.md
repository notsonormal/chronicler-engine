# Sweep every caller when a change alters what a port returns

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Ticket 08 (`dashboard-ui-review-phase2`) changed when `LlmProvider::complete` fails: a reply with only reasoning text now fails instead of returning that text. The lead's recon and the implementer brief listed the roles (narrator, quantifier, options, trigger, arrival, retry). Neither listed the non-role caller `ConnectionTestService::test_connection`, which calls the provider directly with an 8-token probe. A reasoning model spends those 8 tokens on reasoning, so after the change the Settings connection test would fail for a reachable provider. The full gate was green, and the spec-axis review found the problem by reading `connection_test_service.rs`.

The sweep is mechanical: `rg -n "\.complete\(" src/` lists every caller. Nothing prompts for it.

Where should the prompt live?

- **The implementer brief template.** A change to a port's success or failure contract lists every caller of the port method from `rg`, and the brief names each one as in scope or out of scope.
- **The spec-axis review brief.** The reviewer is told to list the callers of every changed port method. This run already did so without being told.
- **`CODING_STANDARDS.md`.** A judgement rule: "a contract change to a port names its callers in the ticket answer."

## Context

- Source: `/retro` 2026-10-10, ticket-08 lead session. Implementer session `01a12666-efe3`, spec review session `01a1267a-ebe3`.
- `src/application/connection_test_service.rs`: `TEST_MAX_TOKENS` is 8, and the call does not go through `LlmCallRecorder`.
- Fix landed in ticket 08: `TokenBudgetSpent` now counts as reachable.
