# Prototype a face generator

Type: prototype (HITL)
Status: resolved
Blocked by: 04

## Question

Can one program that draws a face from parameters (head turn, eye shape, hair style, colours, Expression) reach the reference look for more than one Character?

## Context

- The difference from the rejected approach of [Prototype the art style](04-prototype-the-art-style.md): the quality work goes into one generator that every Character shares. A fault fixed in the generator is fixed for every Character. A Character is a list of parameters.
- The parts are code (SVG paths with parameters), not drawn pictures. The round 3 SVG of Lisette is the nearest start point (`tmp/character-art/r3-svg/gen.py`), and its faults are known: the angle is too frontal, the side locks are symmetric and the neutral Expression is weak.
- ~~Keep the owner's rules from ticket 04: a three-quarter view, eyes like the references, one flat hair colour.~~ Replaced on 2026-11-02 by **The target style** in the map Notes: art that looks good in an anime style, with a free format, resolution and shading.
- Risk (inferred): the generator can hit the same ceiling as a drawn picture, because a model still writes the shapes. Use two review rounds with the owner. If the second round is no closer to the references, record that as the answer.
- Write down the parameter list, and which parameters a new Character needs.
- Run [the shared test of the approaches](../map.md) (map Notes), so the four prototypes compare.
- Throwaway: files under `tmp/`, no change to `src/`.

Done when the owner has seen the two Characters in the strip and the answer records the parameter list and the owner's verdict.

## Progress (2026-11-02)

Round 1 runs: one `strong-implementer` writes `tmp/character-art/facegen/facegen.py` and one parameter file each for Lisette and Gabriella, then a third Character from text only. Shots go to `tmp/character-art/facegen/out/`.

Round 1 is done ([shots](../assets/face-generator/round1/)). Owner question after round 1 (2026-11-02): is the style of the `red` emote set of pi-emote (`github.com/cgxeiji/pi-emote`, `emotes/red`, MIT) possible? Coordinator's check: Red is shaded pixel art with normal head proportions. It is not chibi and it has shaded hair, so it breaks two rules of the target style. Each Red frame is a whole picture, and a blink changes pixels over the full frame. A pixelated facegen render is in `tmp/character-art/pixelate/compare.png`. Waiting for the owner: does the target style change?

Owner answer (2026-11-02):
- The briefs took the owner's rules too strictly, and that is why the art did not improve. The coordinator agrees: each piece of feedback became a hard rule.
- The goal is art that looks good, probably in an anime style, because most Character pictures come from anime-style art. The format and the resolution are free.
- Hair shading is not important. It can go.
- Full-body movement would be nice, but the scope must not grow yet. Waving and other hand poses are ignored (map Out of scope).

Round 2 runs with a goal brief in place of rules. It first tries several looks for one face, then builds the best look for both Characters. Output: `tmp/character-art/facegen/r2-cel/`, `r2-pixel/` and `r2-chibi/` (three agents in parallel, one start look each, shared brief `tmp/character-art/facegen/r2-brief.md`).

Round 2 is done ([shots](../assets/face-generator/round2/): `sheet-80.png` and `sheet-160.png` compare all versions, `dashboard-<expression>.png` shows the three round 2 looks in the strip). All three keep one generator and one parameter file for each Character (known):

| Look | Generator | Character files | Output per Expression | Louise: generator change |
|---|---|---|---|---|
| Chibi (SVG) | 718 lines | 698–1,108 B | 20–33 KB SVG | ears, undercut, hairline, a back-hair fix |
| Cel (SVG) | 612 lines | 878–1,450 B | 12–17 KB SVG | undercut and ears, about 25 lines |
| Pixel (112×112 PNG) | 861 lines | 432–737 B | 3–4 KB PNG | undercut, about 20 lines (35 with the chef garment) |

Owner verdict on round 2 (2026-11-02): all three feel too ugly. Owner's question: is it too western?

Coordinator's reading (inferred, from `sheet-160.png` next to `style-ref-2.jpg`): yes, mostly. The faces use the tools of a western cartoon, not of an anime face. These are round white eyes with a full circle iris and a black pupil, thick slanted brows, a drawn nose, wide mouths with lips and teeth, and acting with brows and mouth. The reference faces use a big dark iris cut by a thick upper lash line, a small white at the sides, almost no nose, a tiny mouth low on the face, thin or hidden brows, and Expressions carried by the eye shape (`^ ^`, `> <`, blank eyes) and by marks. A second cause: the generator lines are even and regular (a row of V cuts, equal clumps). The reference lines taper and the strands are uneven. Waiting for the owner: one last round on that diagnosis, or stop.

Owner answer (2026-11-02): chibi may be the wrong approach. The goal is anime-style faces from code, with different faces, clothes and Expressions, that look somewhat like the existing Character art in `data/images/`. The rounds so far moved little. The map Notes now say this (**The target style**, second update).

Coordinator's reading (inferred): chibi is only part of the cause. The cel version of round 2 had normal anime proportions and was still too ugly. In every round of tickets 04 and 06, a model wrote every shape, and that method seems to stop well below the painted art. Open question to the owner: what does "from code" mean? (a) Code draws every pixel, (b) code puts together parts that are drawn once, as in a character maker, or (c) code edits the painted picture of each Character. The answer decides whether this ticket gets another round or closes with "no".

## Answer

**No. A face generator does not reach a look that the owner accepts (owner, 2026-11-02).** The owner closed the ticket and asked for fresh research instead: [Research: art from code with a fresh view](10-research-fresh-view.md).

- **What works (known).** One generator and one parameter file for each Character. A Character file is 0.4–1.5 KB, and `{}` renders a default. A fix in the generator reaches every Character. Expressions come from one rule table, and a Character adds a default mood (Gabriella's smirk). A third Character (Louise, from her JSON text only) costs one file and 1–7 renders. A new kind of hair or clothes (her undercut, her chef jacket) costs 20–35 lines of generator code.
- **What failed (owner).** Round 1 (chibi, flat) and round 2 (three looks in parallel: chibi, cel, pixel) were "too ugly". The owner asked whether the faces were "too western". Then the owner said that chibi is the wrong target: the art must look somewhat like the painted pictures in `data/images/`, and the rounds moved very little.
- **Why (inferred).** The faces used western cartoon tools: round white eyes with a full iris, thick brows, a drawn nose, wide mouths with teeth. The lines were even and regular. The deeper cause: in all six rounds of tickets 04 and 06 a model wrote every shape. Each round fixed the named faults, but the quality stayed at one level.
- **The parameter list (known).** Head turn and facing; skin, hair, blush and line colours; face width, jaw, chin; eye size, tilt, lid, lash, iris colour and layout; nose and mouth place and kind; default mood (lids, brows, mouth curve, smirk); back hair, fringe (clumps or a hand-placed list), side locks, undercut, ears; headwear; garment (colour, neckline, collar, bow); earrings and necklace; per-Expression overrides. A new Character must set the hair colour, the iris colour and the garment, and the skin if it is not pale. The full tables are in the agent notes.
- **Sizes (known).** Generators of 590–861 lines. Output per Expression: 6–33 KB SVG, or a 3–4 KB PNG for the 112×112 pixel look.

Shots and agent notes: [round 1](../assets/face-generator/round1/) and [round 2](../assets/face-generator/round2/) (`sheet-160.png` compares every version with the painted pictures). The code stays in `tmp/character-art/facegen/`.

### Surfaced

- [Research: art from code with a fresh view](10-research-fresh-view.md), which now also blocks [Choose the Portrait approach](09-grill-choose-the-approach.md).
