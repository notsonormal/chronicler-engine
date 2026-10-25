# Research 06: race tests for the guard and the job id check

Written 2026-10-16 for [Research: race tests for the guard and the job id check](../issues/06-research-race-tests-for-guard.md). Labels: **known** (read in a source or in the code), **inferred** (follows from known facts), **guessed**.

## 0. Summary

**Recommendation: no scheduler tool.** Use deterministic tier-1 tests with the fake that holds a model call (ruling R6), plus one new guardrail rule (rule 18 below).

- The design makes every write one short hold of one lock. So every run of the engine is equal to some order of those holds (inferred, §2.1). A test that holds the model fake at each LLM call can put a request into each gap between two holds of a Job. The gaps that a held call cannot reach give the same records as a gap that it can reach (inferred, §2.2).
- That argument is true only while each writer takes the guard **once** for its check and its save. Four of today's nine writer pairs (research 05 §3, pairs 6 to 9) come from a check and a save in two holds, or from no check. Rule 18 keeps the one-hold shape at compiler and lint level, before any test runs.
- `loom` and `shuttle` both need a unit test that calls `GameWriter` directly in modelled threads. Ruling R9 of [ticket 26](../../turn-state-model/issues/26-decide-test-placement.md) places `gate.rs` at tier 1 only, and sketch 25 §3.2 says that `GameWriter` is not a seam. So either tool needs an exception to R9 (known). Each tool also adds a separate compile to the build slot (§4).
- The gain of a tool over the recommendation is small here: no atomics with weak ordering, one lock, two to three threads per case (inferred).

Six interleavings need a test (§3). One of them is already planned as gap G3 (the stop matrix of ticket 05). Five need a scenario and a tier-1 test. Cost guessed: 5 scenarios, 150 to 250 test lines, one walker row for rule 18.

## 1. The shape under test

From [Decide the shape of the Job and the busy state](../issues/05-decide-job-and-busy-state-shape.md) and sketch 25 §3.2 (known):

- One guard: `GameWriter { entries: Mutex<HashMap<GameId, JobEntry>> }` for all Games (D-07).
- Each request that writes a Game, and each save unit of a Job, takes the guard for one check and one save unit. No LLM call runs under the guard.
- The job id check: each `JobWriter` save checks, under the guard, that the entry still holds its job id. Else the save is refused, and the Job ends.
- `Drop for JobWriter` removes the entry only if it still holds this job id.
- The Job is synchronous code on `tokio::task::spawn_blocking` (sketch 25 §3.3 `start`; today `core.rs` does the same). The handlers are async, but `GameWriter` is synchronous.
- Storage keeps one `std::sync::Mutex` on its backend (`storage/core.rs`, known). A save unit takes it inside the guard. A read takes it with no guard.

Today's code has no test for eight of the nine writer pairs (research 10, harness gaps; known).

## 2. Why deterministic tests reach every interleaving

### 2.1 The holds serialise the writes

Every write is one hold of one lock. Two holds never overlap. So the outcome of a run depends only on the order of the holds of the Job and of the requests (inferred: this is what a lock guarantees for code that touches the shared state only while it holds the lock).

Three things are shared and stay outside the guard. None of them writes a Game:

| Shared item | Who touches it | Why it adds no write race |
|---|---|---|
| The view reads | handlers | They read storage. They write nothing (design §6). |
| The cancel handle | stop; the LLM call | A `CancellationToken` from `tokio-util`, which is thread-safe. The job id check makes a late answer harmless whether or not cancel works (design §5, D-11). |
| `ActiveGame` | `CurrentGame`, switch, create, reset | A request reads it once (design §3.3, D-09). A Job never reads it. |

**Lock order** (inferred): a write takes the guard, then the storage mutex. A read takes the storage mutex only. No path takes the storage mutex and then the guard. So the two locks cannot deadlock. Rule 18 keeps this order (§5).

### 2.2 The gaps of a Job

The holds of one Job, in order (design §4.3, ticket 05 item 1). An LLM call sits between some holds. The fake can hold a call, so a test can put a request into those gaps.

| Gap after the hold | Next hold | Reached by a held call? | Same records as |
|---|---|---|---|
| accept | Input save | no | the gap before the narrator call: the Input save is refused, the Job ends with no write (inferred) |
| Input save | `step` (narrating) | no | the gap before the narrator call (inferred) |
| `step` (narrating) | text save | **yes** (narrator call) | — |
| text save | `step` (analysing) | no | the gap during the analysis call: the text stays with its pre-analysis state (inferred) |
| `step` (analysing) | state update | **yes** (analysis call) | — |
| state update | the event generation or the Options run | **yes** (the next call) | — |
| a refused save | `Drop` | no | the gap before the refused save: the new entry of a second Job already exists at `Drop` time in both (inferred) |

So holding the fake at each LLM call covers each class of order. Gap G3 of ticket 05 already enumerates the stop points of each kind. This research adds the requests other than stop (§3).

### 2.3 The condition: one hold per check and save

Section 2.2 is true only while each writer does its check and its save in **one** hold. Today's pairs show the defect class (research 05 §3, known mechanism):

| Pair | Today's mechanism | The design's cure |
|---|---|---|
| 6, 7 | load and check, then save in a later hold (`claim_and_spawn`, heal) | accept checks and saves in one hold; no heal (D-24) |
| 8 | edit and delete check nothing | refused while busy, under the guard (R-70) |
| 9 | `switch_swipe` checks `is_busy`, then writes | the same, in one hold |
| 5 | stop frees the slot; the task writes on | the job id check (D-08) |
| 2, 3, 4 | the write target is read at write time | every write names its captured Game (D-09) |
| 1 | arrival saves a boot-time copy after its LLM call | the opening is part of the create unit (design §3.2) |

A future edit can split a check and a save into two holds again. Each half compiles and passes its own test. Only a test that puts a request between the two holds finds it. A scheduler tool would find it. A lint finds it sooner and at no build cost (rule 18, §5).

## 3. The interleavings to cover

All at tier 1, through the routes, with the in-memory backend and the holdable fake (rulings R2, R6, R9). Each needs a scenario in `docs/specs/` first (ruling R7 condition 3).

| # | Order | Expected result | Design rule | Today's pair | Scenario today |
|---|---|---|---|---|---|
| I1 | A Job is held; an Action, a Retry, a Retrigger and an `/options` request arrive for the same Game | each answers `200` "Still thinking..."; nothing is written | R-70, D-07 | 6, 9 | 12.1 (Retry) and 15.1 (Retrigger) exist; Action and `/options` need one |
| I2 | A Job is held at each stop point; stop; release | the records of design §4.4; the Game is idle; no failure line | D-08, R-81 | 5 | **gap G3** of ticket 05 (planned) |
| I3 | Job A is held; stop; Job B starts on the same Game and is held; release A, then B | A's late save is refused; A's `Drop` leaves B's entry; B saves normally | D-08, `Drop` | 5 | none |
| I4 | A Job is held; reset; release | the Job writes nothing; the new Game holds only its opening | design §5, R-74a | 2 | none |
| I5 | Job A is held on Game A; switch to Game B; act there (Job B held); release A; release B; switch back to A | each Job's saves land in its own Game; A shows its result | R-74, D-09 | 2, 3, 4 | none (18.3 tests the heal, which goes) |
| I6 | A Job is held; an edit, a delete and a swipe switch arrive for the same Game | each answers `200` "Still thinking..." and writes nothing | R-70 | 8, 9 | none |

**Not a race test.** Two fast clicks (two accepts at the same moment) is I1 after rule 18: the busy check and the new entry are one hold, so the second click always sees busy (inferred). A test with two real concurrent requests passes whenever the code is right, so it cannot be flaky. It also finds a split check only by chance, so it adds little over rule 18 (inferred). Optional.

## 4. The tools

### 4.1 Facts

| Fact | `loom` 0.7.2 | `shuttle` 0.9.7 |
|---|---|---|
| Method | Exhaustive exploration of interleavings under the C11 memory model, with state reduction. A preemption bound (`LOOM_MAX_PREEMPTIONS`, "2 or 3 is enough to catch most bugs") cuts large models [loom docs] | Randomised schedulers (random, PCT) and an exhaustive DFS that "is not recommended" except for small primitives. "Shuttle is not sound (a passing Shuttle test does not prove the code is correct)" [shuttle docs] |
| Type swap | Code must use `loom::sync` types under a cfg. "Any code that does not use loom's replacement types is invisible to loom" [loom docs] | Code must use `shuttle::sync` types under a feature, or the `shuttle-sync` wrapper crate [shuttle docs; wrappers README] |
| Determinism | "Test cases using loom must be fully deterministic." System calls must be mocked [loom docs] | Failing schedules replay from a schedule string [shuttle docs] |
| Threads | `MAX_THREADS = 5` [loom docs] | no fixed limit stated |
| tokio | No tokio runtime model. Under a global `--cfg loom`, tokio 1.51.1 runs `assert!(!enabled)` on its IO driver (`src/runtime/driver.rs`, `create_io_stack`), so a runtime with IO turned on panics [tokio source] | A `shuttle-tokio` wrapper replaces the `tokio` package. It "does not currently model all tokio functionality". Its `spawn_blocking` runs the closure as a future task, and its comment says that blocking work is "not the kind of things which should be run under Shuttle" [shuttle `wrappers/tokio`] |
| How it runs | `RUSTFLAGS="--cfg loom" cargo test --test <name> --release` is the documented way [loom docs]. Cargo passes `RUSTFLAGS` "to every `rustc` invocation" [Cargo book, build cache] | `cargo test --features shuttle` [shuttle docs] |

Sources: [loom docs](https://docs.rs/loom/latest/loom/), [loom repository](https://github.com/tokio-rs/loom), [shuttle docs](https://docs.rs/shuttle/latest/shuttle/), [shuttle repository](https://github.com/awslabs/shuttle) at commit `0df088bc` (`wrappers/README.md`, `wrappers/tokio/README.md`, `wrappers/tokio/impls/tokio/inner/src/runtime.rs`), [Cargo book: build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html), versions from the crates.io API, tokio 1.51.1 source from the local cargo registry.

### 4.2 The cost here

| Cost | Deterministic tier 1 + rule 18 | `loom` | `shuttle` |
|---|---|---|---|
| Test home | tier 1, as R9 says | a unit test of `gate.rs` with modelled threads; **needs an exception to R9** and R2 clause 3 (known) | the same exception. Through the routes only with `shuttle-tokio`, and axum still links the real `tokio` (inferred risk) |
| Production code change | none; rule 18 is a lint | a `sync` module that swaps `Mutex` under a cfg in `gate.rs` | the same, under a feature |
| What it can see | the orders of §3 | the guard and the entry map only. The storage mutex, SQLite and the cancel handle stay invisible unless swapped too | the same as `loom`, at random |
| Build | none new; the tests run in `build.py integration` | with the documented global `--cfg loom`: every dependency compiles again in `--release`, and tokio's IO assert fires. A crate-local cfg or feature avoids both, but the crate and `loom` still compile in a second configuration (inferred) | a second feature set: the crate compiles again in that configuration (inferred) |
| Build slot (problem B) | no new step | one more heavy cargo step per gate run. Scale: a cold worktree build cost 728 s, a seeded one 344 s (seed asset) | one more heavy cargo step |
| What it finds that the recommendation misses | — | weak memory-order bugs in atomics (the design has none in the write path, guessed), and a split hold (rule 18 already refuses it) | a split hold, at random |

## 5. The recommendation

1. **No `loom`, no `shuttle`.** The design has one lock, no lock-free state, and few threads. The orders that matter are few, and the holdable fake reaches each class (§2.2).
2. **Five scenarios and their tier-1 tests:** I1 (for Action and `/options`), I3, I4, I5, I6. I2 is gap G3, already planned. Cost guessed 150 to 250 test lines. They share the holdable fake that ruling R6 already requires.
3. **Guardrail rule 18** (proposal, for the owner):

   | # | Rule | Reason | Level | How |
   |---|---|---|---|---|
   | 18 | Each writer takes the guard once: every function of `generation/gate.rs` calls `hold` at most once | A check and a save in two holds is today's race class (pairs 6 to 9). One hold per writer keeps every write one critical section, and keeps the order guard → storage | compiler + syn (call count) | Compiler: the `Mutex` sits in a private field of one small type whose only method is `hold(game, \|entries, tx\| ..)`, so `hold` is the one place that locks. Syn: the call-count walker of rule 8, one configured row: functions in `gate.rs` call `hold` at most once |

   Rule 18 extends rule 6 of sketch 25 §6.2 (known): rule 6 already makes the guard a private `MutexGuard` that no function returns. The call-site walker family works per file, so it cannot say "one call per function"; the call-count walker can (known, sketch 25 §6.2). The call-count walker is new in the build in any case (rule 8), so rule 18 adds one row, not one walker (inferred). A check in one closure and a save in a second closure still compiles; only the walker refuses it.
4. **When to look again** (inferred). Any of these breaks §2.1, and then `loom` earns a seam contract on the new primitive:
   - a second lock, for example one guard per Game;
   - an atomic with `Relaxed` or `Acquire`/`Release` ordering in the write path;
   - a background Agent (sketch 24 §5) whose save unit checks "this Swipe is still active and no later Job started" outside the one-hold method.

## 6. Open points for the owner

- Accept rule 18, or reject it and accept the cost of a split hold that only a scheduled test finds.
- Confirm that the five scenarios go into `docs/specs/` with the other build-plan scenarios.
- Confirm "no tool" against the R9 exception that a tool needs.
