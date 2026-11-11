# External research — verifying server-driven HTMX UIs

Scope: how mature projects verify HTMX/server-rendered UIs. Sources are primary
(official docs, library source, issue trackers, first-party test suites) or
practitioner PRs with concrete code. Every claim carries an inline link. Claims
I could not source are listed in "Could not source".

Repo constraint that shapes every cost line: Rust + fresh-Chromium
`playwright-rs 0.9.0` (`Cargo.lock`), no Node toolchain, polling-based waits in
`tests/test_utils/wait.rs`.

---

## Area 1 — Request-level verification (the HTTP contract)

**Pattern.** Simulate the requests htmx sends and assert returned HTML and
response headers, without a browser. This is the maintainer-endorsed default.

**Who uses it.**

- HTMX's own creator (1cg / Carson Gross) on the canonical "Unit testing Htmx?"
  question: "for day to day unit tests, I would lean towards the first approach
  [server-side functional/controller tests] because it will be more stable and
  'functional' and then use the second approach for integration testing."
  [stackoverflow.com/questions/67133382](https://stackoverflow.com/questions/67133382/unit-testing-htmx)
  (answer by user `1cg`, score 9).
- django-htmx test helpers: `HTMX_HX_Request` / `HTTP_HX_Trigger_Name` headers
  set through Django `RequestFactory`, then `request.htmx = HtmxDetails(request)`
  [github.com/bigskysoftware/htmx/issues/452](https://github.com/bigskysoftware/htmx/issues/452).
- django-htmx's own suite tests only HTTP helpers (`HttpResponseStopPolling`,
  `retarget`, `reswap`) with `SimpleTestCase`/`RequestFactory` — no browser
  [raw tests/test_http.py](https://raw.githubusercontent.com/adamchainz/django-htmx/main/tests/test_http.py).
- Django's own docs: `RequestFactory` tests view functions "directly, bypassing
  the routing and middleware layers"; the test client "is not intended to be a
  replacement for Selenium or other 'in-browser' frameworks"
  [docs.djangoproject.com](https://docs.djangoproject.com/en/stable/topics/testing/tools/).
- Rails: `ActionDispatch::IntegrationTest` with `assert_dom`/`assert_select`,
  "focus ... on testing how several parts of an application interact"
  [guides.rubyonrails.org/testing.html](https://guides.rubyonrails.org/testing.html).
- Rust/axum: no first-party axum+htmx testing guide found (see "Could not
  source"). The repo already does this tier and asserts HX-Retarget/HX-Reswap/
  HX-Refresh (per task context).

**Covers / misses.** Covers the server contract: status, headers, fragment HTML.
Misses client-side wiring by construction. A practitioner issue states the miss
plainly: the Starlette `TestClient` "returns partial HTML but never runs the
HTMX swap / hx-target / hx-trigger / polling — so a broken hx-target, a JS
console error, or a bad swap ships green"
[mshirel/song-history#505](https://github.com/mshirel/song-history/issues/505).

**Cost for this repo.** Low. Existing HTTP tests already carry the htmx
request/response headers. The open question is *which* browser tests should
demote to this tier, not whether the tier exists.

---

## Area 2 — Browser E2E with explicit readiness

### 2a. Waiting on htmx lifecycle events (`afterSettle`)

This is the single most-corroborated pattern, and three independent projects
implement it to fix exactly the repo's flake.

| Project | Mechanism | Why (quoted) |
|---|---|---|
| fintrack PR #8 | `page.evaluate` installs a `htmx:afterSettle` counter; snapshot baseline; `wait_for_function` on count > baseline | "htmx wires those triggers while processing the swap, which happens *after* the /cell response is received. So `expect_response('/cell')` returning does NOT mean the editor is interactive — selecting/typing in that gap fires a change event that htmx isn't listening for yet, the POST never goes out, and the test times out." [mrdefenestrator/fintrack#8](https://github.com/mrdefenestrator/fintrack/pull/8) |
| NextSlope-10x PR #29 | `page.addInitScript` installs counter; `awaitNextAfterSettle(baseline)` | "processNode runs in doSettle — after `htmx:afterSwap` but before `htmx:afterSettle`), so a click landing in that window is a silent no-op. Condition-based wait, not a sleep." [wbiniecki/NextSlope-10x#29](https://github.com/wbiniecki/NextSlope-10x/pull/29) |
| manja PR #89 | `document.body.addEventListener('htmx:afterSettle', …)` counter + `WaitForFunction` | "aria-expanded changes before the sidebar outerHTML swap settles. Wait for the replacement control to finish HTMX processing before activating it." [araihu/manja#89](https://github.com/araihu/manja/pull/89) |

Two of three arm the counter *before* the action and wait for the counter to
cross a baseline — this closes the race where the swap completes between arming
and waiting.

### 2b. The "console-signal with targetId" readiness protocol

**Independent evidence exists for a console-signal, but not for targetId
scoping.** A maintained gist by Khalid Abuhakmeh registers
`document.body.addEventListener('htmx:afterSettle', () => console.log('playwright:continue'))`
and waits with `page.WaitForConsoleMessageAsync` matching that exact string
[gist](https://gist.github.com/khalidabuhakmeh/f2d99c1d2ca5bd1ee1b8bdf003e0aad7).
That is a bare global sentinel, not a target-scoped one. I found **no** source
using a `targetId` on `htmx:afterSettle` as a readiness token, and the prior
survey's Stack Overflow citation is invalid (it concerns a malformed trigger
name). Treat "targetId-scoped console signal" as unsourced.

Note: target-scoping is *possible* in principle — `htmx:afterSettle` exposes
`detail.elt` = "the updated element" [htmx.org/events](https://htmx.org/events/)
— but no practitioner source does it.

For this repo the console variant is also moot as a mechanism: playwright-rs
0.9.0 exposes no console-message wait or event (`wait_for_console_message` /
`on_console_message` are absent from the crate source), so a bridge would have
to signal through the page itself and be read via `evaluate_value` polling.

### 2c. `waitForResponse` / `waitForURL` on the htmx request URL

Widely used, but as a *completion* signal, not an *interactivity* signal.

- kazurayam's htmx+Hono+Playwright app uses
  `page.waitForResponse(/\/random/, { timeout: 10000 })` before typing
  [kazurayam issue #23](https://github.com/kazurayam/htmx-app-in-typescript-with-hono-playwright-on-bun/issues/23).
- ocramz/htmx-intersect's accepted E2E suite: "Tests use deterministic waits
  (`waitForFunction`, `waitForResponse`) rather than fixed timeouts"
  [ocramz/htmx-intersect#2](https://github.com/ocramz/htmx-intersect/pull/2).
- IBL5 PR #1805 is a systematic sweep replacing `waitForTimeout`, `networkidle`,
  and `waitForNavigation` with `toBeVisible`/`toBeAttached`/`toHaveAttribute`/
  `waitForResponse`/`waitForURL` [a-jay85/IBL5#1805](https://github.com/a-jay85/IBL5/pull/1805).
- Caution, from the fintrack post-mortem: a response arriving does **not** mean
  the swapped element is wired (see 2a quote). DixieData likewise recommends
  "a `waitForResponse(hx-post)` before the `waitForFunction`" *plus* the DOM
  assertion, not instead of it
  [valueforvalue/DixieData#709](https://github.com/valueforvalue/DixieData/issues/709).

### 2d. `networkidle`, and why htmx polling breaks it

Playwright marks it `DISCOURAGED`: it "consider[s] operation to be finished when
there are no network connections for at least 500 ms"
[playwright.dev/docs/api/class-page](https://playwright.dev/docs/api/class-page#page-wait-for-load-state).
With five containers on `hx-trigger="load, every Ns"` (2s/4s/5s/5s/2s), a
500 ms quiet window never occurs, so `networkidle` can only time out. IBL5
removed it from every E2E spec, replacing it with DOM/response assertions
[a-jay85/IBL5#1805](https://github.com/a-jay85/IBL5/pull/1805). `playwright-rs`
0.9.0 exposes `WaitUntil::NetworkIdle`; do not use it here.

### 2e. Disabling animations

Not sourced for htmx. Playwright's screenshot assertions accept an `animations`
option, but that is screenshot-scoped, and `playwright-rs` 0.9.0 exposes it only
through `to_have_screenshot`'s builder. No htmx-specific
animation-disable recipe found.

### 2f. What "htmx has finished processing this element" means operationally

`htmx:afterSettle` fires only after the settled tasks run, and those tasks
include `makeAjaxLoadTask`, whose body calls `processNode(child)` — the function
that attaches trigger listeners. Source: `dist/htmx.js` at tag `v1.9.10`,
`makeAjaxLoadTask` lines 899–906 (`processNode(child)` then
`triggerEvent(child, 'htmx:load')`), pushed as a settle task at line 924, run in
`doSettle` at lines 3644–3646, with `htmx:afterSettle` fired at lines 3647–3651
[htmx v1.9.10 dist/htmx.js](https://github.com/bigskysoftware/htmx/blob/v1.9.10/dist/htmx.js#L3632).
Operationally: "processed" = listeners attached = `processNode` done = at or
before `htmx:load`, and definitely by `htmx:afterSettle`.

---

## Area 3 — Headless-component / JSDOM tier

**Observed, but rare for htmx, and practitioners themselves say it cannot cover
swaps.**

- Vitest supports `jsdom` and `happy-dom` environments
  [vitest.dev/guide/environment](https://vitest.dev/guide/environment).
- A htmx frontend using happy-dom exists: LumeWeb/s3-server PR #17 ships
  "22 vitest tests (MSW + happy-dom)" for a Vite + Alpine + htmx panel
  [LumeWeb/s3-server#17](https://github.com/LumeWeb/s3-server/pull/17).
- The strongest evidence is an explicit boundary statement. power-map has 17
  Vitest suites under happy-dom simulating "intricate UI (typeahead comboboxes,
  merge modals, cardstack reorder, focus-scroll guards)" and deliberately adds a
  real-browser smoke tier for "what simulation can't catch (real focus, real
  scroll, real HTMX swap behavior)"
  [CannObserv/power-map#368](https://github.com/CannObserv/power-map/issues/368).
  Its design note also says the happy-dom suites "simulate these" flows while
  "current tests assert on partial-response HTML but never on actual swap
  behavior, hx-boost navigation, or flash-trigger event handling"
  [CannObserv/power-map#300](https://github.com/CannObserv/power-map/issues/300).

**Known limitation.** jsdom does no layout or rendering: "Layout: the ability to
calculate where elements will be visually laid out as a result of CSS, which
impacts methods like `getBoundingClientRects()` or properties like `offsetTop`"
is listed as unimplemented
[jsdom README](https://github.com/jsdom/jsdom#unimplemented-parts-of-the-web-platform).
It also cannot predict asynchronous script completion.

**Cost for this repo.** High and unjustified. No Node toolchain; practitioners
report the tier cannot observe htmx swap behavior anyway.

---

## Area 4 — HTMX's own guidance

**There is no testing documentation.** Maintainer-opened issue #452 "Docs: How
to test a htmx based application" is closed with no docs added; the thread
carries the only substantive guidance
[bigskysoftware/htmx#452](https://github.com/bigskysoftware/htmx/issues/452).

**Maintainer position (Carson Gross).** Prefer server-side functional/controller
tests for day-to-day stability; use client JS tests with a mocked server for
integration. He names the exact tooling: "The htmx test suite uses chai.js &
mocha.js ... and sinon.js to mock out the server side"
[stackoverflow.com/questions/67133382](https://stackoverflow.com/questions/67133382/unit-testing-htmx).

**How htmx's own suite exercises swaps.** 1.9.10 uses Mocha/Chai/sinon
(`sinon.fakeServer` + `server.respond()`), and disables history plus the settle
delay for tests via a meta tag:
`<meta name="htmx-config" content='{"historyEnabled":false,"defaultSettleDelay":0}'>`
[test/index.html @ v1.9.10](https://github.com/bigskysoftware/htmx/blob/v1.9.10/test/index.html).
Master's `test/index.html` still carries the same meta config. PR #3273 migrated
the framework to web-test-runner plus Playwright 3-way browser testing
[bigskysoftware/htmx#3273](https://github.com/bigskysoftware/htmx/pull/3273).

**Config knobs relevant to testability.**

- `historyEnabled` — "defaults to true, really only useful for testing".
- `historyCacheSize` — "can be set to 0 to avoid storing any HTML in the
  localStorage cache".
- `defaultSettleDelay` — defaults to 20; load-bearing for listener timing
  (Gap A).
- `attributesToSettle` — defaults to `["class", "style", "width", "height"]`.
  [htmx.org/docs/#config](https://htmx.org/docs/#config)

**Debugging guidance (useful for triage).** `htmx.logAll()`, and the
console-only `monitorEvents(htmx.find("#theElement"))`; breakpoints in
`issueAjaxRequest()` / `handleAjaxResponse()`
[htmx.org/docs](https://htmx.org/docs/#debugging).

**Cost for this repo.** Low and immediately applicable: the suite-level
`defaultSettleDelay:0` precedent is the cheapest structural knob (see Gap A),
though it removes the settling *transition* the repo may want at runtime, so it
belongs in test config, not production.

---

## Area 5 — Eliminating timing from verification

**Playwright's own model.** Actions run actionability checks ("Visible, Stable,
Receives Events, Enabled, Editable") before acting, and web-first assertions
auto-retry "until the condition is met, similarly to auto-waiting before
actions" [playwright.dev/docs/actionability](https://playwright.dev/docs/actionability),
[playwright.dev/docs/best-practices](https://playwright.dev/docs/best-practices).
Cypress matches: arbitrary `cy.wait(<number>)` is an anti-pattern, flagged at
lint time [docs.cypress.io/api/commands/wait](https://docs.cypress.io/api/commands/wait).

**The gap no framework closes.** Actionability knows nothing about
application-level listener attachment. That is why every sourced fix is an
event-driven bridge, not a smarter poll:

- Event-listener bridge, armed before the action: install a counter via
  `addInitScript` or `evaluate`, snapshot the baseline, act, then wait for the
  counter to advance
  ([NextSlope-10x#29](https://github.com/wbiniecki/NextSlope-10x/pull/29),
  [fintrack#8](https://github.com/mrdefenestrator/fintrack/pull/8),
  [manja#89](https://github.com/araihu/manja/pull/89)).
- `waitForResponse` / `waitForURL` as event-driven completion for requests that
  actually fire ([IBL5#1805](https://github.com/a-jay85/IBL5/pull/1805),
  [ocramz/htmx-intersect#2](https://github.com/ocramz/htmx-intersect/pull/2)).
- Playwright's `page.waitForFunction` is the usual vehicle
  [playwright.dev/docs/api/class-page](https://playwright.dev/docs/api/class-page#page-wait-for-function).

**Repo-specific binding limit.** `playwright-rs` 0.9.0 has **no**
`wait_for_function` and **no** `wait_for_selector`. It does provide
`page.evaluate` / `evaluate_value`, `page.add_init_script`, `page.on_response`,
`page.on_request`, and an `expect()` assertion wrapper that polls every 100 ms
with a 5 s default timeout (source: local crate
`~/.cargo/registry/.../playwright-rs-0.9.0/src/protocol/page.rs` and
`src/assertions.rs`; published docs
[docs.rs/playwright-rs/0.9.0](https://docs.rs/playwright-rs/0.9.0/playwright_rs/assertions/fn.expect.html)).
So the counter bridge must be a Rust poll loop over `evaluate_value` — the repo
already has `wait_for_condition_async` for exactly this shape.

---

## Gap A — listener-attachment timing (direct answer)

**Wait for a settle on the target before interacting. `htmx:afterSwap` is too
early; listeners attach after it.**

Evidence chain in `dist/htmx.js` @ `v1.9.10`:

| Step | Line | Fact |
|---|---|---|
| `defaultSettleDelay: 20` | 55 | settle is deferred 20 ms by default |
| `swap … insertNodesBefore` | 917–924 | insertion pushes `makeAjaxLoadTask(child)` into `settleInfo.tasks` |
| `makeAjaxLoadTask` | 899–906 | body calls `processNode(child)` (attaches trigger listeners), then fires `htmx:load` |
| `htmx:afterSwap` fires | 3632 | **before** `doSettle` runs |
| `doSettle` runs tasks | 3644–3646 | this is where `processNode` (listener attach) happens |
| `htmx:afterSettle` fires | 3647–3651 | after the tasks |
| deferral | 3682–3683 | `setTimeout(doSettle, swapSpec.settleDelay)` |

Source:
[htmx v1.9.10 dist/htmx.js](https://github.com/bigskysoftware/htmx/blob/v1.9.10/dist/htmx.js#L3632).
Docs confirm the ordering: "the DOM is settled" follows "A settle delay is done
(default: 20ms)" in Request Order of Operations, and `hx-swap` states "The
default settle delay is 20ms"
[htmx.org/docs/#request-operations](https://htmx.org/docs/#request-operations),
[htmx.org/attributes/hx-swap](https://htmx.org/attributes/hx-swap/).

**Consequence.** `htmx:afterSettle` on the target is the correct readiness
condition. Practitioners agree, with the same causal explanation
([fintrack#8](https://github.com/mrdefenestrator/fintrack/pull/8),
[NextSlope-10x#29](https://github.com/wbiniecki/NextSlope-10x/pull/29)).
`htmx:load` fires slightly earlier, per swapped child, and also at init on
`body` (line 3897), so it is a valid but noisier anchor. The repo's hypothesis —
"listeners attach after the element becomes visible, with no readiness wait" —
is **correct and confirmed by source**.

**One caveat for this repo.** A document-level `htmx:afterSettle` counter cannot
say *which* swap settled, and five pollers fire constantly. Arm a baseline
immediately before the interaction and wait for the counter to *increase* — the
pattern two of the three practitioner PRs use.

---

## Gap B — structural fixes vs waits

**No `htmx.config` knob or `hx-*` attribute makes a change-triggered POST
"impossible to not fire". Two of the named knobs can actively *suppress* it.**

| Knob | Documented effect | Does it help? |
|---|---|---|
| `hx-sync` | Default strategy `drop` = "drop (ignore) this request if an existing request is in flight" [docs](https://htmx.org/attributes/hx-sync/) | **No — can cause the miss.** If a poll shares the sync element, the POST is silently dropped |
| `hx-indicator` | Adds `htmx-request` class "for the duration of the request" [docs](https://htmx.org/attributes/hx-indicator/) | Only an in-flight signal; no dispatch guarantee |
| `changed` modifier | Fires "only if the value of the element has changed" [docs](https://htmx.org/attributes/hx-trigger/) | **No — makes non-firing more likely.** Source compares against `lastValue` captured at initNode (line 2113) and at listener setup (lines 1486–1489, compare at 1531) |
| `hx-preserve` | "keep an element unchanged during HTML replacement … preserved by id when htmx updates any ancestor element" [docs](https://htmx.org/attributes/hx-preserve/) | **Yes — structural.** If the `select` is never destroyed, its listener never needs re-attaching |
| test-config `defaultSettleDelay:0` | Collapses the 20 ms wait to one task tick [htmx docs](https://htmx.org/docs/#config), as htmx's own suite does [test/index.html](https://github.com/bigskysoftware/htmx/blob/v1.9.10/test/index.html) | **Shrinks, does not remove** — `setTimeout(doSettle, 0)` still defers; listener attach stays asynchronous, so the settle-gate wait remains necessary |

**Sourced practitioner root causes for "the form post sometimes doesn't fire".**
(a) listener-attachment gap — the dominant, best-documented cause
([fintrack#8](https://github.com/mrdefenestrator/fintrack/pull/8),
[NextSlope-10x#29](https://github.com/wbiniecki/NextSlope-10x/pull/29));
(b) a full navigation replacing swap-in with focus loss (manja:
"causing full navigation and focus to move to `body`"
[manja#89](https://github.com/araihu/manja/pull/89));
(c) element replacement racing activation before settle (same PR).

**Structural recommendation.** `hx-preserve` on the posture selects, or stable
ids that let htmx settle attributes, removes the race by construction. Its
documented caveats: it can relocate elements, and `hx-swap="none"` with an
`hx-preserve` element risks losing it [hx-preserve docs](https://htmx.org/attributes/hx-preserve/).

---

## Gap C — `afterSwap` vs `afterSettle` vs `load` (direct answer)

| Event | htmx docs wording | Fires relative to DOM insert / settle | Right anchor? |
|---|---|---|---|
| `htmx:afterSwap` | "triggered after new content has been swapped into the DOM" | DOM inserted; listeners **not** attached | No |
| `htmx:afterSettle` | "triggered after the DOM has settled" | after settle tasks, i.e. after listener attach | **Yes** (target-scoped) |
| `htmx:load` | "triggered when a new node is loaded into the DOM by htmx. Note that this event is also triggered when htmx is first initialized, with the document body as the target" | immediately after `processNode` per swapped child (source line 905) | Valid, element-scoped; noisier |

Source: [htmx.org/events](https://htmx.org/events/), with ordering grounded in
[dist/htmx.js @ v1.9.10](https://github.com/bigskysoftware/htmx/blob/v1.9.10/dist/htmx.js#L3632)
(`afterSwap` line 3632 → tasks line 3644 → `afterSettle` line 3651) and
`makeAjaxLoadTask` lines 899–906.

`htmx:afterSettle`'s detail exposes `elt` = "the updated element", so
target-scoping is documented-possible; no practitioner source uses it (Area 2b).

---

## Could not source

- A **targetId-scoped** console-signal readiness pattern. The gist exists but is
  a bare global string, not scoped. The prior survey's citation is invalid.
- Any htmx-specific **animation-disabling** recipe.
- A **first-party axum/htmx testing guide**. Axum+htmx projects exist; none was
  a maintained testing reference.
- Evidence that **JSDOM/happy-dom can exercise htmx 1.x attribute processing**.
  Practitioners use those tiers for component JS and explicitly exclude swap
  behavior.
- A **Rails htmx request-spec** example. Rails guides cover integration tests
  generally, with no htmx-specific content.
- A maintainer statement on **listener-attachment timing**. Issue #1799
  ("How quickly do HTMX elements bind to document events?") is closed "no longer
  relevant"; #2839's answer (set `defaultSettleDelay` to 0) is a workaround, not
  a timing statement.

---

## Recommendation shortlist

Ranked for "robust verification by construction" in this repo.

| # | Approach | Cost | Coverage / tradeoff |
|---|---|---|---|
| 1 | **Settle-gate the swap→interact step.** Add one `add_init_script` counter on `htmx:afterSettle`; expose `wait_for_htmx_settle(page)` implemented as a Rust poll over `evaluate_value` (reuse `wait_for_condition_async`). Snapshot baseline before the select, wait for increase. | Low — one init script + one helper in `tests/test_utils/` | Closes the exact race; matches three independent practitioner fixes. Requires each call site to arm a baseline |
| 2 | **`hx-preserve` (or stable ids on) the posture selects** so the change listener survives the swap. | Low–medium — template + server fragment contract | Removes the race structurally; no wait needed. Caveats: relocation, `hx-swap="none"` interaction |
| 3 | **Demote the posture autosave contract to the HTTP tier.** Assert the POST/response fragment with the htmx request headers; keep only "the browser wires the change POST" in the browser tier, behind #1. | Low — reuses existing HTTP harness | Cuts Chromium round-trips (the browser binary dominates gate wall time — ≈2.5 min of the ≈3 min standard gate, per build docs). Deliberately does not test client wiring |
| 4 | **`waitForResponse`-style completion helper** via `page.on_response`/`on_request` for flows that do emit a request. | Low | Proves "the request went out"; does **not** prove the element was wired (see Gap B) |
| 5 | **Ban `networkidle` in this suite.** Five `every Ns` pollers make it unsatisfiable; Playwright marks it DISCOURAGED. | Trivial | Removes a whole class of guaranteed timeouts |
| 6 | **Do not add a JSDOM/happy-dom tier.** | — | No Node toolchain; practitioners report it cannot see swap behavior |

One-line trading summary: #1 is the direct, low-cost fix for the known flake; #2
is the only true "make it impossible to not fire" option; #3 is the only option
that removes the test entirely. #1 and #3 compose, and #2 can replace #1 if the
select need not be re-rendered on each swap.

## Per-option assessment: performance, reliability, maintainability

Added on follow-up request. Labels: [S] sourced/measured, [I] inferred,
[G] guess.

| Option | Performance | Reliability | Maintainability |
|---|---|---|---|
| Baseline: DOM polling | Browser tier ≈2.5 min for 26 tests ≈ 5.8 s/test [I]; each wait pays up to one poll interval (100–500 ms) on top of the condition's true latency [S: `wait.rs` intervals] | Two recorded flake classes: CDN stall (fixed) and the posture no-fire race; polling cannot tell settled from mid-swap DOM [S: flake-investigation] | 9 polling helpers in `wait.rs`; flake triage needs engine-log + DOM-dump forensics [S] |
| 1. Settle-gate | Wait returns at settle instead of at the next poll tick; bounded by server round-trip + ≤20 ms settle + Rust poll granularity [S: `defaultSettleDelay=20`]. Suite-level gain modest — cost is dominated by spawn, not waits [I] | Closes the swap→interact race; two of three practitioner fixes arm the baseline *before* acting so the swap cannot be missed [S]. Residual risk: call-site discipline | +1 helper +1 init script; replaces polling helpers over time; every call site must arm a baseline |
| 2. `hx-preserve` on posture selects | Neutral to slightly faster swaps (preserved subtree is not re-created) [I] | Removes the race on the app side — the listener never needs re-attaching [S: `handlePreservedElements`]. Documented caveats: relocation, `hx-swap="none"` loss risk [S: docs] | Template + server-fragment contract change; future editors must understand preserve semantics; posture-only scope |
| 3. Demote posture autosave to HTTP tier | In-process `oneshot` tests: ms each [I]; the whole non-browser integration tier runs ≈20 s vs browser ≈2.5 min [S: build docs] | No browser timing at all for the demoted assertions; a thin browser test behind #1 keeps the wiring covered | Grows `tests/http/`; the behaviour lives in two tiers (wiring vs contract); spec/test-tag bookkeeping [S: map fog] |
| 4. `waitForResponse` helper | Event-driven; no added latency [S: `page.on_response` exists] | Proves dispatch, not wiring — response ≠ interactive (fintrack post-mortem) [S]; complement to #1, not a replacement | One more helper; only useful where a request actually fires |
| 5. Ban `networkidle` | Removes a guaranteed-timeout class; no per-test cost | Five `every Ns` pollers make a 500 ms quiet window unsatisfiable [S] | One-line convention; enforcement is review-time or a later architecture test |
| 6. JSDOM tier | Survey estimates 10–50 ms/test — unsourced [G]; moot here | Cannot observe swap behavior (practitioners exclude it) [S]; `getBoundingClientRect` unsupported — `showEditForm` depends on it [S] | Worst: introduces a Node/Vitest toolchain into a Rust-only repo |
| Config profile `defaultSettleDelay:0` | Collapses the 20 ms settle wait per swap to one task tick [S] | Shrinks the race window, does not remove it [S: `setTimeout(doSettle, 0)`]; masks the settle transition under test | One injected script; htmx's own-suite precedent [S] |
| De-serialize browser binary (post-redesign) | Potential large cut: 26 tests across 8 cores instead of 1 [G — untried; 41-port ceiling, Chromium CPU contention] | Safe only once the race is structurally closed — serialization was a stabilization knob [S] | Deletes a nextest override; simplification |
| Shared server/browser fixture | Amortizes spawn cost across tests [I] | State bleed between tests; fresh-per-test was chosen deliberately so no event ordering is assumed [S: flake-investigation] | Shared lifecycle infra; invariants.rs already runs this shape [S] |

**Speed levers, ranked.** The settle-gate buys correctness, not speed. The
speed story is: demote what does not need a browser (#3), then de-serialize the
browser binary once the race is structurally impossible, then optionally share
the fixture (ticket 04, question 4).

---

## Verification notes (session, 2026-09-15)

Load-bearing claims re-verified against ground truth after drafting:

- **Gap A chain, vendored artifact.** `assets/htmx.min.js` (the shipped 1.9.10)
  contains `defaultSettleDelay:20`, fires `htmx:afterSwap` immediately after
  insertion, then defers a closure via `setTimeout(s, settleDelay)` that runs
  `n.tasks` and only then fires `htmx:afterSettle`.
- **Gap A chain, unminified source.** `dist/htmx.js @ v1.9.10`:
  `makeAjaxLoadTask(child)`'s body is `processNode(child); …;
  triggerEvent(child, 'htmx:load')`, and `insertNodesBefore` pushes it into
  `settleInfo.tasks` — listener attachment is a settle task, and `htmx:load`
  fires inside that same task. `handlePreservedElements` swaps the old element
  (listeners intact) back in — the mechanism behind Gap B's `hx-preserve` fix.
- **playwright-rs 0.9.0, local crate source.** No `wait_for_function` or
  `wait_for_selector` anywhere in `src/`; `add_init_script`, `on_response`, and
  `evaluate_value` present in `src/protocol/page.rs`; `expect()` defaults are
  5 s timeout / 100 ms poll (`src/assertions.rs`).
- **htmx test harness.** `test/index.html @ v1.9.10` carries
  `<meta name="htmx-config" content='{"historyEnabled":false,"defaultSettleDelay":0}'>`
  verbatim; Mocha/Chai/sinon confirmed.
- **Practitioner PRs exist and corroborate.** fintrack#8 ("waits for htmx's
  `afterSettle` so the swapped-in `<select>`'s hx-trigger is wired before
  selecting"), NextSlope-10x#29 ("deterministic `htmx:afterSettle` wait closes
  the post-swap listener-rebind race"), manja#89 ("test counts
  `htmx:afterSettle` events and waits…"). The deeper mechanism wording quoted
  for NextSlope-10x#29 comes from its PR description; the fetched summary page
  confirms the race-fix claim but not that verbatim text.
- **Maintainer quote, via Stack Exchange API** (SO blocks direct fetches).
  1cg answer (score 9) verbatim: "for day to day unit tests, I would lean
  towards the first approach because it will be more stable and 'functional'"
  plus the chai/mocha/sinon sentence.

Corrections made during verification:

- The shortlist originally said "browser tier is ~87% of gate wall time". That
  figure had no source in this repo's assets. Replaced with the supported claim
  (browser binary ≈2.5 min of the ≈3 min standard gate, per build docs).
- Playwright's `networkidle` DISCOURAGED annotation is cited from the docs URL
  but was not re-fetched this session. The recommendation does not depend on
  it: five `every Ns` pollers make a 500 ms quiet window unsatisfiable
  regardless of the label.
- The Gap B table originally said `defaultSettleDelay:0` makes `doSettle` run
  synchronously. The vendored source defers via `setTimeout(s, settleDelay)`;
  with 0 the wait becomes one task tick, not zero. The window shrinks, but
  listener attachment stays asynchronous — the settle-gate wait is still the
  structural fix. (Found while answering a follow-up; corrected above.)
