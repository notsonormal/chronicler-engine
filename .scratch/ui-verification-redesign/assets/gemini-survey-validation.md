# Gemini deep-search survey — validation against the repo

Analysis of
[HTMX Verification Testing Patterns.txt](./HTMX%20Verification%20Testing%20Patterns.txt)
(the survey) for ticket 05 of the UI-verification-redesign map. The survey is
23K with 24 cited sources; its source 5 is a repomix dump of this repo, so its
repo-specific claims are checkable directly.

**Headline verdict.** The survey's *architectural* observations are sound — the
three-tier model, the polling-vs-lifecycle diagnosis, and the targetId-scoped
readiness protocol are correct and worth adopting. Its *repo-specific* detail is
unreliable: several of its concrete claims about this repo no longer hold, and
the two claims carrying the most design weight needed real correction. Ticket 02 should
treat the survey as a source of patterns, not of facts about this repo.

Two of the errors share one cause: the survey describes an earlier state of
this repo. `wait_for_story_log_change` and `wait_for_element_text` were in
`wait.rs` until the 2026-07-25 refactor removed them; only `HX-Request`
simulation was never there at all. The survey is a stale snapshot, not a
fabrication — which is the actionable point: re-verify every repo fact against
today's code, and keep the patterns.

## 1. Fact-check against the code

Verdicts: **true** / **partly true** / **false** / **stale** (true of an
earlier state of the repo) / **wrong-repo**.

| # | Survey claim | Verdict | Evidence |
|---|---|---|---|
| 1 | `wait.rs` contains `wait_for_story_log_change` and `wait_for_element_text` | **stale** | Neither name is in `tests/test_utils/wait.rs` today — 9 public fns, and the repo already records the names as nonexistent at `docs/plans/fix-test-police-findings-for-ticket-10-options-agent-pipelin.md:57`. Both existed until the 2026-07-25 refactor (`d1937ba`) deleted them |
| 2 | Helpers poll every 200ms or 500ms | **partly true** | 200ms in `wait_for_llm_idle`, `wait_for_element_children`, `wait_for_status_ready`, `wait_for_element_persist`; 500ms in `wait_for_status_ready_or_error`; 100ms in `wait_for_status_generating` (`tests/test_utils/wait.rs:25,55,121,176,153,193`) |
| 3 | `#story-log` polls `every 2s` | **true** | `assets/index.html:36` |
| 4 | `#status-display` polls `every 5s` | **true** | `assets/index.html:79` |
| 5 | `.visual-sidebar` polls `every 5s` | **true** | `assets/index.html:42` |
| 6 | Implicit: those are the three polling containers | **partly true** | Two more exist: `#options-dock` `every 2s` (`:52`) and `.llm-messages-panel` `every 4s` (`:119`). Four distinct intervals: 2s, 4s, 5s, 5s |
| 7 | Client JS `onStatusPoll(el)` exists | **true** | `assets/index.html:158`. Maps `narrating`/`quantifying`/`generating-event` to labels and calls `setButtonState` — as described |
| 8 | Client JS `onStatusPoll` also maps `options` | **false (omission)** | `assets/index.html:167-175` maps a fourth phase, `options` → "Generating options...". The survey's JSDOM example asserts only three |
| 9 | Client JS `showEditForm(id)` exists | **true** | `assets/index.html:232` |
| 10 | Client JS `cancelEdit()` exists | **true** | `assets/index.html:268` |
| 11 | `showEditForm` uses `getBoundingClientRect().height` to size the textarea | **true** | `assets/index.html:239`. So the survey's JSDOM-incompatibility point (item 15) genuinely applies here |
| 12 | Client JS `saveActionArea()` exists | **true** | `assets/index.html:356` |
| 13 | Slash-command menu with `/impersonate` and `/guide` | **partly true** | `SLASH_COMMANDS` at `assets/index.html:450` holds **three** entries — `/impersonate`, `/guide`, `/options` |
| 14 | `htmx:beforeSwap` error-toast listener | **true** | `assets/index.html:196`; `showError` at `:185` |
| 15 | Client JS `restoreLlmExpandedState()` | **unmentioned** | `assets/index.html:713`, wired at `:121`. Survey omits it |
| 16 | htmx pinned at **1.9.10** | **true** | Vendored `assets/htmx.min.js` contains `version:"1.9.10"`; tag at `assets/index.html:7` |
| 17 | Polling helpers are the *flakiness* mechanism | **partly true** | The incident record puts the observed failures elsewhere: a CDN fetch stall (fixed, now vendored) and the posture `hx-post` no-fire (`assets/flake-investigation-2026-09-13.md`). Polling is a structural weakness, not the recorded cause |
| 18 | HTTP tier simulates HTMX headers (`HX-Request`, `HX-Target`, `HX-Trigger`) | **false** | No `HX-*` request header is set anywhere in `tests/`. `tests/http/test_helpers.rs` sets only `content-type` (`:76,94,176`). Nothing in `src/` reads `HX-Request`, and there is no `is_htmx`-style branch — the app has no full-page/partial fork to test |
| 19 | HTTP helpers assert `HX-Retarget` and `HX-Refresh` | **true** | Built at `src/adapters/driving/http/builders/headers.rs:21,24` (plus `HX-Reswap`); asserted at `tests/http/actions.rs:787,810,828`, `tests/http/games_switch.rs:41`, `tests/http/requires_migration/fragment.rs:610` |
| 20 | Ports allocated from 3010–3050 via file-based locking | **true** | `tests/test_config.json` `port_range` 3010–3050; `get_available_port` at `tests/test_utils/server.rs:367` |
| 21 | `capture_failure_state` writes `tmp/screenshots/` + `tmp/test_diagnostics/` | **true** | `tests/test_utils/browser.rs:154-205`, exactly those two directories |
| 22 | `with_test_page` waits on `#story-log .log-entry` | **true** | `tests/test_utils/browser.rs:87` — the shared precondition named in the map's Not-yet-specified |
| 23 | Askama templates / response headers / worlds posture form | **partly true** | Askama is real (`askama::Template` in every template module). The posture form is real but different: it posts per-field with `hx-post="…/posture"`, `hx-trigger="change"`, `hx-include="closest .posture-group"`, targeting `#world-posture-status` (`src/adapters/driving/http/worlds/templates/worlds.rs:84,90,96`) — the survey never mentions this form, which is the map's live bug |
| 24 | `waitUntil: 'networkidle'` "never resolves in Chronicler" | **not applicable** | No Playwright call in `tests/` uses any `WaitUntil` option; `page.goto(&url, None)` passes defaults. The named failure cannot occur here because the setting is never used |
| 25 | CSRF/form processing is validated in this repo's HTTP tier | **wrong-repo** | No CSRF anywhere in `src/` or `assets/`. It appears only inside the survey's generic Django paragraph |

**Claim 25 is the tell.** The point of naming a repo is to test the claim
against it. A shared CSRF assertion against a codebase with no CSRF is
made-up detail, and it sits beside the poll intervals, which are real. That mix
is why the survey reads as authoritative until checked.

### htmx.config defaults inside the vendored file

Read at `assets/htmx.min.js` and cross-checked against the unminified 1.9.10
`dist/htmx.js`:

| Knob | Survey default | Actual 1.9.10 | Verdict |
|---|---|---|---|
| `defaultSwapDelay` | 0 ms | 0 | **true** |
| `defaultSettleDelay` | 20 ms | 20 | **true** |
| `historyEnabled` | true | true | **true** |
| `timeout` | 0 (no limit) | 0 | **true** |
| `logAll` | false | `false` — `logAll` is a *function*, not a config key. `htmx.logAll()` arms a logger; the `logger` field defaults to `null` | **partly true** (right effect, wrong shape) |

So `htmx.config.defaultSettleDelay = 0` is a real, available knob in this
version. The survey's config table is the most reliable section it has.

## 2. Spot-check of the two loaded claims

### (a) htmx#2787 — leading newline suppresses swap/settle events

Survey's use: it advises that Askama partials must not emit leading newlines
before the root tag, or `htmx:afterSwap`/`htmx:afterSettle` never fire.

Checked against
[the issue](https://github.com/bigskysoftware/htmx/issues/2787):

- Title: "htmx:afterSwap & htmx:afterSettle do not fire when response partial
  begins with newlines". Opened 2024-08-03, **closed 2024-08-12**.
- Reporter's framing, verbatim: "*This happens on htmx 2.0.0/1, but worked fine
  on htmx 1.9.12.*"

So the bug is **(i) a 2.0.0/2.0.1 regression only, and (ii) already fixed**. This
repo pins 1.9.10, and the reporter says 1.9.12 was unaffected — 1.9.10 is five
patch releases below the line the reporter calls clean.

**Inferred** (the mechanism, from source rather than from the issue text): the
1.9.10 `swapOuterHTML` walk in `assets/htmx.min.js` handles a text node
predecessor with its `else` branch —

```js
if(i==null){n=u(t).firstChild}else{n=i.nextSibling}
r.elts=r.elts.filter(function(e){return e!=t});
while(n&&n!==t){if(n.nodeType===Node.ELEMENT_NODE){r.elts.push(n)}n=n.nextElementSibling}
```

`n.nextElementSibling` skips text nodes, so a leading `\n` cannot terminate the
walk early and cannot empty `settleInfo.elts`. That is the shape 2.0.0 lost.

**Verdict: the claim does not apply here.** It is doubly irrelevant — the
version is outside the affected range, and the recommended mitigation
(suppressing leading newlines in partials) guards against a defect this repo's
htmx does not have. Ticket 02 should not carry it forward as a live risk.

### (b) Stack Overflow 69538244 — "afterSettle not working with hx-trigger"

Checked via the Stack Exchange API (the HTML page is behind a 403).

- Question title: "htmx:afterSettle not working with hx-trigger". Asked
  2021-10-12, about **htmx 1.x**.
- The asker's markup: `<div id="product-gallery" hx-trigger="htmx:afterSettle" hx-get="{% url 'products' %}" hx-swap="outerHTML">` — the asker wrote the *event name* as a trigger string.
- Accepted answer: nothing to do with `afterSettle` firing. It is advice about
  HTTP redirects — "If you want a response which was triggered via htmx to do a
  full page reload, then you should not return a http redirect response (302)".
- Other answer: "AFAIK you can't use `afterSettle` like this:
  `hx-trigger="htmx:afterSettle"`" and recommends `hx-swap-oob` instead.

**Verdict: cite does not support the claim.** The survey lists this source among
"deterministic lifecycle hook integration" evidence. The thread is a user
mis-writing a trigger name, and its accepted answer concerns redirects. It
carries no evidence about `afterSettle` reliability, and the repo's own htmx
1.9.10 is not implicated.

## 3. Distilled patterns, and what each would cost here

Each row: what the pattern is, what applying it takes in this repo, and whether
it bears on **ticket 03** (root-cause the posture no-fire) or **ticket 04**
(decide the verification design).

| Pattern | What it is | What applying it takes here | Bears on |
|---|---|---|---|
| **Request-header contract tier** | Simulate the headers htmx sends (`HX-Request`, `HX-Target`) so the server picks partial vs. shell | Low value as stated: the server has no shell/partial fork and never reads `HX-Request`, so simulating it tests nothing. The real, present contract is the **response** headers, and `tests/http/` already asserts `HX-Retarget`/`HX-Reswap`/`HX-Refresh` (`tests/http/actions.rs:787`) | 04 — reframes the tier as "assert response headers", which already exists; no new work |
| **Lifecycle-event readiness protocol** | Replace DOM polling with a `document.body` listener that emits `playwright:htmx_settled:<targetId>` on `htmx:afterSettle`; the test blocks on that message | New helper in `tests/test_utils/browser.rs` + a `wait_for_htmx_target_settle`; the existing 9 polling helpers in `wait.rs` get re-examined against it. The targetId scoping is what makes it survive background polling | 03 and 04 — the direct candidate answer for both |
| **TargetId-scoped signals** | Include the target element id in the readiness signal so background interval swaps cannot satisfy a lock meant for a user action | Falls out of the protocol above for free. Necessary here because four containers poll concurrently | 04 — required, not optional, given claim 6 |
| **`htmx.config` test knobs** | `defaultSwapDelay = 0`, `defaultSettleDelay = 0`, `historyEnabled = false`, `timeout = 2000`, `logAll = true` | One injected script before the first test action. Values are confirmed real in 1.9.10. Note `logAll` is `htmx.logAll()`, not a config key | 04 — cheap, low-risk; `logAll` also improves `capture_failure_state` forensics |
| **JSDOM / headless component tier** | Run client JS in a virtual DOM (htmx's own suite, or Vitest + global-jsdom) | Requires a Node toolchain this repo does not have (no JS test runner in `package.json`). The survey concedes JSDOM cannot run `getBoundingClientRect()` — and `showEditForm` depends on exactly that (`assets/index.html:239`) — nor Askama | 04 — a cost with a demonstrated blind spot over the code that matters most; the weakest of the five |
| **htmx's own harness** | How htmx tests itself, as a model for a JS layer | **Correction to the survey**: htmx 1.9.10 ran `mocha-chrome test/index.html` (`package.json` at tag). The Mocha/Chai/**web-test-runner**/**Playwright** matrix the survey attributes to "the htmx repository" is *current master*, a later evolution | 04 — correct the provenance; the pattern is still informative but is not a 1.9.10-era design |

### What genuinely bears on ticket 03

The survey's readiness protocol is the leading candidate answer for the posture
no-fire, and the connection is direct: ticket 03's leading hypothesis is that
`wait_until_visible` returns before htmx has attached its `change` listener to
the swapped-in select. A `htmx:afterSettle`-anchored wait for the
`.worlds-panel` target would be exactly the condition ticket 03 says is missing —
"the harness has no 'htmx has processed this element' wait"
(`assets/flake-investigation-2026-09-13.md`).

Two riders, both inferred:

1. If ticket 03 confirms the race is htmx's listener-attachment ordering rather
   than Playwright's dispatch timing, the fix is a **harness-side wait**, and the
   event bridge is that wait. Ticket 03's own stopping rule anticipates this: a
   confirmed nondeterminism means ticket 04 must pick a design "where the race
   cannot exist".
2. `htmx.config.logAll` (or `htmx.logAll()`) during a repro would settle ticket
   03 quickly — it logs the swap/settle sequence directly, so a missing
   `htmx:afterSettle` for the panel becomes visible in the engine-side capture.

## 4. Recommendation for ticket 02

The survey leaves three genuine gaps that ticket 02 should research, and the
fact-check above bounds what is already settled:

| Already settled by this analysis | Still open for ticket 02 |
|---|---|
| Poll intervals, container set, client-JS inventory, port range, capture paths, response-header assertions — all confirmed, no external research needed | How mature htmx projects express **readiness after a swap** without polling — the survey asserts a console-signal pattern but cites a thread that does not support it |
| htmx#2787 is a fixed 2.0 regression; not a risk at 1.9.10 | Whether `hx-sync`, `hx-indicator`, or `htmx.config` knobs make the **posture `change`** no-fire impossible rather than merely waited-around |
| The Stack Overflow source is about a malformed trigger name, not `afterSettle` reliability | Independent evidence for the **listener-attachment ordering** question ticket 03 raises — the survey assumes it, it does not evidence it |

**Verdict for ticket 02's framing.** External research is needed for the
readiness protocol's *evidence base*, not for the repo facts — those are now
checked. The survey's patterns are worth keeping; its citations for the
load-bearing ones are not.
