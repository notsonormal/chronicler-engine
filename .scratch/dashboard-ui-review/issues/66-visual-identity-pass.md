# Visual identity pass: SVG icon sprite and the chosen palette

Type: task (AFK)
Status: resolved
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
- **Chosen palette: D — Neutral reader, colour-blind safe**, dialogue marked
  by colour only (not bold). The full token and hardcoded-value tables are in
  the answer of [Prototype three calmer dark palettes](56-prototype-dark-palettes.md);
  apply them as written. The reference implementation is
  `tmp/ui-review/t56/palettes.css` (local only, the `html[data-palette="D"]`
  block plus the shared overrides).
- **Rename `--color-accent-green`** (for example to `--color-accent-ok`). Its D
  value is blue `#8ab4f8`, so the hue name no longer matches. This choice
  protects the user, who has red/green colour blindness: green vs red OK/error
  was near-identical in simulation.
- Narration prose changes role: `.log-entry.narration .text` uses
  `--color-text-primary`, not `--color-accent-cyan`.
- Hardcoded gradients, glows and `rgba(…)` tints are replaced with values
  taken from the tokens (`color-mix`, no neon glow), as in the answer.
- Colour must not be the only signal (WCAG 1.4.1). Give the Connections
  sub-tab's degraded dot a non-colour cue, or confirm that its `title` and
  the banner are enough.
- `assets/games.css` reads the undefined tokens `--color-text-secondary` and
  `--color-border-primary`. Map them to existing tokens or define them.
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
  matches its name (`--color-accent-green` is the one known case).
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

## Answer

Resolved. One commit carries both passes: the glyph controls become a sprite icon set, and
the stylesheet moves to palette D.

**Icons.** One 13-symbol `<symbol>` sprite sits at the top of `<body>` in
`assets/index.html`, hidden by zero size plus `overflow: hidden` (not `display: none`, which
breaks `<use>` in some browsers). Every icon is stroke-only — `fill: none`,
`stroke: currentColor`, 14×14 by default — so a control tints its own icon. Paths are Lucide
(ISC; the Feather-derived ones also MIT), vendored with `assets/LICENSE-lucide.txt`.

Replaced: story-log Edit (`#i-pencil`), Delete (`#i-trash`), Retrigger (`#i-zap`) and the
swipe pair (`#i-chevron-left` / `#i-chevron-right` / `#i-refresh-cw`); the Send button
(`#i-send`) and its generating spinner (`#i-loader-circle`), including the copy the shell JS
writes on a status swap; the edit-mode Save/Cancel the shell injects (`#i-check`, `#i-x`);
options-dock Regenerate (`#i-refresh-cw`) and Edit (`#i-pencil`); Games Reset
(`#i-rotate-ccw`); the text-check preview header (`#i-spell-check`); and the Settings back
link (`#i-chevron-left`). Both hand-copied stub fixtures (`action_area.html`, `games.html`)
were updated to match, and the dead `.btn-icon` span is gone.

The forward swipe control stays one button: with a later swipe it is `#i-chevron-right`
named "Next swipe"; on the latest swipe it is `#i-refresh-cw` named "Retry". That keeps the
existing `title` tooltips and the three selectors that read them.

Every icon-only button carries an `aria-label` equal to its `title`, and every `<svg>`
carries `aria-hidden="true"`.

**Colour.** Palette D applied as written, with `--color-accent-green` renamed to
`--color-accent-ok` (now blue `#8ab4f8`). Hardcoded gradients, glows and `rgba()` tints
became tokens: `--color-button-{primary,cyan,danger,send}-{start,end}`, the `--color-tint-*`
family via `color-mix(in srgb, …)`, and one `--shadow-focus-ring` that replaces every neon
focus glow. Narration prose moved to `--color-text-primary`; quoted dialogue moved to
`--color-accent-orange` (finding 3.1 — it rendered in error red); input text and the
swipe/timestamp greys moved to tokens (finding 3.2). `assets/games.css` and
`assets/worlds.css` were mapped onto real tokens, so their undefined
`--color-text-secondary` / `--color-border-primary` reads and the hardcoded fallbacks that
always won are gone. Rules with no markup left (`.connection-status`, `.btn-check`,
`.preview-compare`, `.preview-arrow`, `.preview-send-form`, `.action-btn.retry-btn:hover`)
were deleted rather than repainted.

The Connections sub-tab's degraded dot became a `#i-triangle-alert` icon with a `title`, so
the signal is no longer colour alone (WCAG 1.4.1); the rule and its class were renamed to
`.subtab-degraded-marker`.

**One deliberate deviation from palette D.** `--color-error-gradient-start` is `#bd5252`,
not palette D's `#c25555`: white toast text on `#c25555` measures 4.44:1, just under the
ticket's 4.5:1 bar, and white is already the lightest text, so the gradient stop itself had
to darken. `#bd5252` measures 4.67:1. Every other palette value is unchanged.

**Measured contrast.** I recomputed every text pair. The minimum is 4.67:1 (the toast).
Ticket 56's 15 measured pairs all sit at 5.08:1 or above; two further pairs sit between 4.5
and 5 — the Send label on the gradient's top stop (4.69:1) and the toast (4.67:1). Both
clear WCAG AA 4.5:1, the bar ticket 66 states; the "at least 5:1" line describes only the
pairs ticket 56 measured.

**Tests.** Tier 1 (HTTP), observing the response only:

- `tests/http/story_log.rs` scenario 8.6 (`test_story_log_icon_buttons_have_accessible_names`)
  — every icon-only button in the rendered log has a non-empty `aria-label` equal to its
  `title`, and no `<svg>` is reachable by assistive technology. It first asserts the fixture
  really renders Edit, Delete, Retrigger and Swipe, so it cannot pass vacuously.
- `tests/http/dashboard.rs` scenario 39.2
  (`test_every_referenced_icon_is_defined_once_by_the_shell`) — the shell defines each symbol
  once, and every icon referenced by the shell or by story-log / games / connections-new /
  action-area resolves to a definition. A missing symbol draws nothing and raises nothing,
  so the reference is the only place a typo shows.
- `tests/http/settings.rs` extended — the degraded Connections sub-tab now also asserts the
  marker carries `#i-triangle-alert`, pinning the non-colour cue.

Unit: `templates_tests.rs` asserts the accessible names, the state-dependent swipe icon and
name, and the spinner; `settings/templates/settings_tests.rs` asserts the degraded marker's
icon. Docs: `docs/specs/story_log.md` gained scenario 8.6, `docs/specs/dashboard.md` gained
39.2, and both endpoint lists were widened.

**Docs.** `ui_design.md`'s token and component tables now match the shipped values. The
stale Dialogue bubble row and the whole Sender colour column went with them, because the
shipped CSS has no `.dialogue` or `.sender` class — quoted speech renders as `<q>` inside the
entry text. That also removed the `--color-log-dialogue` and `--font-size-sender` token rows,
which exist in no stylesheet at all. Ticket 56's handoff had left those to ticket 23, but they
were parts of the same tables this ticket rewrote.

**Verified in a browser.** A probe server on `--port 3001` with its own DB
(`tmp/probe66.log`): 13 symbols, zero unresolved `<use>` references, no raw glyph character
left in the rendered DOM, `--color-accent-green` empty, computed narration text
`rgb(230,230,230)`, quoted dialogue `rgb(232,184,120)`, active tab and Send label
`rgb(138,180,248)`. Screenshots: `tmp/t66-game.png`, `tmp/t66-settings.png`,
`tmp/t66-games.png`, `tmp/t66-presets.png`.

`python build.py` is green (1570 integration, 68 browser, 165 guardrail, 1 architecture).
Uncommitted, pending review and `/commit-and-push`.

**Review follow-up.** A two-axis review (Standards and Spec, independent readers) plus an
extra-lens pass ran over the diff. It confirmed the icon set, the palette values, the contrast
sweep and the sprite references, and found drift the change had left behind. Every confirmed
finding is now fixed:

- `docs/diataxis/reference/frontend/dashboard.md` — the glyph references (▶ Send, ■ Stop,
  ✎ edit, 🗑 delete, ◀/▶ swipe, ♻ retrigger, ↻ reset) became sprite ids, "Ready in green"
  became `--color-accent-ok`, "four `log_type` classes" became three (`dialogue` never
  existed), and the button-label claims became "Generating…".
- `ui_design.md` — the swipe-control gate now reads "the last entry is a Narration or Input"
  rather than `swipe_count > 1`; the corrected-text value is `--color-text-primary` with no
  `word-break`, because this change deleted the rule that supplied both.
- `docs/diataxis/explanation/dashboard_design.md` — gained a Palette section carrying the
  design rationale (the blue accent, the amber dialogue, the measured contrast floor), which
  reference mode must not hold.
- `tests/http/support/http_assertions.rs` — `referenced_icons` and `defined_icons` share one
  `icon_ids` helper, and opening-tag detection is quote-aware, so a `>` inside an attribute
  value cannot truncate the tag.
- `tests/http/options.rs` and `docs/specs/options.md` 24.14 — the options dock's icon
  references are now checked against the shell sprite, closing the one GET-reachable
  fragment the icon scan had skipped.
- The module summaries of `tests/http/dashboard.rs` and `tests/http/story_log.rs` name the
  coverage they now carry, and scenario 39.2's title no longer over-claims.

Two review claims did not reproduce: `attribute_value` does see an `aria-label` that is a
tag's first attribute, and it does catch a mismatched label when a `title` exists; and
scenario 8.6's "a Narration with one Swipe" is accurate, because `Message::new` always
builds one swipe.

**Follow-ups (not fixed here).**

- The Games tab's "Rename" is a native `<details>` whose UA `::marker` still draws a
  triangle, while `.preset-add-toggle` hides its marker and draws a CSS triangle. Two
  disclosure cues for the same widget kind; pre-existing, recorded in
  [_resolved/37-follow-up-small-review-issues-2.md](_resolved/37-follow-up-small-review-issues-2.md).
- `CONTEXT.md` lists **Retry** as a deprecated term for redoing the last generation, but the
  swipe control's tooltip and `docs/specs/swipe_new.md` still use it. This ticket says keep
  the existing tooltips, so the wording stands pending a user decision.
- `settings/handlers/settings_tests.rs:176` is unformatted in `HEAD` (unrelated to this
  ticket); a plain `python build.py` will rewrite it.
