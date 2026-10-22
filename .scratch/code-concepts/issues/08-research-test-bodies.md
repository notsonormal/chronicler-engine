# Research: the concepts in the test bodies

Type: research
Status: resolved

## Question

Which concepts does the code in the `*_tests.rs` unit tests in `src/` (about 19,000 lines) and `tests/http/`, `tests/storage/`, `tests/llm/`, `tests/bootstrap/` and `tests/browser/` (about 26,000 lines) follow, and where do opposing or overlapping concepts meet in it?

Write the asset in the shape that the map's Notes give (Concepts, Meeting points, Shapes with no name, Questions). Do not rule on a concept or label a meeting point; write it as a question.

Dig the concepts out of the code. You get no list of concepts: do not open `assets/research-01-lens.md` or the other research assets. Read the code, and find the shapes that it repeats and the choices that it makes. Give each one the name that software-engineering books and well-known essays give it (a principle, an architecture style, a design style, a pattern or an idiom), and its opposite. When no such name fits, put the shape under Shapes with no name.

This is too much to read whole. Sample each folder, and say in the asset how you chose the sample. Look for the shapes a test author can choose: what a test arranges (by running the engine, or by seeding records), what it fakes, what it asserts (observable behaviour, or internals), table or property tests, and naming.

## Done when

Every module in the scope is read, or is named in the asset as sampled with the reason. Every row cites `path:line` evidence and has a known, inferred or guessed label.

## Answer

Asset: [research-08-test-bodies.md](../assets/research-08-test-bodies.md). The researcher read whole files or large parts of about 25 files (the tier-1 support, the three seam-contract unit files, four storage files, the bootstrap and LLM files, the browser runner) and sampled about 35 more (heads, name lists, one or two bodies each). These file counts are approximate. About 55 of the 87 unit-test files and most tier-1 and stub-browser bodies were only counted by grep. The asset names every file in each group and says why. The researcher did not open the other research assets. The researcher also opened `tests/STRATEGY.md` and the unit-test allowlist for context, and cites them only where the code shows the same choice. The researcher checked the 282 citations with a script. The script found 3 that bound to the wrong file (a `:line` shorthand after another path); the researcher wrote them in full. No cited line was out of range.

- **47 concepts.** The strongest (known): test tiers split by what is faked, with one test binary per tier; ports and adapters as the test seam (drive the router with `oneshot`, fake only the LLM port with the strict `ScriptedModel`); real in-memory SQLite as the storage double and sociable tests; behavioural tests at tier 1 that read a read seam and not `Storage`; Specification by Example (272 of 272 tier-1 tests carry a `SCENARIO` tag); contract tests per seam with `CASE` tags; fault injection at the storage seam; latch and barrier control (`hold`, `release`, `Barrier`) for races; test hooks inside the production crate behind a default-on `testing` feature; a ratchet (quarantine folder and legacy unit list; 78 of 87 unit-test files are on it).
- **23 meeting points, each written as a question.** Examples: three LLM-port fakes serve the unit tier (strict `ScriptedModel`, lenient `MockBackend`, hand-written stubs); the same handler behaviour has a unit test and a tier-1 test (text-check save, set narrator, duplicate preset); storage behaviour has two homes, and 20 `test_sqlite_` tests are twins of another test that differ only in the storage constructor; five constructors build a test storage; two ways to wire a test app (the builder and the production composition root); a process-wide `SettingsTestGuard` is taken 66 times although each test builds its own storage; four ways to wait (latch, poll, event, sleep); three ways to handle a missing prerequisite; five ways to assert an error; three ways to make a failure (fail a call, fail a write, corrupt the record); the Deprecated Term "Turn" appears in test code.
- **10 shapes with no name, 11 questions.** Examples of shapes: the "wiring guard" browser test; the tagged-value mapping check; the in-test design table `STOP_ROWS`.
- **Hazards to check (inferred, not tested):**
  - `Cargo.toml:55` sets `default = ["testing"]`, so the storage failure hooks and the test builders may compile into a default build.
  - `tests/bootstrap/run_branches.rs:14-50` binds ports 3010-3050 and deletes database files next to the test executable; two runs from two checkouts may meet.
  - `tests/llm/flow_llm_tests.rs:50-53` returns early (a pass) when the API key is absent; the `#[ignore]` on the same tests hides this until it is lifted.
  - `src/adapters/driven/storage/llm_messages_tests.rs:76` orders two rows with a real 10 ms sleep.
  - `tests/test_utils/stub_server.rs:44-47` serves four canned HTML fragments; they can drift from the templates (`tests/STRATEGY.md:197` names this cost).
  - `src/adapters/driving/http/action/handlers/actions_tests.rs:16-54` tests `ActionForm` serde from JSON, while the handler takes a form-encoded body; the tests may pin nothing that production reads.

- **Check by the coordinator:** spot check of four citations against the code. All match (known). `src/adapters/driven/storage/llm_messages_tests.rs:76` sleeps 10 ms between two saves. `tests/llm/flow_llm_tests.rs:50-53` prints "Skipping" and returns when the key is absent. `tests/storage/world_storage.rs:431` and `:657` are the twin tests, and `tests/helpers/fixtures.rs:7-11` shows that `create_test_storage` is SQLite `:memory:` plus one seeded Game row. The `default = ["testing"]` hazard is the same one that [the test infrastructure](07-research-test-infrastructure.md) names; it stays inferred.
