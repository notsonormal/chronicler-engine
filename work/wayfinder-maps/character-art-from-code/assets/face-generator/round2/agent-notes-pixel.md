# Face generator, round 2: shaded pixel art (`r2-pixel`)

Labels: **known** = seen in a render or a file. **inferred** = my conclusion. **guessed** = no evidence.

## The look I chose, and why
- Shaded pixel art on a native 112×112 grid, a bust like Red. I kept the start direction. No change of direction was needed. (known)
- Every shape is a mask of grid pixels (sampled at 4× and cut at 50%). Every outline is a mask dilated by 1 px. Every shade is a mask shifted against another mask. Nothing is scaled down, so the outlines stay clean. (known)
- Proportions: normal head, big anime eyes (about 15 px wide, 14 px tall), small nose and mouth, a turn of 6–13° and an optional roll. I chose this because it is the shape of the painted pictures and of Red. (inferred)
- Hair has base, shade, light and line tones. A fan of strand lines runs from the hair part to the clump gaps. A gloss band crosses the dome. Hair is not flat, because flat hair looked dead next to Red. (known)
- Output is PNG, 112×112, alpha. The browser scales it to 80 px with its default smoothing. That looks acceptable in the strip. (known)

Files (all in `tmp/character-art/facegen/r2-pixel/`): `facegen_px.py` (generator), `chars/*.json` (Characters), `sheet_px.py`, `dash_shot.py`, `zoom.py` (tools; the dashboard script only loads the page and edits the DOM, no Action, no POST).
Run: `python3 facegen_px.py [name ...]` writes `out/<name>/<expression>.png`.

## How the generator works (known)
Head-local coordinates (origin at the face centre) go through a small `Head` transform (roll about the chin). Eyes, nose and mouth sit at head longitudes, so a turn moves them and narrows the far eye. Layers, back to front: back hair, body and garment, neck, face with jaw shade and blush, eyes, brows, nose, mouth, front hair (fringe clumps, side locks, undercut), brows repainted over the fringe, headwear, jewellery, marks (vein, sweat drop, tear).
Eyes: an upper and a lower lid curve; the Expression moves the lid at the inner and outer corner, covers part of the eye, and sets the iris size. Iris has a dark top band, a light lower crescent, a pupil, one big and one small highlight. Happy replaces the eye with an arc.
The Expressions are the table `RULES`. A Character gives a default `mood`, and the table keeps a share of it per Expression.

## Parameter list
A Character file holds only values that differ from `DEFAULTS`. Every parameter has a default, so `{}` renders.

| Group | Parameters |
|---|---|
| Head | `turn` (deg), `roll` (deg), `facing`, `skin`, `blush`, `face.w`, `face.chin`, `face.jaw` |
| Eyes | `eye.iris` (colour), `eye.w`, `eye.h`, `eye.gap`, `eye.y`, `eye.tilt`, `eye.lash`, `eye.flick`, `eye.iris_w`, `eye.iris_h`, `eye.line` |
| Brows | `brow.color`, `.thick`, `.y`, `.len`, `.arch`, `.over` |
| Nose, mouth | `nose.y`, `nose.kind`, `mouth.y`, `mouth.w` |
| Mood | `mood.cover` (half lid), `lid_in`, `lid_out`, `brow_dy`, `brow_ang`, `brow_skew`, `curve`, `skew` (smirk) |
| Hair | `hair.color`, `hair.dome` (`rx`, `ry`, `cy`), `hair.back` (`len`, `w`, `flare`, `wave`, `top`, `curl`, or null), `hair.fringe` (`n`, `tip`, `depth`, `arc`, `slope`, `split`, `shift`, `part`, `part_depth`, `bow`, `jit`, `seed`, `lean`, `origin_dx`, `origin_y`), `hair.locks` (`len`, `y0`, `thick`, `inset`, `flare`, `drift`, `wave`, `near`, `far`), `hair.undercut` (`y`, `x`, `low`), `hair.shine` |
| Wear | `headwear` (`kind: headdress`, `color`), `garment` (`kind: maid / gown / chef`, `color`, `trim`, `bow`, `scarf`, `button`), `earring`, `necklace` |
| Overrides | `expr.<name>` replaces any Expression value for that Character |

A new Character must set (inferred from Louise): `skin`, `eye.iris`, `hair.color`, the hair group (`fringe`, `locks`, `back`) and `garment`. For a distinct personality also `mood`, `mouth`, `turn`/`roll`.

## Sizes (known)
- Generator: `facegen_px.py` 861 lines, 753 non-blank.
- Character files, compact JSON: Lisette 432 bytes, Gabriella 737, Louise 485. On disk, pretty-printed: 599, 1,041, 685.
- PNG per Expression (a whole Portrait, 112×112):

| Character | neutral | happy | angry | surprised | sad |
|---|---|---|---|---|---|
| Lisette | 3,608 | 3,301 | 3,662 | 3,699 | 3,767 |
| Gabriella | 3,620 | 3,239 | 3,646 | 3,766 | 3,728 |
| Louise | 2,939 | 2,577 | 2,921 | 3,123 | 3,079 |

## Render rounds for Lisette and Gabriella: 8 Chrome sheets (`out/sheet-r1.png` … `sheet-r8.png`) plus free zoom checks with Pillow
| Round | Faults seen | Fixed? |
|---|---|---|
| 1 (Lisette only) | Hair mass huge, face small; purple shade on the back hair; white "ear muffs" (lappets) beside the face; bib and straps boxy | r2 |
| 2 | Gabriella: hair a helmet; side locks like thin straps; fringe a straight cut | r3 |
| 3 | Fringe lines too short; Gabriella's sweep too weak; hair shade olive-brown | r4 |
| 4 | Fringe edge smooth, no clumps (depth too small against the slope); a hard line under the neck; mouth tiny | r5 |
| 5 | Dome flat on top; no gloss | r5 (gloss band added) |
| 6 | Brows hidden by the fringe, so angry and sad read from the eyes only | r7 (brows repainted over the hair) |
| 7 | Lock sides too bulbous; back hair stuck out at the top right (Gabriella) | r8 |
| 8 | Final. Faults left: see below | |

Generator fixes that reached every Character at once (known): the warm hair shade, the gloss band, brows over the hair, the neck and neckline outline rule, the eyelid tint.

## How good it looks (my reading; the owner decides)
- At 80 px (known, from `out/dashboard-*.png` and `out/sidebar-all-expressions.png`): the five Expressions read, and Lisette and Gabriella read as two people at once. The cause is hair shape and colour, clothes and eye colour, not face shape. Blue and purple eyes survive the scale-down. Brows, tears and the smirk are only 1–2 px, and they blur.
- Large (known): it looks like clean, simple pixel art. It is cute, not as rich as Red. Red has deeper hair shading with many strands, a narrower face and a shaded body. My bodies are flat blocks. Hair is much better than a flat fill, but the strand work is regular. (inferred: a human pixel artist would still win by a lot.)
- Against the painted pictures (known): colour, hair colour, headdress and mood match. Gabriella's sly smirk reads in neutral. Lisette's neutral reads calm and a bit sweet, not as stern as the painting.
- Weakest Expressions (known): Gabriella sad (the tear sits on the earring and hair), Lisette surprised (the far iris shows a white ring), both angry ones are readable but the eyes darken. Happy and neutral are the best.
- The two faces share the same eye, nose and mouth shapes. Only size, tilt, colour, lids and mood differ. (known)

Do they read as different people? Yes at both sizes (known). Different face geometry is thin, so with the same hair and clothes they would look like siblings (guessed).

## Louise cost (third Character, from the JSON text only, no picture)
- Render runs: 7 (default params; undercut v1 still bald with a flat cut; stubble at eye level; a mask bug crashed it; stubble correct; shared-shading re-run; final dome and fringe tweak). Wall time of the Louise work, about 2.5 minutes of tool time plus my reading. (known)
- Generator change: one, the `hair.undercut` parameter (about 20 lines): it cuts the side hair above a line and paints a dithered stubble tone. Everything else was parameters. (known)
- Honest caveat: I had read the Louise JSON before I wrote the generator. So the `chef` garment kind was already in the generator and is not counted. Counting it, the cost is the undercut plus the chef garment (about 35 lines). (known)
- Result: reads as a third person (tan skin, yellow eyes, black undercut, chef whites) at 80 px. Weak: the top hair is a plain lump, the stubble band looks like a cap, the face is round. (known)

## Answer to the ticket question for this look (inferred)
One generator can produce a consistent pixel-art set for several Characters, and a new Character costs a short JSON plus occasional generator work for a new hair type. The result clears "usable and attractive" for the strip. It does not reach the Red reference. The gap is body and hair richness, which sits in the generator, so it can still be closed in one place.
