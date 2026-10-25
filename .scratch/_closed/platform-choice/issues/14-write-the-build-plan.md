# Write the build plan

Type: task
Status: resolved
Blocked by: 12, 13, 15

## Question

Write the build plan of this map's Destination into one file in `docs/plans/`. The plan takes the slices in the order of [Decide the build path](09-decide-build-path.md) (P0 to P5, then the core step) and gives each slice its content, its spec scenarios, its tests (rewritten, moved to a seam contract, or kept on the legacy list), its guardrail rules, its doc changes, and its gate. An implementer can start each slice with nothing left to decide. This is AFK work: it assembles decisions and makes none. A gap that needs a decision goes back to the owner as a new ticket.

## Context

- The path, the branches, the red-time rule and the rule for other work: [Decide the build path](09-decide-build-path.md).
- The Job shape, gaps G1 to G3, rules 17 and the `SavedText` change: [Decide the shape of the Job and the busy state](05-decide-job-and-busy-state-shape.md).
- The test rule, the legacy list and the seam contracts: [Decide the test rule outside the designed scope](08-decide-test-rule-outside-designed-scope.md).
- The storage seam and the 17 constraints: [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md), [research 10](../assets/research-10-storage-constraints.md).
- The dev loop (`build.py test-pattern` target selection): [Prototype: decide the crate split and the dev-build loop](07-prototype-crate-split-and-dev-build-loop.md). The plan puts this `build.py` change into P0 or before it.
- The race tests and rule 18: [Decide the race tests and guardrail rule 18](12-decide-race-tests-and-rule-18.md).
- The guardrails, docs and module shape: [Decide the rebuilt repository's module shape, docs and guardrails for agents](13-decide-module-shape-docs-and-guardrails.md).
- The edges and the later efforts: [research 04](../assets/research-04-architecture-deepening-sort.md).
- The shape on one page: [The rebuild at a glance](../assets/shape-at-a-glance.md). The plan opens with it.
- The design: [target design](../../turn-state-model/assets/target-design.md) version 2, [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) (§7 fate tables), [sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md).

## Done when

The plan file exists in `docs/plans/`, passes `python build.py validate-docs`, and gives each slice the items above. The owner has read it. The build map (one `task` ticket per slice, Notes that override "plan, don't do") is a next step, not part of this ticket.

## Comments

**2026-10-17, from [Decide the race tests and guardrail rule 18](12-decide-race-tests-and-rule-18.md) (resolved).** Its answer has the scenario and test list with a slice for each item (P4: the holdable fake, the 12.1 and 15.1 rewrites, I1 for Action and `/options`; core step: I2 to I6 and rule 18), and notes on `wait_idle` and the legacy `actions_tests.rs` busy test.

**2026-10-17, from [Decide the rebuilt repository's module shape, docs and guardrails for agents](13-decide-module-shape-docs-and-guardrails.md) (resolved).** Its answer has a rule-by-rule table with a slice for each of the 18 rules (§1). It also gives the fate of today's 23 guardrails (§3), the doc fates (§4), and the build-plan notes (§6). P0 takes the four table-driven walkers, the `generate_guardrails_doc.py` change, the dead-path assertion and the `registry_tests.rs` rework. The row of rule 3 names ticket 11's `transaction` and `transaction_new_game`. This ticket is now unblocked.

**2026-10-17, from [Review: the design and the build decisions against standard engineering practice](15-review-design-against-standard-practice.md) (resolved).** Its rulings amend tickets 08, 09, 11 and 13, and they win over those tickets where the two conflict. For the plan:
- 14 rules, not 18.
- Tier 1 runs on SQLite `:memory:`.
- Plain unit tests are allowed on pure `domain/model/` code, and there is no count pin.
- The core step names its red window.
- The target tests (I2 to I6 and the R-59 scenario) come before the core branch opens.

This ticket is now unblocked.

**2026-10-17, plan written (claimed, not resolved).** The plan is [`docs/plans/rust-rebuild-build-plan.md`](../../../../docs/plans/rust-rebuild-build-plan.md). It passes `python build.py validate-docs` (plans are exempt from the doc rules, known). It found four points that no ticket decides. They went to the owner as [Decide the four gaps that the build plan found](16-decide-the-build-plan-gaps.md), and the plan marks each one **Open: ticket 16, gap N**:
1. P5 against ticket 15 Q4: the `Tx` with no in-memory arm, today's Game writes, and where tier 1 switches to SQLite `:memory:`.
2. Where the red target tests (I2 to I6, R-59) live before the core branch opens, with `main` gated green.
3. Which slice makes the tier-1 fake strict and runs the real Quantifier (ticket 26 R6).
4. How the storage seam contract gets the rule-3 capability token.

Also found: ticket 15's "14 rules" is 15 by its own list (inferred count error). Target design D-34 limits P4 to the analysis and Options hooks, so the plan builds no `save_rewrite`. This ticket resolves when the owner has read the plan and ticket 16 is resolved.

**2026-10-17, from [Decide the four gaps that the build plan found](16-decide-the-build-plan-gaps.md) (resolved).** The owner took option A for all four gaps and confirmed 15 rules. The plan is changed in place and has no **Open** mark left. This ticket resolves when the owner has read the plan.

**2026-10-17, owner review of the plan.**
- The plan was rewritten with `/writing-for-agents`: a reading path for each slice, one checkable **done** criterion in §2, a **Done when** list per slice, the one rule in §2, the rule index in §2 in place of a duplicate §10, no history and no cached counts.
- **The commit gate** (owner): `/simple-after-plan` before every commit. The plan points to the skill and lists only the exceptions per commit (option E, in place of copying the steps into the plan): a step with empty input is skipped; C4 stays a separate red commit with `clippy` plus the tests it does not break in place of the full gate; C7 (docs only) skips the comment cleanup and the code review; the retro runs once per preparation slice and once at the end of the core branch. Amends ticket 09 Q7, Q8.

**2026-10-17, independent review of the plan.** A `reviewer` agent checked the plan against the sources and the code: [report](../assets/plan-check-report.md), 3 blockers, 19 should-fix, 16 nits. The coordinator verified the three blockers in the code. The text fixes that the sources decide are applied in the plan. Nine items need a decision and went to the owner as [Decide the nine gaps that the review of the build plan found](17-decide-the-plan-review-gaps.md). This ticket now also waits on ticket 17.

**2026-10-17, ticket 17 resolved.** The owner chose option A for all nine review gaps. The plan carries the rulings in place and has no open mark left. The remaining condition of this ticket is that the owner reads the plan.

**2026-10-17, plan review (`/improve-ai-plan`).** Owner rulings: (1) the strict, holdable tier-1 fake is a new test-only type in `src/test_support/`; `MockBackend` stays the lenient runtime provider for tier 3 and dev runs; (2) the first commit of P4 is the strict fake with the real Quantifier on today's `AgentRegistry`, before the Agent edge. Fixes the sources already decide: P4 keeps `LlmCallRecorder` (call-record seam case, ticket 26 R5); scenario 3.3's 200 ms delay becomes a hold; P5 records the `integration` time around the SQLite switch; `job.rs` keeps step-level `#[instrument]` logging.

## Answer

**2026-10-17. The build plan is [`docs/plans/rust-rebuild-build-plan.md`](../../../../docs/plans/rust-rebuild-build-plan.md). The owner has read it.**

- It opens with a reading path, the design sources in order of authority, and the sketch text that later rulings changed. It points to [the rebuild at a glance](../assets/shape-at-a-glance.md) as the first page.
- §2 holds the rules for every slice: order, the one unit-test rule, the test fates, the 15 guardrail rules by slice, docs, the dev loop, the commit gate (`/simple-after-plan` with per-commit exceptions), and a six-item "done" definition.
- §3 to §8 give P0 to P5, and §9 gives the core step (six areas, the target tests I2 to I6 and R-59, commits C1 to C7 with C4 as the one red commit). Each has its content, spec scenarios, tests, guardrail rules, docs and a "Done when" list.
- The gaps went to the owner: [ticket 16](16-decide-the-build-plan-gaps.md) (four gaps from writing the plan) and [ticket 17](17-decide-the-plan-review-gaps.md) (nine gaps from the independent review, [report](../assets/plan-check-report.md)). The `/improve-ai-plan` review added two rulings (a separate strict tier-1 fake; the strict fake first in P4). No open mark is left.
- Precondition: the owner merges `dashboard-ui-review-phase3` to `main` before P0.
- `python build.py validate-docs` passes. It exempts `docs/plans/`, so it proves little.

Next step: the build map, with one `task` ticket per slice (ticket 09 Q9).
