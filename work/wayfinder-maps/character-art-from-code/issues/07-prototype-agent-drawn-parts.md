# Prototype parts drawn by an agent

Type: prototype (HITL)
Status: claimed
Blocked by: 05

## Question

Can an agent draw the parts of a kit (hair, fringe, eyes, brows, mouths, headwear, collar, marks) one at a time, well enough that the owner keeps them, so that the project has a kit with no outside art?

## Context

- This gives the kit for [Prototype a parts kit with a recipe](05-prototype-parts-kit.md) without a bought or licensed kit. Use the canvas, the layer order and the recipe that ticket 05 records.
- The owner reviews the parts and keeps the good ones. A rejected part is drawn again. Record how many parts the owner keeps on the first, second and third try.
- Code draws every part. Use no part of the sample in `tmp/maid/`: an outside kit is out of scope (owner, ticket 05). Ticket 05 already drew eyes, mouths and marks with code (`tmp/character-art/kit/standins.py`); the composer and the recipe are in `tmp/character-art/kit/`.
- **The example to aim at (owner, 2026-11-02): the `red` set of pi-emote** (`tmp/character-art/reference/pi-emote-red/`): shaded pixel art, a three-quarter bust, normal proportions. Its frames come from an image model (inferred, [parts-kit findings](../assets/parts-kit/findings.md)), so an agent may not reach the same finish. Hands and props are out of scope (map).
- The three-tone mask of ticket 05 cannot carry many shade tones. One way: a grey value layer with a colour ramp for each palette slot. The format is open.
- One part is a much smaller task than a whole face (inferred).
- Aim for parts that look good, in an anime style. The format, the resolution and the shading are free. Read **The target style** in the map Notes: the earlier rules of ticket 04 are not hard rules.
- Run [the shared test of the approaches](../map.md) (map Notes), so the four prototypes compare.
- Throwaway: files under `tmp/`, no change to `src/`.

Done when the owner has seen the two Characters in the strip, built only from parts that the agent drew, and the answer records the keep rate and the owner's verdict.

## Progress (2026-11-03)

Try 1 runs: one `strong-implementer` writes a style sheet, a composer and each part of the kit, one part at a time, for Lisette and Gabriella. Brief: `tmp/character-art/agentparts/brief.md`. Output: `tmp/character-art/agentparts/`. The owner reviews `review/parts-try1.png` and answers by part id (keep or redraw). Try 2 redraws the rejected parts. Try 3 runs only if parts are still rejected. Louise (the cost of a third Character, from her JSON only) comes after the owner accepts the canvas (changed after the try 1 review).

Try 1 is done (13.6 min, 12 looks, agent session `46ae7e73`). An 80×80 canvas shown 1:1 in the strip, with `image-rendering: pixelated`. A part is an 80×80 index PNG (slot and shade 0–5), written by its own Python script. A recipe turns one colour per slot into a six-tone ramp. 35 parts: 14 shared, 9 for Lisette only, 12 for Gabriella only. Agent notes: `tmp/character-art/agentparts/notes.md`. Waiting for the owner's review of `review/parts-try1.png`.

Owner review of try 1 (2026-11-03): "The eyes are bad in general. I think jewels in gabriella's hair don't work. I'm wondering if increasing the size of the portrait and then sizing it down would help? Gabriella face looks a bit like a bat somehow? Face shape maybe? Lisette has too strong a chin. Listette's maid dress doesn't really work well."

Coordinator's reading by part id (the owner named no ids):
- Rejected (12): all eyes P13–P19, both bases P01 (Lisette's chin) and P02 (Gabriella's face shape), the maid dress P09, and the hair comb P08 and the earrings P12 (both sit in Gabriella's hair).
- Kept (23): the parts the owner did not name. Assumption: no comment means keep.
- First-try keep rate: 23 of 35 (66 %), if the assumption holds.
- The owner's size question: try 2 draws at 160×160 and shows the drawing in the 80 px box. The kept parts are ported to the new canvas with the same design. Brief: `tmp/character-art/agentparts/brief-try2.md`.

Try 2 is done (17.0 min, about 17 looks, a fresh agent, session `8488b477`). Output: `tmp/character-art/agentparts/try2/`.
- Redrawn (10): seven eyes, two bases and the maid dress (three versions). Dropped: P08, P12. Ported (23): spline parts by a scale factor with one script edit each; the mouths and marks were text grids and had to be drawn again at 160 with the same design.
- The size question (agent, known from its shots): 160 shown at 80 helps the eyes and the small curves. At device scale factor 1 the Portrait is soft, not crisp pixel art. At factor 2 it is sharp. The hair and the clothes gain nothing from the port: they need new detail, not more pixels. Part of the eye gain comes from the new eye design, not from the canvas (not separated).
- `STYLE.md` and the try 1 files were enough for a fresh agent to continue the kit. The try 1 eye guidance in `STYLE.md` caused the dark ring, and the fresh agent fixed it there.
- Waiting for the owner's review of `try2/review/parts-try2.png`.
