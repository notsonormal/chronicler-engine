# Assess CRAP and mutation testing tools for test strategy

Type: research
Status: resolved
Assignee: pi
Blocked by: 01

## Question

What do `crap4java`, `crap4go`, `crap4clj`, `mutate4java`, `mutate4go`, `clj-mutate`, and `negative-test-experiment` measure or demonstrate about test quality, and can any of those metrics or workflows be applied to Chronicler Engine's Rust test suite?

## Answer

### 1. What each tool/measure does

**CRAP (Change Risk Anti-Pattern) — crap4java, crap4go, crap4clj**

All three compute the same metric per function/method:

```
CRAP = CC^2 * (1 - coverage)^3 + CC
```

- `CC` = cyclomatic complexity (decision points + 1).
- `coverage` = fraction of instructions/forms/statements covered by tests.
- A threshold of `8.0` is used as a gate; anything above is flagged as high-risk.

The implementations differ only in source parsing and coverage ingestion:

- `crap4java` parses Java methods with the JDK compiler tree API and reads JaCoCo `INSTRUCTION` counters.
- `crap4go` parses Go with `go/ast`, reads `go test -coverprofile` output, and counts `if`, `for`, `range`, `switch`/case, `select`/comm, `&&`, and `||`.
- `crap4clj` reads Cloverage LCOV/HTML output and counts `if`/`when`/`cond`/`and`/`or` clauses, `loop`, `catch`, etc.

All report a sorted table (worst CRAP first) and exit non-zero when the threshold is exceeded.

**Mutation testing — mutate4java, mutate4go, clj-mutate**

These tools apply small syntactic mutations to one source file at a time, re-run the file's tests, and report whether each mutant is killed, survived, uncovered, or timed out. They share a common rule set:

| Category | Mutations |
|---|---|
| Arithmetic | `+` ↔ `-`, `*` ↔ `/` (Go/Java); `inc` ↔ `dec` also (Clojure) |
| Comparison | `>` ↔ `>=`, `<` ↔ `<=` |
| Equality | `==` ↔ `!=` / `=` ↔ `not=` |
| Logical | `&&` ↔ `||` |
| Boolean | `true` ↔ `false` |
| Conditional | `if` ↔ `if-not`, `when` ↔ `when-not` (Clojure) |
| Constant | `0` ↔ `1` |
| Unary | `!expr` → `expr`, `-expr` → `expr` (Java) |
| Reference | rvalue → `null` (Java) |

All three support:

- AST-based discovery (so comments/string literals are not mutated).
- Coverage filtering to skip uncovered mutation sites.
- Differential mutation via an embedded footer manifest (hashes of functions/declaration scopes), so repeated runs only retest changed scopes.
- `--scan` to list sites without running tests.
- `--update-manifest` to refresh the embedded manifest.
- `--mutate-all` to ignore the manifest.
- `--max-workers` parallel isolated worker copies.
- A default mutation-warning threshold of 50 sites.

**negative-test-experiment**

This is not a tool; it is a controlled experiment in which Hunt the Wumpus was implemented eight times under two axes:

1. Test discipline: three-laws, test-last, bundling, none.
2. Whether CRAP was forced below 4 after implementation.

Each program then grew a second test suite via mutation testing.

Key findings from `experiment-summary.md` / `experiment-conclusion.md`:

- All eight programs passed the same acceptance suite (25/0), even the one with zero unit tests.
- Forcing CRAP < 4 split complex functions and/or added tests, raising line coverage to ~97–99% but consistently lowering cleanliness scores.
- Mutation testing produced operator-level tests (flipping `if`/`if-not`, `=`/`not=`, `1`/`0`, etc.) that the original discipline suites often missed.
- The best combined outcome was **bundling with CRAP off** (design 4, coverage 5, cleanliness 4).
- Mutation, unlike CRAP, did not change the program design; it only revealed holes in the existing tests.

### 2. Comparison table

| Tool / Study | Language | Measures | Mutation rules | Coverage source | Differential? | Directly usable for Chronicler? |
|---|---|---|---|---|---|---|
| `crap4java` | Java | CRAP per method | n/a | JaCoCo XML | No | No — Java/Maven only. |
| `crap4go` | Go | CRAP per function | n/a | `go test -coverprofile` | No | No — Go only, but formula is language-agnostic. |
| `crap4clj` | Clojure | CRAP per function | n/a | Cloverage LCOV/HTML | No | No — Clojure only. |
| `mutate4java` | Java | Mutation score per file | ~10 AST rules | JaCoCo line | Yes (embedded manifest) | No — Java/Maven only. |
| `mutate4go` | Go | Mutation score per file | ~9 AST rules | Go coverprofile | Yes (embedded manifest) | No — Go only. |
| `clj-mutate` | Clojure | Mutation score per file | ~14 rules + suppression | Cloverage LCOV | Yes (embedded manifest) | No — Clojure/Speclj only. |
| `negative-test-experiment` | Clojure | Discipline × CRAP × mutation | n/a | Cloverage | n/a | No — it is a study, not a tool. |

### 3. Chronicler Engine's current test setup

- Tests are run from `build.py` (repository root, not `scripts/build.py`).
- `build.py` uses `cargo nextest run --no-fail-fast` and optionally `cargo llvm-cov nextest` for coverage.
- Unit tests live in sibling `*_tests.rs` files inside `src/` and must be registered in the module root; `scripts/check_test_structure.py` enforces this.
- Integration tests live in `tests/` and mirror `src/` paths within each test binary.
- There are architecture/guardrail test binaries (`tests/infrastructure/architecture.rs`, `tests/infrastructure/guardrails/mod.rs`) that check module boundaries, style, and structural rules.
- LLM tests are `#[ignore]`'d and run separately with `python build.py --llm-only`.
- `.config/nextest.toml` sets 4 test threads, 1 retry, and 60 s slow-timeout (300 s for LLM profile).
- `scripts/coverage_summary.py` consumes `cargo-llvm-cov` JSON and prints overall coverage plus files below 80%.
- There is no CRAP or mutation testing step today.

### 4. Rust equivalents

For CRAP:

- `cargo-tarpaulin` or `cargo llvm-cov` already produces line/function coverage.
- Cyclomatic complexity can be computed with `cargo-cyclonedx`, `cargo-metrics`, or a small `syn`-based parser (the project already depends on `syn` for guardrail tests).
- Combining the two into a CRAP report is straightforward and could be added to `scripts/` or as a small standalone crate.

For mutation testing:

- `cargo-mutants` is the mature Rust mutation-testing tool. It mutates source, runs `cargo test`, reports survived/killed mutants, and supports source filtering. It does not embed manifests in source files, but it can run incrementally against `git diff`.
- There is no direct Rust port of Uncle Bob's embedded-manifest differential model, but `cargo-mutants -d <file>` gives a similar single-file workflow.

### 5. Applicability ratings

| Item | Rating | Rationale |
|---|---|---|
| `crap4java` / `crap4go` / `crap4clj` as tools | **Irrelevant** | Language-specific; cannot run against Rust source or `cargo` coverage. |
| CRAP formula and workflow | **Pattern to adopt** | The formula is simple and language-agnostic. A Rust script could compute it from `cargo llvm-cov` + `syn` complexity. Useful as a risk radar before refactoring. |
| `mutate4java` / `mutate4go` / `clj-mutate` as tools | **Irrelevant** | Language-specific AST mutators. |
| Mutation-testing workflow | **Pattern to adopt** | `cargo-mutants` can run the same workflow today. Best used on small, risky files after unit tests pass, not as a per-PR gate. |
| Embedded differential manifest | **Pattern to consider** | Clever for incremental runs, but polluting source files with machine-generated footers conflicts with the repo's existing cleanliness standards. `cargo-mutants`'s file-level filtering is a simpler substitute. |
| `negative-test-experiment` lessons | **Pattern to adopt** | Confirms that acceptance tests alone are insufficient, that CRAP improves coverage at the cost of cleanliness, and that mutation testing finds test gaps that coverage metrics miss. |
| Threshold gating at CRAP > 8 | **Pattern to consider** | Would add noise unless the codebase is first brought under the threshold. Start with reporting only. |

### 6. Recommendations

1. **Do not adopt the Java/Go/Clojure tools.** They are language-bound and would require a translation layer that is not justified when Rust tooling exists.

2. **Introduce a lightweight CRAP report script** (e.g., `scripts/crap_report.py`) that:
   - Runs `cargo llvm-cov --json`.
   - Parses each Rust function/method with `syn` to compute cyclomatic complexity.
   - Computes `CRAP = CC^2 * (1 - coverage)^3 + CC` per function.
   - Prints the top-N worst functions.
   - Initially runs only on demand and in CI as a report, not a gate.

3. **Try `cargo-mutants` on one high-risk file** as a pilot, for example a quantifier agent or pipeline core file. The workflow should mirror Uncle Bob's recommendation: run on one file, cover/ignore uncovered mutations, kill survivors, then move to the next file.

4. **Keep the embedded-manifest idea on hold.** It is elegant for incremental mutation, but Chronicler Engine already has strong test structure and nextest filtering conventions; adding machine-generated footers to source files would create formatting/noise issues.

5. **Use the `negative-test-experiment` finding as guidance, not a mandate.** The experiment shows that high coverage does not guarantee good tests and that mutation testing catches operator-level blind spots. This supports adding mutation testing selectively, but not forcing CRAP < 4 as a hard rule, because the cleanliness cost was high.

### 7. No new tickets recommended

No specifiable follow-up emerged from this research. Any future work (e.g., a `cargo-mutants` pilot or a CRAP report script) should be created as a deliberate task with explicit scope, not inferred from this ticket.
