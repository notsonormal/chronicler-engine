# facegen notes: one face generator, parameter files for each Character (ticket 06, round 1)

Labels: **known** = seen in a render or a file. **inferred** = my conclusion from known facts. **guessed** = no evidence.

## Files
- `facegen.py`: the one generator (590 lines, 530 without blank lines). `python3 facegen.py [name ...]` writes `out/<name>/<expr>.svg`.
- `chars/lisette.json`, `chars/gabriella.json`, `chars/louise.json`: the Characters. Louise is the third-Character test.
- `sheet.py` (sheets through Chrome), `zoom.py` (one big view), `dash_shot.py` (dashboard shots; it loads the page and edits the DOM only, no Action, no POST).
- Nothing outside `tmp/character-art/facegen/` changed. `r3-svg/gen.py` was read and its pieces (headdress ring, collar frill, ribbon, vein, sweat drop, tear) were reused.

## How the generator works (known)
The head is a turned cylinder. A feature at head longitude `phi` shows at `x = xc + R*sin(yaw + phi)`. Its width shrinks with `cos(yaw + phi)`.
One rule narrows the far eye, the far fringe clumps, the far jaw and the far mouth half, and widens the near cheek. So "the whole skull turns" is one formula, not hand-placed shifts.
Every shape is a spline through a point list, so strokes keep one width (no scaled groups). Layers: back hair, body, ear, face, blush, brows, eyes, nose, mouth, headwear, fringe cap, side locks, jewellery, marks.
The Expressions come from the table `RULES` in the generator. A Character can add a default mood and per-Expression overrides.

## Parameter list
A Character file holds only values that differ from `DEFAULTS`. All parameters have a default, so a file may be `{}`.

| Parameter | Type / range | Meaning |
|---|---|---|
| `turn` | degrees, 0..45, default 29 | head yaw; more = more profile |
| `facing` | "right" / "left" | mirror of the whole Portrait |
| `skin`, `hair`, `blush` | colour | skin, the one flat hair colour, blush. Outline, shade and gloss colours derive from them |
| `line` | colour or null | outline colour; null = hair colour darkened |
| `face.w` | 20..26, default 24 | half-width of the head |
| `face.jaw` | 0..1 | 0 round, 1 pointed |
| `face.chin` | px, default 0 | longer or shorter chin |
| `face.cheek` | 0.9..1.1 | near-cheek width |
| `face.ear` | bool | draws the near ear |
| `eye.w`, `eye.h` | half-width px, total height px | eye size |
| `eye.tilt` | px, + = outer corner up | cat eye or droop |
| `eye.lid` | 0..1 | lid cover in every Expression |
| `eye.lash`, `eye.flick` | stroke px, flick px | lash weight and outer flick |
| `eye.iris` | colour | iris; the darker top band derives from it |
| `eye.k`, `eye.y`, `eye.gap`, `eye.r`, `eye.look` | iris size, eye line, eye spread (degrees), radius factor, gaze shift | eye layout |
| `mouth_y`, `nose_y`, `nose` | px, px, "dot" / "tick" | mouth and nose place |
| `mood.lid`, `mood.brow`, `mood.curve`, `mood.skew`, `mood.mw` | numbers | default mood: lower lids, brow shift, mouth curve, one-sided smirk, mouth width. Each Expression rule has a weight `m` (neutral 1, surprised 0) |
| `back.len`, `back.near`, `back.far`, `back.flare`, `back.top` | px | back hair: length, volume near and far, outward flare, crown height |
| `fringe.n`, `.near`, `.far`, `.top`, `.depth`, `.sweep`, `.jit`, `.seed`, `.lean`, `.bulge`, `.end`, `.lines` | numbers | fringe: clump count, tip height at the near and far end, notch depth, side sweep, irregularity and its seed, clump lean, bulge, end height, clump lines |
| `lock_near`, `lock_far` | `{len, w, flick}` or null | side locks, each its own shape |
| `gloss` | list of `[phi, y, size]` | white gloss spots |
| `headwear` | null or `{kind: headdress / band / bow, color}` | headwear |
| `garment` | `{color, neckline: high / scoop, collar: frill / plain / none, collar_color, bow}` | clothes |
| `earring`, `necklace` | null or `{color, gem}` | jewellery |
| `expr.<name>` | rule overrides | per-Expression changes (not needed so far) |

Must set for a new Character (inferred from Louise): `hair`, `skin` (if not pale), `eye.iris`, `garment`. For a different look also: the hair style group (`back`, `fringe`, locks, `headwear`) and the `mood`. The rest can default.

## Sizes (known)
- Generator: `facegen.py` 590 lines.
- Character files: lisette 445 bytes, gabriella 684 bytes, louise 365 bytes.
- SVG per Expression (each holds the whole Portrait, 96 px):

| Character | neutral | happy | angry | surprised | sad |
|---|---|---|---|---|---|
| Lisette | 16,007 | 9,222 | 16,271 | 16,483 | 16,264 |
| Gabriella | 14,115 | 7,370 | 14,439 | 14,645 | 14,438 |
| Louise | 12,669 | 5,832 | 12,919 | 13,111 | 12,984 |

The SVG is large because it holds the hair and body again for each Expression. The real store is the Character file plus the generator (inferred).

## Render rounds for Lisette and Gabriella: 8 of 10 (sheets `out/sheet-r1.png` ... `out/sheet-r8.png`)
| Round | Faults seen | Fixed? |
|---|---|---|
| 1 | Head cut at the left edge; the far side lock covered the far eye; fringe a spiky zigzag; jaw too long and pointed; faces small under a helmet of hair | r2 |
| 2 | Fringe still a wavy line, not clumps; face too small; Gabriella neutral read angry (heavy lid and low brow) | r3 |
| 3 | Fringe tips now pointed; the lash line dropped in a thick vertical bar at the inner corner; far lock still on the far eye | r4 (lash trimmed, lock moved out) |
| 4 | Far eye still on the edge; blush outside the face; happy mouth tiny | r5 (turn 29, narrower far eye by `cos^1.1`, bigger open mouth) |
| 5 | Outline too light on blonde hair; Gabriella smirk a tiny hook; gloss spots sat on the headdress | r6 (darker outline rule, `mood.mw`, gloss defaults) |
| 6 | One gloss spot sat on the hair outline | r7 (gloss placed from the cap curve) |
| 7 | Gabriella hair too heavy for the face | fixed in the same round (crown and back volume lowered) |
| 8 | Near side lock was a rectangle | fixed (tapered lock). Final |

Faults not fixed: the fringe is still smoother than the references (known); Gabriella's near eye is half hidden by the fringe tip in neutral (chosen, "over one eye"); the far eye of both is partly under the far lock.
Generator fixes that reached every Character at once (known): lash trim (r4), far-eye rule (r5), outline colour rule (r6), lock taper (r8; Louise re-rendered with it, no change needed).

## Closeness to the references (my reading, inferred; the owner decides)
| Axis | Lisette | Gabriella |
|---|---|---|
| Angle | medium-high. Features sit right of the centre, the far eye is narrow and near the far edge, the near cheek is wide, the jaw and the locks are asymmetric. The skull outline itself is still near-symmetric, so it is less "turned" than the references | medium-high, mirrored; the near ear helps |
| Eyes | high. Heavy lash line with a flick, flat iris, darker top band, one big and one small highlight, no rings, no lower lash. The sclera is a little whiter than in the references | high. Half-lid, cat tilt and purple iris make her look different from Lisette |
| Hair | medium-high. One flat colour, 2 gloss spots, 2 clump lines. The fringe has pointed clumps of unequal size but is still smoother than the references; the cap is a clean arc | medium. The swept fringe works, but the hair mass is big and looks like a helmet |
| Outline and finish | high. Thick dark outline, flat colours, one skin shade. Cleaner and more vector-like than the stepped reference 1 | same |

Weakest Expressions at 80 px (known, from the strip shots): Lisette: neutral (calm, but the flat mouth is tiny) and sad against neutral (the tear carries it). Gabriella: neutral and sad, because her hair hides much of her face at 80 px; her smirk is the only mood cue.
The anime marks carry angry, surprised and sad, as before.

## Do Lisette and Gabriella read as two people in one shared style? (inferred: yes)
Different through parameters only: hair colour, fringe (blunt, five clumps against swept, four clumps), back hair (bob against shoulder-length with flare), locks (two against one), ear, headdress, collar and bow against scoop neckline with necklace and earring, eye tilt, lid, lash and iris colour, narrower face, mirrored view, and the default smirk. At 80 px in the strip they are two people. The shared style is clear: same outline weight, same eyes family. Gabriella still has the same face "kit" as Lisette (same nose and mouth drawing). That is a limit of one generator (inferred).

## Third Character: Louise (from her JSON text only)
- Read only `description`, `personality`, `summary`. Wrote `chars/louise.json` (365 bytes): tanned skin, black hair, yellow iris, short back hair, no locks, 3-clump fringe swept the other way, white chef collar, a lively mood.
- Render rounds: 1 (a second render after the lock fix, no change). Time: about 25 s of clock time from reading her JSON to the first sheet (tool timestamps; my own thinking time is not separate, so "under one minute" is known, "about 2 minutes" is a safe guess).
- It read as a third person at once (`out/louise-sheet-r1.png`, `out/louise/sheet.png`).
- Not expressible without a code change (known): a real undercut (shaved sides with a short sharp edge: it is only approximated by short back hair), the double-breasted chef jacket with buttons, a chef hat, a neckerchief or a tank top, skin tone that changes the line colour, a dark outline for dark hair (works, but the face outline and hair outline are the same dark, so the hairline has less contrast). Headwear kinds are only `headdress`, `band`, `bow`. Clothes are only the three neckline/collar choices. Each new hairstyle family or outfit is a code change (inferred).
- Louise's weakest points (known): the white jacket and the white collar merge; the hair cap is thin on the near side.

## Verdict for the ticket (inferred; the owner decides)
One generator reached a similar reference-like look for two Characters and a third file worked at once. This answers "yes" for faces. The ceiling is the same as the drawn picture in one place: a model still wrote every shape in the generator, and the fringe and hair outlines are still smoother than the references. Costs of a new Character are low (a file of 400 to 700 bytes, one render round for Louise). Costs of a new hairstyle family or outfit are code changes.
Round 2 with the owner can aim at: a more turned skull outline, a more broken fringe with more clump lines, and bigger heads on the faces (hair mass smaller).

## Paths (all under `tmp/character-art/facegen/out/`)
- Round sheets: `sheet-r1.png` ... `sheet-r8.png` (both Characters, 5 Expressions, 80 px row and 220 px row). Final: `sheet-both.png` (= r8).
- Per Character: `lisette/sheet.png`, `gabriella/sheet.png`, `louise/sheet.png`, `louise-sheet-r1.png`.
- SVG: `lisette/<expr>.svg`, `gabriella/<expr>.svg`, `louise/<expr>.svg`.
- Dashboard shots at 80 px (painted, facegen Lisette, painted, facegen Gabriella): `dashboard-neutral.png`, `dashboard-happy.png`, `dashboard-angry.png`, `dashboard-surprised.png`, `dashboard-sad.png`.
- Strip crop with all five Expressions for both Characters: `sidebar-all-expressions.png` (full page: `_all.png`).
