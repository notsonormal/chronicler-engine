# Retire the toast and route its callers

Type: task (AFK)
Status: resolved
Blocked by: 63, 64

## Question

`#error-notification` is retired: every failure the toast carries today has a routed surface. What changes across `assets/index.html`, the stylesheet and the specs?

## Context

- Decided in [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md), and sequenced last on purpose: retiring it before the banner (51), the inline slots (25, 26) and the clamped status display (53) exist would make failures go dark.
- Eight `showError(...)` call sites in `assets/index.html`: the status-text path, `htmx:beforeSwap` for htmx responses, `submitEdit`, `submitGenerationRequest` (new swipe and retrigger), `switchSwipe`, `deleteMessage`, and the delete path that shows the server's own text.
- Scenario 16.7 ("Action failure renders an error toast") and 16.8 ("A newer error is not hidden by an older error's timer") describe the toast and must be replaced or removed with it. 16.9 acknowledges the toast as the route for a confirm-time error; check it. 16.10 names it too (ticket 53).
- Remove the element, the `showError` helper, its hide timer and the `.error-notification` styles.
- `docs/diataxis/reference/frontend/dashboard.md` describes the error banner as the error path (finding 6, doc drift). Ticket 23 aligns that doc; this ticket should not leave it describing a removed element.
- Ticket 34 replaced weak tests in this area and ticket 40 drives the retrigger control. Check that removing the toast scenarios moves their coverage rather than deleting it.

## Done when

- No `showError` call site remains and `#error-notification` is gone from the markup and the stylesheet.
- Every failure the toast used to carry now lands on the banner, the status display, or an inline slot, shown per call site.
- The spec scenarios that describe the toast are either replaced by the routed surface's scenarios or removed with the reason recorded in the answer.
- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.

## Answer

Resolved. `#error-notification`, `showError`, its hide timer and the `.error-notification`
styles are gone, and every failure the toast carried reports on a routed surface.

**Routing, per call site** (`assets/index.html`):

| Call site | Surface now |
|---|---|
| the status-text path in `applyStatusDisplay` | the status display itself — the server's clamped line + Details disclosure (53/63). The toast call and the `lastStatusError` dedupe are deleted. |
| the `htmx:beforeSwap` `isError` listener | deleted; `htmx:responseError` / `htmx:sendError` (64) own the path |
| `submitEdit` | the story log's own error slot, `#story-log-error` |
| `submitGenerationRequest` (new swipe, retrigger) | the status display, through `showStatusError` |
| `switchSwipe` | the story log's own error slot |
| `deleteMessage` | the story log's own error slot, with the server's text as the raw detail |

**Two client changes the retirement needed.**

- `#story-log-error` sits outside the polled `#story-log`, so the 2s morph cannot drop a
  message the log reported.
- `inlineErrorSlotFor` now adds the container's slot when the server rendered none, and
  never creates one on the success path. `.posture-override` and `.posture-group` joined
  `INLINE_ERROR_CONTAINERS`. Without this, the games, worlds and posture action failures
  would have gone silent — those surfaces never had a slot, so the toast was their only
  report.

**Specs.** 16.7 and 16.8 are removed: 16.19 already covers a failed action's inline slot,
and 16.8 described the toast's own 5s timer, which no longer exists. 16.10 is rewritten
onto the status display; its dedupe half was toast state. 30.4, 30.5 and 30.10 name their
routed surface. New 30.18 and 30.19 cover the two call sites that had no coverage at all:
a failed delete and a failed swipe switch.

**Tests.** Tier 2 (`tests/browser/stub/`). `test_error_toast_on_action_failure` and
`test_newer_error_keeps_toast_visible` deleted; the 16.10 test rewritten; 30.4/30.5/30.10
and 16.25 moved onto their slots; 30.18/30.19 added. `read_error_toast` left
`tests/test_utils/browser.rs`; the stub gained a failing `/history/delete` route and a
`set_switch_failing` handle. Mutation proof: with `showStoryLogError`, `showStatusError`
and the fallback-slot creation stubbed out, exactly 30.4, 30.5, 30.10, 30.18, 30.19 and
16.25 fail.

**Docs.** `dashboard.md` no longer draws or describes the toast and its failure paragraph
names the three surfaces and scopes its non-2xx rule, so the posture route's 200 error
fragment no longer contradicts it; `ui_design.md` loses its Error Notification section and
the now-unused `--color-error-gradient-start` token. Two Rust doc comments that named the
toast were retargeted: `error_response` in `src/adapters/driving/http/utils/error.rs` and
the `StubActionOutcome::Error` variant in `tests/test_utils/stub_server.rs`. Full alignment
stays with [23](23-align-frontend-docs.md).

**Fetch failure paths report the server.** The three fetch-based failure paths
(`submitEdit`, `submitGenerationRequest`, `switchSwipe`) and `deleteMessage` now share one
`throwServerDetail(response)`: it reads the response body and rejects with that text, so the
Details disclosure shows what the server said instead of a client-built status line. It
reuses `plainResponseText`, the strip the htmx path already applied, so the disclosure shows
the body's text rather than its markup, and a body with no text falls back to the status
line. The story-log and status specs now read "its Details disclosure holds the raw server
text", and the four assertions that looked for the string `500` now check the stub's own
message (`Stub save failure`, `Stub retry failure`, `Stub retrigger failure`,
`Stub swipe switch failure`).

**Two-axis review and its fixes.** The standards axis and the spec axis each found real
defects, all fixed here.

- The generated fallback slot carried the fixed value `region`, so two generated slots in one
document both emitted `id="region-popover"` and the second slot's Details control resolved the
first slot's popover. The value now counts up per generated slot.
- `.worlds-panel` and `.games-panel` fetch once with `hx-trigger="load"`, so `isPollRequest`
is false for them and they were not error containers: a 500 on either fragment left the tab
empty with no message. HEAD's `htmx:beforeSwap` listener toasted exactly that, so retiring the
toast had silenced it. Both panels joined `INLINE_ERROR_CONTAINERS`, which routes the failure
through the same generated slot; scenario 16.33 and its stub-tier test cover both panels and
assert the failure banner is not raised. The Settings and Prompt Presets panels need no
change: their fragment handlers carry no 500 path.
- `read_error_disclosure` and `error_details_open` now live once in
`tests/test_utils/browser.rs`, replacing the deleted `read_error_toast`; the copies in
`tests/browser/stub/story_log.rs` and `tests/browser/stub/dashboard.rs` are gone, which is the
resolution ticket 36 asked for.
- Coverage the review found missing: 30.19 now asserts the log keeps its entries, and 30.5,
30.10 and 16.10 assert the Details disclosure starts closed.
- Row-level coverage replaced the over-broad mutation-proof claim. A new stub route answers
`POST /reset` and one answers `POST /worlds/:key/delete` with a 500, and scenario 16.34 with
its test drives the shipped reset and world-delete controls. The slot must be a direct child
of the row, so the test proves the nearest container is the row and not the panel: removing
`.game-item` fails the game iteration and removing `.world-item` fails the world iteration,
both verified. Earlier the claim that the fallback covered the games and worlds rows rested
on no test at all.

**Hand-off to 23.** Every fetch-based failure now passes the server's body, so "the raw
server text" is accurate for the story-log delete, save, retry, retrigger and swipe-switch
failures alike; the older rule that those texts must say "the raw text" or "the failed
request's status" no longer applies. `dashboard.md`'s non-2xx sentence keeps "the raw server
text" and its third surface reads "the failing region's own error slot", which covers the
story log's slot beside the polled log. The posture save that fails validation answers 200
with a fragment that replaces the posture controls, so it appears in neither the banner nor a
slot; that path stays undocumented and is 23's to place.

`python build.py` is green: architecture 1, guardrails 165, integration 1570 (2 skipped),
browser 70 (`logs/build_20261008_202621.log`). Uncommitted, pending review and
`/commit-and-push`.

Note: `tests/STRATEGY.md`, `tests/AGENTS.md`, `map.md` and issue 32 carry another session's
uncommitted ticket-32 work. They are not part of this ticket.
