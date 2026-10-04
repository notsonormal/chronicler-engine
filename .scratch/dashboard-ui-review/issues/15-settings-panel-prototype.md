# Settings panel: roles, buttons and text-check controls

Type: prototype (HITL)
Status: open
Blocked by: 08

## Question

How should the Settings panel look so that:

- the active Narrator and Quantifier connections are visible at the top,
- "Set as Narrator/Quantifier" buttons do not dominate the Edit/Delete controls, and
- Text Check controls cannot contradict each other ("Disabled" with "check before sending" ticked)?

Also: should the ✓ buttons in the story log show when text check is disabled?

## Context

- Findings 4.1, 4.2, 4.3. Screenshots 08, 10, 11.
- Blocked by [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md), because connection health may show on this panel.
- The "Add LlmProviderConfig" copy is in [Copy sweep](20-copy-sweep.md), not here.
- Added from [Keep the command form when a text-check result shows](03-keep-command-form-on-text-check.md): `checkCurrentInput()` in `assets/index.html` has no caller in `assets/`, `src/` or `tests/`, so the "✓ on the player input" button does not exist in the shipped UI. Decide it together with finding 4.3.
- Added from [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md): this panel is now the **only** place to check role health when nothing is wrong. The header keeps no always-on health element, so add a per-role health summary (Healthy / Degraded, from the newest LLM attempt per role) and a connection test here. The ambient signal is the failure banner ([51](51-add-failure-banner.md)), which does not replace this.

## Done when

- A rough prototype is linked from the ticket, and the user has reacted to it.
- The decision is in the ticket answer. Implementation tickets are created.
