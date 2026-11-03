# Face generator round 2, direction "cel" (folder `tmp/character-art/facegen/r2-cel/`)

Labels: **known** = seen in a render or a file. **inferred** = my conclusion. **guessed** = no evidence.

## The look I chose, and why
Anime cel bust in SVG, normal proportions, head turned about 12-16 degrees (known). I kept the start direction. I did not change direction.
- Clean outline, one shadow tone on skin, hair and clothes, glossy eyes with a gradient iris, two highlights and a lid shadow.
- Hair is a set of pointed clumps, each with a shade tip and a strand line. A thin shine band crosses the crown.
- Bust down to the chest. The view box is cropped (`8 4 112 112`) so the face is large at 80 px.
- Why: the painted pictures are soft and painterly. Clumps, a crown shine and big glossy eyes are the parts that make them read as anime. This direction keeps those parts and drops the painting.
- It is new code (`facegen2.py`). Only the idea "head is a turned cylinder" came from round 1: a feature at longitude `phi` sits at `x = cx + R*sin(yaw+phi)`.

## Files
- `facegen2.py`: generator. `python3 facegen2.py [name ...]` writes `out/<name>/<expr>.svg`.
- `chars/lisette.json`, `chars/gabriella.json`, `chars/louise.json`.
- `sheet.py`, `zoom.py`, `dash_shot.py`: copies and adaptations of the render tools. The dashboard tool loads the page and edits the DOM only. No Action, no POST.
- Outputs: `out/sheet-both.png` (= `sheet-r7.png`), `out/sheet-r1..r7.png`, `out/dashboard-<expr>.png` (5), `out/sidebar-all-expressions.png`, `out/louise/sheet.png`.

## Parameters
Every parameter has a default. A Character file holds only the differences. A file may be `{}` (inferred; the defaults give a valid brown-haired girl).

| Parameter | Meaning |
|---|---|
| `hair`, `skin`, `blush`, `line` | colours. Shade, shine, outline and lip tones derive from them |
| `yaw`, `facing` | head turn in degrees; `left` mirrors the whole Portrait |
| `lid` | default lid droop in every Expression (Gabriella 0.25, Lisette 0.12) |
| `face.w`, `.jaw`, `.chin`, `.top` | head half-width, jaw point (0 round, 1 sharp), chin length, skull top |
| `eye.w`, `.h`, `.tilt`, `.lash`, `.flick`, `.k`, `.gap`, `.y`, `.iris`, `.look` | eye size, cat tilt, lash weight, flick, iris size, spread, eye line, iris colour, gaze |
| `brow.y`, `.w` | brow height and weight |
| `mouth.y`, `.w`, `.lip`, `.lips` | mouth place, width, small lower-lip mark, or a lip fill colour |
| `nose.y` | nose height |
| `back.len`, `.l`, `.r`, `.flip_l`, `.flip_r`, `.top` | back hair length, side volume, end flip |
| `fringe.part`, `.root_y`, `.rs`, `.sag`, `.tips` | parting, clump roots, root spread, and the list of clumps `[phi, tip_y, width, lean, bulge]` |
| `locks` | side locks: `{side, x, root_y, len, w, out, lean, bulge}` |
| `headwear` | `{kind: headdress, color}` (only headdress so far) |
| `undercut`, `ear` | shaved side patch (`line`, `depth`) and the near ear |
| `garment` | `{color, neckline: high/scoop/v, trim, trim_kind: frill/lace/plain, apron, bow, buttons}` |
| `earring`, `necklace` | `{color, gem}` |
| `expr.<name>` | per-Expression overrides (curve, skew, mw, lid, ...) |

A new Character must set (inferred from Louise): `hair`, `skin`, `eye.iris`, `garment`, and the `fringe.tips` list. For a different hair shape also `back`, `locks`, `headwear`. For a different mood: `expr.neutral`.
The hair list is the costly part: tips and locks are hand-placed numbers.

## Sizes (known)
- Generator: 612 lines (541 without blank lines).
- Character files: lisette 1,394 bytes, gabriella 1,450 bytes, louise 878 bytes (indented JSON).
- SVG per Expression (each holds the whole bust): Lisette 13.8-16.7 KB, Gabriella 12.3-15.4 KB, Louise 12.9-15.7 KB. Neutral gzips to 3.6-4.0 KB.
- The SVG is large because each Expression repeats hair and body. The real store is the Character file plus the generator (inferred).

## Render rounds (sheet rounds `sheet-r1..r7`, plus about 9 zoom views of single faces)
| Round | Faults seen | Fix |
|---|---|---|
| 1 | Face a pointed triangle; hair a hood; fringe rays met at one point; eyes small; locks narrow | jaw points rebuilt; roots spread (`rs`); view box cropped; eyes larger |
| 2 | Locks covered the outer eyes; Gabriella bangs hid both eyes; angry brows cut the eye like sticks | locks moved out; tips raised; brow limits |
| 3 | Angry and sad lids a steep triangle (slant too strong); mouth tiny (width in degrees too low) | slant 0.42 to 0.2; mouth width 11 to 14+ |
| 4 | Face hidden by hair; both looked alike | face width 30-33; eyes bigger; strand lines in clumps |
| 5 | Happy eyes slanted like angry slashes; smirk was a hook | closed-eye arc rebuilt; smirk skew lowered, lip fill added |
| 6 | Dot at the inner end of closed eyes; lash too heavy at the inner corner | lash split: thin full line, thick outer part |
| 7 | Smirk dimple too big | dimple smaller. Final |

Generator fixes that reach every Character (known): jaw points, root spread, slant limit, closed-eye arc, lash split, outline tint.

## How good it looks (my reading; the owner decides)
- At 80 px (known, from `sidebar-all-expressions.png`): Lisette reads clearly as an anime maid. Eyes, bow and headdress carry it. Gabriella reads as a blonde with purple eyes and a collar. Her face is small under the hair.
- Large (known): Lisette is the better of the two. Her eyes, bangs and expressions work. It does not reach the painted look: flat skin, little depth, no soft light.
- Weakest Expressions: Gabriella neutral (the smirk is small and the near eye hides under the fringe), Lisette sad (the eye gloss looks grey), both angry (brows are mostly under the hair).
- Strongest: happy and surprised for both (inferred from the sheets).
- Honest limit: the clumps look like leaves at large size. The painted hair has far more strands. The Gabriella and Lisette faces share one build. The outline is thick and dark on blonde hair.

## Two different people?
Yes at 80 px (known): brown against blonde, headdress against none, blue against purple eyes, black dress against purple gown, plain mouth against smirk. No (known): head shape, nose, eye placement and lock shape are the same. A viewer who looks at the faces only, with the hair covered, would see one face (inferred).

## Louise cost
- Written from the JSON text only (878-byte file).
- Render rounds: 3 (first render, after generator change, final sheet). Wall time: about 10 minutes, I did not time it (guessed).
- First render needed no change to look like a person. It failed as an undercut: no shaved side, no ear.
- Generator change: `undercut` (stubble patch with hair marks) and `ear`, about 25 lines. Both are now parameters and any Character can use them.
- Not fixed: the undercut patch is small and is hard to see at 80 px; the crown shine on black hair is grey and too wide.
- Reading: a new Character with ordinary hair costs one parameter file and 1-2 renders. A new hair type costs generator work (inferred from one sample).

## Answer to the ticket question (my view, inferred)
One generator reaches a good anime look at strip size for more than one Character. It does not reach the painted look at large size. The parameter list above is the cost of a Character.
