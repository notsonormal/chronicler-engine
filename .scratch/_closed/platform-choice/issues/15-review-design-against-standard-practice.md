# Review: the design and the build decisions against standard engineering practice

Type: research
Status: resolved
Blocked by: —

## Question

The design and the build path came out of many grilling rounds. Each answer was reasonable on its own. Does the whole still match core engineering principles and standard designs and architectures? Where does it differ from the standard way, and does a stated requirement justify each difference? The owner knows of no requirement that forces the design to be unusual.

## Context

- **Origin.** The owner, 2026-10-17, after [Decide the rebuilt repository's module shape, docs and guardrails for agents](13-decide-module-shape-docs-and-guardrails.md): "Sometimes, we can get caught up in the details and in the Q/A that the final design kind of becomes a bit strange."
- **The shape on one page:** [The rebuild at a glance](../assets/shape-at-a-glance.md).
- **Method.** Three independent read-only reviewers from three model families, none of whom took part in the grilling. The coordinator merges their findings. The owner decides each deviation.
- **Scope.** The design ([target design](../../turn-state-model/assets/target-design.md) version 2, [sketch 25](../../turn-state-model/assets/sketch-25-new-design.md), [sketch 24](../../turn-state-model/assets/sketch-24-agent-hooks.md)) and this map's decisions (tickets 05, 07, 08, 09, 11, 12, 13). Not the language choice (out of scope).

## Done when

- A list of the deviations from standard practice, each with the standard it differs from, the requirement that justifies it (or "none found"), and a recommendation: keep, simplify, or replace.
- The owner rules on each deviation that a reviewer recommends to change. A ruling that changes a decision reopens or amends that decision's ticket.
- [Write the build plan](14-write-the-build-plan.md) waits on this ticket.

## Comments

**2026-10-17, the three reports.** [rev-deepseek](../assets/review-15/review-15-rev-deepseek.md), [rev-glm](../assets/review-15/review-15-rev-glm.md), [rev-kimi](../assets/review-15/review-15-rev-kimi.md) (copied from `tmp/`).

All three reviewers say the runtime design is standard: the state on each Swipe, memory-only busy with a job id check (a fencing token), one mutex that is never held across an LLM call, the unit of work on `Tx`, `JobPlan` plus one runner, `SavedText`, the hook registry, and `CurrentGame`. The deviations are in the things around the design.

| # | Finding | DS | GLM | Kimi | Checked by the coordinator |
|---|---|---|---|---|---|
| F1 | The core step is a big-bang rewrite. Split it into green merges | high | med | high | **known:** ticket 09 cites sketch 25 §2.2 for refusing a throw-away adapter, but §2.2 is about not twinning today's modules. It does not mention a migration adapter. P4 already accepts throw-away glue |
| F2 | Allow plain unit tests on pure domain functions | med | high | high | **known:** ticket 08 R2: "A pure domain function gets no contract" and so no unit test |
| F3 | Drop the unit-test count pin | med | med | — | **known:** ticket 08 R4 |
| F4 | Trim the rules that guard other custom choices (7, 8, 10, 17, the indirect half of 18) | yes | partly | yes | — |
| F5 | Run tier 1 on SQLite `:memory:`, not on the hand-written in-memory backend | med | — | — | **known:** tier 1 defaults to `Storage::new_in_memory()` (`test_app_builder.rs:137`); `DbPool` is one `Mutex<Connection>` and `DbPool::new(":memory:")` already works (`db.rs:12`, `fixtures.rs:613`) |
| F6 | Rule 3 as a capability token (compiler), not a name-matched walker | low | — | — | — |
| F7 | Minority, against the other two: a persisted job row, a `StorageBackend` trait, `generate` as a state machine | — | cleared | proposed | ticket 05 measured the state machine (prototype 02); R-92 and D-05 decide memory-only busy |

**2026-10-17, owner rulings, round 1.**

- **Q2 = C (F2, F3).** A pure function in `domain/model/` may have plain unit tests, with no seam-contract form. The count pin goes. The allowlist of where unit tests may live stays. HTTP scenarios stay the main behaviour net, and seam contracts stay for the ports. Amends [ticket 08](08-decide-test-rule-outside-designed-scope.md) R2 and R4, and turn-state-model ticket 26 R2.
- **Q3 = accept all (F4, F6).**
  - Rule 7 keeps only its field half: a `Job` holds no record field. The parameter half goes.
  - Rule 8 goes. One tier-1 scenario for R-59 takes its place: the preset changes between the Narration and the event Narration.
  - Rule 10 goes.
  - Rule 17 goes. The status test catches a missing step name.
  - Rule 18 keeps the direct count only. The indirect half becomes a convention in `gate.rs`: a helper takes the guarded data as `&mut` and does not lock.
  - Rule 3 becomes a capability token: `Storage::transaction*` takes a token that only `gate.rs` can build. It moves to the compiler level.
  - Result: 14 rules. The syn rows are 1, 2, 6, 7 (field half), 13 and 18 (direct).

  Amends [ticket 13](13-decide-module-shape-docs-and-guardrails.md) §1 and §2.
- **Q4 = B (F5).** Tier 1 runs on SQLite `:memory:`. The new records (`stories`, `docks`, `Tx`) get no in-memory arm. The in-memory backend keeps today's kept methods. The copy-and-swap `Tx` (ticket 11 R3) goes. The typed `TxWrite` key (R4) stays: `BackendKind::Test` wraps either backend (known, `storage/core.rs:28`), so failure injection works on SQLite. The storage seam contract for the new records runs on SQLite only. Amends [ticket 11](11-decide-storage-seam-of-rebuild.md) R1, R3 and R4.
- **Q5 = reject (F7).** No persisted job row, no `StorageBackend` trait, no state machine for `generate`.
- **Q1 (F1): open.** The owner asks what a split would actually gain in a solo setup with a known end result and no dependants.

**2026-10-17, owner ruling, round 2: Q1 = B.** The core step stays one step on one branch (ticket 09). Two of the three reasons for a green `main` do not apply here: no other contributor, and no user or deploy that depends on the app. The third reason stays: the implementer is a coding agent that steers by the tests, and the record change turns most tier-1 tests red (inferred). Two conditions close that gap:

1. **A short red window.** The plan orders the commits of the core branch so that the time with red tier-1 tests is as short as possible, and it names the commit where tier 1 goes red and the commit where it is green again.
2. **The target tests come first.** Before the core branch opens, the stop and race scenarios (I2 to I6, ticket 12) and the new R-59 scenario (Q3) are written, so the agent has a fixed target and not only the old tests.

Ticket 09's reason changes: it cited sketch 25 §2.2 for refusing a throw-away adapter, but §2.2 does not say that (known). The reason is the owner's: solo work, a known end result, and nothing depends on the app during the build.

## Answer

The runtime design is standard, and all three reviewers say so. The deviations were around it, and the owner trimmed them:

| Ruling | Change | Amends |
|---|---|---|
| Q1 = B | One core step stays. The plan names the red window of tier 1 and keeps it short. The target tests are written before the branch opens | ticket 09 |
| Q2 = C | Plain unit tests are allowed for pure functions in `domain/model/`. The count pin goes, and the allowlist stays | ticket 08 R2, R4; turn-state-model ticket 26 R2 |
| Q3 = accept | 18 rules become 14. Rules 8, 10 and 17 go, rule 7 keeps its field half, and rule 18 keeps its direct count. Rule 3 becomes a capability token. Rule 8 is replaced by an R-59 scenario | ticket 13 §1, §2 |
| Q4 = B | Tier 1 runs on SQLite `:memory:`. The new records get no in-memory arm, so no copy-and-swap `Tx` | ticket 11 R1, R3, R4 |
| Q5 = reject | No persisted job row, no `StorageBackend` trait, no state machine | — |

**2026-10-17, amended by [Decide the four gaps that the build plan found](16-decide-the-build-plan-gaps.md).** The rule count is 15, not 14 (Q5, owner confirmed). Rule 3's token has a second constructor for `test_support`, held by one call-site row (Q4).
