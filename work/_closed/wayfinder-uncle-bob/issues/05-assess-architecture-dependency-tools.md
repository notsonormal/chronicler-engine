# Assess architecture and dependency tooling for code health

Type: research
Status: resolved
Assignee: pi
Blocked by: 01

## Question

What do `arch-view`, `dependency-checker`, and `Acceptance-Pipeline-Specification` offer for understanding architecture, enforcing dependency rules, or specifying acceptance behavior, and are any applicable to Chronicler Engine's layered architecture or testing approach?

## Answer

### Repository summaries

#### `arch-view` (https://github.com/unclebob/arch-view)

`arch-view` is a Clojure architecture visualizer. It scans `.clj`/`.cljc`/`.cljs` source files, reads each file's `ns` form, and extracts `:require` dependencies to build a namespace dependency graph. It marks namespaces containing `defprotocol`, `defmulti`, or `definterface` as abstract, detects dependency cycles, removes cycle edges so the remaining graph can be topologically ranked into layers, and renders an interactive SVG diagram. The diagram shows high-level namespaces at the top, leaf/source modules at the bottom, dependency triangles, and cycles listed at the bottom. It can also export the architecture graph to EDN for headless consumption.

Key limitations for Chronicler Engine:
- Source scanner is Clojure-specific; it reads Clojure `ns` forms and cannot parse Rust modules.
- Component concept is implicit (derived from namespace segments), not configurable.

#### `dependency-checker` (https://github.com/unclebob/dependency-checker)

`dependency-checker` is a Clojure dependency-rule enforcer. It scans Clojure source files, derives components from the second namespace segment, and checks whether component-to-component dependencies match a configured `:allowed-dependencies` map. It supports `:forbidden-dependencies`, namespace-level `:allowed-exceptions`, `:ignored-components`, `:fail-on-violations`, and `:fail-on-cycles`. It reports per-component fan-in, fan-out, instability, abstractness, and distance from the main sequence ("Zone of Pain"/"Zone of Uselessness"). It can also generate a starter config by inferring observed dependencies.

Key limitations for Chronicler Engine:
- Dependency extraction is Clojure-specific (`ns` clauses, `(require ...)`, dynamic lookups like `requiring-resolve`).
- Component discovery assumes a Clojure namespace hierarchy.

#### `Acceptance-Pipeline-Specification` (https://github.com/unclebob/Acceptance-Pipeline-Specification)

The Acceptance Pipeline Specification (APS) is a language-neutral specification for turning Gherkin feature files into executable acceptance tests. The pipeline is:

```text
feature file -> Gherkin parser -> JSON IR -> entrypoint generator -> generated tests -> project test runner
```

It defines:
- A supported Gherkin subset (`Feature:`, `Background:`, `Scenario:`, `Scenario Outline:`, `Examples:`, `Given/When/Then/And`).
- A canonical JSON IR shape for features, scenarios, steps, and example tables.
- An IR-DRY checker that reports duplicated or similar step text.
- A project-specific entrypoint generator contract.
- A runtime contract that expands scenarios, prepends background steps, resolves placeholders, and routes steps to project-specific handlers.
- An acceptance mutator that mutates example values in the JSON IR (not application source code) and runs the same generated tests against the mutated IR to check whether changed specification data is detected.
- A persistent runner-worker protocol over stdin/stdout newline-delimited JSON.

The spec is portable, but every concrete project must write its own parser adapter, entrypoint generator, runtime, step handlers, and runner adapter.

### Comparison against Chronicler Engine

#### Architecture and dependency direction

Chronicler Engine already enforces a hexagonal/layered architecture through `arch-lint.toml` and the guardrail test suite:

- `domain/model` must not depend on `application`, `narrative`, or `server`.
- `application` must not depend on `server`.
- Port traits live in `application/ports`; driven adapters implement them.
- Driven adapters must not depend on the application layer.
- Adapters must not depend on each other.
- Storage DB models/mappers are isolated from all other layers.

These rules are machine-checked by `arch-lint` and by custom `syn`-based walkers in `tests/infrastructure/guardrails/`.

#### What Chronicler already does that these tools cover

| Tool capability | Chronicler equivalent | Status |
|---|---|---|
| Layered architecture visualization | None (text docs + `arch-lint.toml` only) | Gap |
| Dependency-cycle detection | `arch-lint` scope rules, cargo build cycle errors | Covered |
| Component dependency rules | `arch-lint.toml` `[[deny-scope-dep]]` | Covered |
| Forbidden/allowed dependency lists | `arch-lint.toml` `[[deny-scope-dep]]` | Covered |
| Fan-in/fan-out and main-sequence metrics | None | Gap |
| Gherkin feature specs | `docs/specs/*.md` | Covered |
| Spec-to-test traceability | `scripts/validate_feature_spec.py` | Covered |
| Acceptance mutation | None | Gap |
| Generated acceptance test entrypoints | N/A — tests are hand-written Rust | Not applicable |

#### Acceptance testing conventions in Chronicler

Chronicler's specs are written as Markdown files with Gherkin scenarios (`docs/specs/*.md`). `scripts/validate_feature_spec.py` parses scenario IDs and checks that every declared scenario has a covering integration test annotated with `// [path/to/spec.md] SCENARIO: X.Y` in `tests/http/` or `tests/browser/behaviour.rs`. Integration tests use a mix of Rust-only `tower::ServiceExt::oneshot` HTTP tests, SQLite-backed service tests, real `chronicler_engine` binary tests driven by Playwright, and storage-direct tests.

This is a Rust-native, hand-written acceptance harness. It does not generate test entrypoints from a JSON IR and does not perform acceptance mutation.

### Applicability ratings

| Repository | Rating | Rationale |
|---|---|---|
| `arch-view` | **Pattern to adopt** | The Clojure implementation cannot be used directly on a Rust crate, but the idea of an auto-generated layered-dependency visualization with cycle highlighting would complement Chronicler's existing `arch-lint` rule enforcement. A Rust equivalent (e.g., based on `cargo metadata`, `syn`, or `rustdoc` JSON) could produce the same interactive view. |
| `dependency-checker` | **Pattern to adopt (metrics only); otherwise covered** | The rule-enforcement side is already handled by `arch-lint.toml` and the syn-based guardrails. The novel contribution is the main-sequence metrics (instability, abstractness, distance). A Rust tool could compute those metrics from the module graph, but the dependency-direction rules themselves would be redundant. |
| `Acceptance-Pipeline-Specification` | **Pattern to adopt (acceptance mutation); otherwise not directly usable** | The full APS pipeline is designed for projects that generate test entrypoints from Gherkin IR. Chronicler's tests are hand-written Rust, so the parser/generator/runtime contract is not a drop-in fit. The acceptance-mutation concept (mutate example values, run the same tests, report killed/survived/errors) is the most transferable idea, but it would require a custom Rust runner adapter and a way to feed mutated scenario data into the existing test suite. |

### Conclusion and recommendations

1. **Do not adopt the Clojure tools directly.** All three repositories are Clojure-centric or require project-specific adapters that do not exist for Rust.

2. **Keep the existing `arch-lint` + guardrail stack.** Chronicler already enforces the dependency-direction rules that `dependency-checker` provides, and it does so at the Rust module level with `deny-scope-dep` and custom `syn` walkers. Switching to a Clojure-derived tool would be a regression in precision.

3. **Consider a Rust-native architecture visualizer as a future enhancement.** A tool that reads `cargo metadata` or `rustdoc` JSON and emits a layered SVG/module graph with cycle highlighting would fill the visualization gap left by `arch-view`. This would be documentation/infrastructure, not a code change, and should be prototyped only if the team wants a living architecture diagram.

4. **Consider acceptance mutation as a quality workflow, not a tool import.** The APS mutator is conceptually useful — it would check whether Chronicler's scenario examples actually exercise the application by mutating example values and expecting tests to fail. However, implementing it means writing a Rust runner adapter around the existing integration-test harness, which is a non-trivial project-specific build. Treat it as a candidate for a future quality-improvement spike rather than a direct adoption.

5. **No immediate code changes or new tool dependencies are warranted.** The research outcome is architectural insight, not an integration task. The existing conventions (hexagonal layers, `arch-lint`, spec-to-test traceability) already cover the parts of these repositories that would be directly valuable.

No follow-up tickets are recommended at this time; the applicable ideas would require their own spikes with clear scope before implementation.
