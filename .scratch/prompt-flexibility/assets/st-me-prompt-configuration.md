# How ST and ME let an author change a call

Answer to [How ST and ME let an author change a call](../issues/02-st-me-prompt-configuration.md).

## Scope and method

Both apps were read from source at the pinned commits. No app was run.

| App | Commit | Clone |
|---|---|---|
| SillyTavern (ST) | `06bde939fb1e9c4c8d8641d810f0a916b5bce127` | `/tmp/research/st` |
| Marinara Engine (ME) | `457e827a3d3291be978121f8c50c98c5dd9d644d` | `/tmp/research/marinara` |

**Status marks.** Each row starts with **known** or **inferred**.

- **known** — the source file states it, and the file path is given.
- **inferred** — the source file implies it, and the reader must combine two places to get it.

Every path is relative to the app's clone root. Official docs are a secondary source and are listed apart in [Secondary sources](#secondary-sources).

**Short answer.** Both apps give the author the same four levers: the text of each named part, the order of the parts, the role each part is sent as, and the depth at which a part joins the history. Both let a preset or a per-chat override hold those choices. ST keeps every lever in the browser and in one preset file. ME keeps them on the server in database rows. Neither app lets the author change the engine's own token-budget maths or its provider quirks.

---

## SillyTavern

ST builds a chat-completion prompt in the browser. One OpenAI preset holds the connection, the samplers, and the prompt list. A prompt list has two halves: `prompts[]` holds the text and the role, and `prompt_order[]` holds the order and the on/off switch.

| Part of a call | Mechanism | Changeable by an author | Source path |
|---|---|---|---|
| **known** Text of a named prompt (Main, Auxiliary, Post-History Instructions, Enhance Definitions, any custom prompt) | `prompts[].content`. Edited in the Prompt Manager editor or in Quick Prompts Edit. A copy of the text is also kept in the browser settings, so a preset carries its own prompts | Yes. The author writes the text. `{{char}}`, `{{user}}`, `{{group}}` and other macros are substituted at build time | `public/scripts/PromptManager.js` (`class Prompt`, `preparePrompt`), `default/content/presets/openai/Default.json` |
| **known** Order of the parts | The drag-and-drop prompt list. Stored as `prompt_order[].order` in the preset. A prompt is placed in the array slot of its index | Yes. The author drags a prompt. A prompt that is not in the list is not sent | `public/scripts/PromptManager.js` (`getPromptCollection`), `public/scripts/openai.js` (`ChatCompletion.add`, `addToChatCompletion`) |
| **known** On/off switch of each part | `prompt_order[].enabled`. The strategy is `global`, keyed by the dummy character id 100001, or `character`, keyed by the character id | Yes. A disabled prompt is skipped. `main` stays in the list with empty text, because extensions need its slot | `public/scripts/PromptManager.js` (`getPromptCollection`, `isPromptDisabledForActiveCharacter`), `public/scripts/openai.js` (`promptOrder.strategy`, `dummyId`) |
| **known** Role of each part | `Prompt.role` = `system`, `user`, or `assistant`. The role of a system prompt can be overridden from the prompt list entry for the same identifier | Yes | `public/scripts/PromptManager.js` (`class Prompt`), `public/scripts/openai.js` (`preparePromptsForChatCompletion`) |
| **known** Position of each part | `injection_position`: `RELATIVE` (0) follows the prompt order; `ABSOLUTE` (1) joins the chat history at a depth | Yes. Set per prompt in the editor | `public/scripts/PromptManager.js` (`INJECTION_POSITION`), `public/scripts/openai.js` (`populationInjectionPrompts`) |
| **known** Depth of an in-chat part | `injection_depth`, default 4. Depth 0 is after the last message, depth 1 is before it | Yes | `public/scripts/PromptManager.js` (`DEFAULT_DEPTH`), `public/scripts/openai.js` (`populationInjectionPrompts`) |
| **known** Order of parts that share one depth | `injection_order`, default 100. A higher number is placed nearer the start of that depth group. The roles inside one group run system, then user, then assistant | Yes | `public/scripts/openai.js` (`populationInjectionPrompts`) |
| **known** Which generation types send a part | `injection_trigger` list. An empty list means every generation type. Values match the generation type, for example `normal`, `continue`, `impersonate`, `swipe`, `regenerate`, `quiet` | Yes. Triggers are picked per prompt in the editor | `public/scripts/PromptManager.js` (`shouldTrigger`, `getPromptCollection`) |
| **known** Lock a part against a character card override | `forbid_overrides`. When true, the card's `system_prompt` cannot replace `main`, and its `post_history_instructions` cannot replace `jailbreak` | Yes | `public/scripts/openai.js` (`preparePromptsForChatCompletion`) |
| **known** Markers | A prompt with `marker: true` has no text. The build fills it with runtime data: world info before, world info after, character description, character personality, scenario, persona description, dialogue examples, chat history | The author sets the marker's place in the order, its role, its position, its depth and its on/off switch. The author cannot write the marker text | `default/content/presets/openai/Default.json`, `public/scripts/openai.js` (`preparePromptsForChatCompletion`, `populateChatHistory`, `populateDialogueExamples`) |
| **known** Main Prompt (`main`) | A prompt with default text. Holds the global instruction to the model | Yes | `default/content/presets/openai/Default.json`, `public/scripts/openai.js` (`default_main_prompt`) |
| **known** Auxiliary Prompt (`nsfw`) | A prompt with empty default text. Sent with the `system` role. The build appends it after the marker prompts and before the in-chat prompts, and its index in the prompt order sets its place | Yes | `public/scripts/openai.js` (`systemPrompts = ['nsfw', 'jailbreak']`, `addToChatCompletion`) |
| **known** Post-History Instructions (`jailbreak`) | A prompt with empty default text and the `system` role. Its default place is the bottom of the list, so it is the last text before the reply. It is a normal prompt-list entry, not a special block | Yes. The place comes from the prompt order, and the role comes from the prompt list entry | `public/scripts/openai.js` (`systemPrompts = ['nsfw', 'jailbreak']`), `default/content/presets/openai/Default.json` |
| **known** Enhance Definitions | A prompt that asks the model to enrich the character. Shipped in the preset and disabled in the default order | Yes | `default/content/presets/openai/Default.json`, `public/scripts/openai.js` (`default_enhance_definitions_prompt`) |
| **known** Character description, personality, scenario | Markers filled from the character card. `scenario_format` and `personality_format` are substituted into the scenario and the personality text before they are sent | The card text is author data. The two format templates are editable | `public/scripts/openai.js` (`preparePromptsForChatCompletion`), `default/content/presets/openai/Default.json` |
| **known** Persona description | `power_user.persona_description`. Added to the prompt only when `persona_description_position` is `IN_PROMPT` (0). Other values are `TOP_AN` (2), `BOTTOM_AN` (3), `AT_DEPTH` (4), `NONE` (9) | Yes | `public/scripts/power-user.js`, `public/scripts/personas.js` (`persona_description_positions`), `public/scripts/openai.js` (`preparePromptsForChatCompletion`) |
| **known** Character card `system_prompt` | Replaces the `main` text when `power_user.prefer_character_prompt` is on and the prompt does not set `forbid_overrides`. A chat-level `chat_metadata.system_prompt` wins over the card value | Yes. The author writes the card field. The user turns the switch on or off | `public/script.js` (`getCharacterCardFieldsLazy`), `public/scripts/openai.js` (`preparePromptsForChatCompletion`) |
| **known** Character card `post_history_instructions` | Replaces the `jailbreak` text when `power_user.prefer_character_jailbreak` is on and `forbid_overrides` is off | Yes | `public/script.js` (`getCharacterCardFieldsLazy`), `public/scripts/openai.js` (`preparePromptsForChatCompletion`) |
| **known** Character card `extensions.depth_prompt` | In-chat injection registered as an extension prompt. Depth defaults to 4 and the role to `system` | Yes. The card sets the text, the depth and the role | `public/script.js` (`depth_prompt_depth_default`, `depth_prompt_role_default`, `setExtensionPrompt(inject_ids.DEPTH_PROMPT, ...)`) |
| **known** Author's Note | Extension prompt `2_floating_prompt`. Per-chat `note_prompt`, `note_position`, `note_depth`, `note_interval`, `note_role`. Defaults come from `extension_settings.note.default*`: position `IN_CHAT` (1), depth 4, interval 1, role `system` | Yes. The author or the user sets all five values per chat | `public/scripts/authors-note.js` (`metadata_keys`, `DEFAULT_POSITION`, `DEFAULT_DEPTH`, `DEFAULT_INTERVAL`, `DEFAULT_ROLE`, `setFloatingPrompt`) |
| **known** World Info entry positions | `world_info_position`: `before` (0, ↑Char), `after` (1, ↓Char), `ANTop` (2, top of the Author's Note), `ANBottom` (3, bottom of the Author's Note), `atDepth` (4, @D), `EMTop` (5, before the example messages), `EMBottom` (6, after the example messages), `outlet` (7, a `{{outlet::name}}` macro only) | Yes. Set per entry | `public/scripts/world-info.js` (`world_info_position`, `checkWorldInfo`) |
| **known** World Info in-chat depth and role | A `atDepth` entry carries `depth` (default 4) and `role` (default 0 = system), plus `order`. Entries with the same depth and role share one inserted message | Yes | `public/scripts/world-info.js` (`newWorldInfoEntryDefinition`, `checkWorldInfo`) |
| **known** World Info wrapper | `oai_settings.wi_format`, default `{0}`, where `{0}` is the entry text. An empty format inserts the text as it is | Yes | `public/scripts/openai.js` (`formatWorldInfo`, `default_wi_format`), `default/content/presets/openai/Default.json` |
| **known** Extension prompts with position before or after the main prompt | An unknown extension prompt whose position is `BEFORE_PROMPT` or `IN_PROMPT` becomes a system prompt with `position` `start` or `end`. This is how `/inject position=before` and `/inject position=after` arrive | Yes. A script or extension registers it | `public/scripts/openai.js` (`preparePromptsForChatCompletion`, `getPromptPosition`) |
| **known** Impersonation prompt | `oai_settings.impersonation_prompt`. Built as a `system` control prompt with the identifier `impersonate`. Added only when the generation type is `impersonate` | Yes. The author edits the text in the preset | `public/scripts/openai.js` (`default_impersonation_prompt`, `preparePromptsForChatCompletion`, `populateChatCompletion`) |
| **known** Place of the control prompts | The impersonation prompt and the quiet prompt are the last prompts, after the history and the example messages. The quiet prompt is always the final one | No. The engine fixes the place. Only the text is the author's | `public/scripts/openai.js` (`controlPrompts`, `chatCompletion.add(controlPrompts)`) |
| **known** Impersonation assistant prefill | `oai_settings.assistant_impersonation`. Sent as the Claude assistant prefill on an impersonate call only | Yes | `public/scripts/openai.js` (`generate_data.assistant_prefill` for `type === 'impersonate'`) |
| **known** The quiet prompt on an impersonate call | The optional `/impersonate <text>` argument becomes the `quietPrompt` system message, placed after the impersonation prompt | Yes. The caller passes it | `public/scripts/slash-commands.js` (`/impersonate`), `public/scripts/openai.js` (`populateChatCompletion`) |
| **known** Group nudge | `oai_settings.group_nudge_prompt`. A group-only prompt. Skipped when the generation type is `impersonate` | Yes | `public/scripts/openai.js` (`noGroupNudgeTypes`, `populateChatHistory`) |
| **known** New chat, new group chat and new example chat markers | `new_chat_prompt`, `new_group_chat_prompt`, `new_example_chat_prompt`. A system message before the history and before each example block | Yes. An empty value turns the marker off | `public/scripts/openai.js` (`populateChatHistory`, `populateDialogueExamples`) |
| **known** Continue nudge | `continue_nudge_prompt`. A `system` message placed after the moved last message, only when the type is `continue`. With `continue_prefill` on, the last message is sent as an assistant prefill instead and the nudge is not used | Yes | `public/scripts/openai.js` (`populateChatHistory`, `continue_prefill`) |
| **known** The `continue_postfix` string | Appended to the cycle prompt before the continue nudge. Values are none, space, newline, double newline | Yes | `public/script.js` (`continue_postfix`), `public/scripts/openai.js` (`continue_postfix_types`) |
| **known** Replace empty message | `send_if_empty`. Sent as the user message when the box is empty and Send is pressed | Yes | `public/scripts/openai.js` (`populateChatHistory`, `emptyUserMessageReplacement`) |
| **known** Names in history | `names_behavior`: `NONE` (-1) sends no name. `DEFAULT` (0) prefixes the name in a group chat, and on a message that carries an avatar, when the message is not the user's. `COMPLETION` (1) sends no name. `CONTENT` (2) prefixes the name in the content. A narrator row never gets a name | Yes | `public/scripts/openai.js` (`character_names_behavior`, `setOpenAIMessages`) |
| **known** Squash system messages | `squash_system_messages`. Joins neighbour system messages that carry no name, and leaves `newMainChat`, `newChat` and `groupNudge` apart | Yes | `public/scripts/openai.js` (`ChatCompletion.squashSystemMessages`) |
| **known** Use system prompt | `use_sysprompt`. Sent to the server, which folds the leading system messages into a separate `system` field for a source that takes one. Without it the system turns stay in the message list | Yes | `public/scripts/openai.js` (`generate_data.use_sysprompt`), `src/endpoints/backends/chat-completions.js` (`useSystemPrompt`), `src/prompt-converters.js` (`convertClaudeMessages`) |
| **known** Samplers and limits for chat completion | One preset holds `temperature`, `frequency_penalty`, `presence_penalty`, `top_p`, `top_k`, `top_a`, `min_p`, `repetition_penalty`, `openai_max_context`, `openai_max_tokens`, `seed`, `n`. Each source deletes the fields that it does not accept | Yes. All are preset fields with a UI control | `public/scripts/openai.js` (`generate_data`, `openai_max_stop_strings` blocks), `default/content/presets/openai/Default.json` |
| **known** Reasoning and verbosity | `reasoning_effort`, `verbosity`, `show_thoughts`. Sent as `reasoning_effort`, `verbosity`, `include_reasoning` | Yes | `public/scripts/openai.js` (`getReasoningEffort`, `getVerbosity`, `generate_data`) |
| **known** Stop strings | `getCustomStoppingStrings(openai_max_stop_strings)`. Built from the preset and the macros | Partly. The stop list comes from settings, and the engine decides the cap | `public/scripts/openai.js` (`generate_data.stop`) |
| **known** Token budget | `tokenBudget = openai_max_context - openai_max_tokens`. Each message is charged as it is added. A message that does not fit is dropped, oldest first | No. The engine fixes the rule. The author sets only the two numbers | `public/scripts/openai.js` (`ChatCompletion.setTokenBudget`, `checkTokenBudget`) |
| **known** Instruct template (text-completion sources only) | A preset of sequences: input, output, system, suffixes, stop sequence, `names_behavior`, `skip_examples`, `sequences_as_stop_strings`, `activation_regex` | Yes. 38 shipped templates | `public/scripts/instruct-mode.js`, `default/content/presets/instruct/*.json` |
| **known** Context template and story string (text-completion sources only) | `story_string` is a Handlebars-style template over `system`, `wiBefore`, `description`, `personality`, `scenario`, `wiAfter`, `persona`, `anchorBefore`, `anchorAfter`. It also carries `story_string_position`, `story_string_depth`, `story_string_role`, `example_separator`, `chat_start`, `use_stop_strings`, `names_as_stop_strings`, `always_force_name2` | Yes. 34 shipped templates, and the author can edit the text | `public/scripts/power-user.js` (`defaultStoryString`, context defaults), `public/script.js` (`applyStoryStringInject`), `default/content/presets/context/Default.json` |
| **known** System prompt preset (text-completion sources only) | `power_user.sysprompt` holds `content` and `post_history`, from one of 13 named presets | Yes | `public/scripts/power-user.js`, `default/content/presets/sysprompt/Neutral - Chat.json` |
| **known** Text-completion samplers | A separate preset family under `presets/textgen`. Six templates, and the Default template holds 61 fields, including `temp`, `top_p`, `min_p`, `rep_pen`, `dry_*`, `mirostat_*`, `sampler_priority` and `grammar_string` | Yes | `default/content/presets/textgen/Default.json` |
| **inferred** Chat history content | The history is a marker whose text is the chat rows. A row marked `is_system` is filtered out before the build, so a display-only row never reaches the model | No. The author sets a row's system type, not its inclusion rule | `public/script.js` (`coreChat` filter), `public/scripts/openai.js` (`populateChatHistory`) |

---

## Marinara Engine

ME builds the prompt on the server. A prompt preset is a set of database rows: one preset row, section rows, group rows and choice-block rows. `assemblePrompt` reads the rows and builds the message list.

| Part of a call | Mechanism | Changeable by an author | Source path |
|---|---|---|---|
| **known** Text of a section | `prompt_sections.content`. Edited in the preset editor | Yes. `{{...}}` macros and choice-block variables are resolved into it | `packages/server/src/db/schema/prompts.ts`, `packages/server/src/services/prompt/assembler.ts` (`resolveSection`) |
| **known** Order of the sections | `promptPresets.sectionOrder`, a JSON array of section ids. `assemblePrompt` walks that array | Yes. The author reorders the array | `packages/server/src/db/schema/prompts.ts`, `assembler.ts` (phase 1) |
| **known** Role of a section | `prompt_sections.role` = `system`, `user`, or `assistant` | Yes | `packages/server/src/db/schema/prompts.ts`, `assembler.ts` (`resolveSection`) |
| **known** On/off switch of a section | `prompt_sections.enabled` = `"true"` or `"false"` | Yes | `assembler.ts` (phase 1) |
| **known** On/off switch of a group | `prompt_groups.enabled`. A disabled group drops every section that belongs to it | Yes | `assembler.ts` (phase 1) |
| **known** Position of a section: ordered or at depth | `injectionPosition` = `ordered` or `depth`, with `injectionDepth`. A `depth` section joins the history at that depth | Yes | `packages/shared/src/types/prompt.ts` (`InjectionPosition`), `assembler.ts` (phase 5) |
| **known** `injectionOrder` | The column sets the `ORDER BY` of `listSections`, so it orders the database read. `assemblePrompt` then walks `sectionOrder`, and it never reads the column. The built prompt therefore follows `sectionOrder` alone | No. The column does not place a section in the prompt | `packages/server/src/db/schema/prompts.ts`, `packages/server/src/services/storage/prompts.storage.ts` (`listSections`), `assembler.ts` (phase 1) |
| **known** Wrapper format of the whole preset | `promptPresets.wrapFormat` = `xml`, `markdown`, or `none`. The XML tag comes from the section name. A Markdown heading level comes from the group depth | Yes | `packages/server/src/services/prompt/format-engine.ts` (`wrapContent`), `packages/shared/src/types/prompt.ts` (`WrapFormat`) |
| **known** Wrapper of one section | `skipWrap` = `"true"` sends the section text without the preset wrapper. A marker always keeps its wrapper | Yes | `assembler.ts` (`resolveSection`, `skipWrap`) |
| **known** Groups | `prompt_groups.name` becomes the wrapper tag or the Markdown heading. `parentGroupId` nests a group. `order` sorts the groups | Yes | `packages/server/src/db/schema/prompts.ts`, `assembler.ts` (`buildGroupMessages`, `wrapGroup`) |
| **known** Markers | `isMarker` plus `markerConfig.type`. The type list is `character`, `lorebook`, `persona`, `chat_history`, `chat_summary`, `current_scene_summary`, `recalled_scenes`, `recalled_messages`, `id_macro_cards`, `world_info_before`, `world_info_after`, `dialogue_examples`, `agent_data` | The author picks the type, the place, the order, the role and the depth. The text is runtime data. An `agent_data` marker is the exception: it keeps the section text as a `{{agent::TYPE}}` template | `packages/shared/src/types/prompt.ts` (`MarkerType`, `MarkerConfig`), `assembler.ts` (`resolveSection`) |
| **known** Choice blocks (preset variables) | `choice_blocks` rows, one per variable: `variableName`, `question`, `options`, `multiSelect`, `separator`, `randomPick`, `displayMode`. A chat picks one value per variable, and the value replaces `{{variableName}}` | Yes. The author writes the options. The default preset ships 7: `role`, `guidelines`, `narration`, `pov`, `tense`, `length`, `language` | `packages/server/src/db/schema/prompts.ts`, `packages/shared/src/types/prompt.ts` (`ChoiceBlock`), `assembler.ts` (choice merge) |
| **known** Macros in section text | `{{user}}`, `{{char}}`, card fields, `{{outlet::name}}`, `{{agent::TYPE}}`, `{{lastGenerationType}}`, `{{idle_duration}}`, and the choice variables | Yes. The author writes them | `packages/server/src/services/prompt/macro-context.ts`, `assembler.ts` |
| **known** Chat history marker | `chat_history` marker returns the chat rows as several messages and sets `contextKind: "history"`. The depth injection is measured from it | The place in the order is the author's. The content is the chat | `assembler.ts` (`resolveSection`, `findHistoryBounds`) |
| **known** Lorebook positions | `position` 0 = before the character blocks, 1 = after them, 2 = at a depth with `role` and `depth`, 7 = a named Outlet that only a `{{outlet::name}}` macro reads. Each entry also has `order`, `sticky`, `cooldown`, `delay`, `ephemeral`, `group` | Yes. Set per entry | `packages/server/src/services/lorebook/prompt-injector.ts` (`buildWorldInfoBlocks`, `getDepthInjectedEntries`), `packages/server/src/db/schema/lorebooks.ts` |
| **known** Impersonation instruction text | `DEFAULT_IMPERSONATE_PROMPT` when the chat has no custom template. A per-chat template list (`impersonate-prompt-templates`, at most 20 templates) and a per-chat `impersonatePrompt` override it. `{{user}}`, `{{persona_description}}` and `{{impersonate_direction}}` are filled in. A line that holds only an empty placeholder is dropped | Yes. The author writes a template. The default is engine text | `packages/shared/src/constants/impersonate.ts`, `packages/server/src/services/conversation/impersonate-prompt.ts`, `packages/shared/src/schemas/impersonate-prompt-templates.schema.ts` |
| **known** Place and role of the impersonation instruction | Appended as the final `user` message after assembly, before the generation tail messages | No. The engine fixes it | `packages/server/src/routes/generate.routes.ts` (`finalMessages.push({ role: "user", content: impersonateInstruction })`) |
| **known** Which sections an impersonate call keeps | On an impersonate call, every non-marker section is dropped, unless the call resolved a dedicated impersonate preset. Markers and their runtime data stay | Partly. The rule is the engine's. The author escapes it by assigning an impersonate preset | `assembler.ts` (`input.impersonate === true && input.preserveImpersonatePresetSections !== true && section.isMarker !== "true"`), `packages/server/src/routes/generate.routes.ts` (`preserveImpersonatePresetSections`) |
| **known** Impersonate preset and connection | `impersonatePresetId` and `impersonateConnectionId` on the generate request. The impersonate preset is the first candidate, before the request, connection and chat candidates | Yes. The author sets both per call | `packages/server/src/routes/generate/prompt-preset-selection.ts`, `packages/shared/src/schemas/chat.schema.ts` |
| **known** Agents block on impersonate | `impersonateBlockAgents` skips the agents for that call | Yes | `packages/shared/src/schemas/chat.schema.ts` |
| **known** Generation guide | `generationGuide` becomes one extra final `system` message: `Take the following into special consideration for your next message: <text>` | Yes. The author passes the text per call | `packages/server/src/routes/generate/generate-route-utils.ts` (`buildGenerationGuideInstruction`), `generate.routes.ts` |
| **known** An agent's prompt | `agent_configs.promptTemplate`, else the built-in default for that agent type. Templates can also be named options on the agent | Yes. Edited in the Agent Editor | `packages/server/src/db/schema/agents.ts`, `packages/server/src/services/agents/agent-executor.ts` (`getDefaultPromptForAgent`, `renderAgentPromptTemplate`) |
| **known** An agent's connection and parameters | `agent_configs.connectionId`, plus `agent_configs.settings` and the executor's `temperature` and `maxTokens` (defaults 4096, range 128 to 32768). A connection `maxTokensOverride` caps the result | Yes | `packages/server/src/db/schema/agents.ts`, `packages/server/src/services/agents/agent-executor.ts` (`normalizeAgentMaxTokens`, `resolveAgentTemperature`, `applyAgentMaxTokensCaps`) |
| **known** Conversation-mode system prompt | `promptPresets.conversationPrompt`, else the built-in `DEFAULT_CONVERSATION_PROMPT`. A preset with an empty field falls back to the built-in | Yes | `packages/server/src/db/schema/prompts.ts`, `packages/shared/src/constants/conversation-prompt.ts` |
| **known** Game-mode Game Master prompt | `buildGmSystemPrompt` reads the game prompt template from the chat or the setup. The chat then adds `customGmPrompt` at the end. The template is one of two built-in prompts or a custom one | Yes. The author picks or writes the template and the extra text | `packages/server/src/services/game/gm-prompts.ts` (`buildGmSystemPrompt`, `buildGmFormatReminder`), `packages/server/src/services/generation/game-gm-prompt-runtime.ts` (`gameSystemPrompt`, `customGmPrompt`), `packages/shared/src/constants/game-prompt.ts` |
| **known** Game-mode fixed blocks | The role block, the `game` block from genre, setting, tone and difficulty, the rating block for SFW or NSFW, the state blocks, and the ruleset reference are built in code | No. The author sets the inputs: genre, setting, tone, difficulty, rating, ruleset | `packages/server/src/services/game/gm-prompts.ts` |
| **known** Generation parameters | `promptPresets.parameters`: `temperature`, `topP`, `topK`, `minP`, `maxTokens`, `maxContext`, `frequencyPenalty`, `presencePenalty`, `reasoningEffort`, `verbosity`, `serviceTier`, `assistantPrefill`, `assistantReasoningPrefill`, `customThinkingTags`, `customParameters`, `enabledParameters`, `squashSystemMessages`, `showThoughts`, `useMaxContext`, `stopSequences`, `strictRoleFormatting`, `singleUserMessage` | Yes. All are preset fields | `packages/shared/src/schemas/prompt.schema.ts` (`generationParametersSchema`), `packages/shared/src/constants/defaults.ts` (`DEFAULT_GENERATION_PARAMS`) |
| **known** Which of those parameters reach the request | `strictRoleFormatting` merges adjacent same-role messages. `squashSystemMessages` joins the leading system messages. `singleUserMessage` folds the whole prompt into one user message. `showThoughts` and `reasoningEffort` are sent to the provider | Yes, through the preset fields | `assembler.ts` (phases 3, 4, 8), `packages/server/src/routes/generate/dry-run-route.ts` (parameter send) |
| **known** Connection-level parameters | `api_connections.defaultParameters` for new chats, `maxContext`, `maxTokensOverride`, `maxParallelJobs`, `maxRequestsPerMinute`, `enableCaching` with `cachingAtDepth` | Yes. Set per connection | `packages/server/src/db/schema/connections.ts` |
| **known** Which preset one call uses | The candidate list is impersonate preset, then request preset, then connection preset (roleplay only), then chat preset. The first one that exists wins. A preset that differs from the chat preset starts from that preset's `defaultChoices` | Partly. The author sets all four ids. The precedence order is the engine's | `packages/server/src/routes/generate/prompt-preset-selection.ts` (`buildGenerationPromptPresetCandidates`, `resolveGenerationPromptPresetChoices`) |
| **inferred** Provider quirks | The provider layer removes or rewrites parameters that a model rejects, for example reasoning-off template kwargs, and adapts role names | No. The engine fixes it | `packages/server/src/services/llm/providers/openai.provider.ts` |

---

## Appendix A: SillyTavern default prompt texts, verbatim

From `default/content/presets/openai/Default.json` at the pinned commit, unless a row names a code file.

| Field | Text |
|---|---|
| `prompts[]` identifier `main`, name Main Prompt | `Write {{char}}'s next reply in a fictional chat between {{char}} and {{user}}.` |
| `prompts[]` identifier `nsfw`, name Auxiliary Prompt | *(empty string)* |
| `prompts[]` identifier `jailbreak`, name Post-History Instructions | *(empty string)* |
| `prompts[]` identifier `enhanceDefinitions`, name Enhance Definitions | `If you have more knowledge of {{char}}, add to the character's lore and personality to enhance them but keep the Character Sheet's definitions absolute.` |
| `impersonation_prompt` | `[Write your next reply from the point of view of {{user}}, using the chat history so far as a guideline for the writing style of {{user}}. Don't write as {{char}} or system. Don't describe actions of {{char}}.]` |
| `group_nudge_prompt` | `[Write the next reply only as {{char}}.]` |
| `new_chat_prompt` | `[Start a new Chat]` |
| `new_group_chat_prompt` | `[Start a new group chat. Group members: {{group}}]` |
| `new_example_chat_prompt` | `[Example Chat]` |
| `continue_nudge_prompt` | `[Continue your last message without repeating its original content.]` |
| `continue_postfix` | `" "` (one space; the `SPACE` enum value) |
| `wi_format` | `{0}` |
| `scenario_format` | `{{scenario}}` |
| `personality_format` | `{{personality}}` |
| `send_if_empty` | *(empty string)* |
| `assistant_prefill` | *(empty string)* |
| `assistant_impersonation` | *(empty string)* |
| `use_sysprompt`, `squash_system_messages`, `continue_prefill` | `false`, `false`, `false` |
| `names_behavior` | `0` (`DEFAULT`) |
| `openai_max_context`, `openai_max_tokens` | `4095`, `300` |
| `temperature`, `frequency_penalty`, `presence_penalty`, `top_p`, `top_k`, `top_a`, `min_p`, `repetition_penalty` | `1`, `0`, `0`, `1`, `0`, `0`, `0`, `1` |
| `seed`, `n` | `-1`, `1` |

Code fallbacks in `public/scripts/openai.js` differ in one place. `default_main_prompt` reads `Write {{char}}'s next reply in a fictional chat between {{charIfNotGroup}} and {{user}}.`, and the shipped preset overrides it with `{{char}}`.

Default prompt order from the same file, entry `character_id: 100001`. The live Prompt Manager uses `dummyId: 100001`.

1. `main` — on
2. `worldInfoBefore` — on
3. `personaDescription` — on
4. `charDescription` — on
5. `charPersonality` — on
6. `scenario` — on
7. `enhanceDefinitions` — off
8. `nsfw` — on
9. `worldInfoAfter` — on
10. `dialogueExamples` — on
11. `chatHistory` — on
12. `jailbreak` — on

The same file also carries an entry for `character_id: 100000`, with the same list and no `personaDescription`. That is the default dummy id in `public/scripts/PromptManager.js`; the live setup overrides it to 100001 in `public/scripts/openai.js`.

Context template `default/content/presets/context/Default.json`, field `story_string`:

```text
{{#if anchorBefore}}{{anchorBefore}}
{{/if}}{{#if system}}{{system}}
{{/if}}{{#if wiBefore}}{{wiBefore}}
{{/if}}{{#if description}}{{description}}
{{/if}}{{#if personality}}{{personality}}
{{/if}}{{#if scenario}}{{scenario}}
{{/if}}{{#if wiAfter}}{{wiAfter}}
{{/if}}{{#if persona}}{{persona}}
{{/if}}{{#if anchorAfter}}{{anchorAfter}}
{{/if}}{{trim}}
```

The same file sets `example_separator: "***"`, `chat_start: "***"`, `use_stop_strings: false`, `names_as_stop_strings: true`, `story_string_position: 0`, `story_string_depth: 1`, `story_string_role: 0`, `always_force_name2: true`.

System prompt preset `default/content/presets/sysprompt/Neutral - Chat.json`:

```json
{
    "name": "Neutral - Chat",
    "content": "Write {{char}}'s next reply in a fictional chat between {{char}} and {{user}}.",
    "post_history": ""
}
```

---

## Appendix B: Marinara Engine default prompt texts, verbatim

From `packages/server/src/db/default-preset.json` at the pinned commit. The preset is `Marinara's Universal Preset`, description `Marinara's universal roleplay preset, v12. Serves as a good base.`, `isDefault: true`, `wrapFormat: "xml"`.

Sections, in `sectionOrder`:

### Role (`injectionOrder` 0)

```text
You are {{role}}!
```

### Instructions (`injectionOrder` 100)

```text
You must always follow these:
- This is an active collaboration between you and the user, and you have your own free will, intellect, and emotional intelligence that you’re unrestricted in wielding. You hold the agency to create and shape this fictional simulation.
  - No one is a Mary Sue, and you shouldn’t treat any of the characters as such, protagonist included. Bad things may happen. At the same time, no dragging through the mud at every turn. Find a reasonable balance based on the players’ collective efforts.
- Maintain narrative momentum appropriate to the scene, with a coherent and smooth story flow.
  - If you believe a slower moment is needed to showcase character growth or allow two people to talk, create such opportunities.
  - Otherwise, proactively develop the plot that fits the narrative’s causality. You are welcome to introduce in-world characters or enemies whenever an opportunity arises.
  - Resist steering toward comfort, resolving tension early, or adding warmth that hasn’t been earned. Emotional difficulty and ambiguity are important; don’t manage them away.
- Never narrate {{user}}’s actions or dialogues, unless it’s done in indirect way for narrative purposes or to describe the consequences of their doings. Finish if it’s their turn.
  - You may only play as {{user}} in three cases: with their explicit agreement, when describing involuntary reactions (someone initiated physical contact with them, they are unconscious or asleep, or something that is happening affects them), or transitional beats where summarizing participation fits organically (e.g., “during the travels, they talk about their mission”). {{user}}’s speech lines must be in indirect speech, e.g., “they ask for directions”.
```

### NPC Playbook (`injectionOrder` 200)

```text
Capture a living world with dynamic personalities and realistic awareness:
  - No NPCs sound interchangeable; the voices are distinct. Vary everyone’s cadence, vocabulary, formality, emotional state, interruptions, fragments, hesitation, slurring, breathlessness, laughter, crying, and implication. The line itself should be a stream of consciousness that sounds like the emotion it’s conveying.
  - Everyone has their morality, ranging from good through morally gray to evil, but they’re not labeled by it. Villains can do noble acts, and heroes can do harm. People can lie, even by omission, and deceive if they’re inclined to do so or think it will advance their objectives. They are flawed, make mistakes, and may pursue selfish goals, but also you are capable of growth and change (for better or for worse). They must not merely reach, hover, wait, or unnaturally pause; they must fully grab, touch, and commit.
  - No one is omniscient. Both the NPCs and players should know only what they have personally witnessed, inferred from available evidence, learned from public reputation, or were told by someone in the scene. No knowing another location’s events, hidden motives, secret arcs, private thoughts, or offscreen revelations unless that information plausibly reached them. When unsure, they should be wrong, suspicious, confused, or curious instead.
  - Play as all NPCs, creating scenes of your own and altering how the plot unfolds. However, keep in mind that the player characters are not under your jurisdiction and are played by different people.
```

### Guidelines (`injectionOrder` 300)

```text
{{guidelines}}
```

### Setting — marker `{"type":"lorebook"}` (`injectionOrder` 400)

### Characters — marker `{"type":"character"}` (`injectionOrder` 500)

### Dialogue Examples — marker `{"type":"dialogue_examples"}` (`injectionOrder` 600)

### Persona — marker `{"type":"persona"}` (`injectionOrder` 700)

### Past Events — marker `{"type":"chat_summary"}` (`injectionOrder` 800)

### Chat History — marker `{"type":"chat_history"}` (`injectionOrder` 900)

### Output Format (`injectionOrder` 1000)

```text
Think step by step first. Then, respond in the following output format:
- Write in the {{tense}} tense, {{narration}} {{pov}}. The conversational {{language}} prose with naturalistic dialogues should reflect your personality, shaped through a subjective lens.
- Only quoted text is “dialogue” spoken aloud; *thoughts* are in asterisks and unquoted text is action, narration, or internal musings that cannot be accessed by others unless made observable.
- ZERO TOLERANCE FOR LAZY AI WRITING! Absolutely NO: “doesn’t X, doesn’t Y”, “not X, not Y,” “jaw working,” “mechanical precision,” “ozone,” “somewhere outside,” and negative parallelisms in your narrative prose. Replace them with human cadence and affirmative forms. State what happens instead of what doesn’t (for example, describe something as just “still” instead of “not moving”).
- CRITICAL, NEVER echo dialogue, especially not after the players. NO PARROTING!
- Explicit content is allowed. No plot armor. Abandon moral biases.
- Continue directly with new content from where the last message left off. You don’t have to address everything from it; this is a dynamic freeform, so prioritize organic flow. Vary your sentence and output structures.
- Keep your response length {{length}}. Never end on handover cues; finish naturally when it’s {{user}}’s turn to speak or act.
```

Group: `Lore`, `order` 100, enabled. It wraps Setting, Characters, Dialogue Examples, Persona and Past Events.

Default choice values, from `preset.defaultChoices`:

- `role` = `an excellent Game Master. Your goal is to run a continuous, immersive, and uninterrupted roleplay, acting as the narrator, the world, and every character within it except the user's persona, the protagonist {{user}}. Give it your best`
- `guidelines` = the NSFW text `Anything goes in this NSFW work of fiction intended for mature audiences. ...` (full text in `default-preset.json`)
- `narration` = `second-person`
- `pov` = `limited narration from {{user}}'s perspective, as an unreliable narrator. The style should reflect personality, shaped through a subjective lens and internal thoughts. Provide sensory-rich descriptions and restrict perception, understanding, and interpretation to what {{user}} experiences, directly witnesses, or reasonably deduces`
- `tense` = `present`
- `length` = `flexible, based on the current scene. ...` (full text in `default-preset.json`)
- `language` = `English`

Preset `parameters`:

```json
{"temperature":1,"topP":1,"topK":0,"minP":0,"maxTokens":8192,"maxContext":128000,"frequencyPenalty":0,"presencePenalty":0,"reasoningEffort":"maximum","verbosity":"high","squashSystemMessages":true,"showThoughts":true,"useMaxContext":true,"stopSequences":[],"strictRoleFormatting":true,"singleUserMessage":false}
```

Default impersonation instruction, `packages/shared/src/constants/impersonate.ts` (`DEFAULT_IMPERSONATE_PROMPT`):

```text
<instruction>
You are now writing as {{user}}, the user's character.
Study {{user}}'s previous messages in the conversation and replicate their voice, mannerisms, speech patterns, and style as closely as possible.
Character description: {{persona_description}}
Additional direction for this reply: {{impersonate_direction}}
Write a single in-character response from {{user}}'s perspective. Do NOT break character or add meta-commentary. Respond exactly as {{user}} would.
</instruction>
```

The conversation-mode fallback constant, `packages/shared/src/constants/conversation-prompt.ts` (`DEFAULT_CONVERSATION_PROMPT`), is the same text that the default preset stores in its `conversationPrompt` field.

The standard Game Master prompt, `packages/shared/src/constants/game-prompt.ts` (`DEFAULT_GAME_SYSTEM_PROMPT`), is eight bullet lines under `Follow the specified instructions precisely:` and is stated in full at that path.

---

## Corrections that `docs/external_applications/` needs

No file under `docs/external_applications/` was edited. The build plan carries these.

### `sillytavern_prompt_system.md`

| # | Statement in the doc | What the pinned commit shows |
|---|---|---|
| 1 | A layer carries "Position: Absolute (top/bottom) or Relative (depth in history)" | `injection_position` has two values, `RELATIVE` (0) and `ABSOLUTE` (1, in-chat). "Absolute" means in-chat depth, not the top of the prompt |
| 2 | Post-History Instructions: "Role: System (injected as user)" | The `jailbreak` prompt has the `system` role and its place comes from the prompt order. Neither the role nor the place is forced |
| 3 | Main Prompt: "Position: Absolute (top)" | The `main` prompt is a normal prompt-list entry with `RELATIVE` position |
| 4 | Chat History: "Injected at configurable depth relative to end" | `chatHistory` is a marker whose place comes from the prompt order. The depth mechanism belongs to prompts and extension prompts, not to the history |
| 5 | The layer list has no trigger, no on/off switch and no lock | Each prompt has `injection_trigger`, `enabled`, `injection_order` and `forbid_overrides`. These are the main levers the author uses |
| 6 | No mention of the utility prompts | The preset also holds `wi_format`, `scenario_format`, `personality_format`, `group_nudge_prompt`, `new_chat_prompt`, `new_group_chat_prompt`, `new_example_chat_prompt`, `continue_nudge_prompt`, `continue_postfix`, `send_if_empty`, `assistant_prefill` and `assistant_impersonation` |
| 7 | No mention of the Author's Note or the card's depth prompt | Both are in-chat extension prompts with a settable position, depth, role and interval |
| 8 | World Info: "Trigger: keyword matching", "priority" | The entry has eight positions, a depth and a role for `atDepth`, `order`, `sticky`, `cooldown`, `delay`, `group` and `probability`. A keyword is not the only trigger: `constant` and vectorized entries exist |
| 9 | "Context Templates: SillyTavern uses template variables (Handlebars-style)" | The Handlebars-style template is the text-completion `story_string`. Chat completion uses markers and utilities instead. The doc does not say which API path each template serves |
| 10 | Character Card Format lists only the V1 fields | Cards also carry `system_prompt`, `post_history_instructions`, `character_version`, `creator_notes`, and `extensions.depth_prompt` with `prompt`, `depth` and `role`. These four are prompt levers |
| 11 | "each layer can be configured with … Order: Numeric ordering within same depth/role" | Order and depth apply only to an in-chat prompt. A relative prompt is placed by its index in the prompt list |
| 12 | The doc does not name the file that holds these layers | One OpenAI preset file holds the connection, the model, the samplers, `prompts`, `prompt_order` and the format templates. A preset is what an author exports, imports and shares |

### `marinara_engine.md`

| # | Statement in the doc | What the pinned commit shows |
|---|---|---|
| 1 | "Sections — Ordered prompt fragments with `injectionOrder` (0, 100, 200... 1200)" | `assemblePrompt` walks `preset.sectionOrder`, an array of section ids. `injectionOrder` only orders the database read and never places a section. The default preset's orders run 0 to 1000 in steps of 100 |
| 2 | "Markers — (lorebook, character, persona, chat_history, dialogue_examples, chat_summary)" | 13 marker types exist: the six named, plus `current_scene_summary`, `recalled_scenes`, `recalled_messages`, `id_macro_cards`, `world_info_before`, `world_info_after` and `agent_data` |
| 3 | "Groups — Logical containers (e.g., 'Lore' group holds Setting, Characters, Persona, Past Events markers)" | Correct, and the group can nest through `parentGroupId`, can be disabled, and its name becomes the wrapper tag |
| 4 | "Variables / Choice Blocks — The Default preset has 7 variables" | Correct: `role`, `guidelines`, `narration`, `pov`, `tense`, `length`, `language`. The preset is now named `Marinara's Universal Preset`, and a choice block also carries `displayMode`, `optionSort`, `multiSelect`, `separator` and `randomPick` |
| 5 | "Wrap Format — `xml`, `markdown`, or `none`" | Correct. Add that `skipWrap` turns the wrapper off for one section, and that the tag comes from the section name |
| 6 | "Built-in Agent Ecosystem … 20+ agents … File: `packages/shared/src/constants/agent-prompts.ts`" | That file holds three prompt constants at the pinned commit: the chat-summary prompt, the long-term-memory summary prompt and the narrative-director prompt. The built-in agent list is registered at startup from the capability-package manifests, so a fixed count does not hold |
| 7 | "Each agent has its own system prompt" | Correct. Add that each agent also has its own `connectionId`, `promptTemplate`, `settings`, `temperature` and `maxTokens` |
| 8 | No mention of the Impersonate preset or the impersonate instruction | The impersonation instruction is a final `user` message. Non-marker sections are dropped unless a dedicated impersonate preset is resolved |
| 9 | No mention of the generation guide | `generationGuide` adds one final `system` message |
| 10 | No mention of the preset selection precedence | Impersonate preset, request preset, connection preset (roleplay only), chat preset |
| 11 | "Marinara uses a preset system where users can compose, reorder, and configure prompt sections via UI" | Correct. Add that the same preset also holds `conversationPrompt`, `gamePrompt`, `parameters` and `wrapFormat`, so one preset decides three chat modes |

### `marinara_engine_system_prompt.md`

| # | Statement in the doc | What the pinned commit shows |
|---|---|---|
| 1 | Preset name "Default", description "Marinara's general-purpose roleplay preset" | Name `Marinara's Universal Preset`, description `Marinara's universal roleplay preset, v12. Serves as a good base.`, `systemKey: marinara-universal-preset` |
| 2 | Three static sections: Role at 0, Instructions at 500, Output Format at 1200 | 11 sections in `sectionOrder`: Role (0), Instructions (100), NPC Playbook (200), Guidelines (300), Setting (400), Characters (500), Dialogue Examples (600), Persona (700), Past Events (800), Chat History (900), Output Format (1000) |
| 3 | Marker table: Setting 100, Characters 200, Persona 300, Past Events 400, Dialogue Examples 600, Chat History 700 | Setting 400, Characters 500, Dialogue Examples 600, Persona 700, Past Events 800, Chat History 900 |
| 4 | `{{role}}` default text ends "who is played by the user" | Ends "except the user's persona, the protagonist {{user}}" |
| 5 | Instructions text is a numbered list whose item 5 is `{{guidelines}}` | The text is a different, shorter three-item list. `{{guidelines}}` is now its own section at order 300 |
| 6 | Output Format text | The pinned text is different in every line. It opens `Think step by step first.` and its last line is `Keep your response length {{length}}. Never end on handover cues; finish naturally when it’s {{user}}’s turn to speak or act.` |
| 7 | No NPC Playbook section | `NPC Playbook` is a section at order 200 and carries the distinct-voices, morality and no-omniscience rules |
| 8 | Parameters block | Still correct. `maxTokens` is 8192 and `maxContext` is 128000 in the preset |
| 9 | No mention of what `isDefault` or `wrapFormat` is | `isDefault: true`, `wrapFormat: "xml"`, `author: Marinara` |

---

## Secondary sources

These are official docs. They agree with the source on the levers, and they are not used for any mechanism above.

| Source | What it confirms |
|---|---|
| https://docs.sillytavern.app/usage/prompts/prompt-manager/ | A prompt has a Name, a Role, Triggers (`normal`, `continue`, `impersonate`, `swipe`, `regenerate`, `quiet`), a Position (`Relative` or `In-Chat`), a Depth and an Order. The Utility Prompts are the format templates, Group Nudge, New Chat, New Example Chat, Continue Nudge and Replace Empty Message. Squash system messages and Use system prompt exist |
| https://docs.sillytavern.app/usage/core-concepts/worldinfo/ | The position list matches `world_info_position`: before and after the character, before and after the example messages, top and bottom of the Author's Note, and a depth position where depth 0 is the bottom of the prompt |

No official ME documentation page was read for this asset. Every ME claim comes from source at the pinned commit.

---

## Known gaps

- **No live run.** Neither app was started. A reading of source cannot show what a model does with a given prompt, only what the app sends.
- **ST text-completion path was read only in part.** The chat-completion path is traced end to end. The text-completion path (`getTextGenGenerationData` in `public/scripts/textgen-settings.js`, `createRawPrompt` in `public/script.js`) was read only where the story string and the injections enter it.
- **ME built-in agent count.** The manifests are registered at startup, and this asset does not state a number.
- **ME game mode was read at the prompt-builder level.** The full list of what `buildGmSystemPrompt` emits per game state is longer than the table says.
- **ST presets beyond the shipped Default were not compared.** The numbers in this asset come from `Default.json` and from the code defaults.
