# Decide whether and how to run a clean-room design of the generation model

Type: grilling
Status: resolved
Blocked by: 15, 16

## Question

Does this map run a clean-room design of the generation model as a reference point for the fix-or-rewrite decision? If it does: at which scope, with which role for SillyTavern and Marinara Engine, in which order relative to [Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), and behind which wall?

## Context

- **Method, inputs, wall, procedure and order options:** [Research: method and inputs for a clean-room design of the generation model](15-research-clean-room-design.md) and its [asset](../assets/research-15-clean-room-method.md). Its §7 lists the six owner decisions this ticket answers.
- **Scope:** [Research: which known problems a clean-room design would collapse, at each scope](16-research-clean-room-collapse.md) and its [asset](../assets/research-16-clean-room-collapse.md) give the scope options and the collapse matrix. The order changes the scope result: 3 settled cells per scope hold only for a run after ticket 01 answers its first question, and O-1 and O-2 also do unless the owner accepts a reading of prior decision 6 (asset §4.2). One owner answer before the run settles each group.
- **Coordination with `architecture-deepening`:** the research 16 asset (§5) compares three options for the overlapping tickets of that map: proceed, pause, or fold into a new effort. A run at B overlaps 5 of its tickets, and a run at C overlaps 11. Its next pick, ticket 09, overlaps at every scope.
- **Time-sensitive.** Ticket 01 is on the frontier now. The orders "before ticket 01" and "in place of part of ticket 01" close when a session takes ticket 01.
- **What cannot be clean.** The owner has seen the code, so every ruling and every answer to the clean agent carries some of its shape (research 15 §5). Record which answers in this grilling are about requirements and which are about the run.
- **Terms:** `CONTEXT.md` deprecates "turn". Use Message, Swipe, Snapshot, generation, Action, Action Pipeline.
- **Skills:** `/grilling` and `/domain-modeling`.

## Done when

- Each of the six decisions in research 15 §7 has an answer, or is ruled out with a reason.
- If the run goes ahead at scope B or C: one of the coordination options in research 16 §5.2 is chosen for the overlapping `architecture-deepening` tickets.
- If the run goes ahead: its steps (spec, review, clean run, comparison) are tickets in this map, wired, and ticket 01's `Blocked by:` line matches the chosen order.
- If it does not: the map's fog patch "Fix or rewrite" says that no reference design will exist, and why.

## Answer

Grilled with the owner on 2026-10-12, in four rounds. The labels below follow the Context rule: **run** answers are about the run or the effort, and **requirement** answers go into the spec.

**The run goes ahead, with a changed purpose.** The clean design is a reference for fixes in the current code and also a possible rewrite target (Q3). It does not depend on a language. The owner doubted how much a walled design adds over "starting fresh with a clean plan, stronger models and a stronger understanding of how to build an AI-native project". The answer to that doubt (inferred): a design gets its fixes from the spec, not from the wall. The wall adds two things only: a design with no anchor on the code, and a test of the spec. So the route puts a walled run in front of an open fresh design.

| # | Research 15 §7 decision | Answer | Kind |
|---|---|---|---|
| 1 | Run at all | Yes, as six steps on one route: write the spec, review it, a walled run that tests the spec and gives a first draft, close the spec gaps, an open fresh design, and a comparison with the code | run |
| 2 | Scope | B, the game-state lifecycle, in this map, "for now". The [platform choice](../../platform-choice/map.md) map can extend the design to the whole application. | run |
| 3 | Role of SillyTavern and Marinara | Out of the walled run, so that the run tests the spec alone. In the open design (ticket 22) as prior art. | run |
| 4 | Order | The clean-room tickets run first. Tickets 01, 02 and 03 wait for the comparison, because the owner ruled that their questions are implementation decisions, and a design from the spec may go another way. After the comparison, each one gets a new question, is closed, or is ruled out of scope. | run |
| 5 | Wall | A raw one-shot `pi` run with no tools and no context, from a neutral temporary directory. Two clean agents: `opencode-go/glm-5.3-flash` and `opencode-go/kimi-k2.7-code`. An Opus session writes the spec. | run |
| 6 | Spec review and owner answers | The leak script, then a fresh-context `judge-glm` review, then owner sign-off. Each owner edit goes into a log as a ruling. The owner answers the clean agents' questions after the run (ticket 21), not during it. | run |

**Requirement answers (Q11, Q12).** The owner treats pointer, anchor and phase-order rules as implementation, so the spec states behaviour at the level a player observes:

- **Swipe switch:** "After a swipe switch, the game is as that Swipe's generation left it." This one sentence must be in the spec, because restoring the world is not universal: SillyTavern restores only the text (research 13), Marinara restores the world (research 14), and `swipe_switch.md` is silent. `gg01` decision 3 is module shape and stays out. The conflict between ruling 1 and `gg01` was never a requirement conflict (research 16 §4.2).
- **Retry:** prior decision 6 goes in as written: "A new Swipe redoes the last generation, with its stored inputs." The rewind point is left to the design. If the two clean agents read the sentence differently, ticket 21 closes the gap.

**Coordination with `architecture-deepening` (research 16 §5.2): pause, by owner intent.** The owner will not work on that map until this effort ends, so its map gets no edit and no marker.

**Scope change: the language and platform.** The owner asked whether to rewrite the whole engine in TypeScript or another language, because the Rust build and test loop is slow on this host and performance does not matter much for this application. That question is beyond this map's Destination, so it moved to a new map, [platform choice](../../platform-choice/map.md), as a skeleton. The owner says the answer will be clearer after the clean design. The new map's Destination is the draft from Q7. Its seed asset holds the build facts gathered in this grilling. The owner named the problems that a language move must solve: the edit-to-test loop, parallel agents waiting for the build slot, and design churn and defects (Q4 = A, B, D). The full gate time is not one of them: about 80 % of it is the browser tier, and a new language keeps most of that.

**Tickets created and wired:**

- [Write the behaviour spec for the game-state lifecycle](18-write-clean-behaviour-spec.md): task. On the frontier.
- [Review the spec for leaks and sign off its requirements](19-review-clean-spec.md): task. Blocked by 18.
- [Run two walled clean designs from the spec](20-run-walled-clean-designs.md): task. Blocked by 19.
- [Answer the clean agents' questions and close the spec gaps](21-close-clean-spec-gaps.md): grilling. Blocked by 20.
- [Write the target design of the game-state lifecycle in an open session](22-write-target-design.md): grilling. Blocked by 21.
- [Compare the target design with the code](23-compare-target-design-with-code.md): research. Blocked by 22.
- Tickets 01, 02 and 03 are each blocked by 23.
