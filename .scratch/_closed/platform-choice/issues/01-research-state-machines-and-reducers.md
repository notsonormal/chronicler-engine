# Research: what explicit state machines and reducers give the new design, in Rust and in other languages

Type: research
Status: resolved
Blocked by: —

## Question

Matt Pocock reports that coding agents write more robust code, and code that is easier to change, when he asks them for explicit state machines or reducers. Does that hold, and what would it give this effort?

Answer three parts:

1. **The claim.** What is the evidence that explicit state machines or reducers make agent-written code more robust? Where does the evidence come from (practice reports, studies, tool authors), and where does it apply (frontend, backend, long-running work)?
2. **This design.** Where would an explicit state machine or a reducer change the [target design](../../turn-state-model/assets/target-design.md) and [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md)? Which of the 36 known problems (target design §10) would it prevent or detect, that the design does not already prevent?
3. **The language question.** How does each candidate language express state machines and reducers, and how does that change the platform choice?

## Context

- **Seed.** [The state-machine idea (Matt Pocock post)](../assets/seed-state-machine-idea.md): the source post, and a chat assistant's claims about this repo. The assistant read today's code, not the target design. Check each claim before you use it.
- **What the design already has (known).**
  - Target design §4.4 gives the states of one generation (room check, narrator call, text save, analysis and game rules, state update) and what stays after a stop or a restart. The Job has the same states one level up.
  - Target design §4.2 gives the plan of each request as a fixed order of steps. Sketch 25 §3.3 leaves the code shape open: "one runner over the plan, or one function per request kind" (Q1).
  - Sketch 25 makes the world rules a pure function, `apply_world_rules(start, analysis, world, mode)`. That is already a reducer in shape (inferred).
  - The dashboard action area already runs one client state machine (`checking → generating → preview → idle`): [Rework the action area so a text check cannot strand it](../../dashboard-ui-review/issues/42-rework-the-action-area.md).
- **The problems that a language move must solve** (map Notes): A, the edit-to-test loop; B, parallel agents that wait for the build slot or run out of memory; D, design churn and defects. This idea bears on D. It can also bear on the test oracle (map fog), because a state machine can generate its own test paths (inferred).
- **What depends on Rust** (map Notes, handoff item 4): the guardrail rules at compiler level (sketch 25 §6.2) and one trait per Agent hook. A state machine in the type system (for example a typestate) adds to that list. A state machine in a library or in data does not (inferred).
- **Terms.** Use `CONTEXT.md`: Job, Action, Message, Swipe, Snapshot, Retry, Trigger. "Turn" and "Action Pipeline" are deprecated.

## Scope

1. **The claim and its evidence.** The seed holds the post text. The post is opinion, and it says "for frontend". Find the replies if they can be read. Find other reports from practice, studies, or tool authors on agents and state machines (XState and Stately, statecharts, the Elm architecture, Redux reducers). Separate measured results from opinion. Record the claimed reason: fewer impossible states, one place for transitions, transitions that a test can enumerate.
2. **The forms.** Compare the forms that "a state machine" can take:
   - an enum and a `match`, with a `transition(state, event) -> state` function;
   - a typestate (each state is a type, so the compiler refuses a bad transition);
   - a statechart library (XState v5 in TypeScript; for Rust, `statig`, `rust-fsm`, `sm`, or others that the research finds);
   - a reducer only, `(state, event) -> state`, with no named machine;
   - a machine held as data, which a tool can draw, check and walk.

   For each form, record how much code it adds, what it checks and when (compile time, test time, run time), and whether it can generate test paths (for example `@xstate/graph`, or `proptest-state-machine` in Rust).
3. **Fit to the design.** Map the forms onto the design. Candidate places: the states of one generation (§4.4), the Job and its plans (§4.2, sketch 25 Q1), the guard and the job entry (§3.3), the Retry by the kind of the last Message (§4.5), the world rules, and the dashboard. For each place, say whether the design is already a state machine in all but name, which known problems (§10) an explicit form would catch, and what it costs. Name the places where a state machine does not fit.
4. **Fit to agents.** How well do current models write and change each form? Look for evidence, not opinion: reports, benchmarks, or a small trial. A trial is out of scope for this ticket; propose one as a prototype ticket if the evidence is thin.
5. **Fit to the language question.** For Rust, TypeScript, and any other language that the research finds relevant, give the forms that the language supports well, the tool support, and the checks that the form moves from the compiler to tests or to run time. Say whether this idea makes the platform choice clearer, and in which direction.

Report facts and options. Do not decide the form, and do not decide the language. Those choices are grilling decisions for the owner.

## Output

An asset `../assets/research-01-state-machines-and-reducers.md` with:

- The claim, its sources, and a measured-or-opinion label for each source.
- A table of the forms: form, code cost, what it checks and when, test-path generation, Rust support, TypeScript support.
- A table of the design places: place, already a state machine (yes / partly / no), known problems caught, cost.
- The fit to agents, with sources.
- The effect on the platform choice, and any new questions for the map's fog.

Label each fact known, inferred, or guessed. Cite symbols and section numbers, not line numbers.

## Answer

Resolved 2026-10-15. Assets: [research 01](../assets/research-01-state-machines-and-reducers.md) (summary and the fit to the design) and [web sources](../assets/research-01-web-sources.md) (the outside evidence).

1. **The claim.** The post is opinion (practice report, "for frontend"). Its replies cannot be read. No study or benchmark compares agent-written code with and without an explicit state machine (known: none found). Measured work supports the testing half of the claim (state-machine and stateful tests find more faults), not the robustness half.
2. **This design.** The design already has the main gain by other means: no stored state that can disagree (derived status, memory-only busy mark, derived start and current state), one writer, one generation path, and a `JobWriter` token that is a typestate in all but name. An explicit form prevents none of the 36 known problems that the design does not already settle (inferred). It adds detection: tests generated from the tables of target design §4.3 to §4.5. It closes three small gaps: G1, every step sets its step name through one wrapper; G2, a token from `save_text` that `save_state_update` consumes (D-03 by structure); G3, a generated test that stops a Job at every step boundary and checks the §4.4 table. Places where it does not fit: prompt assembly, storage, the views, create and reset, the Agents' calls, and the status (a derived value).
3. **The language question.** The idea does not make the platform choice clearer. Rust is stronger at compile time (typestate with move semantics, exhaustive `match`). TypeScript is stronger for a machine held as data (XState v5, `@xstate/graph` path generation). The design needs neither a statechart library nor a machine held as data, so XState is not a reason to move. G2 adds at most one row to "what depends on Rust". The model-based test that the design needs exists in both languages (`proptest-state-machine`; `fast-check` `fc.commands`). SWE-bench Multilingual gives Rust 58 % and JavaScript/TypeScript 35 % for one model, but on different repositories per language, so it is not a like-for-like comparison.

New questions went to the map's fog: the build plan's use of G1 to G3 and fork 1; a measured trial of the forms (shared with "A measured feedback loop"); race tests around the guard.

## Comments

- 2026-10-15, owner feedback: the question is about the design in general, not the post's "for frontend" premise. Would the design improve if some parts were definitely a state machine, not "mostly one"? Added [research 01 §3.6](../assets/research-01-state-machines-and-reducers.md#36-parts-that-are-definitely-a-state-machine): the busy state of a Game probably earns it; the Job is open; the other parts do not. The Job case went to [Prototype: the Job as a pure state machine, next to a straight `generate`](02-prototype-job-as-pure-state-machine.md).
