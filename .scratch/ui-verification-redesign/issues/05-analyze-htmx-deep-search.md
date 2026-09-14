# Analyze the Gemini deep-search survey on HTMX verification patterns

Type: research
Status:
Blocked by:

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
