# Invariants the LLM theme relies on but does not enforce

Scope: model calls, Agents, prompting and the handling of LLM output.
Read-only investigation. No file under `src/`, `tests/`, `docs/`, `data/` or `scripts/` was changed.

Epistemic tags: **VERIFIED** = I read the reliance and every write path I name.
**INFERRED** = I name what I did not trace.

---

## llm-01. A Quantifier answer that means "nobody is present" says so; an answer that omits the field is not that answer.

1. **Invariant.** The Quantifier result holds an empty Characters-present set only when the model explicitly answered with an empty list. A missing or differently named field means "no reading", which keeps the Characters present.
2. **Relied on at.** `src/application/agents/quantifier/utils/parser.rs:14-16` — `#[serde(default, alias = "npcs_in_room")] characters_present: Vec<String>` cannot tell a missing key from `[]`. `parser.rs:43` then labels every parsed object `QuantifierConfidence::High`. `src/application/agents/quantifier/utils/orchestration.rs:166-186` treats High as authoritative and keeps the (possibly empty) list, so `src/domain/model/utils/game_rules.rs:63-66` writes `characters_present = []`.
3. **Write paths that could violate it.** External LLM output: any answer that is valid JSON without that key — `{}`, `{"movement": {...}}`, or `{"present": [...]}` / `{"characters": [...]}` from an author-edited Quantifier preset (`data/prompt_presets/quantifier/*.json` seeds, and any edited preset saved through `/prompt-presets`). The alias covers only one of the alternate names, and `PromptPreset::validate` does not check the answer shape.
4. **Enforcement.** NONE. The type has no "field absent" state, and no write path checks it. No test covers a missing `characters_present` (`parser_tests.rs` covers only `[]` and unknown ids).
5. **Consequence if violated.** The scene empties silently: every Character present is recorded as having `Left` (`quantifier.rs:37-70` set difference, applied at `game_rules.rs:141-155`), `currently_meeting` goes false, and any Trigger that reads those encounters changes its behaviour. The player never sees an error.
6. **Likelihood.** Plausible today: a model that answers a slightly different shape, or an author preset that names the field `characters`/`npcs`, is a normal LLM or World-author event.
7. **Epistemic status.** VERIFIED for the parser, the confidence label, and the empty write. I did not survey every seeded preset's asked-for shape.
8. **Suggested enforcement.** Give the field a "was it present" signal (e.g. `Option<Vec<String>>` or `#[serde(deny_unknown_fields)]` plus a required field) and make its absence `QuantifierConfidence::Low` in `try_parse_json_full`.

---

## llm-02. A movement destination names a Room the Swipe state already knows, or a place the World deliberately lacks — never a second copy of a Room that already exists.

1. **Invariant.** Applying a destination twice for the same place lands the Persona in the same Room.
2. **Relied on at.** `src/domain/model/utils/game_rules.rs:136-144` — `walk` looks the destination up with `state.room_by_id`, which matches a Room **id** only (`src/domain/model/map.rs:20-26`, `state/swipe_state.rs:59-61`). `game_rules.rs:124-133` turns every other value into a fresh Room.
3. **Write paths that could violate it.** External LLM output, invited by the prompt: `src/application/agents/quantifier/prompt.rs:52-62` prints `<Room id=".." name=".."/>` for every Room, so a name is as visible as an id; `src/domain/model/quantifier.rs:11` states "by id or name". `Room::new_dynamic` keys the new Room by `dynamic_{unix_seconds}` (`map.rs:50-56`), never by the destination text, so the same place named twice makes two Rooms with the same `name` and different ids.
4. **Enforcement.** DOC-ONLY for "by id or name"; NONE at the lookup. The prompt never lists the Game's added Rooms (`orchestration.rs:35-46` builds `all_rooms` from the World map only), so an added Room can never be re-entered by its own name either.
5. **Consequence if violated.** Wrong data stored, once per generation: `dynamic_rooms` grows with duplicates of one place, each with the placeholder description "A place you have never seen before", so the visual sidebar loses the real Room image (`image_path` is `None`) and the Game state stops matching the story.
6. **Likelihood.** A normal player trigger: the model answers with a `<name>` it was shown, which is the natural reading of the prompt.
7. **Epistemic status.** VERIFIED for the id-only lookup and the timestamp id. INFERRED for how often a model answers a name rather than an id — I did not run a model.
8. **Suggested enforcement.** Resolve the destination through one function that tries id, then `name` across map Rooms and `dynamic_rooms`, then falls back to a new Room (`game_rules::walk`).

---

## llm-03. A movement destination is reachable from the current Room.

1. **Invariant.** A move lands on a Room that the current Room's exits lead to.
2. **Relied on at.** Nothing checks it. `game_rules.rs:136-144` accepts any Room of the map, and `Room::exits` (`map.rs:42`) is written by World authors and **never read** anywhere in `src/` (grep for `.exits` returns only its declaration and the empty map of `new_dynamic`).
3. **Write paths that could violate it.** External LLM output. `src/application/agents/quantifier/prompt.rs:52-62` lists **every** Room of every region of the World, so the model is invited to pick a Room on the other side of the map.
4. **Enforcement.** NONE. `assert_room_exists` (`game_rules.rs:236-248`) only checks that the Room exists, and only under the `diagnostics` feature.
5. **Consequence if violated.** The Persona teleports. The Game state records a Room the story never walked to, and the World author's `exits` have no effect on gameplay at all.
6. **Likelihood.** An LLM output can do it today; nothing in the prompt or the rules stops it.
7. **Epistemic status.** VERIFIED (the exits field has no reader; the Room list is map-wide).
8. **Suggested enforcement.** One predicate in `game_rules::walk` that accepts the current Room, a Room named by `current_room.exits`, and an explicitly new destination; and list only the reachable Rooms in the Quantifier prompt.

---

## llm-04. Every prompt part sent as a `user` or `assistant` message holds text.

1. **Invariant.** A history entry and the Player Input carry non-empty text, so the request body never holds `{"role":"user","content":""}`.
2. **Relied on at.** `src/adapters/driven/llm/transport/utils/request.rs:70-74` drops an empty part only when `role == System`: `filter(|part| part.role != PromptRole::System || !part.text.is_empty())`. The history parts come from `src/application/prompting/assembler.rs:449-459`, which copies `entry.text` unchanged.
3. **Write paths that could violate it.** `src/adapters/driving/http/history/handlers/history.rs:20-32` passes `form.text` to `GameWriter::edit` with no check; `src/application/generation/gate.rs:560-573` calls `tx.set_swipe_text(at, text)`; `src/adapters/driven/storage/transaction.rs:206-216` writes it to `message_swipes.text` (no `CHECK`). The browser is no guard: the edit textarea has no `required` and no empty check (`assets/dashboard-story-log.js:71-77`). The narrator's own answer is guarded (`narration/narrator.rs:145-153` rejects a blank answer), so the edit route is the one hole.
4. **Enforcement.** PARTIAL. The system part is dropped when empty (`request.rs:73`); the `user` and `assistant` parts are not. `merge_single_user_message` drops every empty part (`prompting/prompt_merge.rs:9`), so the `single_user_message` connection setting hides the problem on those Connections only.
5. **Consequence if violated.** An empty text block reaches the provider. Providers that reject empty content answer HTTP 400, so every later generation of that Game fails until the text is restored; the player sees the failure banner, not "the text is empty".
6. **Likelihood.** A normal player can do it: clear the text in the History editor and press Save.
7. **Epistemic status.** VERIFIED for the empty text arriving at the wire. INFERRED that a given provider rejects it — I did not test a provider.
8. **Suggested enforcement.** Refuse an empty text in `GameWriter::edit` (and in the router's form), so the invariant lives at the write, not at the transport.

---

## llm-05. Every Agent call fits the Connection's context window.

1. **Invariant.** The Quantifier and Options prompts and their response length stay inside the Connection's `max_context_tokens`.
2. **Relied on at.** `src/application/agents/quantifier/utils/orchestration.rs:47-63` passes `input.story` (the four newest Messages, no cut) straight into the prompt, and `recorder.complete("quantifier", &prompt, call)` at line 98 sends `settings.call.max_tokens` unchanged. `src/application/agents/options/utils/orchestration.rs:26-40` does the same for Options. `src/application/agents/quantifier/prompt.rs:19-45` adds every Character id/name and every Room id/name of the World, plus the full descriptions of the Characters present.
3. **Write paths that could violate it.** A long Narration (the narrator may answer up to `max_tokens`, e.g. 2048) repeated four times in `AGENT_HISTORY_WINDOW`, plus a large World's id lists, against an Ollama default window of 8192 (`llm_backend.rs:70-76`).
4. **Enforcement.** PARTIAL. The narrator path is bounded: `TokenBudget::history_space` and `history_cut` (`prompting/token_budget.rs:57-62`, `prompting/assembler.rs:192-216`) plus `TokenBudget::response_tokens` (`token_budget.rs:67-87`). The two Agent paths hold no `TokenBudget` and never read `connection.resolve_max_context_tokens()`; `src/domain/model/agent.rs:104-108` carries only `call` and `option_count`.
5. **Consequence if violated.** The Quantifier fails on the provider's context error, twice, and then "keeps the Room and the Characters present" (`orchestration.rs:150-160`): the scene silently freezes and the game rules stop applying any reading. The Options run lands as an `OptionsFailure` notice. No story data is lost.
6. **Likelihood.** A long story text plus a small-window Connection. The player can trigger it with a long Narration and the default Ollama window.
7. **Epistemic status.** VERIFIED for the missing budget. INFERRED for the real token size of a World's id lists.
8. **Suggested enforcement.** Put a `TokenBudget` on `AgentSettings` and cut `HookInput::story` with the same `HistoryCut` the narrator uses, in `Job::hook_settings`.

---

## llm-06. The engine's token estimate is an upper bound for the model's token count.

1. **Invariant.** `estimate_tokens` never counts fewer tokens than the model will see, so a prompt that fits `max_context_tokens` by the estimate also fits it on the provider.
2. **Relied on at.** `src/application/prompting/token_budget.rs:92-96` — `CHARS_PER_TOKEN = 4`, `text.chars().count().div_ceil(4)`. The whole fit leans on it: `history_space` (`token_budget.rs:57-62`), the cut (`assembler.rs:192-216`) and the clamped response length (`token_budget.rs:67-87`).
3. **Write paths that could violate it.** Any text with a token count near its character count: a non-Latin story or Persona text, markup-heavy text, and the chat-template overhead (`MESSAGE_OVERHEAD_TOKENS = 4`, `token_budget.rs:13-14`), which is a guess. The window itself is a second guess: `connection.resolve_max_context_tokens()` falls back to the provider default (`settings.rs:446-449`, `llm_backend.rs:70-76`), which no check ties to the named model.
4. **Enforcement.** NONE. The code calls it an estimate (`token_budget.rs:12-14`), and nothing compares the estimate with the real usage; the response's `usage` field is read only for the `TokenBudgetSpent` failure text (`transport/utils/response.rs:28-37`).
5. **Consequence if violated.** The narrator sends an over-long prompt and the provider answers a context-overflow error. The Job fails with that error line; no data is corrupted, but the Game cannot advance until the player shortens the text or the window setting.
6. **Likelihood.** A player who writes in a non-Latin script, or a Connection whose `max_context_tokens` is left empty for a small-window model.
7. **Epistemic status.** VERIFIED for the arithmetic and the unvalidated window. INFERRED for which providers accept an over-long prompt instead of refusing it.
8. **Suggested enforcement.** Record the provider's reported prompt tokens (`usage.prompt_tokens`) per Connection and use the larger of the estimate and the observed ratio; keep `resolve_max_context_tokens` a required field for a non-default model.

---

## llm-07. A byte index into a provider's body falls on a character boundary.

1. **Invariant.** Code that shortens an LLM answer or an HTTP error body slices it by characters, not by bytes.
2. **Relied on at.** `src/adapters/driven/llm/transport/utils/response.rs:139-141` — `format!("{}...", &error_body[..500])` on the `String` from `response.text()`. A second site: `src/application/agents/quantifier/utils/orchestration.rs:108-111` slices `&response[..response.len().min(200)]` inside `tracing::debug!`.
3. **Write paths that could violate it.** External input: a non-2xx body longer than 500 bytes whose 500th byte is inside a multi-byte character (a localised error page, an HTML error page, a model echo). The Quantifier site needs `RUST_LOG=debug` and a Quantifier answer whose 200th byte splits a character.
4. **Enforcement.** NONE at either site. No `is_char_boundary` and no `chars().take()`.
5. **Consequence if violated.** A panic inside the model call. On the Job path the `JobWriter` drop frees the busy state, so the Game is not stuck, but the failure line is never saved: the player sees the spinner stop with no error. In the Connection test the call runs inside the HTTP handler (`connection_test_service.rs:59-83`), so the request dies without an answer.
6. **Likelihood.** Only a provider that returns a long non-ASCII error body. Unusual, but no World author or player is needed — it depends on the provider.
7. **Epistemic status.** VERIFIED that the slice is byte-indexed on a `String`. INFERRED for how often a body crosses the boundary.
8. **Suggested enforcement.** One helper that truncates by `chars()` (or `char_indices`) and both sites use it.

---

## llm-08. Every Agent named in the settings is registered, or the failure is reported.

1. **Invariant.** An `AgentConfig` whose `agent_type` names no Agent stops the boot, or at least keeps the other Agents and is logged.
2. **Relied on at.** `src/application/agents/runner.rs:63-66` returns `Err(EngineError::Config("Unknown agent type"))` for that one config. `src/bootstrap/wiring.rs:147-152` answers that error with `.unwrap_or_default()`: `AgentRunner::default()` holds **no** Agent at all, and nothing logs the error.
3. **Write paths that could violate it.** `AppSettings.agents` in the settings row. The seed file `data/settings.json:74-86` and any hand edit of the row; no HTTP route writes it (grep for `agents` under `src/adapters/driving/http` finds none). A typo in `agent_type`, or a config written by a newer version.
4. **Enforcement.** NONE. `AgentRunner::is_empty` (`runner.rs:100-102`) exists but no production path calls it.
5. **Consequence if violated.** Silent wrong behaviour: the Quantifier never runs, so the Characters present and movement never change and Triggers keyed on encounters never fire; the Options dock shows "Options agent is not available" on every run. The startup log shows no reason.
6. **Likelihood.** Only a code bug or a manual edit of the settings JSON — a player cannot reach it.
7. **Epistemic status.** VERIFIED.
8. **Suggested enforcement.** Make `build_app_graph` propagate the `AgentRunner::from_configs` error, as it already does for `check_connection_slots` on the line above.

---

## llm-09. The agent name a call records is one of the declared Role names.

1. **Invariant.** `LlmCallRecorder::complete` gets a name equal to `Role::agent_name()` for one of `Role::ALL`.
2. **Relied on at.** `src/application/games/view_query.rs:249-267` filters the retained LLM messages by `message.agent_name == role.agent_name()` to build each Role's health, and `src/adapters/driving/http/builders/headers.rs:55-70` turns a missing entry into `NoCalls`. A name that matches no Role is invisible to both.
3. **Write paths that could violate it.** `src/application/agents/quantifier/utils/orchestration.rs:98` passes the literal `"quantifier"`; every other call site uses a constant (`narrator.rs:139-144`, `options/utils/orchestration.rs:79`, `ports/llm_provider.rs:10-14`). A rename of `Role::agent_name()` for Quantifier, or a typo in that literal, writes a name no Role owns. `connection_test_service.rs:30` and `mock.rs:118-119` hold the same kind of literal but never reach the table.
4. **Enforcement.** NONE for the literal: nothing ties the argument type to `Role`. (The other three call sites are PARTIAL: they use a constant, which still is not checked against `Role::ALL`.)
5. **Consequence if violated.** The Quantifier's health cell always reads NoCalls, and the failure banner never names a failed Quantifier call, while the LLM record dialog still shows the rows. A broken Quantifier Connection therefore looks healthy.
6. **Likelihood.** Only a code edit. No player, LLM or World author can trigger it.
7. **Epistemic status.** VERIFIED (I read the filter and every call site).
8. **Suggested enforcement.** Change the argument to `Role` (or a `&'static str` produced only by `Role::agent_name()`), so the literal cannot exist.

---

## llm-10. A call sends `max_tokens` of at least 1.

1. **Invariant.** No request body carries `max_tokens: 0`.
2. **Relied on at.** `src/adapters/driven/llm/transport/utils/request.rs:80` writes `payload["max_tokens"] = json!(request.call.max_tokens)` unconditionally.
3. **Write paths that could violate it.** An Options or Quantifier preset with `"max_tokens": 0` in its `parameters`: `GenerationParameters.max_tokens` is `Option<u32>` with no lower bound (`generation_parameters.rs:15-24`), `PromptPreset::validate` checks only `refused_custom_key` and `option_count` (`prompt_preset.rs:104-125`), and `max_tokens_for` passes the 0 through (`settings.rs:372-377`). `src/application/pipeline/job.rs:477-484` builds `CallSettings` for the two Agent paths from that value.
4. **Enforcement.** PARTIAL. The narrator path clamps: `TokenBudget::response_tokens` ends in `.max(1)` (`token_budget.rs:82-85`) and `narrator.rs:139-144` replaces `call.max_tokens` with it. The Quantifier and Options calls take `plan.settings` unchanged.
5. **Consequence if violated.** A provider that refuses `max_tokens: 0` answers 400: the scene freezes (the Quantifier falls back to keeping present) and Options shows a failure notice. Wrong behaviour, no data loss.
6. **Likelihood.** An author-edited preset through `/prompt-presets`. Nothing rejects the value.
7. **Epistemic status.** VERIFIED for the value reaching the wire from an Agent call. INFERRED that a provider refuses 0.
8. **Suggested enforcement.** One clamp in `LlmConnection::max_tokens_for` (or a `NonZeroU32` field on `CallSettings`), so every call path shares it.

---

## Checked and enforced (ruled out)

- **The Quantifier result names only Characters of the Game's World.** Enforced twice: `parser.rs:36`/`159` filter by the World's ids, and `orchestration.rs:173-178` filters again against `world.characters`; `game_rules.rs:63-66` also drops an unknown id.
- **The Options set is non-empty.** Enforced at the write of the dock: `src/application/agents/options/utils/orchestration.rs:53-70` returns `Err` for a zero-parse answer, and `pipeline/job.rs:357-368` turns it into an `OptionsFailure` instead of an empty offer. (The count can be fewer than `option_count` and two Options can be equal; I found no reader that relies on either, so I do not report it.)
- **The model answer is non-empty after sanitization.** `transport/utils/response.rs:63-70` rejects a blank `content`, and `narration/narrator.rs:145-153` rejects a text that `sanitize_llm_output` emptied. The failure is recorded.
- **A missing `choices[0]`, a null `message`, an `error` object and non-JSON are loud failures.** `response.rs:46-79` returns `ParseError`/`Http`/`EmptyResponse`, never a silent empty answer.
- **Every model call is recorded, also on failure and timeout.** `llm_recorder.rs:59-90` saves a failure row for every `LlmCallFailure` and a row for every answer; a failed record write on the success path fails the call (`llm_recorder.rs:87`). The two intentional exceptions are documented: the Connection test records nothing (`connection_test_service.rs:1-8`) and a call that fails before a send stores an empty `raw_request_json`.
- **The dock holds at most one record per Game and the LLM message window stays bounded.** `game_docks.game_id` is the primary key (`storage/utils/plumbing.rs:655-660`), and `save_llm_message` prunes to the newest 50 rows (`storage/llm_messages.rs:37-46`), which is what `role_health` reads.
- **The newest Messages are the ones an Agent reads.** `stories.rs:189-206` reads `ORDER BY m.id DESC LIMIT n` and reverses, so `AGENT_HISTORY_WINDOW` gives the newest entries in ascending order.
- **The `option_count` of a preset is at least 1 and belongs to an Options preset.** Enforced at both writes: `PromptPreset::validate` (`prompt_preset.rs:110-122`), called by `PromptPresetService::save_preset` (`prompt_preset_service.rs:56-59`) and by the seed loader (`bootstrap/run.rs:254-257`).
- **A `custom` sampler cannot shadow a key the engine writes.** `GenerationParameters::refused_custom_key` plus the same `validate` call (`generation_parameters.rs:27-32`), and `build_request_payload` writes the engine keys after `custom`.
- **The history space is not negative.** `TokenBudget::history_space` uses `saturating_sub` throughout (`token_budget.rs:57-62`), and `HistoryCut::of`/`drop_to_free` bound `kept_from` by the entry count (`history_cut.rs:24-59`), so the `kept_tokens -=` subtraction cannot pass zero.
- **The system part is first and the request has a stable shape.** `assembler.rs:258-262` builds `[system, first user, …history…]`, `part_tokens` skips index 0 as the system part (`assembler.rs:178-183`), and `render_prompt` always appends the non-empty fixed after-layers, so the last message is a `user` part.
- **The template placeholders of author text cannot be injected by the Player Input or the Impersonate direction.** `assembler.rs:470-472` sanitizes it with `sanitize_for_prompt` (`builders/sections.rs:7-29`), and only author-controlled text goes through `TemplateVars::render`.
- **An empty `System` prompt part never reaches the wire.** `request.rs:73` drops it; in `single_user_message` mode `prompt_merge.rs:9` drops every empty part.
