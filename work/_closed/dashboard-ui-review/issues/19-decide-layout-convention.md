# Decide the panel layout convention and supported viewports

Type: grilling (HITL)
Status: resolved
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

## Answer

**Option A1 — one centered column, and the tab body is the only scroll region.**

- Every panel renders in the same content column: `max-width` 960px, 24px
  padding. This replaces Settings/Games at 800px, Prompt Presets at 900px, and
  Worlds with no cap.
- The tab body scrolls. The inner scroll boxes on Games, Prompt Presets, and
  Worlds are removed; the scrollbar lands at the viewport edge instead of
  floating mid-page (finding 4.6). Settings already scrolls the whole tab.
- The Worlds text inset stops changing because the form view gets the same card
  frame as the list view (or the card inset is removed in both). The review's
  "32px → 16px" was panel padding (16px) plus card padding (16px) stacking, not
  a class toggle.
- The tab bar scrolls horizontally below 1024px, so the 4.9 overflow cannot
  happen.
- **Viewports are declared, desktop-first:** ≥1024×700 is fully supported;
  768–1024 is best-effort (the existing 768px media query stays); phone layouts
  are out of scope and are documented as such.

**Pushed to the fog:** editing Map and Scenarios JSON in structured fields
rather than plain textareas (finding 4.8). It is a separate question and can
wait for a later pass.

### Graduated

[Make the dashboard panels consistent: save model and layout](58-panel-consistency.md),
shared with the save-model decision.
