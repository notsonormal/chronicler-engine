# Research 01: outside evidence for explicit state machines and reducers

Scope: outside evidence only. This asset does not analyse the target design or sketch 25.
The lead maps the forms onto the repo design in a separate asset.

Labels: **known** = read in a cited source. **inferred** = follows from known facts.
**guessed** = not supported by a source. Source types: **measured**, **practice**, **opinion**,
**docs**, **vendor**.

## 1. The claim and its sources

### 1.1 The seed post

- **Known.** Matt Pocock, post "I think I'm addicted to getting agents to build state machines"
  (<https://x.com/mattpocockuk/status/2110818314499072493>, 2026-10-15). Text matches the seed
  exactly. Metadata from the public fxtwitter mirror
  (<https://api.fxtwitter.com/mattpocockuk/status/2110818314499072493>): 928 likes, 115 replies,
  14 quotes, 604 bookmarks, 62,786 views.
- **Known.** Source type: **opinion / practice report**. The post gives no data, no measurement,
  no before-and-after. It recommends the idea "for frontend".
- **Known (limitation).** The replies cannot be read. `x.com` returns HTTP 403. `xcancel.com`
  returns HTTP 451. Nitter is shut down under X Corp. cease-and-desist letters (stated on
  <https://nitter.net/>). Thread Reader App returned its entry form, not an unrolled thread.
  Anonymous `r.jina.ai` access to `x.com` is blocked. **I do not know what the replies say**, and
  the seed's claim that the replies mention XState and David Khourshid stays **guessed**.
- **Known.** The claimed mechanisms in the post are: (a) "a huge bump in the robustness",
  (b) "future updates will be trivial and testable". The post does not name fewer impossible states,
  one place for transitions, or enumerable test paths. Those mechanisms come from the other sources
  below.

### 1.2 Sources for the mechanisms

| Source | Type | Claim | Mechanism |
|---|---|---|---|
| Richard Feldman, "Making Impossible States Impossible", Elm Conf 2016 (<https://www.youtube.com/watch?v=IcgmSRJHu_8>) | opinion (talk), **known** | Design the model so invalid states cannot be represented; the compiler then rules out a class of bugs | fewer impossible states |
| The Elm Architecture (<https://guide.elm-lang.org/architecture/>) | docs, **known** | A program is Model (state), View (state to HTML), Update (state from messages); Redux took the pattern | one place for transitions; `(state, msg) -> state` |
| Redux Fundamentals, Part 3 (<https://redux.js.org/tutorials/fundamentals/part-3-state-actions-reducers>) | docs, **known** | A reducer is a pure function `(state, action) -> state`; tests are plain function calls | reducer shape; easy unit tests |
| Stately / XState docs, Graph & Paths (<https://stately.ai/docs/graph>) | vendor docs, **known** | States are nodes and transitions are edges; the graph gives model-based testing, visualization and validation (reachability) | enumerable test paths; drawing and checking a machine |
| David Khourshid interview, "State Machines for AI Agents" (Agent Conf 2026, <https://www.callstack.com/podcasts/state-machines-for-ai-agents-with-david-khourshid>) | practice/opinion, **known** (summary text) | Keep the workflow deterministic and let the model stay flexible inside one step; traces show where an agent gets stuck; guards handle unexpected outputs | one place for transitions; guards at the boundary of a model call |
| Stately Agent (`statelyai/agent`, <https://github.com/statelyai/agent>; docs <https://stately.ai/docs/packages/agent/machines>) | vendor, **known** | "Agent logic as state machines: deterministic, inspectable, resumable"; the machine declares states, legal transitions and which events the model may choose; the machine never calls the model | long-running workflows; the machine owns control flow |
| Stately MCP `create_state_machine` (<https://dev.stately.ai/docs/packages/mcp>) | vendor, **known** | A tool makes a new XState v5 machine from a natural-language request and returns the source plus a diagram | models can author machines (vendor claim, no measurement) |
| Stately "Generate flow" (<https://stately.ai/blog/2023-09-06-generating-flows>) | vendor, **known** | Experimental premium feature: auto-create machines from text descriptions | same |
| `Alfredvc/aharness` (<https://github.com/Alfredvc/aharness>, <https://github.com/Alfredvc/aharness/blob/main/docs/architecture.md>) | practice/tooling, **known** | A workflow harness for Codex: "the FSM owns the states, gates, typed submissions, approval routing"; it ships an FSM-authoring skill for Codex | long-running agent workflows; prompts cannot enforce a process, an FSM can |
| OpenAI, "Harness engineering: leveraging Codex in an agent-first world" (<https://openai.com/index/harness-engineering/>) | practice report, **known** (summary) | One team shipped a product with all code written by Codex; the work moved to environments, feedback loops and scaffolding | not state-machine specific; supports "make the loop explicit" |
| Blog practice reports: <https://blog.ax0x.ai/coding-agents-are-state-machines>, <https://rcoelho.dev/en/blog/state-machines-agentes/>, <https://rebeccamdeprey.com/blog/why-agent-should-think-in-ltl>, <https://brightlume.ai/blog/why-ai-agent-needs-state-machine-not-prompt-chain>, <https://claudify.tech/blog/claude-code-xstate> | opinion, **known title only** | Agent flows have implicit states inside prompts; an explicit machine makes control flow checkable | fewer impossible states; enumerable paths |

**Inferred.** Every source for the *agent* + state-machine link is opinion, vendor material, or a
practice report. **I found no controlled study or benchmark that measures agent-written code
quality with and without an explicit state machine.**

### 1.3 Academic evidence, adjacent to the claim

| Source | Type | Finding |
|---|---|---|
| "Assessing, Comparing, and Combining State Machine-Based Testing and Structural Testing: A Series of Experiments", IEEE TSE 2010 (<https://doi.org/10.1109/tse.2010.32>) | **measured** | Controlled experiments on state-machine-based testing: fault detection and cost compared with structural testing |
| "Stateful Testing: Finding More Errors in Code and Contracts", ASE 2011 (<https://doi.org/10.1109/ase.2011.6100094>) | **measured** | Stateful testing generates new test cases that break inferred invariants; finds more errors |
| LLM-FSM, FSM reasoning in RTL generation (<https://arxiv.org/abs/2602.07032>) | **measured** | A benchmark that makes models recover FSM behaviour from prose and write it as RTL. State-dependent code generation is hard for current models |
| AutoFSM (<https://arxiv.org/html/2512.11398>) | **measured** | Multi-agent FSM code generation reports frequent syntax errors, low debug efficiency and heavy reliance on test benches. The fix is structure plus tests, not the model alone |
| LLM-Aided Testbench Generation and Bug Detection for FSMs (<https://arxiv.org/html/2406.17132>) | **measured** | Tool feedback (simulation traces) plus prompting improve FSM test coverage |
| Eywa: Automating Model-Based Testing using LLMs (NSDI 2026, <https://www.usenix.org/system/files/nsdi26-mondal.pdf>) | **measured** | LLMs build state-machine models of protocol behaviour from RFCs, then model-based testing generates high-coverage tests |
| "Designing FSMs Specifications from Requirements with GPT 4.0" (<https://arxiv.org/html/2603.29140v1>) | **measured** | FSM quality from prose specs matters, because FSM faults survive into testing |

**Inferred.** The literature supports the *testing* half of the claim (a machine makes tests
enumerable) more strongly than the *robustness* half.

## 2. The forms

### 2.1 Comparison table

Code cost is a rough judgement: **low** = tens of lines per machine, **medium** = a file plus
handlers, **high** = a new dependency plus setup and typing.

| Form | Code cost | What it checks, and when | Test-path generation | Rust support | TypeScript support |
|---|---|---|---|---|---|
| (a) enum + `match`, `transition(state, event) -> state` | low | Exhaustiveness of state handling at **compile time** (Rust `match`, TS `never` check). Transition legality and guards only at **run time** | none built in; you enumerate paths yourself | native; **known** | native; **known** |
| (b) typestate (each state is a type) | medium-high | Transition order, "use after move" and undefined transitions at **compile time**; strongest of the forms | hard: states are types, so a generic path walker is not available off the shelf | strong, uses ownership and move semantics; **known** | possible with branded types, weaker (no move semantics, structural typing); **inferred** |
| (c) statechart library | medium (XState v5) to high (Rust crates with macros) | Hierarchy, guards, entry/exit actions and reachability. Rust `sm` also gives **compile-time** transition checks; `statig` checks at **run time** | XState v5: yes (see below). Rust: none found | `statig` 0.4.1 (2025-07), `rust-fsm` 0.8.0 (2025-07), `sm` 0.9.0 (2019), `sfsm` 0.4.3 (2022) — all **known** from the crates.io API | XState v5 is the reference implementation; **known** |
| (d) reducer only, `(state, event) -> state` | low | Nothing about transitions; only what the function body asserts. Pure, so tests are cheap | none built in; property tests are easy | trivial; **known** | trivial; **known** |
| (e) machine as data (drawable, checkable, walkable) | medium-high | Schema, reachability and dead-state checks at **tool time**; transition legality at **run time** | yes for XState-shaped data (`@xstate/graph`, Stately Studio) | no off-the-shelf equivalent found; the macro crates generate *code*, not data; **inferred** | XState config is data; Stately Studio draws and edits it, MCP converts XState, SCXML, XGraph and D2; **known** |

### 2.2 Notes per form

**(a) enum + match.**
- **Known.** Rust `match` must be exhaustive. TypeScript discriminated unions with a `never`
  default give the same check: "you can use narrowing and rely on `never` turning up to do
  exhaustive checking in a `switch` statement" (<https://www.typescriptlang.org/docs/handbook/2/narrowing.html>).
- **Inferred.** Exhaustiveness is not legality: a `match` arm can return any state, so bad
  transitions still need tests or guards.
- **Known.** Kotlin sealed classes give the same guarantee: with a `when` expression over a sealed
  hierarchy "the Kotlin compiler [checks] exhaustively that all possible cases are covered"
  (<https://kotlinlang.org/docs/sealed-classes.html>).

**(b) typestate.**
- **Known.** Cliffle, "The Typestate Pattern in Rust" (<https://cliffle.com/blog/rust-typestate/>)
  is the common reference for the Rust form.
- **Known, and important for this ticket.** The `statig` docs answer "What advantage does this
  have over using the typestate pattern?": the typestate pattern enforces operation validity at
  compile time, but it suits *API design*, whereas `statig` models "a dynamic system where events
  originate externally and the order of operations is determined at run time". With typestate in
  an event loop "we'd have to use an enum to wrap all our different states", which adds boilerplate
  "for little advantage as the order of operations is unknown so it can't be checked at compile
  time" (<https://docs.rs/statig/latest/statig/>).
- **Known.** The `sm` crate shows the compile checks: a used-after-move machine gives E0382, an
  undefined transition gives E0599 ("no method named `transition`"), and an illegal initial state
  gives E0277. Its `as_enum()` plus exhaustive `match` keeps the state-variant check
  (<https://docs.rs/sm/latest/sm/>).

**(c) statechart libraries.**
- **Known.** XState v5 uses `setup()` + `createMachine`; Stately stopped the v4 `Machine`
  constructor and `interpret()` (<https://stately.ai/docs>, <https://claudify.tech/blog/claude-code-xstate>).
- **Known.** Rust crates, from the crates.io API on 2026-10-15:
  `statig` 0.4.1 (2025-07-05, 4.67M downloads, 250k recent, hierarchical, `no_std`, async actions),
  `rust-fsm` 0.8.0 (2025-07-21, 1.43M downloads, 322k recent),
  `sm` 0.9.0 (2019-11-09, 1.94M downloads, 64k recent),
  `sfsm` 0.4.3 (2022-05-15, 22.9k downloads, 529 recent).
- **Inferred.** `sm` and `sfsm` look stalled (last releases 2019 and 2022, low recent downloads).
  `statig` and `rust-fsm` look maintained. None of them is a statechart *engine* with a visual
  editor or a path generator.

**(d) reducer only.**
- **Known.** Elm Architecture and Redux both define the shape as a pure `(state, event) -> state`
  function (<https://guide.elm-lang.org/architecture/>,
  <https://redux.js.org/tutorials/fundamentals/part-3-state-actions-reducers>).
- **Inferred.** A reducer is the cheapest useful form for agents: one function, one file, no
  dependency, and the tests are direct function calls.

**(e) machine as data.**
- **Known.** XState v5: `getShortestPaths(machine)` returns the shortest path to every reachable
  state by Dijkstra; `TestModel` "creates a test model that represents an abstract model of a
  system under test"; `testCoverage()` checks that all state nodes were traversed
  (<https://stately.ai/docs/graph>, <https://stately.ai/docs/xstate-test>).
- **Known.** `@xstate/test` is now folded into `@xstate/graph`: "The latest version of the
  model-based testing utilities (prev. `@xstate/test`) are now part of the latest `@xstate/graph`
  package" (<https://stately.ai/docs/xstate-test>).
- **Known.** fast-check supplies the same idea without a machine object:
  `fc.commands([...])` with `check(model)` / `run(model, real)` per command, runners
  `modelRun`, `asyncModelRun` and `scheduledModelRun` (for race detection), and a `replayPath`
  for shrinking (<https://fast-check.dev/docs/advanced/model-based-testing/>).
- **Known.** Python Hypothesis gives `RuleBasedStateMachine` with `rule`, `initialize`,
  `precondition`, `invariant`, `Bundle` and `consumes`, plus settings `max_examples` and
  `stateful_step_count` (<https://hypothesis.readthedocs.io/en/latest/stateful.html>).
- **Known.** Rust has `proptest-state-machine` 0.9.0 (published 2026-10-10, in the proptest
  repository): "Strategies and test runners for Proptest State Machine tests", with the
  `prop_state_machine!` macro; "currently, only the `sequential` [mode] is supported"
  (<https://docs.rs/proptest-state-machine/latest/proptest_state_machine/>,
  <https://crates.io/api/v1/crates/proptest-state-machine>).
- **Inferred.** The Rust `quickcheck` crate (1.1.0) does not carry state-machine testing; its crate
  root documents properties only (<https://docs.rs/quickcheck/latest/quickcheck/>). Haskell has a
  separate `quickcheck-state-machine` package. **I did not verify the Haskell package in this
  session.**

### 2.3 What each form moves from the compiler to tests or run time

| Form | Compile time | Test time | Run time |
|---|---|---|---|
| (a) enum + match | exhaustive state handling | transition legality via tests | guards, unknown events |
| (b) typestate | order of transitions, use-after-move | the values inside states | events from the outside world |
| (c) statechart (Rust `sm`) | legal transitions for typed events | actions and guards | events from queues, hierarchy | 
| (c) statechart (XState, statig) | shape and types only | path generation (XState) | guards, unknown events, illegal targets |
| (d) reducer | exhaustiveness only | all behaviour | nothing extra |
| (e) machine as data | schema, if generated | reachability, dead states, paths | transition legality |

## 3. Fit to agents

Evidence on "do agents write each form well" is **thin**. What exists:

| Source | Type | Finding |
|---|---|---|
| SWE-bench Multilingual (<https://www.swebench.com/multilingual.html>) | **measured** | 300 tasks, 9 languages, SWE-agent + Claude 3.7 Sonnet. Per-language resolution: **Rust 58.14%** (25/43), Java 53.49%, PHP 48.84%, Ruby 43.18%, **JavaScript/TypeScript 34.88%**, Go 30.95%, C/C++ 28.57%. Rust had the highest rate although its gold patches touched more lines. Caveat: 43 tasks per language, no human difficulty labels |
| RustAssistant / "Fixing Rust Compilation Errors using LLMs" (<https://arxiv.org/html/2308.05177v1>, ICSE 2025) | **measured** | About 74% peak accuracy on real Rust compilation errors, from prompting plus an iteration loop with the compiler. **Inferred:** a compiler-driven repair loop on Rust is workable |
| MultiPL-E (<https://arxiv.org/abs/2208.08227>) | **measured** | A polyglot code-generation benchmark across many languages, including Rust and TypeScript. I did not extract per-language numbers |
| LLM-FSM and AutoFSM (see 1.3) | **measured** | Current models make many errors when they must implement state-dependent behaviour, and structured generation plus tests is the reported mitigation |
| Stately MCP and "Generate flow" (see 1.2) | vendor | A vendor ships "text description -> XState v5 machine". No accuracy numbers published |
| `aharness` (see 1.2) | practice | A project treats an FSM as the harness around Codex, and ships an FSM-authoring skill for Codex. No measurement |

**I found no benchmark or study that compares agent-written code with and without explicit state
machines.** The seed post stays **opinion**. **Guessed:** nothing in the sources says that a
particular form (typestate vs reducer) is easier or harder for a model to write. A trial is needed
(§5).

**Known, related.** Agents already work with machines indirectly: Eywa has an LLM build state
machines from prose, and the FSM testbench papers have an LLM use tool feedback.

## 4. Language fit

| Language | Forms it supports well | Tool support | Checks that move to tests or run time |
|---|---|---|---|
| Rust | enum + match; reducer; typestate (move semantics); `sm`/`statig`/`rust-fsm`; `proptest-state-machine` for the test side | crates.io (above); `proptest` is actively maintained; no visual editor and no machine-to-path generator found | legal transitions in `statig`; guards and external events at run time; hierarchy is runtime data unless `sm!` is used; serializability of state is manual |
| TypeScript | discriminated unions + `never`; reducer; XState v5 statecharts; machine as data | strongest of the set: XState v5, `@xstate/graph` path and coverage generation, Stately Studio editor, Stately MCP generation, `aharness` | transition legality at run time (XState rejects or ignores an illegal event); typestate is weaker; you get test paths instead of compiler proofs |
| Kotlin | sealed classes + exhaustive `when`; statecharts via libraries not checked here (**I don't know**) | not researched in this session | same as TS for transitions; no path generator checked |
| Elixir/Erlang | `gen_statem` is the OTP behaviour for state machines (<https://www.erlang.org/doc/apps/stdlib/gen_statem.html>) | OTP standard library; no static checks | all checks at run time, except pattern match warnings |
| Python | reducer; Hypothesis `RuleBasedStateMachine` for stateful tests | Hypothesis is mature | all checks at test time or run time; no compile-time layer |
| Elm / F# / OCaml | discriminated unions + exhaustive match; the Elm Architecture is the reducer pattern | Elm is the origin of "impossible states"; no statechart engine | exhaustiveness at compile time; transitions at test time; **not researched further in this session** |

**Does this idea make the platform choice clearer?** **Known facts suggest it does not decide it,
but it does split the criteria:**

- **Inferred.** Rust wins on *the compiler as the checker* (form b, and `sm!`-style statecharts) and
  on measured agent performance on repository tasks (SWE-bench Multilingual). It loses on the
  *test oracle*: no Rust crate found gives machine-to-test-path generation. `proptest-state-machine`
  needs a hand-written reference model.
- **Inferred.** TypeScript wins on *the machine as data* and on the *test oracle* (XState v5,
  `@xstate/graph`, Stately Studio, `aharness`). It loses on compile-time enforcement: `never`
  checks state coverage, not transition legality. Typestate in TS is a pattern, not a language
  guarantee.
- **Known.** Reducers are equal in both languages, and are the cheapest form of all.
- **Guessed.** The repo has mostly backend places, and the seed post recommends the idea "for
  frontend". The evidence does not say the idea is weaker on the backend; it says the *frontend* is
  where it was reported.

## 5. Open questions and a suggested trial

Open questions for the map fog:

1. **Known gap.** No measured study exists on agent-written state machines. Does the "huge bump in
   robustness" reproduce on this repo, and in which form?
2. Does a *machine as data* pay for itself when only one runner consumes it? XState's path
   generation is the main reward for that form (**inferred**).
3. How does a machine survive a restart? A typestate cannot be stored; a persisted state must be
   serializable, which pushes back to a plain enum (**inferred**).
4. What is the cost of async inside a Rust statechart crate for HTTP work? `statig` advertises async
   actions and handlers (**known**), but no evidence was found on axum-shaped workloads.
5. Do generated test paths actually reach the failure modes at issue? Unverified.

Suggested trial (shape only, prototype ticket):

- Pick **one** small stateful unit in this repo, for example the generation or Job state, and write
  it three ways with the same agent and prompt: (1) Rust enum + `match` with a `transition`
  function; (2) Rust `sm!` typestate or `statig`; (3) a reducer function.
- Measure: agent iterations to first green build, compile errors, lines changed, tests the agent
  wrote unprompted, and defects a human reviewer finds in a mutation-style check.
- Separately, if the owner weighs the test oracle: model the same unit in an XState v5 machine and
  compare `@xstate/graph` generated paths against hand-written tests. This is a TypeScript trial
  and does not answer the Rust question.
- Cost note: the Rust typestate and `sm!` forms interact with async and with SQLite persistence;
  both need a serializable boundary (**inferred**).

## Appendix: what could not be checked

- The seed post's replies: not readable (§1.1). The claim about XState and David Khourshid in the
  replies stays **guessed**.
- The seed's chat-assistant claims about this repo are **out of scope** for this asset.
- Haskell `quickcheck-state-machine` and Kotlin/F#/OCaml statechart tooling: **not researched**.
- MultiPL-E per-language numbers: **not extracted**.
