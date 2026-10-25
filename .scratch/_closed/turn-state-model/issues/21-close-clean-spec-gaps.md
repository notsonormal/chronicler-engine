# Answer the clean agents' questions and close the spec gaps

Type: grilling
Status: resolved
Blocked by: 20

## Question

For each row of `../assets/clean-run-gaps.md`, does the owner require a behaviour, or is the point a free choice that the design may make? This grilling is step 4 of the clean-room route. Its answers make the spec complete enough to design from, and, after a rewrite, complete enough to test against.

## Context

- The gaps come from [Run three walled clean designs from the spec](20-run-walled-clean-designs.md): the assumptions of each clean design, their questions, and the points where the designs disagree. The Opus design is from the spec writer's family, so a gap that only the GLM or Kimi design shows still counts.
- The 17 open questions of `../assets/clean-spec.md` §6 (OQ-01 to OQ-17) are gaps too. Each clean design answers or asks each one, so each must have a row in `clean-run-gaps.md`. OQ-01 and OQ-02 are conflicts between two sources, not silences. The Answer of [Write the behaviour spec for the game-state lifecycle](18-write-clean-behaviour-spec.md) maps each OQ to its origin.
- The owner treats pointer, anchor and phase-order rules as implementation (ticket 17, Q11 and Q12). Answer each gap at the level that a player observes.
- The agent does not answer for the owner. Facts that the code answers are not grilled. Look them up.
- **Skills:** `/grilling` and `/domain-modeling`.

## Done when

- Each gap is answered, marked "free" (the design chooses), or marked open with a reason.
- Each owner answer goes into `../assets/clean-spec-rulings.md`.
- `../assets/clean-spec.md` has a new version, and the Answer lists its changes.

## Answer

**Every gap has an answer.** The owner answered all 38 rows of `../assets/clean-run-gaps.md` and the 17 open questions in seven grilling rounds (Q1–Q31). One point is "free": B-07, how the engine keeps an event's analysis from moving the player. No point stays open. The answers are rulings OR-04 to OR-50 in [`clean-spec-rulings.md`](../assets/clean-spec-rulings.md).

**The central ruling (OR-04).** The owner said: "In principle the state is allocated to the swipe." In the spec this is R-39: when no work runs, the game state is the state that the active Swipe of the last Message left, other than the offered set. A generation, a swipe switch, a delete, a failed Retry and a stop all keep this rule. Many other answers follow from it: A-17, B-01, B-03, B-05 and the offered set after a delete.

**Owner choices against the recommendation.**

| Gap | Recommendation | Owner ruling |
|---|---|---|
| A-10, a World with no opening text | an automatic continue at creation | OR-06: an opening Narration with empty text. The player can Retry it "like any message". This led to OR-23 (a Retry on any last Message) and OR-24 (a Retry of the opening is a Narration generation). |
| B-04, a failed event after an Impersonate | no Retrigger on an Input | OR-27: Retrigger is offered on that Impersonate Input. |
| A-07, `POST /status/reset-generating` | remove the route | OR-30 and OR-31: keep it as a stop. The stop undoes the whole request, and the send button becomes a Stop button while the Game is busy. |

**Rulings that no design proposed:** OR-24 (a Retry of the opening), OR-28 (the only Message cannot be deleted), OR-31 (a stop undoes the whole request), OR-32 (option D for the failure line), OR-48 (after a stop, the input box holds the player's text again).

**Rulings that differ from today's code** (known from the lookups in this session, unless marked): OR-04 (a delete keeps the latest state today), OR-06 (today the process writes an LLM opening at each start), OR-09 and OR-49 (today notices are `[System]` Messages in the story), OR-23 (today a Retry needs an Input or a guided Narration), OR-27, OR-29 (today an edit and a delete run while the Game is busy, and a swipe switch answers `503`), OR-31 (today the route frees the busy mark and the work continues), OR-33 (today the `400` check runs first), OR-34 and OR-35 (today `[System]` rows), OR-36 (inferred), OR-37, OR-40 (today an option's text is read as a command). Ticket 23 classifies each one.

### `clean-spec.md` version 2

- Version 1 is kept as [`clean-spec-v1.md`](../assets/clean-spec-v1.md), because the three designs and the gap list use its numbers.
- **Counts:** 93 requirements (81 before), 6 environment constraints, 0 open questions. §6 became a table that maps each OQ to the requirement that answers it. A new §7 lists the free points.
- **New requirements (12):** R-17a (the trigger context of each generation), R-21a (what clears a failure), R-25a (the room report of the opening), R-39 (the game state belongs to the Swipe), R-43 (a Retry of the opening), R-74a (leaving a busy Game), R-81 (the stop), R-84 (a notice is not a Message), R-85 (the notice shows in the dock), R-86 (R-83 gives the only notice), R-87 (the header line for a failed LLM role), R-93 (cut-off work is not reported).
- **Changed requirements:** R-01, R-06, R-09, R-14, R-16, R-17, R-18, R-19, R-20, R-21, R-22, R-23, R-27, R-30, R-31, R-32, R-33, R-36, R-38, R-41, R-44, R-45, R-62, R-65, R-66, R-70, R-76, R-77, R-83.
- **Other sections:** the terms Message, Opening Narration, Swipe, Game state, Status, Busy, Trigger context, Retry and Notice, and a new term Stop; the HTTP table (the busy answers, the `400` for no Game, the stop, the option mark on `POST /action`); §3.2 (OR-44); §3.4 (OR-45); §3.7 (the Delete and Retrigger controls, the notice in the dock, the Stop button, the header line). Each change cites its ruling as `[OR-n]`.
- **Checks.** The leak check (`research-15-leakcheck.py`) finds 0 `src/` names, 0 `src/` paths and 0 `.rs` names (tested). A grep finds each ruling from OR-04 to OR-50 cited at least once (tested). The spec writer checked each rewording against its ruling, as the ticket-19 lesson asks. No independent reviewer read version 2.
- `clean-run-prompt.md` is not changed. It still tells a clean agent to answer the open questions. A new walled run from version 2 must update it first.

### The glossary

`CONTEXT.md` **Swipe** now reads: "One of the texts of a Message. Every Message has at least one Swipe. A Swipe carries the inputs of its generation and the game state that its generation left." The owner approved "One version of a Message". The entry says "One of the texts" instead, because its `_Avoid_` list bans "version".

These glossary changes describe the target and not today's code. They go to the plan after ticket 23:

- **Message:** remove "or system log". A notice is not a Message (OR-09).
- **Options:** "after a non-impersonate narration message" becomes "after the last Narration that the work saved" (OR-37).
- **New entries:** Stop (OR-31) and Notice (OR-49).

### Feature specs that the rulings contradict

The plan edits these. This ticket does not edit them.

| Feature spec | Scenario | Ruling |
|---|---|---|
| `docs/specs/swipe_new.md` | 11.1 (no Input → `400`) | OR-23 |
| `docs/specs/swipe_new.md` | I.2 (a Retry never adds a Message) | OR-08 |
| `docs/specs/retrigger.md` | 14.3 (an Input as the last Message → `400`) | OR-27 (except an Impersonate Input) |
| `docs/specs/options.md` | 24.5, 24.17, 24.18, 24.20 (a System message in the story log) | OR-09, OR-49 |

The rulings also need new scenarios that no feature spec has: a delete that gives back the state (OR-04), the busy answers of edit, delete and swipe switch (OR-29), the stop (OR-31), the dock after a failed event and after an Impersonate's event (OR-36, OR-37), a Retry of the opening (OR-24), and the notice in the dock (OR-49).

### For ticket 22

- R-39 is the invariant that the target design must keep. R-81 adds a constraint: a stop undoes saves that the work already made, but a restart keeps them (R-93, E-02). The design must support both.
- R-85 keeps a notice after a restart, and R-21a keeps a failure line after a restart. So both live in storage, not only in memory.

### Lessons from the grilling (known)

- Questions with sub-parts (a), (b), (c) and no lettered options did not work (Q14, Q17, Q18). The owner answered by option letter only when each question held one decision with Option A and Option B.
- Spec terms that the owner does not see in the product needed a second wording: the "trigger context" became "the ⚡ Retrigger Event button" (Q8), and the "header banner" became "the one-line note in the dashboard header" (Q30). Name the control that the player sees.
- One description of today's code was wrong in round 7 (the header line). A lookup in `GameViewQuery::role_health` corrected it before the owner ruled.
