---
trigger: when_working_in_chronicler_engine
---

# Chronicler Engine Developer Rules

When you (the AI) are tasked with building, debugging, or extending the `chronicler_engine`, you **MUST** adhere strictly to the following workflow.

## Development Workflow

All non-trivial work follows a 5-stage pipeline. Do not skip stages or blend their outputs.

### Stage 1: Specification
- **Specs are requirements, not plans.** A spec defines *what* the system should do, *why*, and the design constraints. It does NOT list file changes, code diffs, or implementation steps.
- **Location:** `chronicler_engine/docs/specs/`
- **No Rogue Coding:** All features must originate from a spec. If the user asks for a feature without one, create the spec first and get approval.
- **Atomic & Domain-Specific:** Specs must be logically decoupled. Do not combine independent domains into a single spec.
- **Living Documents:** When a feature changes the contract of an existing spec, update that spec too. Specs must always reflect the current intended behavior.
- **Status Field:** Every spec has a `Status` field: `Proposed`, `Approved`, `Implementation Starting`, `Completed`.

### Stage 2: Planning
- After specs are approved, produce an **implementation plan** as a conversation artifact.
- The plan maps specs to concrete file changes, new modules, and integration points.
- The plan references which specs it implements.
- Get user approval on the plan before proceeding.

### Stage 3: Tasking
- Decompose the plan into small, atomic, testable tasks in `chronicler_engine/docs/specs/<feature>/task.md`.
- Each task should touch as few files as possible to reduce drift.

### Stage 4: Implementation (TDD)
- For each task, write a **failing test** first using `#[test]`.
- Run `cargo test` and verify the test fails.
- Implement until `cargo test` passes.
- Mark the task complete and move to the next.

### Stage 5: Validation
- Run `cargo fmt`, `cargo clippy`, and `cargo test` on the full project.
- Update spec statuses to `Completed`.
- Produce a walkthrough artifact summarizing what changed.

## Rust Idioms and Best Practices
- Ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass successfully.
- Prefer explicit error handling logic. Use `Result` heavily for parsing strings/data.
