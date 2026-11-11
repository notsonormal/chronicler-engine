# Decide the rule that chooses a disposition, and a limit on TLA+ models

Type: grilling
Status: resolved
Blocked by: —

## Question

Which rule chooses the disposition of an implicit state machine? The four dispositions are: leave it, make it explicit in the code, give it a TLA+ model and a verify skill, or document it as a generated diagram. Is there a limit on the number of TLA+ areas? What does an apply ticket (03 to 05) write for each machine?

## Context

- **The draft rule from charting.** The owner has not decided it.
  1. **TLA+ model and verify skill**: the machine has parts that run at the same time, and their order can break a rule. Examples are the screen stamp against a load, or the failure line against a Job.
  2. **Explicit in the code**: the writers are in two or more modules, and a combination that is not valid can be reached or stored.
  3. **Generated diagram**: the docs already describe the machine in prose, and the TLA+ model or the code can generate the diagram.
  4. **Leave it**: the machine is local to one file, and no combination that is not valid can be reached.
- **Points that the grilling must cover.**
  - What "explicit in the code" means in Rust: one enum and one transition function, a typestate (a move that consumes a token, as `JobWriter` does), or a private type with a checked constructor. And what it means in the browser JS.
  - Whether a diagram must come from a TLA+ model or from the code. Statechart-models ticket 09 decided that the diagrams of the gate are generated into `docs/diataxis/reference/diagrams/`.
  - The limit on TLA+ models. Each one needs a maintain pass on each change of a covered function. `gate.rs` and `job.rs` had about one commit a day (statechart-models ticket 09). A limit can be a count or a test of value.
  - Whether one machine can get two dispositions, for example explicit in the code and a diagram.
  - The columns of the disposition table of an apply ticket, and what counts as evidence.
- **Known facts.**
  - Platform-choice research 01: an explicit form prevents none of the known problems that the design does not already settle. It adds detection, which means tests generated from the tables.
  - The catalogue labels each machine Implicit, Scattered or Explicit. Of the 42 in scope, most are Implicit.
- Use `/grilling` and `/domain-modeling`. The owner answers for themselves.

## Answer

Resolved 2026-11-09 in three grilling rounds (owner). The rule has **five dispositions**, not four. A state machine has three uses: prevention (types stop a state that is not valid), detection (generated sequences of events check the code against the machine), and validation (a person checks the machine against intent through a diagram). The measured evidence supports detection (platform-choice research 01), so detection is a disposition of its own.

**What "a state that is not valid can be reached" means.** A sequence of requests reaches the state today, or stored data holds it. The code read gives the path with file:line. A state that the types can hold but no path reaches does not count. A machine that is only open (for example `pub` fields, as in `PromptContext`) can get a small fix in its own module with no disposition.

**The five dispositions and their tests.** A machine can get more than one.

| Disposition | A machine earns it when | Form |
|---|---|---|
| Explicit in the code | a state that is not valid can be reached, whatever the number of files | Rust: the smallest form that makes the state impossible to write (an enum in place of flags, a private type with a checked constructor, or a typestate). JS: one state object for each module and one function that makes the moves; only that function writes the DOM attributes. |
| Model-based test | the machine becomes explicit (the test is generated from its table), or two or more places make its moves, even if no state that is not valid can be reached | A reference state machine in the tests only. Generated sequences of events run against the real code, and each step compares the two states. Production code does not change. |
| TLA+ model and verify skill | the machine has parts that run at the same time, and a client can see a different result in some order, or the TLA+ model replaces prose in `docs/diataxis/` | No count limit. An area that shares a variable with `VerifyGate.tla` (the busy registry, `failLine`, `optFail`) extends `verify-gate`. Any other area gets its own `verify-<area>` skill. |
| Diagram | the machine has a TLA+ model or a transition table | Generated from the TLA+ model or the table, never hand-written. Its first use is validation by the owner. Where `docs/diataxis/` describes the machine in prose, the diagram also replaces that prose. |
| Leave it | it earns none of the above | — |

**What an apply ticket writes.** One table row for each machine, with these columns: row id and machine; the state that is not valid and its path (file:line, known or inferred); the places that make the moves (count and file:line); parts that run at the same time, and a client-visible order difference; the prose in `docs/diataxis/` that describes the machine; the dispositions and the form; the behaviour question that the machine depends on. An inferred path is not enough for a disposition other than "leave it". It becomes a "to check" item of the work ticket, and the disposition holds only if the check confirms the path.

**Changes to other tickets.** The map's Destination names five dispositions. Ticket 06 shows the owner a diagram for each disposition that costs work, so tickets 03 to 05 draw a review sketch of each such machine.

## Comments

### 2026-11-09: grilling round 1 (owner accepted every recommendation)

1. **Explicit in Rust (Q1, C).** The smallest form that makes the combinations that are not valid impossible to write: an enum in place of flags, a private type with a checked constructor, or a typestate. The apply ticket chooses the form for each machine. Reason: an explicit form adds detection, not prevention (platform-choice research 01), so it is worth its cost only where it prevents a state that is not valid.
2. **Explicit in the browser JS (Q2, A).** One state object for each module and one function that makes the moves. Only that function writes the DOM attributes. Reason: most dashboard risks are a flag that one place sets and another place must clear.
3. **TLA+ models (Q3, B).** A test of value, with no count limit. The machine has parts that run at the same time. Also, a client can see a different result in some order, or the TLA+ model replaces prose in `docs/diataxis/`.
4. **Diagrams (Q4, B).** From a TLA+ model, or from a transition table in the code that a script draws. Never hand-written.
5. **The two gaps of the draft rule (Q5, C).** A machine where a state that is not valid can be reached becomes "explicit in the code", whatever the number of files. A machine where no such state can be reached is "leave it", whatever the number of writers.

### 2026-11-09: grilling round 2 (Q1, Q2 and Q4 accepted; Q3 and Q5 open)

1. **"A state that is not valid can be reached" (Q1, A).** A sequence of requests reaches the state today, or stored data holds it. The code read gives the path with file:line. A state that the types can hold but that no path reaches does not count. A machine that is only open (for example `pub` fields, as in `PromptContext`) can get a small fix in its own module with no disposition.
2. **Two dispositions (Q2, B).** "Explicit in the code" and "TLA+ model" can go together when a machine passes both tests. A diagram can be added to either.
3. **The home of a new TLA+ area (Q4, C).** An area that shares a variable with `VerifyGate.tla` (for example the busy registry, `failLine`, `optFail`) extends `verify-gate`. Any other area gets its own `verify-<area>` skill.
4. **The owner's comment on Q3 (diagrams):** "A state machine itself is a form of verification/testing right? You shouldn't just think it limited to documentation." Round 1 framed the explicit form as prevention only. Platform-choice research 01 found that the measured evidence supports the testing half (stateful tests find more faults), not the prevention half. Round 3 takes this up.
