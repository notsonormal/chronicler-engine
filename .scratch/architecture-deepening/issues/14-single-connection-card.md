# 14 — One definition of the connection card

Type: grilling
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Do we commit to rendering the settings panel's connection cards through the
same builder the edit/cancel fragments use (the prompt-preset pattern) — and
if so, what is the card's interface?

## Background

This is **candidate F** of the 2026-10-04 review. It is small. See
`assets/architecture-review-2026-10-04.html`, card F.

The friction: the connection card exists twice.

- `src/adapters/driving/http/settings/templates/settings.rs`: the panel
  template's `{% for conn in connections %}` loop hand-writes
  `<div class="connection-card">` with the Narrator/Quantifier badges and four
  hx buttons (Edit, Delete, Set as Narrator, Set as Quantifier).
- `src/adapters/driving/http/builders/connections.rs`: `connection_card_html`
  builds the same card for the fragment endpoints in
  `settings/handlers/settings.rs`.

Only the builder side is unit-tested (`builders/connections_tests.rs`). The
panel copy can drift without a failing test.

The precedent already exists: `builders/presets.rs` `preset_card_view` returns
`SafeHtml`, and `prompt_presets/templates/prompt_presets.rs` renders it in the
panel. Its doc comment states the intent: an edit-refreshed card is identical
to the same card in the panel.

Correction to the scout's report: the panel copy **is** escaped. The template
sets `ext = "html"`, so Askama escapes `{{ conn.name }}`. The risk is drift
between the copies, not XSS.

## What this ticket resolves

- **Commit or reject.**
- **Interface.** Builder function vs askama sub-template; what it takes (the
  connection plus narrator/quantifier ids?).
- **What survives.** The builder tests plus any panel tests in
  `tests/http/**`.

## Constraints

- Card markup is part of what browser tests drive. Check `docs/specs` for
  connection-card scenarios.
- Decision ticket, no implementation. Small enough that the grilling may
  conclude "just do it" and hand straight to planning.

## Notes

- Resolution uses `/grilling`.
- Related: ticket 15 (error policy) and the map's fog "HTTP adapter
  cohesion".
