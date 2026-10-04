# Visual identity pass: SVG icon sprite and the chosen palette

Type: task (AFK)
Status: open
Blocked by: 56

## Question

Two passes over the same CSS, shell and templates, done as one commit so the
stylesheet is rewritten once.

### Replace the glyph controls with an SVG icon sprite (ticket 13)

Replace every font-glyph control in the dashboard with an inline SVG served from
one sprite, per [Decide the icon-button approach](13-decide-icon-buttons.md).

- One `<symbol>` sprite defined early in `<body>` in `assets/index.html`; each
  control renders a small `<svg aria-hidden="true"><use href="#i-…"/></svg>`
  tinted by `currentColor`.
- Paths from a permissive 24×24 stroke set (Lucide/Feather style, ~2px stroke).
  No new dependency, no network fetch.
- Controls to replace:
  - story-log Edit, Delete, Check, Retrigger (`NarrativeLogTemplate` in
    `src/adapters/driving/http/templates.rs`), plus the swipe `◀`/`▶` in the
    same template;
  - edit-mode save/cancel (`assets/index.html`, `showEditForm`);
  - text-check dismiss (`TextCheckPreviewTemplate`);
  - options-dock regenerate and edit (`OptionsDockTemplate`);
  - Games reset (`games/templates/games.rs`).
- `▶` keeps one control across both states. Its glyph and `aria-label` change
  with `next_swipe_index`: a navigation arrow when a next swipe exists, a "new
  swipe" glyph otherwise. Do not split navigation and generation into two
  buttons.
- Every icon-only button gets an `aria-label`; the `<svg>` gets
  `aria-hidden="true"`. Keep the existing `title` tooltips.
- The sprite lives in the shell, which the stub server serves verbatim
  (`tests/test_utils/stub_server.rs` `DASHBOARD_SHELL`), so stub-browser tests
  see it. Fragments rendered by ticket 50 carry only the `<use>` references.
- CSS: `.action-btn` already sizes 24×24; size the SVG inside it. The
  `currentColor` tint inherits the per-button hover colours in
  `assets/styles.css`.
- Test (tier 1, HTTP): the rendered story-log fragment exposes an accessible
  name on each icon-only button. This observes the response only, so it holds
  under every option in ticket 31. Update any existing assertion that names a
  removed glyph.

### Apply the chosen palette and fix the colour contrast

Apply the palette chosen in
[Prototype three calmer dark palettes](56-prototype-dark-palettes.md) to the
shipped dashboard, and fix the two contrast findings it lands on. Every text
colour must meet WCAG AA (4.5:1).

- Decision: [Decide whether to keep the neon palette](14-decide-palette.md);
  prototype: [Prototype three calmer dark palettes](56-prototype-dark-palettes.md).
- Finding 3.1: quoted dialogue renders in `--color-accent-red`
  (`rgb(255,68,68)`, 3.6:1 on the narration bubble); `ui_design.md` specifies
  orange `#ffb347`. Red speech also reads as an error.
- Finding 3.2: the swipe counter and timestamps use `#888` on `#1a3a3a`:
  3.46:1 at 11–12px.
- This absorbs [Fix dialogue colour and small-text contrast](12-fix-dialogue-colour-contrast.md):
  the palette values and the contrast fixes touch the same rules in
  `assets/styles.css`, so they ship as one commit instead of two sessions.
- Update the token values in `assets/styles.css`, plus `assets/games.css` and
  `assets/worlds.css` where they define colour, and the remaining non-token
  hardcoded hexes (`#cccccc` input text, `#00cccc` narration sender, the
  send-button gradient).
- Dialogue must use the dialogue colour, not `--color-accent-red`. This is a
  role-mapping bug in the `.dialogue` rules, not only a token value, so the
  palette pass alone does not fix it.
- Update the token tables in
  `docs/diataxis/reference/frontend/ui_design.md` to match; that file is the
  authority for token→usage.
- Keep token names hue-based. Rename a token only if its new value no longer
  matches its name.
- Pure CSS and doc values: no new test. Update any existing assertion the change
  breaks.

## Done when

- No font-glyph control remains in the shipped dashboard. Every icon-only
  control has an accessible name, and the sprite is defined once.
- Every colour token in `assets/styles.css` matches the chosen palette, and the
  `ui_design.md` tables match the shipped values. Dialogue uses the documented
  colour, and every measured text meets at least 4.5:1.
- `python build.py` is green, the user reviews the diff, then commit through
  `/commit-and-push`.
