# Authoring art from code: what the formats are and what an agent can draw

Collected on 2026-11-02. Builds on the [tibo.dog findings](../tibo-dog/findings.md). That file is not repeated here.

Labels: **known** = read in a primary source or seen in a test run here. **inferred** = a conclusion from known facts. **guessed** = no evidence.

## Short answer

- A coding agent can draw a readable face with an Expression in three formats: a text grid with a palette (24×24 and 32×32 tested), drawing code with shapes (48×48 tested), and SVG vector shapes (64×64 viewBox tested).
- The agent drew best when it could look at a render of its own output and fix it. See "Agent quality" below.
- A text grid converts without loss to an indexed PNG and to inline SVG. So an editor, a `data/` file and the browser can all use one data form.

## The tests (known)

One agent (the model that wrote this file) drew a generic head-and-shoulders figure with five Expressions: neutral, happy, angry, surprised and blink. Each test used a fixed palette of about 12 colours. The agent saw a PNG render of each result with the `read` tool, and could fix it. The scripts are in [`tests/`](tests/). Limits of the test: one model, one generic figure, no human rating, no test of a given Character description.

| Test | What was drawn | Result |
|---|---|---|
| A. Layered text grid, 24×24 | One base grid and one small face patch for each Expression. [Result](grid24-expressions.png). [Same size as on screen, ×3](grid24-actual-size-x3.png). | All five Expressions read at once. Two faults in the first try (see below). |
| B. Whole-frame text grid, 32×32 | One complete grid for each Expression. [Result](grid32-neutral-happy.png). [Text of the neutral frame](tests/grid32_neutral.txt). | Reads well. Eyes have an iris, there is a nose and shading. |
| C. Drawing code, 48×48 | Rectangles, ellipses, polygons and lines on an indexed canvas. [Result](prims48-expressions.png). | Expressions read. The fringe still hides the brows after one fix. |
| D. SVG, 64×64 viewBox | Curves and ellipses, one group for each part. [Result at 32, 64 and 192 px, and the grid as SVG](svg-and-grid-svg.png). | Best first try. The Expressions are the clearest. No fix was needed. |
| E. Converter round trip | Grid text → indexed PNG → grid text. | Identical, with a fixed palette order. See "Converters". |

### Faults seen (known)

- **Test A, first try** ([image](grid24-first-try.png)). The "happy" mouth was drawn one row below the patch, so it did not show. The blush was on one cheek only. The agent saw both faults only in the render. The grid text gave no sign of them.
- **Test B.** Two rows had the wrong number of characters. A length check in the script caught them at once. Without such a check, a short row shifts the picture.
- **Test C.** The agent put the hair too low, so the brows hid in it. One edit did not fully fix it.
- **Test D.** No fault found. Curves have no row or column to miscount.

## What other people report

- **Asset Forge** ([project page](https://matthewdeaves.github.io/assetforge/)). A hobby project. Claude Sonnet draws with commands (`rect`, `circle`, `ellipse`, `line`, `polygon`, `fill`) on a 32–64 px canvas with a fixed palette. A server rasterizes the commands. The page says that raw pixel arrays "produce incoherent results", and that shapes work better. The page gives its own scores (an LLM judge checked against human grades): 4.0 to 4.5 of 5 for 32×32 and 48×48 sprites, including a front-facing farmer and a wizard. These are the project's own numbers. **Known** that the page says this. **Not checked** by me. My tests A and B do not agree with "incoherent" for 24–32 px, with a base and patch structure and a render to look at. The difference may come from the model, the subject, or the render loop.
- **Pixel Art Bench** ([page](https://www.emergentmind.com/pixel-art-bench)). Models get a 24×24 canvas, a palette of 2 to 10 colours, and answer with a JSON grid of palette digits. The page shows a Mario example from one model. **Inferred:** 24×24 with at most 10 colours is a size the benchmark authors think models can do. I did not find a published score table.
- **Frostward** (from the [earlier findings](../tibo-dog/findings.md)). Grids work for 16×16 to 32×32 with 4 to 10 colours. At 64×64 with gradients a person can no longer read the grid.

## Table of formats

"Agent quality" is from the tests above unless a source is named.

| Format | Authoring cost | Agent quality | Size limit | Tooling and editor round trip | Fit with Chronicler |
|---|---|---|---|---|---|
| **Text grid + palette** (one frame) | Low to type, medium to fix. One character for each pixel. A short row shifts the picture. | Good at 24×24 and 32×32 (tests A, B). Needs a length check and a render. | Frostward: 16–32 px, 4–10 colours. Tests here: 32 px, 12 colours. Not tested above 32. | Round trip to an indexed PNG is lossless (test E). Any pixel editor opens the PNG. | Good. A grid is an array of strings in JSON. It is readable in a diff. It can be checked with `data/schemas/`. The server can turn it into inline SVG with no new crate (inferred). |
| **Text grid, layered** (base + patch for each Expression, patch has an offset) | Lowest cost for Expressions. A patch is about 6 rows. One base serves every Expression. | Good (test A). The first try had two faults in the patches. | Same as a grid. | Same as a grid. Editor tools do not know "patch": layers map to patches only through an agreed layer-name rule. | Best match for goal 1 (Expressions follow the story) and goal 4 (clothes are one more layer). Palette and base are shared. |
| **Whole frame for each Expression** (tibo.dog) | High. Every Expression repeats the whole picture. 5 Expressions = 5 grids. | Good at 32×32 (test B). | Same as a grid. | Aseprite frames and tags are made for this. | Simple, but the cost for each new Expression is the whole picture. Same Expression list for every Character is easy to check. |
| **Layered parts with anchors** (tibo.dog body and arm) | Medium. Needs an anchor point for each part and a join rule. | Not tested. **Guessed:** the agent can place parts, but alignment faults are likely without a render. | Parts follow the grid limits. | Aseprite `--split-layers` exports each layer. Slices carry named regions with bounds. | Heavier than the patch idea. Fits clothes and held items. The tibo.dog hero uses it for a swinging arm, not for faces. |
| **Drawing code** (shapes and pixels on a canvas) | Medium. Short code for big areas. Hard to give fine detail. | Medium (test C). Shapes are good for large forms. Spatial faults appear (hair over brows). Asset Forge reports the same method as the best of its tests. | 32–64 px (Asset Forge). 48 px tested here. | No editor round trip. Output is a grid or PNG. Code is the source. | Needs a place to run code. The server is Rust and the repo scripts are Python (known). A drawing script is code in `data/`: a larger trust and test surface (inferred). Not a fit for an author who is not a coder. |
| **SVG vector shapes** | Medium. Curves and coordinates. A render is needed to check. | Best first try (test D). Expressions are clear at 32, 64 and 192 px. | None. It scales. | Any editor that reads SVG can open it (**not tested**). SVG text is easy to diff. | Strong. The server sends HTML, and `assets/index.html` already has inline `<svg>` with `<use href="#i-…">` and a CSS spin class (known). No new crate. The look is smooth, not pixel art. |
| **Grid → SVG rects** (a render target, not a way to write) | None. A script makes it. | Not applicable. | A 24×24 Expression gave about 100 rects and 5–6 KB of text (test D). | Lossless from the grid (known). `shape-rendering: crispEdges` keeps the edges sharp ([MDN](https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Attribute/shape-rendering)). | Lets pixel art use inline SVG, so one render path serves both styles. Size is not a goal for a local server. |
| **Editor file** (Aseprite `.ase`, PNG) | Lowest for a human. Highest for an agent: it cannot draw in an editor. | Not applicable. The agent edits text, not an editor file. | Any. | See the next section. | Binary files in `data/`. A build step must read them. |

## Converters and editors (known unless marked)

- **Aseprite CLI** ([docs](https://github.com/aseprite/docs/blob/main/cli.md)). `aseprite -b file.ase --sheet sheet.png --data sheet.json` writes a sprite sheet and a JSON file. `--format json-array` or `json-hash` picks the JSON shape. `--split-layers` makes one item for each layer. `--list-layers`, `--list-tags` and `--list-slices` add layers, tags and slices to the `meta` part of the JSON. `--palette` swaps the palette. `--sheet-pack` packs many files into one atlas. This is the same kind of output as the tibo.dog manifest (rectangles for each frame).
- **Aseprite file format.** The binary `.ase` format has a public [specification](https://github.com/aseprite/aseprite/blob/main/docs/ase-file-specs.md): layers, cels, tags, palette. The Rust crate [`asefile`](https://docs.rs/asefile/latest/asefile/) (MIT, version 0.3.8) reads it, including layers, and returns `image::RgbaImage`. The repo `Cargo.toml` has no `image` crate (known). So reading `.ase` in the server adds two crates. **Inferred:** a build-time script is cheaper than a server dependency.
- **Aseprite licence.** The [EULA](https://github.com/aseprite/aseprite/blob/main/EULA.txt) says you may not distribute copies of the software. You may compile and change the source for personal use. A World author needs the editor, not a copy from Chronicler. **Inferred:** Chronicler must not bundle Aseprite. I did not run Aseprite. No Aseprite command was tested here.
- **Free editors.** [LibreSprite](https://github.com/LibreSprite/LibreSprite/) is a GPL v2 fork of the last GPL v2 Aseprite commit. [Pixelorama](https://pixelorama.org/faq/) is MIT. I did not test their export. **Guessed:** both write PNG.
- **Grid ⇄ indexed PNG** (test E). Grid text → indexed PNG → grid text was identical, in 956 bytes of PNG against 600 bytes of text. A person can open the PNG in a pixel editor. This works only if the palette order is fixed.
- **Grid ⇄ RGBA PNG** (test E). Matching by colour failed in my test. Two palette keys (`o` outline and `e` pupil) had the same colour, so the key was lost. **Inferred:** the spec must say that every palette key has its own colour, or that a converter goes through an indexed PNG.
- **Frostward converter.** About 300 lines turn an Aseprite PNG into a grid, with a strict `--palette` mode (from the earlier findings).
- **Python tooling in the repo.** The sandbox has Pillow. The repo scripts I checked (`scripts/extract_images.py`) import only the standard library, so a PNG converter in `scripts/` adds a dependency or must read PNG with the standard library. **Inferred.**

## How each format reaches the screen

- **PNG with `image-rendering: pixelated`.** The browser scales by the nearest neighbour to a whole multiple of the size ([MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/image-rendering)). A baked PNG needs a build step.
- **Inline SVG.** No bake step. The server writes the shapes into the HTML fragment. A part (`<g>`) can carry a class that CSS shows or hides. **Inferred.** The repo already puts a CSS class on inline SVG (`icon-spin`, known).
- **Portrait size on screen.** The browser tier accepts 50–120 px wide (known, map). **Inferred:** a 32×32 grid at 3× (96 px) or a 24×24 grid at 4× fits.
- **Animation (input for a later ticket).** `steps()` in CSS jumps between fixed values, so a strip of frames can step without a script ([MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/easing-function/steps)). Group toggling works for inline SVG. I tested neither.

## What I could not settle

- Whether an agent can draw a **given** Character (Carla, with her hair, clothes and mood) so that the owner accepts it. All tests drew one generic figure.
- Agent quality **without** a render to look at. All tests had one.
- Grids larger than 32×32, and the 38×87 px, 100-colour range of the tibo.dog sprites.
- How a different model does. One model was used.
- Editor tools: I ran none of Aseprite, LibreSprite, Pixelorama or an SVG editor.
