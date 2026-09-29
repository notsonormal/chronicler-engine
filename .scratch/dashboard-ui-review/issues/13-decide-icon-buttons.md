# Decide the icon-button approach

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Icon-only buttons (`✎`, `🗑`, `↻`, `✓`, `▶`) use font glyphs. These can render as empty boxes when the font is missing. The buttons have `title` but no `aria-label`. `▶` means both "next swipe" and "new swipe" (Retry). Which approach should they use: glyphs with a bundled fallback font, inline SVG icons, or text labels? And how do "next swipe" and "new swipe" become distinct?

## Context

- Finding 3.3. Screenshots 01, 16.
- Adding `aria-label` is clearly needed and needs no decision. Do it in the implementation ticket.
- The box rendering was seen in headless Chrome. It may not happen on the user's desktop.

## Done when

- The decision is in the ticket answer. Implementation tickets are created.
