# Agent Process Docs

How to work in this repository. Each doc below owns one branch of the work; the table names the branch that reaches it. The always-loaded `AGENTS.md` router carries a terser version of this table.

## This folder

| Doc | Holds | Reach it when |
| --- | --- | --- |
| [`development-loop.md`](development-loop.md) | Communication, decision-making and test-first norms; the build loop, command reference, final validation, concurrent builds. | Building, testing, iterating, or deciding how to proceed. |
| [`permissions.md`](permissions.md) | The permission system and its approval boundaries. | Touching git, config, or any permission-gated action. |
| [`delegation.md`](delegation.md) | Subagent sizing and model-specific rules. | Delegating work to a subagent. |
| [`documentation.md`](documentation.md) | The documentation index and the generated-index/pre-commit rules. | Writing docs or making a commit. |

## Neighbouring docs

- `tests/AGENTS.md` — integration test structure, tiers, and the failure-handling protocol.
- `tests/STRATEGY.md` — the normative test-placement rule.
- `CODING_STANDARDS.md` — router for the coding rules: implementation, code comments, code reviews, tests.
- `CONTEXT.md` — the domain glossary; use its vocabulary.
- `ENVIRONMENT.md` — build limits, target seeding, sccache, and build diagnostics.
- `STRUCTURE.md` — the generated source-tree index.
