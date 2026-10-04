# Decide whether to keep the neon palette

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The dashboard uses a neon palette: cyan `#00ffff` narration on a teal bubble, `#00ff00` primaries, and `#ffff00` system text. It has high contrast, but it is tiring for long reading. Is it an intended identity? If not, what replaces it?

## Context

- Finding 3.4. Screenshots 01, 20.
- The token tables in `docs/diataxis/reference/frontend/ui_design.md` are the authority.
- If the answer is "change it", the next step is probably a `prototype` ticket (a few palette options side by side), not a direct edit.
- [Fix dialogue colour and small-text contrast](12-fix-dialogue-colour-contrast.md) waits on this.

## Done when

- The decision is in the ticket answer. Follow-up tickets are created.

## Answer

The neon palette is not an intended identity. It is the default inherited from
the SillyTavern lineage; no commit, doc or decision chose these values. It is
replaced, and the replacement values are chosen by a prototype: three dark
palettes on one representative page.

The replacement is a full token value pass, not an accent-only tweak — the
story-log bubble backgrounds carry as much of the reading fatigue as the
foreground accents. In scope: the accent text colours, the `--color-log-*`
bubble backgrounds, borders, the hardcoded button gradients, and the remaining
non-token hardcoded hexes (`#cccccc` input text, `#00cccc` narration sender, the
send-button gradient). The dark base stays; light mode is out of scope, because
the review measured the dashboard as desktop-first.

Token names stay hue-based (`--color-accent-cyan` and friends); only values
change. A token is renamed in the implementation ticket only if its new value
no longer matches its name. A full role-based rename has no finding behind it
and belongs in its own ticket if wanted.

Prototype brief: one self-contained page showing three candidate dark palettes
over the same representative markup — the Game tab's narration, dialogue and
system bubbles, the status pill, a Settings connection card, and the tab bar —
desktop only. The user picks one.

### Graduated

[Prototype three calmer dark palettes](56-prototype-dark-palettes.md)
(prototype, HITL) and [Apply the chosen palette and fix the colour contrast](57-apply-chosen-palette.md)
(task, AFK; blocked by it).
[Fix dialogue colour and small-text contrast](12-fix-dialogue-colour-contrast.md)
is folded into the palette implementation, so the values and the contrast fixes
ship as one commit.
