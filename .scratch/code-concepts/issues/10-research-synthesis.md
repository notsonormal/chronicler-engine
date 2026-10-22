# Research: merge the areas and find where concepts meet across them

Type: research
Status: resolved
Blocked by: 02, 03, 04, 05, 06, 07, 08, 09

## Question

When the area assets are put together, which concepts hold across the whole codebase, which hold in some areas only, and where do opposing or overlapping concepts meet *across* areas? Where do the docs (ticket 09) name a concept that the code does not follow, or miss one that it does?

## Done when

`assets/research-10-synthesis.md` has one table of concepts with a column per area, a list of the meeting points across areas (each a question with evidence from both sides), the shapes with no name that more than one area found, and the questions from every area asset, merged and grouped. It does not rule on any of them.

## Answer

Assets: [research-10-synthesis.md](../assets/research-10-synthesis.md) (the merge of areas 02–09) and [research-10-docs-vs-code.md](../assets/research-10-docs-vs-code.md) (the docs against the code). Two researchers wrote them in parallel. Neither asset rules on a concept.

- **97 merged concepts** (19 stance, 58 pattern, 20 idiom). 13 hold in five or more of the seven code areas, 75 in two to four, 9 in one. 87 more single-area concepts are listed by reference, so all 398 area rows are reachable. The widest: test hooks compiled into production behind the default-on `testing` feature (all seven code areas); fail fast, with its opposite (degrade or skip) in the same areas; ports and adapters with no port for storage; the dependency rule; concrete type over trait; sibling `*_tests.rs` files.
- **22 meeting points across areas** (XM-1 to XM-22), each a question. The strongest: four atomicity mechanisms (`worlds.rs:175-182` uses a transaction, `update_world` does not); two writers of one Game record (`gate.rs:240` against `catalogue.rs:81-85`); settings read per call, at boot (`wiring.rs:161-166`) or "from a global static" that does not exist; stop, skip or fall back on a bad input at boot; which fake stands for the model (`MockBackend` in `tests/STRATEGY.md:15` against `ScriptedModel`).
- **17 shapes with no name** that more than one area found; **114 merged questions** in ten themes.
- **Docs against code:** 34 concepts that the docs name and the code follows; 23 questions where the code follows a doc concept in part or not at all; 25 concepts that the code follows and the docs do not name. The strongest: `architecture_system.md:21` "port traits only" against line 30 "may call `Storage` directly" (nine application files import storage); "settings read at the point of use" against the boot-time Agent list and a seed that parses `settings.json` on every boot; `startup.md:35` "skip a bad file" against a seed that stops on a bad map, persona or preset; the `testing` feature, which no doc names; the `guardrails.md` rule numbers, which come from two `.scratch` plans.
- **Citation re-check:** all 153 citations of asset 09 into the six edited docs still hold. Asset 09 has no row for the new `AutoplayRun` ownership row or for the run hold as a second source of "busy".

**Check by the coordinator:** six citations against the code: nine non-test files in `src/application/` import `adapters::driven::storage`; `Cargo.toml` has `default = ["testing"]`; `architecture_system.md:21` and `:30`; `worlds.rs:175-182`; `wiring.rs:161-166` (`.unwrap_or_default()`); `deepseek.rs:25-30`. All match.

**Open (known unknowns):** whether a release package ships the `testing` feature (`build.py` has no `--features` flag, so the default is on; packaging not traced); what happens to a running Job at shutdown. P37 (Decorator) is a guessed merge of three wrappers.
