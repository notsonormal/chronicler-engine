# Task 20: a decision for each doc practice

Status: reference. Asset of the ticket [Recommend a decision for each doc practice](../issues/20-task-recommend-the-doc-practices.md) in the [map](../map.md). Made on 2026-11-01 at repo commit `1a0c9162` plus the working tree.

This asset recommends. It edits no doc, skill or `AGENTS.md`, and it drafts no feature-map page. Each claim is labelled **known** (read or counted in a file) or **inferred** (my reasoning). A source follows each claim.

## Summary

Six of the seven practices end as covered or dropped. The map gets no follow-on build and no brief. Reason: the evidence shows that Chronicler already has the useful part of each practice, and the missing parts belong to tickets that exist.

| Practice | Recommendation | Cost if adopted | Follow-on |
|---|---|---|---|
| C-01 One always-loaded spec | **Covered.** No change. | A Hex-style file would add the Reference content to every session (inferred) | none |
| C-02 Invariants with exact values | **Drop** for `docs/diataxis/` and for the instruction files. **Covered** in `docs/specs/`, where a test pins the value. | One written rule reversed, and a new class of prose that goes stale (inferred) | none. Q2 asks the owner |
| C-03 Docs change with the behaviour | **Covered.** No new rule. | 0 | none |
| C-09 Feature map by capability | **Covered in substance.** Do not merge specs into one page for each capability. | A merge touches 20 spec files, the surface rule of the validator, and 38 files that name a browser spec (inferred, from a count) | none. Q1 asks the owner |
| C-10 Stable behaviour id | **Covered** by the scenario number. The one open part (where a rule id for a TLA+ model lives) belongs to `statechart-models` ticket 04. | 0 here | a pointer in the `statechart-models` ticket, no new map |
| C-11 A mechanical check on the map | **Covered.** Carry one constraint to the proof-record follow-on of ticket 12. | A few lines in `validate_feature_spec.py`, inside the C-12 task (inferred) | none of its own |
| C-32 Test name carries the number, boundary triad | **Drop.** The `SCENARIO` tag already joins a test to a value. | A guardrail rule over about 1451 test names (inferred) | none |

Where to confirm: in [ticket 16](../issues/16-grilling-confirm-the-recommendations.md), as one extra round of at most 3 questions (Q3 below).

## 1. What I read

- **Known.** Tickets 08, 10, 11, 12 and 19 and their answers. Assets `research-10`, `research-12`, `research-19`, `task-11`. In `candidates.md`: rows C-01, C-02, C-03, C-09, C-10, C-11, C-32 and the G2 row. `excerpt-poteto-feature-maps.md`. `statechart-models/map.md` and its ticket 04. `code-concepts/map.md`.
- **Known.** In the repo: `docs/AGENTS.md`, `AGENTS.md`, `docs/specs/autoplay.md`, `docs/specs/browser_autoplay.md`, `tests/http/autoplay/run.rs`, `scripts/validate_feature_spec.py` (docstring and names), `.agents/skills/chronicler-ui-investigator/SKILL.md`.
- **Known.** The owner's position: an AI writes and reads the docs, no rewrite on a hypothesis, no more `pstack` research (ticket 20 context; ticket 08 answer).
- **Not read.** The full text of research 01, 04 and 05. I used their figures as `candidates.md` and the ticket 08 answer give them.

## 2. Facts that several practices use

| # | Fact | Label | Source |
|---|---|---|---|
| F1 | `AGENTS.md` is 29,112 B. The generated Structure block is 21,567 B (74%). The hand-written part is about 7.5 KB. The Hex instruction file is 100,349 B. | known (counted) | `AGENTS.md` between `AUTO-STRUCTURE` markers; research 01 summary |
| F2 | 92% of 407 code-work sessions read a doc, plan or instruction file. 56% read a Diátaxis or spec file. 175 of those 227 read before the first code edit. | known | research 10 part 2, 3.1 |
| F3 | A brief names the doc in 58% of the implementer reads (423 of 736). The other 313 reads are found by the agent. Specs: 89 of 268 reads are named. | known | research 10, 3.2 |
| F4 | Agents follow a `[DOC: …]` anchor in 14% of the cases (20% in October). | known, with limits | research 10, 3.3 |
| F5 | Of 230 read-then-edited doc pairs, about 7.7% (limits 3.5% to 20.7%) fix a statement that was wrong before. 0 of 8 stale cases were in `docs/specs/`. 6 of 8 were wrong names, places or behaviour. No agent acted on one. | known for the sample, inferred for the 230 | research 19 summary, parts 4 to 6 |
| F6 | `docs/specs/` has 31 files and 561 scenarios in 42 numbered groups. `validate_feature_spec.py` checks that each scenario has a test and each tag has a scenario. | known (counted) | `docs/specs/*.md`; `scripts/validate_feature_spec.py` docstring |
| F7 | 10 capabilities have two spec files, `X.md` and `browser_X.md`: autoplay, dashboard, games, llm_messages, options, prompt_presets, screen, settings, story_log, worlds (20 of 31 files). Swipes have three files (`swipe_new.md`, `swipe_switch.md`, `browser_swipes.md`). | known (listed) | `ls docs/specs` |
| F8 | The validator binds a spec to its observation surface. `browser_*.md` is tagged only from `tests/browser/`. Other specs are never tagged from there. | known | `scripts/validate_feature_spec.py:1-10`, `find_surface_violations` at :211 |
| F9 | Scenario 41.9 pins a budget of 0, 100 and 101, with `many`. Test `test_a_budget_that_is_zero_above_the_maximum_or_not_a_whole_number_answers_400` carries the tag `SCENARIO: 41.9`. | known | `docs/specs/autoplay.md:129-152`; `tests/http/autoplay/run.rs:458-470` |
| F10 | `browser_autoplay.md` states "default 10, from 1 to 100". Scenario 42.1 pins the 10. | known | `docs/specs/browser_autoplay.md:5`, `:13` |

## 3. The practices

### C-01: one always-loaded spec

- **Recommendation: covered. No change.**
- **Hex (known).** One file of 100,349 B holds the purpose, a hand-written module map and the invariants. Every session loads it. (F1; research 01.)
- **Chronicler (known).** The always-loaded `AGENTS.md` holds the Structure block, which `scripts/generate_structure_index.py` generates from `//!` lines, plus workflow rules (F1). `docs/AGENTS.md:23` (Where a fact lives) puts each fact in one layer, and the layers are separate files.
- **Why (inferred from F2 and F3).** A pull model already works. Agents read a domain doc in 56% of code-work sessions, mostly before the first edit. A delegate finds two of three spec reads without a brief. A Hex-style file would put the Reference content into every session, also into the 20% of sessions that research 10 could not sort, and into review sessions. The data does not show that an always-loaded invariant list would change what an agent does.
- **Cost if adopted (inferred).** About 3 times the hand-written part of `AGENTS.md` for every session, plus a second home for facts that Reference docs state. That breaks "put each fact in exactly one layer" (`docs/AGENTS.md:25`).
- **The gap that stays (known).** 315 October sessions have no `project_context` section, so I cannot say what those sessions loaded (research 10, 3.5). The gap does not change the recommendation. A larger always-loaded file would cost more, not less.
- **Written conflict.** None. `docs/AGENTS.md:23-38` already answers "where does a fact live".

### C-02: invariants with exact values

- **Recommendation: drop for Diátaxis and the instruction files. Covered in specs.** The owner decides (Q2), because two written rules oppose the practice.
- **Written conflicts (known).**
  - `docs/AGENTS.md:149` "Values stay in the code": constants and caps stay in the source file. The Hex practice puts them in prose.
  - `docs/AGENTS.md:155` "No negative explaining": state what a thing is. Hex writes 92 lines with `never`, `do not` or `must not` in its Invariants range (research 01). The practice would bring prohibitions back.
- **What Chronicler already does (known).** A spec states a value when a test pins it. Scenario 41.9 pins 0, 100 and 101 (F9). `browser_autoplay.md:5` states "default 10, from 1 to 100" and scenario 42.1 pins the 10 (F10). So the conflict is with the Diátaxis docs only, as the owner said (ticket 08 answer). The spec is the place of a value that a client sees and a test pins.
- **Why drop for Diátaxis (inferred from F5).** The 8 stale statements all sit in prose docs that copy a fact: line numbers, helper names, file counts, option names. None is in a spec, where a test pins each statement. A value stated in Reference prose has no test, so it joins the class that goes stale. Hex shows the same: 11 of 611 named checks no longer resolve (research 04, via `candidates.md` C-11).
- **What the data cannot settle.** No data shows whether an exact value helps an agent (research 10 part 7, last paragraph). The owner's wish for no rewrite on a hypothesis points the same way.
- **Cost if adopted (inferred).** The values would go to 19 Diátaxis docs, or to `AGENTS.md`. Each one would be a second home of a value that the code or a spec already holds. It would also reverse two rules in `docs/AGENTS.md`.
- **One wording mismatch (known, not a decision).** The row "Fine-grained behaviour … values, copy text" of `docs/AGENTS.md:32` says values live "in the code and its tests". Specs also hold values, because a test pins each one. Nothing breaks. If the owner wants the table exact, add a clause to the spec row: "a value that a client sees and a test pins". It is one sentence, so it fits in the next `docs/AGENTS.md` edit. I do not make it a ticket.
- **Overlap with other maps (known).** No overlap. `statechart-models` and `code-concepts` mention no practice of "exact values" (search of both maps and their tickets).

### C-03: docs change with the behaviour, in both directions

- **Recommendation: covered. Add no rule.**
- **Evidence (known).** About 79% of the read-then-edited pairs record the change of the session or of its unit (66% own change, 13% a planned doc step). 0 of 8 stale statements were acted on. 0 of 8 are in specs (F5). `docs/AGENTS.md:38` says a behaviour change edits its spec and its test, and a Diátaxis doc changes when a change adds, removes or renames a part, an invariant or a name.
- **The narrow rule that research 19 offers already exists (known).** Research 19 says that if a new rule is wanted, it covers fact copies in prose docs. Two rules cover them now. `docs/AGENTS.md:117` "Cite symbols, not line numbers" is enforced by `LINE_CITATION` and catches 2 of the 8. `docs/AGENTS.md:149` "Values stay in the code" is not enforced by a script. The 6 other stale cases (names, places, behaviour) are not in a class that a rule on timing reaches (research 19, part 7).
- **Cost seen (known).** Audit work: ticket 33, ticket 24 and two review sessions. No wrong action. A cost that no data shows: wrong statements that agents read and did not meet (research 19, part 8).
- **The gap that stays (known).** The pair list holds specs and Diátaxis only. `AGENTS.md` prose and skills are not measured. Hex deletes the rules of a removed feature in the same commit (`a8a4645d`, research 01). No Chronicler rule says the same for `AGENTS.md` prose or a skill. No data shows a stale case there. I do not ask for a rule, and I name the gap here only.
- **Written conflicts.** None.

### C-09: feature map by user capability

- **Recommendation: covered in substance. Do not merge the specs into one page for each capability.** The owner confirms the split (Q1).
- **The four missing parts of ticket 08, one by one.**

| Missing part | Decision | Reason |
|---|---|---|
| One page for each capability (Autoplay is in two specs) | **Drop the merge.** | See below. |
| Proof records | **Not here.** Ticket 12 and ticket 16 hold C-12. | Ticket 12 recommends one dated Observed paragraph inside each spec, narrow. |
| Known gaps | **Not here.** Ticket 12 and ticket 16 hold C-13 and C-14. | Ticket 12 recommends a Proof limit list and a Known gaps list inside the spec. |
| A rule for which docs exist | **Covered.** | `docs/AGENTS.md:23` (Where a fact lives) and the "Subfolder shape" section: create a folder only when a doc earns the mode. |

- **Why not merge (known unless marked).**
  - The split is by observation surface, and the validator enforces it (F8). A merged page needs a change to `find_surface_violations`, or a page whose halves follow two rules (inferred).
  - The pair is findable by name: 10 of 10 capabilities follow `X.md` and `browser_X.md` (F7). The investigator skill points a reader to `docs/specs/<feature>.md` (`SKILL.md:91`) and names `docs/specs/browser_*.md` as its selector vocabulary (`SKILL.md:101`).
  - No data shows a find problem. Every one of the 31 specs was read by at least one code-work session. A delegate finds two of three spec reads without its brief (F3). Research 10 part 7, question 4, says the data does not show that an agent needs one page for each capability. It also says a change of the unit changes what a brief and a skill name.
  - The surface rule has a reason: a browser scenario and an HTTP scenario fail in different tiers (inferred; `tests/STRATEGY.md` tiers).
- **Lauren Tan's use case (known).** A vague report arrives. The agent must find the feature, its entry point and the DOM element. She keeps that in a feature map ([08:55]–[11:23]). In Chronicler, the browser specs hold the selectors, and the investigator skill uses them (`SKILL.md:101`). What the data does not show: how often a vague report arrives. The owner is the only user (research 12, "Who observes and who reads").
- **Cost if adopted (inferred).** Merging 10 pairs touches 20 spec files. The 38 files outside `.scratch/` and `old-docs/` that name a `browser_*.md` path (`rg` count: `tests/AGENTS.md`, `tests/STRATEGY.md`, 36 test files) would change as well. The scenario numbers need not change. The validator needs a new surface rule. I judge it as 2 sessions or more, so it would leave as a brief for a new map.
- **Fallback if Q1 is "merge" (inferred).** First add one line to each `browser_X.md` that names its pair. That is 10 files, one session. The owner can then judge the merge against a measured need.
- **Written conflicts.** `CODING_STANDARDS.md:11-15` gives each source file a DOC anchor, so Reference follows the code. `docs/AGENTS.md:23` places facts by layer. Neither forbids a spec page for each capability. A capability-first Reference would conflict with the 222 anchors (known, `candidates.md` C-09).

### C-10: stable behaviour id

- **Recommendation: covered by the scenario number. Hand the one open part to `statechart-models` ticket 04. No new map.**
- **Known.** A scenario id `N.M` names one event rule. A test carries it in a `SCENARIO` tag, and the validator checks both directions (F6). 561 ids are in 42 groups. The numbering has 36 gaps (16 groups). I did not check the cause of each: a deleted scenario, or one moved to a per-surface spec (research 19 lists such a move as P2 #1). Either way, an id is not reused (inferred).
- **How it differs from Hex (known).** A Hex id names a behaviour for all docs, and no code or test reads it (research 04). A Chronicler scenario id names an event rule for the tests. `docs/AGENTS.md:40` says "Cite no spec scenario numbers in a Diátaxis doc". So Chronicler keeps the id in two layers (specs and tests) and out of the third on purpose. A Hex-style id in Diátaxis docs would reverse that rule (inferred; the reason of the rule is not written).
- **The open part.** Where the rule id of a TLA+ model lives: a scenario, a doc sentence, or a comment in the TLA+ model. `statechart-models` ticket 04 is open and lists it. The owner's leaning there is that the specs hold the event rules and `game_flow.md` need not hold rule ids (ticket 04, "The owner's leanings"). That leaning fits this recommendation.
- **Cost here (inferred).** 0. A one-line note in `statechart-models` ticket 04 can say that the hex map found the scenario id to be the stable id.
- **Overlap (known).** `statechart-models` only. `code-concepts` names no id practice.

### C-11: a mechanical check on the map

- **Recommendation: covered. Carry one constraint.**
- **Known.** `spec-coverage` runs `validate_feature_spec.py`: 561 declared, 561 covered (research 12, check table). Hex has no such check, and its map drifts (research 04).
- **The limit (known, research 19 part 6 and 7).** A check on ids, links and tags catches 0 of the 8 stale cases. The `LINE_CITATION` rule catches 2 of 8 today. The other 6 need a comparison of a statement with the code. Neither repo has that check.
- **The constraint.** Any new doc shape gets a check on its ids and links, and the check protects the link, not the truth of the text. It applies once, now: the proof record of ticket 12. Research 12 names the check: each "Scenarios N.M" in the record must resolve to a declared scenario, a few lines in `validate_feature_spec.py` (inferred; not built). It belongs in the follow-on of C-12, not in a ticket of its own.
- **Cost (inferred).** Inside the C-12 task: 0 extra sessions if built with it.
- **Written conflicts.** None.

### C-32: test name carries the number, with a boundary triad

- **Recommendation: drop.** It follows C-02, which is dropped for Diátaxis.
- **Hex (known).** A test name carries the invariant's number, and a loop runs `[299, 300, 301]` (research 05; `candidates.md` C-32). The join between the invariant and the test is the number alone: "No ID links the invariant to the test" (research 05, part "How an invariant from `AGENTS.md` shows in a test").
- **Chronicler (known).** The join is stronger. The `SCENARIO` tag links a test to a numbered scenario, and a validator checks it (F6). Scenario 41.9 is a boundary check. It pins 0, 100, 101 and two non-numbers, and the test loops over the refusals (F9). At the unit tier, `tests/STRATEGY.md:250-251` says a test name describes the behaviour. A test name does not carry "100".
- **Why drop (inferred).** The number in a name is Hex's substitute for a link that Chronicler has. A rule that puts it in about 1451 test names would be a guardrail with no measured gain.
- **Cost if adopted (inferred).** A guardrail rule and a rename of the names that hold a value. No session estimate, because I do not recommend it.
- **Written conflicts.** None found.

## 4. Research 10 part 7, questions 1, 2 and 4

| Question | Answer from the data | What the data cannot settle |
|---|---|---|
| 1. A route that agents find alone, or one that briefs carry? | Both work already. 56% of code-work sessions read a domain doc. A brief names 58% of a delegate's reads. The anchor is followed in 14% (F2 to F4). The aim is not a choice. Change nothing. | Whether a session that did not read a doc would have done better. A read is not use (research 10, part 8). |
| 2. Do agents need the Explanation docs? | 12% of code-work sessions read one. 8 Explanation files were read at all. They are read 3.4 times less than Reference. | Whether the owner reads them. This question is outside the 7 practices (it concerns the Diátaxis set). I did not turn it into a question. The coordinator can pass it to a doc-set review (`diataxis-doc-review` skill exists). |
| 4. Is a spec the right unit for a capability page? | Specs are the most read doc type, and agents find them by file name (F2, F3, F7). The unit is a spec, joined to its browser pair by name. See C-09. | Whether one page for each capability would help an agent more. |

Question 3 of research 10 is answered by ticket 19 (F5, C-03, C-11).

## 5. Questions for the owner

At most 3. Each has my recommendation.

1. **C-09.** Autoplay and nine other capabilities stay in two specs each, `X.md` and `browser_X.md`, joined by name and by the surface rule of the validator. Do you confirm the split? Recommendation: yes. The data shows no find problem, and a merge costs 2 sessions or more. If you say no, the fallback is a one-line pointer in each `browser_X.md` first.
2. **C-02.** Do you confirm that `docs/diataxis/` keeps "Values stay in the code" and "No negative explaining", and that a value that a client sees lives in a spec scenario with a test? Recommendation: yes. Both rules are written, and your wish for no rewrite on a hypothesis agrees with them.
3. **Where to confirm.** Do you confirm the doc practices in ticket 16, as one extra round? Recommendation: yes. The round holds Q1 and Q2 and no build. A round of its own would add a session for two answers.

## 6. What I did not do or check

- **Not checked.** The cause of the 36 gaps in the scenario numbering.
- **Not checked.** Whether the 1451 test names ("test_" count) come from today's tree. The figure is from `candidates.md` K-37.
- **Not checked.** The count of 38 files that name a browser spec is a text match on `browser_<name>.md`. A match can be a generic mention.
- **Not checked.** The reason behind "Cite no spec scenario numbers in a Diátaxis doc" (`docs/AGENTS.md:40`).
- **Not measured.** How often a vague user report reaches an agent. This is the use case of Lauren Tan's feature map.
- **Not measured.** Whether an exact value in prose changes what an agent does. No data exists in the assets.
- **Limit.** One reader made these judgements. The recommendations rest on research 10 and 19, whose own limits apply: a read is not use, and only edited docs are in the pair data.
- **Line numbers.** `docs/AGENTS.md` line numbers (23, 25, 32, 38, 40, 117, 149, 155) are from the working tree of 2026-11-01. They move with the next edit, and the generated index block can shift them.
