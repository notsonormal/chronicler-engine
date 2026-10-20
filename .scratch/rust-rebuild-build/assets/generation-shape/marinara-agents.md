# Marinara Engine: the Agent system (research notes)

Source: a shallow clone at `tmp/marinara`. All paths below are relative to `tmp/marinara/packages/` unless they start with `docs/`.
Labels: **known** = read in source or docs (path and symbol given). **inferred** = my reading; not stated directly.

Abbreviations used in the tables:
- `types` = `shared/src/types/agent.ts`
- `pipe` = `server/src/services/agents/agent-pipeline.ts`
- `exec` = `server/src/services/agents/agent-executor.ts`
- `gen` = `server/src/routes/generate.routes.ts`
- `editor` = `client/src/components/agents/AgentEditor.tsx`

## 1. Phases

Marinara calls this concept the **phase** (`AgentPhase`). Users see the label **Pipeline Phase**. There are three values. **Known**: `types` `AgentPhase`, `AGENT_PHASE_VALUES`; `docs/agents/agents-overview.md` "The three phases".

| Code name | UI label | When it runs | Can change the prompt | Can change the reply |
|---|---|---|---|---|
| `pre_generation` | Pre-Generation | Before the main generation call. **Known**: `gen` comment "Phase 1: Pre-generation agents"; `pipe` `preGenerate`. | Yes. `context_injection` and `director_event` text becomes injections (`pipe` `runPreGenerationAgents`). `prompt_patch` edits `finalMessages` (`gen` `applyPromptPatchOperations`). **Known.** | No. The reply does not exist yet. |
| `parallel` | Parallel | Starts at the same time as the main call. It does not get `mainResponse`. **Known**: `gen` "Phase 2: Fire parallel agents alongside the main generation"; `pipe` `runParallel`. With `sequentialExecution` (for example a local GPU in Game chats), it runs after the main call. **Known**: `gen` near `pipeline.runParallel()`. | No. **Known**: docs "does not wait for the reply and cannot change it". | No. |
| `post_processing` | Post-Processing | After the main reply is complete **and saved**. The host creates the message (`chats.createMessage`, `savedMsg`) first. Then it calls `pipeline.postGenerate(completedResponse, …)`. **Known**: `gen` "Collect parallel results + Phase 3: Post-processing agents". | Not this turn. Tracker output becomes prompt context on the **next** turn (`injectAsSection`, `COMMITTED_TRACKER_AGENT_TYPES` in `server/src/services/generation/committed-tracker-context.ts`). **Known.** | Yes, but only through `text_rewrite`. The host writes the new text into the saved message with `chats.updateMessageContent`. **Known**: `gen` "Text rewrite/editing agents: run after ALL other agents". |

More facts:
- The pipeline object has the methods `preGenerate`, `runParallel` and `postGenerate` (`pipe` `createAgentPipeline`). **Known.**
- A post-processing Agent can opt in to read the same turn's pre-generation injections (`settings.includePreGenInjections`) and parallel results (`settings.includeParallelResults`). Other Agents get `undefined` (`pipe` `buildAgentContext`). **Known.**
- Two sub-steps run inside post-processing, outside the generic batch:
  - `lorebook-keeper` runs separately after `postGenerate`. **Known**: `gen` `lorebookKeeperAgent`, `executeAgent`.
  - `text_rewrite` Agents are removed from the pipeline and run last. **Known**: `gen` `textRewriteAgents`.
  So post-processing has three ordered sub-steps: the generic batch, then Lorebook Keeper, then the rewrite chain. **Inferred** from the code order.
- **Hold Message Until Rewrite** (`holdForRewrite`) delays the `assistant_message_ready` event to the client until the rewrite finishes. The save is not delayed; the message is already in storage. **Known**: `gen` `sendAssistantMessageReady`, `shouldHoldForTextRewrite`.
- A separate earlier step exists for custom Agents with `character_activity_update`. It is a "routing" pre-generation pipeline (`gen` `routingPipeline.preGenerate()`). It runs before the main pipeline and selects the active chat Characters. **Known.**
- The manifest field `execution` has the values `"pipeline" | "feature" | "host"`. "feature" and "host" Agents do not run in the generic phase pipeline. Feature packages (World Maps, Long-Term Memory, Calls, games) plug into their own surfaces. **Known**: `shared/src/features/agents/agent-manifest.types.ts`, `agent-registry.ts` `isBuiltInAgentHostManaged`; `docs/agents/built-in-agents.md`.

## 2. Built-in Agent types

Marinara has its own grouping: `AgentCategory = "writer" | "tracker" | "misc"` (`types`). The UI adds **Apps** and **Custom Agents** sections (`docs/agents/agents-overview.md`). **Known.**

The id constants are in `types` `BUILT_IN_AGENT_IDS`. The runtime manifests are not in the Engine repo. Fresh installs download them from the Pasta-Devs/Marinara-Agents repo, and `BUILT_IN_AGENT_MANIFESTS` starts empty (`shared/src/features/agents/agent-registry.ts`). For that reason, the phase column comes from `docs/agents/built-in-agents.md` (**known**, docs). The result column comes from `exec` `AGENT_RESULT_TYPE_MAP` (**known**, source).

| Category | Id | Name | Phase | Result type | Purpose |
|---|---|---|---|---|---|
| writer | `prose-guardian` | Prose Guardian | post | `text_rewrite` | Removes banned words and repetition from the reply. |
| writer | `continuity` | Continuity Checker | post | `text_rewrite` | Fixes logic errors in the reply (place, timeline). |
| writer | `card-evolution-auditor` | Card Evolution Auditor | post | `character_card_update` | Proposes edits to a Character card. A user must approve each edit. |
| writer | `director` | Narrative Director | pre | `director_event` (+ `secret_plot`) | Adds a story push on demand. Keeps a hidden Secret Plot arc. |
| writer | `knowledge-retrieval` | Knowledge Retrieval | pre | `context_injection` | Summarises the relevant lorebook and file content into the prompt. |
| writer | `knowledge-router` | Knowledge Router | pre | `context_injection` (default fallback, **inferred**) | Injects the matching lorebook entries word for word. |
| tracker | `world-state` | World State | post | `game_state_update` | Date, time, weather, location, characters present. |
| tracker | `expression` | Expression Engine | post | `sprite_change` | Picks a sprite or expression for each Character. |
| tracker | `quest` | Quest Tracker | post | `quest_update` | Quest objectives, completion and rewards. |
| tracker | `background` | Background | post | `background_change` | Picks an uploaded background image for the scene. |
| tracker | `character-tracker` | Character Tracker | post | `character_tracker_update` | Characters present, with mood, outfit, thoughts and stats. |
| tracker | `beholder` | Beholder | post | `context_injection` | Clothing, held items, wounds and species per Character. |
| tracker | `persona-stats` | Persona Stats | post | `persona_stats_update` | Status bars of the player persona. |
| tracker | `custom-tracker` | Custom Tracker | post | `custom_tracker_update` | Fields that the user defines. |
| tracker | `inventory-tracker` | Inventory Tracker | post | `inventory_tracker_update` | Money, equipped gear and carried items. |
| tracker | (package) | Quartermaster, Relationship Tracker | post | not in the map (**unknown**) | Persona inventory and outfits; the relationship web. |
| tracker | `memory-nag` | Memory Nag | post | `memory_nag` | Selects relevant vault memories for the next reply. |
| misc | `echo-chamber` | Echo Chamber | parallel | `echo_message` | Simulated audience reactions in a widget. |
| misc | `combat` | Combat | parallel | `game_state_update` | Initiative, HP and turn order during an encounter. |
| misc | `illustrator` | Illustrator | post | `image_prompt` | Writes an image or video prompt and calls the media provider. |
| misc | `lorebook-keeper` | Lorebook Keeper | post | `lorebook_update` | Creates and updates lorebook entries from the chat. |
| misc | `html` | Immersive HTML | post | `text_rewrite` | Adds in-world HTML elements to the reply. |
| misc | `spotify` | Music DJ | post | `spotify_control` / `youtube_control` / `local_music_control` | Plays music that fits the mood. |
| misc | `haptic` | Haptic Feedback | post | `haptic_command` | Controls connected devices through Intiface. |
| misc | `cyoa` | CYOA Choices | post | `cyoa_choices` | Adds "What will you do?" choice buttons after the reply. |

Retired ids still listed: `about-me-keeper`, `prompt-reviewer`, `response-orchestrator`, `schedule-planner`, `chat-summary`, `autonomous-messenger`, `youtube`, `secret-plot-driver` (`types` `RETIRED_BUILT_IN_AGENT_IDS`). **Known.**

## 3. Result types

The vocabulary is `AGENT_RESULT_TYPE_VALUES` in `types`. It has **32** entries. **Known.**
- Each result is an `AgentResult` with the fields `{agentId, agentType, type, data, tokensUsed, durationMs, success, error}`. **Known.**
- Each run is also stored in the `agent_runs` table (`agentsStore.saveRun`, column `resultType`). **Known**: `server/src/services/storage/agents.storage.ts`.
- In the "Applied at" column, "post loop" means the `for (const result of sortedResults)` loop in `gen`.

| # | Result type | Producer | Applied at (host) |
|---|---|---|---|
| 1 | `game_state_update` | world-state; combat; custom | A new game-state snapshot row per (message, swipe) with date, time, location and weather (`gameStateStore.create`). With World Maps on, the location goes to `materializeAssistantSpatialState`. Combat output is excluded here (`agentType !== "combat"`). **Known** (post loop). |
| 2 | `text_rewrite` | prose-guardian, continuity, html; custom | Reply text: `chats.updateMessageContent` on the saved message. This runs last. **Known.** |
| 3 | `sprite_change` | expression; custom | The server validates the expressions, then the UI shows the sprite. **Known**: `gen`, `agent-event-dispatcher.ts`. |
| 4 | `echo_message` | echo-chamber | `agent_runs` row plus the UI widget. **Known**: `agents.storage.ts`, `client/src/hooks/use-generate.ts`. |
| 5 | `quest_update` | quest; custom | The game-state snapshot of the message and swipe. **Known** (post loop). |
| 6 | `image_prompt` | illustrator; custom | Starts image generation asynchronously (`pendingIllustration`). **Known.** |
| 7 | `context_injection` | knowledge-retrieval, beholder, knowledge-router; custom (default) | Pre phase: text injected into the prompt. Post phase: a recorded note, or committed tracker context for the next prompt. **Known**: `pipe`; docs `custom-agents.md`. |
| 8 | `continuity_check` | none found | No host handler found. **Inferred**: vocabulary only. |
| 9 | `director_event` | director | Pre phase: prompt injection. **Known**: `pipe`. |
| 10 | `lorebook_update` | lorebook-keeper; custom | Lorebook entry writes. These can wait for user approval (`markLorebookResultForApproval`). **Known.** |
| 11 | `character_card_update` | card-evolution-auditor | UI approval modal only. The host never applies it automatically. **Known**: `types` comment on `CharacterCardFieldUpdate`. |
| 12 | `character_card_create` | custom only | UI approval modal (`AgentWriteApprovalModal.tsx`). **Known.** |
| 13 | `background_change` | background; custom | The server removes invented filenames, then saves the chat background. **Known.** |
| 14 | `character_tracker_update` | character-tracker; custom | Merges `presentCharacters` into the snapshot. **Known.** |
| 15 | `persona_stats_update` | persona-stats; custom | Merges into the snapshot. **Known.** |
| 16 | `custom_tracker_update` | custom-tracker; custom | Merges into the snapshot. **Known.** |
| 17 | `inventory_tracker_update` | inventory-tracker; custom | Merges into the snapshot. **Known.** |
| 18 | `spotify_control` | spotify (Music DJ); custom | Server playback fallback (`applySpotifyAgentPlaybackFallbacks`) plus the client. **Known.** |
| 19 | `youtube_control` | Music DJ with YouTube; custom | Client player. **Known**: `use-generate.ts`. |
| 20 | `local_music_control` | Music DJ with local files; custom | Client player. **Known.** |
| 21 | `haptic_command` | haptic; custom | Server device command. **Known**: `gen`. |
| 22 | `cyoa_choices` | cyoa; custom | Stored as `cyoaChoices` in the message swipe extra (`updateMessageExtraForSwipe`). **Known.** |
| 23 | `secret_plot` | director (secret-plot runtime) | Agent memory and `agent_runs`. A failure aborts the generation. **Known**: `gen` `criticalFailed`. |
| 24 | `game_master_narration` | none found | No handler found. **Inferred**: vocabulary only. |
| 25 | `party_action` | none found | No handler found. **Inferred**: vocabulary only. |
| 26 | `game_map_update` | none found as an Agent result | The same name is used as an SSE event for map sync. **Known**: `gen` `sendSseEvent(... "game_map_update")`. |
| 27 | `game_state_transition` | none found | No handler found. **Inferred**: vocabulary only. |
| 28 | `prompt_patch` | custom only (pre phase) | The main prompt messages, through `applyPromptPatchOperations`. **Known.** |
| 29 | `character_activity_update` | custom only (routing pre phase) | Which chat Characters are active this turn. **Known.** |
| 30 | `frontend_theme_update` | custom only | Temporary CSS in the client. **Known**: `use-generate.ts`. |
| 31 | `about_me_update` | custom (retired about-me-keeper) | Chat About Me text. **Known**: `gen`. |
| 32 | `memory_nag` | memory-nag | A persistent Agent prompt section for the next reply. **Known**: `runtime-agent-sections.ts`. |

Multiplicity and combination:
- **Each Agent produces one result type per run.** `resolveAgentResultType` (`exec`) returns one type. The order of precedence is: the Music DJ setting, then `html`, then `settings.resultType`, then `AGENT_RESULT_TYPE_MAP`, then `"context_injection"`. **Known.**
- Exceptions:
  - `spotify` can produce three types, and the setting chooses one. **Known.**
  - `director` produces `director_event` and also `secret_plot` through a separate runtime (`server/src/services/multiplayer/generation-policy.ts`: `director: ["director_event", "secret_plot"]`). **Known.**
  - A failed result always has `type: "context_injection"` (`pipe` `errorResult`). **Known.**
- **Two Agents can produce the same result type.** Examples: world-state and combat both produce `game_state_update`. A custom Agent can also produce any tracker type.
- How the host combines them in the post loop:
  - The host sorts the results with `RESULT_ORDER = { game_state_update: 0 }`, and every other type gets 1. This puts the world-state snapshot first. Dependent tracker merges then add to that snapshot. **Known**: `gen` comment "Sort so game_state_update … is processed before dependent types". `retry-agents-route.ts` uses the same rule.
  - Inside one rank, the order is the order of the `postResults` array (post results, then parallel results). JS sort is stable. **Inferred.**
  - Each tracker result reads the current snapshot row and merges into it through `resolveTrackerGroupUpdate` and the field locks. So it is a sequential merge, and the later write wins for a field that is not locked. **Inferred.**
- `text_rewrite`: the built-in rewriters are first merged into **one** editor request (`mergePairedBuiltInRewriteAgents`, `server/src/services/generation/prose-guardian-settings.ts`). The remaining rewrite Agents run one after the other, as a chain. Each one gets the text of the previous one (`currentResponseForRewrite`). **Known.**
- Pre-phase injections are collected into a list. All injections go into the prompt. **Known**: `pipe`.
- `character_activity_update`: **first valid wins**. The loop does `break` after the first valid result. **Known**: `gen`.

## 4. Custom Agents

Yes, a user can define a custom Agent. **Known**: `docs/agents/custom-agents.md`; `editor`.
- **Phase**: the user picks one of three **Pipeline Phase** buttons. The value is stored in `AgentConfig.phase`.
- **Result type**: the user picks a **Result Type**. The value is stored in `settings.resultType` and read by `resolveAgentResultType`.
- The editor offers 22 of the 32 types (`CUSTOM_AGENT_RESULT_TYPE_OPTIONS`). **Known.**
- Each result type needs a capability (`CUSTOM_AGENT_RESULT_CAPABILITY`, `CUSTOM_AGENT_CAPABILITY_IDS` in `types`). Examples: `edit_trackers`, `edit_messages`, `edit_main_prompt`. At apply time the host checks `customAgentCanApplyResult` / `customAgentCanEmitResult`. **Known.**
- Some result types force a phase (`editor` `resolveCustomAgentPhase` and the button click handler):
  - `text_rewrite` forces `post_processing`.
  - `prompt_patch` and `character_activity_update` force `pre_generation`.
  - **Known.** The docs name only Text Rewrite and Prompt Patch.
- Other phase-related settings: context sources (`CUSTOM_AGENT_CONTEXT_SOURCE_IDS`), and the opt-ins `includePreGenInjections` and `includeParallelResults` ("Turn Data Access"). **Known.**
- Cadence: `runInterval` (for example Illustrator every 5 messages). Activation keywords and decision questions can skip a run. **Known**: `types` `getDefaultBuiltInAgentSettings`; docs.

## 5. Order, concurrency and failure inside a phase

- **Grouping and batching.** `executePhase` (`pipe`) groups the Agents of a phase by provider instance, model and a context key (`groupByProviderModel`, `agentBatchDataKey`). A group of more than one Agent becomes **one** LLM call (`executeAgentBatch`). The output is split per Agent. **Known.**
- **Rewrite lane.** Post-phase `text_rewrite` Agents get their own lane, `"rewrite"`, so they never share a call with trackers (`getAgentBatchLane`). **Known.**
- **Tool-using Agents** are not batched. Each one runs alone. At most 4 tool Agents run at the same time per group (`AGENT_GROUP_MAX_CONCURRENT_TOOL_CALLS`). **Known.**
- **Concurrency.** Groups run in parallel, at most 8 at a time (`AGENT_PHASE_MAX_CONCURRENT_GROUPS`). Each connection also has a limit (`maxParallelJobs`, from 1 to 16). With `sequentialExecution`, the limit is 1. **Known.**
- **No fixed order of execution inside a phase.** The order is only the order of application: `RESULT_ORDER`, then Lorebook Keeper, then the rewrite chain. **Known**, together with §3.
- **Failure handling:**
  - A rejected group becomes one failed `AgentResult` per Agent (`success: false`). The other groups continue (`settleAgentJobsWithConcurrencyLimit`). **Known.**
  - A batch call that throws marks every Agent in the batch as failed (`exec` `makeError`). **Known.**
  - Invalid JSON gets one repair retry (`invalidJsonAgentError`). **Known.**
  - Post phase: each failed result gets one automatic retry, but not after a timeout (`gen` "Auto-retry failed agents once", `shouldAutomaticallyRetryAgentResult`). **Known.**
  - Pre phase: a failed `secret_plot` is critical and stops the generation. Other failures are logged as warnings, and the generation continues. **Known.**
  - Parallel phase: errors are caught and ignored ("Non-critical — parallel agents may fail independently"). **Known.**
  - The user can retry Agents by hand later (`retry-agents-route.ts`). In "Manual Trackers" mode, tracker Agents run only by hand. **Known.**

## 6. The Agent closest to a "quantifier"

There is no single Agent that matches. Two post-processing tracker Agents cover the job together:
- **Characters present.** `character-tracker` produces `character_tracker_update`. Its `data.presentCharacters` is merged into the game-state snapshot row of (message, swipe). **Known**: `gen` "Character Tracker agent → merge presentCharacters into latest game state".
- **Movement / location.** `world-state` produces `game_state_update` with `location`. The host applies it **first** (`RESULT_ORDER`). With World Maps on, the host passes `location` as `locationGuidance` to `materializeAssistantSpatialState`, which resolves it to a map node. **Known**: `gen`.
- World State also lists the characters present (docs). In code, its snapshot copies `presentCharacters` from the previous snapshot, and Character Tracker supplies the new list. **Known**: `gen` comments near `snapshotChars`.
- With World Maps on, room-level movement can also come from the **main reply**, not from an Agent. The model writes a spatial `move destination_id=…` directive, and the host parses and removes it (`extractAssistantSpatialDirective`, `server/src/services/spatial-context/state-resolution.ts`). **Known.**
- So the closest match to a quantifier is world-state (`game_state_update`, location) together with character-tracker (`character_tracker_update`, characters present). Both are in `post_processing`. Both write to the per-swipe snapshot after the reply is saved. **Inferred.**
- Related: `AgentContext.sceneCheck` is a "request-local scene check shared by tracker calls". The first eligible tracker call claims it. **Known**: `types`. Its use is tied to advanced memory (`gen` "Could not attach scene check to post-processing trackers"). **Inferred.**
