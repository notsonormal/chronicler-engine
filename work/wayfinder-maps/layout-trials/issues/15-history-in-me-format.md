# Try the history in the ME format

Type: task (HITL)
Status: resolved
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

## Answer

Resolved on 2026-11-02. **Owner verdict: keep.** The build is committed on `layout-trials`. The reference commit is `8a015b71`. The owner gave no answer on the filter question, so option (a) stands: the sanitize filter covers only the `<PlayerInput>` layer. The reference worktree `/workspace/chronicler-engine-lt15` is removed.

### The build

- **known** Each history entry of a Narrator-role call is one chat message with no label. A Narration or event Narration is `assistant`, an Input is `user`, and a System entry is `system`. `<ConversationHistory>` opens on the first entry and closes on the second-to-last. `<LastMessage>` wraps the last entry. A one-entry history has `<LastMessage>` only (ME rule).
- **known** D3 (a): the first `user` message holds KnownCharacters, Persona and WorldLore. D2 (b): GameState, CharactersPresent, the splice, PlayerInput and Guide join the last message when it is `user` (the last entry, or the first `user` message when no entry is kept), or else make a new final `user` message. D1 (b): `<PlayerInput>` is left out when its text is the last kept entry.
- **known** D6 (a): `LlmProvider::complete` and `LlmCallRecorder::complete` take `&[PromptPart]` (`PromptPart { role: PromptRole, text }`, `src/domain/model/prompt_part.rs`). The docs call it a "prompt part". `single_user_message` writes `[SYSTEM]` and `[ASSISTANT]` headers.
- **known** D4/D7/D8: the transport returns the payload it sent, on success and on failure, and `raw_request_json` holds it. New rows leave `system_prompt` and `user_prompt` empty. The LLM record dialog shows the parts of the payload, and falls back to the two columns for an old row. Spec 33.1 now shows the Raw Request JSON of a failed call. Spec 33.6 is new.
- **known** The budget counts `MESSAGE_OVERHEAD_TOKENS = 4` for each part (an estimate).
- **known** `scripts/trial.py` reads a new row from `raw_request_json` and finds the history by its tags. It stops with an error on a merged `single_user_message` pack. It reads `tmp/lt03`, `tmp/lt15/reference` and `tmp/lt15/probe` with the same results as before.
- **known** The sanitize filter covers only the `<PlayerInput>` layer. In a Narration call the current Input is a history entry, which the filter never covered (option (a), no code change).

### `/simple-after-plan`

- Comment pass (session `43fd9d06`): 22 comments deleted, 15 rewritten, all accepted. Doc review (`daa16705`): 4 must-fix and 8 nice-to-have, all fixed.
- `/code-review`, Standards (`2eafa64c`) and Spec (`1f17a179`). Accepted: the "part" names, the legacy tests moved to seam cases, one clause per case, the shared helpers, two names, and `trial.py` read by tags (both axes). Rejected: merge of `ChatCompletionFailure` and `LlmCallFailure` (layer rule), the read order of `trial.py` (it needs the old user text), a test for the dev mock fallback (legacy file), and two `user` messages after a cut (the trial tested it). Found by the coordinator: spec 33.1 went against D4. Fixes in session `4d83cb10`, and one missing case added (`narrator-after-blocks-join-the-first-user-message`).
- Full gate `python build.py`: guardrails 309, integration 1648, browser 44, all passed.
- Retro: new [102 — A spec scenario that the ticket's rule changes stays as it was](../../retro-issue-bucket/issues/102-spec-scenario-against-the-ticket-rule.md). Recurrences added to 98 (tests in legacy files) and 100 (the review bundle takes the whole tree).

### The trials

Reference: `8a015b71` in the worktree `/workspace/chronicler-engine-lt15`, port 3015, pack `tmp/lt15/reference/trial-1`. New: the build on port 3001, pack `tmp/lt15/new/trial-1`. Same setup copy, same Connection, both after the data fixes of ticket 12.

| Item | Reference | New |
|---|---|---|
| Cycles, failed calls | 100, 0 | 100, 0 |
| Wall time | 1209 s | 1054 s |
| Tokens in / out | 3,075,602 / 46,138 | 2,917,705 / 38,527 |
| Narration cached | 70.4 % | 85.8 % |
| Impersonate cached | 56.7 % | 86.3 % |
| Event narration cached (1 call) | 43.2 % | 40.8 % |
| Quantifier cached | 35.4 % | 43.6 % |
| Narrator-role calls cached | 63.9 % | **86.0 %** |
| Total cached | 60.0 % | **80.4 %** |
| Narrator-role calls with 0 cached | 39, spread over the run | 3, one at each history cut |
| Entries with a label line | 0 | 0 |
| Input words, mean (max) | 144 (284) | 126 (230) |
| Narration words, mean (max) | 183 (335) | 144 (259) |
| First history cut | call 1184, 24 entries | call 1206, 25 entries |

- **known** After each history cut, the payload starts with two `user` messages. The provider accepted all 94 such calls.
- **known** The label lines were already 0 in the reference (6 entries in `tmp/lt01`, 2 in `tmp/lt03`). So this pair cannot show that the build removes them. **inferred** The data fixes of ticket 12, or chance, removed them before the build.
- **I don't know** why the reference missed the cache on 39 Narrator-role calls spread over the run, while `tmp/lt03` missed only at the cuts.

### The judge

Two `reviewer` judges, one per story, with neutral names and one prompt (`tmp/lt15/judge/prompt.md`, key `tmp/lt15/judge/key.txt`: X = reference, Y = new). Verdicts: `tmp/lt15/judge/judge-X.md`, `tmp/lt15/judge/judge-Y.md`.

| Fault type | Reference (X) | New (Y) |
|---|---|---|
| 1. A fact that had left the history | 3 | 1 |
| 2. Contradicted a fact in the history, World or card | 19 | 23 |
| 3. Short entry of an absent Character | 0 | 1 |
| 4. No story clock | 2 | 4 |
| 5. Who knows what | 3 | 6 |
| 6. Role breaches (speaker labels) | 4 (0) | 3 (0) |
| 7. Wrong Room or Characters present | 14 spans | 20 runs |
| 8. Drift from the card | 4 | 4 |
| 9. Does not answer the entry before it | 4 | 3 |
| 10. Re-narration and repetition | 11 patterns | 9 patterns and 1 replayed scene |
| 11. Other | 12 | 6 |
| Narrations over 150 words | 83 of 102 | 29 of 101 |
| Scores (5 axes), overall | 3 3 3 3 3, 6/10 | 2 3 3 3 3, 6/10 |

- **known** The coordinator checked three findings of Y against the story: all are real.
- **known** One new kind of fault: #552 of Y wraps its paragraphs in `<Jezebel>…</Jezebel>` and `<Carla>…</Carla>`. No entry of X has such a tag. **inferred** The model copies the XML wrap of the history, as it copied the labels before. 1 of 101 Narrations.
- **known** The rewind loop of Y (#569–#576) follows a wrong `<GameState>`: call 1122 had "Front Gates" while #568 was in town. That is the known Room fault of places off the estate, not the format.
- **inferred** The counts differ in both directions within the noise of one trial for each side. Both judges give 6/10, and both name the era drift as a top fault.
- **known** The new Narrations keep the "under 150 words" rule of `output_format` far more often (29 against 83 over the limit).

### Expected against found

- Label lines about zero: yes, but already zero in the reference.
- No new kind of fault: one new kind, the XML speaker tags, once.
- Cached share does not drop by more than a few points: it rose by 22 points (Narrator-role calls) and 20 points (total).
