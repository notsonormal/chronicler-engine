# Shape the test setup and the trial

Type: prototype
Status: open
Blocked by: 01

## Question

What does a test setup fix, how does a trial start, how many times does it repeat, what does it save, and what does the reading pack look like?

## Context

- Autoplay plays N cycles on one Game (`POST /autoplay` with a budget). The `llm_messages` table already stores every prompt, request and answer.
- A first rough reading pack exists: [story transcript](../assets/autoplay-story-transcript.md) and [the stored Impersonate prompt](../assets/impersonate-prompt-157.txt), made by hand from the probe DB.
- Points to settle: what is fixed (World, Persona, opening, presets, connections, cycles); how a trial starts (a script against a server, a CLI subcommand, or a test harness); repeats, because the same setup gives different output each time; how failed calls show (the Quantifier failed every cycle with `content_filter`, which left the game state stale with no visible sign); how two trials sit side by side. Do not define criteria for good output: that is not yet specified (see the map).
- A trial calls a paid model many times. Settle a cost limit for one trial.
- Reuse what exists: the LLM messages panel (`/fragment/llm-messages`) and the probe server of `/chronicler-ui-investigator` (`target/ui-investigator/`, port 3001), which the first Autoplay run used.
- Long play is where memory and state fail. The reading pack may need to show what fell out of the history window and which state changed. See [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md) and [Find how ST, ME and other tools keep long-term state](14-long-term-state.md). They do not block this ticket: the first prototype runs on the current engine.
- From [Find how prompt caching works for our providers and our layout](12-prompt-caching.md): record per call the cached tokens (`usage.prompt_tokens_details.cached_tokens`, already in `raw_response_json`), the upstream host and the cost (for OpenCode Go, `usage.opencode.allowance`, because its `usage.cost` is 0). The first trial gives the first pair of calls of one kind on OpenCode Go, so it answers whether that connection caches. The random card order (a `HashMap`) means two trials of one setup do not send the same prompt.
- From [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md): a trial needs about 39 cycles of this World's prose to show the first history cut, or a connection with an 8192-token context to show it at about cycle 10. The cycle depends on how long the model writes, so a test setup must fix `response_length` and the connection context, or two setups differ in a second variable. The reading pack can report the loss from the stored request alone: the Messages that `<ConversationHistory>` does not hold, and the offset of the front cut.
- Use `/prototype`: a script that exports a trial from the database into a reading pack is a cheap first artifact.
