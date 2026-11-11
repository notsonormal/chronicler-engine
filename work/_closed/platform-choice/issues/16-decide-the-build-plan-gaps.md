# Decide the four gaps that the build plan found

Type: grilling
Status: resolved
Blocked by: —

## Question

[Write the build plan](14-write-the-build-plan.md) put the decided tickets together and found four points that no ticket decides. Each point blocks part of one slice. For each point, which option does the build take?

1. **The `Tx` on the in-memory backend in P5.** Ticket 09 and ticket 11 R7 put today's Game writes (`create_game`, `delete_game`) onto `Storage::transaction` and `Storage::transaction_new_game` in P5. Ticket 15 Q4 then said that `Tx` gets no in-memory arm and that tier 1 runs on SQLite `:memory:`. Today tier 1 runs on the in-memory backend (known: `TestAppBuilder::build_service_with_storage` defaults to `Storage::new_in_memory()`). Legacy unit tests also create Games on the in-memory backend (known: 18 `new_in_memory` calls in `games_tests.rs`, 4 in `catalogue_tests.rs`). No ticket says where the tier-1 switch lands.
2. **Where the target tests live before the core branch opens.** Ticket 15 Q1 says that I2 to I6 and the R-59 scenario are written before the core branch opens. Ticket 09 Q6 says that `main` passes the full gate after each slice. Most of these tests are red on today's code (ticket 12: I2 to I6 "Green today? no").
3. **The strict fake and the real Quantifier at tier 1.** Ticket 26 R6 (turn-state-model) makes the tier-1 fake strict (an unscripted call fails the test) and removes the empty `AgentRegistry` path, so every tier-1 turn runs the real Quantifier against a scripted answer. Ticket 12 Q4 puts the *hold* of the fake into P4. No ticket places the strictness and the real Quantifier. Today tier 1 uses `AgentRegistry::default()` (known, `src/test_support/context.rs`, `tests/http/support/app_wiring.rs`) and a `MockBackend` that answers unscripted calls.
4. **How the storage seam contract gets the rule-3 token.** Ticket 15 Q3 makes rule 3 a capability token: `Storage::transaction*` takes a token that only `gate.rs` can build. The storage seam contract in `tests/storage/` must open a transaction without `GameWriter` (ticket 26 R5: a failure in the middle of a save unit). `tests/storage/` is an outside test crate, so it cannot call a constructor that is private to `gate.rs`.

## Context

- The plan: [`docs/plans/rust-rebuild-build-plan.md`](../../../../docs/plans/rust-rebuild-build-plan.md). Each gap is marked **Open: ticket 16, gap N** where it applies.
- [Decide the build path](09-decide-build-path.md), [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md), [Decide the race tests and guardrail rule 18](12-decide-race-tests-and-rule-18.md), [Review: the design and the build decisions against standard engineering practice](15-review-design-against-standard-practice.md).
- Turn-state-model [ticket 26](../../turn-state-model/issues/26-decide-test-placement.md) R5, R6.

## Options and recommendations

The recommendations are the plan writer's (inferred). The owner decides.

### Gap 1: the `Tx` on the in-memory backend

| Option | What P5 does | Cost |
|---|---|---|
| **A (recommended)** | P5 switches tier 1 to SQLite `:memory:`. P5 writes `Tx`, `TxWrite`, the two entries and `newest_game_of_world`, and tests `Tx` only in the storage seam contract (SQLite). Today's Game writes stay off `Tx` until the core step, where create and reset (area 5) move onto `GameWriter` anyway | The `Tx` has no production caller in P5. The legacy in-memory tests do not break |
| B | P5 moves today's Game writes onto `Tx`. `Tx` gets an in-memory arm for the Game record only, with no unit (the writes land directly) | Goes against the words of ticket 15 Q4 ("`Tx` gets no in-memory arm") |
| C | P5 moves today's Game writes onto `Tx`, SQLite only. The legacy tests that create Games on the in-memory backend move to SQLite `:memory:` | 16 test files that call a Game write and build `Storage::new_in_memory()` change in P5 (known, `grep`). Every tier-1 test that uses the `TestAppBuilder` default also moves to SQLite `:memory:` |

### Gap 2: the target tests before the core branch

| Option | Where | Cost |
|---|---|---|
| **A (recommended)** | On `main`, at the end of P5: the scenarios in `docs/specs/` and their tests, each red test marked `#[ignore = "red until the core step: <case>"]`. The core step removes each `#[ignore]` when its test goes green. A test that is green today (perhaps R-59) has no `#[ignore]` | `main` stays green. The repository already uses `#[ignore = "..."]` (`tests/llm/flow_llm_tests.rs`). Not checked: whether `validate_feature_spec.py` accepts an ignored test as cover (inferred yes: it reads the tags) |
| B | The first commit of the core branch, before any code change | The red window of tier 1 opens at the first commit |

### Gap 3: the strict fake and the real Quantifier at tier 1

| Option | Where | Cost |
|---|---|---|
| **A (recommended)** | P4, with the hook traits and the holdable fake. Every tier-1 test that runs a turn scripts its analysis answer | P4 grows. The rework lands on a green slice, not in the red window of the core step |
| B | The core step | The red window gets longer |
| C | A new slice after P4 | One more slice and one more full gate |

### Gap 4: the rule-3 token in the storage seam contract

| Option | How | Cost |
|---|---|---|
| **A (recommended)** | `gate.rs` has one more constructor for the token, `#[doc(hidden)]`, and only `src/test_support/` calls it. One call-site row (the walker of rule 2) allows that call only in `test_support/`. arch-lint already keeps production code out of `test-support` (inferred from `arch-lint.toml`) | One more syn row, after ticket 15 trimmed the rows |
| B | The storage seam contract runs its save units through `GameWriter`, not through a raw `Tx` | The storage contract then depends on the application layer. The "failure in the middle of a save unit" case needs a `GameWriter` save unit with more than one record write |
| C | The token is built in the storage module (for example `Tx::token()`), and rule 3 stays a syn call-site row as ticket 13 had it | Undoes the compiler-level part of ticket 15 Q3 |

## Also for the owner to confirm (no option)

Ticket 15 says "18 rules become 14". Rules 8, 10 and 17 go, so 15 rules stay: 1, 2, 3, 4, 5, 6, 7, 9, 11, 12, 13, 14, 15, 16, 18 (inferred: the count in ticket 15 is wrong; its list of six syn rows is right). The plan lists 15.

## Done when

The owner has chosen an option for each of the four gaps. The plan is changed in place, and each **Open: ticket 16** mark is replaced by the ruling.

## Grilling notes

**2026-10-17, owner.**

- Q1 (gap 1) = **A**. P5 moves tier 1 to SQLite `:memory:`. P5 builds `Tx`, `TxWrite`, the two entries and `newest_game_of_world`, and tests `Tx` only in the storage seam contract (SQLite). Today's Game writes stay off `Tx` until the core step, where create and reset move onto `GameWriter`.
- Q2 (gap 2) = **A**. The target scenarios and tests land on `main` at the end of P5. Each red test carries `#[ignore = "red until the core step: <case>"]`; a test that is green today has none. The core step removes each `#[ignore]` when its test goes green (C5). P5 checks that `validate_feature_spec.py` counts an ignored test as cover.
- Q3 (gap 3) = **A**. P4 makes the tier-1 fake strict (an unscripted call fails the test) and removes the empty `AgentRegistry` path, so the real Quantifier runs at tier 1 and each turn scripts its analysis answer. About 95 tier-1 tests in 6 files run a turn (inferred from a `grep` count).
- Q4 (gap 4) = **A**. `gate.rs` has a second token constructor, `#[doc(hidden)]`, which only `src/test_support/` calls. One call-site row allows that call only in `test_support/`. The storage seam contract gets the token through `test_support`.
- Q5 (the rule count) = **confirmed**: 15 rules stay (1, 2, 3, 4, 5, 6, 7, 9, 11, 12, 13, 14, 15, 16, 18). The "14" in ticket 15 is a count error.

## Answer

Resolved 2026-10-17 by the owner, in a grilling (Q1 to Q5). Each ruling took the recommended option.

| Gap | Ruling | Slice |
|---|---|---|
| 1. `Tx` on the in-memory backend | **A.** P5 moves tier 1 to SQLite `:memory:` and builds `Tx`, `TxWrite`, the two entries and `newest_game_of_world`. The storage seam contract is the only user of `Tx` in P5. Today's Game writes stay off `Tx` until create and reset move onto `GameWriter` in the core step. The legacy in-memory unit tests stay as they are | P5, core |
| 2. The target tests | **A.** On `main` at the end of P5. A red test carries `#[ignore = "red until the core step: <case>"]`; C5 of the core step removes it. P5 checks that `validate_feature_spec.py` counts an ignored test as cover | P5, core C5 |
| 3. The strict fake and the real Quantifier | **A.** P4: the fake becomes strict and holdable, the empty `AgentRegistry` path goes, and each tier-1 turn scripts its analysis answer (about 95 tests in 6 files, inferred) | P4 |
| 4. The rule-3 token in the storage seam contract | **A.** A second token constructor in `gate.rs`, `#[doc(hidden)]`, called only from `src/test_support/`; one call-site row allows that call only in `test_support/` | core C2 |
| 5. The rule count | **15 rules**, confirmed. The syn rows are 1, 2, 6, 7, 13, 18 and the token-constructor row of gap 4 | — |

**Amends.** Ticket 11 R7 (today's Game writes do not move onto `Tx` in P5). Ticket 15 Q3 (15 rules, not 14; one syn row for rule 3's test constructor). Ticket 12 Q4 (P4 also makes the fake strict).

**Applied.** [`docs/plans/rust-rebuild-build-plan.md`](../../../../docs/plans/rust-rebuild-build-plan.md) and [The rebuild at a glance](../assets/shape-at-a-glance.md) were changed in place; the plan has no **Open** mark left.

**2026-10-17, amended by [ticket 17](17-decide-the-plan-review-gaps.md) Q2.** The Q4 ruling cannot be built: a token type declared in `gate.rs` makes `storage/` import `application/`, which arch-lint denies. The token is now `TxToken` in `storage/transaction.rs` with a `pub(crate)` constructor, and one call-site row allows `TxToken::new` only in `generation/gate.rs` and `test_support/`. There is no hidden second constructor. The count of syn rows stays seven.
