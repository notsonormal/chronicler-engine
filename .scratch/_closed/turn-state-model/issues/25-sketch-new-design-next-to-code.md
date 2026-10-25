# Sketch the new design next to today's code

Type: prototype
Status: resolved
Blocked by: 23

## Question

What does the [target design](../assets/target-design.md) look like as a code shape, area by area, next to the shape of today's code? The sketch is a rough artifact for the owner to react to. [Decide where the new design of the game-state lifecycle makes sense](24-decide-fix-or-rewrite.md) judges it area by area.

The sketch does not choose the final code shape. The target design leaves the code shape open (D-16). The [platform choice](../../platform-choice/map.md) map decides how and where the result is built.

## What the sketch shows

The owner chose this in the grilling of ticket 24 (2026-10-12, Q8 = A and Q9 = A).

- **Terms:** today's Rust crate: module paths, types and traits (Q9). The platform-choice map can translate the sketch later.
- **Scope:** the "designed" scope of ticket 24 (Q7 = A):
  - the work of every player request: Action, Retry, Retrigger, `/options`, swipe switch, edit, delete, stop, create, reset and Game switch;
  - the records, the writer rule, the status, the order of a generation's steps, and what each view reads.
  
  The edges show up only as seams with their contracts (`clean-spec.md` §3): the narrator call, the Quantifier, the Options call, the world rules and Triggers, prompt assembly, the storage engine and the LLM call log. The sketch does not design their inside.
- **Six areas:**
  1. the records and their storage;
  2. the writer rule and the guard: the work entry, the per-Game guard, the work id, and the Game id on each write;
  3. the work and its steps: the plans and the generation steps of design §4.2 and §4.3;
  4. the reads of the views: story log, dock, status line, input box, header banner;
  5. create, reset and the opening;
  6. the dock and notices.
- **For each area:** the new modules, their interfaces and their seams to the edges, in `/codebase-design` terms. Next to them: today's modules for the same job ([research 12](../assets/research-12-pipeline-shape.md) §1).
- **Two walk-throughs, each in both shapes:**
  - a free-text Action that fires a Trigger;
  - a Retry, followed by a swipe switch.
- **Forks.** Where the shape has a real choice, show both options briefly, with the trade-off. Known forks:
  - one step runner, or one function per request (ticket 22, round 2, Q4);
  - a derived current state, or a live record plus copies (design D-01, against GLM, Kimi and today's code);
  - a busy mark in memory only, or a saved one with a start-up pass (design D-05, against GLM and Kimi).
- **Fate table.** Each of today's modules in the scope is kept, changed, replaced or deleted. Give its non-test line count. The modules are those of research 12 §1a, plus the storage of Messages, Swipes, Snapshots and Games, the state part of `domain/model/state/`, and the handlers of the routes in scope.

## Inputs

- [`target-design.md`](../assets/target-design.md): records §2, writes §3, work §4, stop §5, views §6, decision log §9.
- [`clean-spec.md`](../assets/clean-spec.md) version 3: §2 (the HTTP interface) and §3 (the black-box contracts).
- [Research 12](../assets/research-12-pipeline-shape.md): §1 (module map, seams, depth), §2 (entry paths), §3 (saves).
- [Research 23](../assets/research-23-design-vs-code.md): §4 (which function writes each state today). §6 shows where the code changes. Use it as a map of the code, not as a fix list.
- The three walled designs, for the forks: [GLM](../assets/clean-design-glm.md), [Kimi](../assets/clean-design-kimi.md), [Opus](../assets/clean-design-opus.md).
- [Research 04](../assets/research-04-generation-snapshots.md) (the save sequence) and [research 05](../assets/research-05-game-state-writers.md) (the writers), when an area needs them.

## Context

- **Why this ticket exists.** In the grilling of ticket 24 the owner said: "The point is to present some kind of new design and see where it makes sense. How I am supposed to know whether it is better or not if we don't at least sketch out the new design". The verdict waits for this sketch.
- **The comparison is of shapes, not of points.** The owner: "The code is the shape right? These are just 'points on a graph'." So the sketch puts whole areas side by side. It does not list single points of difference.
- **Not a fix list.** The behaviour differences of research 23 (its "code defect" rows, the 11 "decide first" rows of §5.3, the failure lines) are not part of this question. The sketch follows the behaviour of the target design. Where a behaviour choice changes the shape, name it.
- **Prototype, so human in the loop.** Show the owner the areas as they take shape, and take the owner's reactions before the asset is final. Thin code stubs (the `/prototype` skill) only if an area stays unclear; the owner left them out (Q8).
- **Terms:** the design's terms for the new shape (Swipe state, start state, current state, work, work entry, dock record, event record). Snapshot for today's code.
- **Skills:** `/codebase-design` (its `DESIGN-IT-TWICE.md` if a fork needs more than a short note); `/domain-modeling` if a term changes.

## Output

`../assets/sketch-25-new-design.md`

## Done when

- The asset covers the six areas, the two walk-throughs, the forks and the fate table.
- Each new module names its interface and its seams. Each area names today's modules for the same job.
- The owner reacted to the sketch, and the Answer records the reactions.

## Answer

Resolved on 2026-10-14. The asset is [`sketch-25-new-design.md`](../assets/sketch-25-new-design.md), draft 3. Draft 1 and draft 2 went to the owner. The owner's reactions are in sketch §9, and the rulings are in sketch §5.

**The sketch** (draft 3) covers the six areas, the two walk-throughs in both shapes, the seven forks and the fate table:

- Today's files in scope hold 7,289 non-test lines (whole files, known). By fate: 2,750 replaced, 827 deleted, 3,712 changed, 0 kept.
- The new modules are about 1,765 lines (guessed). About 580 of them are code that moves with no change of logic.
- A free-text Action that fires a Trigger saves about 10 whole Snapshot rows today, with no transaction. In the new shape, it saves 7 save units, each one transaction, each with only what changed.

**Reactions to draft 1:**

1. Pass ids and save at once. Do not populate and pass around large objects.
2. Enforce the design with code structure and guardrails.
3. Each guardrail explains its reason in a comment, not with a decision number.
4. Prefer less code. The owner asked whether draft 1 was a like-for-like copy of today's modules. Draft 2 removed the twins that failed the deletion test.

**Grilling of draft 2** (Q1–Q9):

| Q | Topic | Ruling |
|---|---|---|
| Q1 | Fork 1: where the order of a request lives | Not decided in the design. The build plan picks the code shape. One constraint: one `generate` for every kind of generation. |
| Q2 | Persistence | The three-line rule: between steps, pass ids, never loaded records; a step loads what it needs when it starts; a save writes only what changed. It must be carried by the code structure and guardrails, not by a written list. |
| Q3 | Fork 5: the storage write | One repository per record. Record writes exist only inside a transaction. Each save unit is one `GameWriter` or `JobWriter` method. |
| Q4 | Fork 7: the guard | One guard for all Games. It does not block two Games' Jobs at the same time. Switching between Games while each Job runs is in the design. Side by side in two tabs is not, on the condition that it stays cheap to add: only `CurrentGame` reads the active-Game pointer. |
| Q5, Q7 | The term | **Job**, judged on its merits. "Action Pipeline" goes. "Turn" stays deprecated: one Job holds several Messages, and a Retry and an Options request are Jobs but not turns. |
| Q6 | How the code carries the persistence rule | The structure plus three guardrails: a type check, a call count (at most one LLM role call per function in the Job folder) and a call site (only `GameWriter` opens a transaction). |
| Q8 | Guardrails | Part of the design. The build writes each guardrail before or together with the code that it guards. |
| Q9 | Two guardrails that check nothing today | An issue outside this map: [Make the two path-limited guardrails check real files](../../retro-issue-bucket/issues/50-dead-path-guardrails.md). |

Forks 2 and 3 were not asked again: the owner ruled on them in ticket 22 (the current state is derived; the busy mark is in memory only).

**Open:** fork 4 (one `LlmRoles` trait in place of the `Agent` trait, `AgentRegistry` and `ExecutionPhase`, or no new seam). [Decide where the tests of the new design sit](26-decide-test-placement.md) decides it.

**For ticket 24** (proposals from the rulings above; ticket 24 decides):

- D-07: "a per-Game guard" becomes "a guard".
- D-16 gains the one-`generate` constraint.
- The design's "work" becomes Job, and `CONTEXT.md` gets the term with the verdict.
- The 15 rules of sketch §6.2 are part of the design.

**For the platform-choice map:** side by side in two tabs is a wish. The guardrails at compiler level depend on Rust.

**For ticket 26:** draft 3 changed rows that its proposal checked against draft 2. These are the storage seam (area 1), the persistence rule (§2.1) and the guardrail numbers (§6.2: "one generation path" is now rule 13, and rules 7–9 are new).
