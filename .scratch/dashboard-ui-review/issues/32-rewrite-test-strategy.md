# Rewrite the test strategy around one checkable tier-1 rule

Type: task (HITL)
Status: open
Blocked by: 31

## Question

How should `tests/STRATEGY.md` read, so that an agent placing a test, or writing a scenario, takes the same path every run?

## Context

- The user is not confident in the current rules. Vague terms carry the load: "behavioural authority", "client-observable", "curl-observable". Read `.agents/skills/writing-for-agents/SKILL.md` before writing. The audit points at these levers:
  - **Leading word.** Replace the loose terms with the one term that [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) chooses, and use it throughout.
  - **Completion criteria.** "Spec completeness is mandatory … every failure mode, every edge case" and "delete the weaker one" have no clear point where they are done. Make each one checkable, or reword it as guidance.
  - **One map of the tree.** The tier table has no row for `tests/bootstrap`, `tests/infrastructure`, or `tests/llm`. `tests/AGENTS.md` sorts tests by binary, and the standards docs sort them by fixture pattern. One table should place every `tests/` directory.
  - **Negation.** Prefer stating the target over listing bans.
- The rules that nothing enforces today should say so, or gain a check. Examples: unit "every branch gets a test", the XSS assertion rule, and the storage backend-pair "character-for-character identical" rule. Coverage-% targets are out of scope, by precedent from the `test-strategy-execution` map. [unverified list: [docs_drift.md](../assets/test-audit/docs_drift.md), Part 3]
- Keep what works. The UI placement rule ("The rule, in order", tie-breaker "file down") placed every browser test correctly in the audit.
- The tier-1 decision is settled ([Decide what a tier-1 test may observe](31-decide-tier-1-observations.md)): the leading word is **domain outcome**; a read seam is an HTTP GET or a method on an application read service/port; storage is not a read surface; the rule judges helper bodies too; a spec Given/Then uses `CONTEXT.md` terms. The 58 hard-leak scenarios reword on touch; the 45 raw field reads and the storage observation reads move now in [Migrate tier-1 tests to observe through legal read seams](61-migrate-tier-1-test-reads.md).
- Update the **Tests** bullet in this map's Notes if the rule it cites changes.

## Done when

- `tests/STRATEGY.md` uses one term for the tier-1 boundary, and has no internal contradiction.
- Every directory under `tests/` has exactly one row in the tier table.
- Every rule is either checkable or marked as review-only.
- `python build.py validate-docs` passes. The user reviews the diff. Commit after approval.
