# Make LLM Messages rows keyboard-operable

Type: task (AFK)
Status: open
Blocked by: —

## Question

The LLM Messages panel has zero focusable elements. Each row header is `<div class="llm-message-header" onclick="toggleLlmMessage(this)">` (`src/adapters/driving/http/templates.rs:159`) with no `role`, `tabindex` or key handler, so the System / User / Response / raw-JSON bodies cannot be opened without a mouse and Tab leaves the panel for `<body>`. Make each row expandable with Enter and Space and expose the expanded state to assistive technology — for example a `<button aria-expanded>` header with the body as a labelled region, or a native `<details>`/`<summary>`. Keep the existing toggle behaviour and the JS-held expansion state that survives the panel's htmx refresh.

## Context

- Finding K2 (P1) of [Review keyboard use and screen-reader output](07-review-keyboard-screen-reader.md), and the one dashboard task that could not be completed by keyboard alone. Evidence `tmp/ui-review/99-llm-messages-keyboard.png`, `tmp/a11y-llm-messages.txt`, `focusables: []` (local only).
- The mouse path works (click → `.llm-message-card.expanded`) and must keep working.
- [Record failed LLM attempts and show them in LLM Messages](41-record-failed-llm-attempts.md) adds failed rows to this same panel; it is not a blocker, but the rows it adds need this keyboard path too.
- Announcing the panel's content to a screen reader is [Announce dynamic state changes to assistive technology](47-announce-state-changes-to-at.md)'s; this ticket is about reaching the content at all.

## Done when

- Every LLM Messages row can be expanded and collapsed with the keyboard, its expanded state is exposed to assistive technology, and expansion still survives the panel's refresh.
- A browser test drives the keyboard path (Tab to a header, Enter to expand, Space to collapse).
- Test placement follows `tests/STRATEGY.md`, and the answer names the tier. Until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves, write new spec Givens and Thens in `CONTEXT.md` terms rather than field names.
- `python build.py` is green, the user reviews the diff, and the work is committed through `/commit-and-push`.
