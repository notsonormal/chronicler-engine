# Research 13: how SillyTavern binds state to swipes, retries and notices

Ticket: [Research: how SillyTavern binds state to swipes, retries and notices](../issues/13-research-sillytavern-swipe-state.md).

Upstream repo: `https://github.com/SillyTavern/SillyTavern`, cloned read-only to `tmp/sillytavern-13` at commit **`06bde939fb1e9c4c8d8641d810f0a916b5bce127`**, `package.json` version **`1.19.0`**. Every citation below is a path inside that clone plus a symbol name (a function, constant, field or class); where a line number is given it is the line at that commit. The SHA in this header is the only commit citation in the asset.

Scope: this asset answers the seven repo questions of the ticket. It reports facts only. It names no design and it compares SillyTavern with nothing.

Labels: **known** = read in the code at that commit, or stated in a prior asset cited here. **inferred** = follows from the code, not run. **tested** = pinned by a test that was read, or by a command that was run. Nothing in this asset is newly tested; the clone and the version string are the only commands run.

Prior assets this one builds on and does not repeat: [steering 03 SillyTavern core](../../steering-and-guided-generation/research/03-sillytavern-core.md), [steering 02 GuidedGenerations-Extension](../../steering-and-guided-generation/research/02-guided-generations-extension.md), [options 03 §1](../../narrator-modes-and-options/research/03-option-generation-prior-art.md), [reasoning replies §2](../../dashboard-ui-review-phase2/assets/01-reasoning-replies-research.md).

---

## Step 1: what the prior assets already answer

The table gives, per Step-2 question, what the four prior assets settle, the source path, and what stays open.

| # | Question | Answered by step 1 | Source | Left open |
|---|---|---|---|---|
| 1 | Swipe storage | A swipe creates an alternative on the **last** message only; `regenerate`, `swipe` and `quiet` do not first save a new user message | `03-option-generation-prior-art.md` §1 (quoting `public/script.js:4340` at its older commit; `public/script.js` `Generate` at the cloned commit); `02-guided-generations-extension.md` (`guidedSwipe.js` calls `context.swipe.right()` to "create a new swipe on the last (AI) message") | The shapes of `swipes`, `swipe_id`, `swipe_info`, `extra`; which field is per swipe and which per message |
| 2 | State per swipe | Chat metadata is a real per-chat store that holds prompt-time steering: `/inject` writes `chat_metadata.script_injects` and calls `setExtensionPrompt`, then `saveMetadataDebounced()` | `03-sillytavern-core.md` §Storage | Whether a swipe switch changes chat variables, extension state or `chat_metadata`; whether anything is bound per swipe |
| 3 | Regenerate and continue | An ephemeral `/inject` is flushed by a one-shot listener on `GENERATION_ENDED` / `GENERATION_STOPPED`, so the previous attempt's guide does not survive into a retry; a new generation and a retry share the same inject and differ only in the trigger (`/trigger` vs `context.swipe.right()`) | `03-sillytavern-core.md` §Retry vs new generation; `02-guided-generations-extension.md` §"Retry vs new generation" | What a regenerate cuts from the chat, what it restores, what it keeps; how a regenerate of an older message differs |
| 4 | Checkpoints and branches | Nothing. The four assets do not mention checkpoints, branches or chat copies at all | — | Everything: what is copied, whether that is the only full rollback |
| 5 | Writers during a generation | There is a generation-unlock flag pair and commands wait on it: `/impersonate` runs `waitUntilCondition(() => !is_send_press && !is_group_generating, 10000, 100)` and toasts `Cannot run /impersonate command while the reply is being generated.`; GuidedGenerations wraps its swipe in `finally { /flushinject }` | `03-sillytavern-core.md` §Impersonate; `02-guided-generations-extension.md` §"How transience is enforced" | The flag's definition, what each writer (edit, delete, swipe) does while it is set, and whether a stale copy can overwrite a newer one |
| 6 | Notices | `/comment` creates a real chat row with `is_system: true` and `extra.type === system_message_types.COMMENT`; such rows are filtered out before prompt assembly by `let coreChat = chat.filter(x => !x.is_system || (canUseTools && Array.isArray(x.extra?.tool_invocations)))`; `/sys` (`sendNarratorMessage`) creates a real row with `extra.type === NARRATOR` that reaches the model as a `system` turn | `03-sillytavern-core.md` §Display-only system/comment messages, §Storage role | Their effect on "the last message" rules (swipe arrows, regenerate, continue), and where errors surface |
| 6b | Notices, second group | The empty-content rule is swipe-aware: `const shouldDeleteMessage = type !== 'swipe' && ['', '...'].includes(getMessage) && !reasoning`, so a **swipe** is never deleted for being empty, and reasoning keeps an otherwise empty message alive (`extra.reasoning`) | `01-reasoning-replies-research.md` §2.3 | Whether reasoning, and any other `extra` field, is per swipe or per message |
| 7 | Location or scene display | Nothing. The four assets do not mention location or scene display | — | Everything |

Consequence for the repo effort: questions 4 and 7 are wholly open; questions 1, 2, 3 and 5 are open in their detail; question 6 is open in its "last message" effects and in its error channel. Steps 2.1–2.7 below go to the repo for all of them.

---

## 1. Swipe storage

**Answer: partly open.** The storage shape is fully known. The per-message/per-swipe split is known for the fields `syncMesToSwipe` and `syncSwipeToMes` copy; it is **inferred** for `name`, `title`, `force_avatar`, `original_avatar`, `is_user` and `is_system`, which no sync function touches.

### 1a. The declared shape (known)

`public/global.d.ts` declares the two types:

- `ChatMessage` — `name`, `mes`, `title`, `gen_started`, `gen_finished`, `send_date`, `is_user`, `is_system`, `force_avatar`, `original_avatar`, `swipes?: string[]`, `swipe_info?: SwipeInfo[]`, `swipe_id?: number`, `extra?: ChatMessageExtra`.
- `SwipeInfo` — `send_date`, `gen_started`, `gen_finished`, `extra`, all optional.

So a swipe is **not a row**. A Message row owns an array of alternative texts (`swipes`), a parallel array of per-swipe metadata (`swipe_info`), and one index (`swipe_id`).

### 1b. Per swipe versus per message (known)

| Field | Level | Written by | Evidence |
|---|---|---|---|
| `swipes[i]` (the alternative text) | per swipe | `saveReply` (`type === 'swipe'`), `StreamingProcessor.onProgressStreaming`, `syncSwipeToMes`, `updateMessage` | `public/script.js` `saveReply`: `item.swipes[swipeId] = item.mes`; `onProgressStreaming`: `chat[messageId].swipes[chat[messageId].swipe_id] = processedText`; `updateMessage`: `ensureSwipes(mes); mes.swipes[mes.swipe_id] = text` |
| `swipe_info[i].send_date`, `.gen_started`, `.gen_finished`, `.extra` | per swipe | `saveReply`, `onProgressStreaming`, `syncMesToSwipe` | `public/script.js` `saveReply` / `onProgressStreaming`: `item.swipe_info[swipeId] = { send_date, gen_started, gen_finished, extra: structuredClone(item.extra) }` |
| `swipe_id` | per message (which swipe is active) | `swipe()`, `syncSwipeToMes`, `saveReply`, `deleteSwipe`, `logprobs.js`, `bookmarks.js`, `/addswipe` | `public/script.js` `loadFromSwipeId`: `chat[mesId].swipe_id = newSwipeId` |
| `swipe_info` and `swipes` array length | per message | `saveReply` (`lastMessage.swipes.length++`), `addOneMessage`, `ensureSwipes` | `public/script.js` `ensureSwipes` backfills both arrays |
| `mes`, `send_date`, `gen_started`, `gen_finished`, `extra` | per message, but a **live mirror of the active swipe** | `syncSwipeToMes` (load direction) and `syncMesToSwipe` (save direction) | `public/script.js` `syncSwipeToMes`: `targetMessage.mes = targetMessage.swipes[targetSwipeId]`, then `send_date`, `gen_started`, `gen_finished`, `extra = structuredClone(targetSwipeInfo?.extra) ?? {}`; `syncMesToSwipe` assigns the same four fields in the other direction |
| `name`, `force_avatar`, `original_avatar`, `is_user`, `is_system` | per message | `saveReply`, `sendNarratorMessage`, `sendCommentMessage` | **inferred**: no sync function reads or writes them, and they are absent from `SwipeInfo` |
| `title` | per message | `saveReply`, only on the branch where the new swipe is the last one | `public/script.js` `saveReply`: `if (lastMessage.swipe_id === lastMessage.swipes.length - 1) { lastMessage.title = title; ... } else { lastMessage.mes = getMessage; }` |

Divergence the two arrays can hold, **inferred** from `saveReply` and `ensureSwipes`: `swipes[i]` may be a string while `swipe_info[i]` is a backfilled empty object, and `ensureSwipes` logs `The message had missing or invalid swipe_info for a swipe. It has been backfilled.` when that happens.

### 1c. The two sync functions are the whole read/write interface (known)

- `syncMesToSwipe(messageId)` writes the message-level `mes`, `send_date`, `gen_started`, `gen_finished` and a deep clone of `extra` into `swipe_info[swipe_id]`, and — only when `chat_metadata.tainted || chat.length > 1` — writes `mes` into `swipes[swipe_id]`. Its comment: *"Only sync swipes if the chat is not pristine, so that macros in the greeting can resolve again on swipe"*.
- `syncSwipeToMes(messageId, swipeId, targetMessage)` copies the swipe back out and returns `false` (after `console.warn`) when the swipe data is invalid; its signature accepts a `targetMessage`, which is how `bookmarks.js` rewrites a clone.
- Both return booleans and both bail out when the arrays are missing, so a caller must check the result. `swipe()`'s `loadFromSwipeId` handles `false` by toasting `syncSwipeToMes has returned false. Attempting to swipe back!` and reverting.

### 1d. Swipe generation is stored as an out-of-range index (known)

`swipe()` with `direction === SWIPE_DIRECTION.RIGHT` past the end sets `newSwipeId = chat[mesId].swipes.length`, assigns `chat[mesId].swipe_id = newSwipeId` **before** generating, and only then runs `Generate('swipe')`. The slot is filled later: `saveReply` does `lastMessage.swipes.length++` and then, because `swipe_id === swipes.length - 1`, writes the text and metadata.

`public/scripts/macros.js` `getLastMessageId({ exclude_swipe_in_propress = true, filter = null })` encodes that state as "swipe in progress": `if (exclude_swipe_in_propress && message.swipes && message.swipe_id >= message.swipes.length) { continue; }` (it walks the chat backwards). The same test appears in `public/scripts/macros/definitions/chat-macros.js`.

### 1e. The multi-swipe (`n` parameter) path (known, with one inference)

`public/scripts/openai.js` `createGenerationParameters` builds `canMultiSwipe = settings.n > 1 && !noMultiSwipeTypes.includes(type) && multiswipeSources.includes(settings.chat_completion_source)` with `noMultiSwipeTypes = ['quiet', 'impersonate', 'continue']`, and `sendOpenAIRequest` collects each alternative by index: `const swipeIndex = parsed.choices[0].index - 1; swipes[swipeIndex] = (swipes[swipeIndex] || '') + getStreamingReply(...)`.

Those alternatives are attached in one call in `saveReply`:

```js
const swipeInfoExtra = structuredClone(item.extra ?? {});
delete swipeInfoExtra.token_count;
delete swipeInfoExtra.reasoning;
delete swipeInfoExtra.reasoning_duration;
const swipeInfo = { send_date: item.send_date, gen_started: item.gen_started, gen_finished: item.gen_finished, extra: swipeInfoExtra };
const swipeInfoArray = Array(swipes.length).fill().map(() => structuredClone(swipeInfo));
parseReasoningInSwipes(swipes, swipeInfoArray, item.extra?.reasoning_duration);
item.swipes.push(...swipes);
item.swipe_info.push(...swipeInfoArray);
```

`public/scripts/reasoning.js` `parseReasoningInSwipes` then rewrites each alternative's text and gives each its own `extra.reasoning`, `extra.reasoning_duration` and `extra.reasoning_type`, but only when `power_user.reasoning.auto_parse` is on. **inferred**: the cloned `extra` means the alternatives of one batch start with identical extra except for the three deleted keys.

---

## 2. State per swipe

**Answer: answered for chat variables and for the two bundled state-carrying extensions; partly for anything else.**

### 2a. What does **not** change on a swipe switch (known)

`swipe()` and its inner `loadFromSwipeId` touch exactly four things: `chat[mesId].swipe_id`, the deleted message-level data (`clearMessageData`: `extra.memory`, `extra.display_text`, `extra.media`, `extra.inline_image`, `extra.files`, `extra.fileLength`, `extra.generationType`, `extra.negative`, `extra.title`, `extra.append_title`, plus `gen_started` and `gen_finished`), the message-level fields restored by `syncSwipeToMes`, and the DOM.

Nothing in that path reads or writes `chat_metadata`. So **known**: chat variables, chat metadata and every chat-scoped extension store survive a swipe switch untouched.

- Chat variables live in `chat_metadata.variables` — `public/scripts/variables.js` `getLocalVariable` / `setLocalVariable` (`chat_metadata.variables[name] = value` then `saveMetadataDebounced()`), exposed as `{{getvar}}`, `{{setvar}}` and the `/setvar`-family commands. Global variables live in `extension_settings.variables.global` (`getGlobalVariable` / `setGlobalVariable`), which is not chat-scoped at all.
- The only `chat_metadata` field that a swipe can move **indirectly** is the in-context marker: prompt assembly calls `setInContextMessages(msgInContextCount, type)` which, for `type === 'swipe' | 'regenerate' | 'continue'`, does `msgInContextCount++` and then `chat_metadata.lastInContextMessageId = Math.max(0, chat.length - msgInContextCount)`. The comment says *"No metadata save on purpose, gets hopefully saved via another call"*. **known**.

### 2b. What **is** per swipe: everything a bundled extension writes into `extra` (known)

`extra` is cloned into `swipe_info[i].extra` by `syncMesToSwipe`, so a field that an extension stores in `mes.extra` becomes per swipe as soon as the user swipes away, and the target swipe's copy replaces it.

| Store | Field | Written by | Per swipe? |
|---|---|---|---|
| Memory / summary extension (`public/scripts/extensions/memory/index.js`) | `mes.extra.memory` on a chat message; `setMemoryContext(value, saveToMessage, index = null)` picks `index ?? context.chat.length - 2`, sets `mes.extra.memory = value`, then `saveChatDebounced()`; `getLatestMemoryFromChat(chat)` scans the reversed chat (skipping the last message) for the newest `extra.memory` | `setMemoryContext` | **known**: yes, via `swipe_info[i].extra.memory`, because the field is in `extra` and the prompt channel reads the *active* swipe's `extra` |
| Memory / summary extension | Re-summarisation on chat events: `onChatEvent` is registered for `MESSAGE_DELETED`, `MESSAGE_UPDATED` and `MESSAGE_SWIPED` and calls `summarizeChat(context)`; it deletes the stored summary when the last message's text hash changed (*"Message has been edited / regenerated - delete the saved memory"*) and restores the latest available memory when the chat got shorter | `onChatEvent` | **known** |
| Translate extension (`public/scripts/extensions/translate/index.js`) | `handleIncomingMessage` is registered on `MESSAGE_SWIPED`; it writes `extra.display_text` (declared in `BaseMessageExtra`) | `handleIncomingMessage` | **known** field lives in `extra`; **inferred** per swipe for the same reason |
| Vectors extension (`public/scripts/extensions/vectors/index.js`) | Vector collection keyed by `const chatId = getCurrentChatId()`; `insertVectorItems(chatId, ...)`, `getSavedHashes(chatId)`, `deleteVectorItems(chatId, hashes)`; text is hashed from `context.chat.filter(x => settings.keep_hidden || !x.is_system).map(x => ({ text: String(substituteParams(x.mes)), hash, index }))` | `synchronizeChat`, reached from the `MESSAGE_SWIPED` listener `onChatEvent` | **known**: keyed by chat id, not by swipe. A swipe switch keeps the same chat id, so the collection does not change by itself; the vectorised text is the active swipe's `mes` |
| TTS (`public/scripts/extensions/tts/index.js`), logprobs (`public/scripts/logprobs.js`) | register on `MESSAGE_SWIPED` | `resetTtsPlayback`, `debouncedRender` | **known** the listeners exist; no persisted state found in either |

`getCurrentChatId()` (`public/script.js`) returns `groups.find(...)?.chat_id` for a group, else `characters[this_chid]?.chat` — i.e. the chat file name.

### 2c. Other per-swipe carriers (known)

- Reasoning: `extra.reasoning`, `extra.reasoning_duration` and `extra.reasoning_signature` are set per swipe by `saveReply`; `parseReasoningInSwipes` sets `extra.reasoning`, `extra.reasoning_duration` and `extra.reasoning_type` on each alternative of a multi-swipe batch.
- `title`: set on the message only while the new swipe is the newest (§1b).
- Swipe-picker display: `public/scripts/swipe-picker.js` reads `swipe_info[index].send_date` and `swipe_info[index].extra.token_count` to render each alternative.

### 2d. Messages written directly into message-level `extra` without a sync (inferred)

`bookmarks.js` `createBranch` pushes the branch name into `lastMes.extra.branches` and `createNewBookmark` sets `lastMes.extra.bookmark_link`, then saves the chat — neither calls `syncMesToSwipe`. **inferred**: such a field lands in the message-level `extra` only; the next `swipe()` runs `syncMesToSwipe` before switching, which copies it into the *currently active* swipe's `swipe_info`, and then `syncSwipeToMes` replaces the message-level `extra` with the target swipe's clone. So a message-level `extra` write is visible only while that swipe stays active.

---

## 3. Regenerate and continue

**Answer: answered.** There are three distinct gestures, and only one of them deletes anything.

### 3a. The three gestures and what each cuts (known)

| Gesture | Route | What happens to `chat` | New row? |
|---|---|---|---|
| Regenerate (`#option_regenerate`, `/regenerate` → `Generate('regenerate')`) | `public/script.js` `Generate` | `chat.length = chat.length - 1` — the **last** chat entry is removed from the in-memory array before the request (`deleteItemizedPromptForMessage`, `chat.length = chat.length - 1`, `removeLastMessage`, `MESSAGE_DELETED`) | yes: `saveReply` falls through to the `else` branch and `chat.push(newMessage)` |
| Overswipe right past the last swipe with `OVERSWIPE_BEHAVIOR.REGENERATE` | `swipe()` | nothing is removed. `swipe_id` is set to `swipes.length` (out of range) and the new alternative fills that slot in `saveReply` | no |
| Continue (`#option_continue`, `/continue` → `Generate('continue')`) | `Generate` + `saveReply` | nothing is removed. `saveReply` does `lastMessage.mes += getMessage` and, while streaming, `swipes[swipe_id] = processedText` | no |

The regenerate-vs-swipe distinction is explicit in `Generate`:

```js
if (type !== 'regenerate' && type !== 'swipe' && type !== 'quiet' && !isImpersonate && !dryRun && !depth) {
    is_send_press = true;
    textareaText = String($('#send_textarea').val());
    ...
} else {
    textareaText = '';
    if (chat.length && lastMessage.is_user) {
        //do nothing? why does this check exist?
    } else if (type !== 'quiet' && type !== 'swipe' && !isImpersonate && !dryRun && !depth && chat.length) {
        deleteItemizedPromptForMessage(chat.length - 1);
        chat.length = chat.length - 1;
        await removeLastMessage();
        await eventSource.emit(event_types.MESSAGE_DELETED, chat.length);
    }
}
```

**known**: the delete happens in memory; the file is rewritten from the in-memory array on the next `saveChat*`, so the removed row (all its swipes and `swipe_info`) is not restored by anything.

### 3b. What the prompt for each gesture contains (known)

`public/script.js` `Generate`, just after `getCharacterCardFields()` and the extension-prompt setup:

```js
let coreChat = chat.filter(x => !x.is_system || (canUseTools && Array.isArray(x.extra?.tool_invocations)));
if (type === 'swipe') {
    coreChat.pop();
}
```

- **Swipe**: `is_system` rows are dropped, then the last surviving row — the swipe target — is popped. The model sees the chat *without* the message it is replacing.
- **Regenerate and continue**: nothing is popped. For regenerate the row was already removed from `chat`, so the history ends with the previous assistant message; for continue the message being extended is still in the history.

`getNextMessageId(type)` returns `type == 'swipe' ? chat.length - 1 : chat.length`, and is used only to stamp the itemized-prompt record (`mesId: getNextMessageId(type)`).

### 3c. What a regenerate of an older message does (known)

**There is no such gesture in core.** Three independent blocks:

- `isMessageSwipeable(messageId, message)` requires `(messageId == chat.length - 1)` — the comment reads *"If the message is the last message, and it exists."* `swipe()` calls it, and `refreshSwipeButtons` uses it to decide whether to show the chevrons at all.
- `#option_regenerate` runs `Generate('regenerate')`, which always targets `chat[chat.length - 1]`.
- `/swipe` calls `swipe(null, direction, { source: SWIPE_SOURCE.SLASH_COMMAND, ... })` with no `forceMesId`, so it defaults to the last message; `swipeChatCallback` first runs `waitUntilCondition(() => !is_send_press && !is_group_generating, 10000, 100)`.

The swipe picker is the deliberate exception, and it is read-only for older messages:

```js
// Unlike message swiping, this supports historical AI messages for inspection and branching.
export function canOpenSwipePickerForMessage(messageId) { ... }
// Historical AI messages can open the picker, but only the currently swipeable message may jump.
export function canJumpToSwipeForMessage(messageId) {
    return canOpenSwipePickerForMessage(messageId) && isSwipingAllowed() && isMessageSwipeable(messageId, message);
}
```

An older message's alternatives can therefore be **viewed**, **branched** (`/swipe-picker` → `branchChat(messageId, { swipeId })`) and **deleted** (`deleteSwipe(index, messageId)`, allowed when `canJumpToSwipe || swipeId !== currentSwipeId`), but not jumped to.

Two further cases:

- Regenerate on a **user** message (`public/script.js`, `#option_regenerate`): *"Attempting to regenerate a user message will instead generate a new message."* The code path is the `lastMessage.is_user` branch above, which deletes nothing, so the user row stays and a fresh assistant row is appended.
- Truncation from an older message is a different command: `public/scripts/power-user.js` `doMesCut` (the `/cut` SlashCommand) walks the range, calls `deleteMessage(mesIDToCut, null, false, deleteToolCalls)` for each message and returns the cut text. The messages between stay deleted; nothing is stored for a later restore.

### 3d. Swipe-target selection in group chats (known)

`public/scripts/group-chats.js` `activateSwipe(members, { allowSystem = false })` does not trust `chat[chat.length - 1]`: when the last message `is_user`, is `is_system`, or has `extra.type === system_message_types.NARRATOR`, it walks the chat backwards past all such messages and activates the owner of the last real character message (`message.original_avatar`). `activatePooledOrder` has the same skip logic.

---

## 4. Checkpoints and branches

**Answer: answered.** Checkpoints and branches both copy the chat to a new chat **file**; a swipe switch is the only in-place rollback, and it rolls back only that one message's text and `extra`.

### 4a. Checkpoint (`public/scripts/bookmarks.js`)

`createNewBookmark(mesId, { forceName })` is the block-flag button and `/checkpoint-create`:

1. `const newMetadata = { main_chat: mainChat, integrity: uuidv4() };` — **"Mint a fresh integrity slug so the checkpoint is distinguishable from its parent (#5942)"**. `main_chat` records the parent's session name (`getBookmarkName`, `getMainChatName`, `bookmarkNameToken = 'Checkpoint #'`).
2. `saveChat({ chatName: name, withMetadata: newMetadata, mesId })` — `saveChat` builds `const metadata = { ...chat_metadata, ...(withMetadata || {}) };` and `const trimmedChat = ... chat.slice(0, Number(mesId) + 1)`, i.e. the checkpoint file holds every message **up to and including** `mesId`, with all swipes and `swipe_info` of each, plus a **copy** of the parent's whole `chat_metadata` (so chat variables, `tainted`, chat backgrounds and any other metadata key are copied at that moment and diverge afterwards).
3. `await saveItemizedPrompts(name)` copies the itemized-prompt cache; group chats go through `saveGroupBookmarkChat(groupId, name, metadata, mesId, chatData)` with the same `{ ...chat_metadata, ...metadata }` spread.
4. `lastMes.extra.bookmark_link = name;` then `saveChatConditional()` — the link is written back into the **source** chat's message `extra`.

### 4b. Branch (`public/scripts/bookmarks.js`)

`branchChat(mesId, { swipeId })` → `createBranch(mesId, { swipeId })`:

- `getBranchChatSnapshot(mesId, { swipeId })` → `const snapshot = structuredClone(chat.slice(0, Number(mesId) + 1))`, and when a `swipeId` is given, `syncSwipeToMes(null, swipeId, snapshot[mesId])` rewrites the cloned target message so `mes`, `extra` and `swipe_id` reflect the **chosen** swipe. `createBranch` validates the id (`Number.isInteger(selectedSwipeId) && ... < lastMes.swipes.length`) and toasts `Could not prepare the selected swipe for branching.` when the sync fails.
- `newMetadata = { main_chat: mainChatName, integrity: uuidv4() }` — the same fresh slug rule.
- `saveChat({ chatName: name, withMetadata: newMetadata, mesId, chatData: branchChatSnapshot })`.
- The source message gets `lastMes.extra.branches.push(name)`.
- `branchChat` then `saveItemizedPrompts(fileName)` and **opens the new chat** (`openCharacterChat` / `openGroupChat`); the comment says *"Use Checkpoints and `/checkpoint-create` instead if you do not want to jump to the new chat."*

### 4c. State that does **not** travel with a copy (known/inferred)

- **known**: the copy carries messages and `chat_metadata` only. Per-chat extension stores that are keyed by chat id are therefore empty in the copy — for vectors, the collection id is `getCurrentChatId()` and the id is the file name, so a checkpoint or branch starts with no vectors until re-indexed. (`public/scripts/extensions/vectors/index.js`, `chatId` used as the collection id.)
- **inferred**: in-memory-only state does not travel either. `itemizedPrompts` is copied by `saveItemizedPrompts(chatId)`, which stores the array in IndexedDB keyed by the chat id — so a checkpoint or branch gets a copy only where the code explicitly calls it (both do). The token cache is not per chat at all: `public/scripts/tokenizers.js` `saveTokenCache` writes one global entry, `objectStore.setItem('tokenCache', tokenCache)`.

### 4d. Other rollback primitives (known)

- `deleteLastMessage()`, `deleteMessage(id, swipeDeletionIndex, askConfirmation, deleteToolCalls)`, `deleteSwipe(swipeId, messageId)` (refuses the last remaining swipe: `Can't delete the last swipe.`), and `/cut` (`doMesCut`).
- Chat **backups**: `src/endpoints/chats.js` `trySaveChat` calls `getBackupFunction(handle, cardName)(backupDirectory, cardName, jsonlData)` on every successful save, and `public/scripts/chat-backups.js` `BackupsBrowser` lists (`/api/backups/chat/get`), downloads (`/chat/download`), restores by import (`restoreBackup`) and deletes them. That is a whole-file rollback outside the chat UI.
- Chat import (`public/script.js` `importCharacterChat`, `/api/chats/import`) and `openCharacterChat` select any chat file.

So: **known** that a checkpoint or a branch is the only operation that copies a chat's messages *and* its metadata into a second chat; a swipe switch is the only operation that rolls one message's state back in place.

---

## 5. Writers during a generation

**Answer: answered for the client-side flags, the four writers, and the stale-copy guard. The guard is about the chat file's lineage, not about a generation.**

### 5a. The flags (known)

| Flag | Declared | Set/cleared by | Meaning |
|---|---|---|---|
| `is_send_press` | `public/script.js` `export let is_send_press = false; //Send generation` | set true in `Generate`, `#option_regenerate`, `#option_impersonate`, `#option_continue`, `animateSwipe`; cleared in `unblockGeneration`, `onError`, `onFinishStreaming` paths | a generation is running |
| `is_group_generating` | `public/scripts/group-chats.js` `let is_group_generating = false; // Group generation flag` | group generation wrapper | a group generation is running |
| `isGenerating()` | `public/script.js` `export const isGenerating = () => (is_send_press || is_group_generating)` | — | the single predicate callers use |
| `swipeState` | `public/script.js` `export let swipeState = SWIPE_STATE.NONE;`, values in `public/scripts/constants.js` `SWIPE_STATE = { NONE, SWIPING, EDITING }` | `swipe()` sets `SWIPING` before the animation, `endSwipe` restores `NONE`; **nothing in the repo assigns `EDITING`** — a repo-wide search finds three reads of it (`sendTextareaMessage`, `isMessageSwipeable`, the `#option_continue` handler) and no write | only one concurrent swipe; the `EDITING` branch is unreachable at this commit |
| `isChatSaving` | `public/script.js`, used by `saveChatConditional` | `saveChatConditional` waits `waitUntilCondition(() => !isChatSaving, DEFAULT_SAVE_EDIT_TIMEOUT, 100)` | one save at a time |

There is **no server-side generation lock** found: the generation runs in the browser, and `/api/chats/save` accepts any array.

### 5b. What each writer does while `isGenerating()` is true (known)

| Writer | Behaviour | Evidence |
|---|---|---|
| Swipe (chevrons, keyboard, auto-swipe) | Blocked twice: `swipe()` toasts `Cannot swipe while generating. Stop the request and try again.` / `Swipe aborted` when `isGenerating() && (swipes && !swipesHidden && (swipeState === SWIPE_STATE.NONE))`, then returns silently when `!isSwipingAllowed()` (which contains `!isGenerating()`). `refreshSwipeButtons` also adds `hideAllSwipeButtons` to `<body>` whenever `!isSwipingAllowed()`, and `Generate` calls `hideSwipeButtons()` at its start | `public/script.js` `swipe`, `isSwipingAllowed`, `refreshSwipeButtons` |
| Slash `/swipe`, `/regenerate`, `/impersonate` | Wait instead of failing: each wraps its work in `waitUntilCondition(() => !is_send_press && !is_group_generating, 10000, 100)` and on timeout toasts `Cannot run /<cmd> command while the reply is being generated.` | `public/scripts/slash-commands.js` `swipeChatCallback`, `regenerateChatCallback`, and the inline `callback` of the `name: 'impersonate'` command |
| Send a message | `sendTextareaMessage()` returns early on `swipeState !== SWIPE_STATE.NONE` or `is_send_press`, and calls `hideSwipeButtons()` first with the comment *"Swipe buttons must be hidden now, otherwise concurrent generations are possible."* `Generate` calls `deactivateSendButtons()`. The separate `swipeState == SWIPE_STATE.EDITING` refusal at the top of the same function is unreachable (see §5a) | `public/script.js` |
| Edit a message | **Allowed.** `messageEdit(editMessageId)` has no generation guard: it only refuses nothing, sets `this_edit_mes_id`, hides the buttons and opens a textarea. `messageEditDone` → `updateMessage(div)` → `saveChatConditional()`. `updateMessage` writes the edit into the **active swipe**: `if (mes.swipe_id !== undefined) { ensureSwipes(mes); mes.swipes[mes.swipe_id] = text; }` | `public/script.js` |
| Move a message (edit arrows) | Blocked with a console warning: `The message #${sourceId} was not moved to #${targetId} because a generation is in progress.` | `public/script.js` `messageEditMove` |
| Delete a message, UI path | The delete dialog is not opened: `openMessageDelete` shows it only `if (fromSlashCommand \|\| (!is_send_press) \|\| (selected_group && !is_group_generating))`, else logs `ERR -- could not enter del mode` | `public/script.js` |
| Delete a message, slash path | **Bypasses the guard.** The `/del` command (aliases `delete`, `delmode`) calls `$('#option_delete_mes').trigger('click', { fromSlashCommand: true, ... })`, and `/cut` calls `deleteMessage(...)` per message directly. Neither checks `is_send_press` | `public/scripts/power-user.js` `doDelMode`, `doMesCut`, and the `name: 'del'` / `name: 'cut'` command registrations |
| Write the message being generated | The row is created during streaming: `StreamingProcessor.onStartStreaming` calls `saveReply({ type, getMessage: text, fromStreaming: true })` with the first chunk, then `onProgressStreaming` mutates `chat[messageId]` and `swipes[swipe_id]` on every chunk. The file is written at the end: `onFinishStreaming` → `await saveChatConditional()` | `public/script.js` |
| Abort | `stopGeneration()` calls `streamingProcessor.onStopStreaming()` and `abortController.abort('Clicked stop button')`, then emits `GENERATION_STOPPED` | `public/script.js` |

**inferred** from the delete rows: because a generation writes the whole in-memory `chat` array at the end, a delete performed through a slash command mid-generation is only as safe as the array the generation is holding; nothing in the code re-reads the file before the final save.

### 5c. How a stale save is prevented (known)

Two mechanisms, neither keyed to a generation:

1. **The integrity slug.** `getChat()` mints one per loaded chat when absent: `if (!chat_metadata.integrity) { chat_metadata.integrity = uuidv4(); }`. `saveChat` sends the whole `chat` array plus `chat_metadata` (which includes the slug) to `/api/chats/save`. On the server, `src/endpoints/chats.js` `trySaveChat` reads the first line of the existing file, parses `jsonData?.chat_metadata?.integrity` and compares it: a mismatch throws `IntegrityMismatchError`, the route answers `response.status(400).send({ error: 'integrity' })`, and the client pops `ERROR: Chat integrity check failed while saving the file. ... To confirm an overwrite (and potentially LOSE YOUR DATA), enter OVERWRITE ...` — otherwise `window.location.reload()` and no save. Checkpoints and branches mint a fresh slug, which is what makes parent and copy distinguishable.
2. **The debounced save.** `saveChatDebounced` captures `chid` and `selectedGroup` and aborts the timeout when either changed (`Chat save timeout triggered, but group changed/chid changed. Aborting.`); `swipe()` calls `cancelDebouncedChatSave()` first with the comment *"Cancel pending save to prevent accidental swipe_id overwrites."*; `saveChatConditional` serialises saves through `isChatSaving`. Group saves repeat the same integrity handling in `public/scripts/group-chats.js` `saveGroupChat`.

The first line of a chat file is the header, so the whole file is rewritten on every save (`tryWriteFileSync(filePath, jsonlData)`); there is no per-message or per-swipe merge.

---

## 6. Notices

**Answer: answered for storage and prompt exclusion, and for the last-message effects; answered for the error channel. One part stays open: whether a swipe is offered on a trailing `/comment` row in the shipped UI.**

### 6a. Three different things, three storage shapes (known)

| Kind | `is_system` | `extra.type` | Reaches the model? | Stored where |
|---|---|---|---|---|
| `/sys`, `/nar`, `/sysgen` (`sendNarratorMessage`) | only if the text is nothing but a bias block (`const isSystem = bias && !removeMacros(text).length`) | `system_message_types.NARRATOR` (`'narrator'`) | **yes**, as role `system` | a real `chat` row, `chat.push(message)` / `chat.splice(insertAt, 0, message)`, then `saveChatConditional()` |
| `/comment` (`sendCommentMessage`), system help rows (`sendSystemMessage` → `getSystemMessageByType`) | `true` | `COMMENT`, `GENERIC`, `HELP`, `WELCOME`, `EMPTY`, `SLASH_COMMANDS`, `FORMATTING`, `HOTKEYS`, `MACROS`, `ASSISTANT_NOTE` | **no** | a real `chat` row pushed or spliced, `sendSystemMessage` does `chat.push(newMessage); addOneMessage(newMessage);` |
| `/inject`, author's note (`/note`) | no row at all | — | yes, at assembly time | `chat_metadata.script_injects` + `setExtensionPrompt`, `saveMetadataDebounced()` |

The prompt-side exclusion is one filter in `Generate` (quoted in §3b): `let coreChat = chat.filter(x => !x.is_system || (canUseTools && Array.isArray(x.extra?.tool_invocations)));`. `public/scripts/openai.js` `setOpenAIMessages` then maps what survives to roles:

```js
let role = chat[j].is_user ? 'user' : 'assistant';
...
// If this symbol flag is set, completely ignore the message.
if (chat[j].extra?.[IGNORE_SYMBOL]) { j++; continue; }
// 100% legal way to send a message as system
if (chat[j].extra?.type === system_message_types.NARRATOR) { role = 'system'; }
```

A second, independent mechanism hides a row from every prompt without removing it from the chat: `IGNORE_SYMBOL` (`public/scripts/constants.js` `Symbol.for('ignore')`, declared in `global.d.ts` as `[IGNORE_SYMBOL]?: boolean` with the comment *"Will exclude this message from prompt processing"*). Both prompt builders honour it — `setOpenAIMessages` above and `public/script.js` `formatMessageHistoryItem`, which returns `''` for such a row (*"This can be used to hide messages without affecting the number of messages in the chat."*). Neither honours `is_system`; that filter lives only in `Generate`.

The vectors extension is a third exclusion point with its own rule: `context.chat.filter(x => settings.keep_hidden || !x.is_system)`.

### 6b. Effects on "the last message" (known)

Every "last message" rule is its own `chat[chat.length - 1]` test; there is no shared accessor. The rules that a trailing display-only row changes:

| Rule | Code | Effect of a trailing `is_system` row |
|---|---|---|
| Swipe chevrons and swipe | `isMessageSwipeable` requires `messageId == chat.length - 1` and excludes `is_user`, `extra.isSmallSys`, `extra.swipeable === false` — it does **not** exclude `is_system` | The chevrons attach to the display-only row. `getOverswipeBehavior` returns `OVERSWIPE_BEHAVIOR.LOOP` for it (`else { return OVERSWIPE_BEHAVIOR.LOOP; }` after `else if (!message?.is_user && !message?.is_system) return OVERSWIPE_BEHAVIOR.REGENERATE;`), so a right overswipe loops instead of regenerating the narration. A `isSmallSys` row (compact `/comment`, assistant note, group `EMPTY`) is not swipeable and gets `OVERSWIPE_BEHAVIOR.NONE` |
| `/addswipe` | `public/scripts/slash-commands.js` `addSwipeCallback` | Refuses outright: `if (lastMessage.is_system) { toastr.warning('Can't add swipes to system messages.'); return ''; }` |
| Regenerate | `Generate('regenerate')` deletes `chat[chat.length - 1]` | The display-only row is deleted, not the narration. **inferred**: the history still ends with the narration and `type === 'regenerate'` does not pop from `coreChat`, so the run behaves like a continue and appends a new assistant row after the narration |
| Continue-on-send | `sendTextareaMessage`: `if (power_user.continue_on_send && ... && !lastMessage.is_user && !lastMessage.is_system)` | Continue-on-send is disabled while a display-only row is last |
| Prompt bias scan | `getBiasStrings`: for a swipe it skips the last row (`if (type === 'swipe' && chat.length - 1 === i) continue;`), then scans back to the first `is_user`, `is_system` or `NARRATOR` row and takes `extra.bias` | A display-only row can be the bias carrier; the scan's comment treats `is_system` and `NARRATOR` as carriers |
| Last-in-context highlight | `setInContextMessages`: `chatElement.find('.mes:not([is_system="true"]), .mes.toolCall').eq(-msgInContextCount)` | Display-only rows are skipped when the "last in context" mark is placed; for `swipe`, `regenerate` and `continue` the count is one higher because the target row is not sent |
| Group member activation | `group-chats.js` `activateSwipe(members, { allowSystem })`: `if (lastMessage.is_user \|\| (!allowSystem && lastMessage.is_system) \|\| lastMessage.extra?.type === system_message_types.NARRATOR)`, then a backwards scan with the same three tests; `activatePooledOrder` repeats the `is_system` / `NARRATOR` skip | Both skip backwards past `is_user`, `is_system` and `NARRATOR` rows to find the real speaker, with an `allowSystem` option |

### 6c. Where errors surface (known)

Errors go to toasts and the console, never into the chat:

- `Generate`'s `onError(exception)`: `if (typeof exception?.error?.message === 'string') { toastr.error(exception.error.message, t\`Text generation error\`, { timeOut: 10000, extendedTimeOut: 20000 }); }`, then `unblockGeneration(type)`, `console.log(exception)` and re-throw.
- `public/scripts/openai.js` uses toasts for its own failures, for example `toastr.error(message, t\`API returned an error\`)`, `toastr.error(getChatCompletionErrorMessage(data, response), 'Chat Completion API')`.
- `#option_regenerate` refuses during an edit with `toastr.warning(... 'You cannot regenerate the message you are editing.')`; `swipe()` uses `toastr.warning`, `endSwipe` uses `toastr.error(errorMessage)` when `syncSwipeToMes` fails; `saveChat` uses `toastr.error(t\`Check the server connection and reload the page to prevent data loss.\`, t\`Chat could not be saved\`)` plus `console.error`.
- The only chat-row notices are the deliberate system/help rows of §6a and `/comment`; command output can become one through `public/scripts/slash-commands/SlashCommandReturnHelper.js` (`if (type.startsWith('chat')) sendSystemMessage(system_message_types.GENERIC, htmlOrNotHtml)`).

### 6d. Open

**Open**: whether the shipped UI lets a user swipe or regenerate a trailing non-compact `/comment` row. The code path allows it (`isMessageSwipeable` does not exclude `is_system`), but the observation is from reading; it was not run, and no test in `tests/` was read for it.

---

## 7. Location or scene display

**Answer: answered negatively for core, with one chat-level display datum as the nearest analogue.**

- **known**: core has no "current location" or "current scene" state. `rg -i "location|scene"` over `public/scripts/` returns only URL/location APIs, macro-parser node locations, token-allocation comments, and the stable-diffusion extension's prompt keywords. `ChatMetadata` in `public/global.d.ts` declares only `tainted`, `integrity`, `scenario`, `persona` plus an open index signature.
- **known**: `chat_metadata.scenario` is a per-chat **scenario text override** for the character card, resolved by the `scenario` resolver of `public/script.js` `getCharacterCardFieldsLazy` (`const scenarioText = chat_metadata.scenario || character.scenario || ''`). It is written together with two sibling overrides by `public/script.js` `setCharacterSettingsOverrides`: `chat_metadata.scenario = pendingChanges.scenario; chat_metadata.mes_example = pendingChanges.examples; chat_metadata.system_prompt = pendingChanges.system_prompt; await saveMetadata();`. `public/scripts/group-chats.js` reads the same key (`const scenarioOverride = String(chat_metadata.scenario || '')`). All three are prompt text in chat metadata, not a displayed room name.
- **known**: the nearest per-chat display datum is the **background**, stored in chat metadata under `public/scripts/backgrounds.js` `BG_METADATA_KEY = 'custom_background'` (and a chat-local list under `LIST_METADATA_KEY = 'chat_backgrounds'`), written by `saveBackgroundMetadata(file)` → `chat_metadata[BG_METADATA_KEY] = file; saveMetadataDebounced();`, cleared by `removeBackgroundMetadata`, and read by `isChatBackgroundLocked`. It is per chat, not per message and not per swipe.
- **known**: core can *choose* a background from the user's list with one quiet LLM call — `backgrounds.js` `autoBackgroundCommand` with `const autoBgPrompt = 'Ignore previous instructions and choose a location ONLY from the provided list that is the most suitable for the current scene. Do not output any other text:\n{0}'`, then `generateQuietPrompt({ quietPrompt: prompt })`, then a `Fuse` match against the list; the result is applied by clicking the matching tile, which goes through `saveBackgroundMetadata`. No location string is stored; only the chosen image file name is.
- **known**: the stable-diffusion extension has scene-shaped generation modes (`generationMode.SCENARIO`, `generationMode.BACKGROUND`) whose prompts ask for a location/lighting keyword list. It produces images, and no location state is persisted.
- **known**: no bundled extension in this version displays a location, a room, or a scene panel. `public/scripts/extensions/` contains `assets`, `attachments`, `caption`, `connection-manager`, `expressions`, `gallery`, `memory`, `quick-reply`, `regex`, `stable-diffusion`, `token-counter`, `translate`, `tts`, `vectors` and an empty `third-party/`. A repo-wide, case-insensitive search for `tracker` finds no tracker extension (only `SECURITY.md`, test files and model files). The `Timelines` extension named in a `bookmarks.js` comment (*"Export is used by Timelines extension"*) is third-party and not bundled.
- **inferred**: a story-log room label has no core home. The mechanisms that could carry one are the per-swipe `extra` bundle (`swipe_info[i].extra`), the message-level `extra` (which the sync functions overwrite from the active swipe), and chat metadata (per chat, so a swipe switch would not move it).

---

## 8. Summary of what stays open

| # | Question | Status | What is missing |
|---|---|---|---|
| 1 | Swipe storage | **partly** | Whether the non-synced message fields (`name`, `title`, `force_avatar`, `original_avatar`, `is_user`, `is_system`) are ever intended to vary per swipe; no test or doc was read that states the intent |
| 2 | State per swipe | **answered** for chat variables, memory/summaries and vectors; **inferred** for the other `extra`-carrying extensions | No extension was traced end to end; the conclusions rest on `syncMesToSwipe` / `syncSwipeToMes` and on each extension's storage field |
| 3 | Regenerate and continue | **answered** | — (the "regenerate a trailing system row behaves like a continue" claim is inferred, not run) |
| 4 | Checkpoints and branches | **answered** | Whether third-party extensions keyed by chat id exist and would need copying; not surveyed |
| 5 | Writers during generation | **answered** for the UI paths and the integrity guard | No server-side check of chat mtime or content was found; a concurrent overwrite from a second tab holding the same slug is not addressed by the code read here (**inferred**) |
| 6 | Notices | **answered** except one detail | Whether a trailing non-compact `/comment` is actually swipeable in the shipped UI (§6d) |
| 7 | Location or scene display | **answered** (negatively for core) | Any third-party location panel; not surveyed |

---

## 9. Patterns in this evidence that bear on tickets 01, 02 and 03

Facts only, each with its SillyTavern evidence. No design is proposed.

### Bear on [01 – Model a generation's states, its Snapshot pointers and the game-state writers](../issues/01-model-a-generations-states.md)

1. **A generation's alternates are stored on the Message, not as sibling records.** One row owns `swipes[]`, `swipe_info[]` and `swipe_id`; there is no row or record per generation attempt. (§1a, `public/global.d.ts`, `public/script.js` `saveReply`.)
2. **Each alternate has its own metadata bundle, and the message-level metadata is an explicit mirror of the active one.** `swipe_info[i].extra` is a deep clone of `extra`, and the two sync directions (`syncMesToSwipe`, `syncSwipeToMes`) are the only thing that moves state between them; both return a boolean and both can refuse. (§1b, §1c.)
3. **"A generation is in flight on this message" is encoded in the data as an out-of-range index.** `swipe_id = swipes.length` before the request, filled on completion; macros test `message.swipe_id >= message.swipes.length`. (§1d.)
4. **The rewind point for a swipe is "history without the target message."** `coreChat.pop()` after the `is_system` filter and before assembly. (§3b.)
5. **Only one swipe gesture deletes anything: regenerate removes the last chat entry in memory** (`chat.length = chat.length - 1`) and appends a fresh row; overswipe-regenerate and continue mutate in place. (§3a.)
6. **The target of every swipe/regenerate gesture is the last chat entry**, enforced by `isMessageSwipeable`'s `messageId == chat.length - 1` and by the UI paths; older messages can be inspected, branched and deleted but not regenerated. (§3c.)
7. **There is a client-side "is generating" flag pair and no server-side one.** `is_send_press || is_group_generating` via `isGenerating()`; commands wait on `waitUntilCondition(() => !is_send_press && !is_group_generating, ...)`; the delete slash path bypasses it. (§5a, §5b.)
8. **Writers during a generation are guarded per call site, not by one rule.** Swipe and message-move are blocked, edit is allowed, delete is blocked in the UI and open through slash commands, and the generated row is mutated in memory during streaming and written to the file at the end. (§5b.)
9. **A whole-file write with a lineage slug is the stale-copy guard.** `chat_metadata.integrity` (a uuid minted on load, refreshed by every checkpoint and branch) is compared server-side; a mismatch fails the save with `{ error: 'integrity' }` and the client reloads unless the user types `OVERWRITE`. (§5c, `src/endpoints/chats.js` `checkChatIntegrity`, `trySaveChat`.)

### Bear on [02 – Replace `System` Messages with an error channel outside the Message history](../issues/02-replace-system-messages.md)

10. **SillyTavern separates three storage classes and the separator is the prompt, not the row.** `NARRATOR` is a row that reaches the model as `system`; `COMMENT`/`GENERIC`/help rows are rows that are filtered out by `coreChat`'s `!x.is_system`; `IGNORE_SYMBOL` is a flag on any row that makes both prompt builders skip it while the row keeps its position. (§6a.)
11. **Errors never become chat rows in the code read.** Generation failures are `toastr.error(...)` plus `console.log`; the only chat-row notices are deliberate `/sys`, `/comment`, help rows and command output with `chat` return types. (§6c.)
12. **"The last message" has no shared accessor, so a display-only trailing row changes every rule differently.** Evidence: chevrons and overswipe behaviour (`isMessageSwipeable`, `getOverswipeBehavior`), regenerate's delete of `chat[chat.length - 1]`, continue-on-send, the prompt-bias scan, the last-in-context highlight, group member activation, and `/addswipe`'s explicit refusal. (§6b.)
13. **The prompt filter has one exception that keeps a system row in context**: `chat.filter(x => !x.is_system || (canUseTools && Array.isArray(x.extra?.tool_invocations)))`. (§3b, §6a.)
14. **A user-visible notice can also be stored outside the history entirely.** `/inject` and the author's note live in `chat_metadata.script_injects` and reach the prompt through `setExtensionPrompt`, never through `chat`. (§6a, and steering asset 03 §Storage.)

### Bear on [03 – Decide where the room a generation arrives in lives](../issues/03-decide-where-the-room-lives.md)

15. **Core stores no location or scene state at all**; every `scene`/`location` hit is a prompt keyword, a URL API or a macro-parser offset. (§7.)
16. **The nearest chat-level display datum is the background file**, `chat_metadata['custom_background']` (`BG_METADATA_KEY`, with `LIST_METADATA_KEY` for the chat-local list), written with `saveMetadataDebounced()` and applied by an image tile click — per chat, not per message or swipe. (§7.)
17. **Core can derive a location-shaped choice with one quiet LLM call from a closed list** (`autoBackgroundCommand` with `autoBgPrompt` = *"choose a location ONLY from the provided list that is the most suitable for the current scene"*), and it stores only the chosen file, not the chosen name. (§7.)
18. **A per-swipe display bundle already exists**: `swipe_info[i].extra`, cloned from `extra` and restored into the message-level `extra` by `syncSwipeToMes`. `swipe()` first runs `clearMessageData`, which drops `display_text`, `title`, `media`, `inline_image` and other display keys from the message, and then calls `loadFromSwipeId` → `syncSwipeToMes`. (§1b, §1c; `swipe()` `clearMessageData`.)
19. **A message-level `extra` field written outside the sync is effectively bound to the active swipe** (bookmark links, branch lists), because the next `swipe()` syncs it into the outgoing swipe and then overwrites `extra` from the incoming one. (§2d; `bookmarks.js` `createNewBookmark`, `createBranch`.)
20. **Any per-chat extension state is copied by a checkpoint or branch** (`saveChat` spreads `chat_metadata` into the new header, `saveGroupBookmarkChat` does the same for groups) **but not by a swipe switch** (`swipe()` never touches `chat_metadata`). Chat-id-keyed stores such as the vectors collection do not follow a copy at all. (§2a, §4a, §4b, §4c.)
