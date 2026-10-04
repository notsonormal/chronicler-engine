# Prototype three calmer dark palettes

Type: prototype (HITL)
Status: open
Blocked by: —

## Question

Build a cheap artifact to react to: three candidate dark palettes for the
dashboard, per [Decide whether to keep the neon palette](14-decide-palette.md).
The user picks one.

## Context

- Decision: [14](14-decide-palette.md).
- One self-contained page showing the three palettes over the same
  representative markup: the Game tab's narration, dialogue and system bubbles,
  the status pill, a Settings connection card, and the tab bar. Desktop only;
  no light mode.
- The point of the prototype is the reading experience on long prose. Each
  candidate is a full value pass over the tokens in `assets/styles.css`,
  including bubble backgrounds, borders and the hardcoded button gradients —
  not accents alone.
- Keep the dark base. Keep the token names hue-based unless a value stops
  matching its name.
- Put the artifact under `tmp/ui-review/` and link it from this ticket, not
  from the map.
- Skills: `/prototype`.

## Done when

- The three palettes are rendered and the user has picked one.
- The chosen values are recorded in the answer, or handed to
  [Apply the chosen palette and fix the colour contrast](57-apply-chosen-palette.md).
