# Decide the UI verification design: what the browser tier is for, and what the app exposes for it

Type: grilling
Status:
Blocked by: 01, 02

## Question

**HITL.** Grill the user — do not infer an answer from the evidence alone.
Consume the resolutions of "Audit the current browser-test design",
"Research how mature projects verify HTMX apps", and "Root-cause the hx-post
change-event no-fire" and settle, from first principles:

1. **What the browser tier is for.** After the tier audit, which behaviours
   genuinely require a real browser, and which should verify at request level?
   What happens to the current 26 tests (keep / move to HTTP / delete)?
2. **Readiness.** How does a test know the UI is ready to act on — the
   no-fire race (resolved in ticket 03: `change` dispatches inside htmx's
   20 ms `defaultSettleDelay` window, before trigger listeners attach) must
   be structurally impossible in the chosen design, not polled around.
3. **App-side vs harness-side.** Should the dashboard emit explicit readiness
   signals (htmx lifecycle hooks, a ready flag), should the harness interpret
   htmx state, or should verification skip the browser where possible so the
   question dissolves? Weigh app-changes against test-changes honestly per the
   user's "nothing is sacred, prefer simplification" standing preference.
4. **Fixture shape.** Per-test server+browser vs shared fixture (invariants.rs
   already shares one) — and whether the shared `#story-log .log-entry` gate
   at browser.rs:87 survives, and as what.
5. **What gets deleted.** Harness surface, wait primitives, duplicative tests —
   the design should name what it removes, not only what it adds.

The map's destination is a shipped redesign, so this decision is the last
planning step: the verdict on each of the five above, plus the concrete
implementation tickets it graduates (name them, create them as child issues,
wire their blocking). The remaining map is executing those tickets.

Record the decision under `## Answer`, create the implementation tickets the
decision specifies, and append a one-line gist + link to the map's
Decisions-so-far.
