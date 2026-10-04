# Morph the story-log poll swap

Type: task (AFK)
Status: open
Blocked by: —

## Question

Stop the 2s story-log poll from destroying DOM state. Replace the poll's `innerHTML` swap with a morph, so an idle poll touches no nodes and a changed poll touches only the nodes that changed. This fixes finding 2.4, its keyboard consequence K1, and — as a separate shell change — makes the log keyboard-scrollable (K7).

## Context

Decision: [Decide how the story-log poll keeps DOM state](10-decide-story-log-poll-swap.md). Option B — client-side morph.

Changes:

- Vendor idiomorph's htmx extension at `assets/idiomorph-ext.min.js` (`https://unpkg.com/idiomorph/dist/idiomorph-ext.min.js` — the `idiomorph-ext` build bundles the extension). 0BSD. The existing `/assets` static route and the stub server's `ServeDir` both serve it with no wiring change.
- `assets/index.html`: `<body hx-ext="morph">`.
- `assets/index.html`: `#story-log` `hx-swap="innerHTML"` → `hx-swap="morph:innerHTML"` (morphs the children, leaves the container itself alone).
- `assets/index.html`: add `tabindex="0"` and an accessible name (`aria-label`) to `#story-log`, so a keyboard user can focus it and arrow-scroll (K7). Do **not** add `role="log"` — it is an ARIA live region and would announce the narrative on every poll; announcing is [Announce dynamic state changes to assistive technology](47-announce-state-changes-to-at.md). If [Expose the dashboard to assistive technology](46-expose-dashboard-to-assistive-technology.md) is in flight, coordinate the accessible name.
- `NarrativeLogTemplate` (`src/adapters/driving/http/templates.rs`): add `id="entry-{{ entry.id }}"` to each `.log-entry`. Morph matches on `id`; `data-id` will not do, and the 50-entry cap removes the oldest entry every turn, so removal matching matters.
- Keep `pausePolling`/`resumePolling` (the edit-mode pause). Morph does not cover a stale server view overwriting an in-progress edit.
- Scope: the story log only. Leave `#options-dock`, `#status-display`, `#visual-sidebar` and `#llm-messages-panel` on `innerHTML`.
- `switchSwipe`'s direct `innerHTML` write stays. A following morph onto equal content is a no-op. No hash, no bookkeeping.

Tests — **tier 2** (`tests/browser/stub/story_log.rs`), per `tests/STRATEGY.md`. The behaviour is the client's swap, so faking the server does not change it. New scenarios in `docs/specs/browser_story_log.md`, beside Scenario 30.3:

- a text selection in the log survives at least two poll cycles (finding 2.4);
- focus inside a `.log-entry` survives at least two poll cycles (K1);
- `#story-log` takes focus and arrow keys scroll it (K7).

The stub fixture `tests/test_utils/stub_fixtures/story_log.html` gains `id` attributes on its entries, so the fixture keeps the structural elements the shell addresses (the stub tier's accepted tax). Exercise morph's removal path if cheap: serve a second fixture without the oldest entry.

Write new spec Givens and Thens in `CONTEXT.md` terms, not field names, until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves.

Watch: the `MAX_LOG_DISPLAY = 50` cap makes removal routine, not an edge case. An edit or swipe that changes an entry still updates that entry's nodes, so a selection inside the changed entry may collapse; a selection elsewhere must survive.

## Done when

- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.
