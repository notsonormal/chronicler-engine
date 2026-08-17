# Slash-command auto-suggestion UI

Type: task
Status: pending
Blocked by: 14

## Question

Add a slash-command auto-suggestion menu to the input box so the three steering commands are discoverable.

Per the design synthesis (`../research/04-design-synthesis.md`, Q14):

1. Today the input is a plain text field (`assets/index.html`, submitted as `ActionForm.command`). Slash commands (ticket 07) are invisible until typed.
2. Typing `/` opens a menu of available commands: `/narrator`, `/impersonate`, `/guide`. Matches ST's command-palette convention (Q14=B). Diverges from GG's button-heavy model.
3. Keyboard navigation (up/down/enter/escape) to select a suggestion; selecting populates the input with the command prefix.
4. No dedicated buttons (Q13=A) — the auto-suggestion menu is the only discoverability affordance.

This is frontend work in `assets/index.html` and any associated JS. Depends on the slash-command parser (ticket 07) existing so the suggested commands actually dispatch.

Blocked by: 07 (slash parser).
