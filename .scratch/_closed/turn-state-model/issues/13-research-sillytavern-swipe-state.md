# Research: how SillyTavern binds state to swipes, retries and notices

Type: research
Status: resolved
Blocked by: —

## Question

How does SillyTavern (https://github.com/SillyTavern/SillyTavern) store an alternate response (a swipe), which state belongs to each swipe, what a regenerate or a swipe switch rolls back, who may write the chat while a generation runs, and how it shows notices that must not reach the LLM?

## Step 1: digest the prior research first

Read these before you open the repo. Copy what answers the questions below, with the source path, and list what is still open:

- `../../_closed/steering-and-guided-generation/research/03-sillytavern-core.md`: §Retry vs new generation, §Storage, §Storage role, and §Display-only system/comment messages.
- `../../_closed/steering-and-guided-generation/research/02-guided-generations-extension.md`: any retry or swipe facts.
- `../../_closed/narrator-modes-and-options/research/03-option-generation-prior-art.md` §1.
- `../../dashboard-ui-review-phase2/assets/01-reasoning-replies-research.md` §2.

Go to the repo only for the questions that step 1 leaves open.

## Step 2: questions for the repo

1. **Swipe storage.** The shape of a message with swipes (`swipes`, `swipe_id`, `swipe_info`, `extra`). Which fields are per swipe and which are per message?
2. **State per swipe.** Is any state (chat variables, extension state such as trackers or summaries, `chat_metadata`) bound to a swipe or a message? On a swipe switch, which state changes and which does not?
3. **Regenerate and continue.** What a regenerate cuts from the chat, what it restores, and what it keeps. How does a regenerate of an older message differ from one of the last message?
4. **Checkpoints and branches.** How a checkpoint or a branch copies the chat and its state. Is that the only full rollback?
5. **Writers during a generation.** Is there a generation lock or an "is generating" flag? What happens to a user edit, delete or swipe while a generation runs? How does a save from a stale copy get prevented, if at all?
6. **Notices.** Display-only system messages (`is_system`, comment messages, `/comment`): how they are stored, how the prompt builder excludes them, and how they affect "the last message" (the swipe arrows, regenerate, continue). Where do errors show (toasts, console)?
7. **Location or scene display.** Does core or a common extension show the current location or scene, and where is it stored?

## Output

An asset `../assets/research-13-sillytavern-swipe-state.md`: the step-1 digest, then one section per question with file and symbol citations into the repo (a commit SHA, symbol names). End with a short list of the patterns that bear on tickets 01, 02 and 03. Report facts; do not recommend a design.

## Feeds

[Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md), and [Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md).

## Answer

The asset is [`../assets/research-13-sillytavern-swipe-state.md`](../assets/research-13-sillytavern-swipe-state.md). It carries the step-1 digest. It gives code citations into `SillyTavern` at commit `06bde939fb1e9c4c8d8641d810f0a916b5bce127`, `package.json` version `1.19.0`, and labels each fact known, inferred or tested. All seven questions are answered; the residue is at the end of this answer. Facts below are **known** unless marked otherwise.

**1. Swipe storage.** A swipe is not a row. One Message owns `swipes[]`, a parallel `swipe_info[]` and one `swipe_id` (`public/global.d.ts`, `ChatMessage`/`SwipeInfo`). `swipe_info[i].extra` is a deep clone of the message-level `extra`, and `syncMesToSwipe` / `syncSwipeToMes` are the only movers between the two levels; both return a boolean and both can refuse. The message-level `mes`, `send_date`, `gen_started`, `gen_finished` and `extra` are a live mirror of the active swipe. A generation in flight is encoded in the data as an out-of-range index: `swipe_id = swipes.length` before the request, filled by `saveReply`.

**2. State per swipe.** `swipe()` touches the index, the message-level fields that `clearMessageData` and `syncSwipeToMes` rewrite, and the DOM. It never reads or writes `chat_metadata`. So chat variables (`chat_metadata.variables`), chat metadata, and chat-id-keyed stores such as the vectors collection survive a switch. Everything an extension writes into `extra` becomes per swipe through the clone — memory summaries and `display_text` included.

**3. Regenerate and continue.** Only regenerate deletes: it removes the last chat entry in memory and appends a fresh row. Overswipe-regenerate and continue mutate in place. The swipe prompt is the history minus the target (`coreChat.pop()` after the `is_system` filter). Core has no older-message regenerate: `isMessageSwipeable` requires `messageId == chat.length - 1`, and an older message can only be inspected, branched or deleted.

**4. Checkpoints and branches.** Both write a new chat file that holds `chat.slice(0, mesId + 1)` plus a spread copy of `chat_metadata` and a fresh `integrity` uuid. A branch can bind a chosen swipe. A swipe switch is the only in-place rollback, and it rolls back one message only.

**5. Writers during a generation.** The flags are client-side: `isGenerating()` is `is_send_press || is_group_generating`. No server-side generation lock exists. Guards are per call site, not one rule — swipe and message-move are blocked, edit is allowed, delete is blocked in the UI but open through `/del` and `/cut`, and the generated row is mutated in memory during streaming and written at the end. A stale save is prevented by the file-lineage `chat_metadata.integrity` slug, compared server-side by `checkChatIntegrity` in `src/endpoints/chats.js`, and not by a generation check.

**6. Notices.** Three storage classes, and the prompt is the separator. `NARRATOR` rows reach the model as `system`. `COMMENT` and the help rows carry `is_system: true` and are dropped by one filter in `Generate`. `IGNORE_SYMBOL` is a per-row flag that both prompt builders honour while the row keeps its position. Errors never become chat rows: they go to toasts and the console. "The last message" has no shared accessor, so a trailing display-only row changes each rule differently — chevrons attach to it, regenerate deletes it, and continue-on-send is disabled by it.

**7. Location or scene display.** Core stores no location or scene state. `chat_metadata.scenario` is a prompt-text override, not a room name. The nearest chat-level display datum is the background file at `chat_metadata['custom_background']`, and `autoBackgroundCommand` picks it with one quiet LLM call from a closed list, storing only the file. No bundled extension in this version shows a location panel.

Residue, all recorded in the asset's §8: whether `name`, `title`, `is_user` and `is_system` are meant to vary per swipe (the code is silent); whether a trailing non-compact `/comment` is truly swipeable in the shipped UI (read, not run); third-party chat-id-keyed stores, which a checkpoint or branch does not copy; and a concurrent overwrite from a second tab holding the same integrity slug.

One correction to the prior reading, for any session that uses it: `public/scripts/script.js` of the steering asset is now `public/script.js`. The quoted code is unchanged.
