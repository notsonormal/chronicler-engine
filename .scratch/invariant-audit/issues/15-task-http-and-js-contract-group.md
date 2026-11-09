# Decide and close the HTTP and JS contract group (tier 4)

Type: task (AFK, decide and do)
Status: open
Blocked by: —

## Question

Give the one verdict for the HTTP and JS contract group under the enforcement order of the map Notes, and put it in place. A part of the group can get its own verdict when one mechanism does not fit it.

| Finding | Invariant that nothing enforces |
|---|---|
| http-02 | Every Game-bound route is in `game_bound_routes`. The test compares the list with its own source, so a route that is not in the list is invisible to it. |
| http-03 | The slash-command names in the browser palette match `Action::parse`. |
| http-04 | The form field names that the JS builds match the Rust form structs. |
| http-05 | The `?v=` cache bust changes with each asset change. |
| http-06 | The `index.html` compiled into the binary matches the `assets/` files read at run time. |

For each finding: a guardrail test (enforcement preference 6), or one source that generates both sides.

## Context

- Findings in full: [the http report](../assets/http.md). The coordinator did not read these in the code.
- An existing guardrail is the precedent: `tests/infrastructure/guardrails/screen.rs` compares `SCREEN_PARTS` with `assets/index.html`.
- http-06 matters only for a split deploy, so an "accept" verdict is likely. An "accept" verdict stops the task and asks the owner (decide-and-do rule).
- The six end conditions of the map Notes apply.

## Answer
