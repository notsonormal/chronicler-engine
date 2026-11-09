# Write a failure line when a Job panics

Type: task (AFK, decide and do)
Status: open
Blocked by: 03

## Question

Make a Job that ends with no records always write a failure line (gate-05), with the text that [ticket 03](03-grilling-small-product-rules.md) (rule 5) decides. Choose the mechanism under the enforcement order of the map Notes.

Today only the `Result` error paths of `Job::run` write the line, through `Job::end` (`src/application/pipeline/job.rs:142-175,385-412`). A panic in the blocking task releases the hold through `Drop for JobWriter`, but writes no line. The accept already cleared the old line, so the status says Ready. The same holds inside an Autoplay run (`Drop for AutoplayRun`).

The candidate mechanisms: a guard in `Job::run` whose `Drop` writes the failure line when the Job did not finish; or `catch_unwind` around the blocking task, which turns the panic into a normal Job failure.

## Context

- Finding in full: gate-05 in [the gate report](../assets/gate.md). The coordinator did not read it in the code.
- [Cut provider text by characters, not by bytes](06-task-cut-provider-text-by-characters.md) removes the one known panic. This task covers every other panic.
- `job.rs` and `driver.rs` are covered by `verify-gate`, so the task runs `/verify-gate` (map Notes).
- The test needs a seam that panics inside a Job, for example through `ScriptedModel`. If no seam exists, say so (map Notes).
- The six end conditions of the map Notes apply.

## Answer
