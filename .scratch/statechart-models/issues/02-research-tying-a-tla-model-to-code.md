# Research: how other teams tie a TLA+ model to code

Type: research
Status: resolved
Blocked by: —

## Question

What methods do teams use in practice to tie a TLA+ (or similar) model to the code it describes, and what does each one cost?

For each method, answer:

1. **What it is.** For example: trace validation (the code logs its steps, and a checker replays the log against the TLA+ model), model-based test generation (the TLA+ model's paths become tests), conformance checks, a TLA+ model kept next to a design doc only.
2. **Who uses it, with what result.** Name the source: an industry report, a paper, a tool's docs. Prefer primary sources.
3. **What stops the TLA+ model from drifting from the code**, and what happens in practice when nothing stops it.
4. **The cost.** The tooling, the people's skill, the time per TLA+ model, the upkeep per code change.
5. **The fit here.** One Rust process, one `Mutex` registry (`BusyRegistry` in `src/application/generation/gate.rs`), a holdable LLM fake (`ScriptedModel`) and a test pause point (`src/application/autoplay/pause.rs`) for tier-1 tests. Which method could reuse that, and which needs new machinery?

End with a short ranking of the methods for this repo, with the reason for each place.

## Context

- Use `/research`. Write the findings as a linked asset in `.scratch/statechart-models/assets/`.
- **Known starting points** (check them; do not trust these lines): MongoDB's "eXtreme Modelling in Practice" (VLDB 2020) on conformance checking and model-based test generation; the 2024 work on validating traces of distributed programs against TLA+ specs; Amazon's reports on the use of TLA+ (Newcombe et al., CACM 2015).
- **The skill in this repo** (`.agents/skills/statechart/SKILL.md`) asks for each bug path to be replayed against the code as a test. That is one point on the scale of methods; place it.
- Platform-choice ticket [Research: what explicit state machines and reducers give the new design](../../_closed/platform-choice/issues/01-research-state-machines-and-reducers.md) already looked at state machines in code. This ticket is about a separate TLA+ model of the code, not about the shape of the code.
- Out of scope: proofs (TLAPS), and model checking of storage or SQL.

## Answer

Asset: [How teams tie a TLA+ model to the code it describes](../assets/02-tying-a-tla-model-to-code.md), with a sources table (URL, and whether each source was read in full, abridged, or in summary only).

**Five methods, ranked for this repo.**

1. **TLA+ model checked once, each rule carried by one tier-1 test, rule ids paired with the tests by a validator.** This is the skill's method (replay each bug path as a test), made durable with the `validate_feature_spec.py` pairing idiom. It reuses `ScriptedModel` and the pause point. It runs no check automatically, and it cannot see a rule whose behaviour changed inside an existing test.
2. **Trace validation.** The code logs its transitions, and TLC replays the log through a Trace spec. It is the only mechanical tie between the TLA+ model and the code. The instrument point here is one function (`BusyRegistry::hold` in `gate.rs`), and tier-1 tests would emit the traces. Cost: a JDK, `tla2tools.jar`, a Trace spec, a logger behind the `testing` feature, and one build step. It does not conflict with the refused scheduler tool: it replays a recorded log; it does not run a schedule.
3. **TLC in the build, code untied.** It keeps the TLA+ model true to itself and says nothing about the code. It is a prerequisite of place 2.
4. **TLA+ model beside a design doc only.** AWS's practice, and AWS's own verdict on the code: "the answer is we do not know". Here it is a fourth source of truth that nothing checks.
5. **Model-based test generation.** The best measured result (MongoDB Realm Sync: 4,913 generated tests, 100% of 86 branches, one real bug), and the worst fit here: it needs a step interpreter and a generator, and it duplicates tier 1.

**Trigger to move from place 1 to place 2** (inferred): the TLA+ model grows past about 20 transitions or one page, a busy-state write path appears that is not a `hold`, a code change breaks a rule that no test names, or the TLA+ model is edited in two consecutive months.

**Evidence.** For trace validation: CCF (NSDI 2025) — 15 log statements, about 2 engineer-months for the first Trace spec and about 1 engineer-week for the second, and a serious safety bug that the existing tests executed but did not assert on. Against: MongoDB Server's trace checking failed after 10 weeks (the spec needed 252 of 345 lines changed); its authors advise tracing modules in a unit-test framework, which is the shape of this repo's tier 1. For hand-replayed paths: MongoDB's 36 hand-written tests reached 18 of 86 branches (21%).

**Starting points.** All three in the ticket are correct (MongoDB VLDB 2020; Cirstea, Kuppe, Loillier, Merz, SEFM 2024; Newcombe et al., CACM 2015, read through Lamport's abridged excerpt because the CACM page returned 403). Added: Howard et al., *Smart Casual Verification of CCF* (NSDI 2025), and `etcd-io/raft` `tla/`, the closest published shape (build tag, NDJSON traces, TLC).

**Coordinator checks.**
- Correct: the `testing` Cargo feature exists; the skill's runner pins `-workers 1 -fp 0 -lncheck final` (`scripts/run.ts`), so a trace check through it uses BFS, not the DFS that CCF needed for speed.
- Wrong in the asset §6: "a `JobEntry` is a job id, an autoplay run id and a progress counter". In `gate.rs`, a `JobEntry` is a job id and a `Step`; the `Holder` is either a plain Job with its entry, or an Autoplay run with its run id, its progress and an optional entry. The conclusion (the state of a TLA+ model of the gate is small) still holds.
- Not checked: whether the pinned `tla2tools.jar` 1.7.4 has the trace-validation features (`StateDeque`, action composition) that the CCF work used.

**Open points the asset hands to the decision ticket:** the gate only, or the gate plus the Autoplay loop; whether a JDK in the build is acceptable at all (if not, places 2 and 3 are out); where the rule ids live (comments in the TLA+ model, `docs/diataxis/` sentences, or `docs/specs/` scenarios).
