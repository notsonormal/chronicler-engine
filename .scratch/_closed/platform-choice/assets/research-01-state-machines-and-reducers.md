# Research 01: what explicit state machines and reducers give the new design

Ticket: [Research: what explicit state machines and reducers give the new design, in Rust and in other languages](../issues/01-research-state-machines-and-reducers.md).
Tree: `4ad85da8`. Date: 2026-10-15.

Labels: **known** = read in a file or a source. **inferred** = follows from known facts. **guessed** = not supported by a source.

Outside sources and their full notes: [research-01-web-sources.md](research-01-web-sources.md). This file gives the summary and the fit to the design.

## 0. Summary

- **The claim is opinion.** No study or benchmark compares agent-written code with and without an explicit state machine (known: none found, §1). The measured evidence supports the testing half of the claim (a machine makes tests enumerable). It does not support the robustness half.
- **The design already has the main gain.** It removes impossible states by derivation (D-01, D-02, D-05) and has one place for each transition (`GameWriter`, `JobWriter`, `generate`). An explicit form prevents none of the 36 known problems that the design does not already settle (inferred, §3.4).
- **What an explicit form adds is detection.** Tests generated from the tables that the design already has (§4.3, §4.4 of the target design), and three small structural gaps (G1 to G3, §3.3).
- **Parts that are definitely a state machine** (§3.6, added after the owner's feedback). The busy state of a Game probably earns it. The Job and one generation are open: a prototype compares the two shapes ([ticket 02](../issues/02-prototype-job-as-pure-state-machine.md)). The other parts do not earn it.
- **The idea does not decide the language.** Rust is stronger on checks at compile time (typestate, exhaustive `match`). TypeScript is stronger on the machine held as data and on path generation (XState v5, `@xstate/graph`). Reducers cost the same in both. The design's best places for the idea need a model-based test, which both languages have (`proptest-state-machine`, `fast-check`) (inferred, §4).

## 1. The claim and its sources

Full table of sources: [web sources §1](research-01-web-sources.md#1-the-claim-and-its-sources).

| Source | Measured or opinion | What it says | Applies to |
|---|---|---|---|
| Matt Pocock, the seed post (fxtwitter mirror: 928 likes, 115 replies) | opinion | Agent-built state machines and reducers feel more robust; later changes are "trivial and testable" | frontend (the post says so) |
| The post's replies | — | Not readable: x.com 403, xcancel 451, Nitter shut down. The seed's "replies mention XState" stays guessed. | — |
| Richard Feldman, "Making Impossible States Impossible" (Elm Conf 2016) | opinion (talk) | Model the data so that a bad state cannot be written | any typed code |
| The Elm Architecture; Redux docs | docs | A reducer `(state, msg) -> state` is the one place for transitions; tests are plain calls | frontend |
| Stately and XState docs (graph, paths) | vendor docs | A machine is a graph; tools draw it, check reachability and generate test paths | frontend and workflows |
| David Khourshid (Agent Conf 2026 interview); Stately Agent; Stately MCP `create_state_machine` | practice / vendor | Keep the workflow in a machine; let the model act only inside one step | long-running agent workflows |
| `Alfredvc/aharness` (a Codex harness); OpenAI "Harness engineering" | practice | A machine around the agent enforces a process that a prompt cannot | the agent's own workflow, not the product code |
| IEEE TSE 2010 (state-machine testing experiments); ASE 2011 (stateful testing) | measured | State-machine-based and stateful tests find faults that other tests miss | tests |
| LLM-FSM, AutoFSM, FSM testbench papers, Eywa (NSDI 2026) | measured | Models make many errors on state-dependent code. Structure plus tests and tool feedback is the reported fix. Models can build a machine from a spec, and model-based tests then cover it well. | hardware RTL, protocols |

**The claimed reasons** (known, from the sources above, not from the post): fewer impossible states (Feldman), one place for transitions (Elm, Redux), transitions that a test can enumerate (XState graph, stateful testing). The post itself names only "robustness" and "trivial and testable" updates.

**Where it applies** (inferred). The practice reports are about frontend state and about agent workflows. None is about a backend like this one: one Job per Game, a short fixed order of steps, and saves under a guard.

## 2. The forms

Full notes, crate versions and links: [web sources §2](research-01-web-sources.md#2-the-forms).

| Form | Code cost | What it checks, and when | Test-path generation | Rust support | TypeScript support |
|---|---|---|---|---|---|
| (a) enum and `match`, `transition(state, event) -> state` | low | Every state handled: compile time. A legal transition: test time. | none built in; `proptest-state-machine` or `fast-check` `fc.commands` with a hand-written model | native | native: discriminated union plus a `never` check |
| (b) typestate (each state is a type) | medium | Transition order and "use after move": compile time. The strongest form. | none: states are types, so no generic walker exists | strong: move semantics. `sm` shows E0382, E0599, E0277. `statig`'s docs say typestate suits an API, not a system whose events come from outside at run time. | weak: branded types, with no move semantics (inferred) |
| (c) statechart library | medium (XState v5) to high (Rust macro crates) | Hierarchy, guards, reachability. Rust `sm`: transitions at compile time. `statig`, XState: run time. | XState v5: yes (`@xstate/graph`). Rust: none found. | `statig` 0.4.1 and `rust-fsm` 0.8.0 (both 2025-07, maintained); `sm` 0.9.0 (2019) and `sfsm` 0.4.3 (2022) look stalled | XState v5, the reference |
| (d) reducer only, `(state, event) -> state` | low | Only what the function body states. Pure, so tests are cheap. | none built in; property tests are easy | trivial | trivial |
| (e) machine held as data | medium to high | Schema, reachability and dead states: tool time. Legal transitions: run time. | yes for XState data (`getShortestPaths`, `TestModel`, `testCoverage`; `@xstate/test` is now in `@xstate/graph`) | none found; the macro crates make code, not data (inferred) | XState config plus Stately Studio and MCP |

Labels: the crate versions, dates and the `@xstate/graph` functions are known (crates.io API and Stately docs, 2026-10-15). The code costs are guessed.

**Exhaustive is not legal** (inferred). An exhaustive `match` in Rust, or a `never` check in TypeScript, proves that every state has a branch. It does not prove that a branch goes to a legal next state. Only typestate, `sm`, or a test proves that.


## 3. Fit to the design

Sources: [target design](../../turn-state-model/assets/target-design.md) version 2, [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) draft 3, [sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md).

### 3.1 The main finding

The design already applies the main lesson of the claim, but by other means (inferred).

- **Fewer impossible states.** The design removes stored state that could disagree with other state. The status is derived (§2.3). The busy mark is in memory only (D-05). The current state and the start state are derived from the Swipes (D-01, D-02). So a stored `Generating` with no Job (F-6, F-7) cannot exist. This is "make impossible states impossible" by derivation, not by a machine.
- **One place for transitions.** `GameWriter` is the only writer (sketch 25 §6.2 rule 3). `JobWriter` is the only write handle of a Job (rule 4). `generate` is the only generation path (rule 13). Each transition of the game state has one place.
- **A token that proves a state.** `JobWriter` has private fields, and only `GameWriter::accept` makes one. So holding a `JobWriter` proves "this Job was accepted". Its `Drop` ends the busy state. This is a typestate in all but name (inferred).
- **Transitions that a test can enumerate.** This part is missing. The design states its machines as tables (§4.3, §4.4, §4.5), but no test walks every path of a table. Sketch 25 rule 13 has one test per generation kind for the order of the save units. No test stops a Job at every step (§3.3 below, place B).

So an explicit form adds little prevention. It adds **detection**: generated tests over the tables that the design already has. It also closes three small gaps (§3.3).

### 3.2 The design places

"Already" means: is the place a state machine or a reducer in all but name? Problem IDs are from target design §10.

| # | Place | Already | Explicit form that fits | Known problems caught | What the explicit form adds | Cost |
|---|---|---|---|---|---|---|
| A | The steps of one generation (§4.3; `generate` in sketch 25 §3.3 and sketch 24 §2.3) | partly. A fixed straight order of 6 steps (8 with the hooks), with early exits on a failure or a stop. The program counter is the state. | enum `Step` plus a wrapper that sets the step name and runs the step | F-2 (status stuck on "narrating"), AD-09, AD-11. All settled or moot in the design. | The status name comes from the step, not from a call that each step must remember (gap G1) | low: one enum (exists as `Step`) and one wrapper function (guessed: under 30 lines) |
| B | The states of one generation after a stop or a restart (§4.4) | yes, as a table. The save units and the job id check make the table true. | the table as test data; a generated test that stops the Job at each step boundary | R-81, R-93, OR-51, OR-53; row W pair 5. All settled. | Detection: one generated test per (kind × stop point), in place of hand-written cases (gap G3) | medium: a fake `LlmRoles` that can block at each call, plus a test driver (guessed: 100–200 test lines) |
| C | The Job and its plan (§4.2; sketch 25 fork 1, Q1) | partly. The plan is a short order of steps with two conditions (a Trigger fired; a Narration was saved and the toggle is on). It has no loop and no outside event except a stop. | fork 1 option A: a plan held as data, and one runner. Or option B: one function per request kind, each a straight order. | AD-04 (open, code shape) | Option A lets a test enumerate (plan × failure point). Option B keeps the order readable as code. This research does not decide Q1. | A: plan enum plus runner. B: about 6 short functions. Similar size (guessed). |
| D | The busy state of a Game: the guard and the job entry (§3.3, §4.1, §5; `GameWriter` in sketch 25 §3.2) | yes. Two states per Game (idle, busy with a job id and a step). Events: accept, step, end, stop, reset, restart. The refusal "Still thinking..." is the busy-state answer. | a pure transition function for the job entry, `(entry, event) -> (entry, answer)`, tested with a model-based test (sequences of events against a simple model) | F-1, F-7, AD-08 (open, code shape), row W pairs 5, 8, 9. All settled except AD-08. | Detection: sequences such as accept → stop → late save → accept. A sequential model test does not test thread interleavings. Those need a scheduler tool (for example `loom` or `shuttle`, guessed; not checked here). | low to medium. `proptest` is already a dev-dependency (known, `Cargo.toml`), and `game_state_tests.rs` uses it (known). |
| E | The Swipe state of one Swipe: pre-analysis, then analysed (§3.1, D-03, D-04) | partly. "At most two writes, by the Job that made the Swipe" is a rule. `JobWriter::save_state_update(at: SwipeAt, ..)` takes any `SwipeAt` (known, sketch 25 §3.2). | a typestate token: `save_text` returns a `SavedText` token with a private constructor, and `save_state_update` takes it by value | row W pairs 6–7 (one writer per Swipe state), S-1. Settled by D-03, but by a rule, not by structure. | Prevention: the compiler refuses a state update on a Swipe that this Job did not save, and a second update with the same token (gap G2) | low: one type (guessed: under 20 lines). It needs move semantics to forbid a second use (§5). |
| F | The Retry by the kind of the last Message (§4.5; `Tail::redo`) | yes, as a decision table: a total function of `MessageKind`. It is not a machine: it has no state that changes. | an exhaustive `match` on `MessageKind` | F-5, F-8, O-2, O-3. All settled by the derived start state (D-02). | Nothing new. The exhaustive `match` already makes a new kind fail to compile until it has a row. | none |
| G | The game rules (`apply_world_rules(start, analysis, world, mode)`, sketch 25 §3.3) | yes. A pure function `(state, input) -> state`: a reducer in shape. | a reducer with property tests | S-1 (settled); F-4 (open, outside); R-14 and D-12 (event mode never moves the player and fires no Trigger) | Detection: property tests such as "in `Event` mode the room never changes". The seed's claim that a reducer makes Snapshots "replayable" adds nothing: each Swipe holds its full state (D-01), so no replay is needed. | low |
| H | The status (§2.3) | no machine. A derived value: the step name if busy, else the failure line, else idle. | none. A derived value is the simpler form. | AD-11 (open, code shape), F-6, F-7 | Nothing. A machine here would bring back a stored status, which D-05 removed. | — |
| I | The dashboard action area ([issue 42](../../dashboard-ui-review/issues/42-rework-the-action-area.md)) | yes, on the client, in `assets/dashboard-action-area.js`. `deriveActionState()` reads the DOM in the order `checking` → `generating` → `preview` → `idle`; `applyActionState(state)` is the one writer of the input and the button (known). It is a derived state with one writer, not a transition table. | the post's own target (frontend). An explicit transition table, or a statechart library. | none of §10. Issue 42 fixed five stranding paths that came from five separate patches (known). | Possibly a clearer client model. The dashboard is outside the designed scope (turn-state-model scope B). | unknown. A statechart library would add a JavaScript dependency to a page that loads plain script files from `assets/` with no bundler (known: `package.json` lists only `jsdom` and two agent tools; inferred: no build step). |

**Places where a state machine does not fit** (inferred):

- **Prompt assembly.** A pure transformation from values to a prompt. It has no states.
- **Storage.** Reads and save units. A save unit lands whole or not at all; it has no visible middle state.
- **The views** (§6). Each view is a derived read.
- **Create and reset.** One save unit each (D-25).
- **The Agents' calls.** Each hook returns one value (D-29). The two-attempt retry loop inside the Agents (AD-12) is a loop, not a machine.

### 3.3 Three gaps that an explicit form closes

These gaps are small. Each is a candidate for the build plan, not a change to the design's behaviour.

| Gap | Today in the design | What can go wrong | Explicit form | Label |
|---|---|---|---|---|
| G1 | Each step calls `JobWriter::step(..)` to set its step name (sketch 25 §3.2). Rule 15 makes `JobWriter::step` the only writer. No rule says that every step calls it. | A new step, for example a new hook, forgets the call. The status shows the step before. This is F-2 again. | One wrapper, `run_step(Step, ..)`, that sets the name and then runs the step; a syn rule that every step function in `pipeline/job.rs` runs through it. Or an outcome test of R-71 per step. | inferred. An HTTP test of R-71 may already cover the named steps; it cannot cover a step that is not written yet. |
| G2 | `save_state_update(at: SwipeAt, ..)` accepts any `SwipeAt`. D-03 ("only the Job that makes a Swipe writes its Swipe state, at most twice") is a rule, with no guardrail in sketch 25 §6.2. | A Job writes a Swipe state on a Swipe that it did not make, or writes it twice. Sketch 24 §5 plans one such exception (background Agents), so the rule needs a visible place. | `save_text` returns a token; `save_state_update` consumes it. A background Agent later gets its own token kind. | inferred |
| G3 | §4.4 states what stays after a stop at each point of a generation. The planned tests check the order of save units per kind (rule 13), not a stop at each point. | A change in step order moves a save point. The stop table becomes false, and no test sees it. F-8 came from a test fixture that drifted from the real save order (known, research 16 §2). | A generated test: for each kind and each step boundary, stop there and check the saved records against the §4.4 table. | inferred |

### 3.4 What the 36 known problems say

- **Prevention.** An explicit state machine or reducer prevents none of the 36 problems that the design does not already settle or make moot (inferred). The 19 settled and 4 moot rows are settled by derivation (D-01, D-02, D-05), the single writer (D-03, D-07, D-08) and the single path (D-16). The 10 "open (outside)" rows are not state problems (storage, prompt assembly, HTTP policy, the transport).
- **The 3 "open (code shape)" rows** (AD-04, AD-08, AD-11). An explicit form is one answer for each: a plan as data for AD-04 (place C), a job-entry transition function in one module for AD-08 (place D), one derived status function for AD-11 (place H). The design already leaves these to the build plan.
- **Detection.** An explicit form makes the tables of §4.3 to §4.5 walkable by a test. That protects the settled rows against regression (G3). Today's history shows the risk: F-2, F-6, F-7 and F-8 were regressions or drift in the order of steps and saves (known, research 16 §2).

### 3.5 The seed's claims about this repo, checked

| Claim (seed §2) | Result | Label |
|---|---|---|
| `generation_status.rs` says "live state machine lives in `application/pipeline/`" | True. The module doc also names `phases.rs`, which does not exist; the phases are in `pipeline_run.rs`. The doc is stale. | known |
| Two state channels: `GenerationStatus` (Idle, Generating, Error) and `GenerationPhase` | True for today's code (`generation_status.rs`). The target design deletes `GenerationStatus` and `InputBuffer`, and `GenerationPhase` becomes `Step` (sketch 25 §7.1). | known |
| Make the generation an explicit FSM with states Idle, Claimed, Narrating, Quantifying, Committing, TriggerContinuation, Error | Mostly done already in the design, with other names: §4.3 (steps), §4.4 (states of one generation), §4.2 (event generation inside the Job). "Committing" is the state update (step 6). "Error" is the failure line, which the design keeps outside the machine (D-05). | inferred |
| GameState as a reducer, "event → new GameState", so Snapshots are replayable | The game rules are already a pure function (place G). Replay is not needed: each Swipe holds its full state (D-01). The design has no `GameState` and no Snapshot. | inferred |
| The HTMX dashboard can be a small state machine | It already is one (place I). | known |
| The idea applies "almost everywhere" | False for this design. It fits places A to E and G. It adds nothing at F and H, and it does not fit prompt assembly, storage, the views, create and reset, or the Agents' calls. | inferred |

### 3.6 Parts that are definitely a state machine

Added 2026-10-15 after the owner's feedback. The owner's question is about the design in general, not about the post's "for frontend" premise: would the design improve if some parts were definitely a state machine, and not "mostly one"? §3.2 asks what an explicit form adds to the parts as they are. This section asks the stronger question.

**What "definitely a state machine" means here.** The code has one state type and one function, `next(state, event) -> (state, effects)`. That function makes no LLM call and does no I/O. A small runner does the effects (the LLM calls, the save units) and gives each result back to `next` as an event. The tables of target design §4.3 to §4.5 are then the code itself. Today they are behaviour that the code must match. (This shape is often called "sans-IO" or "functional core, imperative shell".)

| Part | Definitely a state machine? | Why |
|---|---|---|
| The busy state of a Game: `GameWriter` and the job entry (place D) | **yes, probably** | It is nearly one already: two states (idle; busy with a job id and a step) and six events (accept, step, end, stop, reset, restart). Most of the known defects were here: F-1, F-6, F-7 and row W pairs 5, 8, 9 (known, research 16 §2). With a pure `next`, a sequence such as "stop, then a late save, then accept" is a plain unit test, and a model-based test can generate such sequences. The cost is low (inferred). |
| The Job and one generation: `generate` and the plans (places A, B, C) | **not sure; a prototype compares** | For: the stop, a failure, the hooks and the event generation meet here. The stop table (§4.4) would be code. A new hook would be a new state. A background Agent (sketch 24 §5) could be a second machine of the same kind. Against: the flow is a straight line with early exits. A plain function with `?` is shorter and easier to read. The owner noted that LLM-written code tends to grow (sketch 25 §9, reaction 4). This is fork 1 of sketch 25 (Q1), with a stronger option A. [Ticket 02](../issues/02-prototype-job-as-pure-state-machine.md) compares the two shapes. |
| The Swipe state: pre-analysis, then analysed (place E) | only as a token (G2) | Two states and one writer. A full machine would be too much (inferred). |
| The game rules (place G) | no | Already a reducer: a pure function. |
| The Retry by kind (F), the status (H), the views, prompt assembly, storage | no | Decision tables or derived values. A machine would bring back stored state that the design removed on purpose (D-01, D-05). |

**View** (inferred). The idea earns its place in the job entry, and maybe in the Job. It does not earn its place in the other parts. The design's largest gain came from storing less state, not from naming states.

## 4. Fit to agents

Detail: [web sources §3](research-01-web-sources.md#3-fit-to-agents).

| Source | Type | Finding | Label |
|---|---|---|---|
| SWE-bench Multilingual (<https://www.swebench.com/multilingual.html>) | measured | SWE-agent with Claude 3.7 Sonnet: Rust 58.14 % (25 of 43), JavaScript/TypeScript 34.88 % (15 of 43), overall 42.67 %. Rust was highest although its gold patches change more lines. | known (checked by the lead on 2026-10-15) |
| — the limits of that result | — | One model, about 43 tasks per language, and different repositories per language. So it does not compare the two languages on the same work. It says nothing about state machines. | known (the page); inferred (the effect) |
| RustAssistant (ICSE 2025) | measured | About 74 % of real Rust compile errors fixed by a model in a loop with the compiler | known |
| LLM-FSM, AutoFSM | measured | Models make many errors on state-dependent code; structure plus tests is the fix | known |
| Stately MCP, "Generate flow", `aharness` | vendor / practice | Models can write XState machines and FSM harnesses. No accuracy numbers. | known |

**The gap** (known). No source compares how well a model writes or changes each form (enum and `match`, typestate, statechart, reducer). The fit to agents is not measured.

**Inferred for this repo.** The forms that the design would use (enum and `match`, a token type, a pure function, a model-based test) are plain language features. They need no library that a model must learn. The risk that the sources name, state-dependent code with errors, is the risk that a generated test (G3) detects.

**A trial** (proposed, not run; this ticket is research only). Take one unit of the design, for example the job entry of place D, or one generation with the stop table of place B. Have the same agent and prompt write it in three forms: Rust enum and `match` with a model-based test; Rust typestate (G2); a TypeScript discriminated union with `fast-check` `fc.commands`. Measure the iterations to a green build, the edit-to-test time, the defects that a mutation check finds, and the test lines. A fourth arm, XState v5 with `@xstate/graph`, measures path generation. The trial can share its slice with the fog item "A measured feedback loop" (map).

## 5. Fit to the language question

Detail: [web sources §4](research-01-web-sources.md#4-language-fit).

| Language | Forms it supports well | Tools | What moves from the compiler to tests or run time |
|---|---|---|---|
| Rust | enum and `match`; reducer; typestate with move semantics; `statig`, `rust-fsm`, `sm` | `proptest` (already a dev-dependency here) and `proptest-state-machine` 0.9.0 (sequential only); no visual editor; no path generator | the legal transition, in `statig` or an enum; outside events |
| TypeScript | discriminated union with `never`; reducer; XState v5; machine as data | XState v5, `@xstate/graph`, Stately Studio, `fast-check` model runs including `scheduledModelRun` for races | the legal transition (XState checks at run time); typestate becomes a pattern, not a guarantee; the G2 token cannot forbid a second use (inferred: no move semantics) |
| Elixir / Erlang | `gen_statem` (OTP) | standard library | everything goes to run time |
| Python | reducer; Hypothesis `RuleBasedStateMachine` | Hypothesis | everything goes to test time or run time |
| Kotlin, Elm, F#, OCaml | sealed classes or unions with exhaustive matching | not researched | as TypeScript for transitions |

**What this means for the map's criteria** (inferred):

- **Problem D (design churn and defects).** This idea bears on D. In this design its value is detection: a model-based test over the stop table and the job entry (places B, D; gap G3). Both Rust and TypeScript support that test. So for D the idea does not favour a language.
- **Handoff item 4 (what depends on Rust).** Gap G2 adds one compiler-level rule if it uses a token with move semantics. A place-B or place-D model test adds nothing to the list: it is a test. The list grows by at most one row.
- **The test oracle (map fog).** A machine held as data could generate black-box test paths. That is TypeScript's strongest point (XState). But the oracle tests HTTP and browser behaviour from the specs, not one machine. The value of XState for the oracle is not shown (guessed).
- **Problems A and B (the loop, the build slot).** This idea does not touch them.

**Direction.** The idea does not make the platform choice clearer. It slightly weakens one argument for a move: the design does not need a statechart library, so XState's tools are not a reason to move. It slightly strengthens Rust only through G2, which is small (inferred).

## 6. New questions for the map's fog

1. **Does the build plan adopt G1 to G3?** G1 and G2 are code shape; G3 is a test. They belong to the build plan or to the test oracle, not to the language choice.
2. **Fork 1 (sketch 25 Q1).** A plan held as data (option A) makes (plan × failure point) enumerable by a test. Does that tip Q1 toward option A? This is a build-plan question.
3. **The trial of §4.** Is a measured trial of the forms worth its cost, or does the design's small need (one model test, one token) make it moot? It can share a slice with "A measured feedback loop".
4. **Race tests.** A sequential model test does not test thread interleavings around the guard. Is a scheduler tool (Rust `loom` or `shuttle`; TypeScript `fast-check` `scheduledModelRun`) needed for the guard and the job id check? Not researched (guessed tools for Rust).
