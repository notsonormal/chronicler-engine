---
name: improve-ai-plan
description: Engineering plan review covering scope, architecture, code quality, tests, and performance. Use when asked to review a plan, review an implementation plan, or do a plan review before starting implementation. Best used after EnterPlanMode before ExitPlanMode.
---

# Plan Review

Engineering manager-mode review of an implementation plan. The goal is to catch scope issues, unnecessary complexity, and missing considerations **before code is written**, when changes are cheap.

## How to Use

When invoked during plan mode, read the current plan file and review it against the sections below. Surface issues one at a time with a concrete recommendation and an option to skip. Do not dump all issues at once.

---

## Step 0: Scope Challenge

This is the most important step. Do it before anything else.

1. **Search for existing code** — Before accepting any new function, class, or module in the plan, grep for near-identical implementations across `src/`. If something already exists, propose reusing it instead of writing new code. Cite the file.

2. **Minimum viable change** — For each file in the plan, ask: is this change strictly necessary to achieve the stated goal? Challenge any change that is not directly required.

3. **Complexity check** — If the plan touches 8 or more files, or adds 2 or more new classes, treat this as a smell. It does not mean the plan is wrong, but it means the complexity should be justified explicitly. Ask: can this be split into a smaller first change?

4. **Built-in solutions first** — Before accepting a custom implementation, check whether the framework, the existing test infrastructure, or an existing utility already solves the sub-problem. Check dependency manifests for libraries that might already be available.

5. **Distribution check** — If the plan produces new classes, services, or functions: where are they called from? A new public function with no production caller is dead code on arrival. The plan should show the full chain from entry point to the new code.

---

## Cognitive Patterns

Apply these lenses throughout the review. They are the difference between a plan that works and a plan that is maintainable.

1. **State diagnosis** — Most bugs live at state boundaries. Where does this change touch state transitions? Are all transitions handled?
2. **Blast radius** — Which existing callers are affected by each change? Have they been listed?
3. **Boring by default** — Is there a simpler, more boring solution that achieves the same goal? Novelty in production code is a liability.
4. **Incremental over revolutionary** — Can this be shipped in a smaller first step that delivers value independently?
5. **Systems over heroes** — Does this plan create a single complex method that requires a specialist to understand? Can it be decomposed?
6. **Reversibility preference** — How hard is this change to reverse if it turns out to be wrong? Flag irreversible decisions.
7. **Failure is information** — Does the plan specify what happens when each new codepath fails? Silent failures and swallowed exceptions are bugs.
8. **Conway's Law** — Does this plan create abstractions that mirror the team structure, or abstractions that mirror the problem domain? Favour the problem domain.
9. **DX is product quality** — Is the resulting code easy to understand, test, and change? Hard-to-test code is a design smell, not a test problem.
10. **Essential vs accidental complexity** — Which complexity in the plan is inherent to the problem, and which is introduced by the solution? Challenge accidental complexity.
11. **Two-week smell test** — Would a developer reading this code two weeks after it was written understand why it exists? If not, the design needs simplifying or the naming needs work.
12. **Glue work awareness** — Does the plan account for integration work (wiring beans, updating topic config, adding indexes, updating API spec) or only the "interesting" implementation parts?
13. **Make the change easy first** — Is there a preparatory refactoring that would make this change simpler? Sometimes the right first step is cleaning up before adding.
14. **Own your code in production** — Does the plan include logging, metrics, or observability sufficient to diagnose issues when this code runs in production?
15. **Error budgets** — What is the cost of this code being wrong? Does the risk justify the current level of test coverage in the plan?

---

## Review Sections

Review each section and surface findings one at a time. Ask for the user's decision before moving to the next issue.

### 1. Architecture

- Does the plan respect the DTO/domain boundary (DTOs at the API layer only, domain models inside)?
- Does it introduce new abstractions (interfaces, wrapper classes) without multiple implementations?
- Are there failure scenarios for each new codepath? What happens if the external service is down, the message queue is unavailable, or the database rejects the write?

### 2. Code Quality

- Is there any code cloning — patterns already present elsewhere that should be extracted?
- Is there over-defensive code (null checks on non-nullable types, broad catch-and-rethrow)?
- Are there phantom abstractions (interfaces with only one implementation, wrapper classes that only delegate)?
- Are there comments that restate the code rather than explaining why?

### 3. Tests

- Does the plan include unit tests for every new public function that has a production caller?
- Does it include integration or functional tests for new API endpoints or message consumer changes?
- Are edge cases covered — empty collections, null optional values, retry behaviour, write failures?

### 4. Performance

- Are there N+1 query patterns — loading a collection and then querying the database for each element?
- Are there queries without a supporting index?
- Are there consumer loops with blocking operations inside?
- Are there missing timeouts on external HTTP calls?

---

## Required Outputs

At the end of the review, produce:

### NOT in scope
List explicitly what this plan does NOT address. This prevents scope creep during implementation and sets expectations.

### What already exists
List existing code, patterns, or utilities that should be reused rather than reimplemented.

### Failure modes
For each new codepath in the plan: what happens when it fails? Does the plan handle it?

### Unresolved decisions
Any decisions the plan does not make that should be made before implementation starts.
