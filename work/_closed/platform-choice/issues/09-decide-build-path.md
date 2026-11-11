# Decide the build path

Type: grilling
Status: resolved
Blocked by: 04, 05, 11

## Question

How does the new design land in the crate: in one step, or in slices behind the same HTTP routes? If in slices, what is the order and what keeps the tests green between slices? What happens to new features and other branches during the build?

## Context

- [Decide: keep Rust and build the new design in place](03-decide-keep-rust-build-in-place.md).
- The size (turn-state-model handoff): 3,577 non-test lines replaced or deleted (known); about 1,765 new lines plus about 80 for the Agent hooks (guessed); about 7,400 unit-test lines go.
- [Sketch 25](../../turn-state-model/assets/sketch-25-new-design.md) §7 fate tables; the six areas of the designed scope.
- The scope of the build comes from [Research: sort the open architecture-deepening tickets against the new design](04-research-sort-architecture-deepening.md). The Job shape comes from [Decide the shape of the Job and the busy state](05-decide-job-and-busy-state-shape.md).
- Parallel agents share one build slot (problem B), so a path with many parallel slices has a cost.
- **Added by ticket 04 (2026-10-16).** The storage work comes from [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md). The edges that the build touches, and how: [research 04](../assets/research-04-architecture-deepening-sort.md) §3. One note to carry into the plan (inferred, research 04 §2.8): the `Refusal` → `200`/`400`/`500` mapping gets one home, next to `impl IntoResponse for ApplicationError` in `http/error.rs`.

## Done when

The owner has chosen the path, the slice order (if any), and the rule for other work during the build.

## Comments

**2026-10-16, from [Decide the test rule outside the designed scope](08-decide-test-rule-outside-designed-scope.md) (resolved).** The plan must order this test work (ticket 08 R7):

- `tests/STRATEGY.md` and `unit_test_standards.md` state one unit-test rule for the whole repository, with the seam-contract criterion of ticket 08 R2.
- The legacy list and its pin, held by the call-site guardrail with an allowlist (ticket 08 R4).
- New seam contracts in the build: the `Narrator` (ticket 08 R3) and one for each Agent, because the hook traits break `agent_tests.rs`.
- The rule for other work during the build (ticket 08 R5): a test that the build breaks is updated under the new rule, and a test that does not break stays as it is.

**2026-10-16, from [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) (resolved).** The storage seam keeps the `Backend` enum and puts the repositories on `Storage`, with no arch-lint change. Risk for the build path: the removal of `Storage::game_id` breaks 240 calls in 50 files (`set_game_id`, `current_game_id`, `storage.game_id()`) and 34 `Storage::new_sqlite(pool, game_id)` calls in 9 files. The order of the build must plan for this compile break.

## Grilling notes

**Round 1 (2026-10-16, owner).**

- Q1 = C: preparation slices (green), one core step, then follow-up slices. No throw-away `GameState` ↔ `SwipeState` adapter.
- Q2 = yes: the test rule (ticket 08 R7: `tests/STRATEGY.md`, `unit_test_standards.md`, the legacy list and its pin with the call-site guardrail) lands as the first slice, before any code change.
- Q3: the 12 worktrees were removed on 2026-10-16 (all clean; branches kept). No open worktree touches the designed scope.
- Q4 = A: the designed scope (sketch 25 §7.1 files) is frozen during the build. A player-visible bug in scope gets a scenario in `docs/specs/` that the build must make pass. Work outside the scope continues under ticket 08 R5.

**Round 2 (2026-10-16, owner).** Correction recorded first: the Agents read `&GameState` and the save units write the dock, so neither can be a follow-up slice after the core step.

- Q5: order accepted — P0 test rule, P1 `ActiveGame`/`CurrentGame`, P2 delete the arrival opening, P3 `SwipeState` inside `GameState` with `apply_world_rules`, P4 the Agent edge on values (hook traits, `HookInput`, `Narrator`; < 50 lines of glue that the core deletes), P5 the storage `Tx` mechanism; then the core step.
- Q6 = A: each preparation slice merges to `main` after the full gate; the core step lives on its own branch and merges once.
- Q7 = A: each commit on the core branch passes `clippy` and the tests it does not break; the full gate runs at the merge.
- Q8 = A: one implementer at a time; a reviewer after each slice; the owner approves each commit.
- Q9 = A: a last ticket writes the plan into `docs/plans/`; the build runs as a new map with one `task` ticket per slice.

**Round 3 (2026-10-16, owner).**

- Q10 = A: each guardrail rule lands in the slice that creates the code it guards.
- Q11: there is no other contributor, so no other change runs during the build. A freeze window decides nothing.
- Q12 = B: no follow-up slice. Each doc change lands in the slice that makes it true; the `System` templates and CSS go with the core step.

## Answer

Resolved 2026-10-16 by the owner, in a three-round grilling (Q1 to Q12). Labels: **known**, **inferred**, **guessed**.

### The path

**Six preparation slices, then one core step, then no follow-up slice** (Q1 = C, Q12 = B). Each preparation slice is green with the full gate and merges to `main`. The core step is one step, because the record change (Snapshot → `SwipeState` on the Swipe) touches storage, `GameState`, `MessageService` and the pipeline together, and a green middle state would need a throw-away `GameState` ↔ `SwipeState` adapter (inferred), which sketch 25 §2.2 refuses.

| Slice | Content | Why here |
|---|---|---|
| P0 | The test rule of ticket 08 R7: `tests/STRATEGY.md`, `unit_test_standards.md`, the legacy list and its pin with the call-site guardrail and its allowlist | The pin must exist before a slice deletes or rewrites a test. Then every later test change shows as a pin change, and the guardrail refuses a broken test ported under the old rule (Q2) |
| P1 | `ActiveGame` and `CurrentGame` in place of `Storage::game_id` | Takes the mechanical break (240 calls in 50 files, 34 `new_sqlite` calls in 9 files; known, ticket 11) out of the core step |
| P2 | Delete the arrival opening (D-25): `arrival_service.rs` and its spec scenarios | A player-visible change that stands by itself |
| P3 | `SwipeState` as a field of `GameState` (`movement`, `scene`, the encounter log), with `apply_world_rules(&mut SwipeState, ..)` and the ~330 moved lines | Not throw-away: the core step deletes only the `GameState` shell around them (inferred) |
| P4 | The Agent edge on values: the hook traits, `HookInput`, the typed outputs (sketch 24 §2.2), and `Narrator` with the token budget as a value; the Quantifier, Options and Narrator seam contracts (ticket 08 R3, R6) | Today's Agents read `&GameState`, which the core step deletes. The glue that today's pipeline needs (`GameState` → `HookInput`) is less than 50 lines (guessed) and the core step deletes it. This is the only throw-away code of the path |
| P5 | The storage `Tx` mechanism: copy-and-swap (ticket 11 R3), `TxWrite` (R4), `transaction` and `transaction_new_game` on today's Game writes (R7), `newest_game_of_world` (R6) | Gives the core step a tested `Tx` |
| Core | The records and the schema (area 1), the dock and the `System` templates and CSS (area 6), `GameWriter`/`JobWriter` (area 2), `pipeline/job.rs` with the hooks, `JobPlan`, `run_step`, `SavedText` (area 3, ticket 05), the view reads (area 4), create and reset (area 5), `Refusal` and its `200`/`400`/`500` mapping next to `impl IntoResponse for ApplicationError` in `http/error.rs` (research 04 §2.8), and the deletions of sketch 25 §7.1 | The save units write the dock, so the dock cannot follow the core step (correction recorded in round 2) |

### Branches and red time

- **Branches** (Q6 = A). Each preparation slice merges to `main` after the full gate. The core step lives on its own branch and merges once, when the full gate is green. Before P0, `dashboard-ui-review-phase3` (8 commits ahead of `main`, 0 behind; known) merges to `main`; the owner does that merge.
- **Inside the core branch** (Q7 = A). Commits in dependency order: records and storage → `GameWriter`/`JobWriter` → `job.rs` → the view reads and handlers → the deletions and the test rework. Each commit passes `clippy` and the tests that it does not break. The full gate runs only at the merge.

### Who works, and how

- **One implementer at a time** (Q8 = A). The slices run in order. A reviewer checks each slice, and the owner approves each commit. Reason: the slices depend on each other, and parallel agents wait for the one build slot (problem B).
- **Tests in each slice.** A slice that changes behaviour writes its scenarios in `docs/specs/` before their tests (ticket 26 R7 condition 3, an existing rule). A test that the slice breaks is rewritten into its seam contract or its scenario under the one rule; a test that it does not break stays on the legacy list (ticket 08 R5).
- **Guardrails** (Q10 = A). Each rule of sketch 25 §6.2 (and rule 18, if ticket 12 takes it) lands in the slice that creates the code it guards: for example, rule 3 in P5 and rule 17 in the core step. Ticket 13 decides which rules and which tool.
- **Docs** (Q12 = B). Each doc change lands in the slice that makes it true, so the docs that agents read stay true on `main` after each merge. Ticket 13 decides which docs change.

### Other work during the build

**None** (Q4, Q11). The owner is the only contributor, so no other change runs during the build. A player-visible bug found during the build becomes a spec scenario in the slice that runs. The 12 worktrees were removed on 2026-10-16 (all clean; their branches kept). Ticket 08 R5 governs the tests.

### The plan document

**A new ticket writes it** (Q9 = A): [Write the build plan](14-write-the-build-plan.md), blocked by tickets 12 and 13. It writes one file in `docs/plans/`, in slice order, with nothing left to decide. The build then runs as a new wayfinder map with one `task` ticket per slice, and that map's Notes override "plan, don't do".

**2026-10-17, amended by [Review: the design and the build decisions against standard engineering practice](15-review-design-against-standard-practice.md) (Q1 = B).** The core step stays one step. Its reason is now the owner's: solo work, a known end result, and nothing depends on the app during the build. The citation of sketch 25 §2.2 was wrong, because §2.2 does not refuse a migration adapter. Two conditions are added: the plan names the commit where tier 1 goes red and the commit where it is green again, and keeps that window short; and the stop and race scenarios I2 to I6 and the R-59 scenario are written before the core branch opens.

**2026-10-17, amended by the owner during the review of the build plan ([Write the build plan](14-write-the-build-plan.md)).** Q7 and Q8 change. Before every commit of the build, `/simple-after-plan` runs with the scope of that commit; a step whose input is empty is skipped. Every commit runs the full gate, except C4 of the core branch, which is red on purpose and stays its own commit: its gate is `clippy` plus the tests that it does not break. The retro runs once per preparation slice and once at the end of the core branch. The full gate at the merge of the core branch is the gate of its last commit.

**2026-10-17, amended by [ticket 17](17-decide-the-plan-review-gaps.md) Q8.** "A test that the slice breaks is rewritten into its seam contract or its scenario under the one rule" now applies only to a test whose behaviour breaks. A legacy test that breaks only by a mechanical change (a new parameter, a renamed type) changes in place and stays on the legacy list.
