# Prototype the art style

Type: prototype (HITL)
Status: open
Blocked by: 01

## Question

Which style of art from code does the owner want for Portraits in Chronicler, or is the answer "no"?

## Context

- Make one Character from `data/characters/redmist_estate/` in 2–3 styles, with the formats that [Research: how art from code is written](01-research-authoring-art-from-code.md) recommends. Each style shows at least three Expressions of the same face.
- Show each style inside the current dashboard on the probe server, next to the painted `carla_headshot.png`, at the size of the portrait strip (the browser tier allows 50 to 120 px wide). Also show the styles next to a painted Room picture. Use `/prototype` and `/chronicler-ui-investigator`. Never send an Action.
- The prototype is throwaway: CSS and markup injected into the probe page, or files under `tmp/`. No change to `src/`.
- Let a coding agent write the art from the Character's description in its JSON file, so the prototype also tests goal 3 (map Notes).
- The Dashboard design map runs in parallel and has not chosen a direction yet (map Notes). Judge the style on its own. Matching the direction is a later step in the fog.

Done when the owner has picked a style or said "no", and the answer records the choice, the screenshots (linked as assets), and the format and size it implies.
