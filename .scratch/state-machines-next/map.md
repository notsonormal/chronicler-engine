# Map: the next state machines

Labels: wayfinder:map

## Destination

Each implicit state machine in scope has a decided disposition. The five dispositions are: leave it, make it explicit in the code, give it a model-based test, give it a TLA+ model and a verify skill, or generate its diagram (ticket 01). A machine can get more than one. The owner decides the product behaviour that a disposition depends on. Each disposition other than "leave it" is a work ticket in `.scratch/state-machines-work/issues/`, ready to hand off.

## Notes

- **Charted 2026-11-08** from a sweep of the code for implicit state machines. [The catalogue](assets/catalogue.md) lists 53 machines. 42 are in scope: groups B, C, D, E and G. The five scout reports (`assets/scout-*.md`) hold the full file:line lists. [`assets/scan-brief.md`](assets/scan-brief.md) holds the definition the scouts got.
- **The catalogue is a hypothesis.** One model family (DeepSeek Flash) made it. The charting session checked the main claims and refuted one. Each apply ticket checks its machines in the code again before it decides, because the code moves.
- **Plan, don't do.** The tickets make decisions. The work goes to `.scratch/state-machines-work/issues/`, a folder with no map (owner, charting). An apply ticket files a plain defect there at once, because a plain defect needs no decision. The work-records map can move that folder later.
- **Takes over from statechart-models.** The fog item "TLA+ models of other areas, for goal (c)" moved here from [the statechart-models map](../statechart-models/map.md) (owner, charting). Statechart-models keeps the gate, its tickets 14 and 15, and the `/statechart` tooling. Its three goals also apply to a TLA+ disposition here: (a) find bugs, (b) align the code, the specs and the docs, (c) documentation that replaces prose.
- **Two grilling tickets only** (owner, charting). The rule ticket (01) and the behaviour ticket (06) need the owner. The apply tickets (03 to 05) are AFK. They apply the rule, and they collect the questions that only the owner can answer: the product behaviour (what the player sees, which source wins). They do not ask the owner which disposition a machine gets. Ticket 06 shows the owner only the dispositions that cost work.
- **Read first.**
  - [Research: what explicit state machines and reducers give the new design](../_closed/platform-choice/issues/01-research-state-machines-and-reducers.md): an explicit state machine adds detection, not prevention. The busy state earns one, and most other parts do not.
  - [Decide the gaps of the first run, and the future of the verify skill](../statechart-models/issues/09-grilling-decide-the-gaps-and-the-verify-skill.md): what a TLA+ model costs (a maintain pass on each change of a covered function, the staleness warning of `build.py`), and where generated diagrams go (`docs/diataxis/reference/diagrams/`).
- **Terms.** Use `CONTEXT.md`. "Model" has two meanings, so never write it alone. A **TLA+ model** is the file that TLC checks, and a **model call** goes to the LLM. A **disposition** is the decision for one machine. It is a term of this map only.
- **Skills.** `/grilling` and `/domain-modeling` for tickets 01 and 06. `/codebase-design` for an "explicit in the code" disposition. `proptest` is a dev-dependency already (`game_rules_tests.rs`); `proptest-state-machine` would be new. `/statechart` when an apply ticket tries a TLA+ model. Only the person can start `/statechart`, because its `SKILL.md` sets `disable-model-invocation: true`.

## Decisions so far

- **The rule that chooses a disposition** ([01](issues/01-grilling-decide-the-disposition-rule.md)). Five dispositions, because a state machine serves prevention, detection and validation. Explicit in the code: a state that is not valid can be reached today, by requests or in stored data. Model-based test: the machine becomes explicit, or two or more places make its moves. TLA+ model: parts that run at the same time, with a client-visible order difference or prose to replace; no count limit; an area that shares a variable with `VerifyGate.tla` extends `verify-gate`. Diagram: generated from each TLA+ model or transition table, for validation first. Leave it: none of these.
- **The outside check of the catalogue** ([02](issues/02-research-scan-again-with-another-model-family.md)). Five Claude Sonnet scouts, with no access to the catalogue, agree with 53 of its 54 rows and add one machine (B12, the Game config state). The check refuted two claims of scan 1 and five claims of scan 2. Five plain defects went to `state-machines-work` (03 to 07), and ten behaviour questions and fifteen "to check" items went to tickets 03 to 05. Two findings in rows A1 to A7 went to `state-machines-work` too (owner, 2026-11-09), each with a note that its fix brings the `verify-gate` TLA+ model along: 08, a failed Job whose failure line does not save; 09, an accept keeps the Options failure (also question 13 of ticket 03).
- **The rest of the server** ([04](issues/04-research-apply-the-rule-to-the-rest-of-the-server.md)). 13 machines: 9 "leave it", 4 "explicit in the code" (D2 text check, D3 mode preset registry, D4 Allowed Modes, E1 migration ladder). The four explicit forms are value constraints with no transition table, so they get no model-based test and no diagram. D3 and E1 depend on behaviour questions 9 and 5. Ten questions and four draft work tickets wait for ticket 06; bug 20 filed.
- **The server Game state** ([03](issues/03-research-apply-the-rule-to-the-server-game-state.md)). 15 machines: 9 "leave it". TLA+ extension of `verify-gate` for two races a client can see (B1 the two-step status read, B12 the preset check outside the hold). Model-based tests for B2 (the dock record) and B4 (the encounters and fire marks). Explicit in the code for B10 (Role health retention) and B11 (change counters, through bug 11). Ten questions and four draft work tickets wait for ticket 06; bugs 10 and 11 filed.

## Not yet specified

- **The JS test runner** for a model-based test of a browser machine. The repo has no JS test runner today: the browser tier is Rust with Playwright, and `jsdom` is in `package.json`. This is clear after ticket 05 gives a browser machine that disposition.
- **The order and the groups of the work tickets** in `.scratch/state-machines-work/issues/`. This is clear after ticket 06.
- **Machines outside the catalogue** that the second scan (ticket 02) finds after the apply ticket of their area is resolved.

## Out of scope

- **The machines with a TLA+ model**: the busy registry, the Job, the Autoplay run, the active pointer, the two writes of a new Swipe state, `JobPlan`, and the failure line and the Options failure (catalogue rows A1 to A7). The statechart-models map owns them.
- **The test machinery that is compiled into `src/`**: Test storage, `MockBackend`, `AutoplayPause`, `JobTasks` and the Harper cache (catalogue group F; owner, charting). Its risks do not change what a player sees.
- **The work itself.** This map makes decisions only.
