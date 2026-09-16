# Research how mature projects verify HTMX apps: what do the good patterns look like?

Type: research
Status: resolved
Blocked by: 05

## Question

The dashboard is server-rendered HTML + HTMX partial swaps. Its tests drive a
real browser (Playwright) and poll for visible text. This ticket asks: what do
mature projects do to verify server-driven HTMX (or similar fragment-swap)
UIs robustly, from first principles?

Start from the resolution of "Analyze the Gemini deep-search survey": it
validates and distills a 23K external survey already covering request-level
contracts across four ecosystems, lifecycle-event readiness, JSDOM, htmx's
own test harness, and config knobs. Do NOT redo what survives validation;
this ticket fills the gaps the survey leaves after analysis — wrong/unsupported
claims, patterns asserted but not demonstrated in practice, approaches it never
considered — and extends each surviving pattern with concrete application cost
for this repo.

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

## Answer

Research asset: [htmx-verification-research.md](../assets/htmx-verification-research.md)
(with a session verification-notes section appended). No code changes.

**The maturity pattern is event-driven readiness anchored on `htmx:afterSettle`,
now confirmed at source level.** In htmx 1.9.10, swap inserts the DOM
immediately and fires `htmx:afterSwap`; trigger listeners attach only when the
settle task runs `processNode`, deferred by `defaultSettleDelay = 20` ms;
`htmx:afterSettle` fires after the tasks. "Wait for a settle on the target
before interacting" is therefore the sourced readiness condition, and the
listener-race hypothesis for the posture no-fire is confirmed by source. Three
independent practitioner fixes use the same pattern (fintrack#8 — "waits for
htmx's `afterSettle` so the swapped-in `<select>`'s hx-trigger is wired before
selecting"; NextSlope-10x#29; manja#89).

**Nothing makes the POST impossible-to-not-fire except a structural change.**
`hx-sync` (default `drop`) and the `changed` modifier can both silently
suppress a change-triggered POST. `hx-preserve` is the one structural option:
`handlePreservedElements` swaps the old element, listeners intact, back in
place of the fragment's copy. htmx's own suite collapses the window in tests
via meta config `{"defaultSettleDelay":0}`.

**Tiers, per observed practice.** Request-level verification is the
maintainer-endorsed day-to-day default (1cg: "more stable and 'functional'");
this repo already has the tier and asserts HX-Retarget/Reswap/Refresh. Browser
E2E stays for wiring, with the settle-gate. JSDOM/happy-dom: no Node toolchain
here, and practitioners explicitly exclude swap behavior from that tier — do
not add. `networkidle` is unsatisfiable with five `every Ns` pollers — ban it.

**Binding constraint for any readiness helper:** playwright-rs 0.9.0 has no
`wait_for_function`/`wait_for_selector` (verified in crate source). The
counter bridge must be `add_init_script` + a Rust poll over `evaluate_value`
(the repo's `wait_for_condition_async` shape), armed with a baseline before
the interaction.

**Shortlist handed to ticket 04** (ranked): 1. settle-gate the swap→interact
step; 2. `hx-preserve` (or stable ids) on the posture selects; 3. demote the
posture autosave contract to the HTTP tier; 4. `waitForResponse`-style
completion helper (proves dispatch, not wiring); 5. ban `networkidle`; 6. no
JSDOM tier. #1+#3 compose; #2 is the only true by-construction fix; #3 removes
the test.

**Could not source:** a targetId-scoped readiness signal (the survey's pattern
stays unproven — target-scoping is documented-possible via `detail.elt` but
unattested in practice); any maintainer statement on listener-attachment
timing; a first-party axum+htmx testing guide; a Rails htmx request-spec
example.

No fog graduated: every item in the map's Not-yet-specified graduates from
ticket 04's design decision, not from this research. No new tickets surfaced.

**Follow-up: per-option assessment added to the asset.** Each option now carries
performance, reliability, and maintainability, with epistemic labels. Key
numbers: browser tier ≈5.8 s/test [inferred] vs millisecond HTTP-tier tests;
the settle-gate trims poll-tick overhead only — it buys correctness, not speed.
The speed levers are demotion (shortlist #3), then de-serializing the browser
binary once the race is closed (large but untried gain, 8 cores vs 1), then
optionally a shared fixture (tradeoff: state bleed vs the deliberate
fresh-per-test design). Shortlist ranking unchanged.
