# Research how mature projects verify HTMX apps: what do the good patterns look like?

Type: research
Status:
Blocked by:

## Question

The dashboard is server-rendered HTML + HTMX partial swaps. Its tests drive a
real browser (Playwright) and poll for visible text. This ticket asks: what do
mature projects do to verify server-driven HTMX (or similar fragment-swap)
UIs robustly, from first principles?

Research (external docs + examples, not the local codebase) and compare:

1. **Request-level verification** — testing the HTTP contract directly
   (simulate the exact requests htmx would send, assert on returned HTML).
   Who uses this for HTMX, what tools (e.g. Django/Rails/Go/Rust equivalents),
   what does it cover/miss?
2. **Browser-level E2E with explicit readiness** — Playwright/Cypress patterns
   for HTMX apps specifically: waiting on htmx events
   (`htmx:afterSwap`, `htmx:load`, `htmx:trigger`), `HX-Request` headers,
   disabling animations, `networkidle`, or per-element ready hooks. How do
   people express "htmx has finished processing this element"?
3. **Headless-component / JS-DOM approaches** — verifying swap logic without a
   real browser.
4. **HTMX's own guidance** — how htmx docs/essays recommend testing, what
   `htmx.config` knobs exist for testability (e.g. `swaps`, `timeout`).
5. Anything about **eliminating timing from verification entirely** —
   event-driven rather than poll-driven waits.

This ticket's value is the *outside world* — patterns other teams converged
on from their own pain, which cannot be derived by re-reading this repo or
htmx's docs. Judge approaches by observed practice in real projects (htmx's
own test suite, Flask/Django/Rails/Go/Rust htmx apps, published Playwright
recipes), each claim cited to a source. Do not fill the survey from prior
knowledge; if a pattern can't be sourced, say so.

For each: cost, what's covered, what's missed, and who uses it in practice.

Bound the survey: aim for a one-page-per-approach comparison, not an
exhaustive literature review; stop when two consecutive sources stop adding
new approaches.

Deliverable: a linked markdown summary asset with a recommendation shortlist, not
a full proposal — ticket 04 (Decide the UI verification design) consumes it.
