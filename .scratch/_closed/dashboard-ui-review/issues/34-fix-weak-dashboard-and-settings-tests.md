# Fix the weak dashboard and settings tests

Type: task (AFK)
Status: resolved
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
- Added from [Keep the command form when a text-check result shows](03-keep-command-form-on-text-check.md): 16.9/16.10 now drive the shipped submit → preview → confirm path (M1 fixed), but B2 is still open. 16.10's `inject_status_html` (`tests/browser/stub/dashboard.rs`) hand-writes the status span the `/status/generating` poll should swap in; the stub hardcodes `idle`. Serve the error and Ready spans from a stub status outcome and let the real poller swap them, or downgrade 16.10's wording.

## Done when

- Each listed test fails when its named regression is injected by hand (a replaced form, a missing toast, an unswitched connection). Record the check under `## Answer`.
- `StubActionOutcome::Idle` is gone or matches a real response shape.
- `python build.py` is green. The user reviews the diff. Commit after approval.

## Answer

Each test below was proven able to fail: the behaviour it guards was broken in the worktree, the test failed, and the break was reverted.

- **16.5** (tier 3, `test_form_stays_static_after_submission`): asserts the same `#command-form` DOM node, not its id string. Proof: swapping in a clone with the same id made it fail ("was replaced").
- **16.7 / 16.8** (tier 2 stub): the synthetic `htmx:beforeSwap` helper is gone. The stub serves a real 500 from `/action/check` through the engine's `render_error`, and the test submits the shipped form and reads the toast. The body is server text, so a toast built from the input value fails. Proofs: the stub answering 200 gave "toast never displayed"; removing `clearTimeout(errorHideTimer)` left the toast up at 6s; toasting the input value failed 16.7. The test submits with a local `submit_command` helper, not `send_action`, because `send_action` waits for the "Thinking" status, which a 500 never produces.
- **`StubActionOutcome::Idle`** deleted. It returned a shape the real route never returns, and nothing used it.
- **20.2 / 20.3 / 20.4** (tier 1): each follows the save with `GET /fragment/settings` and asserts the Narrator/Quantifier badge moved (the previous holder loses it). Per ticket 31 being open, the read-back is the shipped GET, not storage. Proof: removing the two role assignments in `save_settings_handler` failed all three while the body still said "Settings saved!".
- **B2 / 16.10** (tier 2 stub): `inject_status_html` is gone. The stub serves `/status/generating` (idle text or the real error span, per server through a `StubStatus` handle) and the test waits for the shipped 5s poll. Proofs: an always-idle stub failed the first assertion; removing the observer's `lastStatusError = null` failed the dedupe-reset assertion.
- 16.9 is unchanged (ticket 03 already moved it to the shipped path).

Specs 16.5, 16.7, 16.8, 16.10 and 20.2–20.4 reworded to observable behaviour.

**Gate:** worktree on `9b46f1a5`, main unchanged since: `nextest: 1661 passed, 0 failed, 2 skipped`, browser 30 passed (`build_20261001_205821.log`). The first run hit a 16.9 acknowledgement race that passed on re-run.

**Code review** (`/code-review`, ISSUES → fixed): the stub's 500 echoed the submitted command so a toast built from the input would still pass; the "unchanged keeps its badge" assertions held without the switch; 16.5's Gherkin still said "id unchanged".

**Not fixed (judgement, now in ticket 37):** `connection_card` duplicates `preset_card_html_slice`; `submit_command` duplicates `send_action`'s fill script; the `StubStatus` doc over-promises; a redundant inner `Arc<Mutex>`; 16.8's 1.5s timer margin and a single read at 3.5s are load-sensitive; set the error before the confirm swap to save up to 5s in 16.10; the stub's Error arm omits the status-swap headers the real route adds; 16.6 prose names the `send_action("wait")` helper.

