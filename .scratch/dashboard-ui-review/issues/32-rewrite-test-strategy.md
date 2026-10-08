# Rewrite the test strategy around one checkable tier-1 rule

Type: task (HITL)
Status: resolved
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

## Answer

`tests/STRATEGY.md` is rewritten around one leading word, **domain outcome**. The user approved the diff. The change is not committed yet.

- **Term.** A new section, "Domain outcome (the tier-1 rule)", states the decision from [Decide what a tier-1 test may observe](_resolved/31-decide-tier-1-observations.md). A domain outcome is the HTTP response, or stored state read back through a read seam. A read seam is an HTTP GET, or a method on `GameViewQuery`, `MessageService`, `GameCatalogue`, `SettingsService`, `PromptPresetService` or `WorldCatalogue`. The section has a test that a reviewer applies to one line: the line reads the HTTP response or calls a read seam, or it dereferences a `GameState` field or calls `Storage`. It also covers helpers (the rule judges the helper body), arrange steps (seeding through `Storage` is allowed), waits (`wait_idle` may read state) and persistence facts that have no read seam (they belong at the driven-adapter or unit tier). The UI placement rule's step 1 now asks "Is the outcome a domain outcome?". The terms "behavioural authority", "client-observable" and "curl" are gone.
- **One map.** The tier table has one row per directory: the 13 directories under `tests/` and `src/**/*_tests.rs`. The bold rows are the tiers. The other rows are support code or checks outside the tier model. `tests/AGENTS.md` now points at this table instead of listing binaries.
- **Checks.** Every rule ends with a `Check:` line. `spec-coverage` checks the SCENARIO-tag rules, and `guardrails` checks the htmx-settle rule. These rules are review-only: unit "a test for each branch", the domain outcome rule, spec vocabulary, markup, complete specs, same-tier overlap, UI placement, the stub tax and `networkidle`.
- **Rules with an end point.**
  - "Every failure mode, every edge case" is now: a spec covers each distinct response of its route (each success shape, each refusal, each error status).
  - "Delete the weaker one" is now: keep the test whose failure names the regression more precisely.
- **Facts corrected while rewriting.**
  - The htmx-settle guardrail selects files by path (`tests/browser/` outside `stub/`), not by `with_test_page`.
  - The dashboard has more than five pollers.
  - The `tests/AGENTS.md` seam recipe now uses `GameViewQuery::list_latest_llm_messages`, not `Storage`.
- **Not done.**
  - The specs were not reworded. Scenarios that name Rust identifiers are reworded when their spec file is next edited.
  - The unit-standards rules (XSS assertions, backend-pair identity) stay as they are in `unit_test_standards.md`. Ticket 33 owns that document.
  - The tier-2 claim "a fresh page per test" is not verified.

**Possible follow-ups.** No ticket is open for these yet:
- a guardrail that fails when a `tests/` directory has no row in the tier table;
- a grep check for `GameState` field reads in `tests/http/` (it starts green after [Migrate tier-1 tests to observe through legal read seams](_resolved/61-migrate-tier-1-test-reads.md));
- the "validator check for spec prose" item in the map's Not yet specified.
