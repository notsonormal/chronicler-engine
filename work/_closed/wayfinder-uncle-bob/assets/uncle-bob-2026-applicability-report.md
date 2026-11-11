# Uncle Bob 2026 GitHub Repository Applicability Study

**Scope:** All public repositories under `https://github.com/unclebob` that received git pushes in 2026.  
**Destination:** A first-pass assessment of which repositories (if any) are applicable to Chronicler Engine's codebase, architecture, or coding patterns.  
**Map:** `.scratch/wayfinder-uncle-bob/map.md`

## Executive summary

- **34 repositories** had meaningful push activity in 2026.
- **None are directly usable** in Chronicler Engine without a language-port or adapter layer; almost all are Clojure, Java, Go, or JavaScript projects.
- The **actionable value** is in patterns and workflows, not in the tools themselves.
- The map assessed some AI/agent repositories from **two perspectives**: whether they could shape Chronicler's runtime LLM/agent behavior, and whether they could improve development of Chronicler's own Rust codebase. The two perspectives yield different conclusions and are kept separate below.
- **Application-LLM / agent-runtime takeaways:** Multi-agent coordination models are not applicable to Chronicler's single-turn in-process pipeline. The useful pattern is explicit output schemas, design-by-contract validation, and structured diagnostics for agent results.
- **Code-development takeaways:**
  - **Test quality:** CRAP reporting and `cargo-mutants` mutation testing.
  - **Pre-flight structural checker:** A unified cheap-check step before the expensive test suite.
  - **Generated acceptance tests / QA telemetry:** `missile-command` and `empire-2025` show richer generated-acceptance and headless-QA workflows than Chronicler currently has.
- **Future spikes:**
  - Rust-native architecture visualizer inspired by `arch-view`.
  - Acceptance-mutation harness inspired by `Acceptance-Pipeline-Specification`.
  - Multi-agent development methodology from `swarm-forge` (likely poor fit, but worth a deliberate spike before ruling out).
  - Cucumber-style step definitions / generated acceptance tests from `Pharaoh-js` and `missile-command`.

## Applicability rubric

| Category | Meaning |
|---|---|
| **Directly usable** | Could be integrated or run against Chronicler Engine with reasonable effort. |
| **Practice/technique/tooling idea to adopt** | Concrete practice, tool, or implementation idea to translate into Rust/Chronicler. |
| **Already covered** | Chronicler Engine already has *equivalent* guardrails, tooling, or patterns — not merely adjacent ones. |
| **Not applicable** | Does not match Chronicler Engine's language, architecture, or workflow. |
| **Future spike** | Interesting but needs more investigation before a firm applicability call. |

## Perspectives

Some repositories were assessed twice because they touch both runtime behavior and development workflow:

1. **Application-LLM / agent runtime:** Could the tool or pattern run inside, or shape, Chronicler Engine's runtime LLM/agent behavior?
2. **Code development:** Could the tool or pattern be used to develop Chronicler Engine's own Rust codebase?

The two perspectives are reported separately. Repositories that are games, acceptance frameworks, or language-specific development tools were only assessed from the code-development perspective (or marked not applicable to both).

## Findings by perspective

### Application-LLM / agent-runtime perspective

Only AI/agent-related repositories are meaningfully assessed here; all others are not relevant to Chronicler's runtime LLM behavior.

| Repository | Category | Assessment |
|---|---|---|
| `AIR-J` | **Practice to adopt** | Canonical output artifacts, design-by-contract, and structured diagnostics should be applied to Chronicler's agent results, starting with the quantifier. |
| `swarm-forge` | **Not applicable** | Multi-agent worktree/handoff model does not fit Chronicler's single-turn in-process pipeline. Per-role backend selection is already supported, but the coordination model itself is not applicable. |
| `experiment-htw-go-swarm` | **Not applicable** | Hunt-the-Wumpus example built with SwarmForge; demonstrates the same coordination model as `swarm-forge`, which does not fit Chronicler's pipeline. |
| `experiment-htw-clj-swarm` | **Not applicable** | Same assessment as `experiment-htw-go-swarm`. |
| `speclj-structure-check` | **Not applicable** | A Claude skill for Clojure spec structure; not relevant to runtime LLM behavior. |

All other 2026 repositories were not assessed from the application-LLM perspective because they are games, acceptance-testing frameworks, or language-specific development tools.

### Code-development perspective

All 34 pushed repositories are assessed here.

#### Directly usable

_None._

#### Practice/technique/tooling idea to adopt

| Repository | Language | Assessment |
|---|---|---|
| `crap4java` / `crap4go` / `crap4clj` | Java / Go / Clojure | **CRAP formula and reporting workflow.** The CRAP metric is language-agnostic. A small Rust script combining `cargo llvm-cov` coverage with `syn`-based cyclomatic complexity could produce a risk radar before refactoring. Start as a report, not a gate. |
| `mutate4java` / `mutate4go` / `clj-mutate` | Java / Go / Clojure | **Mutation-testing workflow.** `cargo-mutants` already provides the equivalent capability for Rust. Best used as a pilot on one high-risk file, not as a per-PR gate. |
| `negative-test-experiment` | Clojure | **Lessons on coverage vs. test quality.** Confirms that acceptance tests alone are insufficient, that forcing CRAP < 4 raises coverage but lowers cleanliness, and that mutation testing catches operator-level gaps that coverage metrics miss. |
| `AIR-J` | Clojure / JVM | **Canonical representation and structured diagnostics for code.** The "one meaning, one representation" principle and machine-readable diagnostics can guide how AI assistants edit Rust code and report convention violations. |
| `speclj-structure-check` | Clojure | **Pre-flight structural checker as a Claude skill.** The transferable idea is a cheap, deterministic validation step before expensive tests. Chronicler already has the pieces; the improvement is packaging them into a single AI-facing pre-flight command. |
| `missile-command` | Clojure / ClojureScript | **Generated acceptance tests, QA telemetry scripts, and property-test separation.** Chronicler has feature specs and hand-written integration tests, but it does not generate acceptance entrypoints from Gherkin or run headless QA scenarios with scripted events. The pure-core/thin-host boundary is also a stronger form of layer separation than Chronicler currently enforces. |

#### Already covered

| Repository | Language | Assessment |
|---|---|---|
| `experiment-htw-go-swarm` / `experiment-htw-clj-swarm` | Go / Clojure | Hunt-the-Wumpus examples built with SwarmForge; they add no new patterns beyond those captured in the `swarm-forge` assessment. |

#### Not applicable

| Repository | Language | Assessment |
|---|---|---|
| `swarm-forge` | Clojure | The worktree-isolated multi-agent role pipeline is a different development methodology. Chronicler's current tooling covers the same quality goals more cheaply for a single-agent workflow. (Kept as a future spike if the team ever wants to explore multi-agent coding.) |
| `spacewar` | Clojure | Clojure Star Trek game with UI widgets and Speclj tests; no novel tooling or patterns beyond ordinary unit tests. |
| `Pharaoh` | Clojure | Historical 1988 Mac game rewrite; the relevant specification/testing patterns are captured by `missile-command` and the APS assessment. |
| `fitnesse` | Java | JVM-based acceptance-test wiki framework; Chronicler's Rust-native integration tests and feature-spec validation cover the same concerns without JVM overhead. |
| `gospringies` | Go | No description available; no apparent relevance. |
| `sf-orbit-simulator` | Java | Domain-specific orbit simulator; not applicable. |
| `dry4java` / `dry4go` / `dry4clj` | Java / Go / Clojure | DRY-principle helpers for those ecosystems; Chronicler's existing linting and guardrail tests cover equivalent structural concerns natively. |
| `deintroverter4clj` | Clojure | Clojure-specific refactoring helper; no Rust equivalent needed. |
| `scrap` | Clojure | Tool for assessing Speclj spec refactoring; Clojure-specific. |
| `skillBoard` | Clojure | Domain-specific flight-schedule application; no transferable tooling. |
| `springslim` | Java | Java/Spring Slim service; not applicable to Rust. |
| `htw-6-clj-vid` / `htw-clj-six-pack` | Clojure | Courseware/video companion repos for Hunt the Wumpus; no new patterns. |
| `craftsman-series` | None | Archive of old "Craftsman" series content; not code. |
| `ubc-website` | JavaScript | Consulting website; not applicable. |

#### Future spike

| Repository | Language | Assessment |
|---|---|---|
| `empire-2025` | Clojure | **Generated acceptance-test framework, headless QA telemetry, and spec-boundary guards.** Chronicler has integration tests and `validate_feature_spec.py`, but it does not generate test entrypoints from plain-text scenarios or run headless gameplay with scripted events and per-round logging. Worth a spike to see how much of the pipeline can be adapted to Rust. |
| `Pharaoh-js` | JavaScript | **Cucumber step definitions and generated JavaScript tests from Gherkin.** The step-definition pattern is portable; a Rust equivalent could map Gherkin steps to integration-test helpers and reduce hand-written test boilerplate. |
| `arch-view` | Clojure | **Rust-native architecture visualizer.** The Clojure implementation cannot parse Rust, but the idea of an auto-generated layered-dependency visualization with cycle highlighting would complement Chronicler's existing `arch-lint` rule enforcement. A spike based on `cargo metadata`, `syn`, or `rustdoc` JSON could produce a living architecture diagram. |
| `Acceptance-Pipeline-Specification` | Go | **Acceptance mutation harness.** The full pipeline (Gherkin → JSON IR → generated tests) is not a drop-in fit because Chronicler's tests are hand-written Rust, but mutating scenario example values and re-running the same tests is a useful quality workflow. A spike would need a Rust runner adapter around the existing integration-test harness. |
| `dependency-checker` | Clojure | **Main-sequence metrics (instability, abstractness, distance).** The rule-enforcement side is already covered by `arch-lint.toml` and syn-based guardrails; the novel contribution is the metrics. A Rust spike could compute these from the module graph, but the dependency-direction rules would be redundant. |

## Cross-perspective summary

| Repository | Application-LLM | Code development |
|---|---|---|
| `AIR-J` | Practice to adopt | Practice to adopt |
| `speclj-structure-check` | Not applicable | Practice to adopt |
| `swarm-forge` | Not applicable | Not applicable (future spike if multi-agent coding is explored) |
| `experiment-htw-go-swarm` | Not applicable | Already covered |
| `experiment-htw-clj-swarm` | Not applicable | Already covered |
| `missile-command` | — | Practice to adopt |
| `crap4java` / `crap4go` / `crap4clj` | — | Practice to adopt |
| `mutate4java` / `mutate4go` / `clj-mutate` | — | Practice to adopt |
| `negative-test-experiment` | — | Practice to adopt |
| `empire-2025` | — | Future spike |
| `Pharaoh-js` | — | Future spike |
| `Pharaoh` | — | Not applicable |
| `spacewar` | — | Not applicable |
| `fitnesse` | — | Not applicable |
| `arch-view` | — | Future spike |
| `Acceptance-Pipeline-Specification` | — | Future spike |
| `dependency-checker` | — | Future spike |
| All other 2026 repos | — | Not applicable |

## Deeper dives

### 1. Test quality: CRAP and mutation testing

**Sources:** `crap4java`, `crap4go`, `crap4clj`, `mutate4java`, `mutate4go`, `clj-mutate`, `negative-test-experiment`.

Chronicler Engine already uses `cargo llvm-cov` and `cargo nextest`, but it has no CRAP or mutation testing step. The Uncle Bob tools are language-specific, yet the underlying workflows are portable:

- **CRAP reporting.** Combine `cargo llvm-cov --json` function-level coverage with a `syn`-based cyclomatic-complexity pass to compute `CRAP = CC^2 * (1 - coverage)^3 + CC` per function. Emit a sorted risk radar. Run it as an informational report first; do not gate CI until the codebase is under the chosen threshold.
- **Mutation testing.** Use `cargo-mutants` on a small, high-risk file (e.g., quantifier agent or pipeline core). Cover or ignore uncovered mutation sites, kill survivors, then move to the next file. Avoid running it as a per-PR gate because the runtime cost is high.
- **Avoid embedded differential manifests.** Uncle Bob's mutation tools embed hashes in source-file footers to support incremental runs. This conflicts with Chronicler's cleanliness and formatting standards; prefer `cargo-mutants`'s file-level or git-diff filtering.

### 2. Application-LLM: agent output contracts and structured diagnostics

**Sources:** `AIR-J` (application-LLM perspective).

`AIR-J` is a programming language, not an LLM interaction library, but its principles apply to Chronicler's runtime agent layer:

- **Canonical output artifacts.** Agent prompts should produce responses in a single, unambiguous shape. The quantifier already asks for JSON only, but the schema is described in prose. A formal JSON Schema (or equivalent) plus a validator would reduce parse failures and make repair mechanical.
- **Design-by-contract.** Add explicit pre/post-conditions for agent results (e.g., quantifier confidence bounds, required fields) rather than relying only on system-instruction text.
- **Structured diagnostics.** Extend `LlmCallRecorder` or the agent executor to emit machine-readable validation failures (file/line/span/codes) so a downstream agent or skill can fix them without re-reading the entire response.

### 3. Code development: pre-flight structural checker

**Sources:** `speclj-structure-check`, existing Chronicler tooling.

Chronicler already has the checks that a pre-flight skill would run; they are spread across `scripts/` and `tests/infrastructure/guardrails/`:

- `scripts/check_test_structure.py`
- `scripts/validate_docs.py`
- `scripts/validate_feature_spec.py`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- Guardrail/architecture test binaries

A single `pre-flight` skill or script that runs the cheap checks in one command — before the expensive integration suite — would reduce AI-generated review/fix cycles. The output should use the existing `Violation` shape (file, line, severity, message) so repairs are mechanical.

### 4. Code development: richer acceptance testing and QA telemetry

**Sources:** `missile-command`, `empire-2025`, `Acceptance-Pipeline-Specification`, `Pharaoh-js`.

Chronicler has Markdown feature specs in `docs/specs/` and hand-written Rust integration tests. The Uncle Bob game projects go further:

- **Generated acceptance entrypoints.** `missile-command` parses Gherkin, emits a JSON IR, and generates executable test entrypoints. This is not directly portable to hand-written Rust tests, but it raises the question of whether Chronicler's feature specs could drive more boilerplate or traceability.
- **Cucumber step definitions.** `Pharaoh-js` uses JavaScript Cucumber with step definitions. A Rust equivalent could map Gherkin steps to test helpers, reducing duplication between feature specs and integration tests.
- **Headless QA telemetry.** `missile-command` and `empire-2025` support headless runs with scripted scenarios, telemetry, and event logs. Chronicler's tests exercise the HTTP API and storage directly, but they do not simulate a full turn-by-turn gameplay session from a scenario file.
- **Acceptance mutation.** APS mutates example values in scenarios and re-runs generated tests. A Rust version could mutate values in `docs/specs/*.md` and check that the annotated integration tests still fail appropriately.

These are all future spikes; they require design decisions about how much generated-test infrastructure Chronicler wants.

### 5. Code development: architecture visibility and metrics

**Sources:** `arch-view`, `dependency-checker`.

- **Architecture visualizer.** `arch-view` produces an interactive SVG layered-dependency graph. Chronicler has no equivalent visualization; only `arch-lint.toml` and text docs. A spike using `cargo metadata` or `rustdoc` JSON could fill this gap.
- **Main-sequence metrics.** `dependency-checker` computes instability, abstractness, and distance from the main sequence. The same idea could be applied to Rust modules, but Chronicler's layer rules already enforce dependency direction, so the metrics would be informational rather than gating.

## Recommendations

1. **Do not adopt any Uncle Bob tool directly.** All are bound to Clojure, Java, Go, or JavaScript and would require non-trivial adapter or port work.
2. **Pilot CRAP reporting and mutation testing.** Start with a lightweight CRAP script and a one-file `cargo-mutants` pilot. Treat both as informational quality signals before making them gates.
3. **Formalize runtime agent output schemas and validation.** Apply the `AIR-J` canonical-output and design-by-contract patterns to Chronicler's LLM agents, beginning with the quantifier.
4. **Package existing development checks into a pre-flight skill.** Combine the cheap structural/formatting/guardrail checks into one AI-facing command so agents catch mistakes before `cargo nextest` runs.
5. **Spike richer acceptance-test workflows.** Evaluate whether generated acceptance entrypoints, Cucumber-style step definitions, or headless scenario telemetry would reduce test-maintenance burden for Chronicler.
6. **Keep multi-agent development methodology as a future spike, not a current adoption.** `swarm-forge` is likely a poor fit for Chronicler's current single-agent workflow, but it deserves a deliberate small spike before being permanently ruled out.

## Repository coverage summary

| Perspective | Category | Count |
|---|---|---|
| Application-LLM | Directly usable | 0 |
| Application-LLM | Practice to adopt | 1 (`AIR-J`) |
| Application-LLM | Already covered | 0 |
| Application-LLM | Not applicable | 4 (`swarm-forge`, `experiment-htw-go-swarm`, `experiment-htw-clj-swarm`, `speclj-structure-check`) |
| Code development | Directly usable | 0 |
| Code development | Practice to adopt | 6 (`crap4java`/`crap4go`/`crap4clj`, `mutate4java`/`mutate4go`/`clj-mutate`, `negative-test-experiment`, `AIR-J`, `speclj-structure-check`, `missile-command`) |
| Code development | Already covered | 2 (`experiment-htw-go-swarm`, `experiment-htw-clj-swarm`) |
| Code development | Not applicable | 14 (`swarm-forge`, `spacewar`, `Pharaoh`, `fitnesse`, `gospringies`, `sf-orbit-simulator`, `dry4java`/`dry4go`/`dry4clj`, `deintroverter4clj`, `scrap`, `skillBoard`, `springslim`, `craftsman-series`, `ubc-website`, `htw-6-clj-vid`/`htw-clj-six-pack`) |
| Code development | Future spike | 5 (`empire-2025`, `Pharaoh-js`, `arch-view`, `Acceptance-Pipeline-Specification`, `dependency-checker`) |
