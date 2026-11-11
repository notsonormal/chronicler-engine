# Prototype a layer split and a rig

Type: prototype (HITL)
Status: open
Blocked by: 11

## Question

Does See-through split the two headshots into layers that are clean enough, and does the auto-rig of Stretchy Studio then give a blink, a mouth change and brow moves that bake to PNG and read at 80 px? How much hand fixing does each Character need?

## Context

- Family 10 of the [fresh-view findings](../assets/fresh-view/findings.md), route B. It is the one route that gives idle motion (goal 2) and outfit variants (goal 4) on the painted look (inferred).
- Main risk (inferred): layer errors where hair goes over the face. The answer must also say whether the crops in `data/images/` must be made again as bust pictures.
- The licence of the See-through weights is not checked. Check it first.
- See-through runs as a Hugging Face demo or on a rented GPU. The sandbox has no Nvidia GPU (known).
- Bake to PNG. Do not use the Live2D web runtime: its licence for a local app with user content is not settled (findings, guessed).
- Run [the shared test of the approaches](../map.md) (map Notes). Record the hand-fixing time for each Character and the cost of a third Character.
- Throwaway: files under `tmp/`, no change to `src/`.

Done when the owner has seen the two Characters in the strip and the answer records the owner's verdict and the hand-fixing time.
