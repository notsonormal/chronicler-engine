# Prototype a baked VRM toon model

Type: prototype (HITL)
Status: open
Blocked by: 10

## Question

Can a VRM model, rendered headless with its five Expression presets, read as "somewhat like" the painted pictures at 80 px? What is the cheapest way for a coding agent, with no GUI, to make a second face with a different hair colour, fringe and outfit?

## Context

- Family 4 of the [fresh-view findings](../assets/fresh-view/findings.md), route C. It is the only family where Characters, Expressions and clothes are all parts and numbers, with no painted picture and no image model. Its look is the weakest of the three routes (inferred).
- Decide the look by sight first. Use an existing free VRM model and check its licence. Do not learn VRoid Studio before the owner accepts the look.
- Bake to PNG at build time. Chronicler has no JavaScript framework.
- Run [the shared test of the approaches](../map.md) (map Notes), with the closest VRM faces you can make for Lisette and Gabriella. Record the cost of a third Character.
- Throwaway: files under `tmp/`, no change to `src/`.

Done when the owner has seen the renders in the strip next to the painted pictures and the answer records the owner's verdict.
