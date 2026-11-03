# Parts-kit prototype: Lisette, Gabriella and Louise from a kit and a recipe

Collected on 2026-11-02 for [Prototype a parts kit with a recipe](../../issues/05-prototype-parts-kit.md).

Labels: **known** = seen in a render or a file. **inferred** = a conclusion from known facts. **guessed** = no evidence.

## How it was made

- One `implementer` agent (Claude Sonnet) built a composer, a part catalogue and three recipes in 16 minutes and 6 render rounds.
- 16 parts came from the owner's sample in `tmp/maid/`. The agent drew 13 stand-in parts with code: 5 eye sets, 5 mouths and 3 marks (`tmp/character-art/kit/standins.py`, Pillow shapes, 4× supersample).
- The sample has no known source or licence. So the shots stay in the git-ignored `tmp/character-art/kit/shots/`. This file links to them and does not copy them.
- The agent's full report: `tmp/character-art/kit/notes.md`.

## The method (known)

- **Shared canvas.** Every part is a 300×300 cell on one canvas. Parts stack with no anchor points.
- **Layer order.** A `layer` number in the part catalogue (`parts.json`) sorts the parts, as the number prefix of the sample file names does.
- **Mask recolour.** A mask part has only pure red, green and blue pixels. Red is the base fill, green is the gloss on hair or the shade on skin, and blue is the outline. The composer paints red with one palette colour and derives green and blue from it with rules (for hair: lighten 45 %, darken 50 %).
- **Expression.** An Expression swaps four face slots: eyes, brows, mouth and mark.

## The recipe (known)

A recipe is 19–27 lines of hand-written JSON. Lisette:

```json
{
  "id": "lisette",
  "palette": { "hair": "#7a4a2b", "skin": "#fde4d2", "iris": "#3f82e0", "brow": "#4a2c18", "lip": "#e0607a" },
  "parts": [["hair_bob", "hair"], ["skin", "skin"], ["blouse_frill"], ["cap_white"], ["fringe_blunt", "hair"]],
  "face": {"eyes": "iris", "brows": "brow", "mouth": "lip"},
  "layers": {"brows": 15.5},
  "expressions": {
    "neutral":   {"eyes": "eyes_open",   "brows": "brow_calm",   "mouth": "mouth_line"},
    "happy":     {"eyes": "eyes_closed", "brows": "brow_high",   "mouth": "mouth_smile_open", "mark": "mark_blush"},
    "angry":     {"eyes": "eyes_angry",  "brows": "brow_angry",  "mouth": "mouth_grit",       "mark": "mark_vein"},
    "surprised": {"eyes": "eyes_wide",   "brows": "brow_raised", "mouth": "mouth_o",          "mark": "mark_bang"},
    "sad":       {"eyes": "eyes_sad",    "brows": "brow_sad",    "mouth": "mouth_frown",      "mark": "mark_drop"}
  }
}
```

- `palette` names colours. `parts` names the fixed parts and the palette colour of each. `face` gives one palette colour to each face slot.
- The Expression table was the same for every Character. So the catalogue holds a default table, and Louise uses `"expressions": "default"`. Inferred: a shared table with overrides for each Character is the better shape.
- `layers` overrides the layer of one slot. Lisette needs it, because the sample draws brows over the fringe.

## Part count (known)

- 29 parts cover two Characters × five Expressions: 10 body parts (3 back hair, 1 skin, 2 blouses, 2 headwear, 2 fringes), 5 eyes, 5 mouths, 5 brows and 4 marks.
- A third Character (Louise) added 2 parts.
- Guessed: a kit of about 24 parts shows the same at 80 px, because the brows are 1–2 px or hidden under the fringe.

## Results

| Check | Result |
|---|---|
| Composition (layer order, shared canvas, swap of face parts) | Works with no change to the method (known) |
| Mask recolour of hair | Works for brown, blonde and black. Black hair gets a pale grey rim from the gloss rule (known) |
| Mask recolour of skin | Works for fair and tan from one base colour (known) |
| Recolour of clothes | The sample blouses have no mask. Only a hue tint of the ribbon works (known) |
| Third Character (Louise) | 19 lines, about 10 s of tool time, 2 compose runs. One catalogue fix, no recipe fix (known) |
| Expressions at 80 px | Happy, angry and surprised read at once. Sad is close to neutral, and the drop mark carries it (inferred) |
| Closeness to the chibi references | Head, hair and outline: high, because the sample parts are from that game (inferred). Stand-in eyes: medium (known) |

## What the composition did badly (known)

- **The face position is fixed.** The canvas places every face at the same spot, so all Characters look like siblings. A different head shape needs a new head part and new face parts.
- **Brows against the fringe.** Over the fringe they look like scratches in the hair. Under a blunt fringe they vanish.
- **Headwear.** One fixed shape at one fixed place. The nurse cap stands in for a maid headdress and a chef hat, and it reads as a nurse.
- **Clothes.** Only a collar shows.
- **Hair parts must match.** Gabriella's back hair and fringe come from two styles.

## Shots (in `tmp/`, not copied)

- `tmp/character-art/kit/shots/vs-references.png`: the references on top, then Lisette, Gabriella and Louise × five Expressions.
- `tmp/character-art/kit/shots/sidebar-all_x3.png`: the portrait strip at 80 px (×3), next to the painted headshots.
- `tmp/character-art/kit/shots/dashboard-<expr>.png`: the dashboard for each Expression.

## The `red` style (owner, after the shots)

The owner showed the `red` emote set of pi-emote (<https://github.com/cgxeiji/pi-emote/tree/main/emotes/red>, MIT licence, a copy in `tmp/character-art/reference/pi-emote-red/`) as the style to try next.

- 20 frames, each 110×110 px: a three-quarter bust with long red hair, hands and props (known).
- Each frame is a whole picture. The blink frame changes 18 % of the pixels across the whole frame, not only at the eyes (known).
- Each frame has about 7,000 colours with many near-copies (known). So an image model made the frames and someone shrank them to pixel size (inferred).
- What the kit method needs for this style (inferred): many shade tones instead of three. A grey value layer with a colour ramp for each palette slot does that. Hands and props are out of scope (map).
