# Research: how art from code is written

Type: research (AFK)
Status: resolved
Blocked by: none

## Question

What ways exist to write character art as code or data, and which of them can a coding agent use well at Portrait size?

## Context

- Start from [tibo.dog findings](../assets/tibo-dog/findings.md). It already covers the tibo.dog build output and the Frostward devlog. Do not repeat that work.
- Cover these formats: text grids with a palette, drawing code (shapes and pixels), layered parts with anchors (the tibo.dog body and arm), whole frames for each Expression, and vector shapes such as SVG.
- Cover converters from an editor (Aseprite, a PNG) to data, and the reverse direction, so that a human can edit the art in a normal tool.
- Find what size and colour count a coding agent can draw well in each format, with evidence: published examples, or a small test of your own in `tmp/`. A face must show an Expression.
- Note how each format fits Chronicler: the art lives in `data/` next to the World, the server is Rust, and the screen is HTML with no JavaScript framework (map, "What already exists").
- Use `/research`. Prefer primary sources.

Done when a summary is linked as an asset of this ticket, with a table of the formats (authoring cost, agent quality, size limit, tooling, fit with Chronicler), and the answer gives the 2–3 formats that the style prototype should try.

## Answer

The style prototype should try three formats. Each has one Expression drawn for the same Character, in the real dashboard.

1. **Layered text grid with a palette** (24×24 or 32×32): one base grid, and one small face patch for each Expression. Why: an agent drew five readable Expressions this way. A patch is about 6 rows, so a new Expression is cheap. It converts without loss to an indexed PNG (a pixel editor opens it) and to inline SVG rects. It is an array of strings in JSON, so it fits `data/` and the schema checks.
2. **SVG vector shapes** (64×64 viewBox): the clearest Expressions on the first try, at 32 to 192 px, and no size limit. The server needs no new crate, and `assets/index.html` already uses inline SVG. The look is smooth, not pixel art. This is the test of whether the owner wants pixel art at all.
3. **Whole-frame text grid** (32×32, one grid for each Expression): the tibo.dog method. It reads well and is the simplest rule, but each Expression repeats the whole picture. Try it as the baseline against format 1.

Not chosen: drawing code (the agent gave spatial faults, and code in `data/` is a larger trust surface), layered parts with anchors (not tested, no fit for faces), and Aseprite files as the source (binary, and an agent cannot draw in an editor). Aseprite can still be the human's editing tool through the indexed PNG.

Findings: [authoring-art-from-code/findings.md](../assets/authoring-art-from-code/findings.md).

### Surfaced

- **Prototype must test a real Character.** All tests drew one generic figure. Check 2 of the map (an agent writes one Character from its description) is still untested.
- **The agent needs a render tool.** Quality came from looking at a render and a row-length check. The spec should name a script that renders a Portrait to PNG, and a length check in `scripts/validate_data.py`.
- **Palette rule.** Two palette keys with one colour broke the RGBA converter. The spec needs "one colour for each key", or converters go through an indexed PNG.
- **One render path.** A grid converts to inline SVG rects. So pixel art and vector art can share one render path in the browser. This changes the "Not yet specified" item on the build step. Candidate fog: is a baked PNG atlas still needed?
- **Fog for the owner.** Which editor does the owner use (Aseprite may not be bundled, by its EULA)? Are grids above 32×32, or 100-colour sprites like tibo.dog, wanted? Both are untested.
- **Aseprite CLI, LibreSprite and Pixelorama were not run.** Only their docs were read.
