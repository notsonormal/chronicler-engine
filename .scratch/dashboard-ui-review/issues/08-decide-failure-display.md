# Decide how the dashboard shows each kind of failure

Type: grilling (HITL)
Status: open
Blocked by: 04, 05

## Question

For each kind of failure, where does the dashboard show it, how loudly, and how does it clear?

- a background poll that fails
- the server being unreachable
- a degraded Agent (for example, the Quantifier falling back to fallback NPC IDs)
- a generation error
- a user action that fails

And what happens to the top toast (`#error-notification`) and to the header's hardcoded "Connected"?

## Context

- Findings 1.1 and 1.3–1.7. Screenshots 02, 03, 04.
- Earlier proposal (not decided): route by kind.
  - Background problems become a steady state in the header.
  - Generation errors show only in the status display, clamped to one line.
  - Action errors show inline next to the action.
  - The toast is retired, or kept only for rare one-off events.
- A dead server fires `htmx:sendError`, not `htmx:beforeSwap`, so today it shows nothing.
- Read the answers of the two review tickets (04, 05) first. They list the failure paths.
- `CONTEXT.md` has no term for health or a degraded state. Use `/domain-modeling` to settle terms and add them to the glossary.

## Done when

- The decision is in the ticket answer.
- The Theme 1 fog in the map graduates into implementation or prototype tickets.
