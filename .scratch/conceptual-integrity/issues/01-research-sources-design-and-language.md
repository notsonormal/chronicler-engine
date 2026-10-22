# Research: what the sources say about design and language

Type: research
Status: resolved

## Question

What do the sources of design and language say about the questions of this map? Read the map's Notes first (`.scratch/conceptual-integrity/map.md`): the reader is the owner, tickets may end open, and assets start with a summary of at most ten lines.

The sources, as a minimum and not a limit. Add the sources of this family that you find, and say why you added each one.

- Brooks, *The Mythical Man-Month* (1975), chapter 4, "Aristocracy, Democracy, and System Design": conceptual integrity.
- Jackson, *The Essence of Software* (2021), and his papers on concept design: concepts and purposes, the specificity principle, redundancy and overloading, familiarity.
- Evans, *Domain-Driven Design* (2003), and the free *Domain-Driven Design Reference* (2015): ubiquitous language, bounded context, and a model and a language that change together.
- Naur, "Programming as Theory Building" (1985).
- Alexander, *A Pattern Language* (1977) and *The Timeless Way of Building* (1979): forces, patterns that link to each other, and a language that generates designs.

The questions:

1. What gets a name or an entry, and what stays out?
2. What makes writers align on one concept, and what makes them diverge?
3. What happens at a gap: who names the new concept, and how does it become shared?
4. When may a writer choose the opposite of an existing concept?
5. How does a vocabulary stay current: entry, change, removal?
6. What place do the forces (the "why") have?

Write `assets/research-01-design-and-language.md` in the shape that [Research: what the sources say about coordination and vocabulary control](02-research-sources-coordination-and-vocabulary.md) also uses, so that the draft can merge the two:

- **Summary:** at most ten lines, first.
- **Per source:** for each question that the source speaks to, give the claim, a short quote with its location (chapter, section or page), and a label: *primary* (from the source itself) or *second-hand* (from a text about it). For each question that the source does not speak to, say so in one line.
- **What the source assumes about its writers:** whether they remember, whether they can talk to each other, and whether they share a prior (map Notes: writer assumptions). Quote the source where it says so, and label your inference where it does not.
- **Where the sources disagree:** each disagreement as a question, with the two positions and their locations.
- **Questions raised:** new questions that the reading gives.

Record what the sources say. Do not apply them to this repo, and do not rule on them. Label your own interpretation as known, inferred or guessed. Do not open `assets/charting-notes.md`: it holds the coordinator's second-hand summaries of these sources, and they must not steer you.

## Done when

Every listed source is read, from the source itself where it is available and from labelled second-hand texts where it is not. The asset has the shape above, and every claim has a location.

## Answer

Resolved 2026-10-22. Asset: [research 01, design and language](../assets/research-01-design-and-language.md). The researcher read Brooks, Jackson, Evans, Naur and Alexander, mostly from primary texts. The asset lists what was read second-hand or not read (Jackson's book, Brooks ch. 16–19, Evans ch. 14, Alexander's *Notes on the Synthesis of Form*).

- Each source gives a different test for what gets a name: what the user sees (Brooks), what has a purpose (Jackson), what the model needs (Evans), a real conflict of forces (Alexander). Naur gives no test.
- The sharpest disagreement: can the shared understanding be written down? Naur says no. Brooks, Evans and Alexander say yes, in part.
- A second disagreement: one unit for one purpose (Jackson), or one model for code, design and talk (Evans).
- All sources except Jackson assume writers who remember and can talk. No source describes writers with no memory (inferred).
- Ended open: no source says, with a location, what a vocabulary does for a writer who reads it fresh each time and has no one to ask. No source states who checks a choice at a gap before the next writer copies it.
