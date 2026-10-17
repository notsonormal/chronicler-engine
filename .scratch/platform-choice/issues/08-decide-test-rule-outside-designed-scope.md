# Decide the test rule outside the designed scope

Type: grilling
Status: resolved
Blocked by: 04

## Question

Turn-state-model [ticket 26](../../turn-state-model/issues/26-decide-test-placement.md) R1 binds the new test rule (a unit test only inside a seam contract; player-observable behaviour as scenarios) to the modules of sketch 25 only. Does the rule apply to the whole repository in the rebuild? If yes, which modules outside the designed scope need a spec or a seam contract first, and what happens to their unit tests?

## Context

- Ticket 26 R1 to R11: scenarios in `docs/specs/`, two seam contracts (the roles adapter, the storage transaction), the storage tier, the fake at `LlmProvider`, about 7,400 unit-test lines go with the replaced modules.
- Four subsystems have no spec: the Quantifier, Triggers, prompt assembly and storage (seed §3).
- Which modules stay outside the designed scope comes from [Research: sort the open architecture-deepening tickets against the new design](04-research-sort-architecture-deepening.md): [asset](../assets/research-04-architecture-deepening-sort.md) §3 lists the edges and what the build does to each.
- `tests/STRATEGY.md`, `CODING_STANDARDS.md`.

## Done when

The owner has ruled on the scope of the rule, and the answer lists each module outside the designed scope with its test result: scenarios, a seam contract, or today's rule.

## Answer

Resolved 2026-10-16 by a grilling with the owner (Q1 to Q7). Facts read at commit `d067420b`. Labels: **known**, **inferred**, **guessed**.

### Facts

- **known:** 27,112 unit-test lines in 106 `src/**/*_tests.rs` files, with 1,259 test functions.
- **known:** ticket 26 R10 removes about 7,400 of these lines, and about 1,840 more from `game_state_tests.rs`. **inferred:** about 17,900 lines stay outside the designed scope.
- **known:** the `agent_tests.rs` files of the Quantifier and of Options use `from_config_with_storage` or the `Agent` trait, and the hook design (sketch 24) replaces both. Their `prompt`, `parser` and `orchestration` tests read pure parts that move with no change of logic.
- **known:** `assembler_tests.rs` calls `with_storage` or `build_narration_prompt`, which the build deletes (research 04 §2.4).

### R1. Scope: one rule for the whole repository (Q1 = C)

`tests/STRATEGY.md` states one unit-test rule: ticket 26 R2, "a unit test only inside a seam contract". This replaces ticket 26 R1, which bound the rule to the modules of sketch 25 only. A module that does not follow the rule yet keeps its unit tests on a pinned **legacy list** (R4).

### R2. What makes a module a seam contract (Q2 = A)

A seam contract (a `CASE` list, ticket 26 R4) exists only for a black box behind a port boundary that tier 1 fakes or does not go through:

- the storage adapters (driven-adapter tier, `tests/storage/`, both backends);
- the LLM providers and the transport (behind `LlmProvider`);
- the Harper text checker (behind `TextChecker`);
- each Agent (sketch 24 §6, one contract for each Agent);
- the `Narrator` (R3).

A pure domain function gets no contract. Each clause of it that a player can see is a scenario. The inside combinations get a property-test set, as ticket 26 R5 did for `apply_world_rules`.

### R3. The narrator's prompt assembly is a seam contract (Q4 = A)

The `Narrator` (sketch 24 §2.2) is a third box of the Agent kind, behind `LlmProvider`. Its cases are the sections of the prompt, the fit to the token budget and the cleanup of the input (today `assembler`, `builders/sections`, `utils/context`, `sanitize`, `token_budget`). Its clauses that a player can see (2.2, 2.3, 2.5) stay at tier 1 (ticket 26 R6). This takes the place of case `roles-1` of ticket 26 R4 and R5. The prompt text goes into no spec, because specs state what the player sees.

### R4. The legacy list and its pin (Q5 = A)

- The list holds the legacy `*_tests.rs` files.
- A guardrail fails when a file that is not on the list and is not a seam contract holds a unit test. It also fails when the count of unit tests in listed files is different from the pin. This is ticket 26 R2 clause 3 (the call-site guardrail), with an allowlist.
- A bug fix in a legacy module: a player-visible failure gets a scenario. Otherwise the fix adds a unit test, raises the pin, and the change shows in the diff, as with the pin on `requires_migration/`.
- When a module moves off the list, its tests go, and the pin goes down.

### R5. When a module moves to the rule (Q3, Q6)

The owner's words: "I'll just progressively replace the existing tests for the other parts of the code whenever it makes sense." And: "If during the build we change the code and the tests no longer work with the updated version of the code we should update the tests. On other, tests that don't break when doing the build can stay as is."

- **The build:** a test that the build breaks is updated. A test that the build does not break stays as it is, on the legacy list.
- **inferred from R1 and R4:** an updated test is a new test under the one rule. The R4 guardrail fails a new unit test outside a seam contract unless the pin goes up. So a broken test is rewritten into its seam contract or its scenario, not ported under the old rule.
- **After the build:** the owner moves each legacy module when it makes sense. There is no fixed trigger and no later effort that must move them all.

### R6. Each module outside the designed scope

| Module | Unit-test lines (known) | Final home | In the build? |
|---|---|---|---|
| Quantifier Agent | 1,218 | Seam contract (R2) | **Yes** for `agent_tests.rs`, because the hook traits break it. `prompt`, `parser` and `orchestration` stay legacy |
| Options Agent | 490 | Seam contract (R2) | Same as the Quantifier |
| Narrator prompt assembly (`prompting/`) | about 1,260 | Narrator seam contract (R3) | **Yes.** The `Narrator` is new code, and `with_storage` and `build_narration_prompt` go |
| Game rules and Triggers (`trigger.rs`, moved into `apply_world_rules`) | 127 + part of `game_state` | Scenarios plus a property-test set (ticket 26 R5) | **Yes**, already in ticket 26 R9. The build writes the scenarios for Triggers and the Quantifier, which have no spec today (ticket 26 R7) |
| Storage: the designed records (`stories`, `docks`, `transaction`) | — | Driven-adapter seam contract (ticket 26 R5) | **Yes** |
| Storage: the other records (settings, presets, worlds, characters, personas, LLM messages, games, `db`, `core`, `utils`, `models`) | about 3,300 | Driven-adapter seam contract in `tests/storage/` | Legacy, unless the storage seam breaks them (R5). [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) decides that |
| LLM providers and transport | 663 | Seam contract behind `LlmProvider` | Legacy. The build does not change them (AD-17) |
| Harper text checker and the `TextChecker` port | 327 | Seam contract | Legacy |
| `llm_recorder`, `connection_test_service`, `text_check_service` | 663 | Scenarios (`llm_messages.md`, `settings.md`) | Legacy |
| CRUD services (`PersonaCatalogue`, `SettingsService`, `PromptPresetService`, `WorldCatalogue`) | 342 | Scenarios through their routes | Legacy. The last question of ticket 11 decides whether they join the repositories |
| HTTP handlers, builders, templates and views that sketch 25 §7.1 does not name | about 4,000 | Scenarios at tier 1, 2 or 3 | Legacy. `requires_migration/` keeps its own pin |
| Domain models outside sketch 25 (`action`, `agent`, `character`, `map`, `prompt_preset`, `scenario`, `settings`, `template`, `world`, `llm_backend`, `utils/`) | about 1,800 | Scenarios, plus a property-test set where the inside combinations matter (R2) | Legacy |
| Bootstrap (`load`, `run`, `wiring`) | 1,269 | The `tests/bootstrap/` smoke tier, nothing faked (Q7 = yes) | Legacy. The lookup by World (AD-16) goes to ticket 11 |
| `error.rs`, `utils/cli.rs`, `utils/settings.rs` | 608 | The compiler for the shapes, scenarios for the behaviour | Legacy |

"In the build: yes" for an Agent or the Narrator follows from R5: the build breaks those tests. The line counts are sums of the `*_tests.rs` files of each group. The groups with "about" are rounded (inferred).

### R7. What each item goes to

| Item | Goes to |
|---|---|
| The one rule, the seam-contract criterion (R2), the legacy list and the pin (R4) | The build: `tests/STRATEGY.md` "The tiers", `unit_test_standards.md`, and the call-site guardrail with its allowlist. [Decide the build path](09-decide-build-path.md) orders them |
| The Narrator contract and the two Agent contracts | The build, with the hook traits |
| Which storage tests the storage seam breaks | [Decide the storage seam of the rebuild](11-decide-storage-seam-of-rebuild.md) |
| The legacy modules | The owner, one at a time, when it makes sense (R5). No ticket |

**2026-10-17, amended by [Review: the design and the build decisions against standard engineering practice](15-review-design-against-standard-practice.md).** The owner's rulings there change parts of this answer (see its Comments: Q2 for ticket 08, Q4 for ticket 11, Q3 for ticket 13). Where the two conflict, ticket 15 wins.
