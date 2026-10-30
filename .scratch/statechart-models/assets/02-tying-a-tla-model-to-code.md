# Research 02: how teams tie a TLA+ model to the code it describes

Written 2026-10-29 for [Research: how other teams tie a TLA+ model to code](../issues/02-research-tying-a-tla-model-to-code.md).

Labels: **known** (read in the cited source, or in the code), **inferred** (follows from known facts), **guessed**. The source type of each item is **measured**, **practice**, **opinion** or **tool docs** (§10 says which items I read in full, which in an abridged form, and which in a summary only).

Scope: how a model is kept honest with code. Model checking itself (does the model break its own rules) is assumed. Out of scope: proofs (TLAPS), storage and SQL.

## 0. Answer in brief

Five methods are in use. Ranked for this repo:

| # | Method | One-line reason for the place |
|---|---|---|
| 1 | **Model checked once, each rule carried by one tier-1 test, rule ids paired with the tests by a validator** (the skill's method, made durable) | It costs almost nothing extra: the pause point, the holdable fake and the id-pairing validator already exist here. It ties each rule to one test, so neither side can silently drop the other. It cannot see a rule whose behaviour changed inside an existing test. |
| 2 | **Trace validation** (the code logs its transitions; TLC replays the log through a Trace spec) | The only method that mechanically ties code to model on every run. The instrument point here is one function (`BusyRegistry::hold`), and the gate's model state is small. Cost: a JDK, `tla2tools.jar`, a Trace spec, a trace logger behind the `testing` feature, and one build step. CCF and etcd run it in CI. |
| 3 | **Model re-checked by TLC in CI, code not tied** | Cheaper than method 2 and it keeps the model true to itself, but it claims nothing about the code. It is a prerequisite of method 2, not a substitute. |
| 4 | **Model beside a design doc only** | Free, and it is what AWS does, with a stated result: "we do not know" whether the code implements the design. Here it adds a fourth source of truth that nothing checks, which is the owner's stated worry. |
| 5 | **Model-based test-case generation** (the model's paths become generated tests) | The strongest measured result in the literature (100% branch coverage on MongoDB Realm Sync, one bug found), and the worst fit here: it needs a generated-test harness and a step interpreter over the HTTP app, and its premise (a pure function with inputs and outputs) does not describe the gate. |

The one trade-off behind this order: **this model is small.** TLC enumerates it fully, so a reader can check each transition against the code by hand, and a test per rule covers most of the model's space. That is why method 1 earns the first place and method 2 is its upgrade path, not its replacement. Method 2 pays when the model grows past a page, or when the gate changes often enough that the hand-written tests go stale. Named trigger for that switch: §8.

The vendored skill (`.agents/skills/statechart/SKILL.md`) is method 1 without the pairing: it asks for each bug path to be replayed against the code as a test, and leaves the drift between model and test to the reader.

## 1. The starting points, checked

| The ticket's line | What the source says | Status |
|---|---|---|
| MongoDB, "eXtreme Modelling in Practice", VLDB 2020 | Correct. A. Jesse Jiryu Davis, Max Hirschhorn, Judah Schvimer, PVLDB 13(9):1346–1358, 2020. It reports **two** case studies: model-based trace-checking (MBTC) in the MongoDB Server, which failed, and model-based test-case generation (MBTCG) in MongoDB Realm Sync, which succeeded. | known (read the full text) |
| "The 2024 work on validating traces of distributed programs against TLA+ specs" | Correct, and the exact citation is: Horatiu Cirstea, Markus A. Kuppe, Benjamin Loillier, Stephan Merz, *Validating Traces of Distributed Programs Against TLA+ Specifications*, SEFM 2024 (arXiv:2404.16075, v2 of 2024-09-17). | known (read v2) |
| Newcombe et al., "How Amazon Web Services Uses Formal Methods", CACM 2015 | Correct. CACM 58(4):66–73. The paper's own FAQ is the key line for this ticket: "How do we know that the executable code correctly implements the verified design? The answer is we do not know." | known for the quoted parts (read Lamport's abridged excerpt, which marks its quotes as direct; the CACM full text returned HTTP 403) |

Two names to add to the list, because they carry the numbers the ticket needs:

- **`Smart Casual Verification of the Confidential Consortium Framework`** (Howard et al., NSDI 2025; arXiv:2406.17455). It reports the effort and the commit counts of a trace-validation effort that is now in CI.
- **`etcd-io/raft`, `tla/`**: a Go library with a `with_tla` build tag, NDJSON traces and a Trace spec. The closest published shape to what this repo could do.

## 2. The five methods at a glance

| | Method | What ties code to model | Strength of the tie | Measured or reported result |
|---|---|---|---|---|
| M1 | Model beside a design doc only | Nothing. A person reads both. | none | AWS: bugs found in design, 10 to 14 systems, 7 teams, "we do not know" about the code (practice) |
| M2 | Model re-checked by TLC in CI, code untied | The model survives each change; the code is not read | none (model only) | MongoDB runs its specs "alongside our correctness tests" (practice) |
| M3 | Model-based test-case generation | The model's paths become executable tests | strong, one-way (model → tests) | Realm Sync: 4,913 generated tests, 100% of 86 branches, one real bug (measured) |
| M4 | Trace validation (white-box conformance) | The code logs its transitions; TLC checks the log against a Trace spec | strongest, runs on real executions | CCF: bug found that existing tests passed; now in CI (measured). etcd: in CI (practice). MongoDB Server: failed after 10 weeks (measured) |
| M5 | Model checked once; each bug path replayed by hand as a test | A person writes one test per path, once | weak, one-off | MongoDB's hand-written model-derived tests reached 18 of 86 branches (21%) (measured) |

## 3. M1 — a model beside a design doc only

**What it is.** The model is written before or during design, model-checked, and kept in the repository. The code is then written from it. Nothing reads both.

**Who uses it, with what result.** AWS, the largest published user (known, practice). Newcombe et al. report 10 large systems at the time of writing (14 by print, per Lamport's note), 7 teams, and bugs that "had passed unnoticed through extensive design reviews, code reviews, and testing". The most cited one is a DynamoDB replication bug whose shortest trace had 35 high-level steps. Learning cost is published: "Engineers from entry level to principal have been able to learn TLA+ from scratch and get useful results in two to three weeks." The same paper states the limit plainly: asked how they know the code implements the design, "the answer is we do not know."

**What stops the model from drifting.** Nothing. What is left is the design authority: the model is the record of the design, so a deep change is model-checked first ("Is this change safe?"). The model is not expected to follow the code. When the code moves and nobody moves the model, the model becomes a historical document that still looks authoritative (inferred).

**The cost.** The cost is in the model and the person: two to three weeks to first useful result, and a few hundred lines per system (AWS), plus the JDK and the Toolbox. Upkeep per code change is zero, which is exactly the defect.

**The fit here.** Free to adopt, since `docs/diataxis/reference/game_flow.md` and `docs/specs/` already state the rules in prose. It is also the weakest place to put the model: the repo already has three sources of truth (code, `docs/specs/` enforced by tagged tests, `docs/diataxis/` enforced for structure only), and a fourth that nothing checks is the drift the owner asked about (inferred from the map's "the owner's question").

## 4. M2 — the model re-checked in CI, the code untied

**What it is.** The model lives in the repository, is reviewed like code, and TLC runs on it in the test pipeline. The code is not tied to it.

**Who uses it, with what result.** MongoDB: "We use the same GitHub repository and follow the same code review and testing process with our models as we do for our tests and code. We continuously model-check our TLA+ code alongside our correctness tests" (known, practice). The stated purpose is "to ensure that any changes to our specifications are correct, and to document how the model checker should be configured".

**What stops the model from drifting.** It stops the *model* from rotting (a broken model fails the build). It does not stop the code from leaving the model behind: a model that still checks green says nothing about a code change that was never reflected (known consequence of the design; MongoDB's own problem statement is "it can be difficult to ensure in a large engineering organization that changes in code are accompanied by changes in the specification").

**The cost.** One build step, a JDK, a pinned `tla2tools.jar`, a config per model. For a model of this size, TLC is seconds (inferred). The real cost is the pin and the JDK in the build slot, which the map records as absent from this host.

**The fit here.** Cheap and honest, but it is upkeep, not a tie. It is also the floor under M4: a Trace spec is only meaningful when the high-level model has been checked. Place 3.

## 5. M3 — model-based test-case generation

**What it is.** The model checker enumerates the model's reachable states. A generator turns that set into test cases: an input sequence, and the expected result. Each case runs against the real implementation. Fix a code change that breaks the model, and a generated test fails.

**Who uses it, with what result.** Two data points, both strong.

- **MongoDB Realm Sync (known, measured).** The spec was written by transcribing ~1,000 lines of C++ by hand into TLA+; that took "approximately 40 hours over the course of two weeks". TLC's DOT state graph was parsed by a Golang program that emitted C++ tests. With 3 clients and a 3-element array, it generated **4,913 test cases**; they covered **all 86 branches (100%)** of the merge rules, against 21% for the 36 hand-written tests and 92% for the AFL fuzzer over ~8 million runs. It also found a non-terminating merge case that had been faithful in the model and real in the C++ code, in a mature, heavily fuzzed product. Total cost: "one engineer one month" for the spec and the generator. The authors call MBTCG, when it is possible, "the gold standard of ensuring specification-implementation conformance".
- **Modelator / Atomkraft (known, tool docs, practice without published numbers).** Informal Systems' tool takes a TLA+ model, uses Apalache to enumerate behaviours, and drives tests in Rust and Go; it is used by Atomkraft for Cosmos chains. It needs Python 3.8+, Java 17+ and Apalache.

**What stops the model from drifting.** The direction of the tie is model → code, and it is automatic: "These test cases can be automatically generated and run after any change to the model" (known). A code change that contradicts the model fails a generated test, so the author must either fix the code or regenerate the tests from a changed model — and the second choice is a visible diff. That is a real drift alarm, and the strongest published one after trace validation.

**The cost.** A generator and a test harness: the MongoDB team wrote both by hand ("there's no off-the-shelf solution for turning specs into tests", Wayne, known, opinion). MongoDB's favourable case had a decisive property: the implementation was ~1,000 lines behind one function with defined inputs and outputs, and the operation order did not matter. Where the state space is large or a case is slow, the authors say the space must be sampled and the yield falls.

**The fit here.** Poor as a first step, and the reason is structural, not budgetary: the gate is not a function with inputs and outputs. Its steps are HTTP requests plus LLM calls that code can observe. To turn a generated path into a runnable test the repo would need a step interpreter (model action → app call) and a generated-test emitter — new machinery, and a second way to write the tier-1 tests that already exist. Place 5.

## 6. M4 — trace validation (and, next to it, black-box conformance)

**What it is.** The implementation records a trace: the events it takes at the model's level, and the values of the model's variables (or the parts of them that the code can see). A *Trace spec* — a second TLA+ module that reuses the high-level spec's actions, enabled only when the next logged event matches — constrains TLC to the logged behaviour. TLC then asks whether that behaviour is a behaviour of the high-level spec. No counterexample is produced when it fails; the verdict is "no such behaviour", and the Trace spec's own exploration shows how far the log got (known, Cirstea et al. §4).

The black-box relative in the same family is input-output conformance testing (ioco): the implementation is a black box, and only interface inputs and outputs are observed. I did not read a primary ioco source; that description comes from the trace paper's related-work paragraph (inferred). It matters here only as a boundary: the repo's code can log its own state, so the white-box form applies.

**Who uses it, with what result.**

| Report | What was tied | Result | Status |
|---|---|---|---|
| Cirstea et al., SEFM 2024 | Two-Phase Commit, a snapshot-isolation key-value store, EWD 998 (~500 LoC), two Raft implementations, CCF | "In all cases, trace validation quickly identified executions ... that could not be matched to the high-level specification": an implementation shortcut (a prepared-count instead of a set, which double-counts a resend), an over-strict spec, and grain-of-atomicity mismatches | known (measured) |
| CCF, NSDI 2025 | C++ consensus and consistency protocols | 15 new log statements and a one-day driver change; a **serious safety bug** that existing functional tests had executed but not asserted on ("the tests' assertions were not strong enough" — the failing trace was 305 events long); two data-loss issues found with "just about a dozen system executions" (Kuppe) | known (measured) |
| etcd-io/raft | Go Raft library | A `with_tla` build tag, NDJSON traces, a `Traceetcdraft.tla`, TLC by CLI in the repo's tooling; shipped in v3.6 | known (practice) |
| MongoDB Server, MBTC | C++ replication against `RaftMongo.tla` | **Failed and abandoned after 10 weeks by two engineers.** 423 hand-written tests traced; 120 broke under tracing; only 5 hand-written tests and one fuzz run were actually checked; one passed | known (measured) |

MongoDB's failure is the most instructive data in the literature for a small repo, because the causes are named: the spec was written as an idealised design and had to be changed in **252 of 345 lines** (three weeks) before it could describe the code; the state space grew from 42,034 to 371,368 states and model-checking time from 2 s to 14 min; taking a consistent snapshot of the state inside the locking discipline cost about a month; and the trace checker itself (Python post-processing) had to paper over mismatches. The authors' own advice at the end is the one this repo should read: *"generate traces from implementation modules running in a unit test framework, rather than an integration test of the entire multi-process system ... Modules are simpler than the whole, and non-determinism can be more easily eliminated."*

**What stops the model from drifting.** A code change that contradicts the model makes a real execution fail to validate. That is a per-run check, not a per-review one, and it fails in the safe direction. Both CCF and etcd state the working rule: when the change is a legitimate refactor, update the model and model-check it *before* validating the new traces against it (etcd README, known). CCF reports the scale of that maintenance in commits: 88 commits to the Trace spec and 107 changes to the high-level spec (known).

**The cost.**

| Cost item | CCF's number | What it means here |
|---|---|---|
| Instrumentation | 15 log statements at side-effect-free linearization points, immediately after taking the global lock; driver work "approximately one day" | one log call inside `BusyRegistry::hold`, plus the fields the model needs (§7) |
| The Trace spec | ~400 LoC, but "writing the 400 LoC Trace spec itself was a minor task" | small; the work is in the alignment, not the text |
| First Trace spec, end to end | **~2 engineer-months over 4 months**, including the TLC enhancements, the diagnosis of each discrepancy, and a language expert | the honest number for a first attempt; less here (guessed: a fraction, because one process, one lock, no network) |
| A second Trace spec | **~1 engineer-week over 2 weeks**, done by a domain expert, not the formal-methods expert | the marginal cost falls once the pattern and the tooling exist |
| Tooling | TLC with action composition, DFS, and the TLA+ debugger | TLC + a JDK; the skill already needs both |
| Runtime | DFS: "<1 second" where BFS took "about an hour" | DFS is the required setting. "`StateDeque` was added in Jan 2024" [6]; the skill's runner passes only `-workers 1 -fp 0 -lncheck final` (`scripts/run.ts:70`), so a trace check it runs uses BFS (known) |
| Cost of a false alarm | each discrepancy needs a judgement: bad log, bad mapping, or a real difference | the same judgement a reviewer makes today, but with a trace to read |

**The fit here.** The best structural fit of the five, for three reasons (inferred from known facts):

1. **One instrument point.** Every write of the busy state goes through `BusyRegistry::hold` (`src/application/generation/gate.rs:193`), and `GameWriter::hold` delegates to it. One log call there sees every transition of the gate, in the same place the guard already serialises them.
2. **The model state is tiny.** `Holders` is a `GameId → JobEntry` map, and a `JobEntry` is a job id, an autoplay run id and a progress counter. A trace of the gate is a short sequence of small values, which is where trace validation is cheapest — CCF's whole cost was in aligning a distributed Raft, and MongoDB's in snapshotting a database.
3. **The traces already exist.** Tier-1 tests drive the HTTP app with `ScriptedModel` (which holds a model call) and `AutoplayPause` (which stops the loop at a chosen point). Every class of order that `tests/http/races.rs` covers would emit a trace for free.

It does **not** conflict with the closed ruling that refused a scheduler tool (platform-choice ticket 12). Trace validation does not re-run the code under a scheduler; it replays a recorded, committed log through TLC. The ruling's rule 18 (one hold per writer) even helps: one hold per writer is one clean transition per logged event.

## 7. M5 — the model checked once, each bug path replayed by hand

**What it is.** The method the vendored skill states: write the model, check it with TLC, and for each rule that breaks, "replay each bug path against the real code, as a test or a live repro, one step for each action. If a bug does not occur, think that the model is wrong until you show that it is not" (known, `.agents/skills/statechart/SKILL.md`).

**Who uses it, with what result.** The skill publishes no results. The nearest measured evidence is MongoDB's hand-written path: 36 tests written from the spec covered **18 of 86 branches (21%)**, where generated tests covered all 86 (known). The nearest process evidence is CCF's, which did the same thing in both directions: "The counterexamples obtained in this way were translated into new tests, which confirmed previously unknown problems with the implementation" (known). That is the skill's method, executed by a team that already had CI.

**What stops the model from drifting.** Nothing automatic. Two weak things help (inferred):

- A test that names its rule id is an alarm when the rule is deleted: the id has no test.
- A bug path replayed as a test stays as a regression test, so the *code* cannot leave that path behind, even when the *model* does.

**The cost.** Near zero beyond the model: the JDK is needed when somebody edits the model, not in the gate, and each replayed path is one tier-1 test with the machinery this repo already has.

**The fit here.** The cheapest real tie, if it is given the id pairing the repo already uses elsewhere: `validate_feature_spec.py` pairs each `docs/specs/` scenario with a tagged test, and the map lists the same pattern for model rules as a candidate ("rule ids in doc sentences"). The same validator, extended to a model rule id, turns "each bug path becomes a test" into a check that neither a rule nor its test can disappear silently. That is why this method is first: it is the repo's existing idiom, applied to the model.

Its limit is measured, not guessed: hand-written replay from a model reaches a small share of the model's space (21% at MongoDB). Here that limit bites less, because the gate model is small enough to enumerate by reading, and one test per rule covers it (inferred). It bites the moment the model grows.

## 8. The ranking for this repo

| Place | Method | Why this place | Cost here | What it cannot see |
|---|---|---|---|---|
| 1 | **M5 with rule ids paired to tagged tests** | Reuses `ScriptedModel`, `AutoplayPause`, the tier-1 suite and the `validate_feature_spec.py` idiom. Answers the owner's question ("can a model find gaps between spec and code?") for a small model at a cost measured in hours. | the model, plus one test per rule; a new id type in the validator (guessed: a day) | within an existing test, a rule whose behaviour changed. It does not run automatically. |
| 2 | **M4 trace validation on `hold`** | The only mechanical tie, and the instrumentation here is one log call in one function, with a tiny model state. Placement 2 rather than 1 only because the cost is two orders larger and the model is small. | a JDK + pinned `tla2tools.jar`, a Trace spec, a trace logger behind the `testing` feature, a DFS run, one gate step (guessed: 1 to 3 days for a first working loop, after CCF's shape) | a transition the model has no variable for, and any code path the tests never execute |
| 3 | **M2 TLC in the build** | Keeps the model itself true and is a prerequisite of place 2. It says nothing about the code, so it cannot be the whole answer. | one build step + the JDK pin | every code change that leaves the model behind |
| 4 | **M1 model beside the docs** | Free, and it is what the largest published user does. Here it adds a fourth unvalidated source of truth, so it earns a place only as the place where the model *text* lives. | nothing | everything about the code |
| 5 | **M3 model-based test generation** | The best measured result in the literature, and the wrong shape for this repo: it needs a step interpreter and a generator, and it duplicates the tier-1 suite. | new harness + generator (guessed: weeks) | anything outside the generated paths |

**The trigger for moving to place 2** (inferred): any of these, and the hand-written replay set stops covering the model.

- the gate model grows past one page, or past about 20 transitions;
- the busy state gains a second writer path that is not a `hold` (a background Agent save, a second lock per Game);
- a code change breaks a rule that no test names;
- the model is edited in two consecutive months.

The first and the third are the strong ones: they are exactly where a hand-written set goes stale.

**Two notes on the ranking.**

- Place 2 does not need place 3's step to be useful. A Trace spec can be checked by hand, from `build.py` or from a script, until the model settles.
- Place 1 and place 2 compose. The tests of place 1 are the executions that emit the traces of place 2, so the work of place 1 is not thrown away by adopting place 2.

## 9. Open points for the owner

- Whether the model under discussion is the gate only, or the gate plus the Autoplay loop. The answer changes place 2's cost: the loop is the part with the run id, the stop and the reset, and a trace there crosses two owners (`pause.rs` and `gate.rs`).
- Whether a JDK in the build is acceptable at all (the map records none on this host). If it is not, places 2 and 3 are out, and place 1 is the whole answer.
- Whether the rule ids go in the model comments, in `docs/diataxis/` sentences, or in `docs/specs/` scenarios. The map lists this as a not-yet-specified decision, and place 1 depends on it.

## 10. Sources

Read in full, unless marked.

| # | Source | URL | What it supports | Read |
|---|---|---|---|---|
| 1 | Davis, Hirschhorn, Schvimer, *eXtreme Modelling in Practice*, PVLDB 13(9), 2020 | https://arxiv.org/abs/2006.00915 (HTML v1 read; also http://vldb.org/pvldb/vol13/p1346-davis.pdf) | M1's drift problem, M3's numbers (4,913 tests, 100% of 86 branches, 40 h spec, one engineer-month), M4's failure (10 weeks, 252 of 345 lines changed, 2 s → 14 min), the "modules in a unit test framework" advice | full text (HTML) |
| 2 | Cirstea, Kuppe, Loillier, Merz, *Validating Traces of Distributed Programs Against TLA+ Specifications*, SEFM 2024 | https://arxiv.org/abs/2404.16075 (v2 HTML read); https://doi.org/10.1007/978-3-031-77382-2_8 | the Trace spec mechanism, the case studies, the state-space table, the DFS result, the limit that an imprecise trace can be wrongly accepted | full text (v2 HTML) |
| 3 | Howard et al., *Smart Casual Verification of the Confidential Consortium Framework*, NSDI 2025 | https://arxiv.org/abs/2406.17455 (v1 HTML read); https://www.usenix.org/conference/nsdi25/presentation/howard | 15 log statements, one-day driver, ~2 engineer-months, ~1 engineer-week for the second spec, 88/107 commits, the bug whose trace was 305 events and whose tests were too weak, DFS <1 s vs ~1 h | full text (v1 HTML); the USENIX camera-ready was not read |
| 4 | Newcombe, Rath, Zhang, Munteanu, Brooker, Deardeuff, *How Amazon Web Services Uses Formal Methods*, CACM 58(4), 2015 | https://cacm.acm.org/research/how-amazon-web-services-uses-formal-methods/ (403); Lamport's abridged excerpt, quotes marked as direct: https://lamport.org/tla/amazon-excerpt.html | 10 (14) systems, 7 teams, 2 to 3 weeks to learn, "the answer is we do not know" | abridged excerpt only |
| 5 | `etcd-io/raft`, `tla/README.md` | https://github.com/etcd-io/raft/tree/main/tla | the `with_tla` build tag, NDJSON traces, `Traceetcdraft.tla`, the TLC CLI invocation, the "update the model before validating the new traces" rule | read (raw `README.md`) |
| 6 | TLA+ wiki, trace validation page (a quotation of Markus Kuppe) | https://docs.tlapl.us/using:tlc:trace_validation | "just about a dozen system executions" found two data-loss issues in CCF; the DFS flag `-Dtlc2.tool.queue.IStateQueue=StateDeque`; the limit that TV is not exhaustive | read |
| 7 | `informalsystems/modelator` README | https://github.com/informalsystems/modelator/ ; docs at https://mbt.informal.systems/docs/modelator.html | M3 as a tool: TLA+ in, Apalache under it, Rust and Go test interfaces, used by Atomkraft; needs Python 3.8+, Java 17+ | README read; the docs page returned only a snippet (inferred) |
| 8 | Hillel Wayne, *What if the spec doesn't match the code?*, 2024-03-19 | https://buttondown.com/hillelwayne/archive/what-if-the-spec-doesnt-match-the-code/ | why synchronisation is expensive, that specs drift in practice, that "there's no off-the-shelf solution for turning specs into tests", and that MongoDB's case relied on a low-level deterministic target | read (opinion, practice) |
| 9 | `.agents/skills/statechart/SKILL.md` (vendored) | repo path | M5 as the skill states it: replay each bug path against the code as a test; the reader decides behaviour. Also `scripts/run.ts:70`, the pinned TLC flags | read |
| 10 | `.scratch/_closed/platform-choice/issues/12-decide-race-tests-and-rule-18.md` and `06-research-race-tests-for-guard.md` | repo paths | rule 18 (one hold per writer), the refusal of a scheduler tool, the holdable fake as the reach of every class of order | read |
| 11 | `src/application/generation/gate.rs`, `src/test_support/scripted_model.rs`, `src/application/autoplay/pause.rs` | repo paths | `BusyRegistry::hold` as the one write point, the holder record shape, the holdable fake and the pause point | read |

Two sources I did **not** read, and do not rely on beyond a summary: an input-output-conformance (ioco) primary text (§6, marked inferred), and the Apalache ITF trace-format ADR (https://apalache-mc.org/docs/adr/015adr-trace.html), which is the JSON shape Apalache emits for a trace (inferred from the ADR summary only).
