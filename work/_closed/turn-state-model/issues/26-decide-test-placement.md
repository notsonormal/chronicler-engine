# Decide where the tests of the new design sit

Type: grilling
Status: resolved
Blocked by: —

## Question

Where do the tests of the new design sit, area by area? Does the rule "unit tests cover everything, even what tier 1 covers" stay for the designed scope? The [test placement proposal](../assets/test-placement-proposal.md) gives a lean: tier 1 owns the requirements of `clean-spec.md` §4, and unit tests exist only at the black boxes of §3. [Decide where the new design of the game-state lifecycle makes sense](24-decide-fix-or-rewrite.md) uses the answer as part of the cost of a redesign.

## Scope

- **In:** the test home of each module of [sketch 25](../assets/sketch-25-new-design.md) draft 2; which of today's tests in the designed scope survive a redesign; the strict-fake rule; how to arrange state in a test; sketch §3.3 "Test seam" and sketch §6.2 rule 11; what fork 4 takes from the answer.
- **Open point 1 of the proposal decides the rest of the scope.** Ticket 24's Scope puts "the build and test tooling" in the "Not touched" group. So a rule for the whole repo is beyond this map's Destination, unless the owner redraws it.
- **The test oracle** (black-box tests from the specs that can run against a server in another language) belongs to the [platform choice](../../platform-choice/map.md) map. This ticket gives that map the placement, not the oracle.

## Inputs

- [The test placement proposal](../assets/test-placement-proposal.md): the facts (§2), the placement table (§4), the three conditions (§5), the options (§6) and the open points (§8).
- [Sketch 25](../assets/sketch-25-new-design.md): §3 (the six areas), §5 fork 4, §6 (enforcement), §7.1 (the fate of today's test lines).
- [Research 10](../assets/research-10-test-coverage.md): §1e (fixtures built by hand), §2 (lenient fakes), §5 (spec coverage of the defects).
- [`clean-spec.md`](../assets/clean-spec.md) version 3: §3 (the black boxes) and §4 (the requirements).
- `tests/STRATEGY.md`: "The tiers" and "Domain outcome (the tier-1 rule)".

## Context

- **Origin.** The owner, 2026-10-13, after a talk about deep modules and tests at the module interface: "One of the original principles is that 'unit tests should cover everything, even if the e2e/browser tests cover it' but now I'm considering that it might be overly defensive. Especially when the e2e tests run pretty quickly and are directly tied to specs, making they a better measure of accuracy. So then the question would be, where exactly would unit tests fit in this structure at all."
- **Why it blocks ticket 24.** The owner: "if I do [the redesign], the tests will need to be re-done, so this change is quite relevant." The pipeline's 124 unit-test functions (5,132 lines) are written against today's internals (ticket 24, session 1). Where the new tests sit sets what a redesign costs and what it leaves stable.
- **Ticket 25 is still open.** This ticket reads sketch draft 2. Its answer can feed the owner's reactions to the sketch (fork 4, §3.3, §6.2 rule 11). If a later draft changes a module, recheck that row of the placement.
- **Skills:** `/grilling`, `/domain-modeling` (if a tier term changes), `/codebase-design` (seams and interfaces).

## Done when

- Each module of the sketch has a test home (a tier, a seam contract, the compiler, or none), with a reason.
- The answer says which rule replaces "a test for each branch" for the designed scope, or that it stays.
- The answer rules on the three conditions of the proposal (§5), sketch §3.3 and sketch §6.2 rule 11.
- The answer says what goes to ticket 24, to ticket 25 (fork 4), and to the platform-choice map.
- The answer names the sketch draft that it was checked against.

## Comments

**2026-10-14, from [Sketch the new design next to today's code](25-sketch-new-design-next-to-code.md) (resolved).** The sketch is now draft 3. Recheck the rows of the placement that read draft 2:

- **Fork 4 is open, and this ticket decides it.** Ticket 25 handed it here: one `LlmRoles` trait in place of the `Agent` trait, `AgentRegistry` and `ExecutionPhase`, or no new seam.
- **Area 1, storage:** `Storage::commit(game, SaveUnit)` is gone. Draft 3 has repositories that read by `GameId`, and record writes only on a transaction handle (`Storage::transaction`). Each save unit is one `GameWriter` or `JobWriter` method.
- **§2.1:** a new persistence rule (pass ids, load late, save small), with its structure and guardrails.
- **§3.3:** "work" is now **Job** (`pipeline/job.rs`, `JobWriter`). The code shape of the order of a request is left to the build plan, with one `generate` for every kind of generation. The test seam is the same: start a Job with a fake `LlmRoles` and the in-memory `Storage`, then read the Swipe's state.
- **§6.2:** 15 rules, not 13. "One generation path" (rule 11 in draft 2) is now rule 13. New: rule 7 (pass ids), rule 8 (at most one LLM role call per function) and rule 9 (save small). The owner ruled that the guardrails are part of the design.

## Answer

Checked against **sketch 25 draft 3** and its §7.1 and §7.2 fate tables. `clean-spec.md` version 3 holds the requirements and the black boxes. Rulings R1 to R11 below. [Decide where the new design of the game-state lifecycle makes sense](24-decide-fix-or-rewrite.md) uses R10 as part of the cost of a redesign.

### R1. Scope: the designed scope only

Ruling: Q1 = A. The rule binds the modules of sketch 25. Today's tests outside the designed scope keep their rule. A rule for the whole repository sits past this map's Destination: a later issue owns it, and the map's **Out of scope** names it.

### R2. The rule that replaces "a test for each branch"

The rule text for `tests/STRATEGY.md` "The tiers":

> **Unit tier: only inside a seam contract.** A unit test exists only inside a seam contract (below). A behaviour that a player observes is a **scenario** in `docs/specs/`, with one test at the tier that observes it: tier 1 for HTTP, tier 2 or tier 3 for the DOM. A clause inside a black box that no route can reach is a **case** of that box's seam contract, with one unit test. A module between the two has no test of its own: the scenarios that pass through it are its coverage. **Check:** `spec-coverage` and `seam-cases`.

Five clauses carry the rule:

| # | Rule | Enforced by |
|---|---|---|
| 1 | A player-observable behaviour is a scenario, with one test at the tier that observes it | `validate_feature_spec.py` (exists) |
| 2 | A clause of a black box that a route cannot reach is a case, with one unit test | `scripts/validate_seam_cases.py` (new) |
| 3 | A unit test exists only inside a seam contract, at the named file, and calls only that box's entry point | a call-site guardrail (new, the §6.2 walker family) |
| 4 | "A test for each branch" survives only **inside** a seam contract: the branches of the box are its cases. It does not apply to a module | clauses 2 and 3 |
| 5 | The percentage of line coverage stays a diagnostic. No step fails on it | — |

### R3. How "enough coverage" is decided

Ruling: Q2, Q6 = A. The unit of completeness is the behaviour, not the class.

- **known:** the merged coverage report includes the tier-1, storage, bootstrap and browser tiers: `--coverage` runs `cargo llvm-cov nextest --no-report` twice and merges one report. It excludes some *source* files only.
- **known:** no step fails on it. `build.py::_generate_coverage_report` calls `parse_coverage.py` with `check=False`, and `tests/STRATEGY.md` marks the rule "review-only. No step gates on coverage."
- A percentage counts executed lines. The pipeline's pointer rules are 100 % line-covered today and asserted nowhere (research 10 §1b).

So the two deterministic checks are: a declared scenario has a covering test, and a declared case has a covering test. The percentage stays a way to find dead code.

### R4. The two declaration kinds, and their homes

| Kind | States | Home | Gate |
|---|---|---|---|
| **Scenario** | A behaviour that a player observes through HTTP or the DOM | `docs/specs/` | `validate_feature_spec.py` (exists) |
| **Seam case** | One clause of a black box: an input shape, its result, or a failure | The module doc of the box's test file, one `// CASE: <id>` per test | `scripts/validate_seam_cases.py` (new) |

Ruling: Q11 = B. The case list sits in the box's test file, and the rewritten `unit_test_standards.md` is the readable surface.

```rust
// The box's test file, module doc — the declaration.
//! CASE roles-1: prompt assembly renders the World, the Persona, the game state, the story
//!               window, the player's text, the stored inputs, the trigger context and the preset
//! CASE roles-3 [failure]: an unusable analysis answer gives the fallback — no movement, and the
//!               Characters that the scene holds now

// CASE: roles-3
#[test]
fn test_unusable_analysis_answer_falls_back_to_the_current_scene() { /* … */ }
```

The checker fails on a declared case with no tagged test, a tag with no declared case, a test with no tag in a declaring file, and a duplicate id. `build.py` runs it in the guardrails family, next to `spec-coverage`.

### R5. The seam contracts: two, not three

Rulings: Q13 = A, Q14 = agreed.

| Seam contract | Cases | Why a route cannot cover it |
|---|---|---|
| The roles adapter: prompt assembly, the parse, the fallback, the call record | about 10, guessed | The route-reachable clauses are scenarios (2.2, 2.3, 2.5). The residue is the prompt shape, the answer shapes no scenario states, the token budget and the call record |
| The storage transaction: a save unit lands whole (R-90, R-91) | about 8, guessed | No route stops a save half-way |

The **world rules get no case list.** Every observable clause of `apply_world_rules` is a scenario: a movement (1.14), the Characters present and the encounter record (1.6), a Trigger (1.4, 1.8), a contradiction (2.1). A player-observable clause must be a scenario. The interior combinations of the pure function get one **property-test set** over generated inputs, outside the case mechanism.

The seam contract is stated on whichever module holds the adapter. With fork 4 = A that is `application/agents/roles.rs`; with fork 4 = B it is the agent and `pipeline_run.rs` code. The contract survives either way.

**Storage keeps its own tier.** `tests/STRATEGY.md` already names it: driven-adapter, nothing faked, real SQLite. It is not the unit tier, because a tier is defined by what is faked. One suite runs on both backends, with one failure injected in the middle of a save unit. About 50 `_sqlite`/`_in_memory` twins inside `src/` collapse into it.

A seam contract's cases are the same declaration kind at either tier, so no storage test carries the label "unit".

### R6. The tier-1 fake

Ruling: Q10 = A, Q8 = A. Tier 1 fakes the LLM at the **model port** (`LlmProvider`), always, and an unscripted call fails the test. One rule, no judgement per test. Maximum real code runs under the route, including prompt assembly, the parse and the fallback.

- The fake must be able to **hold** a call until the test releases it. Today only one tier-1 test schedules a mid-flight moment, with a 200 ms sleep (`tests/http/actions.rs:610`); the unit tier already uses a real `Barrier` (`SyncQuantifierAgent`, `action_tests.rs:244`).
- A plain turn scripts its analysis answer. "No movement, no new Characters" becomes a stated decision, not a default.
- The empty `AgentRegistry` path and the `[System] NPC detection uncertain` artefact go. Scenario 22.1's unasserted "the player has not moved" line gains an assertion without a tripwire.
- The lower fake also covers the model-answer causes that `validate_feature_spec.py` forces to stay at tier 1: 2.2 (the call fails), 2.3 (the answer is empty), 2.5 (the answer carries no room report).

### R7. The three conditions of the proposal §5

| Condition | Ruling | Consequence |
|---|---|---|
| A strict fake | Q10 = A, Q8 = A | An unscripted call fails every test where it happens. No tripwire needed |
| Arrange through the public write path | Q4 = A | A tier-1 test builds its starting point by running the engine. `seed_main_narration_flow`, `seed_event_flow`, `seed_swipe_with_stored_inputs` and `insert_message_with_swipe` go: they write the pointers by hand in today's save order, which is how the seed defects stayed invisible (research 10 §1e) |
| A behaviour that matters has a scenario | Q5 = A, Q13 = A | Tier-1 coverage is spec coverage, so a designed behaviour needs its scenario first. The scenario is written in `docs/specs/`, with its test, in the build plan. `clean-spec.md` stays a map asset and never becomes the repository's source |

The condition-3 gap, known: `docs/specs/swipe_switch.md` holds 36.1 (a registry-backed status) and 36.2 (the dock). **Neither states what world a switch gives back.** The central ruling — the game state belongs to the Swipe — therefore has no scenario, and the build plan must write one.

### R8. Sketch §3.3 and §6.2 rule 13

- **§3.3's test seam.** `Job`, `JobPlan` and the step functions get **no unit test**: the order of a request is observable. The seam is a tier-1 test: start a Job through its route with the model fake and the in-memory backend, then read the Swipe's state through a read seam.
- **§6.2 rule 13.** Ruling: Q15 = A. The call-site walker stays. The "a test checks the save units of each kind in order" check goes, because it reads the implementation. Its place: a tier-1 test that holds the fake at the analysis step, then asserts that the story log already shows the text and that the status names that step (R-11a).

### R9. The placement of every module

| Module (sketch 25 draft 3) | Test home | Reason |
|---|---|---|
| `domain/model/state/swipe_state.rs` — `SwipeState`, `starting`, `pre_analysis` | Tier 1 | `starting` is create and reset (games 17.1, reset 7.1). `pre_analysis` is R-11a: a stop or a restart mid-generation shows it |
| `domain/model/world_rules.rs` — `apply_world_rules` | Tier 1 per observable clause; one property-test set for the interior combinations; **no case list** | R5 |
| `domain/model/tail.rs` — `Tail` and its rules | Tier 1; the compiler for `SwipeAt` | §4.7 to §4.9 and §4.11 are scenarios: 9.x, 10.x, 13.x, 14.x, 22.x |
| `domain/model/generation.rs` — `GenerationOrder`, `Kind`, `Place` | The compiler | A value type with no rule of its own |
| `domain/model/dock.rs` — `DockRecord` | Tier 1 for the dock view; the compiler for the type | R-83 to R-86; 24.x |
| `domain/model/game.rs`, `generation_status.rs`, `message.rs`, `message_types.rs`, `movement.rs`, `scene_state.rs`, `trigger_context.rs` | The compiler for the shapes; tier 1 for the behaviour that reads or writes them | A shape has no test; its behaviour is a scenario |
| `application/games/active.rs` — `ActiveGame` | Tier 1 | Game switch, create and reset: 18.x, 17.x, 7.x |
| `adapters/driving/http/utils/current_game.rs` — `CurrentGame` | Tier 1 | R-45: 11.2, 14.6 |
| `application/generation/gate.rs` — `GameWriter`, `JobWriter`, every save unit | Tier 1. The compiler for the private fields and the one constructor (guardrail 4) | R-70 to R-74a: the busy refusal, the stop, the switch, the reset. The dock rules and the failure line |
| `application/pipeline/job.rs` — `start`, `Job`, `generate`, the steps | Tier 1 only, no unit tests | R8, §3.3 |
| the roles adapter | **Seam contract** (unit), about 10 cases; tier 1 for 2.2, 2.3 and 2.5 | §3.1 to §3.3, §3.5 |
| `adapters/driven/storage/{stories,docks,transaction}.rs` | **Driven-adapter seam contract**, about 8 cases on both backends, with one mid-unit failure; tier 1 for the reads | §3.6, R-90, R-91 |
| HTTP handlers, `GameViewQuery` | Tier 1 | §3.7 |
| Templates and CSS | Tier 2 or 3 for client behaviour; none for markup alone | The UI placement rule of `tests/STRATEGY.md` |
| Sketch §6.2 rules 1 to 15 | The compiler, arch-lint, the syn walkers (three kinds), the two python checks | Structure, not behaviour. §6.2 already names the mechanism per rule |

The 15 guardrail rules are part of the design (ticket 25, Q8), so their own tests — the walker tests and the checker tests — are part of the build.

### R10. The cost, for ticket 24

| Fate | What | Size |
|---|---|---|
| Goes with its module | The pipeline's 124 unit-test functions | 5,132 lines (known, research 12 §5a) |
| Goes into one suite | The storage `_sqlite`/`_in_memory` twins | about 50 test functions (known) |
| Goes | `message_service`, guard and slot, `arrival_service`, the snapshot storage and mapper, `message_history`, the message and swipe storage | About **7,400** lines (computed from sketch §7.1's test-line column: the modules whose fate is **replaced** or **deleted**) |
| Goes partly | `game_state.rs`, split between `apply_world_rules` and the deleted story methods | About 1,840 lines; the rest of the "changed" modules keep part of theirs |
| Goes | The hand-seeded tier-1 fixtures and their pointer writes | `seed_main_narration_flow`, `seed_event_flow`, `seed_swipe_with_stored_inputs`, `insert_message_with_swipe` |
| Stays | The tier-1 scenario tests, their assertions and their tags. `tests/browser/` | The routes stay |
| Stays | `tests/storage/` as one suite on both backends | The shape is already right |
| New | Two seam contracts | about 18 cases, guessed |
| New | One property-test set for the world rules | guessed |
| New | `scripts/validate_seam_cases.py`, the call-site walker, the strict and holdable fake, the seeding helpers, the scenarios the rule requires (the swipe switch among them) | guessed |

A reshape therefore rewrites the tests of every replaced module, and it leaves the tier-1 scenarios, the driven-adapter shape and the browser tiers stable. That is the answer ticket 24 needs for its cost.

### R11. What each item goes to

Ruling: Q9 = agreed.

| Item | Goes to |
|---|---|
| **Fork 4** (one `LlmRoles` trait, or no new seam) | [Decide where the new design of the game-state lifecycle makes sense](24-decide-fix-or-rewrite.md). The tests **neither require nor forbid** the seam: tier 1 fakes the model port either way, and the seam contract is written on whichever module holds the adapter. Its reasons are design reasons — one narrow interface with three methods in place of the `Agent` trait, `AgentRegistry` and `ExecutionPhase` — which ticket 24 judges |
| Sketch §3.3's seam, §6.2 rule 13, and R10's cost | Ticket 24 |
| The rule text and the seam-contract rule | `tests/STRATEGY.md` "The tiers"; `unit_test_standards.md` is rewritten as the readable surface, with the rule and the two boxes named |
| The test oracle (black-box tests against a server in another language) | The [platform choice](../../platform-choice/map.md) map. This ticket gives that map the placement only |
| A rule for the whole repository, and today's duplicates outside the designed scope | A later issue: past this map's Destination, and named in the map's **Out of scope** |

Ticket 24's blockers 23, 25 and 26 are now all resolved. It is on the frontier, and it is the last ticket of this map.
