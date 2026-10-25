# Apply the chosen palette and fix the colour contrast

Type: task (AFK)
Status: resolved
Blocked by: 56

## Question

Apply the palette chosen in
[Prototype three calmer dark palettes](56-prototype-dark-palettes.md) to the
shipped dashboard, and fix the two contrast findings it lands on. Dialogue
renders in `--color-accent-red` (`rgb(255,68,68)`, 3.6:1 on the narration
bubble) instead of the documented dialogue colour, and the swipe counter and
timestamps use `#888` on `#1a3a3a` at 3.46:1. Every text colour must meet
WCAG AA (4.5:1).

## Context

- Decision: [14](14-decide-palette.md); prototype: [56](56-prototype-dark-palettes.md).
- Absorbs [Fix dialogue colour and small-text contrast](12-fix-dialogue-colour-contrast.md):
  the palette values and the contrast fixes touch the same rules in
  `assets/styles.css`, so they ship as one commit instead of two sessions.
- Update the token values in `assets/styles.css`, plus `assets/games.css` and
  `assets/worlds.css` where they define colour, and the remaining non-token
  hardcoded hexes (`#cccccc` input text, `#00cccc` narration sender, the
  send-button gradient).
- Dialogue must use the dialogue colour, not `--color-accent-red`. This is a
  role-mapping bug in the `.dialogue` rules, not only a token value, so the
  palette pass alone does not fix it.
- Update the token tables in `docs/diataxis/reference/frontend/ui_design.md` to
  match; that file is the authority for token→usage.
- Keep token names hue-based. Rename a token only if its new value no longer
  matches its name.
- Pure CSS and doc values: no new test. Update any existing assertion the change
  breaks.

## Done when

- Every colour token in `assets/styles.css` matches the chosen palette, and the
  `ui_design.md` tables match the shipped values.
- Dialogue uses the documented colour, and every measured text meets at least
  4.5:1.
- `python build.py` is green, the user reviews the diff, then commit through
  `/commit-and-push`.

## Answer

Absorbed into [Visual identity pass](66-visual-identity-pass.md). The palette pass
moved there together with the icon sprite, and carried ticket 12's contrast fixes
with it, so all the `assets/styles.css` value changes ship as one commit. Closed
as absorbed, not fixed.
