# Research: art from code with a fresh view

Type: research (AFK)
Status: resolved
Blocked by: none

## Question

Which methods do people use today to make character art from code, what range of looks do those methods reach, and which of them can give anime-style faces that look somewhat like the art in `data/images/`, with different faces, clothes and Expressions?

## Context

- **Why this ticket (owner, 2026-11-02).** Art from code should give a wide variety of looks, but this map is stuck in one place. Many people make art from code today, and not all of it looks like the attempts of this map. A fresh view may find a better route.
- **Start outside this project.** Survey what exists first. Read the findings of this map only at the end, to mark what was already tried. Do not start from the prototypes.
- **What was tried (read at the end).** Six rounds in [Prototype the art style](04-prototype-the-art-style.md) and [Prototype a face generator](06-prototype-face-generator.md) used one method: a model wrote every shape, as a pixel grid, as SVG or as a parametric generator. The looks were chibi, flat cel and shaded pixel art. The owner found all of them too ugly and too far from the painted art. [Prototype a parts kit with a recipe](05-prototype-parts-kit.md) showed that composing parts on a shared canvas with a recolour works. [Prototype parts drawn by an agent](07-prototype-agent-drawn-parts.md) and [Prototype a conversion of the painted pictures](08-prototype-convert-painted-pictures.md) run in parallel.
- **The target (map Notes, The target style).** Anime-style faces that look somewhat like `data/images/lisette_headshot.png` and `data/images/gabriella_headshot.png`. The format, resolution and shading are free.
- **Places to start, not a list to finish.** Parametric avatar libraries. Character makers with layered parts. Layered sprites in visual-novel engines. 2D rigs with parameters (Live2D, Inochi2D, Spine). 3D anime models with toon shading (VRM, VRoid), rendered live or baked to images. Procedural character generators in games. Shader-based or distance-field faces. What models that write SVG or drawing code reach today. Hybrids where an image model makes a kit once and code composes it. Find at least two families that are not in this list.
- **For each family, record:**
  - shipped examples, with links and a short description of the look
  - how a new Character, an outfit and an Expression are made, and by whom (a human, a coding agent, a tool)
  - the tools and their licences
  - whether it needs an image model, and when
  - how it fits Chronicler: a Rust server with Askama and HTMX, and no JavaScript framework. The art is made when the World is written. A build step or a bake to images is allowed. File size does not matter.
  - an honest reading of how close the look can get to the painted pictures
- **Judge each family by the ranked goals of the map Notes:** Expressions, animation, new Characters without an image model, clothes. Name a conflict with a goal. Do not drop a family because of a conflict: the owner decides.
- **Third-party pictures.** Link them, or keep shots in the git-ignored `tmp/`. Do not copy them into `.scratch/`.
- Use `/research`. Label each claim known, inferred or guessed.

Done when `assets/fresh-view/findings.md` covers at least six families with examples, and names two or three of them for a prototype, each with the question its prototype would test.

## Answer

**No surveyed family gives the painted look, new Characters with no image model, Expressions and clothes together (inferred).** The painted look needs pixels from an artist or an image model. Code-only routes (SVG, parametric, parts) stay far below it. Full survey of 11 families, three of them not in the start list: [fresh-view findings](../assets/fresh-view/findings.md).

- **Close to the look (inferred).** The routes that come close keep the painted picture as the pixel source and move it with data. A single-image neural animator (THA3) and a layer split plus a 2D rig (See-through, then Stretchy Studio) each write an Expression as a list of numbers. Both need a GPU at bake time. Neither gives cheap clothes.
- **Parts for everything (inferred).** A VRM 3D toon model is the only route where Characters, Expressions and clothes are all parts. Its look is medium, and today a human with a GUI makes each Character.
- **Open for the owner.** Goal 3 can mean "no image model at all" or "no image model for each Expression or outfit". Under the second meaning, routes A and B fit.
- **Licences (known, findings).** THA4 weights are CC BY-NC. THA3 weights are CC BY 4.0. The licence of the See-through weights is not checked.
- **Limits (known).** The researcher ran no tool and made no shots. Every statement about quality on the two headshots is inferred. The sandbox has no Nvidia GPU.

### Surfaced

- [What "new Characters without an image model" means](11-grill-what-goal-3-means.md)
- [Prototype a neural animator on the headshots](12-prototype-neural-animator.md) (route A), blocked by 11
- [Prototype a layer split and a rig](13-prototype-layer-split-and-rig.md) (route B), blocked by 11
- [Prototype a baked VRM toon model](14-prototype-baked-vrm.md) (route C)
- All four now also block [Choose the Portrait approach](09-grill-choose-the-approach.md).
