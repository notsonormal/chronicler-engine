# Build the scene entries

Type: task (AFK)
Status: open
Blocked by: 16

## Question

Replace the base story memory with the scene entries of [Decide the form of the scene entries](16-scene-entries.md), test first. At each history cut, writer calls add scene entries to a list that never changes, and every call with a history shows the opening and the scene entries before the history.

Done when the full `python build.py` passes and the check below passes.

## Decisions

From the Answer of ticket 16, the Answer of [Decide how the story memory keeps its size](15-story-memory-size.md) and the map Notes "The memory in principle". The coordinator chose the points marked (coordinator, 2026-11-22).

- **The form and the prompt.** The entry form and the writer prompt of [`candidates/scene_entries.json`](../assets/candidates/scene_entries.json). The prompt stays code, as in the base (owner, 2026-11-11). The writer uses the Connection of the Narration slot and `Role::Memory`.
- **What the writer gets.** The Messages that leave, the cast names (the Persona and the Characters of the World, not the Characters present of the Swipe state), and the last scene entry written before. It reads no other memory.
- **The split (coordinator).** The Messages of one cut go to writer calls of at most about 4,000 tokens of Messages each, with the answer cap of 2,000 tokens. A small cut is one call.
- **A failed call.** A cut-off answer or an answer that does not parse is a failed call. It adds nothing, and its Messages go to the next cut. The entries of the calls before it stay.
- **The block.** `<StoryMemory>` as the last part before the history, with `<Opening>` (the first Message of the story, word for word) and `<SceneEntries>` (oldest first, under the head line of the candidate). No block before the first cut.
- **The space (coordinator).** Half the history space of the Connection. The cut arithmetic reserves it. When the list passes its space, the oldest entry leaves the prompt and stays stored.
- **Storage (coordinator; the map fog said so on 2026-11-22).** A list that grows cannot be copied into every Swipe state. The entries get their own storage, tied to the Swipe whose cut wrote them. A Retry, a swipe switch or a delete that removes that Swipe from the story also removes its entries from the prompt. This needs a migration (map Notes, "Standing rules").
- **The base goes.** The rewritten summary, its `SwipeState` key and its prompt are removed. Old saved Games that hold the key keep working: the key is ignored.
- **The player's story does not change.** The trial pack shows the scene entries of each Cycle. No dashboard view: the memory view is a later ticket.

## Steps

1. **Find the seam first.** Read the Answer and the asset of [Find where the base story memory attaches in the engine](13-base-memory-seam.md), and the code of [Build the base story memory](14-build-base-story-memory.md) (`docs/specs/story_memory.md`). Write a short plan under `tmp/ltm17/plan.md`: the table and its tie to the Swipe, the step of the Job, the split, the reserve in the cut arithmetic, and what of the base is removed. If a decision above does not fit the code, stop and report it before you build.
2. Read `tests/STRATEGY.md` ("The one rule") before you add a test. Write the seam cases first (`/tdd`): the split, a failed call, the removal of the oldest entry, and Retry, switch and delete.
3. Build on the branch `long-term-memory-2`.
4. **Glossary.** "Scene entry" enters `CONTEXT.md` when its code lands (`/domain-modeling`).
5. **Docs.** Change what the change makes wrong, `docs/specs/story_memory.md` first (`docs/AGENTS.md`, "Direction of edits").
6. `/simple-after-plan`: the comment pass, the doc review, `/code-review`, the full `python build.py`, and the retro.

## Check

1. **The trial.** One 100-Cycle trial of `scripts/trial_setups/redmist_julian.json` (`history_tokens: 16000`) on the built change. Pass when:
   - writer calls run at each cut, and the pack shows the scene entries from the first cut on;
   - no writer call fails, or the report explains each failure;
   - the report gives the extra calls and tokens of the writer, the size of the list after each cut, and the cached share of the Narrator-role calls beside the reference trial (`tmp/ltm09/redmist_julian/trial-1`, 85.6 %).
2. **The removal.** A 100-Cycle trial does not fill the space. A seam test covers it. The report says whether a trial with a smaller history size is worth running.

## Not in scope

- The memory view and the edit in the dashboard.
- The Character memory and the world facts.
- The judged check trial of the map (the outside check).
