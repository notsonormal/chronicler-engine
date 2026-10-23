---
name: improve-ai-plan
description: Review an implementation plan before code is written.
disable-model-invocation: true
---

# Plan review

Review the current plan before any code is written, while a change is cheap.

## 1. Check the plan against the code

Read the code that each step touches. Check each claim the plan makes about a type, a function, a file or a behaviour. A plan built on a wrong fact costs the most later.

## 2. Challenge the scope

- **Reuse:** for each new function, type or module, search the repo for something that already does the job. Cite it.
- **Minimum change:** challenge each change that the goal does not need.
- **Blast radius:** list the callers that each change affects.
- **Wiring:** each new public item needs a production caller. Show the chain from the entry point.
- **Rules:** check the plan against `CODING_STANDARDS.md` and the tiers in `tests/STRATEGY.md`.

## 3. Fix, then ask

Fix a clear gap in the plan yourself: a missing step, a wrong path, a missing test.

Ask the user only about the decisions that this review found. The decisions that the plan already settles stay settled.

Ask in rounds. One round holds every decision that does not depend on another open decision. Number the decisions. Give each decision its options and your recommendation. Ask a dependent decision in the next round.

## 4. Add to the plan

- **Not in scope:** what the plan does not do.
- **What already exists:** the code to reuse.
- **Failure modes:** for each new code path, what happens when it fails, and whether the plan handles it.
