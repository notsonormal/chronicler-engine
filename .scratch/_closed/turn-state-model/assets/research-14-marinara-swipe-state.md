# Research: how Marinara Engine binds state to swipes, retries and notices

Asset for wayfinder ticket [Research: how Marinara Engine binds state to swipes, retries and notices](../issues/14-research-marinara-swipe-state.md).

One-line summary: Marinara binds **game state to the pair (message, swipe)**, not to the chat. Every state store that must rewind — game state, hierarchical map position, turn-game engine state — is a row keyed by `messageId` + `swipeIndex`. A regenerate adds a swipe and cuts nothing. A swipe switch restores that pair's row, replays the swipe's chat-variable delta, and moves the chat-global inventory only when it is still exactly what the outgoing swipe left. Failures and notices never enter the prompt: they are SSE frames and toasts, or a field on the message's `extra` that the prompt builder does not read.

## Sources and method

| Item | Value |
|---|---|
| Repo | `https://github.com/Pasta-Devs/Marinara-Engine` |
| Commit | `c56501495a8baf76c258e76534c8728a9b7859bb` (2026-10-07) |
| Version | `2.5.0` (`package.json`) |
| Clone | `git clone --depth 1` to `/tmp/pi-github-repos/Pasta-Devs/Marinara-Engine` |
| Method | Read the source and the repo's own docs. **No test was executed** — the repo's regression scripts need `pnpm install` (no `node_modules` in the clone). |

Epistemic legend, per the map's Notes:

- **[known]** — read directly in the source at this commit.
- **[inferred]** — reasoned from the source, not stated by it.
- **[tested]** — I ran something. **Unused in this asset.** Where the repo's own regression script pins a fact, the script is named inline so a later session can run it.

Citations name the symbol, not a line number (the repo's own docs convention, `docs/AGENTS.md` of chronicler).

## Step 1: digest of the prior research

Read before opening the repo. These facts are settled; the sections below do not repeat them.

| Prior asset | What it already answers | Source paths |
|---|---|---|
| Steering-and-guided-generation research 01 | Guided generation is transient (`generationGuide` + `generationGuideSource`, never a `messages` row); the guide is replayed from `extra.generationReplay` on regenerate; `narrator` is a first-class role mapped to `system` at prompt time; impersonate saves its reply as a `user` message and suppresses the normal preset sections. | `packages/shared/src/schemas/chat.schema.ts`, `packages/server/src/routes/generate/generation-replay.ts`, `packages/server/src/services/conversation/impersonate-prompt.ts`, `packages/server/src/services/prompt/assembler.ts` |
| Guided-generations terminology cross-reference | "Regenerate" is the umbrella term for swipe creation; "retry" is one narrow empty-send behaviour; "Stored guidance" is the UI name for `generationReplay`; `/narrator` became an alias of `/guided`, but the `narrator` **role** survived for engine-generated permanent content. | `docs/chats/messages.md`, `docs/chats/guided-and-impersonate.md`, `packages/server/src/routes/generate/generation-replay.ts` |
| Dashboard-UI-review-phase2 research 01 §3 | Empty content with reasoning present is a **diagnostic failure**, never promoted to a reply. `describeEmptyModelResponse` builds an actionable message from `finish_reason`, token usage and `hadThinking`; the main route sends it as an error and saves nothing. | `packages/server/src/services/generation/empty-response-reason.ts`, `packages/server/src/routes/generate.routes.ts` (empty-response guard) |
| Narrator-modes research 03 §1 | Swipes in the three steering projects are regenerate alternatives, not choice presentation. | — |

**Left open by step 1, answered below:** the swipe schema and its per-swipe vs per-message split (Q1); where game state lives and when it is written (Q2); what a swipe switch changes (Q3); what a regenerate cuts, restores and keeps, and its failure rollback (Q4); the generation lock and the stale-state guards (Q5); the notice channels and whether any reaches a prompt (Q6); how the location is stored and shown (Q7).

---

## Q1. Swipe storage: which fields are per swipe and which are per message

Two tables. `packages/server/src/db/schema/chats.ts`:

**`messages` — per message**

| Column | Note |
|---|---|
| `id`, `chatId` (FK → `chats.id`, cascade), `createdAt` | identity |
| `role` | enum `user` \| `assistant` \| `system` \| `narrator` |
| `characterId` | the speaker. **Per message, not per swipe** — every swipe of a message belongs to the same character |
| `content` | a **denormalized copy of the active swipe's text** |
| `extra` | a **denormalized copy of the active swipe's extra** |
| `activeSwipeIndex` | which swipe is shown |

**`message_swipes` — per swipe**

| Column | Note |
|---|---|
| `id`, `messageId` (FK → `messages.id`, cascade), `createdAt` | identity |
| `index` | 0-based position |
| `content` | this telling's text |
| `extra` | this telling's metadata (`generationReplay`, `thinking`, `generationInfo`, `tokenCount`, `macroVariableChanges`, `diceRollResults`, …) |

Three writers keep the denormalized copy in step, all inside a per-message interruption queue (`withInterruptionQueue`, `chats.storage.ts`):

- `addSwipe(messageId, content, silent?)` — backfills the outgoing swipe's `extra` from the message row, inserts a row at `index = existing.length`, then writes the new `content` and `extra` onto the message and sets `activeSwipeIndex` to it. A `silent` add inserts the row without switching (used for alternate greetings).
- `setActiveSwipe(messageId, index)` — backfills the outgoing swipe's `content` **and** `extra`, then copies the target's `content` and `extra` onto the message. See Q3.
- `removeSwipe(messageId, index)` — refuses the last remaining swipe; when the shown swipe is removed it moves the active index.

Three facts follow.

- **`messages` is a cache of the active swipe, not a second source of truth.** A read of `messages` yields whatever the active swipe held at the last write. [known]
- **(messageId, index) has no database constraint.** `messageSwipes` declares no `uniqueBy`, and the file-backed store only enforces uniqueness where a table declares it (`packages/server/src/db/file-schema.ts`, `uniqueBy`). The index is `existing.length` read inside the per-message queue. [known]
- **A new swipe carries only a fixed allow-list of the message's extra keys.** `freshSwipeMessageExtra` resets `displayText`, `tokenCount` and `generationInfo` to `null`, keeps `isGenerated`, and copies forward `hiddenFromAI`, `hiddenFromAICharacterIds`, `hiddenFromUser`, `isConversationStart`, `conversationStartForCharacterIds`, `reactions`, `personaSnapshot` and `MESSAGE_MARK_EXTRA_KEYS` — and skips `hiddenFromAI`/`hiddenFromUser` when the message is a `commandOnly` anchor. Everything else on the message's extra (`gameDiceTurn`, `diceRollResults`, `cachedPrompt`, `lorebookScan`, `encryptedReasoning`, `generationReplay`) is **not** inherited by the next swipe; the generation path writes it again for the new swipe. [known]

Message lists are decorated with a per-message `swipeCount` (`chats.storage.ts`, `listMessages` / `listMessagesReversed`), not with the swipes themselves.

**Storage layout.** `message_swipes` is the one indirectly-owned table in the file-backed store: it resolves shard ownership through its parent message, so a chat's swipes live with that chat's messages (`docs/development/file-storage.md`). [known]

---

## Q2. Game state: where it lives, who writes it, and at which step

### The shape

`packages/server/src/db/schema/game-state.ts` — `gameStateSnapshots`:

| Column | Note |
|---|---|
| `id`, `chatId`, `createdAt` | identity |
| `messageId`, `swipeIndex` | **the anchor: which telling this state belongs to.** Not a SQL FK; the comment says cascade is handled at application level |
| `date`, `time`, `location`, `weather`, `temperature` | the world clock and place |
| `worldCustomFields`, `presentCharacters`, `recentEvents`, `playerStats`, `personaStats` | JSON columns |
| `manualOverrides`, `fieldLocks`, `hiddenTrackerFields` | user-set values and per-field locks |
| `rulesetLive` | live ruleset sheet state. Its comment states the design intent: *"Lives here, not on the card, so it rewinds with swipes."* |
| `committed` | *"Whether this snapshot has been 'committed' (user sent a follow-up message)."* |

Two sibling stores use the same anchor shape:

- `spatialContextSnapshots` (`schema/spatial-context.ts`) — `messageId` (empty for the pre-message bootstrap state) + `swipeIndex` + `currentLocationId` + `definitionRevision` + `transitionCommandId`. This is the hierarchical-map position.
- `gameEngineState` (`schema/game-engine-state.ts`) — per-(message, swipe) state for the deterministic turn games (UNO, Chess, Poker). Its header comment states the reason: *"Mirrors `game_state_snapshots` so regenerate / branch / undo rewind the game correctly."*

**Fact: the rewind unit is (message, swipe), and it is a deliberate, repeated pattern across three independent state stores.** [known]

### Who writes it, and when

State is written by **post-processing agents after the reply is saved**, not before. In `generate.routes.ts`:

1. The reply is saved (`chats.addSwipe` for a regenerate, `chats.createMessage` otherwise).
2. Once the assistant message is ready, the run moves off the chat-wide slot and onto `activeAgentRuns` — *"Roleplay post-processing is anchored to the saved message/swipe below, so it no longer needs to hold the chat-wide generation slot."* The anchor is `(currentIterationSavedMsg.id, lastSavedSwipeIndex)`. So a new user turn is accepted while the previous turn's trackers still run.
3. `postResults` from the post-processing agents are sorted so `game_state_update` is processed first (`RESULT_ORDER`).
4. `targetSwipeIndex` is captured from the saved message at save time, and the comment states why: *"Determine swipe index for this generation so ALL tracker agents target the same (messageId, swipeIndex) snapshot that the world-state agent creates. Capture the immutable swipe selected when this response was saved. Re-reading the message here can observe a newer user-selected swipe while these agents are still running and attach old results to it."* [known]
5. The world-state agent's result creates the row: `gameStateStore.create({ chatId, messageId, swipeIndex: targetSwipeIndex, ... })`. Character, persona-stats, quest and custom tracker agents then `updateByMessage(messageId, swipeIndex, ...)` into the same row.
6. `gameStateStore.commit(id, chatId)` marks the row committed when the **next** user message arrives. The commit site reads the last assistant message's own `activeSwipeIndex` — *"This ensures swipes/regens always use the state from the user's accepted turn."* [known]

**A turn whose world-state agent fails writes no row at all.** The `create` call sits inside `if (result.success && result.type === "game_state_update" && ...)`. There is no placeholder row. [known]

### The read path, and what happens when the anchor has no row

`getForGeneration(chatId, { preferLatestVisible, visibleAnchor, excludeMessageId, fallbackMessageIds })` (`game-state.storage.ts`):

1. If `preferLatestVisible` and `visibleAnchor` is set, return the exact `(messageId, swipeIndex)` row when it exists.
2. Otherwise return the latest **committed** row (scoped by `fallbackMessageIds` or by excluding a message).
3. Otherwise the latest row of any kind.

**So a missing pointer degrades to a time-ordered fallback, not to an error.** [known]

The anchor itself is resolved by two pure helpers in `generate-route-utils.ts`:

- `resolveVisibleGameStateAnchor(messages)` — walk backwards for the first assistant message (or a `system` message marked `gameStateAnchor: "checkpoint_restore"`) and take its `activeSwipeIndex`.
- `resolveRegenerationGameStateAnchor(messages, regenerateMessageId)` — the same, but over the slice **before** the regenerated message. A regenerate is grounded in the state of the turn *before* the one it replaces. [known]

`shouldPreferLatestVisibleGameState` prefers the visible anchor for an impersonate, a regenerate, or an empty user message. `GET /api/chats/:id/game-state` always passes `preferLatestVisible: true` with the visible anchor, so the HUD shows the state of the telling that is on screen. [known]

### The base state for a new swipe of an existing message

For a regenerate, `trackerBaseGameStateSnapshot` is the **previous swipe's own row**: `gameStateStore.getByChatAndMessage(chatId, messageId, targetSwipeIndex - 1)` when `regenerateMessageId` is set and `targetSwipeIndex > 0`. A new telling starts from what the telling before it ended with. [known]

Related: `allowLatestGameStateFallback = !input.regenerateMessageId` — the fallback to the chat's latest row is **disabled** for a regenerate, so a regenerate cannot silently read a later turn's state. [known]

---

## Q3. Swipe switch: which state changes when the user switches swipes

`PUT /api/chats/:chatId/messages/:messageId/active-swipe` (`chats.routes.ts`) runs inside `withChatSwipeSelectionQueue(chatId, ...)` — *"One swipe selection per chat at a time: which telling is shown and the inventory that follows it are read and changed together. Taken before every other lock … and by nothing else, so it cannot close a cycle."* It calls `storage.setActiveSwipe`.

### What follows the swipe

| State | Follows? | Mechanism |
|---|---|---|
| Message text and extra | yes | `setActiveSwipe` copies the target swipe's `content` and `extra` onto the message row |
| Chat variables (macros) | yes | `setActiveSwipe` returns `{ undo: outgoing.extra.macroVariableChanges, redo: target.extra.macroVariableChanges }`; the route calls `replayVariableChanges`. Comment: *"The chat variables follow the swipe that is shown."* |
| Game state (tracker row) | yes | the next read resolves the row for the newly shown `(messageId, swipeIndex)`; the client also refetches (`refreshVisibleGameState`) |
| Hierarchical map position | yes | `spatialContextSnapshots` is keyed by the same pair |
| Turn-game engine state | yes | `gameEngineState` is keyed by the same pair |
| Party inventory (chat metadata) | **conditionally** | `switchGameInventoryTelling(db, chatId, messageId, from, to)` |
| `gameJournal` (chat metadata) | no | append-only discovery log; not keyed by message or swipe |
| `gameMap` / `gameNpcs` (chat metadata) | no | chat-global discovery state |
| `chat.metadata` summary and similar | no | chat-global |

### The conditional inventory move, in detail

`switchGameInventoryTelling` (`services/game/game-inventory.service.ts`) applies `gameInventoryForTelling(turn, stacks, messageId, from, to)` — *"the stacks become what `to` left, **when they are still exactly what `from` left** … Each telling's own game-state row already carries its detailed inventory, so only the stacks move."*

This is the one place a chat-global store is rewound, and it is guarded twice:

- It only moves when the chat's current stacks still equal the outgoing telling's stacks. Otherwise the stacks are left as the player has them, deliberately.
- If the move fails, the route puts the previous swipe back — *"unless the player has picked another one since, which is theirs to keep."* [known]

Pinned by `scripts/regressions/game-inventory-turn.regression.ts` (not executed here).

### The two-store reconciliation ordinal

`chats.writeOrdinalCounter` plus a per-row `writeOrdinal` solve a problem this map cares about directly. The comment in `chats.storage.ts` states it: a game surface keeps its save in **two** stores — the per-anchor `game_engine_state` row (the authority, *"rewinds with the story"*) and a chat-metadata boot cache (*"chat-global, never rewinds"*). Nothing let a client tell *"metadata is ahead because the last session degraded to metadata-only writes"* from *"the row is behind because the player swiped back"*, so a degraded session's play was lost. One per-chat monotonic counter is now drawn from by both paths, allocated inside the per-chat metadata patch queue, taking the max of the counter and the chat's own mirror so it stays monotonic across a branch, a session carry or a restore. [known]

---

## Q4. Regenerate: what it cuts, restores and keeps

### What it is

Regenerate **adds a swipe to the existing message**. Nothing is deleted.

- Client: `handleRegenerate` calls `generate({ regenerateMessageId, ... })` with the comment *"Regenerate as a new swipe on the existing message."* It is available on any assistant message, not only the last (`ChatMessage.tsx`, `canCreateNextSwipe`). On a touch device it asks for confirmation first.
- Server: `chats.addSwipe(input.regenerateMessageId, fullResponse)`; `savedSwipeIndex = createdSwipe.index`. The previous telling keeps its `content` and `extra` in `message_swipes`.

### What it cuts

- **From the prompt only.** The target message is removed from the history (`chatMessages = chatMessages.filter(m => m.id !== input.regenerateMessageId)`) and the roleplay timeline is sliced to the messages *before* the target.
- **Messages after the target stay in storage.** They are excluded from this generation's prompt, but they are not deleted. A regenerate of an older message therefore leaves a stored tail that was produced by the telling being replaced. [known]
- `resolveRegenerationGameStateFallbackMessageIds` bounds the state fallback to assistant messages before the target, and `getForGeneration` receives `excludeMessageId: regenerateMessageId`.

### What it restores

- The transient direction and impersonate flags: `normalizeGenerationReplay(parseExtra(regenCandidate.extra).generationReplay)` then `applyGenerationReplayToRegenerateInput(input, replay)` — *"For a new generation, the client supplies `generationGuide` directly … For a regeneration (swipe), the previous assistant message stores a `generationReplay` blob in its `extra`."*
- The character target: `input.forCharacterId = regenCandidate.characterId` when the request did not name one (`shouldRestoreRegenerationCharacterTarget`).
- The user-message case (impersonate): `regenerateUserSourceMessage` is rebuilt from the message's own content and attachments, so a user swipe regenerates the same input.

### What it keeps

- Every earlier swipe, with its own `extra` (`thinking`, `generationInfo`, `diceRollResults`, `cachedPrompt`).
- The committed state of every turn before the target.

### Failure and rollback

There is no rollback because there is no partial write.

- **Empty response:** the guard builds `describeEmptyModelResponse(...)`, logs, sends an SSE `error`, and `return null` — **before** `chats.addSwipe`. No swipe is added; the previous telling stays active.
- **Abort / disconnect:** the streaming loop `return null` on `generationSignal.aborted`, again before saving.
- **One exception:** when the model emitted enabled commands or GM verbs but no visible prose, `shouldSaveHiddenGenerationAnchor` lets the turn save a **hidden anchor** — `chats.addSwipe(input.regenerateMessageId, "")` (an empty telling) with `hiddenFromUser`, `hiddenFromAI`, `commandOnly: true` and the turn's `encryptedReasoning`, so the turn's state writes have a message to attach to. [known]
- **A message hidden from AI cannot be regenerated** (`canUseMessageForUserRegeneration`, which rejects a `user` message carrying `hiddenFromAI`).

### Post-processing for the new swipe

The tracker agents run against `(messageId, newSwipeIndex)` with the base state being the **previous swipe's** row (Q2), so the new telling's state is derived from the telling it replaces, not from the chat's latest state.

### A separate route re-runs post-processing alone

`POST /api/generate/retry-agents` (`routes/generate/retry-agents-route.ts`) takes `agentTypes`, and optionally `forMessageId`: *"When set, scope history and game state to this assistant message (as at original generation), not the latest turn."* This is the affordance for re-running extraction without regenerating prose. [known]

---

## Q5. Writers during a generation

### The locks and flags

| Guard | Scope | Behaviour |
|---|---|---|
| `activeGenerations: Map<chatId, ActiveGeneration>` | one generation per chat | a second generate request gets `409 "A generation is already in progress for this chat"`. The record is registered **immediately** after the check — the comment notes that delaying it *"left a small double-submit window where two requests for the same chat could both pass the guard."* |
| `activeAgentRuns: Map<chatId, Set<ActiveGeneration>>` | post-processing agents outliving the reply | `moveToActiveAgentRuns` moves the record off `activeGenerations` and onto `activeAgentRuns`; `/status` reports active while either map holds the chat |
| Abandoned-swipe abort | per agent run | when a user message is saved in roleplay mode, a run whose anchor message's `activeSwipeIndex` no longer equals the run's `swipeIndex` is aborted: *"Cancelling work for abandoned swipe %s:%d before committing the next turn"* |
| `withChatSwipeSelectionQueue(chatId)` | swipe switch + inventory | taken before every other lock, by nothing else, so it cannot close a cycle |
| `withInterruptionQueue([messageId])` | add/remove/set-active swipe, interruption restore | serializes writes to one message |
| `withChatMetadataPatchQueue(chatId)` | metadata | fresh read inside the queue |
| `withMessageExtraPatchQueue(messageId)` | message extra | fresh read inside the queue |
| `RoleplayInterruptionConflictError` | stale expected content | raised when *"The selected response swipe changed."*, *"The preceding message or its selected swipe changed."*, *"The response swipe no longer exists."* |
| `chats.writeOrdinalCounter` / `writeOrdinal` | the two-store order | see Q3 |

### How each writer interacts with a running generation

- **A second generate request** — refused with 409.
- **Swipe switch** — **not** refused by a running generation. It is serialized per chat, and the running agent runs for the abandoned swipe are aborted by the anchor check. [known]
- **Trash restore and interruption restore** — refused with 409 while `activeGenerations` holds the chat (*"Wait for the current generation to finish before restoring messages."*, *"Wait for the current generation to finish before restoring this message."*).
- **Message delete and bulk delete** — **no generation guard**. They go through `removeMessage`/`removeMessages`, and the swipe rows cascade.
- **Plain message edit** (`PATCH /:chatId/messages/:messageId`) — **no generation guard**. [known]
- **Stop** — `POST /api/generate/abort` takes `chatId` and an optional `agentsOnly`, and aborts the generation's `abortController` plus every `activeAgentRuns` entry (or only their `agentAbortController`s when `agentsOnly` is true). Its comment records a race it deliberately avoids: *"Keep the entry registered until the generation route reaches its identity-checked finally block. Deleting here opens a same-chat race where a replacement request can register before the aborted request has unwound."* `GET /api/generate/status/:chatId` reports `{ active, translating }`, where `active` covers both maps. Deleting a chat also aborts its generation (`chats.routes.ts`).
- **A state write by a stale agent run** — prevented by the anchor check plus `targetSwipeIndex` captured at save time, so a run can never attach its result to a newer selected swipe. [inferred from the two mechanisms together; the code comments state each half]

### One lower-level hazard the repo pins

`scripts/regressions/tx-gate-window.regression.ts` pins a storage-layer race: a plain write that passed the transaction gate while `activeTransactionCount` was still 0 could apply after the transaction's first-mutation snapshot, and a rollback then erased a write its caller had already seen succeed. The gate now parks inside the real wait. Relevant here only as evidence that Marinara treats lost updates as a live class of defect. (Not executed.)

---

## Q6. Notices and failures: where they show, and whether any reaches a prompt

### Three channels

| Channel | Stored? | Reaches the prompt? | Examples |
|---|---|---|---|
| **SSE frame → client toast** | no | no | `error`, `agent_error`, `agent_warning`, `roleplay_command_error`, `spotify_command_error`, `selfie_error`, `game_outcome_narration_failed`, `lorebook_image_notice`, `generation_discarded`, `spatial_transition_rejected`, `agents_retry_failed` |
| **A field on the message's `extra`** | yes, on the message | no | `extra.gameDiceTurn` |
| **An ordinary message row** | yes | **yes** | membership and identity notices (`"X has joined the chat."`, `"You are now playing as Y."`), `narrator` recaps and scene summaries |

The client's toast surface is `sonner` (`toast.error` / `toast.warning`), and the generate hook's `showError` makes the error persistent (15 s) and copyable (`use-generate.ts`). `selfie_error` and `spotify_command_error` are emitted by the conversation command runtimes rather than by `generate.routes.ts`; the client consumes them the same way. The empty-response diagnostic from the prior research asset is the canonical failure path: a precise, actionable message, never a reply.

### The stored notice, and how it stays out of the prompt

`gameDiceTurn` is the clearest example of the second channel. When a turn's dice could not all be rolled:

- The server sends an SSE `game_dice_turn_notice` frame. Its comment: *"a clean turn records nothing, and a turn where something could not be rolled says so in plain words instead of leaving the player to guess. The saved flag rides on the message extra; the frame is for the live session log."*
- The server writes `extraUpdate.gameDiceTurn = mergeGameDiceTurnNotices(previousExtra.gameDiceTurn, gameDiceTurnNotice)` — merged, not overwritten, because a `/continue` writes into the same swipe through a shallow merge.
- The client renders it as synthetic narration segments of `type: "system"` (`formatGameDiceTurnNoticeSegments`), one line per unreadable placeholder, branch failure, failed pass, pool overflow and pool mismatch.
- The prompt mapping in `generate.routes.ts` reads `content` and a fixed set of extra keys; `gameDiceTurn` is not among them, and the text is not in `content`. **The notice is therefore visible to the player and invisible to the model.** [known]

### The hidden anchor: a message that exists only to hold a turn's writes

`shouldSaveHiddenGenerationAnchor` saves an anchor when a turn produced enabled commands or GM verbs but no prose. The anchor carries `hiddenFromUser: !sceneRequest`, `hiddenFromAI: !conversationCommandContent`, `commandOnly: true`, `isGenerated: true` and `encryptedReasoning`. Its stated purpose: *"The anchor exists so this can run: a verb-only turn's writes land here, against the anchor's own message, in the same order the saved-message path uses them."* A later turn reads `encryptedReasoning` off the last anchor for continuity. [known]

### The `hiddenFromAI` flag is user-set, and it is per message

`isMessageHiddenFromAI(message)` reads `extra.hiddenFromAI === true`; `isMessageHiddenFromAIForCharacter` also reads `hiddenFromAICharacterIds`. Two writers set it, and both propagate it to **every swipe row** — *"so it survives setActiveSwipe() overwrites"*:

- the single-message extra route, when the partial carries `hiddenFromAI`;
- `bulkSetHiddenFromAI(chatId, messageIds, hidden)` in `chats.storage.ts`, which reports the ids it actually flipped and, on a part-way failure, undoes exactly those ids — *"a caller never records ownership of a half-applied batch"*. The store avoids `db.transaction()` here, so the compensating undo is the atomicity mechanism.

No engine path other than the command-only anchor sets `hiddenFromAI: true`. [known]

### Ordinary system messages DO reach the model

This is the sharpest contrast with the "display-only" pattern:

- Membership and identity notices are created with no hidden flag (`chats.routes.ts`: `role: "system"`, `extra: { conversationMembershipEvent: "joined" }`).
- At prompt mapping, `role` passes through unchanged (`m.role === "narrator" ? "system" : m.role`) with `contextKind: "history"`.
- In conversation mode `resolveConversationMembershipHistoryEvent` gives such a row `author = "System"`, and it is rendered into the history as a normal turn.
- The post-history role policy (`appendNonLeadingSystemMessagesToLastUser`) keeps history-context system messages **in place**, converting them to `user`, while prompt-context system blocks merge into the latest user turn. The header comment states the policy: *"History-context system messages are already positioned in the transcript, so they become user messages in place instead of moving to the latest user turn."* [known]

So Marinara's answer to "should an engine notice reach the LLM?" is: it depends on the notice. A failure is never a message. A membership event is a message and does reach the model. A dice notice is a field on a message and does not.

---

## Q7. Location and scene display

### Where the location lives

Three stores, all anchored to the telling:

- `gameStateSnapshots.location` (+ `date`, `time`, `weather`, `temperature`) — the tracker's plain-text place.
- `spatialContextSnapshots.currentLocationId` (+ `definitionRevision`, `transitionCommandId`) — the hierarchical-map position, also keyed by `(messageId, swipeIndex)`.
- `chats.metadata.gameMap` / `gameNpcs` / `gameInventory` / `gameJournal` — chat-global.

### How it is shown

- The **HUD / map panel** shows the current location, the game day and a time-of-day icon (`docs/game/map-time-weather.md`, `GameSurface.tsx`).
- The **Journal** panel has a "Map" tab listing discovered locations (`GameJournal.tsx`, `LocationsView`), fed by `chat.metadata.gameJournal.locations`.
- **The chat log does not show where each scene took place.** There is no location header on a message; `ChatMessage.tsx` has no location field. The per-telling `location` column is state, not transcript decoration. [known]

### How a move is recorded

The move is written into the **player's own message text**, by the client:

```
packages/client/src/components/game/GameInput.tsx
  body = body ? `*moves to ${pendingMoveLabel}*\n${body}` : `*moves to ${pendingMoveLabel}*`;
```

`docs/game/map-time-weather.md` documents the same behaviour for the player: *"Marinara adds a short line like `*moves to <place>*` to the front of your message."* The server side commits the transition (`commitSpatialOwnerTurn`, `spatial_transition_committed` / `spatial_transition_rejected` SSE frames). [known]

### The journal is append-only, and does not rewind

`reconcileJournal(journal, meta, activeQuests, currentLocation)` (`game.routes.ts`) adds discovered map locations, the current location (*"The party is at X."*), tracked NPCs and quests to `chat.metadata.gameJournal` through `updateJournal` (`services/generation/game-journal-runtime.ts`), whose catch states *"Non-critical; generation should not fail because journal auto-fill failed."* The journal is stored in chat metadata, so it is chat-global and does **not** rewind with a swipe or a regenerate. The client also adds a `location` journal entry whenever `gameSnapshot.location` changes (`GameSurface.tsx`). Session recaps are built from the journal (`buildStructuredRecap`). [known]

---

## Patterns that bear on tickets 01, 02 and 03

Facts only. No design is recommended here.

**For [Model a generation's states, its Snapshot pointers and the game-state writers](../issues/01-model-a-generations-states.md):**

1. **The anchor is a pair, and the pair is the whole pattern.** `(messageId, swipeIndex)` is the key in `game_state_snapshots`, `spatial_context_snapshots` and `game_engine_state` alike. Three independent stores chose the same anchor, and `game_engine_state`'s header says why in one clause: *"so regenerate / branch / undo rewind the game correctly."*
2. **A new telling's base state is the previous telling's row**, not the chat's latest row: `siblingSwipeSnapshot = getByChatAndMessage(chatId, messageId, targetSwipeIndex - 1)`, and `allowLatestGameStateFallback` is false for a regenerate.
3. **A missing pointer degrades to a time-ordered fallback.** `getForGeneration` tries the exact pair, then the latest committed, then the latest of any kind. A turn whose world-state agent failed has no row at all.
4. **Committed is a distinct bit from exists.** `committed` is set when the *next* user message arrives, reading the last assistant message's active swipe — the accepted-turn marker.
5. **A message row is a cache of the active swipe**, written by three functions in one per-message queue, with a fixed allow-list of inherited extra keys.
6. **One turn's state writes can attach to an empty telling.** The hidden anchor (`commandOnly: true`, empty content, `encryptedReasoning`) exists so a verb-only turn has a message to write against.
7. **A chat-global store that must follow the telling needs a guard and an ordinal.** The inventory moves only when it is still exactly what the outgoing telling left, and `writeOrdinalCounter`/`writeOrdinal` let a client order the per-anchor row against the chat-global metadata copy.

**For [Replace `System` Messages with an error channel outside the Message history](../issues/02-replace-system-messages.md):**

8. **Failures are never messages.** The empty-response diagnostic, agent errors and game narration failures are SSE frames rendered as toasts; nothing is saved.
9. **A notice that must persist in the session log but stay out of the prompt is a field on the message's `extra`,** rendered by the client as synthetic log segments — `extra.gameDiceTurn` is the worked example, with a merge rule for continuations.
10. **Marinara's stored system notices do reach the model.** Membership and identity rows carry no hidden flag, are mapped to `contextKind: "history"`, and are kept in place as `user` turns. The engine does not treat a stored notice as display-only.
11. **`hiddenFromAI` is a user-set per-message flag, synced to every swipe row** so a swipe switch cannot resurrect a hidden message. The only engine-set use is the command-only anchor.

**For [Decide where the room a generation arrives in lives](../issues/03-decide-where-the-room-lives.md):**

12. **Marinara puts the move in the player's own input text** (`*moves to <place>*`) and keeps the authoritative position in a per-telling `spatialContextSnapshots.currentLocationId`.
13. **The transcript carries no location.** No message header records where a scene took place; the location is shown by the HUD and the Journal, both reading the current snapshot.
14. **The discovered-place log is chat-global and append-only** (`metadata.gameJournal`), separate from the per-telling position, and it does not rewind.
15. **`definitionRevision` rides with the position**, so a snapshot records which map revision it was taken against.
