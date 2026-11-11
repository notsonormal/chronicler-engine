# Research: how Marinara Engine binds state to swipes, retries and notices

Type: research
Status: resolved
Blocked by: —

## Question

How does Marinara Engine (https://github.com/Pasta-Devs/Marinara-Engine) store an alternate response (a swipe), which state belongs to each swipe, what a regenerate or a swipe switch rolls back, who may write the chat while a generation runs, and how it shows notices and failures that must not reach the LLM? Marinara has a game mode, so also: how does it store and show game state such as the current location?

## Step 1: digest the prior research first

Read these before you open the repo. Copy what answers the questions below, with the source path, and list what is still open:

- `../../_closed/steering-and-guided-generation/research/01-marinara-engine.md`: §Retry vs new generation, §Storage role, §UI display.
- `../../_closed/guided-generations-architecture/research/marinara-terminology-cross-reference.md`: regenerate, `generationReplay`, and the "Additional finding" section.
- `../../dashboard-ui-review-phase2/assets/01-reasoning-replies-research.md` §3: failures shown as a diagnostic, not a reply.
- `../../_closed/narrator-modes-and-options/research/03-option-generation-prior-art.md` §1.

Go to the repo only for the questions that step 1 leaves open.

## Step 2: questions for the repo

1. **Swipe storage.** The schema of a message and its swipes (tables, columns, `extra`). Which fields are per swipe and which are per message?
2. **Game state.** In game mode (and any other mode with state), where game state lives (per chat, per message, per swipe, snapshots), which code writes it, and at which step of a generation (before or after any state-extraction call).
3. **Swipe switch.** Which state changes when the user switches swipes? Does the state follow the swipe?
4. **Regenerate.** What a regenerate cuts, restores and keeps, and how a failed regenerate is rolled back.
5. **Writers during a generation.** Is there a per-chat generation lock or job queue? How do edits, deletes, swipe switches and background tasks interact with a running generation? Any stale-state or lost-update guard?
6. **Notices and failures.** Where errors, fallbacks and system notices show (toasts, diagnostics, stored messages), and whether any reach a prompt.
7. **Location or scene display.** How the current location or scene is stored and shown, and whether the chat log records where each scene took place.

## Output

An asset `../assets/research-14-marinara-swipe-state.md`: the step-1 digest, then one section per question with file and symbol citations into the repo (a commit SHA, symbol names). End with a short list of the patterns that bear on tickets 01, 02 and 03. Report facts; do not recommend a design.

## Feeds

[Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md), and [Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md).

## Answer

Asset: [research-14-marinara-swipe-state.md](../assets/research-14-marinara-swipe-state.md). Read at Marinara `main` commit `c56501495a8baf76c258e76534c8728a9b7859bb` (2026-10-07, version 2.5.0), a fresh `--depth 1` clone. No test was executed — the repo's regression scripts need `pnpm install`. Every fact is marked known or inferred; where a repo regression pins a fact it is named inline.

**The headline: Marinara binds state to the pair `(messageId, swipeIndex)`, not to the chat.** Three independent stores chose that anchor — `game_state_snapshots`, `spatial_context_snapshots` (hierarchical map position) and `game_engine_state` (turn games). `game_engine_state`'s own header says why in one clause: *"Mirrors `game_state_snapshots` so regenerate / branch / undo rewind the game correctly."* `game_state_snapshots.rulesetLive` carries the same intent: *"Lives here, not on the card, so it rewinds with swipes."*

The seven answers, in one line each:

1. **Swipe storage.** `messages` holds identity plus a *cache* of the active swipe (`content`, `extra`, `activeSwipeIndex`); `message_swipes` holds `index`, `content`, `extra`. `characterId` and `role` are per message — all swipes of a message belong to the same speaker. `(messageId, index)` has no database constraint; the index is `existing.length` read inside a per-message queue.
2. **Game state.** Written by post-processing agents **after** the reply is saved, anchored to the `targetSwipeIndex` captured at save time, and marked `committed` when the *next* user message arrives. A turn whose world-state agent fails writes no row at all.
3. **Swipe switch.** Message text and extra, chat macro variables (`macroVariableChanges` undo/redo), the per-telling game state, map position and turn-game state all follow the swipe. The chat-global party inventory follows **only when it is still exactly what the outgoing telling left**. `gameJournal`, `gameMap` and `gameNpcs` do not rewind.
4. **Regenerate.** Adds a swipe; deletes nothing. It cuts the target and the tail **from the prompt only** — later messages stay in storage. A failed regenerate adds no swipe, so there is no rollback. One exception: a prose-less command turn saves an empty *hidden anchor* telling.
5. **Writers.** One generation per chat (`409` on a second). Tracker runs outlive the reply on a separate `activeAgentRuns` map and are aborted when their anchor swipe is abandoned. Trash restore and interruption restore are refused while generating; message delete and plain edit are not guarded.
6. **Notices.** Three channels: an SSE frame rendered as a toast (never stored), a field on the message's `extra` (`gameDiceTurn`, rendered as synthetic log segments, never sent to the model), and an ordinary message row. **Marinara's stored system notices do reach the model** — membership and identity notices carry no hidden flag.
7. **Location.** Per-telling in `game_state_snapshots.location` and `spatial_context_snapshots.currentLocationId`; shown by the HUD and the Journal, **never in the transcript**. The move itself is written into the player's own message text as `*moves to <place>*`.

The two patterns most likely to matter to this map's grilling tickets are the **conditional chat-global rewind** (the inventory move, guarded by "still exactly what the outgoing telling left") and the **per-chat write ordinal** (`writeOrdinalCounter` / `writeOrdinal`) that lets a client order a per-anchor row that rewinds against a chat-global metadata cache that never does.

**Open after this ticket:** nothing in the seven questions. The asset's final section lists the 15 facts that bear on tickets 01, 02 and 03.
