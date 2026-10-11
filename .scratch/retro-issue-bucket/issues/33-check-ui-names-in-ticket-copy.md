# Check that user-facing copy in a ticket names controls that exist

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Decision ticket 01 and execution ticket 08 of `dashboard-ui-review-phase2` set the new failure sentence to "raise Max Tokens or choose a model that does less reasoning." No dashboard control sets `max_tokens`. The Settings handler saves every connection with `max_tokens: None`, so every call uses the 2048 default. The lead's own recon saw this before implementation: a search for a "Max Tokens" label in the Settings template and `assets/` returned nothing. The lead still delegated the sentence verbatim. Only the spec-axis code review raised it, after the code was built.

`AGENTS.md` already says "If an instruction contradicts what you see, say so before acting." In this run that rule did not fire, because a missing UI label did not register as a contradiction.

Where should the check live?

- **Wayfinder, at ticket-writing time.** A decision or execution ticket that fixes user-facing copy confirms that each named control or label exists (`rg` over `assets/` and the HTTP templates), or it records that the control is new work.
- **The implementer brief.** The lead lists any copy that names a UI element as a claim to check before delegating.
- **The spec-axis review only.** This run already caught it there. No change.

## Context

- Source: `/retro` 2026-10-10, ticket-08 lead session. The finding came from spec review session `01a1267a-ebe3`.
- `src/adapters/driving/http/utils/error.rs`: the `TokenBudgetSpent` sentence.
- `src/adapters/driving/http/settings/handlers/settings.rs`: the connection is built with `max_tokens: None`.
- `src/application/prompting/token_budget.rs`: `MAX_RESPONSE_TOKENS = 2048`.
- `.agents/skills/wayfinder/SKILL.md`: the Ticket Types section.
