# Map: the role of TLA+ models in this repo

Labels: wayfinder:map

## Destination

One decision on the role of the statechart skill (`.agents/skills/statechart/`) in this repo: drop it, keep it as an ad-hoc design aid, commit TLA+ models next to the docs with paired rule ids, or use it only to generate tests. One trial TLA+ model of the gate, built from the spec only, backs the decision.

## Notes

- **Plan, don't do.** Tickets produce decisions. The trial TLA+ model of ticket 03 is the one artifact the map makes, and it is the outside check of the decision. Ticket 03 lists the tests it finds as candidates; it lands no test and changes no code (owner, 2026-10-29).
- **Charted 2026-10-29.** Tickets 01 to 04. New tickets number from 05.
- **Origin.** A session that installed the skill and drew three charts: the life of one Job, one Autoplay run, and the state of a new Swipe. The TLA+ models, the views, the reports and the SVGs are in `tmp/statechart/` (git-ignored; its `README.md` holds the commands). Those three TLA+ models were written after a read of `job.rs`, `gate.rs` and `driver.rs`, so they mirror the code and are not spec-only TLA+ models. One rule broke (a stop or a restart between the text save and the state update), and `docs/diataxis/reference/game_flow.md` already accepts that outcome. No code defect was found.
- **The owner's question.** The code, the spec (`docs/specs/`, enforced by the tagged tests) and the docs (`docs/diataxis/`, enforced for structure only) are three sources of truth. Can a TLA+ model find gaps between the spec and the code, or tie the docs to the code, or is it only a mechanical check of the code, like a mutation test?
- **Read first.** The closed platform-choice map ruled on nearby questions:
  - [Research: what explicit state machines and reducers give the new design](../_closed/platform-choice/issues/01-research-state-machines-and-reducers.md) — an explicit state machine adds detection, not prevention.
  - [Research: race tests for the guard and the job id check](../_closed/platform-choice/issues/06-research-race-tests-for-guard.md) — every write is one hold of one lock, so the holdable fake reaches every class of order.
  - [Decide the race tests and guardrail rule 18](../_closed/platform-choice/issues/12-decide-race-tests-and-rule-18.md) — no scheduler tool; the race scenarios live in `tests/http/races.rs`.

  This map asks what a TLA+ model adds on top of those rulings and `races.rs`, not whether model checking is good in general.
- **Toolchain (known).** The skill's runner needs a JDK 11+ and `tla2tools.jar`, or `mise`. This host has neither. The first session used Temurin 21 in `/tmp/statechart-tools/jdk` and `tla2tools.jar` 1.7.4 (checksum matches the pinned one) through the `JAVA` and `TLA2TOOLS` variables. Pele 1.9.0 for the SVGs is in `/tmp/pele-render`. `/tmp` may be cleared.
- **The vendored skill.** Copied from `cursor/plugins` `pstack/skills/statechart` on 2026-10-29, without its `scripts/test/` folder (3.8 MB). It is untracked and has no `skills-lock.json` entry.
- **Terms.** Use `CONTEXT.md`: Game, Job, Message, Swipe, Persona, Trigger, Autoplay run.
- **"Model" has two meanings here; never write it alone.** A **TLA+ model** is the state-machine file that TLC checks: its variables, its start state, its actions and its rules. A **model call** and the **LLM** are the language model, as in `CONTEXT.md`; `ScriptedModel` is the **LLM fake**. **Model checking** is the name of the technique that TLC runs. The two assets of tickets 01 and 02 were written before this rule and use "model" alone for the TLA+ model.
- **Skills:** `/grilling` and `/domain-modeling` for decisions; `/research` for outside facts; `/prototype` and `statechart` for the trial TLA+ model.

## Decisions so far

- [Research: which rule sentences of the gate the tests already pin](issues/01-research-gate-rules-and-their-tests.md) — 39 gate rule sentences: 20 pinned, 16 partly pinned, 3 not pinned; the gaps are the gate's own orders (a delete or a config write during a plain Job, the busy check before another refusal, a stop that clears a failure line, the restart); the six platform-choice interleavings are all present; enough ground for the trial, but a counterexample is a test only where a route reaches it.
- [Research: how other teams tie a TLA+ model to code](issues/02-research-tying-a-tla-model-to-code.md) — five methods; for this repo, first a TLA+ model checked once with each rule carried by one tier-1 test and paired by rule id (the skill's method plus the `validate_feature_spec.py` idiom), then trace validation on `BusyRegistry::hold` as the upgrade path (the only mechanical tie; CCF found a bug that tests executed but did not assert on); TLC-in-CI alone, a TLA+ model beside the docs, and generated tests rank lower; a JDK in the build gates places 2 and 3.
- [Prototype: one TLA+ model of the gate, built from the spec only](issues/03-prototype-spec-only-gate-model.md) — no code defect; three "own id" rules (G2 in the spec, G1 in part, G3 not clear in autoplay.md); six candidate tests for orders that no test forces; 5 of 15 rules true by construction; its value is a one-time audit, and `game_flow.md` need not hold the rules it found (owner).

## Not yet specified

All of these hang on the decision of the role:

- **Where a rule of a TLA+ model pairs: a spec scenario or a doc sentence.** A validator pairs each invariant with one rule, as `validate_feature_spec.py` pairs scenarios with tests. The owner's leaning (ticket 03) is that `game_flow.md` need not hold every rule, which favours the spec scenario. The rules that broke in ticket 03 already map to spec scenarios (39.4, 39.5, 41.6, 41.7, 41.17, the intro of autoplay.md).
- **The toolchain.** `mise` config in the repo, a `build.py` step, a CI step, or none.
- **A repo-local addendum to the skill.** The spec-only discipline (read `src/` only for the replay), where TLA+ models live, the `CONTEXT.md` terms in labels, each gap of the docs as a constant, the "all off" check for rules that hold by construction, small bounds, and a tier-1 sketch for each break (ticket 03).
- **The trigger for a new audit.** What starts a new one-time TLA+ model, for example a new flow or a change to the gate. Candidate flows: the Autoplay loop with stop and reset, a config write against a busy Game, the status rule of `GameViewQuery`.
- **Provenance of the vendored skill.** An entry in `skills-lock.json`, the skill's own tests, or neither.

## Out of scope

- **Mutation tests of the code.** The unclebob-quality-tools map owns `mutator`.
- **Storage transactions, prompts, JSON parsing, UI copy.** A TLA+ model cannot check them.
- **Changes to the gate or to `tests/http/races.rs`.** The trial yields candidate tests only.
- **A trim of `game_flow.md`.** The event rules and the mechanics in `game_flow.md` that a spec or the code already holds. This is docs work, not the role of the skill. Its start list is section 12 of the [ticket 03 asset](assets/03-gate-spec-only-model.md).
- **The findings of ticket 03.** The six candidate tests (G1, G3, R05, R07, R08, R09), the words "a Job of the run" in the intro of autoplay.md (G3), and a spec scenario for the delete of a Game during a plain Job. Each needs a test or spec effort. They are in sections 6, 7 and 11 of the [ticket 03 asset](assets/03-gate-spec-only-model.md).
