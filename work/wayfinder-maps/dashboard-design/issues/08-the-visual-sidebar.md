# Decide what the visual sidebar shows

Type: prototype
Status: open
Blocked by: 03

## Question

What does the visual sidebar show of the state that the engine keeps: the Room, the Persona, the Characters present, and what when a Room has no picture of its own?

## Context

- The critique's P2 issue: the Room name is only the `alt` of the picture, every Room shows `data/images/Redmist Estate.png`, the portraits are 80px with 12px names, about 400px of the column is empty, and no label names the Persona ([the Game tab critique](../assets/game-tab-critique/report.md)).
- `PRODUCT.md` positions Chronicler by the state that the engine keeps. Marinara Engine shows this kind of state in a Tracker Panel ([Reference apps](../assets/reference-apps.md)).
- Also decide: whether the sidebar shows anything when the player reads an older passage in another Room (it always shows the current state now).
- The spec is `docs/specs/visual_sidebar.md`. Data that the sidebar does not render now (the Persona, the Room name as text) is a behaviour change (map Notes, "Behaviour").
- Show 2–3 versions in the chosen direction ([Choose the direction of the Game tab](03-choose-the-direction.md)). Keep the phone rule: the content must fit a compact row later.

Done when the owner has picked a version, and the answer records it, the data it needs, and the build tickets it needs.
