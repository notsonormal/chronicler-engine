# Research: `crapper`, `mutator` and `dryer` for Chronicler Engine

Date: 2026-10-25. Scratch evidence: `tmp/unclebob/` (clones, venv, run output; git-ignored).

## What the three repositories are

| Repo | First commit | Last commit | Commits | `src/` size | Replaces |
|---|---|---|---|---|---|
| [crapper](https://github.com/unclebob/crapper) | 2026-09-30 | 2026-10-21 | 9 | ~3,400 lines Python | `crap4java`, `crap4go`, `crap4clj` |
| [mutator](https://github.com/unclebob/mutator) | 2026-09-30 | 2026-10-21 | 11 | ~3,240 lines Python | `mutate4java`, `mutate4go`, `clj-mutate` |
| [dryer](https://github.com/unclebob/dryer) | 2026-09-30 | 2026-10-03 | 5 | ~1,900 lines Python | `dry4clj`, `dry4java`, `dry4go` |

- All three are Python 3.11+, use `tree-sitter` + `tree-sitter-language-pack` (the pack downloads the grammar on first use), and are 0BSD-licensed: copy, port or vendor with no attribution duty.
- All three now support Rust. The earlier catalogue (`.scratch/_closed/wayfinder-uncle-bob/issues/01-catalog-2026-repos.md`) saw only the per-language predecessors, none for Rust.
- They share one identity model: a function is `(namespace, name)`, with Rust namespace `crate::module` or `crate::module::Type`. They write `.metrics/*.edn` for Uncle Bob's `uml-viewer`. Chronicler has no use for the EDN or the viewer.
- Maturity (inferred): small, young, one author, fast-moving. `mutator/bugs/` holds an open bug note. Treat them as reference designs, not as dependencies.

## crapper

**Mechanism.** tree-sitter walks each `.rs` file. A function is a `function_item` with a body, not nested in a block, not in `mod tests`. CC = 1 + each `if`, `for`, `while`, `loop`, `match` arm, `?`, `&&`, `||`. Coverage is LCOV: per-function coverage = covered `DA` lines inside the function's line range (branch records first, when present). CRAP = CC² × (1 − cov)³ + CC.

**What it solves that the closed pilot did not.** Ticket 01 of `crap-mutation-pilot` chose a Python regex heuristic that counts `match` once (CC undercount) and a join on demangled v0 symbol names (closure folding, monomorphization merge, same-name collisions). crapper removes both problems:

1. It counts `match` arms, from a real parse.
2. It joins on **file + line range**, not on symbol names. No demangling, no collisions.

**Run on Chronicler (copy of `src/`, `*_tests.rs` removed).** 2.5 s for 223 files. With coverage from `target/test-police-cov/llvm-cov/coverage.json` (2026-10-23, converted to LCOV by `tmp/unclebob/cov2lcov.py`, approximate):

| Function | CC | Cov | CRAP | Note |
|---|---|---|---|---|
| `plumbing::migrate_up_to` | 200 | 97% | 201 | CC is mostly `?`; one long migration list |
| `wiring::build_service_with_storage` | 9 | 0% | 90 | stale or uninstrumented file in that coverage run |
| `run::prepare_data` | 13 | 64% | 21 | real hotspot candidate |
| `load::process_world_dir` | 20 | 87% | 21 | real hotspot candidate |
| `stories::StoryReads::story_row` | 10 | 53% | 20 | real hotspot candidate |
| `models::game::DbGame::from_row` | 19 | 100% | 19 | CC is 1 + one `?` per column |

**Gaps for this repo (known, from the run):**

- It scores `*_tests.rs` sibling files as production code. Chronicler declares tests as `mod foo_tests;` siblings, not `mod tests { }`. A port must skip `*_tests.rs` and `src/test_support/`.
- Counting `?` inflates every `from_row` and `migrate_up_to`. Line coverage marks the `?` line covered when only the `Ok` path ran, so the score hides the untested error path. Decide: count `?`, drop `?`, or count `?` only in CC and not in the ranking.
- Coverage: it runs `cargo llvm-cov --lcov` itself, outside the build slot. `build.py --coverage` already writes `coverage.json` via `cargo llvm-cov report --json`. One more `cargo llvm-cov report --lcov` line in `build.py` gives LCOV with no test rerun; or a port reads the JSON segments directly.

## mutator

**Mechanism.** tree-sitter finds operator sites: `&&`/`||`, `!` delete, unary `-` delete, comparisons (`<`→`<=`, `==`→`!=` …), `+`/`-`, `*`/`/`, `true`/`false`, `0`/`1`. It skips sites on lines that coverage does not hit (reports them as *uncovered*). It runs the baseline test command once, then each mutant with timeout = 10 × baseline. It stores a per-function **text hash**. The next run is differential: it reruns survivors and every site in a function whose hash changed; killed mutants in unchanged functions stay killed.

**Isolation.** Each worker is a symlink overlay under `target/mutation-workers/`, with a private copy of the mutated file. The overlay **does not link `target/`**. A backup under `target/mutator-backup/` is restored after a crash.

**Run on Chronicler (`--scan`, no tests run).**

| File | Lines | mutator sites |
|---|---|---|
| `quantifier/utils/parser.rs` | 206 | 2 (cargo-mutants made 20 on the same file) |
| `generation/gate.rs` | 911 | 21 |
| `pipeline/job.rs` | 482 | 7 |
| `utils/game_rules.rs` | 284 | 12 |
| `tail.rs` | 73 | 5 |

**Fit for this repo:**

- **Do not run it as is.** Each worker overlay has no `target/`, so every worker cold-builds the crate. The default Rust command is `cargo test`, which runs every binary, including the browser tier. Both conflict with the build slot (`scripts/build_slot.py`) and `ENVIRONMENT.md` memory limits.
- Operator-only sites are thin for idiomatic Rust: little arithmetic, many `match`, `?` and `Option` combinators. cargo-mutants also replaces function bodies with defaults and deletes match arms — the mutants that found gaps in the prior pilot.
- Ideas worth taking: (1) skip uncovered sites and report them separately; (2) the per-function hash for differential reruns; (3) restore-from-backup on start; (4) baseline-derived timeout. Ideas 1–3 match option A of retro-issue-bucket ticket 83 (moved into this map; evidence in [03](../issues/03-research-mutation-engines.md)).

## dryer

**I did not know `dryer` before this session.** It is five commits old (2026-09-30 to 2026-10-03). Its predecessors `dry4clj`/`dry4java`/`dry4go` are in the earlier catalogue, which rated them "Not applicable" because they had no Rust support. That verdict is now stale.

**Mechanism.** Per function, it normalizes the syntax tree: keeps call/method names, operators and tree shape; drops local names, field names, literals. The fingerprint set is the printed form of every subtree. Score = Jaccard of the two sets. Defaults: score ≥ 0.82, ≥ 4 lines, ≥ 20 normalized nodes. All pairs are compared within one language.

**Run on Chronicler.** 4.2 s, 61 pairs. The repo already runs jscpd 5.4.2 (`scripts/healthcheck.py duplicates`, 2,201 fragments in `report/jscpd-report.json`). **49 of the 61 dryer pairs do not overlap any jscpd fragment.** Of those 49:

- About 30 are idiom noise: builder setters (`fn data(mut self, …) -> Self`), `enum` → `&'static str` matches, `from_row` column lists.
- Some are real near-clones that jscpd misses because names and literals differ, for example `assembler.rs` `render_game_state_layer` vs `render_world_info_layer` (1.00), and `storage/games.rs` vs `storage/personas.rs` reads (0.84).
- `ollama.rs::complete` = `openrouter.rs::complete` (1.00) is found by both tools.

**Fit.** Function-level pairs are a short, reviewable list; jscpd is a long fragment list. dryer complements jscpd; it does not replace it. It needs an ignore list for builder setters and `as_str`-style matches, and should skip `*_tests.rs` and `src/test_support/` (as for crapper).

## Reimplement: where and how

The repo already parses `src/` with `syn` (dev-dependency, `full` + `visit`) in `tests/infrastructure/guardrails/`. `nesting.rs` already visits `if`/`match`/`for`/`while`/`loop` with span lines. The Python `scripts/` use only the standard library plus `jsonschema` and `yaml`; this machine's `python3` has no `pip`.

| Option | New dependency | CC and fingerprint quality | Fit with the repo |
|---|---|---|---|
| A. Vendor or `pip install` the Python tools | `tree-sitter`, `tree-sitter-language-pack`, network grammar download | Good | Poor: new Python deps, EDN output, wrong defaults for test files and the build slot |
| B. Port the Rust subset to stdlib Python | None; needs a hand-written Rust parser | Weak (back to the regex heuristic of the closed pilot) | Good for `scripts/`, but the parse is the hard part |
| C. Port to Rust on `syn`, beside the guardrails walkers | None (already a dev-dependency) | Good (real AST; match arms, closures, spans) | Good: reuses the existing walker code; runs as a cargo example or an `#[ignore]` report test, through `build.py` |
| D. Mixed: `syn` for CC and dry fingerprints; Python stdlib for the LCOV join, report and mutation driver | None | Good | Good: parse in Rust, orchestration in `scripts/`, mutation runs through `build.py test-pattern` (slot-aware, warm `target/`) |

Rough port size for the Rust subset (inferred from the source): CC ≈ 80 lines of `syn` visitor; CRAP join ≈ 100 lines; dry normalizer + Jaccard ≈ 200 lines; mutation driver (sites via `syn` spans, in-place apply, hash check, restore, `build.py test-pattern`) ≈ 250 lines.

## Open questions

This asset was written before the map. The open questions now live on the [map](../map.md) (*Not yet specified*) and in tickets 01–03.
