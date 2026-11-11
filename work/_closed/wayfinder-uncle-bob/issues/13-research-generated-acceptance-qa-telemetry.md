# Research generated acceptance tests and QA telemetry for Chronicler Engine

Type: research
Status: closed
Assignee: assistant
Blocked by: 09

## Question

Could Chronicler Engine reduce integration-test maintenance or improve coverage by borrowing generated-acceptance-test, Cucumber step-definition, or headless-QA-telemetry patterns from the Uncle Bob game repositories?

## Context

The Uncle Bob 2026 applicability report found richer acceptance-testing workflows in `missile-command`, `empire-2025`, `Acceptance-Pipeline-Specification`, and `Pharaoh-js` than Chronicler currently uses:

- Gherkin feature files → generated acceptance entrypoints.
- Cucumber step definitions mapping to test helpers.
- Headless runs with scripted scenarios and telemetry.
- Acceptance mutation (mutate scenario example values and re-run).

Chronicler today has Markdown feature specs in `docs/specs/*.md` and hand-written Rust integration tests checked by `scripts/validate_feature_spec.py`.

## Expected output

1. A survey of which patterns from the Uncle Bob repos are portable to a Rust project with hand-written integration tests.
2. A concrete proposal for the lowest-friction adoption (e.g., generated test scaffolding from feature specs, Cucumber-style step helpers, or a headless scenario runner).
3. An estimate of effort and maintenance cost for the top one or two options.
4. A recommendation on whether to spike a prototype, defer, or abandon the idea.

## Resolution

### 1. What Uncle Bob actually does

I read the four source repos. The common pattern is **Gherkin-first, generated acceptance tests**, not step-helper refactoring.

| Repo | Acceptance approach |
|---|---|
| `missile-command` | Plain `features/*.feature` → `gherkin-parser` → JSON IR → project-specific generator → generated acceptance entrypoints → executable tests. Also includes a QA mode (`--qa`) with scenario files, event scripts, and telemetry output. |
| `empire-2025` | `scripts/run-acceptance-tests.sh` / `clj -M:acceptance-tests` parse scenarios, generate specs, enforce acceptance boundaries, run generated acceptance specs. Headless runs with `--headless=N` produce logs and debug dumps. |
| `Acceptance-Pipeline-Specification` | Defines a language-neutral pipeline: Gherkin → JSON IR → optional IR-DRY checker → entrypoint generator → generated tests → project runner. Acceptance mutation mutates example values in the IR and verifies tests fail. |
| `Pharaoh-js` | 367 Cucumber scenarios in `features/`, step definitions in `features/step_definitions/`, run via `npm run features`. |

Key takeaways:
- Specs are plain `.feature` files, not Markdown-with-Gherkin.
- The pipeline is **generate-then-run**, not hand-write-then-tag.
- Acceptance mutation is a separate quality workflow that checks whether example values are actually wired into the tests.
- Headless QA telemetry is project-specific CLI affordance (QA mode), not a generic test tool.

### 2. The Markdown wrapper is not adding value

Chronicler's specs live in `docs/specs/*.md` as Markdown files containing Gherkin blocks. `scripts/validate_feature_spec.py` checks that every `Scenario N.N` has a `// SCENARIO: N.N` tag in a hand-written Rust test. The Markdown wrapper provides an H1 title and optional prose, but the behavioral authority is the Gherkin itself. The Markdown format appears to be an implementation artifact rather than a load-bearing design choice.

### 3. Options for Chronicler

| Option | Spec format | Tests | Effort | Maintenance | Verdict |
|---|---|---|---|---|---|
| **A. Switch to `.feature` + Rust Cucumber** | Plain `features/*.feature` | Step definitions calling existing HTTP/browser helpers | 3-5 days | High: step defs drift as UI/HTTP surface changes; new dependency | Honest executable-spec path; aligns with Uncle Bob. |
| **B. Markdown + generated Rust stubs** | Keep `docs/specs/*.md` | Parse Gherkin, emit empty Rust test stubs to fill in | 2-3 days | Medium: generator + checked-in boilerplate | **Worse than status quo**: stubs are dead code that still need hand-written assertions. |
| **C. Hand-written step helpers only** | Keep `docs/specs/*.md` | Add `given/when/then` helper functions in `tests/http/` | 1 day | Low | Does not follow Uncle Bob; only reduces duplication. |
| **D. Keep current approach** | `docs/specs/*.md` | Hand-written Rust tests + SCENARIO tags | — | Low | Works; no new value from Uncle Bob repos. |

**Do not choose Option B.** Generated stubs create the illusion of traceability while preserving all the hand-work of the current approach, plus they add a parser and checked-in boilerplate to maintain. Uncle Bob's pipeline works because the generated entrypoints call real step handlers and actually run.

### 4. Recommendation

**Either switch to `.feature` + Cucumber, or keep the current Markdown + hand-written tests and add step helpers for boilerplate.**

Given that the Markdown wrapper is not carrying load-bearing content, the **switch to `.feature` + Cucumber is the better long-term path** if the team wants executable specs and is willing to pay the conversion and step-definition maintenance cost. It removes the awkward Markdown middleman, makes specs directly executable, and replaces `validate_feature_spec.py` with the Cucumber runner's own discovery.

**Do not adopt headless QA telemetry or acceptance mutation now.** Those require a generated-test infrastructure that does not exist, and Chronicler's existing HTTP E2E and browser tests already cover the same surface.

Next step: create an implementation spike ticket to convert one spec file (e.g., `docs/specs/actions.md`) to `features/actions.feature`, write step definitions reusing `tests/http/test_helpers.rs`, and evaluate whether the Rust `cucumber` crate integrates cleanly with the existing test fixtures.
