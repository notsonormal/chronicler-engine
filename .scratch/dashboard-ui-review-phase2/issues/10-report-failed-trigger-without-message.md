# Report a failed trigger continuation without a story-log Message

Type: task (AFK)
Status: open
Blocked by: —

## Question

A failed trigger continuation adds the raw engine error to the story as a `System` Message. How should it report the failure instead, so the story, the prompt history and the Retrigger control stay clean?

## Context

- Finding **S3 (P2)** of the [2026-10-11 re-review](../re-review-2026-10-11.md), shot `tmp/p2-07/19-retrigger-failed.png`.
- `phase_trigger_continuation_llm_call` (`src/application/pipeline/pipeline_run.rs:220`) adds "[Trigger narration failed: {e}]" as `MessageType::System`, and also sets the generation error. The status line and the banner already report the failure.
- The prompt assembler sends every `System` Message to the Narrator as `System: …` (`src/application/prompting/assembler.rs:370`).
- The Message becomes the last entry, so `show_retrigger` (`NarrativeLogTemplate::new`, `src/adapters/driving/http/templates.rs`) is false and the swipe controls are absent.
- Check the other `System` Messages the pipeline adds ("[System] NPC detection uncertain — using room defaults" in the same file, and `action_pipeline/options.rs`) against the same rule, and record what you decide for them.

## Done when

- A failed trigger continuation adds no story-log Message; the status line and the banner report it; Retrigger stays available on the main narration.
- A test pins it. Name the tier.
- `python build.py` is green. Commit after user approval.
