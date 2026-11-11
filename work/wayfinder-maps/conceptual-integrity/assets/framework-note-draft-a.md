# Framework note, Form A: by model

**ROUGH DRAFT for the owner to react to.** This is one of two forms. The owner chooses; the coordinator then makes `framework-note-draft.md`. This form is a framework. It is not a rule list and not a checklist. It does not close any question.

Labels for my own words: **known** (a source or a file says it), **inferred** (I read it from known facts), **guessed** (weak signal).
Citations point to the research assets by section, for example `research-02 §Z39.19 Q5`.

## Summary

1. Nine models are named: Brooks, Jackson, Evans, Naur, Alexander, Schelling, precedent (common law and the Restatements), controlled vocabulary (Z39.19 and ISO 25964), decision records (Nygard, Parnas and Clements).
2. Each model gives a different test for what earns an entry. The tests pull the owner's tension in both directions.
3. The models agree on three things: a reader who cannot talk needs the reason on the page; one form per thing holds inside one context; a change needs a decider and a visible record.
4. The models split on eight questions. The sharpest two: can shared understanding be written down (Naur says no), and does every distinct concept get an entry or only the ones where writers would split.
5. "Concept" has three senses in the models: a word for one thing, a unit with a purpose, and a solution to forces.
6. Almost every model assumes writers who remember, or who can reach a decider. No source tests a writer with no memory. Transfer to agent sessions is inferred or guessed.
7. The owner's starting position: the first half holds inside one context; the second half holds in the models only with a decider, a probation state and a record.
8. The repo evidence is kept apart (section "Repo evidence"). It cannot show that an entry causes alignment.
9. The last section gives twelve questions that the models make the owner ask in a real case.

## Three senses of "concept" (the map leaves the word open)

| Sense | Source | The test |
|---|---|---|
| A word for one thing | Z39.19 §5.3, ISO 25964 §2.11, Evans (names in the model) | One concept, one preferred form. The concept is independent of its terms. |
| A unit with a purpose | Jackson (`research-01 §Jackson Q1`) | One purpose, one concept. Two concepts for one purpose is a fault. One concept for two purposes is a fault. |
| A solution to forces in a context | Alexander (`research-01 §Alexander Q1`) | You can name it, draw it, and show a real conflict of forces. |

- Inferred: the first sense fits a glossary. The second and third fit a catalogue of ways to do things.
- Guessed: the map's question "is that the line between `CONTEXT.md` and the catalogue?" may have the answer "partly". A word and a way can be one entry in one model and two in another.

## The models

### 1. One mind, an architect and a log (Brooks)

- **Claim.** One set of design ideas is better than many good ideas that do not fit. A few minds control the concepts. The architect answers questions. Every answer goes to everyone in a log (`research-01 §Brooks Q2, Q3`).
- **Test for an entry.** What the user sees. The cost of a name is the cost of learning it (`§Brooks Q1`).
- **Assumes about writers.** They remember. They talk. They share a prior built by weekly meetings (`§Brooks, what Brooks assumes`). A named person must answer.
- **Transfer, this repo.** Inferred: the log idea transfers; the person does not. No writer stays. The owner is the only writer who remembers (`research-03 §Who wrote`).
- **Transfer, work team.** Inferred: it fits a team of 7. The architect role is a named owner. Slack is the telephone. Jira or Confluence can hold the log.

### 2. Concepts with purposes (Jackson)

- **Claim.** A concept exists to serve one purpose. Redundancy and overloading are faults. Familiar concepts are preferred to new ones (`research-01 §Jackson Q1, Q2`).
- **Test for an entry.** A purpose. A concept with no purpose "usually spell[s] trouble".
- **Assumes about writers.** Not stated on memory and talk. Inferred: designers who work apart. Known: a large shared prior, as designers who know familiar concepts (`§Jackson, what Jackson assumes`).
- **Scope.** Known: Jackson's concept is a user-facing unit of an app. Two papers say it also matters to developers. Applying it to a glossary of a codebase is my step (inferred).
- **Transfer, this repo.** Inferred: it asks the sharpest question about `Save unit`: one purpose with two names, or two purposes.
- **Transfer, work team.** Inferred: independence of concepts helps teams that work apart. Jira epics and Confluence pages each name things; two names for one purpose is the fault to look for.

### 3. Ubiquitous language and bounded context (Evans)

- **Claim.** The team uses one language in speech, writing and code. A change of word is a change of model. The language holds inside one bounded context only. Models fragment by themselves (`research-01 §Evans Q2, Q3, Q4`).
- **Test for an entry.** The name belongs to the selected model. Keep the core small (`§Evans Q1`).
- **Assumes about writers.** They partly remember. They talk, and speech is central. The shared prior is built inside the team (`§Evans, what Evans assumes`). Known: "a group of any size will probably need … written documents".
- **Transfer, this repo.** Inferred: the repo has one context. The talk that Evans needs is absent between sessions. Only the owner talks.
- **Transfer, work team.** Inferred and strongest here. Seven people with several other teams is the setting of the context map. Where a boundary lies when teams share only a tracker and chat is an open question (`research-01 §Questions raised 9`).

### 4. Theory held by people (Naur)

- **Claim.** The program is a theory in its builders' heads. Documents cannot carry all of it. A lost theory cannot be revived from text (`research-01 §Naur Q5`).
- **Test for an entry.** None. Relevance is the judgment of someone who holds the whole (`§Naur Q1`).
- **Assumes about writers.** They remember. This is the whole point. They talk in close contact. They need familiarity with model solutions (`§Naur, what Naur assumes`).
- **Transfer, this repo.** Inferred: it predicts that every agent session starts a new team with no theory. It says a vocabulary carries only the cheap part. Cockburn's reading narrows the claim: text can "jog memories" (second-hand, `§Naur Q5`).
- **Transfer, work team.** Inferred: it predicts a loss when a person leaves. The work team can pass the theory by contact. The repo cannot.

### 5. A pattern language and forces (Alexander)

- **Claim.** An entry resolves forces in a context. It links to larger and smaller entries. It is a hypothesis. It is tested by use, revised and removed (`research-01 §Alexander Q1, Q5`).
- **Test for an entry.** Nameable, drawable, and a real conflict of forces. A well-formed entry can still be empty ("madhouse balcony", `§Alexander Q1`).
- **Assumes about writers.** They remember, as members of a living culture. They talk on site. The language is the shared prior (`§Alexander, what Alexander assumes`). Known: the language breaks when it becomes private.
- **Transfer, this repo.** Inferred: forces in each entry is the form that lets an agent reason at a gap. Guessed: the language needs writers who test it in use. Agent sessions do not report back.
- **Transfer, work team.** Inferred: a Confluence page could carry the forces. Public testing means GitHub review and Slack objection.

### 6. Focal points (Schelling)

- **Claim.** Writers who cannot talk converge on the option that is prominent and unique. Precedent, analogy and symmetry make a point prominent. Extra detail can drown the signal (`research-02 §Schelling Q1, Q2`).
- **Test for an entry.** Not a test of what to name. A condition for what can be mutually recognised: unique, prominent (`§Schelling Q1`).
- **Assumes about writers.** They cannot talk (tacit case). They share a history that makes things salient. The clue "depends on time and place and who the people are" (`§Schelling, what Schelling assumes`).
- **Transfer, this repo.** Inferred: the only model whose writers cannot talk. Guessed: agents from one model share a prior, so a focal point may exist. Nobody has tested which one. The text and the record are the clues left when memory is gone (`research-02 §Questions raised 4`).
- **Transfer, work team.** Inferred: it explains why a team converges on whatever came first, an old ticket or a template.

### 7. Precedent: follow, distinguish, overrule (common law and the Restatements)

- **Claim.** An earlier choice binds the next writer in its own case. Any writer may distinguish it by its facts. Only a named body may overrule it, and it gives reasons. A Restatement describes what courts do and says so when it departs (`research-02 §US Supreme Court Q4`, `§ALI Q4`).
- **Test for an entry.** The question actually decided. Few questions reach the top court (`§US Supreme Court Q1`).
- **Assumes about writers.** They read the reported opinions. They talk only through the record. They share a trained profession and a hierarchy (`§US Supreme Court, what the texts assume`).
- **Fairness note (inferred).** Courts bind. A Restatement does not bind. I merged them because both give "explain why" at a departure. They differ on force.
- **Transfer, this repo.** Inferred: the record-only channel fits. The hierarchy does not exist. Which writer may overrule is unset (`research-02 §Questions raised 2`).
- **Transfer, work team.** Inferred: GitHub pull requests and Jira tickets already work as a record of rulings. Which person is the court is a question for the owner.

### 8. Controlled vocabulary with warrant (Z39.19, ISO 25964)

- **Claim.** One preferred term per concept. Non-preferred terms point to it. A term enters on warrant. A candidate term is reviewed by an editor. Every change is dated and gets a history note (`research-02 §Z39.19 Q1, Q3, Q5`).
- **Test for an entry.** Literary warrant (the material), user warrant (the users), organisational warrant (the needs). Fringe detail stays out (`§Z39.19 Q1`).
- **Assumes about writers.** Indexers need not remember. The vocabulary must "convey the intended scope to any user". They talk to the editor, not to each other. ISO 25964 says human intellect usually chooses the terms (`§Z39.19, what Z39.19 assumes`; `§ISO 25964, what ISO assumes`).
- **Source limit (known).** The body of ISO 25964-1 was not read. Part of this model is second-hand.
- **Transfer, this repo.** Inferred: the closest model for a reader with only text. `_Avoid_` is a list of non-preferred terms. The note says what chosen meaning excludes.
- **Transfer, work team.** Inferred: it fits Confluence. It also gives a rule for several teams: link specialised vocabularies to a general one (`research-02 §Questions raised 10`).

### 9. Decision records (Nygard; Parnas and Clements)

- **Claim.** Record the decision and its forces, so that a later reader neither accepts blindly nor changes blindly. Supersede; do not edit. Parnas and Clements add: record the rejected alternatives (`research-02 §Nygard Q2, Q4`; `§Parnas Q1`).
- **Test for an entry.** Nygard: architecturally significant. Parnas and Clements: all alternatives. The repo skill: hard to reverse, surprising, a real trade-off (`§Where the sources disagree 6`).
- **Assumes about writers.** They do not remember, and they cannot ask the author. The reader is "a new person coming on to a project" (`§Nygard, what Nygard assumes`). Evidence for Nygard is a self-report only.
- **Transfer, this repo.** Inferred: the closest to the agent case. It tests a decision, not a concept.
- **Transfer, work team.** Inferred: it fits a Confluence or repo folder. It needs one rule for what is "significant".

## Where the models agree

1. **A reader who cannot ask needs the reason on the page.** Nygard, Parnas and Clements, Z39.19 (scope and history notes), ALI ("explain why"), Alexander (forces in the entry), Jackson (the purpose), Brooks (prose explains why). Naur agrees it matters and doubts that text is enough.
2. **Inside one context, one form per thing.** Z39.19, ISO, Evans, Brooks, the ALI style manual, Jackson.
3. **Writers diverge on their own.** Brooks (taste), Evans (fragmentation), Z39.19 (synonyms), Naur (drift), Alexander (private languages).
4. **Most models name a decider and a probation state.** Brooks (architect), Z39.19 (candidate term), Nygard (proposed), ALI (draft), courts. Alexander and Evans give the role to the group.
5. **A departure is allowed and must be visible.** ALI: "say so explicitly". Nygard: supersede. Z39.19: propose with a rationale. Alexander: modify without losing the essence.
6. **Short beats long.** Nygard, Z39.19 §11.1.7, Evans (core small), Alexander, Brooks (cost of learning), Schelling (noise).
7. **A vocabulary changes.** Alexander, Evans, Z39.19 §2.5, Brooks (the manual cycle), Nygard.
8. **Use shows the failure.** Brooks (product test), Evans (continuous integration), Z39.19 (indexing experiments), Alexander (empirical test).

Inferred: items 1 and 4 rest on writers who can reach a record or a decider. They agree less for a writer with neither.

## Where the models disagree

Each item is a question. The positions follow.

1. **What earns an entry? (the owner's tension).**
   - Every distinct concept: Z39.19 §5.3 ("each distinct concept"), Jackson (every purpose), Evans (the language holds the model's names).
   - Only where writers would split: Schelling (uniqueness; clutter drowns the signal).
   - Only what costs less than it teaches: Brooks, Nygard, Z39.19 §11.1.7, Alexander ("not too many").
   - Only a significant decision: Nygard. All rejected alternatives: Parnas and Clements.
   - No test, a person judges: Naur.
   - Inferred: part of the pull comes from the reader's act. A reader who looks up an entry (Z39.19) can bear a large vocabulary. A reader who must hold it (Nygard, Schelling, Brooks) cannot. Entries meet an agent in different ways (`research-04 §Table`).
2. **Can shared understanding be written down?** Naur: no (`research-01 §Where the sources disagree 1`). Brooks, Evans, Alexander: in part. Nobody tests text with no conversation.
3. **Who owns the concepts?** Few (Brooks, an editor). All who build (Alexander). Developers and experts together (Evans).
4. **Does a vocabulary follow usage or lead it?** Follow: Z39.19 §6.6.1.1. Lead: ALI style manual. Both ask for a reason (`research-02 §Where the sources disagree 1`).
5. **Does alignment come from prominence or from prescription?** Schelling: prominence works with no talk. Z39.19: natural language splits, so prescribe.
6. **Settled or right?** Payne (Brandeis): settled matters more. The Practice Statement and Nygard: too rigid a hold harms.
7. **Do reasons live inside the entry or beside it?** Inside: Z39.19, Alexander, Jackson. Beside: Nygard and the repo skill (`research-02 §Where the sources disagree 4`).
8. **One unit, one purpose, or one model for several?** Jackson: no overloading. Evans: one model should serve implementation, design and talk. Inferred: they speak of different units.
9. **One context or several?** Brooks: one set of ideas or scrap the system. Evans: several are inevitable. Alexander: small languages merge into a larger one. Open for the work team.

## The owner's starting position, tested

**Half 1: "When a concept exists, every writer aligns on it."**
- Holds in: Brooks, Evans (inside a context), Z39.19, Jackson (familiarity), the ALI style manual.
- Pulls against it:
  - Alexander: an entry is a hypothesis. Follow the spirit. The strength of an entry sets the freedom (`research-01 §Alexander Q4`).
  - Courts: any writer may distinguish; only a body may overrule (`research-02 §US Supreme Court Q4`).
  - Evans: across contexts the choice is explicit.
  - Nygard: a writer who aligns with no reason is a risk, not a success.
- So "aligns" splits into three acts: follow, distinguish, overrule. The models give them to different writers. Inferred.

**Half 2: "At a gap, an agent reasons from the forces and makes its own choice."**
- Holds in: Alexander (invent, name, test), ALI ("matters of judgment"), Evans (new words come from use), Jackson (find a familiar concept, else invent), Naur (judgment, not rules).
- Pulls against it: Brooks (ask, do not guess; the answer binds all), Z39.19 (nominate; an editor decides), Nygard and ALI (the new item starts as proposed or draft), courts (lower courts act, the top court waits).
- Schelling adds: a choice that others copy becomes the focal point beyond its logic (`research-02 §Schelling Q2`). So the agent's choice may become precedent. This is the map's open question.
- Most models allow the choice. They add a probation state, a record and a body that can overrule. Known: none of them gives that to a writer with no memory.

## Repo evidence (kept apart from the models)

Known, from `research-03` and `research-04`:
- Avoided words fell to about zero in 5 terms and stayed or grew in 6. Each fall came in a planned code commit (`research-03 §Summary`).
- Entries came before code (Job), after code (Swipe state, Save unit), and changed to match code (Retry, Steering).
- Six names with no entry fell in the same commit as Snapshot (`research-04 §Which other ways`).
- Only `AGENTS.md`, `.pi/APPEND_SYSTEM.md` and the compiler always reach an agent. Names have one written source and no check that fails a build.
- No evidence shows which writer met which way. One git identity wrote 782 of 787 commits.

What this does to the models (inferred, not a ruling):
- It fits Schelling and the precedent model: a word stayed where code held it (`NpcCard`, `Storage::transaction`).
- It fits Z39.19 literary warrant: some entries followed the code.
- It does not test Naur, Brooks or Alexander. Nobody talked about a name.
- It cannot separate an entry from the plan that carries it (`research-03 §What the counts can show`).

## No source tests a writer with no memory

- Known: no source in `research-01` or `research-02` tests it. Only Nygard and Parnas and Clements write for a reader who cannot ask.
- Inferred: models that need a person who persists (Brooks, Naur, Evans, Alexander) transfer to the repo only through the owner or through text.
- Inferred: models that work through a record (Z39.19, Nygard, courts) transfer best. They were not tested on a reader with a shared prior and no memory.
- Guessed: the shared prior of a model is a Schelling clue that nobody chose. The owner cannot read it from the vocabulary (`research-01 §Questions raised 11`).

## Questions the models make the owner ask in a real case

1. Who are the writers: do they remember, can they talk, do they share a prior? (all)
2. Would two fresh writers choose differently with no entry? (Schelling)
3. Is it a word for one thing, a purpose, or a solution to forces? (the three senses)
4. One purpose with two names, or two purposes with one name? (Jackson)
5. What is the warrant: the material, the users, or the organisation? (Z39.19)
6. Does the reader look the entry up or hold it in mind? (Z39.19 against Nygard and Schelling)
7. Who decides at a gap, and what state does the new choice have until then? (Brooks, Z39.19, Nygard)
8. Who may distinguish, and who may overrule? (courts)
9. Where do the forces live: in the entry or beside it? (Alexander, Nygard)
10. Where does the context end? (Evans)
11. What would show that the entry does not work? (Z39.19 indexing test, Alexander, Evans)
12. What is lost when the people who hold the theory leave, and what is text? (Naur)
