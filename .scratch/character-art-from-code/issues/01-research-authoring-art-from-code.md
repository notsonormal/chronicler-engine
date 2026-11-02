# Research: how art from code is written

Type: research (AFK)
Status: open
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
