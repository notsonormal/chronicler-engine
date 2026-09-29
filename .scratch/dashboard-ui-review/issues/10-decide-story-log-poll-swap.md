# Decide how the story-log poll keeps DOM state

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The 2-second story-log poll replaces every entry node, even when nothing changed. A text selection is lost within about 2.6 seconds, and hover and focus state reset (finding 2.4). Which approach stops this?

- **Option A:** the server tells the client "unchanged" (for example, a version check and a no-swap response).
- **Option B:** a client-side morph swap that keeps unchanged nodes.
- **Option C:** pause the poll while the user selects text or edits.

## Context

- Check these facts before the grilling:
  - How does edit mode survive the poll today?
  - Which htmx version is bundled (`assets/htmx.min.js`)?
  - Is a morph extension available?
- Poll cadences are in `docs/diataxis/explanation/dashboard_design.md`.

## Done when

- The decision is in the ticket answer. Implementation tickets are created.
