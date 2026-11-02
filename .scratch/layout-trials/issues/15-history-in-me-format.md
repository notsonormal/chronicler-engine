# Try the history in the ME format

Type: task (HITL)
Status: open
Blocked by:

## Question

Build the history of a Narrator-role call in the Marinara Engine (ME) format, then check with a trial that the story quality and the cached share hold. In the ME format, each entry is its own chat message with a role, XML tags wrap the history, and the entries have no `Narrator:` or Persona-name labels.

**Build first (owner, 2026-11-02).** This ticket is an exception to "Decide, then build" in the map Notes. The owner has already chosen to follow ME, so the trial checks what happens. It does not decide. The order is: build, then two trials, one on the commit before the build and one on the commit after it.

## Context

- **Why (owner, 2026-11-02).** The engine was modelled on ME, but its history is different. Nobody chose the difference on purpose, and the owner sees no reason to keep it. This ticket brings the history in line with ME. It does not need a measured gain: keep it unless it does harm.
- **Today (known).** The model port sends two messages. `LlmProvider::complete` (`src/application/ports/llm_provider.rs`) takes one system prompt and one user prompt. `build_request_payload` (`src/adapters/driven/llm/transport/utils/request.rs`) sends one `system` and one `user` message. The whole history is one text block in the user message, inside `<ConversationHistory>`. `render_history_entry` (`src/application/prompting/assembler.rs`) puts a label in front of each entry: `Narrator` for a Narration, the Persona name for an Input, `System` for a System entry. **Inferred:** the labels exist only because the history had to fit into one text.
- **ME at `origin/main` `e5ead06` and at the pinned `457e827` (known, same code; local clone `tmp/marinara`).**
  - One message for each entry. `mapChatHistoryMessageForPrompt` (`packages/server/src/routes/generate.routes.ts`, line 2341) keeps the `user` and `assistant` roles and sends a `narrator` row as `system`.
  - The `chat_history` marker returns these messages into the message list, not one text (`packages/server/src/services/prompt/assembler.ts`, line 924).
  - `expandChatHistory` (`packages/server/src/services/prompt/marker-expander.ts`, line 504) adds the wrap at build time. With `wrapFormat` = `xml`, it puts `<chat_history>` at the start of the first message and `</chat_history>` at the end of the second-to-last message. It wraps the last turn in `<last_message>…</last_message>`. With `markdown`, it uses `## Chat History` and `## Last Message`. With `none`, it adds nothing. The default preset uses `xml` (`packages/server/src/db/default-preset.json`). The stored rows never hold the tags.
  - No speaker names in a one-character chat (`conversationPromptHistoryContent`). Names go into the text only in a group chat with individual turns, when `groupSpeakerNamesInHistory` is on or the mode is `conversation`, and never in `game` mode (`prefixGroupIndividualHistorySpeakers`, `generate.routes.ts` line 3152).
  - `singleUserMessage` joins all messages into one user message. Each non-user message gets a `[SYSTEM]` or `[ASSISTANT]` header, not a speaker name (`assembler.ts`, Phase 8).
  - The Impersonate instruction is a trailing `user` message in ME and a trailing `system` message in SillyTavern ([Try the role of the Impersonate prompt](08-impersonate-prompt-role.md)).
- **The cache cost of the wrap (inferred).** The tags move on each call: the old second-to-last message loses `</chat_history>`, and the old last message loses `<last_message>`. So `xml` breaks the cache at about the last two old messages, `markdown` at about one, and `none` at none. The owner accepts this as the cost of the XML style (owner, 2026-11-02). ME lets the author pick the wrap. This ticket uses `xml` and does not add a setting.
- **Role breaches are not a fault (owner, 2026-11-02).** A Narration that speaks for Julian, or an Input that speaks for another Character, can be right for the story. The faults that [the inconsistencies of two trial stories](../assets/story-inconsistencies.md) calls role breaches came from the label format: Inputs that wrote whole `Narrator:` / `Julian:` transcripts (847 and 1344 words). So this ticket counts the label lines, not who speaks.
- **Settled design:**
  - The role of each entry follows ME: `assistant` for a Narration and an event Narration, `user` for an Input, `system` for a System entry. No code writes a `MessageType::System` row today (known: only mappings and one test fixture name it), so the last case is rare.
  - The Impersonate call keeps the roles. Its instruction is already in the Impersonate preset: the system prompt and the post-history splice. The splice goes in the final `user` message (D2), so it is a trailing `user` instruction, as in ME.
  - The blocks after the history keep their order: `<GameState>`, `<CharactersPresent>`, the splice, then `<PlayerInput>` and `<Guide>` when they render. The cache gain of [Move the changing blocks after the history](09-move-changing-blocks-after-history.md) stays.
  - `single_user_message` joins all messages into one `user` message, with a role header on each non-user part. `merge_single_user_message` (`src/application/prompting/prompt_merge.rs`) already writes `[SYSTEM]\n`, which is the ME form. The build extends it with `[ASSISTANT]`.
  - The Gemma 4 channel close (`close_gemma4_thinking_channel`, `chat_completions.rs`) goes on the last message.
  - The Quantifier and the Options agent keep their own labelled history (`<RecentHistory>`, `quantifier/prompt.rs`, `options/prompt.rs`). They do not write story text.
- **The build.** On `layout-trials`, test first. Change the port, `build_request_payload` and `LlmCallRecorder::complete` (`src/application/llm_recorder.rs`) to carry a list of messages, and change the assembler to give the history as messages. `llm_messages` must still let the LLM record dialog and `scripts/trial.py` show the whole request. Change the docs that the change makes wrong, at least `docs/diataxis/reference/narrative/prompt_system.md`. End with `/simple-after-plan`, as [Move the changing blocks after the history](09-move-changing-blocks-after-history.md) and [Send an OpenCode session for each Game and agent](11-session-per-agent.md) did. Record the commit before the build: the reference trial runs on it.
- **The pack tool (known, from [Try the history as chat messages](07-history-as-chat-messages.md)).** `scripts/trial.py` finds the history through `<ConversationHistory>` in the stored user prompt (`history_block`), and writes each call file from the two stored texts. With the history as messages, the pack would show no history and wrong "holds / drops" numbers, with no error. So the build changes `scripts/trial.py` too, with its tests in `scripts/tests/test_trial.py`. With D5 (a) the tag name stays. With D7 (a), `trial.py` reads the messages from `raw_request_json` for a new row and from the two columns for an old row. It must still read a pack of the reference commit.
- **Measure.** Two 100-Cycle trials (about 3 M tokens each) on the fixed data of [Remove the data conflicts that distort a trial](12-remove-data-conflicts.md), after the build. The reference trial runs on the commit before the build, in a linked worktree with a detached HEAD and its own probe server. The new trial runs on the built `layout-trials`. `tmp/lt01` cannot be the reference: it ran before the data fixes and before the builds of tickets 09 and 11. The new trial is also the reference for later checks.
  - The two trials can run in parallel. If calls fail with a rate error, run them one after the other (map Notes, "Failure modes").
  - The cached share of each call kind, from the packs.
  - The outputs that hold a label line (`Narrator:`, `Julian:`, `System:` at a line start), and the length of the Inputs.
  - An ad hoc judge reads each whole story with the same prompt and neutral names ([Decide how to read an output trial](05-read-an-output-trial.md)). Its question: does the story get worse, by the fault types of [the inconsistencies of two trial stories](../assets/story-inconsistencies.md)? The coordinator checks a sample of the findings before the owner reads them.
- **Expected:** the label lines go to about zero, the judge finds no new kind of fault, and the cached share does not drop by more than a few points. If the trials show harm, the owner decides to fix the build or to revert it. `layout-trials` is not merged to `main`, so a revert is cheap.
- **How:** the worktree steps of [Try the changing blocks after the history](03-changing-blocks-after-history.md): seed the target dir, copy the main probe DB, and start the server from a scratch folder under `tmp/` of the main checkout. Use copies of `scripts/trial_setups/redmist_julian.json` under `tmp/lt15/`. Restart the main probe server first, so it loads the fixed World. After each trial, check that the pack shows the history and the call kinds (map Notes, "Failure modes").

## Plan review (2026-11-02)

The claims above were checked against the code on `layout-trials`.

### Decisions (owner, 2026-11-02)

- D1 (b), D2 (b), D3 (a), D5 (a), D6 (a): as recommended below.
- **D4, the rule (owner):** an LLM message records what the engine sent to the model and what came back, whatever that is. Today it does not in three cases: with `single_user_message` the record shows two texts, but one merged `user` message was sent. On Ollama with Gemma 4 the record lacks the channel close. A failed call stores an empty `raw_request_json`.
- **D7 (a) (owner):** one source, the payload. The transport returns the payload it sent, on success and on failure. The recorder stores it in `raw_request_json`. The LLM record dialog and `scripts/trial.py` show the messages from it. New rows stop writing `system_prompt` and `user_prompt`, and old rows keep them. `parsed_response` stays the sanitized text that the engine used, and `raw_response_json` holds the exact answer.
- **D8 (owner):** the record change is part of this ticket, not a ticket of its own. It repairs the three cases above.

### The options of round 1

- **D1. The last Input appears twice.** Known: in a Narration call, the Input that the Narration answers is the last history entry and also `<PlayerInput>` (`Narrator::user_message`; seen in `tmp/lt03`, the calls `*-narration.md`). In the ME format, `<last_message>` already carries it. Options: (a) keep `<PlayerInput>`, with the repeat; (b) drop `<PlayerInput>` when its text is the last history entry. An Impersonate direction and an event continuation keep their block. Recommended: (b).
- **D2. Where the blocks after the history go.** Options: (a) one new `user` message after the history; (b) the same blocks, but joined into the last history message when that message is a `user` message (a Narration call), and a new `user` message otherwise (an Impersonate or event call). With (a), a Narration call sends two `user` messages in a row. Some chat templates reject that (guessed for `deepseek-v4.1-flash`). Recommended: (b).
- **D3. Where the blocks before the history go** (`<KnownCharacters>`, `<Persona>`, `<WorldLore>`). Options: (a) a first `user` message, so the system message stays the preset text, as the LLM record dialog and `trial.py` show it today; the first history entry is the opening Narration, so the roles alternate; (b) the end of the system message. Recommended: (a).
- **D4. The stored form of an LLM message.** Replaced by the owner's rule and D7 above.
- **D5. The tag names.** Options: (a) keep `<ConversationHistory>` and add `<LastMessage>`, in the CamelCase of the other layers; `trial.py` and the docs keep the name, and `trial.py` reads both packs; (b) the ME names `<chat_history>` and `<last_message>`. Recommended: (a).
- **D6. The port shape, and the name of one item of the list.** Options: (a) `LlmProvider::complete` and `LlmCallRecorder::complete` take a list in place of the two texts; the Quantifier, the Options agent and the connection test send two items; (b) a second method for a list, next to the old one. Recommended: (a), one way to call. The name: CONTEXT.md gives "Message" to the story and "LLM message" to the stored record, and "Turn" is deprecated. A proposal: `PromptPart { role: PromptRole, text }`.

### What already exists

- `HistoryCut` (`src/domain/model/history_cut.rs`) already cuts the history by entry. Each entry becomes one message, so the cut needs no change.
- `merge_single_user_message` writes the `[SYSTEM]` header form.
- `build_request_payload` already builds a `messages` array. Only its input changes.
- `ScriptedModel` records each call as `ModelCall { system_prompt, user_prompt }` (`src/test_support/scripted_model.rs`). It is a test fake, not the LLM message record. It can keep two texts made from the list, so the tier-1 tests that read `calls[i].user_prompt` need few changes.

### Blast radius (known)

- The port callers: `LlmCallRecorder::complete` (`llm_recorder.rs`), `ConnectionTestService` (`connection_test_service.rs`, line 68), the Options agent (`options/utils/orchestration.rs`, line 78), the Quantifier (`quantifier/utils/orchestration.rs`, line 96), and the narrator (`narrator.rs`, line 143).
- With D7 (a): the transport returns the payload on failure too (`transport/utils/client.rs`, `response.rs`), `LlmCallResult::to_message` and `LlmCallRecorder::failed_message` stop taking the two texts, the LLM record dialog (`templates.rs` around line 172, `view_models.rs` around line 231) shows the messages of `raw_request_json` and falls back to the two columns for an old row, and `scripts/trial.py` reads the system message (`call_kind`, the "player character" check), the Quantifier `<LatestMessage>` (`owner_entry`) and the history (`history_block`) from the payload. `tests/storage/llm_message_storage.rs` and `tests/http/llm_messages.rs` check the record.
- The port implementations: `ChatCompletionsBackend`, the mock provider (`providers/mock.rs`), `ScriptedModel`, and the fakes in `connection_test_service_tests.rs`, `llm_recorder_tests.rs` and `quantifier/utils/orchestration_tests.rs`.
- `AssembledPrompt` (`assembler.rs`) holds two texts today. It becomes the system text and the list.
- With D1 (b): the dev mock provider finds its reply through `<PlayerInput>` (`extract_player_input`, `src/adapters/driven/utils/mock.rs`), and `tests/http/autoplay/run.rs` (line 70) checks `<PlayerInput>` in the narrator prompt. Both change.
- The docs: `docs/diataxis/reference/narrative/prompt_system.md` and `docs/diataxis/explanation/prompt_system_design.md` name `<ConversationHistory>` and `<PlayerInput>`. `narration_system.md` describes the port.
- The token budget counts each entry with `estimate_tokens`. Each message also costs a few tokens of chat template. Count them in the fixed part, or accept the small error and say so.

### Not in scope

- A setting for the wrap format (`xml`, `markdown`, `none`).
- The history forms of the Quantifier and the Options agent.
- The event continuation text (`Narrator::continuation_user_message`), which repeats the last Narration. It is a separate fault, if it is one.
- Any change to the presets.

### Failure modes

- **The provider rejects the list**, for example a `system` message in the middle, or two messages of one role in a row. The call fails, the LLM message records the error, and the Job fails as any failed call does. The plan handles it with one probe call of each kind (Narration, Impersonate, event) on the probe server before the trials.
- **A cut leaves no history, or the history is empty.** Today the slot shows a placeholder text. With messages, the list has no history items. The build keeps the placeholder in the first `user` message and tests both cases in `assembler_tests.rs`.
- **A call fails before a payload exists**, for example when the provider does not resolve (`LlmCallRecorder::provider`). Then nothing was sent, so an empty `raw_request_json` is correct under the rule. The dialog shows the error alone. The build tests a failure after the send (the payload is stored) and a failure before it (no payload).
- **`trial.py` reads the new pack wrong** with no error: the history columns or the call kinds go blank. The plan checks each pack after its trial (map Notes, "Failure modes").
- **The budget sends too much.** If the chat-template cost of many messages passes the context window, the provider rejects the call. The overflow guard counts characters, not messages. The build tests a long history near the limit.

Done when the build passes the full `python build.py`, both trials have run, the owner has read the findings and kept or reverted the build, the answer states each design choice, the counts and the cached shares, and the worktree is removed.
