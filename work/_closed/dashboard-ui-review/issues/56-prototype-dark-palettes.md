# Prototype three calmer dark palettes

Type: prototype (HITL)
Status: resolved
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

## Prototype

- `tmp/ui-review/t56/palettes.html` + `palettes.css` (local only); regenerate and
  re-serve with `python tmp/ui-review/t56/build_page.py`. Served copy:
  `http://galway-minipc:3000/assets/prototype-t56/palettes.html?palette=current|A|B|C|D`
  (arrow keys switch; `b` toggles bold dialogue; `c` shows a live WCAG contrast
  table). D was added after the user said they have red/green colour blindness:
  C plus blue for OK/active state. `cvd_check.py` simulates the pairs. Delete
  `assets/prototype-t56/` when this ticket resolves.
- It is a copy of the current `assets/index.html` shell on a current-source
  server, so every fragment is real; the story log loads once instead of
  polling, and one system entry is planted.
- Screenshots: `tmp/ui-review/t56/{current,A,B,C}-game.png`, `D-game-bold.png`, `B-settings.png`.

## Done when

- The three palettes are rendered and the user has picked one.
- The chosen values are recorded in the answer, or handed to
  [Visual identity pass](66-visual-identity-pass.md) (ticket 57 was folded into it).

## Answer

The user picked **D — Neutral reader, colour-blind safe**, with dialogue marked
by colour only (not bold). D is an untinted grey base, narration prose in the
body text colour, amber dialogue, and **blue instead of green for OK/active
state**. Colour carries meaning only on dialogue and UI state.

### Why D

- **It matches the norm.** SillyTavern's default theme (`public/style.css`) is
  neutral grey `rgb(23,23,23)` with off-white `rgb(220,220,210)` prose and
  orange `rgb(225,138,36)` quotes. Marinara uses near-black bubbles, off-white
  prose, and no colour on prose. D follows the same pattern. This also shows that
  ticket 14's premise was wrong: the neon did not come from SillyTavern's
  default theme.
- **Red/green colour blindness.** The user has red/green colour blindness. Under
  a Machado 2009 simulation (`tmp/ui-review/t56/cvd_check.py`), C's green
  Ready/Healthy and red Error/Degraded are delta-E 6.8 apart for deuteranopia,
  which is near-identical. A and B fail the same way (7.7, 9.1). D's blue against
  red is 63 (deutan) and 49 (protan). Narration vs dialogue holds at about 42 in
  every simulation.
- **Contrast.** Every measured text pair is at least 4.5:1 (table below). The
  shipped palette fails three of these pairs: dialogue (3.60), swipe counter /
  time on narration (3.46), and placeholder (2.66).

### Chosen values

`:root` tokens:

| Token | Shipped | D |
|---|---|---|
| `--color-bg-primary` | `#0a0a0a` | `#121212` |
| `--color-bg-secondary` | `#111` | `#191919` |
| `--color-bg-tertiary` | `#0f0f0f` | `#161616` |
| `--color-bg-header` | `#1a1a1a` | `#202020` |
| `--color-border` | `#333` | `#343434` |
| `--color-text-primary` | `#e0e0e0` | `#e6e6e6` |
| `--color-text-muted` | `#888` | `#a3a3a3` |
| `--color-text-placeholder` | `#555` | `#858585` |
| `--color-accent-green` (OK/active: tab, Ready, Healthy, primary/Send, focus, Narrator badge) | `#00ff00` | `#8ab4f8` (blue; rename, see below) |
| `--color-accent-green-bright` (location header) | `#4ade80` | `#a6d6a0` |
| `--color-accent-cyan` | `#00ffff` | `#9cc9d6` |
| `--color-accent-blue-cyan` | `#38bdf8` | `#8ab4e0` |
| `--color-accent-orange` (dialogue, warnings) | `#ffb347` | `#e8b878` |
| `--color-accent-yellow` (system prose, Thinking) | `#ffff00` | `#dcc888` |
| `--color-accent-red` | `#ff4444` | `#ea8080` |
| `--color-accent-pink` | `#ff6b6b` | `#e8a0a0` |
| `--color-button-gradient-start` / `-end` | `#2a2a2a` / `#1a1a1a` | `#2a2a2a` / `#222222` |
| `--color-button-border` | `#555` | `#4a4a4a` |
| `--color-log-input` | `#2a2a2a` | `#272727` |
| `--color-log-narration` | `#1a3a3a` | `#1f1f1f` |
| `--color-log-system` | `#3a3a1a` | `#27251c` |
| `--color-error-gradient-start` / `-end` | `#ff4444` / `#cc0000` | `#c25555` / `#963c3c` |

Hardcoded values in `assets/styles.css`, now taken from tokens:

| Rule | Shipped | D |
|---|---|---|
| `.btn-primary` gradient | `#2a5a2a → #1a4a1a` + green glow | `#283a54 → #202f45`, no glow |
| `.btn-cyan` gradient | `#2a4a5a → #1a3a4a` + glow | `#283338 → #20292d`, no glow |
| `.btn-danger` gradient | `#5a2a2a → #4a1a1a` + glow | `#402626 → #321e1e`, no glow |
| `#command-form button` (Send) | `#00aa00 → #006600` + glow | `#2c4466 → #243650`, no glow |
| hover states | lighter hardcoded gradient + glow | the start colour, `filter: brightness(1.15)` |
| focus glows (`rgba(0,255,0,…)`, `rgba(0,255,255,…)`) | neon box-shadow | `0 0 0 2px` ring, `color-mix(in srgb, var(--color-accent-green) 30%, transparent)` |
| badge / option / error tints (`rgba(0,255,0,.12)` etc.) | neon rgba | `color-mix(in srgb, <accent token> 12%, transparent)` (option button 7% / 15% hover) |
| `.log-entry .text q` | `--color-accent-red`, italic | `--color-accent-orange`, italic (role fix, finding 3.1) |
| `.log-entry.narration .text` | `--color-accent-cyan` | `--color-text-primary` (role change) |
| `.log-entry.input .text` | `#cccccc` | `--color-text-primary` |

Measured contrast in D (prototype, live computed styles): narration prose
13.21, dialogue 9.07, location header 10.01, swipe counter / time 6.53, time on
input bubble 5.92, input prose 11.97, system prose 9.26, muted on panel 6.97,
active tab 8.34, Ready 8.89, Thinking 11.29, error 7.05, degraded health 6.61,
Send label 5.80, placeholder 5.08.

### Handed to [Visual identity pass](66-visual-identity-pass.md)

- **Rename `--color-accent-green`.** It now holds blue, so the map's rule
  ("rename only when the value stops matching the name") applies. Suggested
  name: `--color-accent-ok`. `--color-accent-green-bright` stays green, so it
  keeps its name.
- **Do not use colour as the only signal (WCAG 1.4.1).** The Connections
  sub-tab's degraded dot is colour-only today.
- **`assets/games.css` reads `--color-text-secondary` and
  `--color-border-primary`.** Neither token exists, so the hardcoded fallbacks
  always win. Map them to existing tokens or define them.
- **Shipped Send label:** 2.27:1 against the top gradient stop `#00aa00`. D
  removes this problem.
- Remaining hardcoded `rgba(…)` tints in `styles.css`, `games.css` and
  `worlds.css` (issue tags, preset card, games list) follow the same
  `color-mix` pattern. The prototype covered the ones on the reviewed surfaces.

The served copy `assets/prototype-t56/` is deleted. The artifact and the helper
scripts stay in `tmp/ui-review/t56/` (local only).

