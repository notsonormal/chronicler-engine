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

Classify every section and paragraph in scope as KEEP or DELETE. Keep one only when **both** hold:

1. **It carries purpose, an invariant, or a frame.** It says what a part is *for*, a rule that holds across the system, or a way to view the system that keeps its parts apart. For specs and agent docs, use the test in the Doc types table.
2. **Nothing else carries it.** The code, a test, a spec scenario or another doc does not already say it. One hop into the source counts as "already says it".

The default is DELETE. A doc that earns its place does not make each of its paragraphs earn theirs. "It is accurate" and "it matches the nearby text" are not reasons to keep.

Edge cases, error handling, ordering rules, field lists and UI details live in the code, and the specs pin the ones a client sees (`docs/AGENTS.md` §Reference defers to source).

A paragraph that needs a qualifier to match the code ("when the provider reports them", "only on the narration path") restates the code. Its verdict is DELETE, not a more accurate rewrite.

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
| Doc set | Ticket 08 removed the reasoning-field fallback. `docs/external_applications/marinara_engine.md` still says the engine reads those JSON fields. | Stale. Fix the other doc in the same change. |
| Doc set | A change moves a rule's description to a new doc, but a source file's DOC anchor still points at the old doc. | Stale anchor. Point it at the doc that carries the rule. |
| Paragraph | "**Only `content` is the answer.** The transport reads one field of a reply … If `finish_reason` is `"length"`, the failure reports a spent `max_tokens` budget …" (`narration_system.md`) | DELETE. `parse_chat_response` and its tests carry the rule. Scenario 38.7 carries the user-facing line. |
| Paragraph | "Settings is a singleton row." | KEEP. An invariant that no symbol or type states. |
| Paragraph | A bulleted list of the fields on `GameState`. | DELETE. The struct carries it. |

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
