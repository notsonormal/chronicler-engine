# Drive the retrigger control in a browser test

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Nothing drives `submitRetrigger()` in a browser test: no stub fixture renders a retrigger button, and the stub's `/retrigger` 500 route was removed in ticket 36. Its failure path is the shared `submitGenerationRequest`, which scenario 30.5 already covers through `/swipe/new`, so only the retrigger URL and wiring are unobserved. Is that worth a tier-2 scenario?

## Context

- From [Recover from a failed message save instead of freezing the story log](27-recover-failed-message-save.md) and [round 1 follow-ups](36-follow-up-small-review-issues.md).
- It needs a retrigger button on the stub story-log fixture, a stub `/retrigger` route, and a tagged scenario in the browser story-log spec.

## Done when

- A tier-2 test clicks the retrigger control and asserts the request and the recovery, or the ticket is closed with the reason it isn't worth it.
- `python build.py` is green.

## Answer

Worth a tier-2 scenario — added. `templates_tests.rs` only pinned that the rendered template contains the string `submitRetrigger`, so a rename of the shipped JS function or a change to its URL would break the control silently. Scenario 30.10 (tier 2, `tests/browser/stub/story_log.rs::test_failed_retrigger_posts_to_retrigger_and_recovers`) proves the shipped function exists, is reachable from the control, posts to `/retrigger`, and runs the shared failure recovery (toast, status back to Ready, Send re-enabled).

**Deviation from the ticket's mechanism.** The test does not add a retrigger button to the stub fixture or a stub `/retrigger` route. The fixture files were outside this ticket's file scope during a concurrent run, so the test plants the real template's control (`class="action-btn retrigger-btn"`, `onclick="submitRetrigger()"`) on the canned narration and intercepts `window.fetch` to answer `/retrigger` with a 500. That still exercises the shipped JS — a renamed function throws and a changed URL is not intercepted, either of which fails the test — but it does not exercise stub routing or server-rendered placement. The stub fixture/route gap therefore remains open; server-rendered placement is pinned by `templates_tests.rs`.

Validation: `validate_feature_spec.py` 171 declared / 171 covered; `test-pattern test_failed_retrigger_posts_to_retrigger_and_recovers` 1 passed; `clippy` OK; `browser` 31 passed / 0 failed; `integration` 1531 passed / 0 failed / 2 skipped; `unit` 1198 passed / 0 failed. The full gate failed only at guardrails, on two files from the concurrent 41 work (`src/application/llm_recorder.rs` free fn, `tests/http/llm_messages.rs` multi-line `//!`), not on this ticket's diff; that agent was steered to fix them.

**Gap closed (review follow-up).** The stub-fixture/route gap noted above is now fixed: `tests/test_utils/stub_fixtures/story_log.html` carries the template's retrigger control, `tests/test_utils/stub_server.rs` has a counted `POST /retrigger` → 500 route, and 30.10 drives the fixture's shipped control and the real route (no `insertAdjacentHTML`, no `window.fetch` patch). Residual: the fixture places the control on the narration while the template only renders it on `loop.last && show_retrigger`, so the fixture is slightly unfaithful in placement; harmless for this test but a candidate for the next fixture refresh.
