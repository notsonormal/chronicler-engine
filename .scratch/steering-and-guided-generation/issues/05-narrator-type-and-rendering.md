# Narrator message type and history rendering

Type: task
Status: closed

## Question

Add the `MessageType::Narrator` variant and render narrator entries as bare scene truth in history.

Per the design synthesis (`../research/04-design-synthesis.md`, Q1 + Q5):

1. Add `MessageType::Narrator` to `src/domain/model/state/message_types.rs` (variants today: `Narration`/`Dialogue`/`System`/`Input`). Keep `System` meaning "engine notice."
2. Update `render_history_layer` in `src/application/prompting/assembler.rs` so a `Narrator` entry renders as bare text with **no `{sender}: ` prefix**. Today the renderer forces a sender on every line (`sender.unwrap_or("Narrator")`, `assembler.rs:320`); the `Narrator` branch must skip that format. Bare text is the chronicler-analog of Marinara's `role: system` + no-prefix — the absent prefix is the narrator signal in chronicler's flat `<ConversationHistory>` block. Other types keep the existing `{sender}: {text}` format.
3. Update the `MessageType` match arms downstream: `view_models.rs:55` (the `=> "system".to_string()` arm), storage mappers, and any exhaustive match. The exhaustive match is the safety — every consumer must acknowledge the new variant.
4. Add unit tests in `message_types_tests.rs` / `assembler_tests.rs` for the no-prefix rendering and the type round-trip.

Grounding: Marinara roleplay/game maps `narrator` → `role: "system"`, no prefix (`generate.routes.ts:1448`); ST suppresses the name prefix for `extra.type === 'narrator'` (`openai.js:580,586`). Chronicler chose the new-variant approach (not reuse-`System`-with-flag) so author narrator voice stays separate from engine diagnostics.

## Answer

Implemented on branch `guided-generations`.

1. Added `MessageType::Narrator` variant to `src/domain/model/state/message_types.rs` (last position, after `Input`).
2. `render_history_layer` in `src/application/prompting/assembler.rs` now matches on `message_type`: the `Narrator` branch renders bare text (`{text}\n`) with no `{sender}: ` prefix; all other types keep the existing `{sender}: {text}` format. Added `MessageType` to the module imports.
3. Updated the one exhaustive `match entry.message_type` in non-test source — `src/adapters/driving/http/view_models.rs:52` — with a `MessageType::Narrator => "narrator"` arm. Storage needs no change: `messages.rs` serde-round-trips `message_type` as JSON, so `Narrator` serializes as `"Narrator"` automatically. `game_state.rs:121` (`push_message`) only treats `Narration`/`Dialogue` as swipe-appendable; `Narrator` correctly falls through to the plain `Message::new` append path — a narrator directive is a permanent history row, not a retry swipe.
4. Added two unit tests in `src/application/prompting/assembler_tests.rs`: `test_narrator_message_renders_without_sender_prefix` (asserts the narrator line appears bare and without `Narrator:`/`None:` prefixes while other types keep theirs) and `test_message_type_narrator_serde_round_trip` (asserts `"Narrator"` serialization).

Validation: `cargo clippy --all-targets -- -D warnings` clean; `cargo test --lib` 977 passed; architecture + guardrails tests pass; `python build.py` green (full integration suite, ~3min).

Note for downstream tickets: `Narrator` deliberately does not match the `Narration || Dialogue` predicates in `message_history.rs` / `retrigger.rs` / `retry.rs` — a narrator row is not AI-generated narration, so it is neither a retry target nor a retrigger source. Tickets 06/08/09/10 can rely on this.
