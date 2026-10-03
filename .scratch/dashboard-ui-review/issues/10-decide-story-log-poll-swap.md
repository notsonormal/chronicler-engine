# Decide how the story-log poll keeps DOM state

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The 2-second story-log poll replaces every entry node, even when nothing changed. A text selection is lost within about 2.6 seconds, and hover and focus state reset (finding 2.4). Which approach stops this?

- **Option A:** the server tells the client "unchanged" (for example, a version check and a no-swap response).
- **Option B:** a client-side morph swap that keeps unchanged nodes.
- **Option C:** pause the poll while the user selects text or edits.

## Context

- Check these facts before the grilling:
  - How does edit mode survive the poll today?
  - Which htmx version is bundled (`assets/htmx.min.js`)?
  - Is a morph extension available?
- Poll cadences are in `docs/diataxis/explanation/dashboard_design.md`.
- Added from [Recover from a failed message save instead of freezing the story log](27-recover-failed-message-save.md): `htmx.trigger("#story-log", "htmx:refresh")` (also used for `#visual-sidebar`, `#header` and the LLM panel) fires an event nothing listens for; `assets/htmx.min.js` has no `htmx:refresh`. After save, cancel, delete or swipe, the log only updates on its next 2s poll. [reported by the implementer]

## Done when

- The decision is in the ticket answer. Implementation tickets are created.

## Answer

Decided: **Option B — a client-side morph swap.** The poll keeps running at 2s, but the swap merges the fragment into the existing DOM instead of replacing it. Unchanged nodes keep their identity, so a text selection, hover, focus and scroll position survive. This is finding 2.4, plus its keyboard consequence K1 from [Review keyboard use and screen-reader output](07-review-keyboard-screen-reader.md).

Mechanism: idiomorph, the htmx author's morphing library, as the htmx `morph` extension.

- Vendor `assets/idiomorph-ext.min.js` (0BSD, ~3.3KB gzipped, no dependencies; its README recommends vendoring). The repo already vendors htmx.
- `<body hx-ext="morph">`.
- `#story-log`: `hx-swap="innerHTML"` → `hx-swap="morph:innerHTML"` (morphs the children, leaves the container).
- Each `.log-entry` gains `id="entry-{{ entry.id }}"`. Morph matches on `id`; `data-id` will not do, and the `MAX_LOG_DISPLAY = 50` cap removes the oldest entry every turn, so removal matching matters.
- Keep the edit-mode pause (`pausePolling`/`resumePolling`). Morph does not cover a stale server view overwriting an in-progress edit.
- `switchSwipe`'s direct `innerHTML` write stays. A following morph onto equal content is a no-op, so no hash bookkeeping is needed anywhere.

Rejected:

- **Option A** (server says "unchanged", no-swap). It works, but it adds a client-side hash invariant that must always equal the DOM. Every path that touches `#story-log` must maintain it; `switchSwipe` breaks it on the first read. It also protects only unchanged polls — when content changes, every node is still replaced. More moving parts for less. `hx-ptag` is the htmx analogue, and it has the same fit problem here.
- **Option C** (pause the poll on text selection). Fixes selection only. The finding also names hover and focus, and a pause on hover would stop the log updating at all.

Prior art supports B. SillyTavern holds the chat list client-side and appends/reconciles nodes; Marinara Engine (React) reconciles keyed message components. Both reconcile by identity instead of replacing the list. htmx's own polling pattern recommends morph for this problem.

Scope: the story log only. `#options-dock`, `#status-display`, `#visual-sidebar` and `#llm-messages-panel` keep `innerHTML`; each can adopt morph later as a one-attribute change.

**K7 (P2, from ticket 07)** joins the implementation: `#story-log` gains `tabindex="0"` and an accessible name, so a keyboard user can focus it and arrow-scroll. Morph does not fix K7. It is a separate one-line shell change, kept in the same ticket because both are the poll's keyboard consequences.

Test tier: **tier 2** (stub browser), per `tests/STRATEGY.md`. Selection and focus survival depend on the client's swap, not the server behind it, so faking the server does not change the behaviour. New scenarios belong in `docs/specs/browser_story_log.md`, beside Scenario 30.3. Givens and Thens in `CONTEXT.md` terms until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves. No tier-1 test: no server contract changes.

Graduated [Morph the story-log poll swap](48-morph-story-log-poll-swap.md) (2.4, K1, K7) and [Delete the dead `htmx:refresh` calls and fix the docs](49-delete-dead-htmx-refresh.md).
