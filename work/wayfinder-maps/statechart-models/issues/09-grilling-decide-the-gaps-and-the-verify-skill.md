# Decide the gaps of the first run, and the future of the verify skill

Type: grilling
Status: resolved
Blocked by: 07

## Question

The first run of `verify-gate` (ticket 07) wrote its gaps to [the trial log](../assets/trial-log.md). Ticket 07 creates that file. For each gap, what happens? And does `verify-gate` stay in the repo?

1. **Each gap, by kind.** A code bug, a spec gap, a test gap, or a doc error. For each: fix it, file it in the map that owns its code, or leave it. The trial landed nothing (ticket 04, Q12), so every gap is still open.
2. **The verify skill.** Keep `.agents/skills/verify-gate/` and the two pstack skills, or remove them.
3. **Ticket 08.** If the skill stays, ticket 08 adds the staleness warning to `build.py`: set its `Blocked by:` to `—`. If the skill goes, resolve ticket 08 as not needed.

## Context

- **The decision is from the first run only** (owner, ticket 04, Q15). The trial log has no maintain pass, so this ticket has no data about the upkeep. Known so far: `gate.rs` and `job.rs` each had about one commit a day from 2026-10-05 to 2026-11-05, so a kept TLA+ model needs a maintain pass about once a day.
- **Known gaps from ticket 03**, which the first run most likely finds again: six candidate tests (G1, G3, R05, R07, R08, R09), the words "a Job of the run" in the intro of `docs/specs/autoplay.md` (G3), and a spec scenario for the delete of a Game during a plain Job. Sections 6, 7, 8 and 11 of [the ticket 03 asset](../assets/03-gate-spec-only-model.md).
- **The owner's leanings** (2026-10-30): `game_flow.md` need not cover every use case, because the specs hold the event rules. A TLA+ model is for finding gaps.
- Use `/grilling`. The owner answers for themselves.

## Comments

### 2026-11-06: the code gap is wider than run 1 says, and its fix mutant is not a fix

Found while this ticket read the gaps. The scratch TLA+ model and its configs are in `tmp/t09/`.

- **Two locks (known, a read of the code).** `ActiveGame` has its own mutex. `BusyRegistry::hold` locks the `busy` mutex only. `GameCatalogue::switch_game` reads the Game (`storage.get_game`) and then calls `ActiveGame::set`, outside every hold. `ActiveGame::set` calls `touch_game_updated_at`, an `UPDATE` that answers `Ok` when no row matches.
- **A model gap (known).** `VerifyGate.tla` makes `YouSwitchTheGame` one step, but the code has two steps. So the TLA+ model misses every order in which a delete comes between the Game read of a switch and its pointer set.
- **Three orders leave the active pointer on a deleted Game** (known: TLC on the scratch variant, with the new rule `ActiveExists == pending.reset = 0 => active \in exists`, from games.md 18.2 and 19.2):
  1. The delete check passes, a switch moves the pointer, the delete removes the Game. Run 1 found this order (`DeleteNotActive`).
  2. A switch reads Game 2, a delete of Game 2 passes its check and removes it, the switch sets the pointer to Game 2. 5 states.
  3. A reset creates Game 3 in its hold, a delete of Game 3 passes its check (the pointer still names the old Game) and removes it, then `ActiveGame::set_new` moves the pointer to Game 3. 5 states.
- **The fix mutant `delete-check-inside-the-hold` closes order 1 only** (known: TLC breaks `ActiveExists` on orders 2 and 3 with the fix mutant on). The Answer of ticket 07 and the trial log say that a check inside the hold closes the race. That is not correct.
- **The harm (inferred).** Each later request of the dashboard names a Game that does not exist, until the person switches to a Game. A restart picks the newest Game of the World (`resolve_game_id`), so it also repairs the pointer. Each order needs two requests in a window with no `await`, so only a thread preemption reaches it.
- **The doc gap is also in the spec (known).** The intro of `docs/specs/autoplay.md` says "The run holds its Game from its start to its end, also between two Jobs". `game_flow.md` says the same. The rule `RunHoldsItsGame` quotes the spec sentence, and it skips `pc = "end"` for this reason.

### 2026-11-06: grilling round 1 (withdrawn by the owner)

Round 1 asked about the skill, the code gap, the home of the filed work, a default for the spec and test gaps, and the sentence about the span of a run's hold. The owner: "you are thinking too simply". The goals are (a) find bugs, (b) align the code, the specs and the docs, and (c) use TLA+ models as documentation, so that `docs/diataxis/` holds less prose, which is the least trusted and the hardest to maintain. Round 2 replaces round 1.

The owner on drift: a gap between a checkable TLA+ model and the code is useful information, not a defect. The code is the final source of truth, and each layer around it expands or explains the system.

### 2026-11-06: grilling round 2 (owner)

- **Q1, the Destination: Option A, widen this map.** The role of TLA+ models is decided for (a) bugs, (b) the alignment of code, spec and docs, and (c) documentation that replaces prose. The gate model replaces its part of `game_flow.md`. Each gap of run 1 is closed or left with its reason. The *Out of scope* line "A trim of `game_flow.md`" goes. TLA+ models of other areas go to *Not yet specified*.
- **Q2, a layer: Option A.** `docs/AGENTS.md`, *Where a fact lives*, gets a row: "The order of steps between parts that run at the same time, and the rules that hold in every order" live in the TLA+ model of a verify skill (`.agents/skills/verify-<area>/model/`). Specs keep the observable promises. Diátaxis keeps the names, the invariants and the reasons. Two comparisons of ticket 04 change: (3) a model branch that no request can observe needs no spec scenario, because the TLA+ model is its home; (5) `game_flow.md` restates no order that the TLA+ model holds, and it names the TLA+ model in one sentence. The gate sections of `game_flow.md` are cut to the names and the invariants, with a pointer to the TLA+ model. Rejected: the TLA+ model as a check of the prose only (B), and as the only reference of the gate, with the gate scenarios of `docs/specs/` (C).
- **Q3, the skill: Option A, keep it.** The agent that changes a covered function brings the TLA+ model along before it ends its task (ticket 04, Q10). A gap that it finds goes to the trial log as information, not as a failure. Ticket 08 is no longer blocked.
- **Q4, the code gap: Option A.** One bug ticket, low priority, closes all three orders that leave the active pointer on a deleted Game. That ticket decides the design. It also updates the TLA+ model: a switch in two steps, and the new rule `ActiveExists`.

### 2026-11-06: grilling round 3 (owner)

- **Q8, the `build.py` runs that show the warning: Option A, every run.** The check must read the working tree too, not only `git log -L <sync>..HEAD`, because agents do not commit without approval (`AGENTS.md`). Without that, a change to a covered function shows no warning until the owner commits it.
- **Q5 (the gap rule), Q6 (an invariant that the TLA+ model also checks) and Q7 (what a reader opens): not clear.** The owner did not follow Q5 and Q6. On Q7 the owner said that the TLA+ process makes the diagram from the TLA+ model by itself. Asked again in round 4.
- **A fact for Q7 (known, `tmp/t09/chart/`).** `/statechart` draws no chart when TLC counts more than 500 states, and the smallest verify-gate config (`runs`) has 769. A new config with 1 Game, 1 Action Job and stop has 134 states. With `VerifyGate.view.mjs`, `/statechart` draws it in 4 s as 6 chart states and 13 arrows, and every rule holds.

### 2026-11-08: grilling round 3, continued

- Q5 (revised): option A. A run-1 gap gets a new spec scenario and a test only when the client can see a different result. An order that no request can observe has the TLA+ model as its home.
- Q6: the owner rejected a sentence-by-sentence cut. `game_flow.md` gets a full rewrite, not a list of removed sentences.
- Q7: the owner rejected the framing around the 500-state limit of `/statechart`. The diagrams must be useful, and the tool can change to make them.

Diagram prototype (known, all in `tmp/t09/diagrams/`, the skill's model unchanged):

- The 500-state limit of `/statechart` is a limit on TLC states (`CHART_STATES` in `scripts/ir.ts`), not on the chart. With a view that groups states, the chart size is the number of view states. On a copy of the tool with the limit raised, the unchanged gate model gives:
  - the life of one Job: 134 TLC states, 11 chart states, 16 arrows;
  - the life of one Autoplay run (`runs.cfg`): 769 TLC states, 14 chart states, 18 arrows;
  - the busy state of one Game (`run-and-jobs.cfg` with Action, Autoplay and Stop): 60,976 TLC states, 4 chart states, 7 arrows, in 24 s.
- One view per independent axis. A view that mixed the holder and the failure line gave 8 states and 21 arrows, and it was hard to read.
- A state chart cannot show a race, because a race is an order of steps of two parts. A prototype script (`scenarios.py`) takes a named list of model actions, lets TLC take only those steps, checks every rule on the way, and draws the TLC trace as a Mermaid sequence diagram. A step that the model does not allow stops the script. So each diagram is a checked path of the model. Four scenarios drew: a stop during the narrator call, the late end of a stopped Job, the end of a stopped run, and the run-1 delete race (the diagram ends at the broken rule `DeleteNotActive`).
- Pele 1.9.0 renders both the state charts and the sequence diagrams.

### 2026-11-08: grilling round 4 (owner)

- **Q9, the diagrams: Option C.** The gate model generates three kinds of diagram: a state chart of each part (one view of one axis each), a sequence diagram of each named order (each a path that TLC checks), and a sequence diagram of the path of each broken rule, from the TLC counterexample.
- **Q10, the place: a folder of its own.** The generated diagrams go into `docs/diataxis/reference/diagrams/` (owner: "Might be better to put them in reference/diagrams or something"). `game_flow.md` links to them.
- **Q11, the tool: Option A.** Change `.agents/skills/statechart/`: the limit counts chart states after the view, and a new output draws a sequence diagram of a scenario or a counterexample. Rejected: a second renderer inside `verify-gate` (B).

## Answer

The owner widened the map, kept `verify-gate`, and decided where each gap of run 1 goes. The gate model also becomes documentation, through diagrams that are generated from it.

**The map (Q1, Option A).** The Destination now covers three goals of TLA+ models: (a) find bugs, (b) align the code, the specs and the docs, and (c) documentation that replaces prose. The gate is the first area. The *Out of scope* line about a trim of `game_flow.md` goes.

**The layer (Q2, Option A).** `docs/AGENTS.md`, *Where a fact lives*, gets a row for the TLA+ model of a verify skill (`.agents/skills/verify-<area>/model/`): "the order of steps between parts that run at the same time, and the rules that hold in every order". Specs keep the observable promises. Diátaxis keeps the names, the invariants and the reasons. A model branch that no request can observe needs no spec scenario.

**The skill (Q3, Option A).** `verify-gate` and the two pstack skills stay. The agent that changes a covered function brings the TLA+ model along before it ends its task. A gap that it finds goes to the trial log as information. A gap between the model and the code is useful information, and the code is the final source of truth (owner).

**The gap rule (Q5, Option A).** A gap gets a new spec scenario and a test only when a client can see a different result. An order that no request can observe has the TLA+ model as its home.

**Each gap of run 1** (numbers as in [the trial log](../assets/trial-log.md)):

| Gap | Decision | Ticket |
|---|---|---|
| Code 1, the delete races a switch. It has three orders (see Comments) | fix the code and the model | 10 |
| Model 1, the writes on an idle Game | add configs that reach them | 11 |
| Spec 1, a plain Job whose accept cannot read the story | scenario and test | 11 |
| Spec 2 and 3, a stop just before the drop | leave: the client sees the same result in every order | — |
| Spec 4, a stop clears the failure line on an idle Game | scenario and test | 11 |
| Spec 5, the Narration save clears the Options failure | scenario and test | 11 |
| Spec 6, a delete of a Game that is not active during a plain Job | scenario and test | 11 |
| Spec 7, the window between `GameWriter::replace` and `ActiveGame::set_new` | part of the code gap (order 3) | 10 |
| Spec 8, the `200` of a config write during a plain Job | one line in actions.md 6.4 | 11 |
| Test 1, `reset-keeps-the-holder` | leave: no request can name the old Game | — |
| Test 2, `stop-keeps-failure-line` | the test of spec 4 forces it | 11 |
| Test 3, `accept-without-run-check` | test, through `pause_between_two_jobs` | 12 |
| Test 4, `moved-job-ends-as-failure` | leave: only a log line shows it | — |
| Test 5, `DeleteNotActive` | a test only if a seam can force the order | 10 |
| Doc 1, the span of a run's hold | fix the sentence in the intro of `docs/specs/autoplay.md`; `game_flow.md` goes with its rewrite | 12, 15 |
| Ticket 03, G1 and G3 | scenario and test; "a Job of the run" goes into the intro of autoplay.md | 12 |
| Ticket 03, R05, R07, R08, R09 | leave: one check guards every save of a Job, and tests catch a break of it | — |

**The code gap (Q4, Option A).** One low-priority bug ticket closes the three orders. It decides the design, makes the switch two steps in the model, and adds the rule `ActiveExists`. The fix mutant of run 1 closes one order only (corrected in the trial log, the Answer of ticket 07 and `SKILL.md` of `verify-gate`).

**The docs (Q6).** `game_flow.md` gets a full rewrite, not a cut of single sentences. The rewrite comes after the model fixes and the diagrams.

**The diagrams (Q9 C, Q10, Q11 A).** The gate model generates a state chart of each part, a checked sequence diagram of each named order, and a sequence diagram of each broken rule's path. They go into `docs/diataxis/reference/diagrams/`. `/statechart` gets the change. The prototype ([assets/09-diagram-prototype/](../assets/09-diagram-prototype/)) drew the busy state of a Game from 60,976 TLC states as 4 chart states, and four checked orders.

**Ticket 08 (Q8, Option A).** Ticket 08 is unblocked. Every `build.py` run shows the warning, and the check reads the working tree too.

**New tickets:** 10 to 15 (see the map).
