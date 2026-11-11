# Decide the action area: Send, Autoplay and its budget

Type: prototype
Status: open
Blocked by: 03

## Question

How do Send, Autoplay and the Autoplay budget sit in the action area, and is Autoplay a button next to Send or a mode that the player enters?

## Context

- The critique's P1 issue: `#submit-btn` and `#autoplay-btn` have identical styles and sit 104px apart. `#autoplay-cycles` shows a bare "10" with only a `title`. One click starts 10 paid Cycles with no confirmation ([the Game tab critique](../assets/game-tab-critique/report.md)).
- The critique's fix: Send is the only filled, primary button. Autoplay and its budget form one secondary control with visible text ("Autoplay · 10 cycles"), away from Send. `#autoplay-form` and `#autoplay-controls` in `assets/index.html` already allow the regrouping.
- Also decide: a confirmation or a cost line before a run starts; what the placeholder of the Action input says (the critique: "Enter an action..." does not mention `/` or the Persona).
- The specs `docs/specs/autoplay.md`, `browser_autoplay.md` and `actions.md` hold the current behaviour and selectors.
- This ticket decides the place, the grouping and the behaviour of the controls. The look of a button comes from [Write `DESIGN.md`](04-write-design-md.md) when that ticket is done.
- Show 2–3 versions in the chosen direction ([Choose the direction of the Game tab](03-choose-the-direction.md)).

Done when the owner has picked a version, and the answer records it and the build tickets it needs.
