# Write `DESIGN.md`: type, colour and components

Type: prototype
Status: open
Blocked by: 03

## Question

What are the type, the colour and the core components of the chosen direction, as written in `DESIGN.md`?

## Context

- Write `DESIGN.md` from the direction that [Choose the direction of the Game tab](03-choose-the-direction.md) picked, not from the current look (map Notes). Follow `.pi/skills/impeccable/reference/document.md` (its frontmatter tokens and eight sections) and `reference/new-work.md` for a new visual world.
- The owner picks the type and the colour from screenshots: 2–3 type pairings (a reading face for the story, and a face for the controls), and 2–3 palettes. The agent decides the components (buttons, inputs, cards, tabs, badges) by the rules of `DESIGN.md`.
- Fonts are self-hosted files in `assets/` with a licence that allows this (map Notes).
- `DESIGN.md` is the source of the design values, and `assets/styles.css` follows it (map Notes, "The root docs"). Add `DESIGN.md` to `ROOT_DOCS` in `scripts/vale_lint.py` and in `scripts/validate_docs.py`, and write it in the STE rules that Vale checks.
- `assets/styles.css` already has 51 custom properties (`--color-*`, `--spacing-*`, `--font-*`). Map the new tokens onto them where they fit, so the build changes values, not every rule.
- The detector flagged a coloured left border (`side-tab`) on two components in `assets/styles.css`: `.preset-card.default` on Prompt Presets and `.option-btn`, the Options of the Game tab. It also flagged 11px text on LLM Messages (`tiny-text`, `.llm-message-model` uses `--font-size-xs`). `DESIGN.md` gives a rule for each.
- The build ticket of the fonts adds a browser check that each font loaded (`document.fonts.check`). A missing font file falls back to the system font, and no other check fails (map, "Failure modes").

Done when `DESIGN.md` exists at the repo root with the owner's picks and passes `python build.py vale` and `python build.py validate-docs`, and the answer records the picks and lists the build tickets that it needs (at least: the shared tokens and the story typography).
