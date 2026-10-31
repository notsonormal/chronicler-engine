# Map: Uncle Bob quality tools in build.py

Labels: wayfinder:map

## Destination

`build.py` runs tools equivalent to Uncle Bob's [crapper](https://github.com/unclebob/crapper) (CRAP score), [mutator](https://github.com/unclebob/mutator) (mutation testing) and [dryer](https://github.com/unclebob/dryer) (structural duplicates) for this Rust codebase. A tool goes in only after an evaluation shows that it has value.

## Notes

- **Domain.** Dev-loop quality tooling: `build.py`, `scripts/`, `tests/infrastructure/guardrails/`, coverage (`cargo llvm-cov`), the build slot (`scripts/build_slot.py`, `ENVIRONMENT.md`).
- **This map carries execution.** Wayfinder's plan-only default is overridden: a ticket may build a tool. One ticket per session.
- **Any tool is allowed** (user, 2026-10-25): upstream as is, vendored, ported, or a third-party engine such as `cargo-mutants`.
- **A partial equivalent is no reason to skip a tool.** jscpd (`build.py duplicates`) and the `nesting.rs` guardrail exist; they can shape a tool, not cancel it.
- **The user sets each value bar after seeing real output.** Evaluation tickets are AFK research: they run the tool and explain what the output means. Each one graduates a grilling ticket in which the user decides value. Do not ask for a bar before there is output to look at.
- **Scope of analysis.** Production code under `src/`. Skip `*_tests.rs` siblings and `src/test_support/`. Upstream tools skip only `mod tests { }` and directories named `test`/`tests`, so they score this repo's test files unless told not to.
- **Build slot.** Every cargo compile or test run goes through `build.py` or the build slot. Never run `cargo test` bare: it runs the browser tier.
- **Shared tree.** Other agents edit this checkout at the same time. A mutation run must never leave a mutated file, and must not use `git stash`.
- **Prior work.** [Research asset](./assets/research-crapper-mutator-dryer.md) (2026-10-25: first runs of all three tools on a copy of `src/`); closed map [`crap-mutation-pilot`](../_closed/crap-mutation-pilot/map.md) (its ticket 01 holds the join evidence); closed map [`wayfinder-uncle-bob`](../_closed/wayfinder-uncle-bob/map.md). Related, not moved: retro-issue-bucket [23 — unconditional mutation check](../retro-issue-bucket/issues/23-unconditional-mutation-check.md) decides *when* a mutation check is required.
- **Skills.** `/research` for the AFK tickets; `/grilling` and `/domain-modeling` for the value tickets; `/codebase-design` when a tool's form is chosen; `/tdd` and `/code-review` for a build ticket.
- **Scratch.** `tmp/unclebob/` holds the clones, a venv with the three tools installed, and the first run output. The system `python3` has no `pip`; the venv was bootstrapped with `get-pip.py`.

## Decisions so far

<!-- one line per closed ticket: [title](link) — gist -->

- [Research: what a CRAP report shows on this codebase](./issues/01-research-crap-report.md) — on fresh coverage the report runs in 4 s; whether `?` counts as a branch decides the ranking (it is 25.7 % of all CC and puts 7 `from_row` lists in the top 20); 85.5 % of functions are fully covered, so the report is mostly a complexity list; no branch coverage on this toolchain. Value decision graduated to [Grilling: does the CRAP report have value here, and which CC rule?](./issues/05-grilling-crap-report-value.md).
- [Research: which mutation engine works on this codebase, and what it shows](./issues/03-research-mutation-engines.md) — both cargo-mutants and a driver over `build.py test-pattern` run (in a worktree); 10 survivors on `parser.rs` and `gate.rs`, all equivalent, one sign of redundant code; cargo-mutants has no tree proof and leaves a mutant after `SIGKILL`, the driver does not. Graduated to [Grilling: does mutation testing have value here, with which engine?](./issues/06-grilling-mutation-testing-value.md) and [Grilling: how does a mutation run keep its result and its tree in step?](./issues/07-grilling-mutation-shared-tree-safety.md).
- [Research: what a structural duplicate report shows beside jscpd](./issues/02-research-duplicate-report.md) — dryer finds 52 production pairs at defaults: 10 real (3 that the gate's jscpd misses; its printed summary shows only 1 of the 10), 42 idiom noise that 7 structural rules remove; dryer has no ignore mechanism; test code gives 409 + 1,016 pairs. Value decision graduated to [Grilling: does the duplicate report have value here…](./issues/04-grilling-duplicate-report-value.md).

## Not yet specified

- **Form of each kept tool:** upstream, vendored, or a port (`syn` in Rust beside the guardrails, Python in `scripts/`, or both).
- **Place in `build.py`** for each kept tool: on-demand subcommand, report-only full-gate step scoped to changed files (as `duplicates`), or a gate. The closed pilot and the 2026 study said "never a gate"; the user left this open.
- **Home:** `scripts/healthcheck.py` subcommand (it already dispatches `duplicates`) or a separate script.
- **Differential runs:** per-function hash snapshots so a run redoes only changed functions (mutator does this; `cargo-mutants` has `--iterate` and `--in-diff`).

## Out of scope

- The `.metrics/*.edn` snapshot format and Uncle Bob's `uml-viewer`.
- Languages other than Rust (the upstream tools also cover Clojure, Java, Go, TypeScript, Python).
