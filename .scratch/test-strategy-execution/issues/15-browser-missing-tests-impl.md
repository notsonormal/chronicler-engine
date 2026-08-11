# Add browser tests for responsive layout + error-state toast

Type: task (AFK)
Status: resolved
Assignee: agent (this session)
Blocked by: (none)

## Question

Forward-looking browser-tier gap fill from ticket 14's grilling. Two
sharp, in-scope, presentation-layer gaps were identified that have no
test. This ticket writes the tests — it does not change production code
or re-decide tier placement.

### Gap 1: Responsive layout at narrow viewport (invariant, no spec)

`assets/styles.css` declares `@media (max-width: 768px)` (flex-direction
column, `.story-log` / `.visual-sidebar` width 100%, story-log height
60%, visual-sidebar 40%) and `@media (max-width: 480px)` (header
flex-wrap, action-area column, full-width form controls). The app claims
responsive support. `test_no_horizontal_overflow` and
`test_element_positioning` assert desktop viewport only; nothing asserts
the responsive rules apply at narrow widths.

**Add to `tests/browser/invariants.rs`** (named exemption, no spec link,
test code is the definition — same shape as the other 7 invariants):

- `test_responsive_layout_under_768px` — set viewport width below 768px,
  assert `.main-container` computed `flex-direction` is `column` (desktop
  is `row` or default) and/or `.story-log` width is 100% of container
  rather than the desktop sidebar-split proportion. Use Playwright
  `set_viewport_size` (or `page.set_viewport_size`) then read
  `getComputedStyle` / `getBoundingClientRect`.
  - ponytail: one breakpoint test is the minimum that proves the
    @media rule applies; second breakpoint (`< 480px`) is cosmetic
    detail (header wrap) — add only if the first test doesn't already
    prove the responsive machinery is wired.

### Gap 2: Error-state toast rendering (behaviour, specced)

`assets/index.html` registers a global `htmx:beforeSwap` listener: on
`evt.detail.isError`, it strips tags from `serverResponse` and calls
`showError(text || "Request failed")`, which sets
`#error-notification.textContent` and adds `.visible`; a 5s `setTimeout`
removes `.visible`. A 500 from POST /action (or any htmx swap error)
renders a toast that auto-hides after 5s. The toast rendering, class
toggle, and auto-hide are browser-only — no test asserts any of it.

**Add to `tests/browser/behaviour.rs`** (tagged against
`docs/specs/browser.md` scenario 16.7):

- `test_error_toast_on_action_failure` — trigger a 500 from POST /action
  (e.g. shut down the server mid-request, or POST a payload the handler
  rejects with 500 — pick whichever is cheapest in the existing
  `with_test_page` harness). Assert `#error-notification` gains `.visible`
  class and its text is non-empty. Do NOT wait 5s to assert auto-hide —
  that's flaky and slow; the `setTimeout(…, 5000)` is trivial JS, assert
  only that the toast appears.
  - ponytail: the auto-hide is a 5s `setTimeout`; asserting it would
    burn 5s per run and add flake for no authority gain. Skip it. If the
    auto-hide ever becomes load-bearing (e.g. user can dismiss early),
    add then.

**Add to `docs/specs/browser.md`** (new scenario 16.7):

- Given the dashboard is loaded and the action form is visible, When a
  POST /action returns a 500, Then the `#error-notification` element
  gains the `.visible` class and displays the error text from the
  response body.

### What this ticket does NOT do

- Does not touch production code (no JS / template / CSS changes). If a
  gap test reveals a bug (e.g. the toast doesn't fire on 500), file a
  separate ticket — this ticket writes tests for existing behaviour.
- Does not add accessibility, connection-status-transition, or
  empty-state tests (ticket 14 ruled: no behaviour to assert, or fog).
- Does not re-decide tier placement — STRATEGY.md already allows
  `behaviour.rs` SCENARIO tags + `invariants.rs` named exemption.

### Context

- [Ticket 14 resolution](14-browser-missing-tests.md) — the grilling
  that classified all 6 candidates and graduated this ticket.
- [Ticket 07 resolution](07-browser-tier-design.md) — behaviour /
  invariant split, `browser.md` spec shape.
- Tier rules in `tests/STRATEGY.md` — browser is
  presentation-only; `invariants.rs` named exemption; `behaviour.rs`
  tags allowed.

### Estimate

~3 story points: 2 tests + 1 spec scenario, all in existing files with
existing harness. No new infrastructure.

## Answer

Two browser tests landed + one new spec scenario (16.7). Suite green:
15/15 browser tests pass, validator 53 declared / 53 covered / 0 gaps /
0 orphans / 0 format violations, build + clippy clean.

### Gap 1: Responsive layout invariant

`tests/browser/invariants.rs::test_responsive_layout_under_768px` —
sets viewport to 500x800 via `page.set_viewport_size`, asserts
`.main-container` computed `flex-direction` is `column` (desktop default
is `row`). Proves the `@media (max-width: 768px)` rule in `styles.css`
is wired. Second breakpoint (`< 480px`, header flex-wrap) not tested —
ponytail: one breakpoint proves the responsive machinery; the second is
cosmetic detail.

### Gap 2: Error toast behaviour

`tests/browser/behaviour.rs::test_error_toast_on_action_failure`,
tagged `SCENARIO: 16.7`. Dispatches a synthetic `htmx:beforeSwap`
CustomEvent with `detail.isError=true` and `detail.serverResponse='Internal
server error'` on `document.body`, then asserts `#error-notification`
gains `.visible` and has non-empty `textContent`.

**Why synthetic, not a real 500:** the ticket offered two options —
route interception or a server-rejected payload. Both fail in this
harness:

- `route.fulfill` in this `playwright-rs` (0.9.0) version is empirically
  broken for BOTH status AND body. A `FulfillOptions` with `status: 500`
  and a non-empty body arrives at the page as status 200 with empty body
  — verified with a fetch probe (`/action/check` → `{status: 200,
  bodyLen: 0}`). The playwright-rs doc claims status works and only body
  is broken; that's inaccurate. With status 200, htmx fires beforeSwap
  with `isError: false`, so `showError` never runs.
- The real server has no path that returns 500 from `/action/check`
  without a production-code change (only `dispatch_action` Err or
  template-render failure → 500; mock backend doesn't fail). Production
  changes are explicitly out of scope for this ticket.
- Server-shutdown mid-request produces a connection error, which fires
  `htmx:responseError`, not `htmx:beforeSwap` — the app's listener
  wouldn't fire, so the toast wouldn't render. Not a valid trigger.

The app code under test is the body-level `htmx:beforeSwap` listener →
`showError` → `#error-notification.visible`. htmx's 500→`isError=true`
mapping is htmx's contract, not the app's. The synthetic event
dispatches exactly the event htmx would dispatch on a real 500, so the
listener + showError + class toggle + textContent path is exercised
end-to-end. The test is honest about this in its comment.

### Spec

`docs/specs/browser.md` scenario 16.7 added:

> Given the dashboard is loaded and the action form is visible,
> When a POST /action returns a 500,
> Then #error-notification gains the .visible class
> And #error-notification displays non-empty error text derived from the response body

Header count updated (6 → 7 browser-only interactions).

### What was NOT done

- No production code touched (no JS / template / CSS changes) — per
  ticket scope.
- Auto-hide `setTimeout(…, 5000)` not asserted — ponytail: 5s burn + flake
  for no authority gain; the setTimeout is trivial JS. Comment notes:
  add when auto-hide becomes load-bearing (e.g. user-dismissible early).
- `< 480px` responsive breakpoint not tested — one breakpoint proves the
  machinery.
