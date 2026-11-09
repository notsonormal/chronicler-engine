# `dashboard.md` omits three forget points, the action-area priority and most of the client state

Status: needs-triage
Source: ticket 05 of the state-machine map, doc checks D1, D2, D4 and the scout claim on the priority (`05-dispositions.md`, section 6; catalogue, rows G1, G2 and *Doc mismatches*)

## The defect

`docs/diataxis/reference/frontend/dashboard.md` does not agree with the scripts in four places. Each one is a missing fact or a wide sentence. No product decision is needed.

1. **The forget points of the stamp** (`:125`, and `:85`). The doc says `data-revision` is absent "from the end of a request that is not a read until the next load". The code also forgets the stamp in `leaveSection` (`assets/dashboard-screen.js:407`), in the Previous or Next click of the Games pager (`:396`) and in `clearEditState` (`assets/dashboard-story-log.js:109`).
2. **The priority of the five action-area states** (`:63`). The doc names Idle, Checking, Generating, Autoplay and Preview. The code picks Checking first, then Autoplay, Generating, Preview, and Idle last (`assets/dashboard-action-area.js:64-70`).
3. **The client state** (`:41-50`). The list names no variable. The scripts hold `sendPending`, `checkInFlight`, `previewSentText` (`dashboard-action-area.js:5,:10,:283`), `dialogSession` (`dashboard-dialog.js:60`), `editState` (`dashboard-story-log.js:17`), `screenWaiters` and `requestsInFlight` (`dashboard-screen.js:191,:72`), and three live-region memories (`dashboard-action-area.js:110,:140,:171`).
4. **The rule of what a load skips** (`:18`). The sentence says "A load skips an element that holds unsaved input". The true rule is at `:97`: two kinds, the story-log entry in edit and a control whose request is in flight (`dashboard-screen.js:114-116`). A part has no other draft (`:19`).

## Impact

Low. A reader of the doc gets an incomplete picture of the refresh rules. If the TLA+ model and the generated diagrams of the dashboard replace these paragraphs (ticket 05, the disposition of G1, G2, G6 and G8), this fix is no longer needed. Fix the doc first only if that work does not start soon.

## Done when

- The four sentences name the facts above.
- `python build.py validate-docs` and `python build.py vale` pass.
