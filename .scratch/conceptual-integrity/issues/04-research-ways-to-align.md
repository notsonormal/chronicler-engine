# Research: how this repo makes writers align today

Type: research
Status: resolved

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

## Answer

Resolved 2026-10-22. Asset: [research 04, ways to align](../assets/research-04-ways-to-align.md). The researcher found 20 ways: the 9 of the minimum list and 11 more (plans and tickets, the pre-commit hook, the `build.py` gate, feature specs, `tests/STRATEGY.md`, Diátaxis docs and DOC anchors, the compiler and clippy, the permission config, the delegate roles, `.pi/APPEND_SYSTEM.md`, retro and incident rules). The retro and incident row is partly guessed.

- Only 3 ways always reach an agent session: `AGENTS.md` with its generated Structure block, `.pi/APPEND_SYSTEM.md`, and the compiler.
- Structure and behaviour have many ways that fail a build (arch-lint, walker rows, `CASE` and `SCENARIO` tags, DOC anchors, clippy). Names have one written source, `CONTEXT.md`, and no way that fails a build. `CODING_STANDARDS.md:7` points symbol names at `docs/`, not at `CONTEXT.md`.
- Ways fail in four recorded forms: stale text, a writer who bypasses the way, a way that sees less than its rule needs, and a rule left to review only.
- On the open question of [Research: does a `CONTEXT.md` entry make writers align?](03-research-names-and-alignment.md): the falls of Snapshot, Action Pipeline and Engine commit follow the deletion of code that a plan ordered (`docs/plans/rust-rebuild-build-plan.md:299`, commit `36a1e1c7`). The `CONTEXT.md` change was itself a plan step (`:381`). Six names with no entry fell in the same commit. Review is the only way found that edits an avoided word as a word, with small effect.
- Ended open: git shows no read, so no evidence says which writer met which way. The repo has no control with a glossary but no plan.
