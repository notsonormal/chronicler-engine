# 05: Small shrinks batch

Type: task
Status: open

## Question

Apply the low-impact mechanical shrinks from the ponytail-audit in one batch.

## Work

- Inline `prompt_merge.rs` as `format!("[SYSTEM]\n{system_prompt}\n\n{user_text}")` at the two provider call sites and delete the file
- Replace `truncate_to_budget` char-Vec scan with `text.chars().skip(...).collect()`
- Inline `render_template` as `text.replace("{{user}}", &vars.user)` at call sites and delete `src/domain/model/utils/template.rs`
- Replace `GameViewQuery::get_npc_headshots` HashMap build with a direct linear scan of `npcs_list`
- Simplify `char_span_to_byte_span` using `text.char_indices().nth(...)`
- Delete `LlmCallResult::to_message` and construct `LlmMessage` directly in `LlmCallRecorder::complete`
- Delete `ArrivalTaskContext::new_for_test`
- Inline `spawn_pipeline_task` as `tokio::task::spawn_blocking` at the call site and delete the file
- Delete `ActionPipeline::with_storage` (callers use `with_backends`)
- Gate `ActionPipeline::rebind_for_test` under `#[cfg(test)]`

## Acceptance

- Each item above is removed, inlined, or simplified
- Behavior unchanged
- `python build.py` passes

## Notes

If any item turns out to be larger than expected, split it into a new ticket and leave it out of this batch.
