# Redesign the error and health display

Type: task (AFK)
Status: resolved
Blocked by: 08

## Question

Theme 1 of the review, done as one sweep because all three pieces share one
short-message + anchored-popover fragment, one client listener, and one handler
error shape. Each replaces a surface that lies about failure today.

### The banner (findings 1.1, 1.3)

`Connected` is a static string in `HeaderTemplate`, fetched once with
`hx-trigger="load"`, so the header reads as a live health signal while never
changing. On the review turn the Quantifier failed twice and the engine used
fallback NPC IDs, while the header said "Connected" and the status said
"Ready". Replace it with a banner that appears only when there is something to
say.

### A failed request keeps its region

A fragment failure returns HTTP 200 carrying `<div class="error-message">`
(`render_fragment` → `render_error`), so the failing body *becomes* the element
the request targeted. On a poll it destroys last-good content, and the user sees
an error fragment where the story log or the sidebar used to be. Make a failed
poll change nothing in the DOM, route the failure to its surface, and own the
shared error fragment the form and card tickets consume.

### The status-display error (findings 1.4, 1.5)

`generating_status_handler` renders `GenerationStatus::Error(msg)` as
`<span class="status error">Error: {msg}</span>` at full length. The string
pushed the command input from about 1020px down to 235px, and the string is raw
transport text. Clamp it to one line and move the raw text behind the agreed
disclosure.

## Context

Decided in [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md).
The model is quiet by default.

### Health model

- `Connected` is deleted. The header keeps the game title and the game name only.
- The banner sits directly under the header bar and above the tab bar, in the
  visual zone the toast occupies today. While healthy it is absent and costs 0px;
  when up it costs about 30px.
- **Degraded** means the newest `llm_messages` row for that role carries
  `error_message`, and it clears on the role's next success. Two states, Healthy
  and Degraded — no third state for "the engine worked around it". That
  distinction belongs to the generation, and the status display owns the
  generation.
- Source: the newest row per agent. Roles are `narrator`, `quantifier`,
  `options`, `trigger`. Ticket 41 makes every LLM attempt, success or failure,
  persist a row naming the agent and backend/model, so no new persisted state is
  needed.
- **Unreachable** is client-owned: a body-level `htmx:sendError` /
  `htmx:responseError` listener sets it; any successful response clears it. A
  server cannot render its own unreachability. Role health is the opposite — a
  Quantifier fallback still returns 200 — so it is server-rendered in
  `/fragment/header`, which gains a poll.
- **No close control** and no timer. The banner is present exactly while the
  condition holds.
- The banner names the degraded role and what the engine did instead where it
  can ("using fallback NPC IDs"), and carries a **Details** control opening a
  popover with the per-role list, each role's last error and its backend/model.
  The popover is the anchored pattern the status display also uses — one
  pattern, two call sites. Never render the raw error inline.
- On-demand health lives in Settings (ticket 15), which gains the per-role
  summary and a connection test. Do not add an ambient header affordance; that
  was considered and rejected.
- Semantics for ticket 47: `role="status"` while degraded, `role="alert"` while
  unreachable.

### Failed requests

- The rule: a failure never swaps into the region it describes.
- Polls and their intervals: `#story-log` 2s, `.visual-sidebar` 5s,
  `#options-dock` 2s, `#status-display` 5s, `.llm-messages-panel` 4s.
- A failed poll returns non-2xx with `HX-Reswap: none`, and the region keeps its
  last good content. `render_fragment` currently returns 200 for every failure,
  which is also why finding 05.F1 reaches the user as data loss.
- The client has one `htmx:beforeSwap` listener today: on `evt.detail.isError`
  it strips tags off the response body and shows the toast. That listener and the
  banner listener are the same place. On a failed request, mark the banner; for
  an action — not a poll — write the message into the failing form's inline slot.
- `htmx:sendError` fires for a dead server with no response body at all, so the
  client synthesises the same fragment. That is the concrete gap in ticket 05's
  finding 6: with the engine down, the connection, world and game deletes do
  nothing.
- **This ticket owns the shared short-message + anchored-popover error
  fragment.** [Keep failed forms and cards in place](64-keep-failed-requests-in-place.md)
  consumes it for its inline slots; do not build a second shape.
- Raw server text is not rendered inline. It lives behind the popover, and
  verbatim in the log.

### Status-display error

- One clamped line carrying a short user-facing message, with the raw text
  behind an **anchored popover** — not an inline `<details>`, which grew the
  action area from 64px to about 100px in the mock
  (`tmp/show-me-shots/D-option4-table-status-480.png`, local only).
- The popover is the same pattern the banner's Details control uses: one
  pattern, two call sites. Escape and outside-click dismissal are part of it.
- Raw text goes verbatim to the log; what reaches the user is clamped and mapped.
- A generation error clears on retry, on the reset route, or on dismissal. No
  timer.
- Ticket 47 owns announcing it.
- Scenario 16.10 in `docs/specs/browser_dashboard.md` ("Status errors still
  reach the toast after confirming a preview") names the toast and needs
  updating with ticket 54.

## Done when

- `Connected` is gone. The banner appears on a degraded role and on an
  unreachable server, and disappears when the condition clears.
- A failed poll leaves its region unchanged and marks the banner. A failed action
  renders its message in the inline slot inside the form or card, never over the
  region. A dead server produces that same inline message for a pending action,
  instead of silence.
- A generation error renders as one clamped line, with the raw text reachable in
  the popover and nowhere inline. The action area's height and the command
  input's width do not change when the error appears.
- Tests cover a role degrading then recovering, the unreachable state, a poll
  failure, an action failure with the server up, one with the server down, and
  the clamped line + popover. Tier by `tests/STRATEGY.md` and name it in the
  answer.
- The spec scenario for the banner exists (a new scenario in
  `docs/specs/browser_dashboard.md`, or the right spec if one already covers the
  header). Add or change a scenario only if the spec is incomplete or wrong.
- `python build.py` is green, the user reviews the diff, then commit through
  `/commit-and-push`.

## Answer

Resolved. The header's hardcoded `Connected` is deleted; a banner appears only when
there is something to say, and a failure never swaps into the region it describes.

- **Banner.** Server-rendered in `/fragment/header` (which gained a 5s poll) from the
  newest `llm_messages` row per role; client-owned "unreachable" via a body-level
  `htmx:sendError`/`htmx:responseError` listener. `role="status"` while degraded,
  `role="alert"` while unreachable. A Details control opens an anchored popover with
  each role's last error and backend/model; raw text is never rendered inline.
- **Failed requests.** A failed poll answers non-2xx with `HX-Reswap: none` and keeps
  last-good content; a failed action writes the shared short-message + anchored-popover
  fragment into its own inline slot; a dead server synthesises the same fragment.
  `render_fragment`/`render_error` were reworked so a failure no longer returns 200 as
  the region's replacement.
- **Status error.** One clamped line with a mapped short message; raw text only in the
  anchored popover. `#status-display` is fixed at 240px so the command input's width and
  the action area's height do not change when the error appears.

Review follow-ups applied: `role_health` now reads each role's true newest attempt
(`Storage::latest_llm_message_per_agent`, SQLite + in-memory) instead of scanning the
newest 50 globally; a reachable server's failed action is never reported unreachable
(polls are detected by `hx-trigger` containing `every`); an open Details popover survives
a poll with focus retained; the ignored LLM suite's status assertion was updated to the
terminal-state contract; role ids come from the `AGENT_*` constants; `status_html` is a
`SafeHtml`.

Tests: tier 1 `tests/http/failure_display.rs` (38.1–38.4); tier 2
`tests/browser/stub/dashboard.rs` (16.18–16.27); unit `templates_tests.rs` /
`endpoints_tests.rs`. The toast is retained until [54](54-retire-toast-and-route-callers.md).

`python build.py` is green (1566 integration, 62 browser). Uncommitted, pending review
and `/commit-and-push`.
