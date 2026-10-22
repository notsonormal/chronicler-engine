# List what reaches the model in each call

Type: research
Status: open
Blocked by: —

## Question

For each call in scope (Narration, Event Narration, the regenerate of a user Input, Impersonate, Quantifier, Options): which texts and parameters reach the model, and where does each one come from: a preset field, World or card data, a setting, or code? Which of them can an author change today through `data/` JSON or the existing UI?

## Context

- Start from the layer table in `docs/diataxis/reference/narrative/prompt_system.md` and the rules of each kind of generation in `narration_system.md`. Check them against the code.
- The Narrator role shares one assembly: `src/application/prompting/assembler.rs` (layers, tag names, the history labels `Narrator:` and the Persona's name), `builders/sections.rs` (the part after the history), `utils/context.rs` (what is dropped when the history is over budget), `sanitize.rs`, `prompt_merge.rs` (single-user-message mode).
- `src/application/narration/narrator.rs` passes the global `response_length` to every generation. `PromptPreset::render_field_parts` (`domain/model/prompt_preset.rs`) adds it to `output_format` under "Response Length:". Impersonate drops the `<PlayerCharacter>` layer and the guide; its direction goes into `<PlayerInput>`, which stays as an empty block when there is no direction.
- The Quantifier and Options have their own prompt code: `application/agents/quantifier/prompt.rs`, `application/agents/options/prompt.rs`.
- Parameters: `LlmProviderConfig` (`domain/model/settings.rs`) holds `max_tokens` and `max_context_tokens` for each connection. Roles are Narrator, Quantifier and Options (`domain/model/agent.rs`); Impersonate uses the Narrator connection. No temperature or other sampler is sent (inferred from code search; confirm).
- The `llm_messages` table stores `system_prompt`, `user_prompt` and `raw_request_json` for each call. A real Impersonate prompt is in [the stored Impersonate prompt](../assets/impersonate-prompt-157.txt).

## Output

An asset `assets/what-reaches-the-model.md`: for each call, one table with a row for each text or parameter (what it is, where it comes from, changeable by an author: yes or no), plus one real prompt of that call. Also list each row of `prompt_system.md` that the code contradicts (the build plan fixes the doc). Do not judge or propose here; the decisions belong to later tickets.
