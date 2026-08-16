# Narrator message type and history rendering

Type: task
Status: pending

## Question

Add the `MessageType::Narrator` variant and render narrator entries as bare scene truth in history.

Per the design synthesis (`../research/04-design-synthesis.md`, Q1 + Q5):

1. Add `MessageType::Narrator` to `src/domain/model/state/message_types.rs` (variants today: `Narration`/`Dialogue`/`System`/`Input`). Keep `System` meaning "engine notice."
2. Update `render_history_layer` in `src/application/prompting/assembler.rs` so a `Narrator` entry renders as bare text with **no `{sender}: ` prefix**. Today the renderer forces a sender on every line (`sender.unwrap_or("Narrator")`, `assembler.rs:320`); the `Narrator` branch must skip that format. Bare text is the chronicler-analog of Marinara's `role: system` + no-prefix — the absent prefix is the narrator signal in chronicler's flat `<ConversationHistory>` block. Other types keep the existing `{sender}: {text}` format.
3. Update the `MessageType` match arms downstream: `view_models.rs:55` (the `=> "system".to_string()` arm), storage mappers, and any exhaustive match. The exhaustive match is the safety — every consumer must acknowledge the new variant.
4. Add unit tests in `message_types_tests.rs` / `assembler_tests.rs` for the no-prefix rendering and the type round-trip.

Grounding: Marinara roleplay/game maps `narrator` → `role: "system"`, no prefix (`generate.routes.ts:1448`); ST suppresses the name prefix for `extra.type === 'narrator'` (`openai.js:580,586`). Chronicler chose the new-variant approach (not reuse-`System`-with-flag) so author narrator voice stays separate from engine diagnostics.
