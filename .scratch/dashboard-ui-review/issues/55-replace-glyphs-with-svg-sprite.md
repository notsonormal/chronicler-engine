# Replace the glyph controls with an SVG icon sprite

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Replace every font-glyph control in the dashboard with an inline SVG served from
one sprite, per [Decide the icon-button approach](13-decide-icon-buttons.md).

## Context

- Decision: [13](13-decide-icon-buttons.md).
- One `<symbol>` sprite defined early in `<body>` in `assets/index.html`; each
  control renders a small `<svg aria-hidden="true"><use href="#i-…"/></svg>`
  tinted by `currentColor`.
- Paths from a permissive 24×24 stroke set (Lucide/Feather style, ~2px stroke).
  No new dependency, no network fetch.
- Controls to replace:
  - story-log Edit, Delete, Check, Retrigger (`src/adapters/driving/http/templates.rs`,
    `NarrativeLogTemplate`), plus the swipe `◀`/`▶` in the same template;
  - edit-mode save/cancel (`assets/index.html`, `showEditForm`);
  - text-check dismiss (`TextCheckPreviewTemplate`);
  - options-dock regenerate and edit (`OptionsDockTemplate`);
  - Games reset (`games/templates/games.rs`).
- `▶` keeps one control across both states. Its glyph and `aria-label` change
  with `next_swipe_index`: navigation arrow when a next swipe exists, a
  "new swipe" glyph otherwise. Do not split navigation and generation into two
  buttons.
- Every icon-only button gets an `aria-label`; the `<svg>` gets
  `aria-hidden="true"`. Keep the existing `title` tooltips.
- The sprite lives in the shell, which the stub server serves verbatim
  (`tests/test_utils/stub_server.rs` `DASHBOARD_SHELL`), so stub-browser tests
  see it. Fragments rendered by ticket [50](50-render-stub-fixtures-from-templates.md)
  carry only the `<use>` references.
- CSS: `.action-btn` already sizes 24×24; size the SVG inside it. The
  `currentColor` tint inherits the per-button hover colours in
  `assets/styles.css`.

## Tests

- Tier 1 (HTTP): the rendered story-log fragment exposes an accessible name on
  each icon-only button. This observes the response only, so it holds under
  every option in [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md).
- Update any existing assertion that names a removed glyph.

## Done when

- No font-glyph control remains in the shipped dashboard.
- Every icon-only control has an accessible name, and the sprite is defined once.
- `python build.py` is green, the user reviews the diff, then commit through
  `/commit-and-push`.

## Answer

Absorbed into [Visual identity pass](66-visual-identity-pass.md). The icon sprite
moved into the visual identity pass, so the shell and stylesheet are rewritten
once, alongside the palette. Closed as absorbed, not fixed.
