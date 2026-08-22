# Quick-pass assess remaining 2026 repository groups

Type: research
Status: resolved
Assignee: pi
Blocked by:

## Question

Do the remaining unassessed 2026-pushed repositories — game/examples (`missile-command`, `spacewar`, `Pharaoh`, `Pharaoh-js`, `empire-2025`) and acceptance testing (`fitnesse`) — fall into "Not applicable" / "Already covered," or do any of them deserve a deeper assessment ticket?

## Context

The applicability rubric has been decided in [Determine applicability scoring rubric and final output format](./08-determine-scoring-rubric-and-output-format.md):

| Category | Meaning |
|---|---|
| Directly usable | Could be integrated or run against Chronicler Engine with reasonable effort. |
| Practice/technique/tooling idea to adopt | Concrete practice, tool, or implementation idea to translate into Rust/Chronicler. |
| Already covered | Chronicler Engine already has equivalent guardrails, tooling, or patterns. |
| Not applicable | Does not match Chronicler Engine's language, architecture, or workflow. |
| Future spike | Interesting but needs more investigation before a firm applicability call. |

Seven repository groups have already been assessed in closed tickets. The remaining groups are:

- **Games / examples**: `missile-command`, `spacewar`, `Pharaoh`, `Pharaoh-js`, `empire-2025`
- **Acceptance / specification**: `fitnesse` (note: `Acceptance-Pipeline-Specification` was already assessed in [Assess architecture and dependency tooling for code health](./05-assess-architecture-dependency-tools.md))

This ticket is a quick pass only: enough to categorize each repo and decide whether any need a dedicated deeper ticket before the final report.

## Answer

All six remaining repositories are game implementations or acceptance-testing frameworks. None introduce patterns that Chronicler Engine does not already cover, and none are directly usable in Rust. No follow-up tickets are needed; the results feed directly into the final report.

### Assessment by repository

| Repository | High-level purpose | Category | Rationale | Follow-up? |
|---|---|---|---|---|
| `missile-command` | Clojure/ClojureScript remake of Atari *Missile Command* with a pure game core, thin Quil hosts, Gherkin feature specs, generated acceptance tests, architecture boundary checks, QA telemetry scripts, and property tests. | **Already covered** | The pure-core/thin-host architecture, feature specs, and architecture-boundary enforcement mirror Chronicler's layered architecture, `docs/specs/`, `arch-lint.toml`, and guardrail tests. The Clojure-specific tooling (Babashka, Speclj, APS) is not portable to Rust. | No |
| `spacewar` | Clojure *Star Trek* game with UI widgets, game logic, and Speclj tests. | **Not applicable** | A Clojure game implementation with no novel tooling or patterns beyond ordinary unit tests; language and domain do not match Chronicler. | No |
| `Pharaoh` | Clojure rewrite of a 1988 Macintosh simulation game, preserving original C source and using Gherkin feature specs. | **Not applicable** | Historical game rewrite; the relevant specification/testing patterns are already captured by `missile-command` and the APS assessment. | No |
| `Pharaoh-js` | JavaScript rewrite of the same *Pharaoh* game, driven by Cucumber features and Jest unit tests. | **Already covered** | Same game/spec as the Clojure version; Cucumber+Jest coverage is analogous to Chronicler's existing feature-spec validation and Rust integration-test harness. | No |
| `empire-2025` | Clojure wargame with extensive acceptance-test framework, dependency-checker integration, spec-boundary guards, headless mode, and AI behavior logging. | **Already covered** | The architecture/dependency tooling and acceptance-mutation ideas were already assessed via `dependency-checker` and `Acceptance-Pipeline-Specification`; `empire-2025` is a consumer of those patterns, not a new source. Chronicler already has equivalent guardrails and integration tests. | No |
| `fitnesse` | Java acceptance-test wiki framework. | **Not applicable** | JVM-based wiki/acceptance framework; Chronicler's Rust-native integration tests and feature-spec validation cover the same concerns without JVM overhead. | No |

### Perspective notes

- **Application-LLM / agent behavior:** None of these repositories contain patterns that would run inside or shape Chronicler Engine's runtime LLM/agent pipeline. They are games or testing frameworks, not agent-orchestration tools.
- **Code-development tooling:** The only transferable ideas (feature specs, architecture boundary enforcement, dependency checking, acceptance mutation) have already been assessed in earlier tickets. The remaining repositories either repeat those patterns in a Clojure/Java game context or are pure game code.

### Conclusion

No repository in the remaining groups escapes the **Not applicable** / **Already covered** categories. The final applicability report can therefore summarize these six briefly, without dedicated deep-dive tickets, and focus depth on the **Practice/technique/tooling idea to adopt** and **Future spike** items identified in earlier tickets.
