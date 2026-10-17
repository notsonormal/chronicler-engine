# Test placement for the new design: a proposal

Ticket: [Decide where the tests of the new design sit](../issues/26-decide-test-placement.md).
Design: [`sketch-25-new-design.md`](sketch-25-new-design.md) draft 2. Spec: [`clean-spec.md`](clean-spec.md) version 3.
Code: no change under `src/` or `tests/`.

Status: **proposal, for the owner's grilling.** Nothing here is decided.

## 0. How to read this proposal

- **Labels.** **known**: read in the code, in `tests/STRATEGY.md`, in `AGENTS.md`, or in research 10, research 12 or sketch 25. **inferred**: follows from those, not run. **guessed**: an estimate.
- **Counts** come from `grep` and `wc -l` on the working tree. They are approximate. For example, a `#[test]` count does not include `proptest!` cases.
- **Terms.** Tier names are those of `tests/STRATEGY.md` "The tiers": unit, HTTP E2E (tier 1), stub browser (tier 2), full-stack browser (tier 3), driven-adapter. The design terms are those of sketch 25.

## 1. The question

`tests/STRATEGY.md` says: "**Unit tier: a test for each branch.**" The owner's working principle is that unit tests cover everything, even what the tier-1 and browser tests cover. The owner now asks whether this principle is too defensive. Tier 1 runs fast and maps to the spec scenarios, so it may be the better measure of behaviour. If so, where do unit tests fit in the new design?

This matters for the verdict of ticket 24. A redesign replaces the tests of the modules that it replaces. So where the tests sit sets part of the cost of a redesign, and part of the cost of any later reshape.

## 2. Facts

### 2.1 Size and speed (known)

| Fact | Value | Source |
|---|---|---|
| Non-test lines in `src/` | about 21,900 | `wc -l`, without `*_tests.rs` |
| Unit-test lines (`src/**/*_tests.rs`, 106 files) | about 27,100 | `wc -l` |
| Lines under `tests/` (96 files) | about 27,100 | `wc -l` |
| Unit-test functions | about 1,259 | `#[test]` / `#[tokio::test]` count in `src/` |
| Test functions under `tests/` | about 579 | the same count in `tests/` |
| `SCENARIO:` tags under `tests/` | 284 | `grep` |
| Storage unit tests named `*_sqlite` or `*_in_memory` | 50 | `grep` in `src/adapters/driven/storage/` |
| A helper that runs one test on both storage backends | none found | `grep` |
| A tier-1 test that blocks a fake on a barrier | none | `Barrier` occurs only in `action_tests.rs` (unit) and `tests/storage/preset_storage.rs` |
| `python build.py integration` (tier 1, driven-adapter and lib unit tests) | about 21 s warm | `AGENTS.md` |
| `python build.py browser` | about 92 s warm | `AGENTS.md` |

### 2.2 What the research found about today's tests (known)

1. **The redesign deletes the unit tests of the pipeline.** The pipeline has 124 unit-test functions (5,132 lines) that call `ActionPipeline` methods directly. Their modules are replaced, so these tests go too (sketch 25 §7.1, inferred there). The tier-1 tests go through the routes and stay. Only their parts that seed Snapshots by hand change.
2. **A duplicate test held the opposite of the spec.** `test_retry_flow_impersonate_mode_runs_full_tail` asserts that a re-impersonate runs the full tail. Spec I.6 and 22.1 say that it does not. The test has no scenario tag, so `validate_feature_spec.py` did not see the conflict (research 10 §5).
3. **Fixtures built by hand hid a defect.** `seed_main_narration_flow` and `seed_event_flow` write the Snapshot pointers in the save order of research 04. No test that uses them can detect a pointer defect (research 10 §1e).
4. **Tier 1 was blind too, for two reasons:**
   - **A lenient fake.** The HTTP apps pass an empty `AgentRegistry`. So every turn with no script runs on `QuantifierResult::default()`: no movement, the room's default Characters, and a trailing `System` notice. "The player has not moved" (22.1, 22.2) is true for every implementation (research 10 §2b, §2c).
   - **Display-only assertions.** The status endpoint is backed by the registry. It answers `idle` when the saved status is wrong, so 36.1 cannot fail for a wrong status write (research 10 §1a).
5. **Tripwires work.** 10.2 and 13.2 seed an analysis answer that would move the player if the Quantifier ran. They are the only tier-1 tests that can fail when a Quantifier runs that should not (research 10 §5).

### 2.3 What these facts show (inferred)

- The tests that survive a reshape are the tests at a stable interface: the routes and the spec. The tests at today's inner functions do not survive.
- Two tests of one behaviour at two tiers are two sources of truth. When they disagree, only the tagged one is checked against the spec.
- Tier 1 is a good measure of spec conformance only when its fake is strict and its assertions read domain outcomes. Today neither is always true.

## 3. The proposal: the tests follow the structure of the spec

`clean-spec.md` has two kinds of content:

- **§4, the requirements** (R-nn). A player sees each one through HTTP or the DOM.
- **§3, the black boxes.** Each black box has a contract: its inputs, its result and its failures. The narrator call, the Quantifier call, the Options call, the world rules, prompt assembly, storage and the HTTP views.

The proposal maps the tests to the same split:

- A §4 requirement is tested at tier 1 (or tier 2 or 3 for a DOM-observed scenario), and only there.
- A §3 contract is tested at its seam, by a contract test.
- A module between the two (the orchestration) gets no test of its own. Tier 1 covers it.

## 4. Where each test goes in sketch 25

| Module (sketch 25) | Test home | Reason |
|---|---|---|
| HTTP handlers, `CurrentGame`, `GameViewQuery` | tier 1 | Spec-tied. §3.7 states what each view shows |
| `work::start`, `Work::run`, `Work::generate` | tier 1 only | The route is a thin wrapper over `work::start`. A test at `Work` repeats the tier-1 test with another entry point (inferred) |
| `GameWriter`, `WorkWriter` | tier 1 for R-70 to R-74a; the compiler for sketch §6.2 rule 4 | A fake `LlmRoles` that blocks on a barrier lets a tier-1 test schedule "stop, reset or switch during the narrator call". The LLM calls are the only long waits, so each step boundary can be reached from a route (inferred; no such tier-1 test exists today) |
| `Tail` (`redo`, `retrigger`, `retrigger_allowed`, `can_delete`) | tier 1 | Each rule is a §4.7 to §4.9 or §4.11 requirement. The scenarios 9.x, 10.x, 14.x and 22.x exist. A unit table only for a shape that has no scenario |
| `apply_world_rules` (§3.4) | **unit**: contract tests, or property tests, at the function | A pure function with a large input space: added rooms, one-shot Triggers, encounter conditions, R-16. Through tier 1, each case needs one scripted analysis and one route call |
| The `LlmRoles` adapter (§3.1 to §3.3, §3.5) | **unit**: contract tests with a fake `LlmProvider` that returns raw text | Tier 1 fakes this seam. So no tier-1 test reaches the parsers, the fallback of §3.2 or prompt assembly |
| `Storage` `story.rs` (§3.6, R-90, R-91) | **driven-adapter**: one contract suite that runs on both backends, with a failure injected in the middle of a unit | No route can stop a save unit half-way. One suite also replaces the 50 `_sqlite` and `_in_memory` twins |
| Sketch §6.2 rules 1 to 13 | guardrails and the compiler | Structure rules, not behaviour |

## 5. Three conditions

The proposal moves trust to tier 1. That is safe only if these three conditions hold.

1. **A strict fake.** A call that the test did not script fails the test. The fake has no default answer. Then "a Quantifier ran that should not" fails every test where it happens, with no tripwire.
   - With fork 4 = A (`LlmRoles`), the fake scripts results: "the analysis moves the player to the Mill".
   - With fork 4 = B (no new seam), `MockBackend` must fail on a call with no script, and tests script raw LLM text. Today `MockBackend` answers such a call with `"[Continuation: …]"` (research 10 §2a).
   - So fork 4 = A makes the fake easier to read. The strict rule does not need it.
2. **Arrange through the public write path.** A test arranges state through the routes, or through `commit(SaveUnit)`. It does not build rows or pointers by hand.
3. **A behaviour that matters has a scenario.** Tier-1 coverage is spec coverage. A behaviour with no scenario has no test. `validate_feature_spec.py` already checks that each scenario maps to a test.

## 6. Options

| Option | Tests | After a reshape | A red test names | Risk |
|---|---|---|---|---|
| A. Keep "unit tests cover everything" | most | the unit tests of replaced modules go | the module | two sources of truth that can disagree, as in §2.2 item 2 |
| **B. Tier 1 owns §4; unit tests only at the §3 seams** | fewer | tier 1 and the seam contracts stay | the scenario | a behaviour with no scenario has no test (condition 3) |
| C. Tier 1 only | fewest | everything stays | the scenario | world rules and parsers covered only through scripted turns: slow to write, weak at edge cases |

Lean: **B.** The cost is that a red test names a scenario, such as 9.2, and not a module, such as `Tail::redo`. Tier 1 runs in about 21 s and the LLM call log keeps every prompt. So the cost is small (inferred).

## 7. A proposed rule for `tests/STRATEGY.md`

> **Unit tier: only at a black box.** A unit test calls the interface of a module behind a `clean-spec.md` §3 seam: a pure rule, an LLM role adapter or a parser. It never calls an inner function. A behaviour that a tier-1 test asserts gets no unit test. If a unit test covers such a behaviour, a comment states what tier 1 cannot reach.

This text names `clean-spec.md`, which is a map asset today. A rule for the whole repo needs a stable source for the black boxes. That is an open point (§8).

## 8. Open points for the owner

1. **Scope.** Ticket 24's Scope puts "the build and test tooling" in the "Not touched" group. Is the rule for the designed scope only, or for the whole repo? A change for the whole repo is beyond this map's Destination.
2. **The test oracle.** The [platform choice](../../platform-choice/map.md) map's Destination names "the test oracle (black-box tests from the specs)". Tier 1 calls the router in-process today, so it cannot check a server in another language. Option B makes tier 1 the main oracle. Does this ticket decide only the placement, and leave the oracle to that map?
3. **Sketch §3.3 "Test seam".** The sketch proposes `Work::run` with a fake `LlmRoles` and the in-memory `Storage`, then `load_tail`. This proposal puts no tests at `Work`. Which one stands?
4. **Sketch §6.2 rule 11.** Its "How" column says: "a test checks the units of each kind in order". That test checks the order of the implementation, not an outcome. An alternative: tier-1 checks during the work, for example "the text shows before the analysis ends".
5. **Interleavings.** Can every interleaving of R-73 and R-74 be scheduled from the routes with a barrier fake? This is inferred. One prototype test would show it.
6. **Fork 4.** Does the strict fake change the lean of fork 4 (§5, condition 1)?
7. **Localization.** Is a red scenario, and not a red module, acceptable as the first signal?
8. **Today's tests outside the redesign.** Option B, applied to today's code, would remove duplicates there as well (for example the storage twins). Is that work for this map, for the `architecture-deepening` map, or for a later plan?
