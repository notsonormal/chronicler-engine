# Map: Character art from code

Labels: wayfinder:map

## Destination

A spec that an implementer can build from: art from code for the Characters and the Persona. It covers how the art is written, how it gets to the screen, and how the story selects an Expression. The map can also end early as a recorded "no", if a prototype shows that the look is wrong for Chronicler.

## Notes

- **Origin (owner, 2026-11-02).** A Grok conversation about @poteto's tibo.dog: "all the pixel art is code. every sprite is a file drawn with one shared palette and packed into a sprite atlas at build time." What the shipped game shows: [tibo.dog findings](assets/tibo-dog/findings.md).
- **Goals, ranked (owner, charting).** 1. Expressions that follow the story. 2. Animation, such as a blink or an idle loop. 3. New Characters without an image model: an author or a coding agent writes the art as data. 4. Clothes. Small file size is not a goal, because the server runs locally.
- **Subjects (owner, charting).** The Characters and the Persona. The Persona has no picture today (`data/personas/*.json`).
- **Style is open (owner, charting).** Art from code is the goal. Pixel art is one candidate, not a fixed choice. A prototype ticket decides the style.
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

## Not yet specified

- **The data format and the build step.** How a Portrait is written in `data/` (grids, drawing code, layers, whole frames), where the palette lives, and what bakes it: a `build.py` step, a server start step, or a render for each request. Inline SVG and a baked PNG atlas are both candidates. Depends on the research on authoring and on the chosen style.
- **The Expression list.** One fixed list for every Character, or a list that each Character defines. Probably settled together with what chooses the Expression.
- **The Persona's place on the screen.** Where the Persona's Portrait appears, matched with [Decide what the visual sidebar shows](../dashboard-design/issues/08-the-visual-sidebar.md).
- **Matching the Dashboard design direction.** After both maps choose, check that the chosen art style fits the chosen direction of the Game tab, and decide what changes if it does not.
- **Animation.** Which motions (blink, idle, a change between Expressions), and how they run without a JavaScript framework: CSS frame steps, an SVG animation, or a small script.
- **Clothes (owner, charting: fog).** Whether outfits are in the spec at all. Decide after the data format is chosen.
- **The painted pictures.** Whether art from code replaces `profile_image` and `headshot_image`, sits next to them as a second kind of Portrait, or falls back to them.
- **The shape of the spec.** Where it lives (`docs/specs/`, an ADR, the data schema in `data/schemas/`) and which tests it names.

## Out of scope

- **Room pictures.** Only the Characters and the Persona get art from code (owner, charting).
- **Art made during play.** The engine does not make art for a Character that a Game adds while it runs (owner, charting).
- **Building the feature.** This map ends with the spec.
