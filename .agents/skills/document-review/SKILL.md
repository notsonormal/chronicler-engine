---
name: document-review
description: "Review markdown docs top-down: doc set, document, section, sentence. Use when reviewing any document or .md file."
---

# Document review

Review from the top down: the doc set, then the document, then its sections and paragraphs, then its sentences. A verdict at a higher level settles the levels below it. A doc that should merge into another gets no wording review. A section that should go gets no accuracy check.

At every level, ask whether the text should exist before you ask whether it is correct or well written.

Existing text earns no credit for being there. Nearby docs are not a model of a good doc: they can be as stale as the text under review. Judge each doc and section as if it were new.

## Scope

- **A whole file:** review all of it.
- **A diff:** the diff is where the review starts, not where it stops. Also review the document that contains the change, the section around it, and every other doc that covers the same concept. Report problems the change did not cause under **Pre-existing**, so the author can tell them apart.
- Skip `.scratch/`, `docs/plans/` and `old-docs/`. Those are transient documents.

## Doc types

Each doc type earns its place by a different test. Find the type of each file in scope first.

| Type | What it is for | It earns its place when | Also load |
|---|---|---|---|
| `docs/diataxis/` | Explanation and reference the code, tests and specs cannot carry | It carries purpose, invariants or a frame, in one Diátaxis mode | `.agents/skills/chronicler-docs-hygiene/SKILL.md`; background in `.agents/skills/diataxis-doc-review/SKILL.md` |
| `docs/specs/` | Behaviour a client observes over HTTP or in the browser | Each scenario is in domain terms and a tagged integration test covers it | `tests/STRATEGY.md` |
| A skill, `AGENTS.md`, `CLAUDE.md` | Steering an agent | Each line changes the agent's behaviour against the default | `.agents/skills/writing-for-agents/SKILL.md` |
| `CONTEXT.md` | The domain glossary | Each term is one the domain uses and no other term already names | `.agents/skills/domain-modeling` |
| Any other `.md` (`README.md`, `ENVIRONMENT.md`, `CODING_STANDARDS.md`, `docs/external_applications/`) | The reader need its title names | Each part serves that need, and no other doc already serves it | — |

`docs/AGENTS.md` §Documentation Layers sets which layer owns what. Code and unit tests come first, then specs and integration tests, then the Diátaxis docs.

## 1. Doc set

- **Layer.** Does the content sit in the layer that owns it? Behaviour a client sees belongs in a spec. Behaviour only the code defines stays in the code.
- **Duplication.** Does another doc already say it? Search `docs/`, the root `.md` files and the skills for the key terms.
- **Staleness caused elsewhere.** Does the change make another doc false or incomplete? Look for docs, DOC anchors (`//! [DOC: …]`) and notes that describe the same behaviour.

**Completion:** every concept in scope searched across the doc set, and each hit judged.

## 2. Document

For each document in scope:

- Would you write this doc from scratch today?
- What does it carry that the code, tests and specs cannot?
- Does its value stay above its maintenance cost? A line of doc costs more to keep true than a line of code.
- Is it one Diátaxis mode (`docs/diataxis/explanation/diataxis.md`)? Do its headings still describe what sits under them?

Verdict per document: keep, rewrite, merge into another doc, or delete.

**Completion:** every document in scope has a verdict and a reason.

## 3. Sections and paragraphs

Classify every section and paragraph in scope as KEEP or DELETE. For specs and agent docs, use the test in the Doc types table. For `docs/diataxis/`, first name the kind of fact the unit states, then look up where that kind of fact lives (`docs/AGENTS.md` §Where a fact lives):

| The unit states… | It lives in |
|---|---|
| The purpose of a part, its name, or an invariant: an always-true relation between named parts — what owns what, what restores what, how many of a part exist, where a part shows | Diátaxis Reference |
| Why the system is that way | Diátaxis Explanation |
| An event rule ("when X, then Y") at feature level, or a property that every request to one route keeps | A spec scenario, spec section intro, or spec Properties section |
| Fine-grained behaviour (a rule inside one part, such as how a function reads its input), edge cases, values, copy text, icons, markup, field lists, steps | The code and its tests |

KEEP a unit when its kind of fact lives in this doc's mode and no other doc on the same subject already states it. DELETE it when its kind of fact lives elsewhere, or when a doc on the same subject already states it. A Reference doc and its Explanation doc share a subject: the Reference doc states the fact, and the Explanation doc gives only the reason. A doc on another subject may restate a fact in one sentence when its reader needs it. A unit that mixes kinds keeps its Reference or Explanation part and loses the rest, values included.

Carriers do not cross kinds:

- A spec scenario that pins one consequence of an invariant does not carry the invariant. "Settings is a singleton row" stays in Reference although a test pins the single row.
- A code comment, a type, a glossary entry or seed data does not carry an invariant for the reader of the doc. The doc is where a reader finds what owns what without reading every file.
- In an Explanation doc, the code carries *what* happens, not *why*. Delete an Explanation unit only when another doc already gives the same reason. A reason that is tied to one value (one colour stop, one delay) belongs in a comment beside that value.

The default is DELETE. A doc that earns its place does not make each of its paragraphs earn theirs. "It is accurate" and "it matches the nearby text" are not reasons to keep.

A paragraph that needs a qualifier to match the code ("when the provider reports them", "only on the narration path") restates the code. Its verdict is DELETE, not a more accurate rewrite.

Document References entries are not units. Keep a link only to the paired Reference or Explanation doc of the same subject, or to a doc that the body hands a point to. DELETE every spec link and every link to a doc that is only related. The index in `docs/AGENTS.md` lists every doc.

**Completion:** every section and paragraph in scope has a verdict and a one-line reason.

## 4. Accuracy

On KEEP text only:

- `docs/diataxis/`: run `chronicler-docs-hygiene`.
- `docs/specs/`: check each scenario against `tests/STRATEGY.md` and its tagged test.
- Everywhere: every symbol, path, command and count is real at this commit.

**Completion:** every claim in KEEP text checked against its source.

## 5. Wording

On KEEP text only, apply `.agents/skills/unslop/SKILL.md` and this checklist from `.agents/skills/technical-writing/SKILL.md`:

- One thought per sentence. Split a sentence longer than about 25 words.
- Put the condition before the instruction or fact it guards.
- Put "only" and "not" next to the word they change.
- Every "it", "this" and "they" points at one noun. Repeat the noun when in doubt.
- One name for one thing, across the doc set and `CONTEXT.md`.
- Periods, not semicolons or em dashes.
- Leave an unchanged sentence as it is. Rewording churn costs a reader the same as a new sentence.

**Completion:** every KEEP sentence checked against each rule.

## Worked examples

| Level | Finding | Verdict |
|---|---|---|
| Doc set | A change removes a fallback in the engine. A doc under `docs/external_applications/` still says the engine uses it. | Stale. Fix the other doc in the same change. |
| Doc set | A change moves a rule's description to a new doc, but a source file's DOC anchor still points at the old doc. | Stale anchor. Point it at the doc that carries the rule. |
| Paragraph | "Settings occupy a singleton row in the `settings` table — the engine's only settings read source." (`storage.md`) | KEEP. An invariant. The schema and the bootstrap code each show part of it, but no one place states it. |
| Paragraph | "`message_swipes.snapshot_id` … **not a SQL FK**, deliberately." (`storage.md`) | KEEP. The schema shows the missing FK. Only the doc says it is deliberate. |
| Paragraph | "`llm_messages` is pruned automatically to a fixed row cap." (`storage.md`) | KEEP. It names the rule, not the number. A version that states the cap loses the number. |
| Paragraph | A bulleted list of the fields on `GameState`. | DELETE. Field lists live in the code. |
| Paragraph | A Reference paragraph: "When the player clicks Save, the form closes and the list shows the new row." | DELETE. An event rule. A spec scenario carries it, or the code does. |

## Report

```
Counts: documents N (kept, rewrite, merge, delete), paragraphs N (kept K, deleted D), wording fixes R

# Doc set
- FILE — finding — fix

# Documents
- FILE — verdict — reason

# Delete
- FILE:LINE — which KEEP part fails, and what carries the content instead

# Fix (KEEP text only)
- FILE:LINE — Error | Warning | Info — step 4 or 5 — evidence
  Current: (snippet)
  New: (rewrite)

# Pre-existing (diff reviews only)
- (same shapes as above, for problems the change did not cause)
```

Report the higher levels first. A diff review that deletes nothing from new text says why each new paragraph passed.
