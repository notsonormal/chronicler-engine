# Research: how SillyTavern binds state to swipes, retries and notices

Type: research
Status: open
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
