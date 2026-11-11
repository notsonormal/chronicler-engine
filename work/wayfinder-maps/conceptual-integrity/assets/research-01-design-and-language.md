# Research 01: what the sources of design and language say

Ticket: [Research: what the sources say about design and language](../issues/01-research-sources-design-and-language.md).
Shape matches the asset of ticket 02, so the draft can merge the two.
This asset records what the sources say. It does not apply them to this repo and does not rule on them.

Labels for my own interpretation: **known** (the source states it), **inferred** (I read it from the source), **guessed** (I have no direct text).
Labels for evidence: **primary** (the source's own words) and **second-hand** (a text about the source).

## Summary

1. Five families of source were read, mostly in their own words: Brooks, Jackson, Evans, Naur, Alexander. Details of access are in "Sources and access".
2. Q1 (what gets a name): the five answer with five different tests. Brooks: what the user sees. Jackson: what has a purpose. Evans: what the model needs, kept small. Alexander: what you can name, draw and show as a real conflict of forces. Naur gives no test for a name; he says the choice of what matters needs someone who holds the whole theory.
3. Q2 (align or diverge): Brooks, Evans and Alexander align writers with a shared written or spoken language. Naur says the deeper thing that aligns writers (the theory) cannot be written down and passes only through close contact. Jackson aligns through familiar concepts and concept independence.
4. Q3 (a gap): Brooks has the architect answer and log the answer ("ex cathedra") for all. Evans says gaps surface by using the language, and a new word is a model change. Alexander says invent a pattern, name it, then test it in public. Naur says the holder of the theory fits the change to what exists.
5. Q4 (the opposite of a concept): only Brooks, Evans and Alexander speak to it. Brooks: leave it out, or scrap the system. Evans: draw a new bounded context; inside one context the language is single. Alexander: follow the spirit, not the letter; treat weak patterns "with a certain amount of disrespect". Jackson and Naur give no rule.
6. Q5 (staying current): Alexander says entries are hypotheses that change, and he removed entries (he discarded "most"). Evans says the language and the code change together. Evans says documents that fall behind the language should be archived. Naur says documents cannot carry the theory at all. Brooks keeps one standard text and a weekly log.
7. Q6 (forces, the "why"): Alexander puts forces inside every entry. Jackson puts a purpose inside every entry. Brooks and Naur put the "why" with a person (the architect, the theory holder). Evans puts it in the model, the language and the code.
8. The sources disagree on seven questions (section "Where the sources disagree"). The sharpest: can the shared understanding be written at all (Naur against Brooks, Evans, Alexander)? And does one unit serve one purpose (Jackson) or one model serve implementation, design and talk (Evans)?
9. Writer assumptions: Brooks, Naur, Evans and Alexander assume writers who remember and can talk; the text says so. Jackson does not say (inferred: designers who work apart). Jackson and Alexander rely most on a shared prior; Brooks, Evans and Naur build it by contact. None describes writers with no memory. That gap is the range this map studies. This is **inferred**.
10. I could not answer with a location: what each source says about a vocabulary that a writer reads fresh each time. No source speaks to it directly (see "Questions raised").

## Sources and access

| Source | Read | Label | Where I read it |
|---|---|---|---|
| Brooks, *The Mythical Man-Month*, anniversary ed., ch. 4 | Full chapter | primary | Pearson sample chapter `ptgmedia.pearsoncmg.com/images/0201835959/samplechapter/chap4.html` |
| Brooks, same book, ch. 5 (extracts) and ch. 6 ("Passing the Word", most of it) | Partial | primary (page markers from scan, ±1) | Pearson sample pages; `soloway.pbworks.com` scan via search extraction. **Added**: ch. 6 is the book's answer to "how does the word get passed", which ch. 4 raises but does not answer. |
| Brooks, ch. 16–19 (retrospective) | Not read | none | I did not find a free primary copy. |
| Jackson, "Towards a Theory of Conceptual Design for Software" (Onward! 2015) | Sections 9, 11, 13, 14 as search extracts, not the whole paper | primary (partial) | `groups.csail.mit.edu/sdg/pubs/2015/concept-essay.pdf` |
| De Rosso and Jackson, "Purposes, Concepts, Misfits, and a Redesign of Git" (OOPSLA 2016) | §1–2, §6–7 | primary | `spderosso.github.io/oopsla16.pdf`. **Added**: it states the criteria in short form and says what a concept is for a developer. |
| Jackson, "The Essence of the Essence" (2022), his own summary of the book | Whole | primary for Jackson's claims; it is a summary, not the book | `essenceofsoftware.com/posts/distillation/` |
| Jackson, *The Essence of Software* (2021), ch. 11 sample | Read; not quoted (marked "Draft: not for distribution or quotation") | primary | `essenceofsoftware.com/posts/sample-chapters/eos-11-concept-integrity.pdf` |
| Jackson, *The Essence of Software*, the book | Not read | none | Only the sample, the summary and the papers. |
| Evans, *DDD Reference* (2015, CC-BY) | Parts I and IV in full; Parts II, III and V skimmed | primary (Evans's own summaries of his patterns; printed page numbers) | `domainlanguage.com/wp-content/uploads/2016/05/DDD_Reference_2015-03.pdf` |
| Evans, *Domain-Driven Design* (2003), ch. 2 | Ch. 2, the language and document sections | primary, but the **final manuscript of 2003-04-15**, not the printed book; manuscript page numbers | `fabiofumarola.github.io/nosql/readingMaterial/Evans03.pdf`. **Added**: the Reference is a summary; ch. 2 has the text on gaps, documents and "one model". |
| Evans, ch. 14 | One discussion excerpt with Evans speaking; the chapter itself not read | primary (informal) | `dddcommunity.org/uncategorized/ch14/` |
| Naur, "Programming as Theory Building" (1985) | Whole text | primary. Read as reprinted in Cockburn, *Agile Software Development* (2002), App. B. Page numbers below are the reprint's (pp. 393–405); the paper is pp. 253–261 of *Microprocessing and Microprogramming* 15. | `gwern.net/doc/cs/algorithm/1985-naur.pdf` |
| Cockburn's commentary on Naur ("Applying Theory Building", reprint pp. 405–406) | Read | **second-hand**. **Added**: it is the one text in this family that applies Naur to writing documentation and naming. | same file |
| Alexander, *The Timeless Way of Building* (1979) | Selected passages of ch. 13–18 and 22; the rest not read | primary; OCR text of a scan, page numbers from running headers (±1) | `archive.org/stream/eb_The_Timeless_Way_of_Building_Complete/…djvu.txt` |
| Alexander, *A Pattern Language* (1977), Introduction | Introduction only | primary (page numbers ±1) | `api.pageplace.de` preview of the Oxford ed. |
| Alexander, *Notes on the Synthesis of Form* (1964) | Not read | **second-hand** only, through Jackson ("Essence of the Essence", "Concept Purposes") | Jackson names it as the origin of "purpose distinct from specification". |

## Brooks

### Q1. What gets a name or an entry, and what stays out?

- **Claim.** What belongs in the shared design is what the user sees. The architecture is "the complete and detailed specification of the user interface". How it is made stays with the implementer. Features that do not fit the basic concepts stay out. [primary]
  - "By the architecture of a system, I mean the complete and detailed specification of the user interface." (ch. 4, "Aristocracy and Democracy")
  - "Where architecture tells what happens, implementation tells how it is made to happen." (Blaauw, quoted in ch. 4, same section)
  - "It is better to have a system omit certain anomalous features and improvements, but to reflect one set of design ideas, than to have one that contains many good but independent and uncoordinated ideas." (ch. 4, "Conceptual Integrity")
  - "The manual … describes and prescribes every detail of what the user sees." (ch. 6, "Written Specifications—the Manual")
- **Claim.** The cost of a name is the cost of learning it. "Ease of use is enhanced only if the time gained in functional specification exceeds the time lost in learning, remembering, and searching manuals." So function is weighed against "conceptual complexity". [primary] (ch. 4, "Achieving Conceptual Integrity")
- **Claim.** A system can have few named elements and still be hard, because the combinations are not obvious. "It is not enough to learn the elements and rules of combination; one must also learn the idiomatic usage, a whole lore of how the elements are combined in practice." [primary] (ch. 4, same section)

### Q2. What makes writers align, and what makes them diverge?

- **Claim (align).** Few minds hold the concepts. "The design must proceed from one mind, or from a very small number of agreeing resonant minds." "Someone must control the concepts." [primary] (ch. 4, "Aristocracy and Democracy")
- **Claim (align).** A division of labour between architecture and implementation, and a set of passing-the-word tools: the written manual, one standard text when there are two forms, a declaration that every implementation must include, multiple implementations, a log of questions and answers, an independent product test. [primary] (ch. 6, all sections)
  - "How can a group of 10 architects maintain the conceptual integrity of a system which 1000 men are building?" (ch. 6, opening)
  - "The necessity for strict compatibility among these served as the best possible enforcing agent for the specifications." (ch. 6, "Multiple Implementations", p. 68)
  - "…a necessary link in the chain by which the design word is passed" (ch. 6, "Product Test", p. 69)
  - On direct incorporation: design the declaration and "require the implementations to include that declaration via a compile-time operation" (ch. 6, "Direct Incorporation", p. 66)
- **Claim (diverge).** Many people make design choices, each tempted to improve the earlier ones, for fashion or taste. "The later builders were tempted to improve upon the designs of the earlier ones, to reflect both changes in fashion and differences in individual taste." Disunity usually comes "from the separation of design into many tasks done by many men". [primary] (ch. 4, "Conceptual Integrity")
- **Claim (diverge).** Without an external architecture, implementers spend their effort on design debates. "We finally decided to implement the language unchanged and unimproved, for the debates about language would have taken all our effort." (Conway, quoted in ch. 4, "Aristocracy and Democracy") [primary]

### Q3. What happens at a gap?

- **Claim.** The writer who meets a gap asks the architect and does not guess. The architect's answer binds and goes to everyone. [primary]
  - "It is essential, however, to encourage the puzzled implementer to telephone the responsible architect and ask his question, rather than to guess and proceed. It is just as vital to recognize that the answers to such questions are ex cathedra architectural pronouncements that must be told to everyone." (ch. 6, "The Telephone Log", pp. 68–69)
  - "In it he records every question and every answer. Each week the logs of the several architects are concatenated, reproduced, and distributed to the users and implementers."
- **Claim.** Where architects disagree, a meeting decides; if there is no consensus the chief architect decides; decisions are minuted and "widely disseminated". There is a second, rarer forum (a "court") for the nagging issues. [primary] (ch. 6, "Conferences and Courts", pp. 66–67)
- **Claim.** A fresh concept may come from anyone. "I will certainly not contend that only the architects will have good architectural ideas. Often the fresh concept does come from an implementer or from a user." Control of what enters stays with the architect. [primary] (ch. 4, "Aristocracy and Democracy")
- **Claim.** The architect "suggests, not dictates" when it comes to implementation, and the builder has "the inventive and creative responsibility for the implementation". [primary] (ch. 5, "Interactive Discipline for the Architect")

### Q4. When may a writer choose the opposite of an existing concept?

- **Claim.** A good idea that does not fit is left out. "Good features and ideas that do not integrate with a system's basic concepts are best left out. If there appear many such important but incompatible ideas, one scraps the whole system and starts again on an integrated system with different basic concepts." [primary] (ch. 4, "Aristocracy and Democracy")
- **Claim.** When the manual and the machine disagree, the answer depends on which is cheaper to change. With one implementation "the manual usually loses". With several, the manual wins, because fixing the faithful machines costs more. [primary] (ch. 6, "Multiple Implementations", p. 68)
- Brooks gives no rule for a deliberate exception inside a system. **known** (I found none in ch. 4–6).

### Q5. How does the vocabulary stay current?

- **Claim.** The manual changes in a cycle with feedback. "Round and round goes its preparation cycle, as feedback from users and implementers shows where the design is awkward to use…" [primary] (ch. 6, "Written Specifications—the Manual")
- **Claim.** If there are two forms of the definition, one is the standard and the other is "a derivative description, clearly labeled as such". "Never go to sea with two chronometers; take one or three." [primary] (ch. 6, "Formal Definitions", pp. 64–65)
- **Claim.** Changes reach everyone by a fast route (here, during a series of decision meetings): "each participant found an updated manual, embodying yesterday's decisions, at his seat every morning." [primary] (ch. 6, "Conferences and Courts", the passage on the "fall festivals")
- Brooks does not speak to the removal of an entry as such. **known**

### Q6. What place do the forces (the "why") have?

- **Claim.** The architect holds the forces. "The architect sits at the focus of forces which he must ultimately resolve in the user's interest." [primary] (ch. 4, "Aristocracy and Democracy")
- **Claim.** The purpose of the system is the criterion. "Because ease of use is the purpose, this ratio of function to conceptual complexity is the ultimate test of system design." [primary] (ch. 4, "Achieving Conceptual Integrity")
- **Claim.** The "why" fits prose better than a formal text. Of formal definitions: "What they lack is comprehensibility." With prose "one can show structural principles… Most important, one can explain why." [primary] (ch. 6, "Formal Definitions", p. 65)

### What Brooks assumes about his writers

- **Remember: yes. Talk: yes. Shared prior: built by working together.** [primary]
  - "The same group—architects, users, and implementers—meets weekly for months. No time is needed for bringing people up to date." (ch. 6, "Conferences and Courts", p. 67)
  - The architect is "the user's agent", and "their product must endure longer than that of an implementer" (ch. 4, "Aristocracy and Democracy").
  - The cathedral builders show a long line of builders who each "sacrificed some of his ideas" (ch. 4). **inferred**: this assumes that later builders know the earlier plan and choose to follow it.
- **Inferred:** The scheme needs a named person who answers. A writer with no one to ask has only the manual and the log.

## Jackson

All Jackson sources are primary for Jackson's claims. Scope note (**known**): his "concept" is "a reusable unit of user-facing functionality that serves a well-defined and intelligible purpose" ("Essence of the Essence", "A New Definition of Concept"). It is a design unit of an app for its users. Two texts also say it matters to developers: "something a developer needs to understand to work effectively with its code" (OOPSLA 2016, §2.1, p. 293).

### Q1. What gets a name or an entry?

- **Claim.** A concept gets a name if it has a purpose. A concept without a purpose, or one that arises "from implementation concerns", "usually spell[s] trouble". A purpose without a concept is also a flaw. [primary]
  - "Motivation. Each concept should be motivated by at least one purpose." (OOPSLA 2016, §2.1, p. 293)
  - "Specificity. … the purposes and the concepts that aim to fulfill them should be in one-to-one correspondence." ("Essence of the Essence", "Concept Design Principles")
- **Claim.** Two concepts for one purpose are a fault ("no redundancy"), and one concept for two purposes is a fault ("no overloading"). Causes of overloading: false convergence, denied purpose, emergent purposes, piggybacking. [primary] ("Essence of the Essence", "Concept Design Principles"; Onward! 2015 §13–14)
  - "Non-division. … When the same purpose motivates different concepts in different contexts, confusions tend to arise about why distinct concepts are needed for the same purpose." (OOPSLA 2016, §2.1, p. 294)
- **Claim.** Keep the state small. "The concept state should be no richer than it need be." There is no global data model; each concept has a local one, "just rich enough to support its actions". [primary] ("Essence of the Essence", "A New Definition of Concept"; "A Structure for Describing Concepts")
  - "In traditional modeling, it's all too easy to get carried away with building a data model based on ontological observations that are not actually relevant to the design of the system."
- **Claim.** Entities of a data model are "too small" to be concepts. "An entire conceptual model seems to be too large to count as a single concept, and its constituent entities (or classes or objects) are too small, especially since an informal understanding of a concept tends to involve relationships amongst multiple elements." [primary] ("Essence of the Essence", "Prior Work on Conceptual Design")

### Q2. What makes writers align, and what makes them diverge?

- **Claim (align).** Familiarity. "When possible, a familiar concept should be preferred to a new, unfamiliar one." "When a purpose arises in an app that has arisen before, the same solution should be used." Familiar concepts carry "all the discoveries and refinements previously made". [primary] ("Essence of the Essence", "Concept Design Principles"; "Concept Reuse and Familiarity")
- **Claim (align).** Independence lets teams work apart. "Concept independence lets design scale, because individual concepts can be worked on by different designers or design teams, and brought together later." [primary] ("Concept Independence")
- **Claim (diverge).** Overloading, by four routes (see Q1). Also a concept that is adjusted to the app breaks "integrity": each concept "should continue to behave according to its (app-independent) concept definition" when composed. [primary] ("Concept Design Principles", "Integrity")
- **Claim (naming).** In help material, "use consistent names for a concept throughout". [primary] ("Strategies")

### Q3. What happens at a gap?

- **Claim.** First look for a familiar concept. "Find familiar concepts lurking behind the concepts of your app." If none fits, a new concept is "invented to solve a particular problem, which is called the motivating purpose". [primary] ("Strategies"; OOPSLA 2016, §2.1, p. 293)
- **Claim.** Concepts are "socio-technical inventions that serve a purpose", unlike a domain model, whose structures "are usually discovered rather than invented". [primary] ("Comparisons to Other Approaches", "Conceptual modeling")
- **Claim.** The community could keep "concept catalogs" of the knowledge and of the relations between concepts. He puts this as a possibility. [primary] ("Concept Reuse and Familiarity")
- Jackson does not say who names a new concept in a team. **known**

### Q4. When may a writer choose the opposite of an existing concept?

- **Claim.** His criteria are not rules. "These criteria are neither necessary nor sufficient. They are not necessary because it is possible to violate them and still end up with a good design." [primary] (Onward! 2015, §11, from a search extract of the paper)
- **Claim.** Familiarity violations and integrity violations are named as faults, with examples. He does not give a test for a justified exception. **known**

### Q5. How does the vocabulary stay current?

- **Claim.** Misfits show only in use. "Misfits … are never fully predictable, because they depend on properties of the human environment which may not even be knowable until the design is deployed." [primary] ("Testing and Prototyping Are Not Enough")
- **Claim.** A project can inventory its concepts and then drop, replace, merge or split them: "Identify redundant concepts that could be removed by being replaced by existing concepts"; "split [overloaded concepts] into separate concepts". [primary] ("Strategies")
- Jackson does not describe how a shared vocabulary of a team is kept current. **known**

### Q6. What place do the forces (the "why") have?

- **Claim.** The purpose is the "why", and it is part of every concept: "Each concept in its design should have its own purpose." Purposes "cannot be judged formally". [primary] ("Concept Purposes")
- **Claim.** The operational principle (a short scenario) carries the motivation that a full behaviour description loses: a full description "fails to distinguish the essential aspects … from more arbitrary design decisions" and does not convey "the motivation for the behavior". [primary] ("The Operational Principle")
- **Claim.** The idea of a purpose distinct from the specification comes from Alexander's *Notes on the Synthesis of Form*. [second-hand, Jackson's attribution] ("Concept Purposes")

### What Jackson assumes about his writers

- **Remember / talk:** not stated. **inferred**: the unit of work is a designer or design team that works on concepts "independently" and joins them later, so he assumes no day-to-day talk between concept designers.
- **Shared prior: yes, a large one.** The method relies on designers who know familiar concepts. "…an expert designer who can apply prior knowledge of likely problems and known solutions, often acquired in very different settings." [primary] ("Testing and Prototyping Are Not Enough")

## Evans

### Q1. What gets a name or an entry?

- **Claim.** A model is selective. "A system of abstractions that describes selected aspects of a domain and can be used to solve problems related to that domain." [primary] (Reference, Definitions, p. vi)
- **Claim.** Keep the core small and make it stand out. "Define a core domain and provide a means of easily distinguishing it from the mass of supporting model and code… Make the core small." For general parts: "Leave no trace of your specialties in them." [primary] (Reference, "Core Domain", p. 40; "Generic Subdomains", p. 41)
- **Claim.** The vocabulary of the language is the model's names. "The vocabulary includes the names of classes and prominent operations." It is "enriched with the names of patterns the team commonly applies". [primary] (2003 manuscript, ch. 2, "Ubiquitous Language", p. 26)
- **Claim.** Name the contexts too. "Name each bounded context, and make the names part of the ubiquitous language." [primary] (Reference, "Context Map", p. 29)
- **Claim.** Large-scale structure should be minimal. "An ill-fitting structure is worse than none, it is best not to shoot for comprehensiveness, but rather to find a minimal set that solves the problems that have emerged. Less is more." [primary] (Reference, "Evolving Order", p. 48)

### Q2. What makes writers align, and what makes them diverge?

- **Claim (align).** A language used by everyone, in speech, writing and code. "Commit the team to exercising that language relentlessly in all communication within the team and in the code. Within a bounded context, use the same language in diagrams, writing, and especially speech." [primary] (Reference, "Ubiquitous Language", pp. 3–4)
- **Claim (align).** The people who write code must own the model. "If developers don't realize that changing code changes the model, then their refactoring will weaken the model rather than strengthen it." Whoever contributes to the model "must spend some time touching the code". [primary] (Reference, "Hands-on Modelers", p. 7)
- **Claim (align).** Continuous integration, with tests, "to flag fragmentation quickly", plus use of the language "to hammer out a shared view of the model as the concepts evolve in different people's heads". [primary] (Reference, "Continuous Integration", p. 5)
- **Claim (align).** Names that state purpose let a later writer infer the right purpose. Without that: "that new developer may infer a purpose that the operation or class fulfills only by chance… the two developers will be working at cross-purposes." [primary] (Reference, "Intention-Revealing Interfaces", p. 20)
- **Claim (diverge).** Models split on their own. "When a number of people are working in the same bounded context, there is a strong tendency for the model to fragment. The bigger the team, the bigger the problem, but as few as three or four people can encounter serious problems." [primary] (Reference, p. 5)
- **Claim (diverge).** Translation between jargons hides splits. "Translation muddles model concepts… The indirectness of communication conceals the formation of schisms, where different team members use terms differently but don't realize it." [primary] (2003 manuscript, ch. 2, "Ubiquitous Language", p. 25–26)
- **Claim (diverge).** "Multiple models are inevitable, yet when code based on distinct models is combined, software becomes buggy, unreliable, and difficult to understand." "Model expressions, like any other phrase, only have meaning in context." [primary] (Reference, "Bounded Context", p. 2)

### Q3. What happens at a gap?

- **Claim.** Use of the language shows the gap, and the team names it. "As gaps are found, new words will enter the discussion. With a commitment to the model-based language, these changes will be recognized as changes in the domain model and will lead to updates of class diagrams and renaming of classes and methods in the code." [primary] (2003 manuscript, ch. 2, "Ubiquitous Language", pp. 26–27)
- **Claim.** A new word is a model change. "Recognize that a change in the language is a change to the model. Iron out difficulties by experimenting with alternative expressions, which reflect alternative models. Then refactor the code, renaming classes, methods, and modules to conform to the new model." [primary] (Reference, p. 4)
- **Claim.** The people who name are developers and domain experts together, in conversation. "Resolve confusion over terms in conversation, in just the way we come to agree on the meaning of ordinary words." [primary] (Reference, p. 4)
- **Claim.** A shared part of the model is not changed alone. The shared kernel "shouldn't be changed without consultation with the other team". [primary] (Reference, "Shared Kernel", p. 31)

### Q4. When may a writer choose the opposite of an existing concept?

- **Claim.** Inside a context: no. "Apply Continuous Integration to keep model concepts and terms strictly consistent within these bounds." [primary] (Reference, "Bounded Context", p. 2)
- **Claim.** Across contexts: yes, and the choice is explicit. The options run from conforming to the upstream model, through a translation layer, to "Separate Ways" ("Declare a bounded context to have no connection to the others at all"). [primary] (Reference, pp. 33–37)
- **Claim.** A boundary must be a decision. "Well-defined context boundaries only emerge as a result of intellectual choices and social forces… When these factors are absent, or disappear, multiple conceptual systems mix together." [primary] (Reference, "Big Ball of Mud", p. 38)
- **Claim.** A metaphor may be dropped: "be ready to drop it if it gets in the way." [primary] (Reference, "System Metaphor", p. 49)

### Q5. How does the vocabulary stay current?

- **Claim.** The language and the code change together; the old terms are renamed. (Quotes in Q3.) [primary]
- **Claim.** A written document left behind by the language should leave the active set. "If the terms explained in a design document don't start showing up in conversations and code, it is not fulfilling its purpose." And: "It could be archived as history, but if it is left active it could actually create confusion and hurt the project." [primary] (2003 manuscript, ch. 2, "Written Design Documents", pp. 33–34)
- **Claim.** Why documents decay: "Once a document takes on a persistent form, it often loses its connection with the flow of the project. It is left behind by the evolution of the code, or it is left behind by the evolution of the language of the project." And code can mislead too: "a method name can be ambiguous, misleading, or out of date with the internals of the method." [primary] (same, pp. 32–34)
- **Claim.** Tests for a document: it "shouldn't try to do what the code already does well", and it must be "involved in project activities". [primary] (same, p. 33)
- **Claim.** Before you change a boundary, map what exists. "Map the existing terrain. Take up transformations later." [primary] (Reference, "Context Map", p. 29)

### Q6. What place do the forces (the "why") have?

- **Claim.** The model carries the "why" and is the language's backbone: "Use the model as the backbone of a language." Documents should "illuminate meaning, … give insight into large-scale structures, and … focus attention on core elements", and can "clarify design intent". [primary] (Reference, p. 4; 2003 manuscript, ch. 2, p. 33)
- **Claim.** Code can show what it does, but "the meaning behind a behavior can be hard to convey". [primary] (2003 manuscript, ch. 2, p. 33)
- **Claim.** Relations of power also act as forces on the model. Relationships between contexts "place constraints on the nature of model or pace of change that is feasible. These constraints manifest themselves primarily through non-technical channels…" [primary] (Reference, "Context Map", p. 29)
- Evans does not use "forces" as a term in the sources I read. **known**

### What Evans assumes about his writers

- **Remember: partly. Talk: yes, and speech is central.** [primary]
  - "…the most incisive expressions of the domain often emerge in a transient form that is never captured in the code or even in writing." (Reference, "Ubiquitous Language", p. 3)
  - "Domain experts should object to terms or structures that are awkward or inadequate… Play with the model as you talk about the system." (same)
  - "Verbal communication supplements the code with meaning. But while talking is critical to connecting everyone to the model, a group of any size will probably need the stability and sharability of some written documents." (2003 manuscript, ch. 2, p. 33)
- **Shared prior: built in the team, not assumed.** "Relentlessly exercise the ubiquitous language to hammer out a shared view of the model as the concepts evolve in different people's heads." (Reference, p. 5)
- The scale of "team" is stated: more than ten people gives "more than one conception of the domain" (Evans, informal, `dddcommunity.org/uncategorized/ch14/`). [primary, informal]

## Naur

Page numbers are those of the Cockburn reprint. The headings are Naur's own.

### Q1. What gets a name or an entry?

- Naur has no test for a name or an entry. **known**
- **Claim.** What is relevant in the world is a judgment of someone who holds the whole. "By far the largest part of the world aspects and activities will of course lie outside the scope of the program text, being irrelevant in the context. However, the decision that a part of the world is relevant can only be made by someone who understands the whole world." [primary] ("The Theory to Be Built by the Programmer", p. 397)

### Q2. What makes writers align, and what makes them diverge?

- **Claim (align).** A shared theory, built by contact. "What is required is that the new programmer has the opportunity to work in close contact with the programmers who already possess the theory." [primary] ("Program Life, Death, and Revival", p. 400)
- **Claim (diverge).** Writers without the theory patch. Group B's additions "effectively destroyed [the compiler's] power and simplicity", and ten years later the structure was "still visible, but made entirely ineffective by amorphous additions of many different kinds". [primary] ("Programming and the Programmers' Knowledge", Case 1, pp. 394–395)
- **Claim (diverge).** Even a living team drifts. "Similar problems are likely to arise even when a program is kept continuously alive by an evolving team of programmers, as a result of the differences of competence and background experience of the individual programmers, particularly as the team is being kept operational by inevitable replacements of the individual members." [primary] (p. 401)
- **Claim (documents).** "The program text and its documentation has proved insufficient as a carrier of some of the most important design ideas." [primary] (Case 1, p. 395)
- **Claim (second-hand, Cockburn).** A shared metaphor lets writers guess alike: "An appropriate, shared metaphor lets a person guess accurately where someone else on the team just added code, and how to fit her new piece in with it." Also: "Simple, consistent naming conventions help the next person build a coherent theory." [second-hand] ("Applying 'Theory Building'", pp. 405–406)

### Q3. What happens at a gap?

- **Claim.** The holder of the theory answers from the similarity between the new demand and what the program already does. "The design of how a modification is best incorporated into an established program depends on the perception of the similarity of the new demand with the operational facilities already built into the program." This similarity "cannot be reduced to any limited set of criteria or rules". [primary] ("The Theory to Be Built by the Programmer", item 3, pp. 397–398)
- **Claim.** "In a certain sense there can be no question of a theory modification, only of a program modification. Indeed, a person having the theory must already be prepared to respond to the kinds of questions and demands that may give rise to program modifications." [primary] ("Problems and Costs of Program Modifications", p. 399)
- Naur does not say how a new concept becomes shared by a team. **known** He says the theory passes by contact, not by text.

### Q4. When may a writer choose the opposite of an existing concept?

- **Claim.** Many changes are "correct" in behaviour; they differ in whether they fit the theory. "Some of them perhaps conforming to that theory or extending it in a natural way, while others may be wholly inconsistent with that theory, perhaps having the character of unintegrated patches." "For a program to retain its quality it is mandatory that each modification is firmly grounded in the theory of it." [primary] (pp. 399–400)
- **Claim.** If the theory is lost, he advises a fresh start over a revival: "the existing program text should be discarded and the new-formed programmer team should be given the opportunity to solve the given problem afresh." [primary] (p. 401)
- Naur gives no rule for a justified exception. **known** The choice is "entirely a matter for the programmer to decide" (p. 403). [primary]

### Q5. How does the vocabulary stay current?

- **Claim.** What stays current is the theory held by people, not the text. "The death of a program happens when the programmer team possessing its theory is dissolved." "Program revival, that is reestablishing the theory of a program merely from the documentation, is strictly impossible." [primary] (pp. 400–401)
- **Claim (second-hand, Cockburn).** Documentation "is almost certainly behind the current state of the program". Put in it "that which helps the next programmer build an adequate theory"; "the purpose of the documentation is to jog memories in the reader". [second-hand] (p. 406)

### Q6. What place do the forces (the "why") have?

- **Claim.** The "why" is the second of three things that the theory holder can give, and it is not reducible to rules. "The programmer having the theory of the program can explain why each part of the program is what it is… The final basis of the justification is and must always remain the programmer's direct, intuitive knowledge or estimate." [primary] (item 2, p. 397)
- **Claim.** No fixed method is right. "On the Theory Building View, for the primary activity of the programming there can be no right method." Methods still help to teach: "the quality of the theory built by the programmer will depend to a large extent on the programmer's familiarity with model solutions of typical problems". [primary] ("Method and Theory Building", pp. 402–403)

### What Naur assumes about his writers

- **Remember: yes, and it is the whole point.** The theory "is something that could not conceivably be expressed, but is inextricably bound to human beings" (p. 400). [primary]
- **Talk: yes, in close contact** (p. 400). [primary]
- **Shared prior: partial.** Quality of the theory depends on familiarity with "model solutions of typical problems", "techniques of description", "principles of structuring systems" (p. 403). [primary]
- He puts the writer in a permanent role: "he or she must be given a permanent position" (p. 404). [primary]

## Alexander

Page numbers come from OCR running headers and can be off by one.

### Q1. What gets a name or an entry?

- **Claim.** An entry must be a thing you can name and draw. "You must give it a name. The search for a name is a fundamental part of the process of inventing or discovering a pattern. So long as a pattern has a weak name, it means that it is not a clear concept, and you cannot clearly tell me to make 'one.'" "If you can't draw it, it isn't a pattern." [primary] (*Timeless Way*, ch. 14, "Patterns which can be shared", p. 266)
  - His example: *entry process* → *house street relationship* → *front door indirectly reached from street* → *entrance transition* (pp. 266–268). The test of a good name: "If I tell you to build an entrance transition, you know exactly what to do."
- **Claim.** It must answer a real conflict of forces. "The problem is real. This means that we can express the problem as a conflict among forces which really do occur within the stated context, and cannot normally be resolved within that context." [primary] (ch. 15, "The reality of patterns", p. 282)
- **Claim.** A pattern can pass the format test and still be empty. The "madhouse balcony" has a context, a problem as conflicting forces and a solution, "even though its individual component statements are true, the pattern has no empirical reality as a whole." [primary] (ch. 15, pp. 282–283)
- **Claim.** Keep the set small and layered. "There must not be too many patterns underneath a given pattern." [primary] (ch. 16, p. 319)

### Q2. What makes writers align, and what makes them diverge?

- **Claim (align).** A language that all builders hold. "The expertise is in the language." "A person with a pattern language can design any part of the environment. He does not need to be an 'expert.'" [primary] (ch. 18, "The genetic power of language", p. 352)
- **Claim (align).** The language lives in each user, like genes in each cell. "Each cell contains the DNA, and every cell is able to take on its part within the whole." The same process of repair steers new acts. [primary] (ch. 18, pp. 355–356). The reading that a language works as an internal prior is **inferred**.
- **Claim (diverge).** The language breaks when it becomes private. "The pattern languages which determine how a town gets made become specialized and private." "Since they no longer have a widely shared language… they are also prisoners of the absurd and special languages which they have made in private." [primary] (ch. 13, "The breakdown of language", pp. 231–232)
- **Claim (diverge).** Control does not repair it. Efforts to "control" the environment more "make things still worse"; they "cannot create a whole environment, because they are not sufficiently responsive to the real needs, forces, demands, problems, of the people involved." [primary] (ch. 13, pp. 237–238)
- **Claim.** The language lives only if it is tested by use: "languages stay alive because the people who are using them are also testing them". [primary] (ch. 13, p. 230)

### Q3. What happens at a gap?

- **Claim.** Invent a pattern. "We must therefore invent new patterns, whenever necessary, to fill out each pattern which is not complete." He gives two tests for a gap: functional completeness (an unresolved problem remains) and morphological completeness (an unclear zone remains). His example is the "long blank wall" along a garden that no pattern covers. [primary] (ch. 16, pp. 318–319)
- **Claim.** The first stage is a feeling; then it becomes a named thing. "This morphological feeling, which cannot be exactly stated… is the heart of every pattern. Then, once you discover a fluid field of relationships like this, you must redefine it as an entity, to make it operational." [primary] (ch. 14, pp. 264–265)
- **Claim.** A pattern is shared when it is clear enough to be wrong. "At this stage, the pattern is clearly sharable. People can discuss it, reuse it themselves, improve it, check the observations out for themselves… the pattern is open enough to become empirically vulnerable." [primary] (ch. 14, p. 267)
- **Claim.** Patterns from separate languages can merge into a more general one: *department hearth*, *tangent paths* and *family room circulation* became *common area at the heart*. [primary] (ch. 17, pp. 329–330)
- **Claim.** Anyone can improve a pattern and offer it to the common language. "We hope… that gradually these more true patterns, which are slowly discovered, as time goes on, will enter a common language, which all of us can share." [primary] (*Pattern Language*, Introduction, p. xiv)

### Q4. When may a writer choose the opposite of an existing concept?

- **Claim.** Follow the spirit, not the letter. A Spanish courtyard house put the entrance transition inside the front door: "it was necessary to follow the spirit of the pattern, not the letter." [primary] (*Timeless Way*, ch. 14, pp. 263–264)
- **Claim.** The strength of the pattern sets the freedom. Patterns marked with one asterisk should be treated "with a certain amount of disrespect… seek out variants". Two-asterisk patterns are believed to be true invariants. All patterns remain "hypotheses… free to evolve". [primary] (*Pattern Language*, Introduction, pp. xiii–xiv)
- **Claim.** Each solution "imposes nothing on you. It contains only those essentials which cannot be avoided if you really want to solve the problem." The format is made so that "you can judge it for yourself, and modify it, without losing the essence". [primary] (*Pattern Language*, Introduction, pp. x, xiii)

### Q5. How does the vocabulary stay current?

- **Claim.** Patterns are hypotheses, and they are revised and removed. "We discovered and wrote down many hundreds of patterns. Then we discarded most of these patterns during the years, because we decided that they were absurd; or because we found more subtle formulations of the same ideas; or because we found that they didn't hold water empirically." [primary] (*Timeless Way*, ch. 17, p. 330)
- **Claim.** "I complete the language by creating patterns, and eliminating patterns, until each pattern is complete." [primary] (ch. 16, p. 319)
- **Claim.** "Each pattern may be looked upon as a hypothesis… all tentative, all free to evolve under the impact of new experience and observation." The asterisks mark "our degree of faith". [primary] (*Pattern Language*, Introduction, p. xiv)
- **Claim.** The language is repaired by the same acts that build with it. "…there is no difference between the way the genes control the process of genetic growth which forms the embryo, and the process of repair which heals the cut." [primary] (ch. 18, p. 355)

### Q6. What place do the forces (the "why") have?

- **Claim.** Forces are in the format of every entry. "Every pattern is an instruction of the general form: context → conflicting forces → configuration." [primary] (ch. 15, p. 281)
- **Claim.** Each pattern is stated "in the form of an instruction" with its "empirical background… the evidence for its validity". The format has two purposes: to link each pattern to others, and "to present the problem and solution of each pattern in such a way that you can judge it for yourself". [primary] (*Pattern Language*, Introduction, pp. ix–x)
- **Claim.** Patterns link to each other, and none stands alone. "No pattern is an isolated entity. Each pattern can exist in the world, only to the extent that is supported by other patterns: the larger patterns in which it is embedded, the patterns of the same size that surround it, and the smaller patterns which are embedded in it." [primary] (*Pattern Language*, Introduction, p. xii)
- **Claim.** Forces differ between people, and general rules reach only the general forces. Experts "can only deal with general forces, which are common to all men, and never with the particular forces that make one particular man unique". [primary] (*Timeless Way*, ch. 13, p. 238)

### What Alexander assumes about his writers

- **Remember: yes, as members of a living culture.** The language is held by everyone in a town and used in ordinary acts (ch. 18). [primary]
- **Talk: yes, on the site, in person.** Example: people "walking it out, waving their arms, thinking together, placing stakes in the ground… within the medium of a shared language" (ch. 22, pp. 453–454). [primary]
- **Shared prior: yes, the common language itself.** "Once the common language has been broken down, the individual languages which, in a living culture, are always private versions of the common language, are also broken down." (ch. 13, pp. 238–239) [primary] The writers are many and unrelated: "a thousand different acts, extended over time, and done by unrelated people" (ch. 18, p. 354). [primary]

## Where the sources disagree

Each is a question with both positions. Locations are in the sections above. Labels: **known** where both sides say it, **inferred** where I set the two side by side.

1. **Can the shared understanding be written down?**
   - Naur: no. The theory "could not conceivably be expressed" and revival from documents "is strictly impossible" (pp. 400–401).
   - Brooks, Evans, Alexander: yes, in part, and it is the main tool. Brooks: the manual is "a necessary tool, though not a sufficient one" (ch. 6). Evans: documents "illuminate meaning" but the language is mainly spoken (ms. p. 33). Alexander: "the expertise is in the language" (p. 352).
   - **Inferred:** the gap narrows if "written" means "enough to start a conversation" (Cockburn on Naur, p. 406). Nobody tests a text read with no conversation at all.
2. **Who owns the concepts?**
   - Brooks: a few. "Someone must control the concepts." The chief architect decides (ch. 4; ch. 6, p. 67).
   - Alexander: all who build. "There is no way the living town can be built by professionals" (p. 353). Control is a symptom of a broken language (pp. 237–238).
   - Evans: developers and domain experts together, with developers who touch the model ("Hands-on Modelers", p. 7).
   - **Inferred:** Brooks and Alexander also differ on size. Brooks's architects serve "1000 men" on one product. Alexander's language serves a town over generations.
3. **Is the vocabulary fixed or open?**
   - Brooks: answers are "ex cathedra" and a "standard" text wins over its derivative (ch. 6, pp. 64–69). The manual still goes "round and round" with feedback.
   - Alexander: every pattern is a hypothesis, with "a certain amount of disrespect" allowed (Intro, pp. xiii–xiv).
   - Evans: language change is model change, made by use (ms. pp. 26–27).
4. **One unity or many?**
   - Brooks: one set of design ideas for the system; with incompatible ideas, "scrap the whole system" (ch. 4).
   - Evans: "Multiple models are inevitable"; unity holds only inside a bounded context (Reference, p. 2).
   - Alexander: one common language for a town, made from smaller languages that overlap and merge (ch. 17, pp. 329–330).
   - Jackson: "each concept in a sense having its own bounded context", reached by independence ("Comparisons to Other Approaches").
   - This is the open question of the map about several teams.
5. **One unit, one purpose, or one model for several purposes?**
   - Jackson: no overloading. A concept "that tries to serve multiple purposes is pulled in different directions" ("Concept Design Principles").
   - Evans: "one model should underlie implementation, design, and team communication. Having separate models for these separate purposes poses a hazard." (ms. ch. 2, "Explanatory Models", p. 34)
   - **Inferred:** the two speak of different units (a concept; a whole model). The map's question "one file for many purposes" meets both.
6. **Is a name found by discovery or by invention?**
   - Jackson: in conceptual and domain modelling, structures "are usually discovered rather than invented"; a concept is "a socio-technical invention that serves a purpose". He says DDD "typically starts by constructing a model of the specific domain in a bottom-up fashion" ("Comparisons to Other Approaches"). That is Jackson's reading of Evans, so it is second-hand for Evans.
   - Evans: the model is worked out with domain experts, through use of the language, and "reflects alternative models" when the words change (Reference, p. 4; ms. ch. 2, pp. 26–27). **inferred:** he treats the model as made as well as found; the sources I read do not say.
   - Alexander: both. A pattern is "discovered", then "redefined… as an entity" and named (ch. 14, p. 265).
7. **Rules or judgment?**
   - Naur: similarity "cannot be reduced to any limited set of criteria or rules"; "there can be no right method" (pp. 397–402).
   - Jackson: criteria are heuristics, "neither necessary nor sufficient" (Onward! 2015, §11).
   - Alexander: a pattern is an instruction, tested empirically, but follow "the spirit… not the letter" (pp. 263–264, 281).
   - Brooks: a standard text and a log of rulings (ch. 6).

## Questions raised

Each question is new. None is answered here.

1. **A writer that reads the vocabulary fresh each time.** Naur says revival from text is "strictly impossible", and Brooks, Evans and Alexander rely on talk. No source tests a reader who has only the text and no one to ask. What does each model predict for that reader? (This is the writer range of the map.)
2. **Who is "the architect" or "the theory holder" when no writer stays?** Brooks's log and Naur's contact both need a person who persists. Is the vocabulary, the owner, or the tests the persisting party?
3. **Entry tests side by side.** Alexander: nameable, drawable, a real conflict of forces. Jackson: a purpose, one to one. Evans: part of the selected model, core small. Brooks: user-visible. Do the tests select the same entries from one real case? (Fit for the worked cases.)
4. **Alexander's "madhouse balcony" and the format.** A well-formed entry can still be empty. What would be the check, in a vocabulary, that an entry "makes a difference" (ch. 15, p. 283)?
5. **Jackson's purpose and Alexander's forces.** Are these the same thing at two sizes? Jackson says a purpose is a need, not a mechanism. Alexander's forces are in conflict. Does an entry need the conflict, or only the need?
6. **Evans on "gaps" assumes talk.** "As gaps are found, new words will enter the discussion." Where does the discussion happen if the writers do not meet? Brooks's telephone log is one answer; is there another in these sources?
7. **Precedent is not named in any source as a concept, but all five use it.** Brooks: "idiomatic usage, a whole lore". Naur: "unintegrated patches". Evans: a language that changes by use. Alexander: "private versions of the common language". When a choice made at a gap is copied by the next writer, which source says who checks it? (Brooks: the architect and the log. Alexander: public test. Evans: the team's conversation. Naur: the theory holder.)
8. **Removal.** Alexander discarded most of what he wrote. Evans archives documents. Jackson drops redundant concepts. Brooks does not say. What triggers a removal in each, and who decides?
9. **Where does a context boundary lie when teams only share a tracker and chat?** Evans: boundaries come from "intellectual choices and social forces" (p. 38). Does a boundary exist if nobody chose it? (Open question of the map about several teams.)
10. **"Forces" in a team of people and agents.** Alexander and Brooks use "forces" for pressures in the problem. Evans uses "social forces" for the pressures between teams. Are these the same kind? **guessed:** no, and the note may need two words.
11. **Familiarity as a shared prior.** Jackson's principle works if writers share a catalogue of familiar concepts. For agent writers, a model's training is a prior that no one chose. Does any source speak to a prior that is shared but unknown to the vocabulary's owner? **known:** none that I read.
