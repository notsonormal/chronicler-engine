# Art-style prototype: Carla in three formats

Collected on 2026-11-02 for [Prototype the art style](../../issues/04-prototype-the-art-style.md).

Labels: **known** = seen in a render or a file here. **inferred** = a conclusion from known facts. **guessed** = no evidence.

## How it was made

- Three `implementer` agents (Claude Sonnet) drew Carla in parallel, one format each. Each saw only `data/characters/redmist_estate/carla.json` and a format brief. None saw the painted pictures.
- Each agent had a check and render tool (`tmp/character-art/portrait.py`) and a limit of 8 render rounds. The brief asked for five Expressions: neutral, happy, angry, surprised and sad.
- The brief asked for her eyes to show, so that the Expression can read. The sunglasses go in the pocket or on the hair.
- The dashboard shots inject the Portraits into the portrait strip of the probe server at 80 px (`tmp/character-art/dash_shot.py`). The script sends no Action. The Game on the probe server is "Redmist Estate — 02 Nov 2026 (1)", not the 41-Message fixture of the Dashboard design map.

## Results

| Format | Sheet | File size | Rounds | Strongest | Weakest at 96 px |
|---|---|---|---|---|---|
| Grid 32, layered (base and face patches, 14 colours) | [sheet](sheet-grid-layered.png) | 2.5 KB | 4 | angry, surprised | neutral, sad (sad brows are 1 px off neutral; the tear carries it) |
| SVG, 64×64 viewBox (flat vector with outline) | [sheet](sheet-svg.png) | 15.5 KB | 4 | angry, surprised | neutral, sad, happy (the teeth carry it) |
| Grid 48, one whole frame for each Expression (22 colours) | [sheet](sheet-grid-frames.png) | 13 KB | 5 | neutral, angry, surprised | sad, then happy |

Dashboard shots, with the painted Carla headshot, the painted Room picture and the painted Lisette next to them: [neutral](dashboard-neutral.png), [happy](dashboard-happy.png), [angry](dashboard-angry.png), [surprised](dashboard-surprised.png), [sad](dashboard-sad.png). Sidebar crops of all five: [sidebar-all-expressions.png](sidebar-all-expressions.png).

## What the prototype shows

1. **The Expressions read (known).** In all three formats, angry and surprised read at once at 80–96 px. Sad is the weakest in all three. A tear carries it. The research question "does 32×32 fix angry and happy" has a yes: both read in the Grid 32 version.
2. **The likeness fails (known, from the renders).** All three Carlas read as a man in a suit. The description says "relatively attractive, athletic curves". No version shows that. The agents report the same: "a 32 px grid gives an anonymous face", "faces drawn this way drift to a generic cartoon, and gender and age cues need many small numeric tweaks".
3. **The look clashes with the painted art (inferred).** The painted pictures are soft, detailed anime paintings. Next to them, each code Portrait looks like it comes from another game. This is my reading of the shots. The owner judges it.
4. **No agent wrote the data by hand (known).** All three wrote a Python generator (`gen.py`, `gen_carla.py`) that writes the JSON: mirrored half-grids, shape rules, helper functions for brows and eyes. Two agents said that typing rows directly is "slow and error-prone". For goal 3, the real authoring form is a script, not the data. This feeds the fog item "The data format and the build step".
5. **The render loop is required (known).** Every agent fixed faults that it saw only in the render: brows merged with the fringe, angry read as a grin, the scar read as a rash.
6. **The whole-frame format repeats the bust (known).** Grid 48 is 13 KB for five frames where only the face changes. The layered Grid 32 is 2.5 KB.

## What the prototype did not test

- A second model, or a second agent on the same format.
- A version that matches the painted style (soft shading, anime proportions). No format here aims at that.
- Animation, clothes and the Persona.
- A human editing the art in an editor.

## Round 2: the owner's chibi style (2026-11-02)

The owner rejected round 1: Carla is a woman (`data/images/carla.png`), and the wanted look is chibi anime faces ([reference 1](../../../../tmp/character-art/reference/style-ref-1.jpg), [reference 2](../../../../tmp/character-art/reference/style-ref-2.jpg)). The round 1 brief gave no gender, hid the paintings and asked for "not a chibi". Round 2 gave each agent the paintings and the two references. So round 2 does not test goal 3 (a Character from text only).

| Format | Sheet | File size | Rounds | Time | Weakest at 96 px |
|---|---|---|---|---|---|
| SVG, 64×64 viewBox | [sheet](round2/sheet-r2-svg.png) | 19.2 KB | 3 | 4 min | happy (small crescent eyes; the smile and blush carry it) |
| Grid 48, layered, 20 colours | [sheet](round2/sheet-r2-grid48.png) | 6.7 KB | 4 | 5 min | neutral (reads calm, not stern); sad against neutral |
| Grid 64, layered, 26 colours | [sheet](round2/sheet-r2-grid64.png) | 7.8 KB | 6 | 6 min | sad against neutral; the vein mark and the sweat drop are small |

Dashboard shots: [neutral](round2/dashboard-neutral.png), [happy](round2/dashboard-happy.png), [angry](round2/dashboard-angry.png), [surprised](round2/dashboard-surprised.png), [sad](round2/dashboard-sad.png). Sidebar crops: [all five](round2/sidebar-all-expressions.png).

What round 2 shows (coordinator's reading, inferred unless marked):

- All three read as a woman and as one chibi family. Every Expression reads at 80 px in the SVG version. The anime marks (vein, sweat drop, tear, blush) carry a lot of the Expression.
- **SVG** is the closest to the references in finish: thick outline, glossy hair clumps, big eyes with highlights. It is smoother than reference 1, which has a stepped pixel edge.
- **Grid 48** has the stepped pixel look of reference 1, but it reads younger and more boyish than Carla.
- **Grid 64** has the best likeness: the low aviators and the hair are like the painting. But the sunglasses crowd the lower face, and the hair has few gloss highlights.
- With a style reference and the paintings, each agent took 3–6 render rounds and 4–6 minutes (known). Round 1 took 30–35 minutes for each agent.
- The chibi Portraits still differ in style from the painted Room picture and from the painted Lisette in the same strip. Only a full set of chibi Portraits removes that mix.

## Round 3: Lisette, three-quarter view, reference eyes, flat hair (2026-11-02)

The owner's feedback on round 2: a three-quarter view, eyes like the references, one flat hair colour, and a simpler Character than Carla. The coordinator chose Lisette.

| Format | Sheet | File size | Rounds | Agent's own score |
|---|---|---|---|---|
| SVG, 64×64 viewBox | [sheet](round3/sheet-r3-svg.png) | 17.5 KB | 6 | Angle still a little frontal; all five read; neutral weakest |
| Grid 48, layered, 16 colours | [sheet](round3/sheet-r3-grid48.png) | 5.1 KB | 10 (all) | Angle medium; sad and surprised weak |
| Grid 64, layered, 20 colours | [sheet](round3/sheet-r3-grid64.png) | 8.7 KB | 5 | Angle 7/10, eyes 7/10, hair 8/10, likeness 6/10 |

All three sheets: [sheets-all.png](round3/sheets-all.png). Sidebar crops: [all five](round3/sidebar-all-expressions.png). Dashboard shots: `round3/dashboard-<expression>.png`.

Coordinator's reading (inferred): the SVG version is the only one with an anime look. The two grids still read as pixel art. The owner asked during round 3 whether there are alternatives to the pixel look (see the ticket).
