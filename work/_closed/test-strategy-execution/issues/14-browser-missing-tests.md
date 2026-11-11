# What browser tests are missing?

Type: grilling (HITL)
Status: resolved
Blocked by: 07
Assignee: agent (this session)

## Question

The browser tier audit (planning map ticket 14) was backward-looking:
it classified existing tests as keep or move-down. Ticket 07 resolved
the target state for tests that **exist** — split into `behaviour.rs`
(specced, 6 tests) + `invariants.rs` (not specced, 7 tests). But ticket
07 explicitly deferred the forward-looking question: **what
presentation-layer behaviours have no test at all?**

This ticket grills out that question. It does not write tests — that
graduates as implementation tickets from this resolution.

### Candidate gaps (from ticket 07 grilling)

These are the candidates surfaced but not evaluated. The grilling should
test each against the browser placement rule ("must assert something only
a browser can see") and decide: real gap, already covered, or out of
scope.

- **Responsive layout.** No test asserts layout at narrow viewport widths
  (mobile). `test_no_horizontal_overflow` checks desktop; nothing checks
  `< 768px`. Does the app claim mobile support? If yes, what breakpoints
  matter?
- **Accessibility.** No test asserts ARIA roles, focus management, or
  keyboard navigation. Edit-mode activation: does focus move to the
  textarea? Delete confirm: is focus restored after cancel? Tab order
  through the form?
- **htmx swap visual transitions.** No test asserts htmx swap classes
  (`htmx-swapping`, `htmx-settling`) or transition timing. Does the app
  define swap transitions? If yes, are they load-bearing or cosmetic?
- **Error-state rendering.** No test asserts how a failed action (500
  from /action) renders in the DOM. Does an error toast appear? Does the
  form recover? Is the error styled?
- **Offline/connection-status transitions.** `test_connection_status_indicator`
  (moving down in ticket 08) just checks the element exists. No test
  asserts the indicator *changes* when the server disconnects. Is the
  indicator wired to a JS polling loop that detects disconnects?
- **Empty-state rendering.** No test asserts what the story-log looks
  like with zero entries (new game). Is the empty state styled
  intentionally, or does it just render nothing?

### What this ticket does NOT do

- Does not write tests — that's the implementation ticket that graduates
  from this resolution.
- Does not re-litigate the 13 keep-tests — that's settled (ticket 07).
- Does not decide tier placement for tests that don't exist yet — if a
  gap is real, the graduating implementation ticket places it (browser
  vs HTTP E2E vs unit) using STRATEGY.md.

### Scope guardrail

The destination of this map is "component tier dissolved," not
"exhaustive browser coverage." Gaps identified here graduate as
tickets **on this map** only if they're sharp and in-scope (presentation-
layer, browser-tier). Gaps that are really HTTP E2E or unit gaps belong
on the codebase-wide investigation map (ticket 09 and its graduates), not
here. Gaps that are speculative ("nice to have") stay as fog.

### Context

- [Ticket 07 resolution](07-browser-tier-design.md) — the behaviour /
  invariant split, `browser.md` spec, STRATEGY.md amendments.
- [Browser tier audit asset](../test-strategy/assets/browser-tier-audit.md) —
  the 17/13 classification.
- Tier rules in `tests/STRATEGY.md` — browser is
  presentation-only.

## Answer

Grilling evaluated each candidate against the browser placement rule
("must assert something only a browser can see") and the scope guardrail
(destination = component tier dissolved, not exhaustive browser coverage;
sharp + presentation-layer → graduates on this map; speculative → fog;
non-browser → belongs on the codebase-wide investigation map).

### Real gaps → graduate as one implementation ticket

**1. Responsive layout at narrow viewport.** `assets/styles.css`
declares `@media (max-width: 768px)` (flex-direction column, story-log /
visual-sidebar width 100%) and `@media (max-width: 480px)` (header
flex-wrap, action-area column, full-width form controls) — the app
*claims* responsive support. `test_no_horizontal_overflow` and
`test_element_positioning` assert layout at desktop viewport only; nothing
asserts the responsive rules actually apply at `< 768px`. Browser-only —
need to resize the viewport and read computed styles / `getBoundingClientRect`.
Assertion shape: rendering invariant (computed style at breakpoint), not a
Given/When/Then behaviour → lives in `invariants.rs`, no spec scenario.

**2. Error-state toast rendering.** `assets/index.html` registers a global
`htmx:beforeSwap` listener: on `evt.detail.isError`, it strips tags from
`serverResponse` and calls `showError(text || "Request failed")`, which sets
`#error-notification.textContent` and adds the `.visible` class; a 5s
`setTimeout` removes it. A 500 from POST /action (or any htmx swap error)
renders a toast that auto-hides after 5s. The HTTP response body is
HTTP-observable, but the **toast rendering, class toggle, and auto-hide**
are browser-only. No test asserts any of it. Assertion shape: Given/When/Then
(trigger action → 500 → toast visible → toast auto-hides) → lives in
`behaviour.rs`, tagged against a new `browser.md` scenario 16.7.

Both are sharp and presentation-layer. They graduate together as one
implementation task ticket (child of map): "Add browser tests for
responsive layout + error-state toast" (create-then-wire below).

### Not gaps (no behaviour exists to assert)

**3. htmx swap visual transitions.** No `htmx-swapping` / `htmx-settling`
class is used anywhere; no `hx-swap="... transition:..."` modifier; the
CSS `transition:` rules in `styles.css` are hover/focus/border cosmetic
transitions on static elements, not htmx swap transitions. Nothing
load-bearing to assert — the app defines no swap transitions.

**4. Offline / connection-status transitions.** `HeaderTemplate` hardcodes
`<span class="connection-status connected" id="connection-status">Connected</span>`.
No JS ever toggles `.connected` → `.disconnected`; no polling loop detects
server disconnects. The indicator **does not change** in response to
anything. The "change on disconnect" behaviour doesn't exist — there is
nothing to test. A connection-status change feature is a possible future
feature, not a missing test for existing behaviour.

### Fog (stays in Not yet specified)

**5. Accessibility (ARIA, focus management, keyboard navigation).** Real
presentation-layer gaps exist: no `aria-*` attributes or `role=` anywhere
in templates (grep-confirmed); no `tabindex`; `showEditForm` creates the
`#edit-textarea` but never calls `.focus()` on it (focus does not move);
`cancelEdit` restores innerHTML but restores focus to nothing in particular;
tab order is default DOM order with no keyboard handlers. But the app
doesn't *claim* accessibility support — no ARIA infrastructure, no keyboard
handlers. Writing the tests would assert bugs (focus not moving, no ARIA
roles), which is a code-change decision (does the app claim a11y? should
focus move?) rather than a missing-test decision. That grilling belongs on a
future UI / a11y effort, not on this map (destination = component tier
dissolved, not a11y audit). Stays as fog toward a future effort.

**6. Empty-state rendering.** New games always carry a scenario message
(ticket 06: `create_game_persists_scenario_message_and_swipe`; also asserted
in `src/bootstrap/run_tests.rs` "Restart should not duplicate the scenario
message"). `#story-log` with zero entries never occurs in practice — the
`NarrativeLogTemplate` `{% for entry in entries %}` loop always has at least
one entry. There is no empty state to render and no intentional empty-state
styling. Not a gap — the state doesn't exist.

### Scope discipline

No out-of-scope ruling issued. Two gaps graduated (sharp, browser-tier,
in-scope); four candidates did not (two have no behaviour to assert, two are
speculative / future-effort fog). The graduating implementation ticket is
the last frontier ticket on this map's browser track.

### Graduates

- **New task ticket (child of map):** "Add browser tests for responsive
  layout + error-state toast" — writes 1 invariant test (responsive layout
  at `< 768px`, `invariants.rs`, no spec) + 1 behaviour test (error toast
  on 500 swap error, `behaviour.rs`) + 1 new `docs/specs/browser.md` scenario
  (16.7, error toast). Blocked by nothing (ticket 07 landed; STRATEGY.md
  already allows `behaviour.rs` tags + `invariants.rs` named exemption).

### Not graduated (for the record)

- htmx swap transitions — no behaviour to assert.
- Connection-status transitions — feature doesn't exist.
- Accessibility — fog, future UI / a11y effort.
- Empty-state rendering — state never occurs.
