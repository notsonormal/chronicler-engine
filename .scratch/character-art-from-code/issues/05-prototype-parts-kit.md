# Prototype a parts kit with a recipe

Type: prototype (HITL)
Status: resolved
Blocked by: 04

## Question

Can Portraits put together from a kit of parts, with one recipe for each Character, reach the reference look? And what does the recipe hold?

## Context

- This is how the reference game builds its art (owner). The sample in `tmp/maid/` shows the method (known, from the files): parts in 300×300 cells on one shared canvas, a layer order in the file name number, and for most parts a pure red, green and blue colour mask next to a line-and-shade cell. So a colour is chosen per part when the parts are put together (inferred). The brow file holds a grid of brow variants.
- The sample has no eyes and no mouths, and its source and licence are not known. Use it only inside this prototype, and never copy it out of `tmp/`. Add stand-in eyes and mouths. Their quality is not the question here: [Prototype parts drawn by an agent](07-prototype-agent-drawn-parts.md) tests that.
- What this ticket tests: the composition. That is the layer order, the shared canvas, the recolour from a mask, an Expression as a swap of eye, brow and mouth parts, and a recipe file in `data/` that names parts and colours.
- Ticket 01 rejected "layered parts with anchors" for faces without a test. This ticket is that test, with a shared canvas instead of anchors.
- Write down the smallest kit that covers two Characters with five Expressions each: how many parts, and which.
- Run [the shared test of the approaches](../map.md) (map Notes), so the four prototypes compare.
- Throwaway: files under `tmp/`, no change to `src/`.

Done when the owner has seen the two Characters in the strip and the answer records the recipe shape, the part count and the owner's verdict.

## Answer

**The composition works. The parts must come from code, not from an outside kit (owner, 2026-11-02).**

- **Composition (known).** One shared 300×300 canvas, a layer number for each part, a red/green/blue mask recolour and an Expression as a swap of eyes, brows, mouth and mark. All of it worked with no change. Brown, blonde and black hair and fair and tan skin recolour from one base colour each.
- **The recipe (known).** 19–27 lines of hand-written JSON: a palette, the fixed parts with their colours, one colour for each face slot, and an Expression table. The Expression table was the same for every Character, so a default table lives in the part catalogue.
- **Part count (known).** 29 parts for two Characters × five Expressions: 10 body parts, 5 eyes, 5 mouths, 5 brows and 4 marks. A third Character (Louise) added 2 parts, about 10 s of tool time and 2 compose runs.
- **Code-drawn parts (known).** 13 of the 29 parts were drawn by code (eyes, mouths, marks). They recoloured and composed like the sample parts.
- **Faults of the method (known).** The canvas fixes the face position, so all Characters look like siblings. Brows clash with the fringe. Headwear is one shape at one place. The sample blouses have no mask, so clothes do not recolour.
- **Owner's verdict.** The owner wants a kit because a kit may make a decent style easier to reach, not to use outside art. An outside kit is ruled out. The next test draws every part with code, in the style of the `red` emote set of pi-emote (pixel art, three-quarter bust, many shade tones).

Findings and shots: [parts-kit/findings.md](../assets/parts-kit/findings.md). The shots stay in `tmp/character-art/kit/shots/`, because they contain the sample parts.

### Surfaced

- [Prototype parts drawn by an agent](07-prototype-agent-drawn-parts.md) now aims at the `red` style and uses no sample parts.
- The fog item "The source and licence of a kit" is out of scope.
