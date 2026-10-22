# Give the Narrator's role health a right answer after a Stop

Type: task
Status: needs-triage
Blocked by: —

## Question

After a Stop, the model call that the Stop did not cancel still returns, and its answer still counts for the role health of the Narrator. What should the role health show after a Stop?

## Context

- A Stop removes the holder at once. The running model call is not cancelled and runs to its end, and the id check refuses its save (`docs/diataxis/reference/game_flow.md`, "Stop and Reset").
- The role health reads the result of the call, so the answer of a stopped call can mark the Narrator healthy or failed after the player stopped the work.
- Autoplay makes this more visible: a Stop is the normal end of a long run.
- Not in scope of the review-fixes plan (section 6b). A cancel of the in-flight call is a separate decision.
