# Decide the shape of the Job and the busy state

Type: grilling
Status: resolved
Blocked by: —

## Question

Which code shape does the build use for the Job, and which for the busy state of a Game: option S (straight code), option P (a pure state machine and a runner), or the existing design (sketch 25 §3.3 as written)? With that choice:

1. **Fork 1** (sketch 25 Q1): the plans as code or as data. The constraint stays: one `generate` for every kind of generation.
2. **Gap G1**: the step name set in code (a `run_step` wrapper in S, a `Job::step` with no `_` arm in P), or only checked by the status test.
3. **Gap G2**: a token from `save_text` that `save_state_update` consumes (D-03 by structure), or not.
4. **Gap G3**: a generated test that stops a Job at every step boundary and checks the §4.4 table, or the hand-written table.
5. **Rule 7**, only if option P is chosen for the Job: an exception for the phase fields, or each phase loads again.

## Context

- [Prototype: the Job as a pure state machine, next to a straight `generate`](02-prototype-job-as-pure-state-machine.md): the facts table (S: 116 lines, P: 236 lines for the Job; 91 against 116 for the busy state), and the questions of [asset §8](../assets/prototype-02-job-state-machine.md). Code: `tmp/prototype-02-job-state-machine/` (git-ignored, may be gone).
- [Research 01](01-research-state-machines-and-reducers.md) §3.3 (G1 to G3), §3.6 (the busy state probably earns the machine shape; the Job is open), §6 item 2 (fork 1).
- [Target design](../../turn-state-model/assets/target-design.md) §3.3, §4.2 to §4.4. [Sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §3.2, §3.3, §6.2. [Sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md) §2.3.
- The owner's concerns: LLM-written code tends to grow; rules belong in code structure or guardrails; no silver bullet.

## Done when

The owner has chosen a shape for the Job and one for the busy state, and has answered items 1 to 5. The answer names any change to sketch 25 or the guardrail rules.

## Answer

Resolved 2026-10-16 by the owner, in a two-round grilling (round 1: Q1 to Q5; round 2: Q6 and Q7). Labels as in the map assets: **known**, **inferred**, **guessed**.

### The two shapes

| Part | Choice | Why |
|---|---|---|
| The Job | **Option S**: straight code. `generate` holds the six steps of design §4.3 in order, with `?` on every `JobWriter` call; a refused save ends the Job. | Known (prototype 02): option P costs 236 non-test lines against 116, breaks rule 7, and buys one compiler check ("every effect has a performer"). The stop table stays a test in both shapes. The same 18 mutations gave the same result in both. P's own pure unit test (`pure_p.rs`, 43 lines) is not allowed by ruling R9 of [Decide where the tests of the new design sit](../../turn-state-model/issues/26-decide-test-placement.md). |
| The busy state of a Game | **Option S**: plain methods under one guard, as sketch 25 §3.2 already writes them: `accept`, the job id check inside each save method, `stop`, `reset`, `Drop`. | Known: the two shapes cost about the same (91 against 116 lines with `JobWriter`), and shape P's pure table test found no defect that the shared tests missed. Ruling R9 places `application/generation/gate.rs` at tier 1, and R2 rule 3 allows a unit test only inside a seam contract, so shape P's main gain is not available. |

**Contradiction in the earlier evidence, recorded.** [Research 01](../assets/research-01-state-machines-and-reducers.md) §3.6 says that the busy state probably earns the machine shape, with the cheap model test as its reason. Ruling R9 came first and puts that test out of reach.

**The third candidate was not a third shape** (inferred). "The existing design (sketch 25 §3.3 as written)" is option S: `shape_s/job.rs` is that sketch made concrete, with the job id check and the rewrite hook that the design already requires.

### Item 1: fork 1 — the plan as a value, and one runner (option A)

`JobPlan { input: Option<String>, first: Option<GenerationOrder>, options: OptionsRun }`, built by `GameWriter::accept`; one runner in `pipeline/job.rs` holds the order of a request once: the Input save if the plan has one, `generate` the first order, the event generation if a Trigger fired, the Options run if the condition holds. The constraint stands: every kind of generation runs through the one `generate` (guardrail rule 13).

Reason: the six rows of design §4.2 differ in three values only, and the event generation and the Options run are the same code for every kind. Today's defect pattern is their duplication ("the event tail exists twice, the `retry_target` append three times, the Options step twice" — research 12 §2c, quoted in sketch 25 §3.3).

**The Options condition follows what the work saved, not the request kind** (inferred). R-30 and R-31 make the run a function of the last Narration that the work saved, and an Input-kind save (Impersonate, user regen) leaves the offered set as it was. So one condition derives every row of §4.2, including the `/impersonate` row: an Impersonate generation saves an Input, so only an event Narration can turn its Options run on. `OptionsRun` keeps the two variants of sketch 25 §3.3 (`AfterNarration | OnDemand`). If §4.2 or R-30 is read differently, the plan value needs a third variant, and `OptionsRun` is where that change lands.

**The place of a generation lives in `GenerationOrder`** (known). Free text and Retry make the event generation a new Swipe on the last Message; a Retrigger makes it a new Message at the end (design §4.2, §4.5).

### Item 2: gap G1 — the wrapper and rule 17 (option A)

Each step of `generate` becomes a step function, and each runs through `run_step(Step, ..)`, which sets the status name through `JobWriter::step` and then runs the step. The step functions are the ones sketch 24 §2.3 already names (`before_narration_step`, `narrate_step`, `rewrite_step`, `analyse_step`, the Options step). Rule 17 below forces a new step through the wrapper.

Reason (known, prototype 02 mutations S7 and P6): both shapes compiled when a step forgot its name, and only the status test found it. That is the F-2 defect class, a status stuck on the step before — one of the risks that the owner named for LLM-written code.

### Item 3: gap G2 — the `SavedText` token (option A)

`JobWriter::save_text` returns `SavedText` (a private constructor that holds the `SwipeAt`). `save_state_update` takes it **by value**, so move semantics forbid a second state update on one Swipe, and the compiler forces the update onto the Swipe that this Job's text save made. Cost guessed under 20 lines.

- Sketch 24 §2.4 puts `save_rewrite(at, text)` between the two, so it becomes `save_rewrite(&SavedText, text)`.
- A user regen ends at the text save, so the token is dropped with no state update — as the design already has it.
- A background Agent (sketch 24 §5) later writes one named `trackers[name]` part and gets its own token kind. That exception stays outside this token's proof.
- Rule 7 permits the token: it is not one of the six banned types.

### Item 4: gap G3 — one generated tier-1 stop test (option A)

One parameterised test over **(kind × stop point)**, with the §4.4 table as data. Kinds: Narration, Impersonate, user regen, event (sketch 24 §2.3). The prototype's slice had 8 stop points (known); a stop just before a save unit leaves the same records as a stop during the LLM call before it (prototype 02 §1, inferred). Cost guessed 100 to 200 test lines (research 01 §3.3).

- **Home: tier 1** (known). Ruling R8 gives `pipeline/job.rs` no unit tests, and §4.4 is player-observable: stop, then read the story log and the state.
- The fake must hold a model call until the test releases it. Ruling R6 already requires that, so the matrix costs less than research 01 guessed.
- The behaviour needs a scenario in `docs/specs/` before its test (ruling R7 condition 3; `validate_feature_spec.py` enforces the link). That scenario is build-plan work, not a new decision.

### Item 5: rule 7 — no exception

With option S for the Job, no struct field and no function parameter of `pipeline/job.rs` has the type `Game`, `Message`, `SwipeState`, `Tail`, `DockRecord` or `WorldBundle`. The `Job` holds the `JobWriter`, the roles, the storage and the plan, and the records between steps stay local variables of the step functions. The rule 7 exception that asset §8 question 5 offered is not needed.

### The changes that this answer makes

**Sketch 25 §3.2.** `JobWriter::save_text` returns `SavedText`; `save_state_update` takes `SavedText` by value. The busy state keeps the plain-method shape that the sketch already writes.

**Sketch 25 §3.3.** The steps of `generate` become step functions, each through `run_step`. The order of a request becomes `JobPlan` plus one runner. `generate` keeps no record fields.

**Sketch 24 §2.4.** `save_rewrite(at, text)` becomes `save_rewrite(&SavedText, text)`.

**Sketch 25 §6.2.** Two additions, so the list holds 17 rules (15, plus sketch 24's 16, plus this one):

| # | Rule | Reason (the guardrail's comment) | Level | How |
|---|---|---|---|---|
| 17 | Every step function of `pipeline/job.rs` runs through `run_step` | A step that skips the wrapper leaves the status on the step before it (defect class F-2) | syn (call site) | The existing "method X may be called only in files Y" walker family, one configured row |
| — | Rule 4's row gains one sentence: the `SavedText` token is the compiler-level enforcement of D-03 (one writer per Swipe state, at most twice). D-03 gets no numbered rule | The compiler refuses the mistake that the rule describes; a walker adds nothing | compiler | — |

### What this settles for the tickets that wait on this one

- [Research: race tests for the guard and the job id check](06-research-race-tests-for-guard.md): the guard and the job id check keep the shape of sketch 25 §3.2 — one `Mutex<HashMap<GameId, JobEntry>>`, the job id check inside each save method, `Drop` ends the entry. Its question (interleavings, and the tool) is unchanged and now free.
- [Prototype: decide the crate split and the dev-build loop](07-prototype-crate-split-and-dev-build-loop.md) and [Decide the build path](09-decide-build-path.md) still wait on [Research: sort the open architecture-deepening tickets against the new design](04-research-sort-architecture-deepening.md). The module names that they need are settled: `application/pipeline/job.rs` (the plan, the runner, the step functions) and `application/generation/gate.rs` (`GameWriter`, `JobWriter`).

**No new ticket, and no change to the fog.** The two open fog items ("the design beyond scope B", "module shape and docs for agents") hang on tickets 04 and 07, which this answer does not move.
