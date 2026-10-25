# Research: the concepts in the application layer

Type: research
Status: resolved

## Question

Which concepts does the code in `src/application/` (about 5,100 lines without the unit tests) follow, and where do opposing or overlapping concepts meet in it?

Write the asset in the shape that the map's Notes give (Concepts, Meeting points, Shapes with no name, Questions). Do not rule on a concept or label a meeting point; write it as a question.

Dig the concepts out of the code. You get no list of concepts: do not open `assets/research-01-lens.md` or the other research assets. Read the code, and find the shapes that it repeats and the choices that it makes. Give each one the name that software-engineering books and well-known essays give it (a principle, an architecture style, a design style, a pattern or an idiom), and its opposite. When no such name fits, put the shape under Shapes with no name.

The Job, the writer rule and the Agents were rebuilt recently (`.scratch/_closed/rust-rebuild-build/`). Where old and new shapes sit side by side, record both and say which commit or plan explains it, if one does.

## Done when

Every module in the scope is read, or is named in the asset as sampled with the reason. Every row cites `path:line` evidence and has a known, inferred or guessed label.

## Answer

Asset: [research-03-application.md](../assets/research-03-application.md). The researcher read all 54 non-test files in `src/application/` (5,083 lines) and sampled nothing. The researcher did not open the other research assets. Of the tests, it read only `agents/runner_tests.rs` (lines 1-50) and the test names in `games/catalogue_tests.rs`. Claims about the composition root (`bootstrap/wiring.rs`, out of the area) are labelled inferred.

- **35 concepts.** The five with the strongest evidence (known): the single writer principle, with a capability handle, a fencing token, lock ordering and RAII drop guards (all in `generation/gate.rs`); functional core, imperative shell (`game_rules::apply` has one caller; the narrator takes values, not storage); the dependency rule (`agents/`, `narration/`, `prompting/` import no `adapters::`); errors as values (zero `.unwrap()`; outcome sum types such as `Refusal`, `JobFailure`, `Busy`); tolerant reader with bounded retry in the two Agents.
- **19 meeting points, each written as a question.** Examples: the LLM and text check have ports, but storage has none and nine files take `Arc<Storage>`; `GameWriter` says it is "the only code that opens a unit", but the catalogues call `Storage` directly; three error envelopes meet (`ApplicationError`, `JobFailure`/`Refusal`, bare `EngineError`); fail fast in the Job, degrade in the Options prompt, swallow in the after-narration hook (plan ticket 20, commits `e2483e0c`, `aed5eb9d`).
- **13 shapes with no name, 12 questions.**
- **Hazards to check (inferred, not tested):** `rename_game` does not go through `config_write`, so it may run while an Autoplay run holds the Game; `delete_game` and `switch_game` read the active pointer outside the writer's lock; `GameViewQuery::status` reads two stores in two steps; `JobWriter::drop` takes the non-reentrant registry lock, and no type stops a drop inside a `hold` closure.
- **Check by the coordinator:** spot check of four citations (`pipeline/job.rs:334`, `narration/narrator.rs:27-42`, `gate.rs:243`, `rename_game`) against the code. All four match. The researcher corrected some line numbers after a multi-file `cat -n` and did not re-check every other citation.
