# r2-chibi: shaded chibi, one generator (ticket 06, round 2)

Labels: **known** = seen in a render or a file. **inferred** = my conclusion. **guessed** = no evidence.

## The look I chose and why
- I kept chibi (big head, big eyes low on the face) and dropped the three-quarter turn. The face is nearly frontal with a small `turn` (features shift, far eye narrows). Reason: the owner's chibi references are nearly frontal. A frontal chibi head is simpler, and the quality work goes into the hair and the eyes.
- Shading is on: a fringe shadow on the skin, a gloss ring and white gloss spots on the cap, flow lines from a hair part, a darker lower tone on each fringe clump, a darker back hair, an eye with a dark top band, a lighter lower crescent and two highlights.
- I wrote a new generator (`gen.py`). I did not reuse the round 1 turned-cylinder code. Reason (inferred): the cylinder rule was the source of the helmet and wig shapes, and the new hair needs separate clumps.
- The hair is made of parts, not one flat shape: back hair, cap, fringe clumps, side locks, wisps. Each fringe clump has its own fill, edge line, strand line and tip shade. A Character can list its clumps by hand (`fringe.locks`), as Gabriella does for her sweep over one eye.
- Format: SVG, 128 x 128 view box, transparent background, mirrored with `facing`.

## Files
- `gen.py`: the generator. `python3 gen.py [name ...]` writes `out/<name>/<expr>.svg`.
- `chars/lisette.json`, `chars/gabriella.json`, `chars/louise.json`: the Characters.
- `sheet.py` (sheets through Chrome), `dbg.py` (one large view), `dash_shot.py` (portrait strip of the probe server; it loads the page and edits the DOM only, no Action, no POST).
- Nothing outside this folder changed.

## Parameters
A Character file holds only values that differ from `DEFAULTS` (all have defaults, so `{}` is a valid file).

| Group | Parameters | Meaning |
|---|---|---|
| Colours | `hair`, `skin`, `blush`, `outline` | outline, shade and gloss colours derive from `hair` and `skin` |
| Head | `turn` (-1..1), `facing`, `face.w/top/chin/jaw`, `ear`, `undercut` | face turn, mirror, head size and jaw, ears, stubble tone |
| Eyes | `eye.a/b/gap/y/tilt/open/lash/flick/iris/k/look/lower/shadow` | size, spread, tilt, lid, lash weight, iris colour, eyeshadow |
| Brow, nose, mouth | `brow.y/w/len`, `nose.style/y` (none, dot, tick), `mouth.y/w/skew/curve/lip/color/weight` | each Character has its own nose and mouth; `skew` is the smirk |
| Hair | `back.len/w/flare/curl/wave`, `hairline.y/side`, `fringe.n/span/lenl/lenr/part/swl/swr/bend/tipw/pitch/jit/seed/wisps/flow/part_x`, `fringe.locks`, `lock_l`, `lock_r` (`len/w/flick/in/x/wave/taper`), `gloss`, `ring` | back hair, hairline, fringe (auto or a list of clumps), side locks, gloss |
| Clothes | `headwear` (headdress), `garment.kind/color/trim/bow` (maid, gown, jacket), `earring`, `necklace` | clothes and jewellery |
| Expressions | `expr.<name>` | per-Expression overrides of the rules (not used so far) |

A new Character must set (inferred from Louise): `hair`, `eye.iris`, `garment`. Also `skin` if it is not pale. For a different look: `back`, `fringe`, the locks and `mouth`.
Louise is set with 15 lines of JSON; Gabriella needed the hand-listed `fringe.locks` for her sweep.

## Sizes (known)
- Generator: 718 lines, 661 not blank.
- Character files: lisette 698 bytes, gabriella 1,108 bytes, louise 904 bytes.
- SVG per Expression (the whole Portrait each time): Lisette 30.0 to 32.7 KB, Gabriella 20.6 to 23.9 KB, Louise 19.7 to 22.4 KB. The size is large because every clump has its own clip path. The real store is the Character file plus the generator (inferred).

## Render rounds (9 sheets, `out/sheet-r1.png` to `out/sheet-r9.png`; about 13 large single views between them)
| Round | Faults seen | Fixed in |
|---|---|---|
| 1 | Fringe a jagged crown; ring highlight a muddy lens; hair a helmet with plank locks; faces small | r2 |
| 2 | Cap closed the wrong way (hairline path reversed); clump roots poked over the headdress band; clumps overlapped into a flat slab | r3 |
| 3 | Gabriella's sweep not visible; clump edge lines crossed (X shapes); double outline on the crown | r4 |
| 4 | Neutral mouth of Lisette read as a smile; dark horns at the fringe ends (alt-colour clump showed against the cap) | r5 |
| 5 | Flat hair mass, no flow | r6 (flow lines from a part, darker clump tips) |
| 6 | Hard-edged tip shade; square shade on the neck; pink lip drip | r7 |
| 7 | Louise: back hair folded into hanging lines; stubble patch looked dirty (generator fixes) | r8 |
| 8 | Gabriella looked young and innocent | r9 (lower lids, eyeshadow, narrower jaw, stronger smirk) |

## How good it looks (my reading, inferred; the owner decides)
- At 80 px (known, from the dashboard shots): both read as anime chibi Characters. Lisette is clearly the painted one: brown bob, white headdress, blue eyes. Gabriella reads as a blonde with purple eyes, but she looks young and sweet more than cunning. At 80 px the smirk is a small dot.
- Large (known): closer to the chibi style references than round 1. Weak points: the fringe is still a row of regular V cuts for Lisette; the clump lines are thin; skin has one flat tone plus one shade; no nose on Lisette; the far eye of Gabriella is partly under a lock; Louise's black hair loses its structure because the outline has the same colour.
- It is not as good as the reference game: the references have more hair strand detail, bigger white gloss shapes and hand-made irregular shapes (inferred).
- Weakest Expressions: Gabriella neutral and angry at 80 px (the mood sits in a 2 px mouth); Lisette sad at 80 px (tear and wobble mouth are small). Happy and surprised are the strongest for both.
- Two different people? Yes (known): hair, headdress, eye shape and colour, clothes, face turn and mirror all differ. They share the same face outline and eye style, so they look like two Characters of one game, which is the goal.
- Neutral and sad were weak in round 1. Neutral is now a flat line, lowered lid and flat brow for Lisette, and a smirk with lifted lid for Gabriella. Sad now has water in the eye, lifted inner brow, a wobble mouth and a tear.

## Louise (third Character, text and JSON only)
- Cost: 1 parameter file written from the JSON text; 2 render rounds (one large view, one sheet at r7; she was in later sheets without change). About 10 minutes of work (guessed: I did not time it).
- What needed a generator change (known): `hairline.y/side` (undercut sides), `ear` (ears), `undercut` (stubble tone), a fix of the back hair for short hair (point list folded when `back.len` was small). The `jacket` garment (chef) already existed from the pair.
- What did not need a change: tanned skin, yellow eyes, black hair, fringe swept by `fringe.locks`, no side locks (`lock_l` and `lock_r` null), smile mouth.
- Shot: `out/louise/sheet.png`.

## Honest limits
- No owner view yet. The look is my reading against the references.
- One person's eye (mine) judged it, and I looked at each sheet only once, in a viewer that showed me stale images twice until I used new file names (known; that cost me about 3 misread renders).
- Not tested: a Character with a very different hairstyle (ponytail, long fringe over both eyes) or a different head angle. The generator has no ponytail or side-ponytail part (known).
