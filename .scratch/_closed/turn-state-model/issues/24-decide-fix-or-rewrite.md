# Decide where the new design of the game-state lifecycle makes sense

Type: grilling
Status: resolved
Blocked by: 23, 25, 26

**Reframed on 2026-10-12** in the first grilling session (Comments). The first title was "Decide: targeted fixes or a rewrite of the game-state lifecycle". The file name keeps the old slug, so the links stay valid.

## Question

Where does the new design make sense, area by area, and where does today's design stay? The new design is the [target design](../assets/target-design.md), sketched as a code shape next to today's code in [Sketch the new design next to today's code](25-sketch-new-design-next-to-code.md). The verdict covers only the designed scope (below). The [platform choice](../../platform-choice/map.md) map decides how and where the result is built.

## Scope

The owner chose this on 2026-10-12 (Q7 = A).

| Group | What it holds | Verdict here |
|---|---|---|
| Designed | The work of every player request: Action, Retry, Retrigger, `/options`, swipe switch, edit, delete, stop, create, reset and Game switch. Also the records, the writer rule, the status, the order of a generation's steps, and what each view reads. In code: `pipeline/`, `message_service.rs`, `generation/`, `arrival_service.rs`, `init_game.rs`, `games/`, the storage of Messages, Swipes, Snapshots and Games, the state part of `domain/model/state/`, and the handlers of those routes. That is about 6,600 of 21,900 non-test lines (rough, inferred). | yes |
| Edges, contract only | The narrator call, the Quantifier, the Options call, the world rules and Triggers, prompt assembly, the storage engine and the LLM call log | only their contracts, as the edge of the designed scope |
| Not touched | The LLM transport, Settings, prompt presets, Worlds, Personas, Characters, the text check, templates and CSS, and the build and test tooling | none |

The platform-choice map decides whether the edges and the untouched parts get a clean design of their own (Q4 = A, Q7 = A).

## Inputs

- **The sketch:** ticket 25 and its asset.
- **Test placement:** [Decide where the tests of the new design sit](26-decide-test-placement.md). Its answer is part of the cost of a redesign. The owner added it on 2026-10-13.
- **The evidence from session 1:** the shape points and the evidence for and against a redo (Comments).
- [Research 12](../assets/research-12-pipeline-shape.md): the module map, the churn and the defect history.
- [Research 15](../assets/research-15-clean-room-method.md): §1, the limits of agreement between designs.
- [Research 23](../assets/research-23-design-vs-code.md): §4 and §6, where today's code holds each state and where it changes.

## Context

- **Not a fix list.** The owner: "The original idea was not to 'fix bugs' to use the clean room design to determine if we need to redo the design of at least part of the application. Defects or whatever... that's just becaused of a difference of opinion in the dozens of questions I was asked during the grilling process." So the behaviour differences of research 23 (its "code defect" rows, the 11 "decide first" rows of §5.3, the failure lines) are not part of this verdict. They stay recorded in research 23.
- **Judge whole areas, not points.** The owner: "The code is the shape right? These are just 'points on a graph'." The owner then chose one decision for the whole scope (Q5 = A), and then asked for a sketch to judge: "The point is to present some kind of new design and see where it makes sense."
- **`architecture-deepening`** is paused by owner intent. Its tickets 04, 05, 08, 09 and 10 deepen today's modules in the designed scope. The verdict says what happens to each of them.
- **Terms:** use the design's terms (Swipe state, start state, current state, work) for the new design, and Snapshot for today's code.
- **Skills:** `/grilling`, `/domain-modeling`, `/codebase-design`.

## Done when

- For each area of the sketch, the answer names one of: the new design, today's design, or a mix with the line stated. Each choice has its reason.
- The answer says what the platform-choice map takes from this map.
- The answer says what happens to `architecture-deepening` tickets 04, 05, 08, 09 and 10.
- Nothing about the design of the designed scope is left to decide.

## Comments

### Session 1 (2026-10-12): grilling, then blocked by ticket 25

The session claimed the ticket, read the inputs and grilled the owner in seven rounds. It then set the ticket back to open, blocked by the new ticket 25.

**Owner answers:**

| Q | Question | Answer |
|---|---|---|
| R1 Q1–Q5 | Order with the platform choice; close 01–03; rulings on the 11 "decide first" rows; the first red test for a restored Swipe state | Withdrawn. The owner rejected the fix framing (Context). R1 Q2 came back as R2 Q3. |
| R2 Q1 | Reframe the ticket and the Destination | A: "does the design need to be redone, which parts, and to which shape". The behaviour differences are not part of it. |
| R2 Q2 | Which shape points need a redesign | The owner rejected the point-by-point view: "The code is the shape right? These are just 'points on a graph'." |
| R2 Q3 | Close tickets 01, 02 and 03 | Close. |
| R2 Q4 | Where this map stops | A: this map ends with the redesign verdict. The platform-choice map decides how and where it is built, and whether other parts get a clean design. |
| Q5 | How to judge | A: one decision for the whole scope, with the shape points as evidence taken together. |
| Q6 | Redo or keep | Not answered. The owner asked first what "the lifecycle" means (Q7), and then for a sketch: "How I am supposed to know whether it is better or not if we don't at least sketch out the new design". |
| Q7 | What the verdict covers | A: the designed scope, with the edges as contracts (Scope). |
| Q8 | What the sketch shows | A: one sketch by area, side by side with today's modules, two walk-throughs, the forks, a fate table. |
| Q9 | Terms of the sketch | A: today's Rust crate. |
| Q10 | How the map carries it | A: new ticket 25 (prototype) blocks this ticket; this session records the decisions and stops. |

**The first question, kept for history.** "Does the plan after this map fix today's turn pipeline in place, or rewrite it to the target design? For targeted fixes: in which order do the change clusters go …? For a rewrite: which modules does it replace …? In both cases: what does the new test for a restored Swipe state assert?" The Done-when asked for an order of clusters, a first red test for each slice, and a ruling for each of the 11 rows of research 23 §5.3. The reframe dropped all of these.

**Evidence for the verdict, from session 1.** These are points where three walled designs differ from today's code. The owner ruled that they are evidence taken together, not decisions one by one.

| # | Shape point | Today's code | The three walled designs |
|---|---|---|---|
| S1 | What a Swipe holds | a pointer to a copy saved part-way through its generation | all three: the state after the whole generation |
| S2 | Where the status lives | inside every saved copy of the game state | all three: apart from the state copies |
| S3 | Which Game a write goes to | one process-wide Game id, read at each storage call | all three: the work keeps the Game that it started with |
| S4 | Who can write during work | only other generations are blocked; edit, delete and swipe switch write freely | all three: every writer is blocked (Kimi and Opus also check a work id) |
| S5 | How one generation saves | up to 5 separate storage calls | all three: one save unit |
| S6 | How the steps are ordered | 5 ordering functions for 10 entry paths, plus a separate arrival path | all three: one step sequence for every kind |
| S7 | The current state | a live record plus copies | split: GLM and Kimi as the code; Opus derives it |
| S8 | The busy mark | saved, with a heal step | split: GLM and Kimi save it and clear it at start-up; Opus keeps it in memory only |

- **For a redo:** the walled designs agree on S1–S6, and today's code differs on each of them. These points sit at the core: they touch every ordering function and most of the 24 save sites (research 23 §6). Research 12 §4 shows 108 commits on these files in five months, 63 of them refactors, and a reshape about once a month. Most fixed and open defects come from when a pointer is set and who writes state.
- **Against a redo:** the three designs share one spec and similar training data, so their agreement is weaker than it looks (research 15 §1). S1, S3 and S5 follow from stated requirements (R-36, R-74, R-90). S2, S4 and S6 have no requirement behind them; the walled designs chose them before ticket 21. On S7 and S8 today's code sits with the majority. The pipeline's 124 unit-test functions (5,132 lines) are written against today's internals. The HTTP tests go through the routes.
- **Where the fix framing came from** (for the record): the seed's three defects, the Destination "targeted fixes or a rewrite", ticket 17's Answer ("a reference for fixes"), and the "code defect" class of research 23.

### Session 2 (2026-10-14): grilling, resolved

Status set to `claimed`. Inputs: sketch 25 draft 3, ticket 26's Answer. New asset: [sketch 24: Agent hooks](../assets/sketch-24-agent-hooks.md) (fork 4, option B, asked for by the owner).

| Q | Question | Answer |
|---|---|---|
| Q1 | The verdict for each area | **A:** the new design for all six areas, in today's module homes where they exist (`gate.rs` → `GameWriter`, `GameViewQuery`, `GameCatalogue`). |
| Q2 (round 1) | Fork 4: `LlmRoles`, no seam, or a concrete module | Not answered. The owner: "I'm not sure how this would work if I had multiple agents, some pregeneration and some post generation, maybe even background agents. [Marinara] has a huge amount of agents … I want to get reasonably supportive of it." Asked again as hooks (sketch 24). |
| Q8 | The name of step 5 ("world rules") | **A: game rules** (`apply_game_rules`). The owner: the World is the template; the Game is the playthrough. `CONTEXT.md` has the entry. |
| Q9 | Who decides which Agents run in which generation kind | **B:** each Agent declares its kinds (default: every kind). R-14 and R-18 stay in `generate` and the game rules. |
| Q2 (round 2) | The Agent seam: four fixed hooks, one trait and output type per hook | **A** (after the owner asked for the sketch). |
| Q7 | Build all hooks now, or only those with an Agent | **B:** only analysis and Options now. |
| Q6 | Background Agents that outlive the Job | **C:** not now, kept cheap to add. |
| Q3 | What the Agents read | **A:** the step loads each Agent's settings and passes values. |
| Q4 | Changes to the target design and the glossary | **A:** all five. |
| Q10 | The `architecture-deepening` tickets | **A:** close 04, 05, 07, 08, 09, 10 and 13; notes on 03, 11, 12 and 16. |
| Q11 | The handoff to the platform-choice map | **A.** |
| Q12 | Update the target design now | **A:** version 2. |
| Q13 | An ADR | **A:** not now; the build plan writes it. |
| Q14 | Nothing left to decide | **A:** confirmed. |

The owner also asked whether one Job holding the Narration, the event generation and the Options run is how today's pipeline works. Yes (known): one `spawn_blocking` task under one generation slot runs all of them (`core.rs:196`, `:333`, `:364`). Job names that unit; the change is in shape (one `generate` in place of a separate event path and `retry_event_continuation`).

Facts found (known): one Job holds the Narration, the event generation and the Options run, under one slot today too (`core.rs:196`, `:333`, `:364`). Marinara (clone `c5650149`): three phases, 32 result types applied by the host, tracker rows keyed by `(messageId, swipeIndex)`, post-processing runs that outlive the reply.

## Answer

Resolved on 2026-10-14 (session 2). The design is [`target-design.md`](../assets/target-design.md) **version 2** (version 1 kept as [`target-design-v1.md`](../assets/target-design-v1.md)), with its code shape in [sketch 25](../assets/sketch-25-new-design.md) (draft 3) and, for the Agents, [sketch 24](../assets/sketch-24-agent-hooks.md).

### The verdict, by area

**The new design makes sense for the whole designed scope.** Today's design stays only outside it: the edges keep their inside, and the untouched parts do not change.

| Area | Verdict | The line | Reason |
|---|---|---|---|
| 1. Records and storage | **new** | — | R-39 (the game state belongs to the Swipe) has no home in a pointer to a copy saved part-way through a generation. |
| 2. Writer rule and guard | **new, in today's file** | `gate.rs` becomes `GameWriter` and `JobWriter` | R-70 and R-74 need a guard around every save. Today no save runs under the guard. |
| 3. The Job and its steps | **new** | — | Areas 1 and 2 change every save site in these files (research 23 §6). Keeping them means rewriting them anyway, and keeps the five ordering functions that research 12 links to most defects. |
| 4. View reads | **mix** | `GameViewQuery` stays as the module; its reads follow area 1 | The module is the right home; only its inputs change. |
| 5. Create, reset, opening | **new, in today's `GameCatalogue`** | — | D-25 and OR-06 removed the LLM opening; one create unit replaces three writers. |
| 6. Dock and notices | **new** | — | R-82 to R-86 forbid notices in the story. |

The session-1 evidence "against a redo" no longer holds. In session 1, S2, S4 and S6 had no requirement behind them. Each has one now: D-05 with R-21a and R-72, R-70, and the "one `generate`" ruling (ticket 25 Q1). The areas are coupled: the fate table keeps 0 of today's 7,289 lines unchanged.

### Fork 4: the Agent seam

The owner wants the engine to be reasonably supportive of many Agents, as in Marinara (pre-generation, post-generation, perhaps background). So the sketch-25 `LlmRoles` (three fixed methods) is replaced by **Agent hooks** ([sketch 24](../assets/sketch-24-agent-hooks.md); design D-29 to D-34):

- Four fixed hooks in `generate`: before narration, rewrite, analysis, the Options run. Each has one trait and one output type, so the compiler holds where an output lands. Agents return values and write no game record. The narrator is not an Agent.
- Each tracker Agent owns one named part of the Swipe state, so a Retry and a swipe switch treat tracker values like the rest of the state.
- Each Agent declares the generation kinds that it runs for (Q9). R-14 and R-18 stay in the steps and the game rules.
- Only the analysis and Options hooks are built at first (Q7). Background Agents are not built now, and the design keeps them cheap to add (Q6).
- The step loads each Agent's settings and preset and passes them as values (Q3).

### Changes to the design and the glossary

- [`target-design.md`](../assets/target-design.md) version 2:
  - D-07 is one guard for all Games;
  - D-16 gains "one `generate` for every kind";
  - "work" becomes **Job** (D-27);
  - "world rules" becomes **game rules** (D-28);
  - D-29 to D-34 are the Agent decisions;
  - D-35 makes the persistence rule and the guardrails (sketch 25 §6.2, changed by sketch 24 §4) part of the design;
  - AD-07 and AD-13 are now settled.
- `CONTEXT.md`:
  - new: **Job**, **Game rules**;
  - changed: **Agent**, **Action**;
  - **Action Pipeline** is deprecated ("use Job");
  - **Snapshot** stays until the build removes it.
- Sketch 25 has a header note that points to sketch 24.
- No ADR now (Q13). The build plan writes one when the code lands, and links this map.

### What the platform-choice map takes

Written into its Notes (Q11): the verdict, the cost (3,577 lines replaced or deleted; about 1,845 new, guessed; about 7,400 unit-test lines go; the tier-1, storage and browser tiers stay), the wishes that must stay cheap (side by side in two tabs; background Agents; many Agents), what depends on Rust (the compiler-level guardrails and one trait per hook), and what it decides (how and where the design is built, clean designs for the edges, the test oracle).

### `architecture-deepening`

| Ticket | Fate |
|---|---|
| 04, 05, 08, 09, 10 | resolved: rejected, superseded. Each deepens a module that the new design replaces or deletes. |
| 13 (registry), 07 (preset) | resolved: decided here (D-29, D-33) |
| 03, 11, 12, 16 | open, each with a note: the verdict touches them only at an edge |

Its Priority table and Decisions so far are updated. That map stays paused until the platform choice lands.

### What stays open, and where

Nothing about the design of the designed scope (Q14). These stay open by ruling or as edges: the code shape of the order of a request (the build plan, ticket 25 Q1); repositories as traits or methods (`architecture-deepening` 03); the feature-spec conflicts and glossary changes of ticket 21 (the plan); the scenarios that ticket 26's rule requires (the build plan); the Agent catalogue, batching, per-Game enabling and how trackers show (the Agent edge and the views).
