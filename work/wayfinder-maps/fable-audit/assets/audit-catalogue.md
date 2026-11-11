# Catalogue of audits for Chronicler Engine

Date: 2026-11-10. Commit: `b0c8fe41`. Author: Claude (Fable 5.2).

This document lists the audits that I judge to be of high value for this repository. One of them is done. The others are proposals.

## How to read this catalogue

- **Value** and **cost** are my judgement. I measured neither. Cost S means less than one hour of my work, M means one to three hours, and L means more than that or a need for model credit.
- **Why here** gives the evidence from this repository. An audit without such evidence is not in the catalogue.
- **Proof** says how a result of the audit becomes more than a suspicion. The rule is: find, then prove, then repair. An audit of this catalogue stops after the proof.
- **Overlap** names a record of yours that covers a part of the audit. I read those records only at their first page, unless the text says more.
- Section 7 holds the questions for you. Section 8 holds my recommended order.

## 1. Documents in this folder

| File | What it is | Catalogue audits |
|---|---|---|
| `questions-for-the-owner.md` | Every question of every report, once, with my recommendation. **Start here** | — |
| `engine-defects-and-risks.md` | The first audit: 14 findings, 12 questions. Its name was `REPORT.md` | — |
| `engine-defects-and-risks-notes.md` | The method, the probe commands and the dropped candidates of that audit. Its name was `NOTES.md` | — |
| `story-damage-and-false-screen.md` | Ten findings that I ran on the probe server, 7 questions | A2, A3, A5, B1 |
| `code-lenses.md` | Four design choices behind the findings, 5 questions | B2 to B6 |
| `product-content-and-release.md` | The content as writing, the trial tool, the genre, the release, 6 questions | C2 to C7 |
| `development-environment.md` | Rules, skills, guardrails, a mutation run, the flow of work | D1 to D3 |
| `audit-catalogue.md` | This document | — |
| `leads/` | The tables of the delegates. A row there is a place to look, not a finding | — |
| `local-only/evidence/` | The scripts, logs, screenshots and counts. Git ignores this folder | — |

**Status on 2026-11-10.** Done: A2, A3, A5, B1 to B6, C2 to C7, D1 to D3. Not done: A1 (real play) and A4 (a second model) need you or credit. C1 (the value of each mechanism) needs credit.

A new audit gets its own report file with a name that says what it covers. It does not go into an existing report.

Note: these documents were in `tmp/fable-audit/` until 2026-11-10. They moved here by decision D5 of `../decisions.md`, because `build.py` deletes each entry under `tmp/` that is older than 30 days. The folder `local-only/` is not in git.

## 2. Overview

| ID | Audit | The question | Value | Cost |
|---|---|---|---|---|
| — | **Defects and risks of the running engine** | Does the engine do what it means to do, safely? | Done | Done |
| A1 | Real play | Do the story findings also occur when a person types the Inputs, with a real provider? | High | L |
| A2 | Damage to a story | Can a crash, an edit, a delete, a Retry or a double request leave a Game wrong or stuck? | High | M |
| A3 | Hostile text in a prompt | Can an Input, a Character card or a World text give the model a false scene or a false instruction? | Medium | S |
| A4 | A second model | Which results change with a local or a weaker model? | Medium | M |
| A5 | Triggers under load | Do Triggers work in a World that has many of them? | Medium | M |
| B1 | The screen shows something false | Where does the dashboard say something that is not true? | High | M |
| B2 | More than one source of truth | Which data exists twice and can differ? | High | M |
| B3 | Implicit protocols | Which two parts agree on a shape that no code checks? | Medium | M |
| B4 | Silent failures | Which failure does the engine hide from the player and the tuner? | Medium | S |
| B5 | One rule, several entry points | Which rule differs between the paths that reach it? | Medium | M |
| B6 | The pattern behind the 14 findings | Which design choice made each fault easy? | High | S |
| C1 | Does each mechanism earn its cost? | Is the story better with the Quantifier, the Rooms and the Characters present than without them? | High | L |
| C2 | What can be deleted? | Which concept has no user today? | Medium | M |
| C3 | The measuring tool | Does a number of the trial pack move when the story gets worse? | High | M |
| C4 | Comparison with the genre | What do players of this kind of product expect that is missing? | Medium | M |
| C5 | The content as writing | Are the cards, the opening and the prompt presets good direction for a model? | Medium | L |
| C6 | Readiness for the release | What blocks a public release, a phone layout or accounts? | Medium | M |
| C7 | A cold start by a stranger | Can a person with only `README.md` install, play and make a World? | Medium | M |
| D1 | Rules, skills and guardrails | What enforces each rule that an agent gets? | High | L |
| D2 | Independence of the tests | Do the tests fail when the behaviour is wrong? | Medium | M |
| D3 | The flow of work | Where do the hours, the tokens and the open tickets go? | Medium | M |

## 3. Group A: continue the finished audit

These audits ask the same question as the finished audit, in places that it did not reach.

### A1. Real play

- **Why here.** Every story that I measured is an Autoplay run with one model. Findings 11, 12 and 14 of the finished audit can be weaker when a person plays. Every probe call went to a fake endpoint, so no real failure shape was tested: a wrong key, no credit, a rate limit, a reasoning model that uses the whole `max_tokens`.
- **Method.** One session of 30 to 50 Actions with typed Inputs on a real Connection. Then the counts of `local-only/evidence/trial_story_stats.py` on that story.
- **Proof.** The same counts as for the three trials, side by side.
- **Needs.** Model credit, and a person who plays. Section 7, questions 1 and 2.

### A2. Damage to a story

- **Why here.** The route `/status/reset-busy` exists, so a stuck Game has occurred. I tested Ctrl-C only. I read `GameWriter::edit` and saw that an edit of an old entry keeps the Swipe state of that entry. I ran no edit, delete, Retry or Swipe change on an old entry.
- **Method.** On the probe server with the fake model: `kill -9` in each step of a Job, then a restart. An edit, a delete and a Retry in the middle of a story. The same form post twice at once. After each step, read the Swipe states and the busy state.
- **Proof.** A request sequence that leaves a Game wrong, with the stored rows.
- **Lenses.** "Find every workflow that can become permanently stuck." "Find every operation that is unsafe to retry." "Find every successful operation that can leave the system inconsistent."
- **Overlap.** The formal models of the gate (`verify-gate`) cover the busy state in a model. This audit hits the binary.

### A3. Hostile text in a prompt

- **Why here.** The prompts use blocks such as `<LatestMessage>` and `<PlayerInput>`. I checked where model text becomes HTML. I did not check whether a text can close a block. `sanitize_for_prompt` no longer reaches the typed Input (your retro, R16). A Character card from another person is a planned use.
- **Method.** Inputs and card texts that close a block or imitate one. Read the Quantifier answer and the next Narration on a real model.
- **Proof.** One Input that changes the recorded Room or the Characters present against the story.
- **Needs.** A real model for the last step. The fake endpoint can show only the prompt.

### A4. A second model

- **Why here.** `data/settings.json` ships an Ollama Connection. The engine assumes 8,192 context tokens for Ollama and does not read the `usage` of an answer, so a prompt that the model cuts would be invisible (inferred, not tested). The token estimate and the Quantifier JSON were measured for one model only.
- **Method.** A short trial on a local model and on one weaker hosted model. Compare the token estimate, the parse results and the `single_user_message` path.
- **Needs.** Ollama in the container, or credit.

### A5. Triggers under load

- **Why here.** The shipped World has one Trigger, so the trials say almost nothing about Triggers. The earlier audit found that the Trigger order follows a `HashMap` (world-04, open). The event prompt has three faults (finding 6). `times_met` grows from noise (finding 11).
- **Method.** A test World with several Triggers: thresholds on `times_met`, `repeat`, a Room condition, two Triggers in one step. One Autoplay run on the fake model for the order, and one on a real model for the prompt.
- **Proof.** A Trigger that fires at the wrong time, or two runs with a different order.

## 4. Group B: one lens over the whole code

Each of these audits takes one question and applies it to the whole code. The questions come from the prompt collection `Drix10/awesome-prompts`. Two of its lenses are already used here and are not in this group: the implicit state machines (`work/wayfinder-maps/state-machines-next/`) and the invariants that are not enforced (`work/wayfinder-maps/invariant-audit/`).

### B1. The screen shows something false

- **Why here.** `PRODUCT.md` says that success means the player stays inside the story. I found four cases by accident: **Ready** on a cut Narration, the picture of a wrong Room, "could not be reached" for an HTTP 500, and a new Narration below the fold.
- **Method.** List each thing that the dashboard states: the status, the Room, the Characters present, the role health, the failure banner, the busy controls, the Options, the saved Games. For each one, find the data behind it and the moment when the two can differ.
- **Proof.** A screenshot from the probe server with the stored row next to it.

### B2. More than one source of truth

- **Why here.** The engine copies much data at creation time. I saw: whole Character cards inside each Swipe state, the World settings copied into a Game, a data file against its database row, and the default host in both `cli.rs` and `build.py`.
- **Method.** For each kind of record, list where it is written, where it is read, and each copy. Mark each copy as a snapshot on purpose or as a copy that can drift.
- **Proof.** A sequence that makes two copies differ, and the place where the difference shows.
- **Overlap.** Your catalogue of concepts has E3 (boot seeding). Ticket 07 of the long-term-memory map plans a card change of one Game, which depends on this.

### B3. Implicit protocols

- **Why here.** A prompt preset is text that the author can edit, and a parser in code expects its output shape. The Quantifier preset holds the JSON shape that `parser.rs` reads. The Options preset has two shapes and the parser has three regex strategies. The page scripts and the server agree on the `X-Revision` stamp. `scripts/trial.py` reads the database tables directly.
- **Method.** List each pair of parts that must agree, and what happens when one side changes. The first test is cheap: edit the output format of a Quantifier preset in the dashboard.
- **Proof.** One edit on one side that breaks the other side with no message.

### B4. Silent failures

- **Why here.** The engine prefers to continue a story. A failed Quantifier keeps the old state. `after_narration` drops an Agent error. A bad World file at boot is a log warning. A failed save of a failed LLM message is a warning.
- **Method.** Search each place that turns an error into a default, a warning or nothing. For each one, say who would want to know: the player, the tuner or nobody.
- **Proof.** For each "player" or "tuner" case, the probe run where nothing shows.

### B5. One rule, several entry points

- **Why here.** A typed Input gets no analysis, and an Impersonate Input gets one (finding 12). The Connection form offers the `SELECTABLE` providers and parses `ALL`. A World from a data folder gets Characters, and a World from the dashboard gets none.
- **Method.** List each rule of `CONTEXT.md` and the specs that more than one path reaches: a typed Action, Impersonate, Guide, Retry, Autoplay, the boot seed, the forms. Compare the paths.
- **Proof.** The same input through two paths with two results.

### B6. The pattern behind the 14 findings

- **Why here.** A fix for each finding removes 14 faults. A fix for the pattern removes the next ones too. My guess at two patterns: the engine stores what a model derived as a fact, and a failure prefers silence to a notice.
- **Method.** No new reading. For each finding, name the design choice that made the fault easy, then group the findings.
- **Proof.** Each pattern must explain at least three findings and predict one fault that I can then find.

## 5. Group C: is this the right engine?

The finished audit asked whether the engine does things right. These audits ask whether it does the right things.

### C1. Does each mechanism earn its cost?

- **Why here.** The recorded Room is wrong for about 70 of 202 entries in one trial, and the wrong state then moves the story (finding 11). The Quantifier is half of all calls. 41 to 44 of about 101 Narrations name no Character of the World. If the story is as good without the Room state, findings 9, 11 and 12 become a deletion and not three fixes.
- **Method.** The same trial with one mechanism off for each run: no Quantifier, no Room in `<GameState>`, no analysis after an Impersonate Input. A judge model reads the stories blind, and the counts of `trial_story_stats.py` run on each.
- **Proof.** A table of the runs. A mechanism earns its cost when its run is better on a count that was named before the run.
- **Needs.** Credit for three to four trials of 100 Cycles, and a build with a switch for each mechanism. Section 7, question 1.
- **This audit can change the direction of the product.** No other audit in this catalogue can.

### C2. What can be deleted?

- **Why here.** The code has concepts with no content today: items and inventories (empty in the shipped World), Triggers (one), the text check (off in the shipped settings), `sanitize_for_prompt`, the `mock` provider in the form parser. The source has 21,582 lines without tests, and the tests have 53,425 lines.
- **Method.** One row for each concept of `CONTEXT.md`: its shipped content, its callers, its lines of code and tests, and what a player loses without it.
- **Overlap.** Your catalogue of concepts (`work/wayfinder-maps/code-concepts/`) describes the concepts. It does not ask which of them to remove.

### C3. The measuring tool

- **Why here.** The trial pack counts tokens, time and cache hits. Era drift, wrong Rooms and Characters that the World does not have were visible only after I wrote my own counts. A prompt change is today judged by reading.
- **Method.** List the faults that the three stored trials contain. For each one, check whether a number in `summary.json` or `pack.md` shows it. Propose the missing counts.
- **Proof.** Two trials that differ in story quality and not in the pack, or the other way round.
- **Overlap.** `work/wayfinder-maps/layout-trials/` sorted story faults by hand (ticket 13). This audit asks which of them a script can count.

### C4. Comparison with the genre

- **Why here.** `PRODUCT.md` names a public release as the goal. Similar engines have features that I did not see here: a lorebook, a summary of old story, branching, an export of a story, an author's note.
- **Method.** Research of three or four similar engines from their own documents. Then a gap table against `PRODUCT.md` and the open maps.
- **Needs.** Web research. Section 7, question 4.
- **Overlap.** The long-term-memory map covers memory and the story clock.

### C5. The content as writing

- **Why here.** I checked that the prompts are wired correctly. I did not judge the words. The Impersonate preset forbids narration of the world, and the model does it in every Cycle. The World text says both "present day" and "colonial era" (finding 14).
- **Method.** An editorial read of the five Character cards, the opening and the four default prompt presets. Then a small trial for each rewrite.
- **Needs.** Credit for the trials. The taste is yours, so the result is a set of options and not a ruling.

### C6. Readiness for the release

- **Why here.** `PRODUCT.md` names a public release as the goal and a phone layout as wanted. It leaves two decisions open: how much of the tuning surface a released player sees, and accounts or a local install. The design has parts that fix an answer already: one active Game for the whole server process, the database next to the binary, no login, the `testing` feature in every build.
- **Method.** For the release, the phone layout and each answer to the two open decisions, list what must change. Include the release artefact (a build without `testing`, the packaged folder), the licences of the dependencies and the pictures, and what story text goes to which provider.
- **Proof.** The packaged folder, started as a user would start it.

### C7. A cold start by a stranger

- **Why here.** `README.md` has 202 words. A World that the dashboard creates gets no Character (finding 3). A server that starts from another directory shows an empty page with no message (finding 8).
- **Method.** A fresh agent gets the repository and one goal: install, play ten Actions, then make a new World with two Characters. It gets no other help. I record where it fails and what it had to guess.
- **Proof.** The session record of that agent.

## 6. Group D: the development environment

The repository has a large environment for agents. Known numbers: the root documents hold about 15,400 words, the repository has 41 skills (this session offered 19 of them), and `build.py` with `scripts/*.py` holds 10,253 lines of Python.

Your records already cover much of this group. `work/wayfinder-maps/retro-issue-bucket/audit.md` sorts each proposed fix into five classes (mechanical, wrong prose, output request, vigilance rule, model slip) and says which class works. The weekly retro of 2026-11-04 has 24 candidates. The retro bucket has 26 tickets, and 25 are open. So new candidates are not what this group lacks. An audit here must give something that those records do not have.

### D1. Rules, skills and guardrails

- **Why here.** No record lists every rule with what enforces it. I do not know why 22 skills are on disk and not offered. Three skills have nearly the same name (`grill-me`, `grill-with-docs`, `grilling`), and three are after-plan workflows.
- **Method.** One row for each rule in `AGENTS.md`, `CODING_STANDARDS.md`, `.pi/APPEND_SYSTEM.md` and the skills. Columns: what enforces it (the code, a guardrail test or a lint, the permission config, only the sentence), whether it ever failed a gate (`logs/build_history.txt`), and whether another rule contradicts it. The same for each skill: who calls it, what it overlaps, which path in it is stale. Then a test of each rule of the permission config.
- **Proof.** For a rule with a check, the gate run where it failed. For a rule without one, a past session where an agent broke it.
- **Lenses.** "I wonder how many AGENTS.md rules I could turn into tests." "A prompt is not a guardrail." "Make it impossible in the code, then a lint rule, then a skill, then human review."
- **Overlap.** High, see above. The new part is the complete list. Read `retro-issue-bucket/audit.md` in full first, and use its five classes.

### D2. Independence of the tests

- **Why here.** Line coverage is 95.2 %. Coverage says that a line ran, not that a test would fail on a wrong result. The same kind of agent wrote the code and the tests. `response.rs`, the parser of the model answer, has the lowest coverage (69 %).
- **Method.** Mutation tests with `cargo-mutants` on the game rules, the parsers and the gate. Property tests or fuzzing on `response.rs` and the Quantifier parser.
- **Proof.** A changed rule that no test catches.
- **Needs.** An install of `cargo-mutants`, and gate time: each mutant is one test run.

### D3. The flow of work

- **Why here.** About 205 of the last 379 commits touched no product file. All 16 tickets of the earlier audit are open, and four of them are one-line fixes. A good full gate takes 319 s at the median, and `AGENTS.md` says about 145 s.
- **Method.** The age and the end of each ticket in `.scratch/`. The step times in `logs/build_history.txt`. A read of past sessions for lost turns and repeated errors (`session_search`).
- **Overlap.** `work/wayfinder-maps/metacognitive-audit/` (your habits in the sessions of August) and the weekly retro. This audit asks about the queue, not about habits.

## 7. Questions for the owner

| # | Question | Why it matters | My recommendation |
|---|---|---|---|
| 1 | May an audit spend model credit, and how much? | A1, A3, A4, A5, C1 and C5 need real calls. One trial of 100 Cycles is about 3 million input tokens, 60 % to 80 % of them cached | Yes, with a limit that you name. C1 needs three to four trials |
| 2 | For A1, do you play the session, or do I write the typed Inputs? | Inputs that I write are again the Inputs of a model | You play 30 Actions. I read the result |
| 3 | Is D1 wanted while 25 tickets of the retro bucket are open? | A new list of candidates would add to a queue that does not move | Only the complete list of rules and what enforces each. No new tickets |
| 4 | May C4 use web research? | It reads the documents of other engines | Yes |
| 5 | Where must audit reports live? | `build.py` deletes entries under `tmp/` after 30 days | A folder under `.scratch/` for a report that a ticket cites. `tmp/` for the evidence |
| 6 | Do the findings of a new audit go into `.scratch/` as tickets? | The finished audit made none. Its findings are only in this folder | You decide for each report. I file nothing without your word |

## 8. Recommended order

1. **B6**, the pattern behind the 14 findings. It costs almost nothing, and its result sharpens every later audit.
2. **A2** and **B1** together on the probe server. Both need no credit, and each result can be proved. They cover the two remaining ways to lose a player: a damaged story and a screen that is wrong.
3. **C1** with **C3**. This is the audit that can change the direction. It needs credit and a switch for each mechanism, so it needs your answer to question 1 first.
4. **D1**, as the complete list only.
5. **A1**, when you have time to play.

The other audits are worth doing when their subject comes up: A5 before an author adds Triggers, C6 and C7 before a release, D2 before a large refactor.

## 9. Audits that I do not recommend now

- **Implicit state machines** and **invariants that are not enforced.** Both are done, and their tickets are open.
- **A docs audit.** Two skills exist for it (`chronicler-docs-hygiene`, `diataxis-doc-review`), and `validate_docs.py` runs in the gate.
- **Performance.** A story of 2,000 Messages costs 483 ms for each Cycle on the probe. A model call takes seconds.
- **Accessibility and visual design.** The dashboard-design map has open tickets for both. An audit before those decisions would judge a design that is about to change.

## After the audit

The owner answered the questions on 2026-11-10. The answers are in [the decisions](../decisions.md), and the accepted work is five tickets that [the map](../map.md) lists.
