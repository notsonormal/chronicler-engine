# A refused request ends the pending state of an accepted send

Status: needs-triage
Source: ticket 05 of the state-machine map, row G1 (`05-dispositions.md`, section 4; scan 2, `scan2-frontend.md`, *Action area*; catalogue row G1, "one `sendPending` serves every sender")

## The defect

The action area keeps the pending state of every send in one boolean, `sendPending` (`assets/dashboard-action-area.js:10`). `docs/specs/browser_dashboard.md` Scenario 16.69 says the pending state of a request ends when the screen shows the answer of *its* request. Scenario 16.70 says it ends at once when the engine does not take *its* request.

Retry and Retrigger have no `hx-sync` (`src/adapters/driving/http/templates.rs:25`; the only `hx-sync` on that line is on Save). New options has `hx-sync="this:drop"` (`templates.rs:64`), which stops a second click on the same control only. Each of the three starts the pending state in `hx-on::before-request="updateToThinking()"` (`dashboard-action-area.js:195-198`). Each ends it in `hx-on::after-request="endPendingAfterRequest(event)"` (`:306-310`). That function calls `endPending()` for every answer that is not `done`.

Sequence (known, read end to end):

1. The player presses Retry. The engine accepts. `sendPending` is true and a waiter for the stamp of the answer starts (`:308`).
2. The player presses Retry again, or Retrigger, or New options, before the screen shows that stamp. The engine holds the Game, so it answers `409`.
3. `endPendingAfterRequest` calls `endPending()` (`:309`). `sendPending` is false. `#action-status` is empty and the action area reads Idle, while the first send is still waiting for its screen.
4. The screen shows the first stamp. The status display shows "thinking", and the area reads Generating again.

## Impact

Low. The window lasts from the refused answer to the load of the first stamp: one round trip plus one ask. The server renders Send as enabled until that load, and a click on it gets `409` (the busy registry is the backstop). The flicker is cosmetic.

The behaviour of the overlap is not in a scenario. Scenario 16.70 starts from an idle page. The defect follows from "the answer of its request".

## Done when

- A refused or failed answer ends only the pending state of its own request.
- A browser test starts a Retry, sends a second Retry before the screen shows the first stamp, gets the `409`, and sees `#action-status` still show "Thinking..." until the screen shows the first stamp.
- This is the first piece of the explicit form of G1 (ticket 05, work ticket A): `pending` becomes a count.
