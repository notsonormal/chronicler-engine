# Map: conceptual integrity across writers with no memory

Labels: wayfinder:map

## Destination

The owner endorses a framework note, and three worked cases test it. The note names the models of conceptual integrity across many writers, from people who remember and can talk to agent sessions with no shared memory. It says where the models agree and where they disagree, what each model assumes about its writers, and the questions that the models raise in a real case. Two worked cases apply the note in this repo, to `CONTEXT.md` and to the concept catalogue. A third worked case applies it to a case from outside this repo, to test transfer. Any change to the repo is a hand-off.

## Notes

- **Origin:** the charting conversation of 2026-10-21/22, after the research of the map [the concepts the code follows](../code-concepts/map.md). Record: [charting notes](assets/charting-notes.md).
- **Purpose:** this repo is a test-bed. The note must transfer to larger projects at work and to future projects (owner, 2026-10-22, Q9).
- **Reference context for transfer:** a team of 7 that works with several other teams. They communicate through Jira, Slack, GitHub and Confluence (owner, 2026-10-22).
- **Framework:** a set of named models, where they agree and disagree, and the questions they give the owner to ask in a real case. It is not a rule list and not a checklist (owner, charting).
- **Reader:** the owner. Agents read only what the note produces, such as `CONTEXT.md` and the catalogue (owner, charting Q2).
- **Scope:** conceptual integrity across many writers: shared vocabularies, the other ways that writers align (precedent in the code, tests and guardrails, review), and the forces that a writer reasons from at a gap (owner, charting Q3). Writers range from people who remember and can talk, to agent sessions that share a prior but have no shared memory. This repo is at the agent end of that range. The map's name keeps that end, because this test-bed studies it (owner, Q9).
- **Writer assumptions:** every ticket records what each claim assumes about its writers: whether they remember, whether they can talk to each other, and whether they share a prior. This is how the note says which of its parts transfer (owner, Q9).
- **Open endings:** a ticket may end with a sharper question, or with two models that disagree, instead of an answer. Research and drafts come before grilling. Only the owner closes a question. The section "Open questions" lists every open question, also the ones that have a ticket; it takes the place of "Not yet specified" (owner, charting Q4).
- **Execution:** this map writes the note and the worked cases. Changes to the repo are hand-offs: the catalogue goes back to the map [the concepts the code follows](../code-concepts/map.md); a change to the `/domain-modeling` rule becomes a separate small change (owner, charting Q1).
- **The owner's starting position:** when a concept exists, every writer aligns on it. At a gap, an agent reasons from the forces and makes its own choice (owner, charting).
- **Sources:** Brooks, Jackson, Evans, Schelling, Naur, the common-law method, controlled vocabularies, Alexander and design-rationale practice. The list is a minimum, not a limit (owner, charting Q7). Fowler, "Semantic Diffusion" (owner, 2026-10-22).
- **Outside check:** evidence from history early ([Research: does a `CONTEXT.md` entry make writers align?](issues/03-research-names-and-alignment.md)), and a second reader after the draft ([Research: outside check, a second reader](issues/08-research-second-reader.md)) (owner, charting Q5). [Prototype: worked case outside this repo](issues/09-prototype-worked-case-outside.md) tests transfer (owner, Q10).
- **Evidence:** the research assets of the map [the concepts the code follows](../code-concepts/map.md) (`../code-concepts/assets/`). Tickets read them and do not change them.
- **Asset shape:** each asset starts with a summary of at most ten lines; the detail follows it. The owner reads every asset.
- **Working terms** (they can change; Chronicler domain words follow `CONTEXT.md`):
  - **writer:** an agent session or a person that changes a codebase.
  - **vocabulary:** a shared file of entries, such as `CONTEXT.md` or the catalogue.
  - **entry:** one named concept in a vocabulary.
  - **gap:** a choice that no entry covers.
  - **precedent:** an earlier choice that later writers follow.
  - **force:** a pressure in the problem that a concept answers.
  - **worked case:** the framework applied to one real vocabulary of this repo. It changes nothing.
  - **concept:** open (see Open questions).
- **Skills:** `/grilling`, `/domain-modeling`, `/prototype`. `/writing-for-agents` only for the applications.

## Decisions so far

<!-- one line per closed ticket: its decision, or the sharper question it ended with -->

- [Research: what the sources say about design and language](issues/01-research-sources-design-and-language.md) — Brooks, Jackson, Evans, Naur and Alexander each give a different test for what gets a name; they split on whether shared understanding can be written down (Naur: no). All assume writers who remember and talk. Ended open: what a vocabulary does for a reader with only the text.
- [Research: what the sources say about coordination and vocabulary control](issues/02-research-sources-coordination-and-vocabulary.md) — Schelling, common law, Z39.19/ISO 25964 and ADR practice each give a different test for what earns an entry; at a gap a named body decides and new items start on probation; only that body may overrule, anyone may distinguish. Ended open: no source speaks to a writer that shares a prior but has no memory.
- [Research: does a `CONTEXT.md` entry make writers align?](issues/03-research-names-and-alignment.md) — avoided words fell in 5 terms and stayed or grew in 6; each fall came in a planned code commit, not the entry commit, and some entries followed their code. Ended open: the counts cannot show that an entry causes alignment rather than records a plan that does.
- [Research: how this repo makes writers align today](issues/04-research-ways-to-align.md) — 20 ways; only `AGENTS.md`, `.pi/APPEND_SYSTEM.md` and the compiler always reach an agent; structure and behaviour fail the build, names never do. The falls that ticket 03 counted follow plan-ordered deletions, and six names with no entry fell the same way. Ended open: no evidence of which writer met which way.

## Open questions

| Question | Status |
|---|---|
| What gets an entry? The owner's tension: "every distinct concept, even an obvious one" against a small file. The models disagree here. | [Research: what the sources say about design and language](issues/01-research-sources-design-and-language.md), [Research: what the sources say about coordination and vocabulary control](issues/02-research-sources-coordination-and-vocabulary.md), then [Prototype: draft the framework note](issues/05-prototype-framework-note.md) |
| What purposes does a vocabulary serve (align names, model the domain, teach)? Can one file serve them all? | [Prototype: draft the framework note](issues/05-prototype-framework-note.md) |
| What is a concept: a word for one thing, a way to do one thing, or both? Is that the line between `CONTEXT.md` and the catalogue? | [Prototype: draft the framework note](issues/05-prototype-framework-note.md), [Prototype: worked case, `CONTEXT.md`](issues/06-prototype-worked-case-context.md) |
| At a gap, does an agent's choice become precedent for the next writer, and who reviews it? | fog |
| When a concept exists, when may a writer choose the opposite on purpose? | fog |
| How does a project find its forces and write them down, and how do the forces relate to its concepts? | fog |
| How does a vocabulary stay current: entry, change, removal? | [Research: what the sources say about coordination and vocabulary control](issues/02-research-sources-coordination-and-vocabulary.md), then fog |
| Where does the note live at the end: in the repo, or in the owner's own notes? | fog |
| Can a vocabulary carry the shared understanding to a writer who has only the text and no one to ask? Naur says the theory cannot be written; Brooks, Evans and Alexander say it can, in part. No source in either research family tests a writer with no memory; only Nygard and Parnas and Clements write for a reader who cannot ask the author. | [Prototype: draft the framework note](issues/05-prototype-framework-note.md); evidence from [Research: does a `CONTEXT.md` entry make writers align?](issues/03-research-names-and-alignment.md) |
| Do names make writers align, or do they record a plan that makes writers align? Counts show falls only with a planned rebuild; in 6 terms the avoided word stayed or grew, and 2 entries changed to match usage. | [Research: does a `CONTEXT.md` entry make writers align?](issues/03-research-names-and-alignment.md) (ended open), then [Research: how this repo makes writers align today](issues/04-research-ways-to-align.md) |
| Which other ways make writers align here, and how do they fail? Names have one written source and no check that fails a build; structure and behaviour have many. Should names get a check, or is that the wrong layer? | [Research: how this repo makes writers align today](issues/04-research-ways-to-align.md) (ended open), then [Prototype: draft the framework note](issues/05-prototype-framework-note.md) |
| Which parts of the note transfer to a team of people and agents, and which hold only for this repo? | [Prototype: draft the framework note](issues/05-prototype-framework-note.md), then [Prototype: worked case outside this repo](issues/09-prototype-worked-case-outside.md) |
| Across several teams, where does one shared concept end and a context boundary begin? This repo has one context; the work setting has several. | [Research: what the sources say about design and language](issues/01-research-sources-design-and-language.md) (Evans: bounded context), then [Prototype: worked case outside this repo](issues/09-prototype-worked-case-outside.md) |

## Out of scope

- Changes to the repo. They are hand-offs (owner, charting Q1).
- Agent-facing forms of the note. Agents read only the applications (owner, charting Q2).
- How all of a project's theory reaches an agent: all docs, how agents read, and process (owner, charting Q3).
