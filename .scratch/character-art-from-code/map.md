# Map: Character art from code

Labels: wayfinder:map

## Destination

A spec that an implementer can build from: art from code for the Characters and the Persona. It covers how the art is written, how it gets to the screen, and how the story selects an Expression. The map can also end early as a recorded "no", if a prototype shows that the look is wrong for Chronicler.

## Notes

- **Origin (owner, 2026-11-02).** A Grok conversation about @poteto's tibo.dog: "all the pixel art is code. every sprite is a file drawn with one shared palette and packed into a sprite atlas at build time." What the shipped game shows: [tibo.dog findings](assets/tibo-dog/findings.md).
- **Goals, ranked (owner, charting).** 1. Expressions that follow the story. 2. Animation, such as a blink or an idle loop. 3. New Characters without an image model: an author or a coding agent writes the art as data. 4. Clothes. Small file size is not a goal, because the server runs locally.
- **Subjects (owner, charting).** The Characters and the Persona. The Persona has no picture today (`data/personas/*.json`).
- **The target style (owner, 2026-11-02, second update).** Anime-style faces that look somewhat like the Character art in `data/images/`. Code makes different faces, clothes and Expressions. Chibi is not the target: the chibi rounds moved little toward the existing art. The format, the resolution and the shading are free. The chibi references (`tmp/character-art/reference/style-ref-*.jpg`) are no longer the target. The `red` set of pi-emote (`tmp/character-art/reference/pi-emote-red/`) is the example that [Prototype parts drawn by an agent](issues/07-prototype-agent-drawn-parts.md) aims at (owner, in ticket 05). A brief gives the goal and the faults to fix, not hard rules (owner: the rules of [Prototype the art style](issues/04-prototype-the-art-style.md) stopped the art from improving).
- **The shared test of the approaches (coordinator, 2026-11-02).** Each approach prototype makes Lisette and Gabriella, five Expressions each (neutral, happy, angry, surprised, sad). It shows them in the portrait strip at 80 px with `tmp/character-art/dash_shot.py`, next to the painted pictures. It records the cost of a third Character. The tools of ticket 04 are in `tmp/character-art/`.
- **When the art is made (owner, charting).** When the World is written, by an author or a coding agent, as data in `data/`. Art made during play is out of scope.
- **Parallel with Dashboard design (owner, charting).** The open map [Dashboard design](../dashboard-design/map.md) has two tickets that touch this map: [Choose the direction of the Game tab](../dashboard-design/issues/03-choose-the-direction.md) and [Decide what the visual sidebar shows](../dashboard-design/issues/08-the-visual-sidebar.md). The two maps run in parallel. A style prototype here is shown inside the current dashboard, and the two choices are matched later (see **Not yet specified**).
- **The outside check (owner, charting).** Two checks. (1) The owner chooses from 2–3 versions shown in the real dashboard with a real story. (2) An agent that sees only the finished spec and one Character description writes that Character's art and Expressions, and the owner accepts the result. Check 2 tests goal 3.
- **Names (owner, charting).** "Portrait" (the picture of a Character or of the Persona that the screen shows) and "Expression" (one named look of a Portrait that the story selects) are working names. No `CONTEXT.md` change for now.
- **Plan only.** The map ends with the spec. Building the feature is a later effort.
- **Probe server (known).** `/chronicler-ui-investigator`. The run command, the probe DB and its fixture are in the Notes of [Dashboard design](../dashboard-design/map.md). A prototype session never sends an Action: a turn costs money and changes the Game.
- **Skills.** `/research`, `/prototype`, `/grilling`, `/domain-modeling`, `/chronicler-ui-investigator`, `/codebase-design`.

## What already exists

- **Character pictures (known).** `Character` has `profile_image` and `headshot_image` (`src/domain/model/character.rs`), and `preferred_image()` picks the headshot first. The pictures are painted PNGs in `data/images/`: about 1224×2048 (profile) and 1224×819 (headshot). With the Room pictures they use 25 MB.
- **The portrait strip (known).** The visual sidebar template in `src/adapters/driving/http/templates.rs` renders one `<img>` for each Character present, from `CharacterPortraitView` (`src/adapters/driving/http/view_models.rs`). The browser tier checks that a portrait is between 50 and 120 px wide (`tests/browser/stub/layout.rs`).
- **The Quantifier result (known).** `QuantifierResult` (`src/domain/model/quantifier.rs`) has `present` and `destination` only. After each generation it records the Characters present in the Swipe state.
- **Rendering (known).** The server renders HTML fragments with Askama, and HTMX loads them. There is no JavaScript framework (`PRODUCT.md`).

## Decisions so far

<!-- the index — one line per closed ticket: enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [Research: how art from code is written](issues/01-research-authoring-art-from-code.md) — the style prototype tries three formats: a layered text grid (one base and one face patch for each Expression, 24×24 or 32×32), SVG vector shapes (64×64 viewBox), and a whole-frame grid as the baseline. Drawing code and Aseprite files as the source are rejected.
- [Research: how other apps choose an Expression](issues/02-research-expression-selection.md) — four methods: local classifier, LLM call on the reply, narrator-written tag, word list. All repair a wrong answer in layers, then fall back. A field in the Quantifier JSON adds no model call (inferred). The narrator tag shares the Narration call. The local classifier does not fit Chronicler.

- [Prototype the art style](issues/04-prototype-the-art-style.md) — an agent that draws each whole Portrait does not reach the owner's chibi reference look (three rounds, nine versions). Rejected. Four other approaches go to prototypes. Expressions read at 80 px in every round.
- [Prototype a parts kit with a recipe](issues/05-prototype-parts-kit.md) — the composition works: a shared 300×300 canvas, a layer number per part, a mask recolour and an Expression as a swap of face parts. A recipe is about 20 lines of JSON; 29 parts cover two Characters × five Expressions. The owner rules out an outside kit: code must draw the parts.
- [Prototype a face generator](issues/06-prototype-face-generator.md) — one generator with a small parameter file for each Character works technically: a fix reaches every Character, and a third Character costs one file and 1–7 renders. But in two rounds and four looks (chibi, cel, pixel) the owner found the art too ugly, and too far from the painted art. Rejected. Chibi is not the target. A fresh research ticket follows.
- [Research: art from code with a fresh view](issues/10-research-fresh-view.md) — 11 families surveyed. No family gives the painted look, new Characters with no image model, Expressions and clothes together. Code-only routes stay far below the painted look. Three routes go to prototypes: a neural animator (THA3) and a layer split with a rig move the painted pictures with data, and a baked VRM toon model makes everything from parts. The first two depend on what goal 3 means.

## Not yet specified

- **The data format and the build step.** How a Portrait is written in `data/`, where the palette lives, and what bakes it: a `build.py` step, a server start step, or a render for each request. A text grid converts to inline SVG rects, so pixel art and vector art can share one render path. This puts the baked PNG atlas in question. The format must come with a render script for the agent and a check of row length, and each palette key must have its own colour ([findings](assets/authoring-art-from-code/findings.md)). Depends on [Choose the Portrait approach](issues/09-grill-choose-the-approach.md).
- **The editor for a human.** Which tool a human uses to draw or change a part or a Portrait. Pixel art is allowed again (owner, 2026-11-02), so the pixel editors of ticket 01 (Aseprite, LibreSprite, Pixelorama) may matter. Depends on [Choose the Portrait approach](issues/09-grill-choose-the-approach.md).
- **The Expression list.** One fixed list for every Character, or a list that each Character defines. Probably settled together with what chooses the Expression. A list for each Character avoids the SillyTavern problem with global custom labels. A Character with few Expressions needs a fallback rule ([findings](assets/expression-selection/findings.md)).
- **The Persona's place on the screen.** Where the Persona's Portrait appears, matched with [Decide what the visual sidebar shows](../dashboard-design/issues/08-the-visual-sidebar.md).
- **Matching the Dashboard design direction.** After both maps choose, check that the chosen art style fits the chosen direction of the Game tab, and decide what changes if it does not.
- **Animation.** Which motions (blink, idle, a change between Expressions), and how they run without a JavaScript framework: CSS frame steps, an SVG animation, or a small script. One model to look at: pi-emote's `emotes.json` maps each state to frames, with a blink frame for idle and four talk mouths picked by weight (`tmp/character-art/reference/pi-emote-red/emotes.json`).
- **Clothes (owner, charting: fog).** Whether outfits are in the spec at all. Decide after the data format is chosen.
- **The painted pictures.** Whether art from code replaces `profile_image` and `headshot_image`, sits next to them as a second kind of Portrait, or falls back to them.
- **The shape of the spec.** Where it lives (`docs/specs/`, an ADR, the data schema in `data/schemas/`) and which tests it names.

## Out of scope

- **Room pictures.** Only the Characters and the Persona get art from code (owner, charting).
- **Art made during play.** The engine does not make art for a Character that a Game adds while it runs (owner, charting).
- **Building the feature.** This map ends with the spec.
- **An outside parts kit (owner, 2026-11-02).** The owner wanted a kit because it may make a decent style easier to reach, not to use outside art. So the fog item about the source and licence of a kit is gone ([Prototype a parts kit with a recipe](issues/05-prototype-parts-kit.md)).
- **Full-body movement, hands and props (owner, 2026-11-02).** Full-body movement would be nice, but the owner does not want to grow the scope yet. Poses such as a wave, a hand on the head or a held tablet (as in the pi-emote `red` set) are out too. A later effort can add them.
