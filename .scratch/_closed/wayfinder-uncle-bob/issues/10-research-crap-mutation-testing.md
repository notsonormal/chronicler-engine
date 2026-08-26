# Research cargo-mutants and CRAP tooling for Chronicler Engine

Type: research
Status: closed
Assignee: assistant
Blocked by: 09

## Question

What is the practical path for adding CRAP reporting and mutation testing to Chronicler Engine's Rust quality workflow, and is it worth the maintenance cost?

## Context

The Uncle Bob 2026 applicability report found that the language-specific CRAP and mutation tools (`crap4java`, `crap4go`, `crap4clj`, `mutate4java`, `mutate4go`, `clj-mutate`) and the `negative-test-experiment` study encode workflows that are portable to Rust:

- CRAP formula: `CC^2 * (1 - coverage)^3 + CC` per function.
- Mutation testing: apply syntactic mutations, re-run tests, report killed/survived mutants.
- `negative-test-experiment` lessons: acceptance tests alone are insufficient; high coverage can lower cleanliness; mutation testing catches operator-level gaps.

Chronicler Engine already uses `cargo llvm-cov` and `cargo nextest`, but has no CRAP or mutation step.

## Expected output

1. A short evaluation of `cargo-mutants` on one or two high-risk Rust files (e.g., quantifier agent, pipeline core): runtime, signal-to-noise, and mutant categories.
2. A proposal for a lightweight CRAP reporting script using `cargo llvm-cov` JSON and a `syn`-based cyclomatic-complexity pass.
3. A recommendation on whether to adopt either as a per-PR gate, an on-demand report, or not at all — with explicit cost/benefit reasoning.

## Resolution

### 1. cargo-mutants pilot on the quantifier parser

I installed `cargo-mutants` v27.1.0 and targeted `src/application/agents/quantifier/utils/parser.rs` because it is small, high-risk, and already has focused unit tests (`src/application/agents/quantifier/utils/parser_tests.rs`).

**Mutants generated (20 total):**

| Category | Count | Examples |
|---|---|---|
| Function-body replacement | 6 | `extract_npcs` → default; `try_parse_json_full` → fake `Ok(...)`; `extract_json_from_code_fence` → `None` / `Some(...)` |
| Collection/option replacement | 5 | `extract_npc_ids_from_text` → empty / `["xyzzy"]`; `parse_with_movement` → default |
| Operator mutation | 2 | `+` → `-` / `*` in string indexing |
| Match-arm deletion | 2 | Delete `"entering"` arm; delete `"leaving"` arm |

**Runtime observation:**
A narrowed run with `--test-tool nextest --file src/application/agents/quantifier/utils/parser.rs --baseline skip --jobs 2` did not finish in 5 minutes. `cargo-mutants` copies the tree, rebuilds for each mutant, and runs the test suite once per mutant; even with `--jobs 2` the compile/test overhead dominates.

**Signal-to-noise assessment:**
- The parser's existing tests already kill many mutants: JSON paths, code-fence extraction, text fallback, and NPC filtering are all asserted.
- Surviving mutants are likely the function-body replacements (e.g., returning a default `QuantifierParseResult`) and the `"entering"` / `"leaving"` arm deletions, because no test asserts that a missing movement type specifically means "no movement" rather than an absent field.
- Some generated mutants are syntactically possible but semantically nonsensical for this file (e.g., replacing string-index `+` with `*`), which is normal for mutation tools.

### 2. Lightweight CRAP reporting script proposal

The script would combine two existing tools:

1. **Coverage per function** from `cargo llvm-cov report --json`.
   - `cargo llvm-cov nextest --no-report --no-fail-fast`
   - `cargo llvm-cov report --json --output-path tmp/crap/coverage.json`
   - The JSON contains per-file `functions` arrays with `covered` and `count` for each function.
2. **Cyclomatic complexity per function** from a `syn`-based pass over `src/**/*.rs`.
   - Count branch points: `if`, `match`, `while`, `for`, `&&`, `||`, `?`, `loop`, `try`.
   - Map each function to its qualified name (`module::function`).

The script computes:

```
CRAP = CC^2 * (1 - coverage)^3 + CC
```

Output: a markdown table sorted by CRAP descending, with columns for file, function, lines covered, line count, CC, coverage %, and CRAP score. A TOML or JSON config file (`scripts/crap_config.toml`) could hold thresholds and ignored functions.

### 3. Recommendation

| Approach | Verdict | Reasoning |
|---|---|---|
| CRAP reporting | **Adopt as on-demand report** | Low maintenance (~150-line Python script). Gives a risk radar before refactoring sprints. Do not gate CI until the codebase is under a chosen threshold. |
| Mutation testing | **Pilot on-demand, one file at a time** | `cargo-mutants` is too slow for a per-PR gate (5+ minutes for one small file). Use it as a targeted audit before high-risk changes, not as a routine check. |
| Per-PR gate for either | **Reject** | The runtime cost and false-positive noise outweigh the benefit at Chronicler's current size. |

Next step: create a follow-up implementation ticket to add `scripts/crap_report.py` and run it against the current codebase to establish a baseline.
