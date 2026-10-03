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

Findings from ticket 05 (review create/save/delete flows) that belong to this decision:

- **Network failures on htmx deletes are silent.** With the engine stopped, the confirm dialog closes and nothing happens for connection (screenshot 65), world (66) and game (67) deletes: no toast, item stays. The dead-server gap is already noted above; these are the concrete surfaces. In contrast `deleteMessage()` reports "Failed to delete message" (68) because it checks `response.ok` and catches — use that as the model for the "user action that fails" row.
- **Raw server text reaches the user.** Observed in the toast or inline: `Failed to deserialize form body: missing field 'preset_type'` (prior screenshot 33), `Failed to deserialize form body: missing field 'name'` (35), `Error: Configuration error: Unknown LLM backend 'bogus_provider'` (54, 56), `Invalid map JSON: key must be a string at line 1 column 3` (39), `Invalid scenarios JSON: EOF while parsing a value at line 1 column 0` (42). Decide what reaches the user and what goes to the log.
- Destructive error swaps that this decision gates: tickets [25](25-stop-errors-wiping-panels.md) and [26](26-stop-errors-destroying-cards.md).
- `CONTEXT.md` has no term for health or a degraded state. Use `/domain-modeling` to settle terms and add them to the glossary.

## Done when

- The decision is in the ticket answer.
- The Theme 1 fog in the map graduates into implementation or prototype tickets.
