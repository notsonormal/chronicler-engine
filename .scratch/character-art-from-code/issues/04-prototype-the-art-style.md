# Prototype the art style

Type: prototype (HITL)
Status: resolved
Blocked by: 01

## Question

Which style of art from code does the owner want for Portraits in Chronicler, or is the answer "no"?

## Context

- Make one Character from `data/characters/redmist_estate/` in 2–3 styles, with the formats that [Research: how art from code is written](01-research-authoring-art-from-code.md) recommends. Each style shows at least three Expressions of the same face.
- Show each style inside the current dashboard on the probe server, next to the painted `carla_headshot.png`, at the size of the portrait strip (the browser tier allows 50 to 120 px wide). Also show the styles next to a painted Room picture. Use `/prototype` and `/chronicler-ui-investigator`. Never send an Action.
- The prototype is throwaway: CSS and markup injected into the probe page, or files under `tmp/`. No change to `src/`.
- From ticket 01: try a layered text grid (24×24 or 32×32), SVG shapes (64×64 viewBox), and a whole-frame grid as the baseline. The SVG test is also the test of whether the owner wants pixel art at all. Give the drawing agent a render script and a row-length check: in the research, quality came from looking at renders. The research drew only one generic figure, so this prototype is the first test that draws a real Character from a description.
- In the research renders, the 24×24 grid faces showed angry and happy less clearly than the SVG faces did (coordinator's reading, inferred). Check whether 32×32 fixes this, and ask the owner whether grids above 32×32 are wanted.
- Let a coding agent write the art from the Character's description in its JSON file, so the prototype also tests goal 3 (map Notes).
- The Dashboard design map runs in parallel and has not chosen a direction yet (map Notes). Judge the style on its own. Matching the direction is a later step in the fog.

## Progress (2026-11-02)

The three versions are drawn and shown in the dashboard: [findings](../assets/art-style-prototype/findings.md). Round 1 (Grid 32 layered, SVG 64, Grid 48 frames) was shown to the owner.

Owner feedback on round 1 (2026-11-02):
- All three versions are wrong: Carla is a woman. Her full look is `data/images/carla.png` and `data/images/carla_headshot.png`. The coordinator's brief caused this: it gave no gender and hid the painted pictures.
- The wanted look is chibi anime faces: [style reference 1](../../../tmp/character-art/reference/style-ref-1.jpg), [style reference 2](../../../tmp/character-art/reference/style-ref-2.jpg). The round 1 brief asked for the opposite ("not a chibi").

Round 2 draws Carla in the reference style, with the painted pictures and the references as input.

Done when the owner has picked a style or said "no", and the answer records the choice, the screenshots (linked as assets), and the format and size it implies.

Owner feedback on round 2 (2026-11-02):
- The angle is wrong: the references show a three-quarter view, not a face straight on.
- The eyes look strange in all three versions.
- The hair has noisy shading. One flat hair colour is better.
- Carla is a hard example (suit, sunglasses). Use Lisette or Gabriella.

Round 3 draws Lisette in a three-quarter view, with reference-style eyes and flat hair (coordinator's choice of Lisette: her headdress and bob are close to the references).

Owner question during round 3 (2026-11-02): the pixel look does not hit the anime style; are there alternatives? Coordinator's answer: the grid formats force pixel art. The alternatives are a flat cel SVG, an SVG with tapered line art, an SVG with soft shading, Expressions drawn as code over the painted picture, or an image model at authoring time (against goal 3). Coordinator's recommendation: stop the grids, and run an SVG-only round.

Owner feedback after round 3 (2026-11-02):
- The target is the reference art, not the painted pictures.
- The reference art is a composite of parts, not one picture. The owner added a sample of the parts to `tmp/maid/` ([contact sheet](../../../tmp/character-art/reference/parts-kit-contact.png)). The owner doubts that one drawn picture can match it.

What the sample shows (known, from the files): 18 PNG parts. The number at the start of each file name looks like the layer order (inferred): 3 back hair, 4 hair, 7 skin, 13 blouse, 15 headwear, 16 fringe, 18 brows, 21 speech marks. Each part sits in 300×300 cells on one shared canvas, so the parts stack with no anchor points (inferred from the sizes). Most parts come as two cells: a pure colour mask (only #FF0000, #00FF00 and #0000FF pixels) and a dark line-and-shade cell. So a colour is chosen per part when the parts are put together (inferred). The sample has no eyes and no mouths. Its source and licence are not known.

## Answer

**An agent that draws a whole Portrait does not reach the wanted look. This approach is rejected (owner, 2026-11-02).** This is not a "no" for the map. Four other approaches go to their own prototype tickets.

- **The target style (owner).** Chibi anime faces as in the references (`tmp/character-art/reference/style-ref-1.jpg`, `style-ref-2.jpg`). These are a big head, a three-quarter view, eyes with a heavy upper lash line, a flat iris and highlights, one flat hair colour with a few gloss spots, thick outlines and flat cel colours. The target is the reference art, not the painted pictures in `data/images/`.
- **What the three rounds showed (known).** Every round gave Expressions that read at 80 px in the portrait strip. Each round came closer to the references and then showed a new fault: gender, then angle, eyes and hair, then the pixel look. The grid formats always look like pixel art. In round 3 only the SVG version had an anime look.
- **Why the approach stops (inferred).** The quality work goes into each single picture, so each fault comes back for every new Character. The reference art is a kit of hand-drawn parts that are put together for each Character (owner's sample, `tmp/maid/`). In that kind of system a fault is fixed once.
- **Facts for later tickets (known).** With a style brief, the reference pictures and a render tool, an agent needs 3–6 render rounds and about 5 minutes for one Portrait with five Expressions. Every agent wrote a generator script, not the data by hand. The anime marks (vein, sweat drop, tear, blush) carry much of an Expression at 80 px. The prototype tools are in `tmp/character-art/`: `portrait.py` (check and render), `shot.mjs` (Chrome screenshot) and `dash_shot.py` (puts Portraits into the portrait strip of the probe server, and sends no Action).
- **Third-party art (known).** The reference pictures and the parts sample have no known source or licence. They stay in the git-ignored `tmp/`, not in `.scratch/`.

Findings and shots: [art-style-prototype/findings.md](../assets/art-style-prototype/findings.md).

### Surfaced

- Prototype tickets for the four other approaches: [Prototype a parts kit with a recipe](05-prototype-parts-kit.md), [Prototype a face generator](06-prototype-face-generator.md), [Prototype parts drawn by an agent](07-prototype-agent-drawn-parts.md) and [Prototype a conversion of the painted pictures](08-prototype-convert-painted-pictures.md).
- [Choose the Portrait approach](09-grill-choose-the-approach.md), after the four prototypes.
