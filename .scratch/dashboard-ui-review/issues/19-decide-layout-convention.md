# Decide the panel layout convention and supported viewports

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Panels do not share a layout:

- Settings scrolls the whole tab. Prompt Presets scrolls an inner box.
- Content widths are about 736px (Settings, Games), 826px (Presets), and 1216px (Worlds).
- Worlds padding changes from 32px to 16px after the edit form's Cancel.
- World cards use two layouts, depending on description length.

What convention should every panel follow? And which viewports does the dashboard support? At 480px the tab bar overflows, and the story log shows only about 285px of story.

## Context

- Findings 4.6–4.9, plus the empty sidebar area in finding 3.5. Screenshots 08, 12, 14, 15, 16, 19.
- Editing Map and Scenarios JSON in plain textareas (finding 4.8) is a separate question. Include it or push it to the fog.

## Done when

- The decision is in the ticket answer. Implementation tickets are created.
