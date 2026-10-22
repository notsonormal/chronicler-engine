# Research: how this repo makes writers align today

Type: research
Status: open

## Question

Which ways does this repo use to make writers (agent sessions and people) align on one design? For each way: what does it align, how does a writer meet it, and how does it fail? Read the map's Notes first (`.scratch/conceptual-integrity/map.md`). Record with evidence. Do not judge.

The ways to check, as a minimum. Add the ways that you find.

- `CONTEXT.md`
- `CODING_STANDARDS.md` and `AGENTS.md`
- the skills in `.agents/skills/`, for example `code-review`, `codebase-design`, `domain-modeling` and `writing-for-agents`
- `arch-lint.toml`
- the guardrail and architecture tests (`tests/infrastructure/`)
- the seam contracts and their `CASE` tags
- the generated indexes
- review
- precedent in the code: a writer copies the code that it reads

For each way, record:

- **What it aligns:** names, concepts, structure or behaviour.
- **Which writers it reaches:** agent sessions, people, or both (map Notes: writer assumptions).
- **How a writer meets it:** always loaded, on a trigger, at build time, at review, or by chance.
- **How it fails:** it goes stale, a writer does not meet it, or it is too weak or too strong. Give evidence (`path:line` or a commit).

Use the research assets of the map [the concepts the code follows](../../code-concepts/map.md) where they cover a way, especially `research-06-composition-and-cross-cutting.md`, `research-07-test-infrastructure.md`, `research-10-synthesis.md` and `research-10-docs-vs-code.md` in `../../code-concepts/assets/`. Cite them, and do not repeat their detail.

Write `assets/research-04-ways-to-align.md`:

- **Summary:** at most ten lines, first.
- **One row per way:** the way, what it aligns, how a writer meets it, how it fails, evidence, and a label (known, inferred or guessed).
- **Meeting points between the ways:** places where two ways align the same thing, or pull in different directions, each written as a question.
- **Questions raised.**

## Done when

Every way in the minimum list has a row with evidence, and the asset names the ways that it added.
