# Slash-command auto-suggestion UI

Type: task
Status: resolved
Blocked by: 07

## Question

Add a slash-command auto-suggestion menu to the input box so the three steering commands are discoverable.

Per the design synthesis (`../research/04-design-synthesis.md`, Q14):

1. Today the input is a plain text field (`assets/index.html`, submitted as `ActionForm.command`). Slash commands (ticket 07) are invisible until typed.
2. Typing `/` opens a menu of available commands: `/narrator`, `/impersonate`, `/guide`. Matches ST's command-palette convention (Q14=B). Diverges from GG's button-heavy model.
3. Keyboard navigation (up/down/enter/escape) to select a suggestion; selecting populates the input with the command prefix.
4. No dedicated buttons (Q13=A) — the auto-suggestion menu is the only discoverability affordance.

This is frontend work in `assets/index.html` and any associated JS. Depends on the slash-command parser (ticket 07) existing so the suggested commands actually dispatch.

Blocked by: 07 (slash parser).

## Answer

Implemented the slash-command auto-suggestion menu in `assets/index.html` (JS) + `assets/styles.css` (CSS). Typing `/` in `#command-form input[name="command"]` opens a `position:fixed` palette listing `/narrator`, `/impersonate`, `/guide` in canonical order; typing a prefix filters; ArrowUp/Down move the `.active` highlight (wraps); Enter populates the input with `cmd + " "` and closes; Escape closes; clicking a suggestion populates + closes.

**Survives the action-area re-render.** The menu is a `<body>` child and all wiring (`input`, `keydown`, `mousedown`, `focusout`, capture-phase `submit`, `scroll`/`resize`) is delegated to `document` — so it survives the `#action-area` innerHTML swaps the `/action/check` text-check flow triggers (the input is recreated, the menu is not inside `#action-area`). Scenario 17.7 tests this invariant directly.

**Commands hardcoded in JS, not sourced from the parser.** The three commands are a `const SLASH_COMMANDS` array in `index.html`, matching the variants `Action::parse` recognizes (`guide`/`narrator`/`impersonate`, ticket 07). They are not fetched from the server — the parser lives in the domain layer and has no HTTP surface to enumerate them. Drift risk accepted: a future fourth command would need a one-line JS addition. Flagged for ticket 13 (documentation) if it matters.

**No active-state reset on filter change beyond re-selection.** When the prefix narrows the matches, `slashActiveIndex` resets to 0 (first match active); this is the ST convention and matches the design synthesis (Q14=B).

**Tests.** Seven browser behaviour tests added to `tests/browser/behaviour.rs`, tagged `// [docs/specs/browser.md] SCENARIO: 17.1`–`17.7`, covering: open-on-slash, prefix filter, arrow-key active movement, Enter populates, Escape closes, click populates, reopens after action-area re-render. Scenarios authored in `docs/specs/browser.md` (17.1–17.7); `validate_feature_spec.py` confirms coverage (build step 9 green). Scenario 17.7 uses a synthetic innerHTML swap (not a real submit) because a real submit starts a generation and `ActionAreaViewModel.is_disabled = status.is_generating()` disables the input, blocking the final "type `/`" step — same synthetic-event rationale as scenario 16.7.

**Verification.** `cargo nextest run --test browser slash_menu` → 7/7 pass. `python build.py` → all 12 steps green. Visual screenshot (UI-investigator mandate) confirmed: menu renders above the input, three commands in order, first item highlighted, dark-theme-consistent, no misalignment.

`python build.py` green.
