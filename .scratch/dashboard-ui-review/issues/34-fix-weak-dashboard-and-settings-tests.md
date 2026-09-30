# Fix the weak dashboard and settings tests

Type: task (AFK)
Status: open
Blocked by: —

## Question

Can each of these tests fail on the regression it names, and drive its setup through the shipped path?

## Context

Findings from the test-design review, in [test_design.md](../assets/test-audit/test_design.md):

- **16.5**, `tests/browser/dashboard.rs` `test_form_stays_static_after_submission` (T1). It compares the `#command-form` id string before and after, so a replaced form still passes. Assert node identity instead. Its comment was corrected on the dashboard-ui-updates branch. The command submit retargets to `#status-display`.
- **16.7 / 16.8**, `tests/browser/stub/dashboard.rs` `dispatch_error_toast` (B1). The test dispatches a synthetic `htmx:beforeSwap`. `StubActionOutcome::Error` already serves a real 500 from `/action/check`, and no test uses it. Earlier, [Add browser tests for responsive layout + error-state toast](../../test-strategy-execution/issues/15-browser-missing-tests-impl.md) chose the synthetic event because `route.fulfill` was broken and no real 500 path existed. The stub server is a real server now, so that reason may be gone. Verify first: a failed `/action/check` must reach the toast through htmx. If it does, drive 16.7/16.8 through `send_action`, and fix the file's header comment ("No engine endpoint produces this event").
- **`StubActionOutcome::Idle`**, `tests/test_utils/stub_server.rs` (S1). No test uses it. It returns the full `#action-area` template to `/action/check`, which the client swaps `innerHTML`, so it would nest `#action-area` inside itself. The real route never returns that shape. Delete it, or fix its shape.
- **20.2 / 20.3 / 20.4**, `tests/http/settings.rs:92-139` (D1, W). All three assert only `body == "Settings saved!"`. Add a read-back that proves each switch took effect. A follow-up GET works under every option in [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md). A stored read may depend on that answer, so prefer the GET.
- Out of this ticket: 16.9 / 16.10 (M1, B2) set up their swap through internal JS. [Keep the command form when a text-check result shows](03-keep-command-form-on-text-check.md) removes the failure they guard, so they are handled there.
- Other workers may hold uncommitted edits in `tests/browser/dashboard.rs` and `tests/browser/stub/dashboard.rs`. Check `git status` before editing.

## Done when

- Each listed test fails when its named regression is injected by hand (a replaced form, a missing toast, an unswitched connection). Record the check under `## Answer`.
- `StubActionOutcome::Idle` is gone or matches a real response shape.
- `python build.py` is green. The user reviews the diff. Commit after approval.
