# Research: what the sources say about coordination and vocabulary control

Type: research
Status: resolved

## Question

What do the sources of coordination and vocabulary control say about the questions of this map? Read the map's Notes first (`.scratch/conceptual-integrity/map.md`): the reader is the owner, tickets may end open, and assets start with a summary of at most ten lines.

The sources, as a minimum and not a limit. Add the sources of this family that you find, and say why you added each one.

- Schelling, *The Strategy of Conflict* (1960), chapter 3: focal points and tacit coordination.
- The common-law method: precedent and stare decisis; distinguishing and overruling a case; splits between courts, and how a top court chooses which splits to resolve; the Restatements of the Law (American Law Institute) as a description of what courts do.
- Controlled vocabularies: ANSI/NISO Z39.19 and ISO 25964. Preferred and non-preferred terms, literary warrant and user warrant, scope notes, and how a vocabulary adds, changes and removes terms.
- Design-rationale practice: architecture decision records (Nygard, "Documenting Architecture Decisions", 2011), and the test for an ADR in this repo (`.agents/skills/domain-modeling/SKILL.md`, "Offer ADRs sparingly": hard to reverse, surprising without context, the result of a real trade-off).

The questions:

1. What gets a name or an entry, and what stays out?
2. What makes writers align on one concept, and what makes them diverge?
3. What happens at a gap: who names the new concept, and how does it become shared?
4. When may a writer choose the opposite of an existing concept?
5. How does a vocabulary stay current: entry, change, removal?
6. What place do the forces (the "why") have?

Write `assets/research-02-coordination-and-vocabulary-control.md` in the shape that [Research: what the sources say about design and language](01-research-sources-design-and-language.md) also uses, so that the draft can merge the two:

- **Summary:** at most ten lines, first.
- **Per source:** for each question that the source speaks to, give the claim, a short quote with its location (chapter, section, clause or page), and a label: *primary* (from the source itself) or *second-hand* (from a text about it). For each question that the source does not speak to, say so in one line.
- **What the source assumes about its writers:** whether they remember, whether they can talk to each other, and whether they share a prior (map Notes: writer assumptions). Quote the source where it says so, and label your inference where it does not.
- **Where the sources disagree:** each disagreement as a question, with the two positions and their locations.
- **Questions raised:** new questions that the reading gives.

Record what the sources say. Do not apply them to this repo, and do not rule on them. Label your own interpretation as known, inferred or guessed. Do not open `assets/charting-notes.md`: it holds the coordinator's second-hand summaries of these sources, and they must not steer you.

## Done when

Every listed source is read, from the source itself where it is available and from labelled second-hand texts where it is not. The asset has the shape above, and every claim has a location.

## Answer

Resolved 2026-10-22. Asset: [research 02, coordination and vocabulary control](../assets/research-02-coordination-and-vocabulary-control.md). The researcher read Schelling ch. 3, US Supreme Court texts on precedent, the 1966 House of Lords Practice Statement, the ALI texts on the Restatements, Z39.19, Nygard, Parnas and Clements, and the repo's `domain-modeling/SKILL.md` and `ADR-FORMAT.md` as source texts. The body of ISO 25964-1 is paywalled; its clauses rest on labelled second-hand texts. No Restatement text was read.

- Each source gives a different test for what gets an entry: one form per distinct concept, chosen by warrant (Z39.19); only the question before the court, and only important splits (Rule 10); only significant decisions (Nygard, the repo skill); every rejected alternative (Parnas and Clements).
- At a gap, a named body decides in every source, and the new item first gets a probation status (candidate, proposed, draft). In common law, the lower courts act first and the top court waits ("percolation").
- To choose the opposite: only a named body may overrule; anyone may distinguish. ALI: depart "explicitly and explain why". Schelling: without talk, a choice that is not focal is "not really 'available'".
- Six disagreements: follow usage or lead it; settled or right; prominence or prescription; reasons inside or beside the entry; what of the past stays visible; what earns a record.
- All sources assume writers who can read the record. Only Nygard and Parnas and Clements write for a reader who cannot ask the author. Ended open: no source speaks to a writer that shares a prior but has no memory.
