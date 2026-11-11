# A prompt that still overflows asks for a 1-token answer

Source: the sweep of implicit state machines, 2026-11-08 ([catalogue](../../wayfinder-maps/state-machines-next/assets/catalogue.md), row C4)

## The defect

`LayerRenderer::render_cut_prompt` (`src/application/prompting/assembler.rs:188`) cuts the history to the token space. Then its overflow guard checks the characters against the input limit. On an overflow, it calls `HistoryCut::drop_to_free` one time, which can drop every Message of the history.

If the parts outside the history still overflow, the prompt stays over the limit. An example is a very long Input. Then `TokenBudget::response_tokens` (`src/application/prompting/token_budget.rs:67`) computes `max_context - (system + tokens_after_system + margin)` with `saturating_sub`. That gives 0, and `.max(1)` makes it 1. The call goes out with an overflowing prompt and `max_tokens = 1`.

`response_tokens` returns `EngineError::ContextOverflow` only when the system prompt alone does not fit (`token_budget.rs:76`).

## Impact (inferred)

The provider most likely refuses the call with its own context-length error, or it answers with one token. In both cases the player gets a narrator failure that does not name the cause, not the `ContextOverflow` line.

## Done when

- A prompt that does not fit after the guard fails with `ContextOverflow` (or a new failure kind that names the cause) before the model call.
- A test with a fixed part larger than the input limit gets that failure, and the LLM fake gets no call.
