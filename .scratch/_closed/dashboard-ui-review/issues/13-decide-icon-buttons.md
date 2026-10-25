# Decide the icon-button approach

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

Icon-only buttons (`✎`, `🗑`, `↻`, `✓`, `▶`) use font glyphs. These can render as empty boxes when the font is missing. The buttons have `title` but no `aria-label`. `▶` means both "next swipe" and "new swipe" (Retry). Which approach should they use: glyphs with a bundled fallback font, inline SVG icons, or text labels? And how do "next swipe" and "new swipe" become distinct?

## Context

- Finding 3.3. Screenshots 01, 16.
- Adding `aria-label` is clearly needed and needs no decision. Do it in the implementation ticket.
- The box rendering was seen in headless Chrome. It may not happen on the user's desktop.

## Done when

- The decision is in the ticket answer. Implementation tickets are created.

## Answer

The dashboard uses inline SVG icons, delivered as one `<symbol>` sprite in
`assets/index.html` and referenced per control with `<use href="#i-…">`, tinted
with `currentColor`. Icon paths come from a permissive 24×24 stroke set
(Lucide/Feather style, ~2px stroke). No new dependency and no network fetch. A
bundled fallback font and text labels were both rejected: the font adds a
dependency and a license question for no gain, and text labels widen the dense
per-entry control cluster.

Every glyph control is replaced, not only the ones that rendered as boxes in
the review environment, because the box rendering is font-dependent and a mixed
cluster looks inconsistent at 24px. The controls are: story-log Edit, Delete,
Check and Retrigger; the swipe `◀`/`▶`; edit-mode save/cancel; the text-check
dismiss; the options-dock regenerate and edit buttons; and the Games reset.

`▶` stays a single control. It keeps its position in the swipe row, and its
glyph and `aria-label` change with state: a navigation arrow when
`next_swipe_index` is set, a "new swipe" glyph otherwise (`POST /swipe/new`,
labelled "Retry" today). Navigation and generation therefore stay on one
control, distinguished by icon and label rather than by a second button.

Every icon-only button carries an `aria-label`; the `<svg>` carries
`aria-hidden="true"` so screen readers announce the button, not the graphic.
The existing `title` attributes stay as tooltips.

Token names are not involved. The sprite is defined in the shell, which the
stub server serves verbatim (`tests/test_utils/stub_server.rs`
`DASHBOARD_SHELL`), so stub-browser tests see it.

### Graduated

[Replace the glyph controls with an SVG icon sprite](55-replace-glyphs-with-svg-sprite.md).
