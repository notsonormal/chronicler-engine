# Prototype 02: the Job as a pure state machine, next to a straight `generate`

Ticket: [Prototype: the Job as a pure state machine, next to a straight `generate`](../issues/02-prototype-job-as-pure-state-machine.md).
Code: [`tmp/prototype-02-job-state-machine/`](../../../tmp/prototype-02-job-state-machine/README.md) (Rust, a scratch crate with no link to the engine). Date: 2026-10-15.

Labels: **known** = read in the prototype code or measured in a test run. **inferred** = follows from known facts. **guessed** = not supported by a measurement.

**Warning.** `tmp/` is git-ignored. The code is lost when `tmp/` is cleaned. This file keeps the facts.

## 0. Summary

- **Both shapes pass the same tests** (known). The slice, the stop table of design §4.4 with 8 stop points, two failures, a busy refusal, four busy sequences and a `proptest` model test. Of 18 mutations, every defect that went into both shapes gave the same result in both (§4).
- **Shape P costs about 70 % more non-test code for the Job** (known: 236 against 116 code lines after the change). The order logic itself is about the same size in both shapes. The extra lines are the effect, event and phase types, and the runner.
- **The stop table does not become code in shape P** (known). A refused save never reaches `next`: the runner ends the Job. So in both shapes the stop table lives in a test. Research 01 §3.6 expected "the tables are then the code itself". That is true only for the step-name column of design §4.3.
- **The compiler checks less than expected in shape P** (known). A missing transition compiles and panics at run time. A missing step name compiles, because `Job::step` has a `_` arm. Only "every effect has a performer" is a compile-time check.
- **Shape P conflicts with guardrail rule 7** ("between steps, pass ids", sketch 25 §6.2) (known). The machine must keep the start state and the text in its phase fields, because it has no local variables between two events.
- **The rewrite-hook change was easy in both shapes** (known). Shape S: 2 files, +14 −1 lines. Shape P: 3 files, +23 lines. The shared tests changed the same for both.
- **The busy state is small in both shapes** (known: 91 and 116 code lines with `JobWriter`). Shape P gives a pure table test of 10 rows. It finds the same defects as the shared tests, no more.

## 1. What was built

Fakes only: an in-memory store, a scripted fake LLM, one Game. No database, no HTTP, no threads (known).

| Part | Shared by both shapes | File |
|---|---|---|
| Records | `SwipeState` (room, Characters present, fired Triggers, trigger context), `Message`, `Swipe`, `Dock`, the failure line | [`src/model.rs`](../../../tmp/prototype-02-job-state-machine/src/model.rs) |
| Save units | `Store::write_*`: one method per save unit of design §3.2. Each appends to a log, so a test sees the order. | same |
| Game rules | `apply_game_rules(start, analysis, mode)`, pure. `Mode::Event` never moves the player and fires no Trigger (D-12). | same |
| Fake LLM | `narrate`, `rewrite`, `analyse`, `offer`. Before each answer it calls a test probe with the call number. A probe can read the status or stop the Game: this is a stop during that LLM call. | same |

**The slice** (design §4.2, free text): room check; Input save; Narration (narrator call, text save, rewrite, analysis, game rules, state update); the analysis fires the one-shot Trigger "wolf"; event Narration (the same steps, `Mode::Event`); the Options run.

**Where a stop can come.** A save unit takes milliseconds, so a stop comes during an LLM call, or after accept and before the Job runs. A stop just before a save unit gives the same records as a stop during the LLM call before it (inferred). So the stop points are "before" and "during call n".

## 2. The two shapes, side by side

### 2.1 The Job

| | Shape S (straight) | Shape P (pure machine) |
|---|---|---|
| Files | [`shape_s/job.rs`](../../../tmp/prototype-02-job-state-machine/src/shape_s/job.rs) | [`shape_p/machine.rs`](../../../tmp/prototype-02-job-state-machine/src/shape_p/machine.rs), [`shape_p/runner.rs`](../../../tmp/prototype-02-job-state-machine/src/shape_p/runner.rs) |
| The order of the Action | `run_action` (lines 77–93): room check, `save_input`, `generate`, event `generate` if a Trigger fired, `options_step` if a Narration was saved | `next` (lines 134–192): one `match` on `(state, event)` with 20 arms, with the panic arm. The order is the chain from arm to arm. |
| One generation | `generate` (lines 95–133): the steps of design §4.3 from top to bottom. Each `?` on a `JobWriter` call is a stop point. | 7 `GenPhase` variants: `LoadingStart`, `Narrating`, `SavingText`, `Rewriting`, `SavingRewrite`, `Analysing`, `SavingState`. Each waits for the result of one effect. |
| Reads (load late) | a direct call: `self.store.swipe_state(..)`, `self.store.options_on(..)` | an effect and an event: `Effect::LoadStart` → `Event::Loaded`, `Effect::LoadOptionsToggle` → `Event::Toggle` |
| Data between steps | local variables: `start`, `text`, `at` | fields of the phase variants: `Narrating { start }`, `SavingText { start, text }`, `Analysing { start, at }` |
| Status step name | a `self.writer.step(..)?` call before each step | `Job::step()` derives it from the state. The runner sets it before each effect. |
| A stop | `?` returns `Err(Stopped)`; `run` ignores it; `JobWriter` drops | `perform` returns `Err(Stopped)`; the runner returns. `next` never sees a stop. |
| A failure | a `save_failure` call, then `return Ok(NOTHING_SAVED)` | `Job::fail(line)`: state `SavingFailure`, then `after_generations` |
| An event that does not fit the state | — (no events) | the last arm: `panic!("Job: no transition ...")` |

### 2.2 The busy state of a Game

| | Shape S | Shape P |
|---|---|---|
| File | [`shape_s/busy.rs`](../../../tmp/prototype-02-job-state-machine/src/shape_s/busy.rs) | [`shape_p/busy.rs`](../../../tmp/prototype-02-job-state-machine/src/shape_p/busy.rs) |
| The rule | in each method: `accept` checks `contains_key`; `as_job` checks the job id; `stop` and `reset` remove the entry; `Drop` removes it if it holds this job id | one pure function, `next(entry, event) -> (entry, answer)`, 7 arms (lines 37–48). `GameWriter::transition` runs `next` under the guard, then the save unit when the answer is `Go`. |
| Events | — | `Accept`, `Step`, `Save` (the job id check), `End`, `Stop`, `Reset`. A restart is not an event: memory is lost, so no entry exists. |
| `JobWriter` | the same interface in both shapes: `step`, `save_input`, `save_text`, `save_rewrite`, `save_state_update`, `save_failure`, `save_options`, and `Drop` | same |

## 3. Measurements

Code lines are lines that are not blank and not only a comment (known, from `grep -cvE '^\s*(//.*)?$'`). "v1" is before the rewrite-hook change; "v2" is after.

| | Shape S | Shape P | Shared |
|---|---|---|---|
| Non-test code lines, the Job, v1 → v2 | 108 → 116 (`job.rs`) | 217 → 236 (`machine.rs` + `runner.rs`) | — |
| Non-test code lines, the busy state with `JobWriter`, v1 → v2 | 88 → 91 | 113 → 116 | — |
| Non-test code lines, all, v2 | 209 | 355 | 287 (`model.rs`) |
| Test code lines, v2 | 0 of its own | 43 of its own (`tests/pure_p.rs`) | 328 (`job_slice.rs`, `busy_state.rs`, `common/mod.rs`) |
| Files touched by the rewrite-hook change | 2: `job.rs` +10 −1, `busy.rs` +4 | 3: `machine.rs` +14, `runner.rs` +5, `busy.rs` +4 | `model.rs` +16; tests +30 −10 |
| What the change added | 1 step call, 1 LLM call, 1 save call (7 lines in one place); 1 `JobWriter` method | 2 phases, 2 effects, 1 event, 3 transition arms, 1 step-name arm, 2 runner arms; 1 `JobWriter` method | the step name, the save unit, the fake call; the stop table grew from 6 to 8 rows |
| Where the stop table lives | a test: `STOP_TABLE` in [`tests/job_slice.rs`](../../../tmp/prototype-02-job-state-machine/tests/job_slice.rs) (lines 34–63) | the same test. `next` has no stop. | — |
| Test run time | 0.00 s for the Job tests, 0.32 s for the busy tests with `proptest` | the same | — |

### 3.1 What the compiler checks, and what only a test checks

| Check | Shape S | Shape P | Evidence |
|---|---|---|---|
| A Job writes only through its `JobWriter`, with the job id check | compiler | compiler | known: the same `JobWriter` in both |
| Every effect has a performer | — (no effects) | compiler: `perform` matches every `Effect` | known: mutation P8 gave `error[E0004]` |
| Every state handles every event that can come | — | run time: the `_` arm panics; a test finds it | known: mutation P7 compiled and failed 2 tests |
| Each step shows its step name (gap G1) | test (the status test) | test. A compiler check is possible only if `Job::step` lists every phase with no `_` arm (inferred). | known: mutations S1, S7, P1, P6 compiled and failed the status test |
| The order of the save units | test | test | known |
| The stop table (gap G3) | test | test | known |
| Pass ids between steps (rule 7) | met: records are local variables | broken: `SwipeState` in phase fields and in `Effect::Narrate`, `Effect::SaveText`, `Event::Loaded` | known: the code. A walker for rule 7 would flag shape P. |
| The busy rules | test (shared sequences, `proptest`) | test (the same), plus a pure table test | known |

## 4. Mutation check

`mutate.sh` changes one line, runs all tests and restores the file (known, all runs on 2026-10-15).

| # | Mutation | Result |
|---|---|---|
| S1, P1 | The analysis shows the wrong step name | both caught by the status test |
| S2, P2 | The end of an old Job frees the entry of a new Job | both caught: the hand sequence and `proptest`. P2 also by the pure table test. |
| S3, P3b | The Options run ignores the toggle | both survived: no test turns the toggle off |
| P3, P4 | No Options run after a generation; `narration_saved` not set | caught |
| S4 | The text save keeps the start state, not the pre-analysis state | caught by the stop table |
| S5 | The Job goes on after a failed room check | survived: no test has a room that is not on the map |
| S6, P5 | A refused step-name write is ignored | both survived. The next save is refused, so the records are the same (inferred: an equal mutant). |
| P6, S7 | A new step forgets its step name | both caught by the status test only. Both compiled. |
| P7 | A missing transition for "the rewrite Agent keeps the text" | caught at run time (a panic in 2 tests). It compiled. |
| P8 | The runner forgets the new effect | compile error |
| S8, P9 | The rewrite is not saved | both caught by 3 tests |

So the shape did not change what the tests find (known for these 18). The survivors are gaps in the shared tests, the same in both shapes.

## 5. What was hard or unclear

### 5.1 Shape S

- Little was hard (known: the code compiled at the first build and passed every test at the first run, before and after the change).
- One decision: what `generate` gives back. `Generated { saved, fired }` is enough, because a failed generation never fires a Trigger.
- The stop points are the `?` on the `JobWriter` calls. A reader sees them, but no type names them.
- A new step can forget its `step(..)` call. Only the status test sees it (gap G1 of research 01).

### 5.2 Shape P

- **Data between steps.** The machine has no local variables between two events. So the start state and the text go into each phase variant, and each arm moves or clones them (known). This is where rule 7 breaks. To keep rule 7, each phase would have to load the start state again: one more effect and event per phase (inferred).
- **Reads become round trips.** "Load late" means one effect and one event per read (`LoadStart`, `LoadOptionsToggle`) (known).
- **A catch-all arm is needed.** The `match` on `(state, event)` has 15 states (8, and 7 phases inside `Generating`) and 8 events: 120 pairs. Most cannot happen. A full `match` with no `_` arm is not practical, so a wrong pair is a run-time panic (known: the code; inferred: the size).
- **The stop is outside the machine.** Every state handles a stop the same way, so the runner does it. So the machine does not hold the stop table, which was its main expected gain (known).
- **The status needs care.** `Job::step` returns `None` for "no change". The phase that loads the toggle must not show "options" before the toggle is known. In shape S this is the place of one call. In shape P it is a rule in a second function (known).
- **Reading the order.** To see the order of the Job, the reader follows the chain from arm to arm across 20 arms. In shape S the order reads from top to bottom (inferred: a judgment; the owner decides).

### 5.3 The busy state

- Both shapes were easy (known). The pure `next` is 12 lines and reads as a table. `GameWriter::transition` is a generic wrapper of 12 lines. Shape S has the same rules in 4 short methods.
- The pure `next` gives a test with 10 table rows, with no store and no lock (known). The shared `proptest` model test already covers the same rules in both shapes. The pure test found no defect that the shared tests missed (known, §4).
- **The model test compares the code with a second model** (known: `busy_state.rs` keeps its own `Option<(handle, step)>`). In shape P a model test of `next` alone would compare `next` with a copy of itself. So the model test must run against `GameWriter`, as it does here, in both shapes (inferred).

## 6. What this does not show

- **Threads and races.** The prototype runs the Job inline, with no thread. The guard is tested only in sequence. Research 01 question 4 (a scheduler tool such as `loom` or `shuttle`) stays open (known).
- **The other plans.** Only the free-text Action ran. Retry, Retrigger, `/impersonate` and `/options` were not built. Shape P would need a plan type or one machine per request kind (inferred). That is fork 1 of sketch 25 (Q1).
- **Agent-written code.** One author (this session) wrote both shapes. The prototype does not measure how well a model writes or changes each shape (research 01 §4, the trial).
- **TypeScript.** Not built. In TypeScript the `JobWriter` token has no move semantics (research 01 §5), so the compiler rows of §3.1 change (inferred).

## 7. The author's view

This view is **inferred**. The owner did not choose a shape (2026-10-15): the two shapes are points of comparison with the existing design. See the ticket's Answer.

- **The Job: shape S.** Shape P doubles the code of the Job and breaks rule 7. Its compiler gain is one check ("every effect has a performer"). The stop table stays a test. The rewrite-hook change was easy in both shapes. Gap G1 can close in shape S with a `run_step(Step, ..)` wrapper or the status test, which already exists.
- **The busy state: either shape.** The two cost the same. Shape P reads as a table and gives cheap table tests. Shape S has the same rules in four short methods. Shape P can earn its place later, when a background Agent (sketch 24 §5) adds a second kind of entry (guessed).
- **For the platform choice.** The prototype adds no argument for a language. Shape S uses plain language features. The compiler checks that carry weight here (`JobWriter` as the only write handle; an exhaustive `match` on effects) are Rust checks that TypeScript gives in a weaker form (inferred).

## 8. Questions for the later choice

Open. They go with the owner's choice among option S, option P and the existing design.

1. **Which shape of the Job reads more clearly to you?** Compare `run_action` and `generate` in `shape_s/job.rs` (lines 77–133) with `next` in `shape_p/machine.rs` (lines 134–192).
2. **Does shape P earn its place for the Job, for the busy state, for both, or for neither?**
3. **If shape P for the busy state:** is a pure `next` for the job entry worth about 25 more lines and a generic `transition` wrapper?
4. **Gap G1** (the step name). Close it in code (a `run_step` wrapper in shape S, or a `Job::step` with no `_` arm in shape P), or keep the status test as the only check?
5. **Rule 7.** If shape P is chosen for the Job: does rule 7 get an exception for the phase fields, or does each phase load again?
