# Decide how a generation shows on the Game tab

Type: prototype
Status: open
Blocked by: 03, 05

## Question

What does the player see on the Game tab from Send to the landed Narration, and can the player stop a single Job?

## Context

- The critique's P1 issue, and its fix: a pending row at the end of `#story-log` in player words ("The narrator is writing…", "Updating the scene…"), the status next to the input or inside that row, the exact step names kept for the LLM Messages tab, and Stop for a single Job if the engine supports it ([the Game tab critique](../assets/game-tab-critique/report.md)).
- [Find what the engine offers for a visible generation](05-find-the-generation-facts.md) gives the facts and the cost of each option. The engine already stops a single Job (map, "What already exists"), so Stop is a screen decision.
- The probe cannot run a Job for free, and an Action changes the fixture Game. Show the busy states as static versions injected into the page. The build ticket tests the real states with `ScriptedModel`.
- The same display serves an Autoplay run: the critique found that its progress ("Autoplay: cycle 3/10 Quantifying...") is a footnote in the corner, while watching is the whole experience of Autoplay.
- Show 2–3 versions in the chosen direction ([Choose the direction of the Game tab](03-choose-the-direction.md)). Motion, if any, respects `prefers-reduced-motion`. The live regions (`#narration-announcer`, `#status-announcer`) keep working.

Done when the owner has picked a version, the answer records it with the words of each step, the decision on Stop for a single Job, and the build tickets it needs.
