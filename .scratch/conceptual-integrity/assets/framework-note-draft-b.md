# Framework note, Form B: by the owner's questions

**ROUGH DRAFT for the owner to react to.** This is one of two forms. The owner chooses; the coordinator then makes `framework-note-draft.md`. This form is a framework. It is not a rule list and not a checklist. It does not close any question.

Labels for my own words: **known** (a source or a file says it), **inferred** (I read it from known facts), **guessed** (weak signal).
Citations point to the research assets by section, for example `research-02 §Z39.19 Q5`.
Short names: Brooks, Jackson, Evans, Naur, Alexander, Schelling, Courts and ALI (precedent), Z39.19 and ISO (controlled vocabulary), Nygard and P&C (decision records: Nygard, Parnas and Clements).

## Summary

1. Seven questions of the owner, each with the models, the split, the repo evidence and a sharper question.
2. Q1 (what gets an entry): the models give six different tests. They pull the owner's tension both ways. Part of the pull comes from how the reader meets the entry.
3. Q2 (what aligns writers): precedent, a shared language and a prescribed list all work in some model. Which one works for a writer with no memory is untested.
4. Q3 (a gap): most models allow a choice, and add a probation state, a record and a decider. None gives these to a writer with no memory.
5. Q4 (the opposite): follow, distinguish and overrule are three acts. The models give them to different writers.
6. Q5 (staying current): every model but Naur has a way to change an entry. Removal is the least covered.
7. Q6 (forces): they live inside the entry (Alexander, Jackson, Z39.19) or beside it (Nygard). The repo has no one place.
8. Q7 (one context or several): this is where Evans matters most for the work team.
9. The repo evidence is kept apart. It cannot show that an entry causes alignment.
10. "Concept" has three senses (see the box after the summary). No source tests a writer with no memory.

## Three senses of "concept" (the map leaves the word open)

- **A word for one thing** (Z39.19 §5.3, ISO 25964 §2.11, Evans's model names).
- **A unit with a purpose** (Jackson, `research-01 §Jackson Q1`).
- **A solution to forces in a context** (Alexander, `research-01 §Alexander Q1`).
- Inferred: the first fits a glossary. The second and third fit a catalogue of ways to do things. The line is not clean.

## Writers, in one table

| Model | Remember | Talk | Shared prior | Source |
|---|---|---|---|---|
| Brooks | yes | yes | built by meetings | `research-01 §Brooks, assumes` |
| Jackson | not stated | not stated (inferred: apart) | large, of familiar concepts | `§Jackson, assumes` |
| Evans | partly | yes, speech is central | built in the team | `§Evans, assumes` |
| Naur | yes (the point) | yes, close contact | partial | `§Naur, assumes` |
| Alexander | yes | yes, on site | the language itself | `§Alexander, assumes` |
| Schelling | shared history | no (tacit case) | yes, of "who the people are" | `research-02 §Schelling, assumes` |
| Courts, ALI | read the record | through the record | a trained profession | `§US Supreme Court, assumes` |
| Z39.19, ISO | need not | to an editor | partly, the domain | `§Z39.19, assumes` |
| Nygard, P&C | no | cannot ask the author | developers on one project | `§Nygard, assumes` |

- Known: no row is a writer with no memory and a shared prior.
- Inferred: the rows with "no memory" (Z39.19, Nygard, P&C) are the nearest. None tested agents.
- Repo: agent sessions, no memory, no talk, a shared prior of the model. The owner remembers and talks (`research-03 §Who wrote`).
- Work team: seven people who remember and talk, with other teams who share only Jira, Slack, GitHub and Confluence. Guessed: Slack lets writers talk; Confluence is a vocabulary that can go stale; Jira and GitHub hold decisions and precedent (`charting-notes §After the map`).

## Q1. What gets an entry?

**What each model says**
- Brooks: what the user sees. A name costs learning time (`research-01 §Brooks Q1`).
- Jackson: what has a purpose, one to one (`§Jackson Q1`).
- Evans: what the selected model needs. Keep the core small (`§Evans Q1`).
- Alexander: nameable, drawable, and a real conflict of forces. Not too many under one entry (`§Alexander Q1`).
- Naur: no test. A person who holds the whole judges (`§Naur Q1`).
- Schelling: no rule. A thing can coordinate if it is unique and prominent. Clutter drowns the signal (`research-02 §Schelling Q1`).
- Courts: the question actually decided. The top court takes few (`§US Supreme Court Q1`).
- Z39.19: each distinct concept, one form, on warrant. Fringe detail stays out (`§Z39.19 Q1`).
- Nygard: significant decisions only. P&C: all rejected alternatives (`§Nygard Q1`, `§Parnas Q1`).

**Where they split.** The owner's tension is here.
- Toward "every distinct concept": Z39.19, Jackson, Evans.
- Toward "keep it small": Schelling, Brooks, Nygard, Alexander, Z39.19 §11.1.7.
- Inferred: the split follows the reader's act. A reader who looks an entry up can bear a large list (Z39.19). A reader who must hold the entries in mind cannot (Brooks, Schelling, Nygard).
- Inferred: Jackson and Z39.19 test the concept. Nygard and P&C test the decision. Courts and ALI test both (`research-02 §Questions raised 7`).

**Repo evidence (apart).** Known: `CONTEXT.md` is met by a pointer. `AGENTS.md` is loaded, and 305 of its 408 lines are the generated index (`research-04 §Table`, rows 1 and 2a). Entries are met in different ways, so one size limit may not fit them. Known: the glossary skill and the live case `Save unit` give two tests that disagree (`charting-notes §The Save unit case`).

**Sharper question.** For each place where an entry could live, how does the reader meet it: loaded, followed by a pointer, or looked up? Does the answer change what "too big" means?

## Q2. What makes writers align?

**What each model says**
- Brooks: few minds, a manual, a log, multiple implementations, a product test (`research-01 §Brooks Q2`).
- Jackson: familiar concepts, and concept independence (`§Jackson Q2`).
- Evans: one language, used in speech, writing and code. Developers who touch the model. Continuous integration (`§Evans Q2`).
- Naur: shared theory, passed by contact (`§Naur Q2`).
- Alexander: a language that every builder holds, tested by use (`§Alexander Q2`).
- Schelling: a focal point. Precedent has force beyond its logic (`research-02 §Schelling Q2`).
- Courts: precedent, for predictability (`§US Supreme Court Q2`).
- Z39.19, ISO: a prescribed preferred term, because language gives synonyms and homographs (`§Z39.19 Q2`).
- Nygard: a visible motive, so the reader does not accept or change blindly (`§Nygard Q2`).

**Where they split.**
- Prominence or prescription: Schelling against Z39.19 (`research-02 §Where the sources disagree 3`).
- Control or growth: Brooks wants control of the concepts. Alexander says control breaks the language (`research-01 §Where the sources disagree 2`).
- Naur says the deep alignment cannot be written. The others use written tools.

**Assumes about writers.** The talk models (Brooks, Evans, Naur, Alexander) assume talk. Schelling, Z39.19 and Nygard do not. Inferred: only the second group applies to agents.

**Repo evidence (apart).**
- Known: avoided words fell in 5 terms and stayed in 6 (`research-03 §Summary`).
- Known: each fall followed a plan or a deletion. Six names with no entry fell the same way (`research-04 §Which other ways`).
- Known: precedent in the code kept its own word: `NpcCard`, `Storage::transaction` (`research-04 §9`).
- Known: nothing shows that a writer read an entry.

**Sharper question.** Is an entry a cause of alignment, or a record of a plan that aligns? Schelling's precedent and the repo's plan fit the same counts. What would a case without a plan show?

## Q3. What happens at a gap?

**What each model says**
- Brooks: ask the architect. The answer is "ex cathedra" and goes to everyone in the log (`research-01 §Brooks Q3`).
- Jackson: find a familiar concept first. If none fits, invent one for a purpose (`§Jackson Q3`).
- Evans: use shows the gap. A new word is a model change. Developers and experts name it in talk (`§Evans Q3`).
- Naur: the theory holder answers from the similarity to what exists. No rule reaches it (`§Naur Q3`).
- Alexander: invent a pattern. Name it. Make it clear enough to be wrong, then test it in public (`§Alexander Q3`).
- Schelling: any key that both recognise becomes the key. The framer has power (`research-02 §Schelling Q3`).
- Courts: lower courts decide first. The top court may wait ("percolation") (`§US Supreme Court Q3`).
- ALI: the choice is a judgment. Reporters draft; reviewers decide (`§ALI Q3`).
- Z39.19: the user nominates a candidate. An editor and a board decide. A candidate is marked and often hidden (`§Z39.19 Q3`).
- Nygard: the new decision is "proposed" until stakeholders agree (`§Nygard Q3`).

**Where they split.** Known: every model except Alexander, Evans and Schelling names a decider. Alexander and Evans give the act to the group. Schelling gives it to whoever is conspicuous first. Inferred: in no model is the first writer's choice final, except in Schelling's, where it becomes final by being copied.

**Assumes about writers.** The decider needs a channel. Brooks: the telephone. Z39.19: a nomination form. Courts: the record. Evans: talk.

**Repo evidence (apart).** Known: `docs/agents/domain.md:28` says a missing term is "a signal … reconsider … or a real gap". `AGENTS.md:334` says to hold a reasoned position. No way in the table tells a session with no skill what to do at a vocabulary gap (`research-04 §Questions raised 7`). Known: entries followed code in Swipe state, Save unit and Error status (`research-03 §Per term`).

**Sharper question.** When an agent chooses at a gap, who sees the choice, and what state does it have until someone does? Does a writer with no memory ever read the probation items (`research-02 §Questions raised 1`)?

## Q4. When may a writer choose the opposite of an entry?

**What each model says**
- Brooks: no. Leave the idea out or scrap the system. No rule for a deliberate exception (`research-01 §Brooks Q4`).
- Jackson: the criteria are "neither necessary nor sufficient". No test for an exception (`§Jackson Q4`).
- Evans: not inside a context. Across contexts it is a stated choice, up to "Separate Ways" (`§Evans Q4`).
- Naur: a change is good if grounded in the theory. No rule for an exception (`§Naur Q4`).
- Alexander: follow the spirit, not the letter. The strength of the entry sets the freedom (`§Alexander Q4`).
- Schelling: without talk, a non-focal choice is "not really available" (`research-02 §Schelling Q4`).
- Courts: any court may distinguish. Only the named body may overrule, on four tests (`§US Supreme Court Q4`).
- ALI: depart "explicitly and explain why" (`§ALI Q4`).
- Z39.19: use a non-preferred term and be redirected. Listed exceptions need warrant. A change needs a proposal with a rationale (`§Z39.19 Q4`).
- Nygard: reverse and supersede. The aim is to avoid a blind change (`§Nygard Q4`).

**Where they split.** Settled or right: "more important that the rule be settled" (Payne) against "too rigid adherence … may lead to injustice" (Practice Statement) (`research-02 §Where the sources disagree 2`). Inferred: the models differ on the strength of an entry. Only Alexander marks it, with asterisks.

**Assumes about writers.** Overruling needs a body. Distinguishing needs only the record and the facts. Inferred: a writer that cannot ask can distinguish. It cannot overrule.

**Repo evidence (apart).** Known: two entries changed to fit code (Retry reversed; Steering carve-out). In the Steering commit, a writer chose a name from an outside source and then edited the glossary (`research-03 §Who wrote`). Known: the repo has no marker for how strong an entry is (`research-04 §Questions raised 8`). Known: the ADR format lists "deliberate deviations from the obvious path" (`research-02 §domain-modeling Q4`).

**Sharper question.** Which of the three acts (follow, distinguish, overrule) does an agent session have? If it may distinguish, what shows the next writer that the act was on purpose?

## Q5. How does a vocabulary stay current?

**What each model says**
- Brooks: the manual changes in a cycle with feedback. One standard text (`research-01 §Brooks Q5`).
- Jackson: inventory, then drop, merge or split. No team routine (`§Jackson Q5`).
- Evans: language and code change together. A document that falls behind should be archived (`§Evans Q5`).
- Naur: the theory is the thing that must stay current. Text will lag (`§Naur Q5`).
- Alexander: entries are hypotheses. He discarded most of what he wrote (`§Alexander Q5`).
- Courts: a new opinion overrules by name. The old text is not edited (`research-02 §US Supreme Court Q5`).
- Z39.19: dated history notes, USE references, an optional file of deleted terms. The cost of a change falls on earlier records (`§Z39.19 Q5`).
- Nygard: supersede. Never edit. Never reuse a number (`§Nygard Q5`).

**Where they split.** What of the past stays visible: Nygard and Z39.19 keep it. Z39.19 also allows deleting an entry made in error. P&C record rejected alternatives but give the reader a cleaned path (`research-02 §Where the sources disagree 5`). Inferred: Brooks says nothing on removal.

**Assumes about writers.** A history note helps a reader who was not there. Alexander's removal needs writers who test in use.

**Repo evidence (apart).**
- Known: 22 commits of `CONTEXT.md`. Entries changed after code in some cases (`research-03 §Summary`).
- Known: stale lines survived a green gate and a review found them (`research-04 §Table`, rows 2a and 8). Known: `CONTEXT.md:40` still said "a cancel handle" after the code removed it.
- Known: `scripts/validate_docs.py` checks the source paths of `CONTEXT.md` and nothing else (`research-04 §1`).
- Known: some entries define a term with its own avoided words ("A concrete playthrough session"; "An NPC in a World") (`research-03 §Per term 10`).

**Sharper question.** Z39.19 weighs a change by its effect on earlier records. What are the earlier records of a codebase (code, docs, plans, specs), and which way re-indexes them (`research-02 §Questions raised 8`)?

## Q6. What is the place of the forces?

**What each model says**
- Alexander: forces are in every entry: context, conflicting forces, configuration (`research-01 §Alexander Q6`).
- Jackson: the purpose is in every concept. The operational principle carries the motive (`§Jackson Q6`).
- Brooks: the architect holds the forces. Prose carries the "why" better than a formal text (`§Brooks Q6`).
- Naur: the theory holder explains why. It cannot be reduced to rules (`§Naur Q6`).
- Evans: the model carries the "why". He does not use the word "forces". Relations of power also act on the model (`§Evans Q6`).
- Schelling: a rationale defends a focal point against the next push (`research-02 §Schelling Q6`).
- Courts and ALI: the reasons are the unit that later courts test (`§US Supreme Court Q6`, `§ALI Q6`).
- Z39.19: scope and history notes inside the term record (`§Z39.19 Q6`).
- Nygard: forces are the content of the record. Name them, and name the tension (`§Nygard Q6`).

**Where they split.** Inside the entry (Alexander, Jackson, Z39.19) against beside it (Nygard, the repo skill, which keeps the glossary "devoid of implementation details") (`research-02 §Where the sources disagree 4`). Also: Jackson's purpose is a need. Alexander's forces conflict. Do they differ in size or in kind (`research-01 §Questions raised 5`)? Guessed: "force" has two senses, a pressure in the problem and a social pressure between teams (Evans). The note may need two words.

**Assumes about writers.** A writer that reasons at a gap needs the forces as text. Brooks and Naur keep them in a person. Inferred: that fails for agents.

**Repo evidence (apart).** Known: the charting notes sketch three forces (slow and fallible model; no half-written Game; agents with no memory) and mark them "inferred, not checked". Known: no way in the repo carries forces to a writer. The nearest lines are `AGENTS.md:334` and `:341` (`research-04 §Per-way notes 2`). Known: the glossary has no place for the "why".

**Sharper question.** The owner accepts that an agent reasons at a gap. From what text does it reason? Is "a short list of forces" a vocabulary, a decision record, or a third thing?

## Q7. One context or several?

**What each model says**
- Brooks: one set of design ideas. If ideas are incompatible, scrap the system (`research-01 §Brooks Q4`).
- Evans: several contexts are inevitable. A term has meaning only in a context. Name the contexts. The context map shows the relation, from conformist to "Separate Ways" (`§Evans Q2, Q4`).
- Jackson: each concept is a kind of context, kept independent (`§Where the sources disagree 4`).
- Alexander: small languages overlap and merge into a more general one (`§Alexander Q3`).
- Z39.19: link a specialised vocabulary to a general one (`research-02 §Questions raised 10`).
- Schelling: a focal point depends on who the people are. A different group may not find the same one (`research-02 §Schelling Q2`).

**Where they split.** One unity (Brooks) against many (Evans). Evans says a boundary needs "intellectual choices and social forces" (`research-01 §Evans Q4`). If nobody chose it, does it exist?

**Assumes about writers.** Evans warns that three or four people can fragment a model. Known: more than ten people hold more than one conception.

**Repo evidence (apart).** Known: this repo has one context. It has one source of vocabulary and one place where words are chosen. It gives no evidence on several contexts. Inferred: the repo cannot test this question.

**Sharper question.** In the work team, who chooses the boundary: a person, a tool, or the layout of the trackers? Is a Jira project, a Confluence space or a GitHub repo a context, and who keeps the map between them?

## The owner's starting position, tested

**Half 1: "When a concept exists, every writer aligns on it."**
- Holds: Brooks, Evans (inside a context), Z39.19, Jackson, ALI style manual (Q2).
- Pulls against it: Alexander (the spirit, not the letter), Courts (distinguish and overrule), Nygard (a blind follow is a risk), Evans (across contexts) (Q4, Q7).
- Repo evidence: 6 avoided words stayed. Known. The counts do not say whether writers did not read the entry or found it wrong.

**Half 2: "At a gap, an agent reasons from the forces and makes its own choice."**
- Holds: Alexander, ALI, Evans, Jackson, Naur (Q3).
- Pulls against it: Brooks, Z39.19, Nygard and Courts add a decider and probation. Schelling adds that a copied choice becomes precedent (Q3, Q4).
- Needs: forces as text (Q6). The repo has no one place for them.

## No source tests a writer with no memory

- Known: neither `research-01` nor `research-02` tests it.
- Inferred: models that need a person who persists (Brooks, Naur, Evans, Alexander) transfer to the repo through the owner or through text.
- Inferred: models that work through a record (Z39.19, Nygard, Courts) are nearest. They did not test a shared prior with no memory.
- Guessed: the prior of a model is a Schelling clue that nobody chose.
- Inferred, for the work team: most models were built for settings like it. The new part is the agents among the seven people, not the people.
