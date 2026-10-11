# Make LLM Messages rows keyboard-operable

Type: task (AFK)
Status: resolved
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

## Answer

The row header is now a real `<button type="button" aria-expanded aria-controls>` instead of a `<div onclick>`, so Enter and Space activate it through native button semantics and the expanded state is exposed to assistive technology. `aria-controls` points at the body's new id (`llm-msg-body-{{ msg.id }}`).

- **Keyboard + AT:** `toggleLlmMessage` mirrors the `.expanded` class into `aria-expanded`; `restoreLlmExpandedState` sets `aria-expanded="true"` on rows restored from the JS-held `expandedLlmMessages` set, so an expanded row stays expanded through the panel's htmx refresh. `assets/styles.css` neutralizes the button's default chrome (width, border, font, colour, alignment), so the visual row is unchanged and the mouse path is untouched.
- **Swap race found and fixed:** the restore hook originally ran on `hx-on::after-swap`, which fires before the swap settles — the class landed on a node the swap then replaced while `aria-expanded` stuck on the survivor. The hook moved to `hx-on::after-settle`; the refresh test passed 8/8 runs after the move (it had flaked before).
- **Focus across the 4s poll:** the header carries a stable `id="llm-msg-header-{{ msg.id }}"`, so htmx's built-in id-based focus restore keeps the keyboard on the row when the poll swaps the panel — the mechanism [Restore keyboard focus after in-place htmx swaps](45-restore-focus-after-inline-swaps.md) also relies on. Without it, Tab-to-header focus fell to `<body>` within 4s.
- **Poll interaction:** expanding a row already pauses the poll (`pauseLlmPolling`); the refresh test drives the same `innerHTML` swap directly.

Tests — **tier 2** (`tests/browser/stub/llm_messages.rs`), per `tests/STRATEGY.md`: the behaviour is the shipped client JS over a canned fragment, so faking the engine changes nothing. New spec `docs/specs/browser_llm_messages.md`:

- 35.1 — Tab from the LLM tab button reaches the header, Enter expands (body shown, `aria-expanded="true"`), Space collapses.
- 35.2 — an expanded row is still expanded after the panel is refreshed, and the header still reports it.
- 35.3 — a focused header has focus again after the refresh.

The spec Givens and Thens are written in user-facing terms, not field names. The stub fixture `tests/test_utils/stub_fixtures/llm_messages.html` gained a canned card so the shared shell has a row to drive (its empty state was replaced).

Validation: full gate `python build.py` green — 1532 integration passed / 2 skipped, 43 browser, 137 guardrails, 1 architecture; clippy/fmt/spec-coverage clean. No commit made (awaiting the user's review, then `/commit-and-push`).

Gate note: the run also fixed a pre-existing red gate — `tests/test_utils/stub_server.rs` carried a multi-line `//!` header that fails `guardrails_test_module_header`, introduced in `2182cf8e`. Confirmed pre-existing at HEAD with `git stash`, then condensed. Unrelated to this ticket and intended as its own commit.
