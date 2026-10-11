# Research: how Marinara Engine binds state to swipes, retries and notices

Type: research
Status: open
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
