# Uncle Bob repository applicability study

## Destination

A curated list of Robert C. Martin's GitHub repositories updated in 2026, with a first-pass assessment of which ones (if any) are applicable to Chronicler Engine's codebase, architecture, or coding patterns.

## Notes

- Domain: Chronicler Engine — Rust interactive-fiction/text-adventure engine, LLM-powered narrative generation, HTTP/WebSocket server, HTMX dashboard, layered/hexagonal-ish architecture.
- Framing: some repositories (e.g., `swarm-forge`, `AIR-J`) are assessed from two perspectives:
  1. **Application-LLM**: could the tool/pattern run inside or shape Chronicler Engine's runtime LLM/agent behavior?
  2. **Code-development**: could the tool/pattern be used to develop Chronicler Engine's own Rust codebase?
  Both perspectives may produce separate tickets and should be kept when relevant.
- Skills to consult: research, grilling, domain-modeling, code-consistency-check.
- Standing preferences:
  - Initial catalog is limited to repositories with activity in 2026.
  - Older repositories are out of scope unless a 2026 repo explicitly depends on one.
  - This effort overrides the default plan-only mode for AFK research tickets: pure catalog/research tickets may be executed in the same session they are created, limited to one ticket per session.
  - Execution follow-ups: this map now carries the most concrete findings forward as research tickets. Prototyping and implementation remain out of scope for this map and should move to future maps once research resolves.

## Decisions so far

- [Catalog Uncle Bob's 2026-updated GitHub repositories](./issues/01-catalog-2026-repos.md) — 94 total public repos; 34 had code pushes in 2026. The meaningful 2026 set is the 34 pushed repos, grouped into AI/agent coordination, test-quality metrics, architecture/dependency tooling, acceptance/specification, games/examples, refactoring helpers, and course/website/legacy buckets.
- [Assess SwarmForge for agent coordination patterns](./issues/02-assess-swarm-forge.md) — From the application-LLM/agent-orchestration perspective: SwarmForge's multi-agent worktree/handoff model is not applicable to Chronicler Engine's single-turn in-process agent pipeline; the only transferable idea is structured/layered role prompts, which the existing prompt-preset system already partially covers.
- [Assess CRAP and mutation testing tools for test strategy](./issues/03-assess-crap-mutation-tools.md) — The language-specific CRAP and mutation tools are not directly usable, but the CRAP formula and mutation-testing workflow are applicable; `cargo-mutants` and a lightweight CRAP script would be the practical adoption path.
- [Assess AI-language and skill patterns for LLM interaction](./issues/04-assess-ai-language-skill-patterns.md) — From the application-LLM/prompting perspective: `AIR-J` and `speclj-structure-check` are not directly usable; the transferable patterns are canonical output artifacts, design-by-contract validation, and structured diagnostics for agent results.
- [Assess architecture and dependency tooling for code health](./issues/05-assess-architecture-dependency-tools.md) — `arch-view`, `dependency-checker`, and `Acceptance-Pipeline-Specification` are not directly usable; Chronicler already has layer rules and spec-to-test traceability, so only a Rust-native architecture visualizer or acceptance-mutation harness would be future spikes.
- [Assess SwarmForge as multi-agent development methodology for Chronicler Engine](./issues/06-assess-swarm-forge-dev-methodology.md) — From the code-development perspective: SwarmForge's worktree-isolated role pipeline is a poor fit; Chronicler's existing `build.py`, `arch-lint`, guardrail tests, and integration-test suite already cover the six-pack quality gates more cheaply.
- [Assess AIR-J and speclj-structure-check patterns for AI-assisted Rust development](./issues/07-assess-ai-patterns-rust-dev.md) — From the code-development perspective: AIR-J's canonical-output and structured-diagnostic patterns map well to existing guardrails, but the real gap is a unified pre-flight skill that runs cheap checks before the expensive test suite; no new prototype tool is warranted.
- [Determine applicability scoring rubric and final output format](./issues/08-determine-scoring-rubric-and-output-format.md) — Approved rubric: Directly usable, Practice/technique/tooling idea to adopt, Already covered, Not applicable, Future spike. Final artifact: a single markdown report under `docs/project/` covering all 34 pushed repos briefly, with depth on adoptable ideas and future spikes.
- [Quick-pass assess remaining 2026 repository groups](./issues/09-quick-pass-remaining-repos.md) — None of the remaining game/example or acceptance-testing repositories escape "Not applicable" / "Already covered"; no follow-up tickets needed.
- [Research cargo-mutants and CRAP tooling for Chronicler Engine](./issues/10-research-crap-mutation-testing.md) — cargo-mutants is too slow for a gate; adopt a lightweight CRAP script as an on-demand risk radar and mutation testing as an on-demand per-file pilot.
- [Research canonical output schemas and validation for Chronicler Engine LLM agents](./issues/11-research-agent-output-schemas.md) — Pilot typed-DTO validation with structured diagnostics on the quantifier; keep `LlmCallRecorder` transport-only and degrade invalid quantifier outputs to Low confidence.
- [Research pre-flight structural checker skill for Chronicler Engine](./issues/12-research-preflight-structural-checker.md) — No new pre-flight command is warranted; AI assistants already run the relevant targeted check for the files they change. Close with no implementation.
- [Research generated acceptance tests and QA telemetry for Chronicler Engine](./issues/13-research-generated-acceptance-qa-telemetry.md) — Uncle Bob uses plain `.feature` files and executable generated tests; for Chronicler, generated stubs are worse than the status quo. Either switch specs to `.feature` + Rust Cucumber, or keep Markdown specs and add step helpers. Do not do Markdown-to-stub generation.

## Not yet specified

_None._

## Open tickets

_None._

## Out of scope

- Repositories last updated before 2026 (unless surfaced by a 2026 repo).
- Prototyping or implementation of any pattern; research tickets here only scope the decision.

## Assets

- [Uncle Bob 2026 repository applicability report](./assets/uncle-bob-2026-applicability-report.md) — synthesized final report covering all 34 pushed repositories.
