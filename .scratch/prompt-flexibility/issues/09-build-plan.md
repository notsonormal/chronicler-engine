# Write the build plan for the engine changes

Type: task
Status: open
Blocked by: 06, 07, 08, 10

## Question

Write the build plan that carries out the decisions of the layout model, the call parameters and the test setup, as one plan in `docs/plans/`, ready for an implementer.

## Context

- Follow the decisions of [Decide which parts of a call an author can change](06-the-layout-model.md), [Decide the parameters of each call](07-call-parameters.md) and [Shape the test setup and the trial](08-test-setup-and-trial.md).
- Also follow [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md).
- Blast radius to cover: the tests that assert prompt text (`application/narration/narrator_tests.rs`, the Quantifier and Options prompt tests, the spec scenarios on prompt content), the seed presets in `data/prompt_presets/`, and the preset rows already stored in existing databases if the preset shape changes.
- The owner wants the plan written directly in the repo.
- The plan includes the spec changes, the tests, the doc changes (`docs/diataxis/reference/narrative/prompt_system.md` and others), and one trial on the current data as the check that the changes work.
