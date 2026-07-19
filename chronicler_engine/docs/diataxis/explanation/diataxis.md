---
diataxis: explanation
title: Diátaxis
---

## What Diátaxis sees in documentation

Documentation serves readers who arrive with particular needs at particular moments. Three needs recur: to learn a system from zero, to look up a particular fact, to follow steps toward a result. Each calls for different writing — mixing them in one document fails all three, because the writing that serves one need is in the way for the reader with another.

Diátaxis takes this as its starting point: the reader's need — not the topic — is the unit of organisation. Two readers can arrive at the same topic with different needs, and the documentation owes them different things. Splitting by topic mixes the needs; splitting by need serves them.

From this, Diátaxis identifies four recurring needs and places four kinds of documentation in relation to each other: tutorials, how-to guides, reference, and explanation. Each responds to a different need, and each has a shape — prose, structure, register, what it omits — that follows from the need it serves.

## The four kinds

| Kind | Reader need | Orientation |
|---|---|---|
| Tutorial | Learn from zero | Learning-oriented |
| How-to guide | Achieve a goal | Goal-oriented |
| Reference | Look up a fact | Information-oriented |
| Explanation | Understand why | Understanding-oriented |

**A tutorial is a lesson.** It takes a learner by the hand through an experience in which they acquire skill by doing. The instructor is responsible for the learner's success; the learner's only obligation is to follow. What matters is what the learner *does* and what happens — not what the instructor says. A cooking lesson with a child is a tutorial: the dish produced is incidental, the encounter with knives, heat, timing, and tidying-up is the point.

**A how-to guide is a recipe.** It directs a competent practitioner toward a real-world result. The reader owns the goal; the guide owns the path. Where a tutorial assumes nothing, a how-to guide assumes competence — the reader has their skills and is at work. A clinical manual for an appendectomy is a how-to guide: it lists steps, orders them, accounts for branches, and stays out of the way.

**Reference is a map.** A technical description of the machinery, consulted rather than read. Its structure mirrors the machinery's structure, so a reader who knows the territory can find their way. Reference is austere: describe and only describe; route explanation and instruction elsewhere. A marine chart is the canonical analogy — it could be used by a navigator plotting a course or a magistrate in a legal case; the chart takes no interest in either.

**Explanation is discursive treatment.** It unfolds a subject — placing it in context, drawing connections, considering alternatives, illuminating why things are the way they are. It belongs to reflection, which means it comes *after* direct experience and looks back at it. Harold McGee's *On Food and Cooking* is the canonical example: it teaches no recipe, contains no reference data, and yet changes how a cook thinks about their craft by placing cooking in the context of history, science, and society.

## The compass

The four kinds are related by two axes. A piece of content can inform what the reader *does* (action) or what the reader *knows* (cognition). And it can serve the reader's *study* (acquisition of skill) or their *work* (application of skill).

| Content informs... | And serves... | Therefore it's... |
|---|---|---|
| Action | Acquisition (study) | Tutorial |
| Action | Application (work) | How-to guide |
| Cognition | Application (work) | Reference |
| Cognition | Acquisition (study) | Explanation |

The compass is a decision tool. Where intuition says "this is a tutorial, probably", the compass asks the two questions and yields an answer: action or cognition? acquisition or application? The intersection picks the mode.

Tutorials and how-to guides share the action axis — both contain steps for the reader to follow. They differ on acquisition versus application: a tutorial serves study, a how-to serves work. That is why a tutorial and a how-to guide *look* similar (both have numbered steps, commands, "do this, then that") and are nonetheless different documents serving different needs. The same instructions in the same order can be a tutorial in one frame and a how-to in another; the reader's state, not the prose's form, decides which.

Reference and explanation share the cognition axis — both deal in what the reader knows, not what the reader does. They differ again on acquisition versus application: reference serves work, explanation serves study. Reference is the chart the navigator consults mid-voyage; explanation is the book the navigator reads in the bath, reflecting on why charts are drawn the way they are.

## Where the boundaries blur

The four kinds pair off across the two axes, and the pairs that share an axis are the ones most often confused:

| Adjacent pair | They share | They differ on |
|---|---|---|
| Tutorial ↔ How-to | Action | Study vs Work |
| Reference ↔ Explanation | Cognition | Work vs Study |
| Tutorial ↔ Explanation | Acquisition | Action vs Cognition |
| How-to ↔ Reference | Application | Action vs Cognition |

Each blur has the same shape: two modes agree on one axis and the drift between them happens on the other. A tutorial that drifts toward how-to has started assuming competence where it should be teaching. A reference doc that drifts toward explanation has started reflecting on its machinery instead of describing it. The shared axis makes the drift invisible to the writer; the differing axis is where the correction lives — ask which signal the content is actually sending on the axis where the pair differs, and the mode resolves.

## How Diátaxis is applied

Diátaxis is applied iteratively, not as a one-time classification. The framework treats a documentation set as something living: a draft in progress, an existing doc, an empty page where something is needed. Each pass considers what is there, chooses one improvement — however small — and makes it. A draft misclassified today becomes two correctly-classified docs tomorrow, once the writing surfaces where the seams fall. The framework does not require a complete taxonomy before it starts helping.

## Document References

- [`../../AGENTS.md`](../../AGENTS.md) — the writing-convention layer for this docs tree, including the one-line compass test and the per-mode writing notes.
- [Diátaxis](https://diataxis.fr/) — Daniele Procida's site; the canonical source.
- [Diátaxis framework source](https://github.com/evildmp/diataxis-documentation-framework) — the framework's source repository.

## Source attribution

This document derives from **Daniele Procida, Diátaxis** — [diataxis.fr](https://diataxis.fr/), [github.com/evildmp/diataxis-documentation-framework](https://github.com/evildmp/diataxis-documentation-framework), CC-BY-SA-4.0. The four-kinds and compass tables appear in Procida's *Start here* and *Compass* pages. The adjacent-pair table consolidates the two-axis logic from his *Foundations* page; each same-axis pair is also discussed in its own dedicated page (*The difference between a tutorial and how-to guide*, *The difference between reference and explanation*). The analogies (cooking lesson, recipe, marine chart, Harold McGee's *On Food and Cooking*) are his.
