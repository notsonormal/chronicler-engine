# Research per-function cyclomatic-complexity and coverage sources for a Rust CRAP script

Type: research
Status: resolved
Assignee: pi (GLM-5.2 session)
Blocked by:

## Question

How do we obtain **per-function cyclomatic complexity** and **per-function coverage** for Chronicler Engine's Rust code, in a form that can be joined by a stable function identity, so a CRAP script can compute `CRAP = CC^2 * (1 - coverage)^3 + CC` per function?

## Context

The prior research (`.scratch/wayfinder-uncle-bob/issues/10-research-crap-mutation-testing.md`) proposed combining `cargo llvm-cov` JSON with a `syn`-based cyclomatic-complexity pass, but did not verify the join. Existing Chronicler coverage scripts (`scripts/parse_coverage.py`, `scripts/coverage_summary.py`) read only file-level totals, not per-function data.

Candidate CC sources to evaluate:

1. **Mozilla `rust-code-analysis`** — a standalone CLI/JSON tool that computes cyclomatic complexity per function for Rust. Verify it installs cleanly and emits stable function names/paths.
2. **A small `syn`-based Rust binary** — most accurate but adds a build dependency and a second artifact to maintain.
3. **A Python heuristic** over `src/**/*.rs` (count `if`/`match`/`while`/`for`/`&&`/`||`/`?`/`loop` per function) — cheapest and consistent with `scripts/` conventions, but brittle on macros and nested items.

Candidate coverage source: `cargo llvm-cov` JSON **without** `--summary-only`, which should expose a `functions` array per file. Confirm the function names/regions are stable enough to join to the CC source.

## Expected output

1. A concrete recommendation for the **CC source** and the **coverage source**, with the join key (e.g., `file::function_name` or `file::line_range`).
2. Evidence: actual sample output from each candidate run against one or two `src/` files (e.g., `src/application/agents/quantifier/utils/parser.rs`), showing the joinable fields.
3. A note on failure modes: how macro-generated functions, generic monomorphization, and nested items are represented (or hidden) in each source.
4. A verdict on whether the CRAP script can stay a single Python script or needs a companion Rust binary.

This is pure research; no implementation. The answer feeds ticket 02 (script design) and ticket 04 (implementation).

## Answer

All evidence below was produced by running the candidate tools against
`src/application/agents/quantifier/utils/parser.rs` (the file the prior research
mutated), plus a project-wide scan of the coverage JSON. Research artifacts are
under `tmp/crap/` (`cc_heuristic.py`, `coverage.json`, `rca_install.log`); these
are scratch, not shipped.

### 1. Recommendation: CC source, coverage source, join key

**Coverage source — `cargo llvm-cov` full JSON (already produced by `build.py`).**
`cargo llvm-cov report --json` (no `--summary-only`, no `--skip-functions`) emits
`data[0].functions[]`. Each entry has:
- `name` — a Rust v0 mangled symbol (e.g. `...6parser12extract_npcs`).
- `filenames` — array of source files (one per instantiation; usually one).
- `count` — execution count.
- `regions` — `[[start_line, col, end_line, col, count, ...], ...]`; per-function
  coverage = `regions_with_count_gt_0 / total_regions`.

The per-file `functions` array is empty; the function list lives at the top level
`data[0].functions`. `build.py --coverage` already writes this file to
`target/llvm-cov/coverage.json` (or the `--target-dir` override) using
`cargo llvm-cov report --json --output-path ... --ignore-filename-regex ...`.
The CRAP script reuses that same file; no separate coverage run is needed.

**CC source — Python heuristic, for the pilot.** A `match` keyword counts as 1,
not per arm, so CC is a lower bound on match-heavy code (documented caveat).
Matches `scripts/` conventions; no build dependency; one file.

Rejection of the other two CC candidates:
- **Mozilla `rust-code-analysis` — rejected, does not build.** `cargo install
  rust-code-analysis-cli` (latest, v0.0.25) fails on Rust 1.88 with
  `error[E0308]: mismatched types` in `macros.rs:42`: two transitive
  `tree-sitter` versions (`0.20.9` vs `0.26.12`) produce incompatible `Language`
  types. The crate is unmaintained (last release 0.0.25). See
  `tmp/crap/rca_install.log`.
- **`syn`-based Rust binary — accurate upgrade path, not the pilot default.**
  It counts `match` arms (the dominant Rust branch construct), giving true CC, but
  adds a `syn` build dependency and a second artifact to maintain. Recommended as
  the upgrade IF the baseline (ticket 04) shows the heuristic's match undercount
  hides real risk. Recorded in the map's Not-yet-specified.

**Join key — `(file, bare_function_name)`.** The bare name comes from a stateful
v0-tail parse of the coverage `name` (walk the string; a digit-run is a length
prefix; consume exactly that many identifier chars as a segment; the bare name
is the last valid segment). The same bare name is the `fn` ident from the CC
pass. Closures are dropped (see failure modes). Coverage regions are aggregated
across monomorphization entries that share `(file, bare_name)` before joining.

### 2. Evidence

Coverage side, `data[0].functions` filtered to the parser file (9 entries; the
first 5 are named functions, the last 4 are closures):

```
name (v0 tail)                       regions   cov   span
extract_npcs                         24/24     100%  24-57
parse_with_movement                  36/49     73%   139-205
try_parse_json_full                  28/39     72%   59-112
extract_npc_ids_from_text             17/17    100%  126-137
extract_json_from_code_fence          21/22     95%  114-124
<closure in extract_npcs>              3/3     100%  31-31
<closure in parse_with_movement>       3/3     100%  158-158
<closure in parse_with_movement>       6/6     100%  167-177
<closure in parse_with_movement>       7/7     100%  170-174
```

CC side, the Python heuristic (`tmp/crap/cc_heuristic.py`) on the same file:

```
name                                start-end   CC
extract_npcs                        24-57       3
try_parse_json_full                 61-112      7
extract_json_from_code_fence        114-124     3
extract_npc_ids_from_text           126-137     3
parse_with_movement                 142-205     3
```

v0-tail parse is 100% successful project-wide: 3688 function entries → 3313
named, 375 closures, 0 unparseable. All 5 parser named-functions match the
heuristic names exactly.

Joined CRAP (coverage named-only, closure regions folded into parent,
`CRAP = CC^2 * (1-cov)^3 + CC`):

```
function                           CC     cov      CRAP
extract_npcs                        3   100.0%    3.00
try_parse_json_full                 7    71.8%    8.10
extract_json_from_code_fence        3    95.5%    3.00
extract_npc_ids_from_text            3   100.0%    3.00
parse_with_movement                 3    80.0%    3.07
```

The join works end-to-end. `try_parse_json_full` (highest CC + imperfect
coverage) correctly surfaces as the highest-CRAP function. `parse_with_movement`
shows the heuristic's match-arm undercount in action: it is a 64-line match-heavy
function but scores CC 3 because `match` counts once, not per arm.

### 3. Failure modes

- **Closures.** They appear as separate coverage entries whose v0 name parses
  to the PARENT's bare name (e.g. the closure in `extract_npcs` parses to
  `extract_npcs`), so they collide on the join. Detectable by the v0 closure
  disambiguator pattern `0Bd_|0Bf_|Cs_\d`; the script drops them and folds their
  regions into the parent (a closure only runs if its parent does, so parent
  coverage already reflects whether it was exercised).
- **Generic monomorphization.** One source `fn` becomes multiple coverage
  entries (e.g. `partial_cmp` ×21 in a dependency). Project-wide, 490 of 2355
  named `(file, name)` groups have >1 coverage entry. The script must aggregate
  regions across entries sharing `(file, bare_name)` before computing coverage.
- **Macro-generated functions** (derive macros, `#[async_trait]`, etc.). They
  appear in coverage but not in source, so they are unmatched on the coverage
  side and dropped. Acceptable: they are not hand-written CRAP targets.
- **Dependency code.** 897 of 3688 function entries come from cargo-registry
  dependencies (`http`, `uri`, etc.), and 3 from stdlib. `build.py`'s
  `--ignore-filename-regex` does not exclude them. The CRAP script must apply its
  own positive filter: `filenames` containing the project `src/` path.
- **Match-arm undercount (heuristic only).** A `match` adds 1 to CC, not one
  per arm, so CC is understated on match-heavy code — the dominant Rust branch
  construct. The CRAP signal is weakened exactly where it matters most, but the
  coverage term `(1-cov)^3` still surfaces low-coverage match-heavy functions.
  Documented caveat; the `syn` binary is the fix.
- **Same-file name collisions.** Two `fn new` in different `impl` blocks in one
  file share `(file, name)` but have distinct line spans; the join would merge
  them. Rare; mitigate by also requiring line-span overlap, or accept the small
  error. No collision occurred in the parser file.
- **Line-range join is a fallback, not the primary.** Coverage's first region
  starts at the `pub`/`fn` signature line; the heuristic's `start_line` is the
  opening brace, so start lines diverge for multi-line signatures (off by 2-3).
  End lines match exactly. A `(file, end_line)` join is feasible but shares the
  same-file-collision risk; `(file, name)` is cleaner for the report and is the
  recommended primary key.

### 4. Verdict: single Python script, no companion binary (for the pilot)

The CRAP script stays a single Python file in `scripts/`. It reads the existing
`build.py --coverage` JSON (no new coverage step), runs the Python heuristic over
`src/**/*.rs`, joins on `(file, bare_name)` with v0-tail parsing + closure
filtering + monomorphization aggregation + project-source filtering, and emits the
CRAP table. No `syn` binary, no `rust-code-analysis` dependency.

The trade-off, stated for ticket 02 to carry into the design: the heuristic
undercounts `match` arms, so CRAP is a lower bound on match-heavy modules. This
is acceptable for an on-demand radar (not a gate), and the `syn` binary remains
the documented upgrade if the baseline shows the undercount hides real risk.

### Inputs to downstream tickets

- **Ticket 02 (design):** join key `(file, bare_name)`; v0-tail parse;
  closure filter `0Bd_|0Bf_|Cs_\d` with region fold-to-parent; monomorphization
  aggregation; project-`src/` filename filter; coverage = covered/total regions;
  documented match-arm caveat; output = markdown table sorted by CRAP desc.
- **Ticket 04 (implement):** reuse `target/llvm-cov/coverage.json` from
  `build.py --coverage`; the `--ignore-filename-regex` does not replace the
  project-source filter; follow `scripts/check_python_docstrings.py` (module
  docstring required).
