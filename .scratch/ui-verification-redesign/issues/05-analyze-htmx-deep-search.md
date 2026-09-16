# Analyze the Gemini deep-search survey on HTMX verification patterns

Type: research
Status: resolved
Blocked by:

## Answer

Validated the survey against the repo: [gemini-survey-validation.md](../assets/gemini-survey-validation.md).

**The survey's patterns are sound; its repo facts are not.** Keep the
three-tier model, the polling-vs-lifecycle diagnosis, and the targetId-scoped
readiness protocol. Re-verify every repo-specific fact — the survey is a stale
snapshot of an earlier state of this repo (e.g. it names `wait.rs` helpers
removed in the 2026-07-25 refactor), and it asserts CSRF form processing that
this repo has never had.

**Two loaded claims corrected.**

- htmx#2787 (leading newline suppresses swap events) is a **2.0.0/2.0.1
  regression, already fixed** — the reporter states 1.9.12 was unaffected. This
  repo pins **1.9.10**, so the claim does not apply and the recommended
  mitigation guards against a defect this htmx does not have.
- Stack Overflow 69538244 does **not** support the readiness protocol: its
  accepted answer concerns HTTP redirects, and the thread is a user mis-writing
  `hx-trigger="htmx:afterSettle"`. The survey cites it as lifecycle-event
  evidence, which it is not.

**What is confirmed and needs no external research:** poll intervals and
container set (2s/4s/5s/5s across five containers, not three), client-JS
inventory, the 3010–3050 port range, `capture_failure_state` paths, and the
existing `HX-Retarget`/`HX-Reswap`/`HX-Refresh` assertions. The `htmx.config`
knob table is the survey's most reliable section — `defaultSettleDelay = 20` and
the rest verified inside `assets/htmx.min.js`.

**Bears on ticket 03 and 04.** The readiness protocol is the leading candidate
answer for the posture no-fire: ticket 03's hypothesis is that
`wait_until_visible` returns before htmx attaches its `change` listener, and the
`afterSettle`-anchored wait is exactly the missing "htmx has processed this
element" condition. `htmx.logAll()` for the repro would settle ticket 03
directly. The JSDOM tier is the weakest option — no JS runner exists here, and
JSDOM cannot run `getBoundingClientRect()`, which `showEditForm` depends on.

**Correction for ticket 02:** the Mocha/Chai/web-test-runner/Playwright matrix
the survey attributes to "the htmx repository" is *current master*, not the
1.9.10 era, which ran `mocha-chrome test/index.html`.

No code changes made.

## Question

A Gemini deep-search survey already exists at
[assets/HTMX Verification Testing Patterns.txt](../assets/HTMX%20Verification%20Testing%20Patterns.txt)
(23K, 24 cited sources). It covers request-level contract verification across
ecosystems, browser-level E2E with lifecycle-event readiness, JSDOM component
testing, htmx's own test suite, and htmx config knobs — contextualized against
Chronicler Engine (its source 5 is a repomix dump of this repo).

Before ticket 02 (Research how mature projects verify HTMX apps) can answer
"what's missing from this survey", this ticket must analyse how much of it is
*true and applicable* here.

Do three things:

1. **Fact-check against the actual code.** The deep search makes specific
   claims about this repo; verify each against
   `assets/index.html`, `tests/test_utils/wait.rs`, `tests/http/`,
   `tests/browser/`. Include at minimum:
   - It claims `wait.rs` contains `wait_for_story_log_change` and
     `wait_for_element_text` — appears false; only `wait_for_status_ready`,
     `wait_for_llm_idle`, element-children/visible/hidden helpers exist.
   - It claims background `hx-trigger="load, every Ns"` polling on
     `#story-log`, `#status-display`, `.visual-sidebar` — confirm exact
     intervals and which containers.
   - It claims client JS in `index.html`: `onStatusPoll`, `showEditForm`,
     `saveActionArea`, slash-command menu, `htmx:beforeSwap` error toast
     listener — confirm/deny each.
   - Any claims about Askama templates, response headers (HX-Retarget,
     HX-Refresh), and the worlds posture form.
2. **Extract the actionable patterns.** Distill the survey's concrete
   mechanisms: the request-header-verified contract tier, the lifecycle-event
   readiness protocol ("console signal with targetId" pattern around
   `htmx:afterSwap`/`htmx:afterSettle`), hx-swap/hx-sync/hx-indicator config
   knobs for testability, the JSDOM component-test option, and what
   htmx's own TESTING.md/web-test-runner harness does. For each: what it would
   take to apply here, and whether it bears on the posture no-fire mechanism
   (ticket 03) or the readiness decision (ticket 04).
3. **Spot-check the loaded claims.** Two claims carry heavy design weight and
   deserve source-level validation: (a) the htmx 2.0 "leading newline breaks
   swap events" issue (htmx#2787) — note we pin htmx **1.9.10**, so assess
   whether it applies; (b) the Stack Overflow "afterSettle not firing" report.
   Use `source_check` / fetch the cited pages.

Deliverable: linked markdown asset — a validation table (claim / verdict /
evidence path) + the distilled pattern list. Make no code changes. Ticket 02
consumes this to decide what genuine external-research gaps remain.
