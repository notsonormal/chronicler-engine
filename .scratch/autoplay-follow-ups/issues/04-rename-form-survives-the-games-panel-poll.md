# Keep an open rename form through the Games-panel poll

Type: task
Status: needs-triage
Blocked by: —

## Question

While an Autoplay run holds a Game, the Games panel polls every 5 seconds and replaces its content. The poll closes an open rename form and drops the name the player typed. How should the panel keep that form?

## Context

- Found while carrying out `docs/plans/autoplay-review-fixes-plan.md` section 2.6. The plan expected `hx-swap="morph:innerHTML"` on `.games-panel` to keep the form. The stub-browser check showed that it does not.
- Evidence: with `hx-swap="morph:innerHTML"` on `.games-panel` in `assets/index.html`, a stub-browser check opened the rename form of a saved Game (`.game-rename`), typed a name, and ran one poll. Afterwards `details.open` was `false`, the `open` attribute was gone, the input held the server's value, and the input was not the active element. Idiomorph removes the `open` attribute that the server HTML does not carry, and it resets the input value.
- The plan assumed the result without a test. The check and its spec scenario were removed from the change, because the check failed.
- Options to weigh: `hx-preserve` on the forms, a poll that skips the swap while a form is open (as the story log does during an edit), or a server render that carries the open state.
