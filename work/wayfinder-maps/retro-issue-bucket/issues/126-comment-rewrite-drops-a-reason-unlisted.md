# A comment rewrite drops a reason without listing it

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`chronicler-comment-fixer` lists each deleted comment that states a reason under DELETED REASONS (decision 52). A reason sentence removed inside a rewrite is not listed. Must it be?

## Context

Source: `/retro` 2026-11-22 of [Build the scene entries](../../long-term-memory-and-state/issues/17-build-scene-entries.md), retro session `e91f4262`.

- **known** Comment pass 2 (session `b4a64dd0`) dropped the `lead` sentence of `render_non_blank_layer` inside a rewrite, so it did not appear under DELETED REASONS (`.agents/skills/chronicler-comment-fixer/SKILL.md:144-146`). A second standards review found it, and it was restored.
- **known** The same pass gave a false reason to delete the doc of `Job::run_step` ("`Option<Kind>` carries it"). DELETED REASONS let the coordinator catch it.

## Options to decide

- **A.** One line in the skill: DELETED REASONS also lists each reason sentence that a rewrite removes.
