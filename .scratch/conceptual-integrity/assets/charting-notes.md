# Charting notes: how this map came about

Record of the charting conversation of 2026-10-21/22, written by the coordinator. [Prototype: draft the framework note](../issues/05-prototype-framework-note.md) reads it. The two source research tickets do not read it, because its summaries of the models are second-hand and must not steer them.

## Summary

- The map [the concepts the code follows](../../code-concepts/map.md) finished its research: 97 merged concepts, 22 meeting points across areas, and 114 questions.
- The owner judged that a grilling session over that many items is not practical, and that "add this concept or not" is not the useful level.
- The owner wants a framework: models that apply to this repo, to `CONTEXT.md`, and to work in general.
- The owner's starting position: when a concept exists, every writer aligns on it. At a gap, an agent reasons and makes its own choice.
- The coordinator offered models, then merged them into a checklist. That closed the owner's main tension too early (see "Where convergence was forced").
- One live case: `Save unit` in `CONTEXT.md` against "transaction" in the code.

## The owner's words (verbatim)

1. After the synthesis: "Well fundaemntally a 78 question grilling sesion isn't a practical outcome. And 'add this concept or don't add this concept' is not exactly that helpful as of that either"
2. "Hmm, I feel like there is a higher level conceptualization/philosophical that I want to do, but I'm not entirely sure what exactly I want. Like pick word A or pick word B is not nesscarily going to be super helpful long term? I feel like I want to built onto something but I'm not sure what"
3. "The context.md (esssentially a glossary) has it's own tension. What terms should go in there and what shouldn't. I don't want it to get bloated but then doesn't it make sense to add every distinct concept even eif it seems self-evident? That kind of tension is something I'm no sure how to solve [...] I'm not with it reasoning and making a choice based on that reasoning. The ideal with naming concepts in the context.md is to align everything on a single concept when that concept exists, to avoid having multiple different concepts at the same time. HOWEVER, rather than looking for a simple solution, I'm trying to build up a strategy/mental models/understanding/framework for myself that I can apply holistically both to this project and these problems and in general"
   - The owner later confirmed the reading of "I'm not with it": **"I'm fine with it reasoning"**. At a gap, the agent may reason and choose.
4. "So another thing I'm wondering, have we convergence on a single answer for a single question, when there are actual multiple questions/concepts (juggling balls in the air) , or is that that it's really narrowing down it? I'm just wondering if I rerailed an existing conversation for forced a convergence by bringing up CONTEXT.md"

Earlier, in the charting of the first map: "Sometimes it might make sense to use an overlapping or opposing concept, but at least if it knows we are using the other one, it can realise it can follow that if it make sense".

## The models, as the coordinator summarised them

Second-hand and from memory. Nobody checked these summaries against the sources. The source research does that.

| Model (source) | The idea | What it seemed to say about the tension |
|---|---|---|
| Conceptual integrity (Brooks, *The Mythical Man-Month*, 1975) | One set of design ideas is better than many good ideas that do not fit together. | The owner's ideal, under its established name. Consistency is worth more than a choice that is better in one place. |
| Focal point (Schelling, *The Strategy of Conflict*, 1960) | People who cannot talk converge on the option that stands out. | Agents from one model share one prior. Where the prior is unanimous, they converge with no entry; where it splits, they diverge. The test of an entry: would two fresh agents choose differently? |
| One concept per purpose (Jackson, *The Essence of Software*, 2021) | Concepts and purposes are one to one. The failures are redundancy (two concepts, one purpose) and overloading (one concept, two purposes). | A distinct concept is a distinct purpose. A synonym and a meeting point are both redundancy. At a meeting point, ask "one purpose or two?", not "X or Y?". |
| Precedent (common law: stare decisis; the Restatements of the American Law Institute) | Courts decide a case when it arrives, and later courts follow it. A Restatement describes what courts do. A split between courts stays until it causes harm. | "Align on the concept when it exists" is stare decisis. A vocabulary grows where real cases happen. Only splits that cost something need a ruling. |
| Shearing layers (Brand, *How Buildings Learn*; Foote and Yoder, "Big Ball of Mud") | Parts change at different speeds. | Some bloat is fast facts in a slow file. Long-term value sits on the slowest layer. |
| Theory building (Naur, 1985) | The real program is the theory in its builders' heads. Docs cannot hold all of it. | Each agent session is a new team with no theory. A vocabulary carries the cheap part of the theory. |
| Pattern language (Alexander, 1977) | A pattern resolves forces in a context and links to larger and smaller patterns. | Concepts as a network with forces, not a flat list. |
| Drivers and tactics (Bass, Clements and Kazman, *Software Architecture in Practice*) | Quality attributes and constraints select the tactics. | A short list of drivers from which the concepts follow. |
| "X over Y" statements (the Agile Manifesto; the Zen of Python) | Both sides have value, and one side wins by default. | A form for the tensions: "X by default, Y on purpose". |
| Controlled vocabularies (ANSI/NISO Z39.19) | Preferred and non-preferred terms; literary warrant: a term enters when the material needs it. | `_Avoid_` is a list of non-preferred terms. Literary warrant is close to the owner's `CONTEXT.md` tension. |

**Where the models disagree (the owner's tension):** Jackson gives every distinct purpose a concept, even an obvious one. Schelling needs an entry only where fresh writers would split. The checklist hid this disagreement.

## A sketch of forces (inferred, not checked)

The concepts of the first map seemed to cluster around a few forces:

| Force | Concepts that answer it |
|---|---|
| The model is slow, costs money, can fail at any step, and gives different text each time | plan, then execute (order as data); asynchronous request-reply; a snapshot per Swipe; a strict `ScriptedModel`; test tiers split by what is faked |
| A Game record must never be half-written, and more than one request can reach one Game | single writer; the save unit (Unit of Work); a fencing check on the job id; the run hold |
| Agents write most of the code, and each session starts with no memory | guardrail tests; seam contracts with `CASE` tags; generated indexes; `CONTEXT.md`; "no panics" enforced by a lint |

## The `Save unit` case (known, at charting)

- `CONTEXT.md:107-109`: "**Save unit**: One write to storage that lands whole or not at all. _Avoid_: Transaction, batch".
- `src/adapters/driven/storage/transaction.rs:2`: "The transaction: one unit of record writes that lands whole".
- The word "transaction" was in 7 files under `src/`, and "save unit" in 2.
- The term entered `CONTEXT.md` on 2026-10-19 (`4eeadba2`).
- The `/domain-modeling` rule (`.agents/skills/domain-modeling/CONTEXT-FORMAT.md:29`) keeps general programming concepts out. An atomic write is a general concept, and it got in. The two tests ("unique to this context?" and "does the prior split?") disagree here.
- Open: one purpose with two names, or two purposes (the promise, and the storage mechanism that keeps it)?

## The "two purposes" idea (guessed)

The `CONTEXT.md` tension may feel impossible to solve because the file has two purposes. It aligns names: for that purpose, only terms with a split prior matter, and each extra entry costs attention. It also models the domain: for that purpose, every concept with a place in the model matters, even an obvious one. The two purposes have different rules for what goes in. In Jackson's terms, the file may be overloaded. The catalogue may have the same two purposes: to stop an agent from using its own default, and to describe the design as a whole.

## Where convergence was forced

1. The coordinator merged the models into a checklist of four questions. Its second question ("would two fresh agents choose the same?") answered the owner's tension with "only where the prior splits", and it hid that the models disagree on that point.
2. The coordinator built an answer on a reading of the owner's words that the owner had not confirmed, and the reading was wrong.
3. After that reading, the coordinator dropped the forces.
4. The coordinator ended with a narrow question and its own position.
5. The `CONTEXT.md` example brought the inclusion question with it ("which terms go in?"). The coordinator answered that question, not the tension behind it.

## Ideas raised and not chosen

- Select concepts by agent error: search past sessions and reviews for places where agents chose the opposite of the repo. This is close to "user warrant".
- Organise the catalogue by situation (a choice that an agent meets), not by concept.
- Compress the 97 concepts into 10–15 stances, and react to a draft.
- Four tickets on the first map (the limits of the catalogue, meeting points, an outside check, the agent-facing form). The owner had them deleted: too many items, at the wrong level.

## After the map was created (2026-10-22)

The owner's words (verbatim):

5. "So generally, I tend to use project as a test-bed for ideas around agents, llms and such. Which I why I often get de-railed and end up putting more effect things like then would be warnenered considering the scope of the project and what is does. This is a real game that I am developing but it is also a personal project and a projection of some of my thoughts. So this isn't about simple fixing the problem here, but building a holstic understanding that can be brought to my bigger projects at work and at projects in the future"
6. "The work enviroment is (for reference) a team of 7, working with several other teams. Communicating via jira, slack, github and confluence."

Decisions:

- **Q9:** a Purpose line (this repo is a test-bed, and the note must transfer); a wider scope (the writer axis, from people who remember and can talk to agent sessions with no shared memory); a rule that every ticket records what each claim assumes about its writers; "writer" now means a writer of any codebase.
- **Q10:** a third worked case, from outside this repo, tests transfer. Its form waits for the draft.

Observations by the coordinator:

- Most of the sources come from settings where writers remember and can talk: Brooks's teams, courts, indexers. Schelling is the exception: his players cannot talk, which is the agent case. (inferred)
- The work setting has several teams, so bounded contexts (Evans) matter there. This repo has one context. (inferred)
- In the work setting, the channels match the ways to align: Slack lets writers talk, Confluence is a vocabulary that can go stale, and Jira and GitHub hold decisions and precedent. (guessed)
