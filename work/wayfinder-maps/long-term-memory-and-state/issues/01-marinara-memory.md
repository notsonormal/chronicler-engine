# Find how Marinara Engine keeps memory with its agents

Type: research
Status: resolved
Blocked by: —

## Question

How do the memory agents of Marinara Engine (ME) work, each one alone and all of them together? Which parts of the design have evidence that they work, and which parts only exist in the code?

## Context

- The owner knows that ME keeps memory with several agents. The owner wants the whole system read before this map chooses a design (map Notes, "The mechanism is open").
- Two research tickets of Prompt flexibility already read ME at the same commit, as part of a wide survey. Start from their ME rows, check them, and go deeper. Do not repeat what they got right.
  - [Find how ST, ME and other tools keep long-term memory](../../prompt-flexibility/issues/13-long-term-memory.md) ([asset](../../prompt-flexibility/assets/long-term-memory.md))
  - [Find how ST, ME and other tools keep long-term state](../../prompt-flexibility/issues/14-long-term-state.md) ([asset](../../prompt-flexibility/assets/long-term-state.md))
- The pinned commit is `457e827a3d3291be978121f8c50c98c5dd9d644d`. The clone was at `/tmp/research/marinara` on 2026-11-02. If it is gone, clone it again outside the repo.
- Start points, from the two assets:
  - `packages/server/src/services/advanced-memory*.ts`, `memory-recall*.ts` and `chat-summary/`;
  - `generation/advanced-memory-context.ts` and `generation/long-term-memory-runtime.ts`;
  - `services/agents/agent-pipeline.ts`, `prompt/marker-expander.ts` and `prompt/advanced-memory-prompt.ts`;
  - `packages/shared/src/types/advanced-memory.ts` and `packages/shared/src/constants/agent-prompts.ts`;
  - `db/default-preset.json`;
  - the Lorebook Keeper, the Memory Nag, the Card Evolution Auditor and the tracker agents.
- Earlier notes in this repo: `docs/external_applications/marinara_engine.md` and `work/_closed/rust-rebuild-build/assets/generation-shape/marinara-agents.md`.

Points to cover:

1. **The agents.** List every ME agent or service that writes, changes or reads memory. For each one: what it stores, its prompt (quote the key rules and give the path), its trigger and how often it runs, its input, its model call, its cost for each Message, and its default (on or off).
2. **How they work together.** The order in the pipeline, which agent reads the output of which, which agents share one call, and what happens when two of them disagree. Draw it as one Mermaid flowchart.
3. **Story memory and Character memory.** How ME separates the memory of the story from the memory of each Character: the audience list, the recollections, `knowledgeStarts` and the leak check. Does a Character memory change while that Character is absent? The owner's working view says that it does (map Notes).
4. **Time.** How ME records the time of the story: the World State `date` and `time` fields, who writes them, and whether a memory record carries a time.
5. **Card changes.** How the Card Evolution Auditor finds a durable change, what it ignores, and how a change reaches the next prompt. This map writes card changes with no approval (map Notes), so say what the approval of ME protects against.
6. **Swipe, edit and delete.** For each agent, what a Retry, a swipe switch, an edit of an older Message and a delete do to its output.
7. **Failure and the cap.** What each agent does when its call fails, and what ME does when the memory text passes its cap.
8. **Evidence.** What shows how well the system works: tests, issues, changelog entries and user reports. Say "I don't know" where no evidence exists.

## Output

An asset `assets/marinara-memory.md` with the agents table, the flowchart and one section for each point. Cite each fact with `path:line` at the pinned commit, and label it known, inferred or guessed. Add facts for these tickets:

- [Choose the memory candidates to try](05-memory-candidates.md)
- [Decide how the story keeps its time](06-story-clock.md)
- [Decide what a card change of one Game holds and who writes it](07-card-change.md)

**The check.** List each claim about ME in the two Prompt flexibility assets that this read found wrong.

This ticket finds facts. It does not choose a design.

## Answer

Asset: [How Marinara Engine keeps memory with its agents](../assets/marinara-memory.md). Source: ME at `457e827a`, plus the Marinara-Agents repo at `f9a05f76` (the prompts of the downloadable Agents live there; the map does not pin it).

- **known** ME has eight separate memory writers: Advanced Memory (one service), Chat Summary, Memory Recall, Long-Term Memory (LTM), Lorebook Keeper, Memory Nag, Card Evolution Auditor and the Director Secret Plot. Each is off by default in a Roleplay chat. None of the post-processing Agents reads the output of another in the same reply. Trackers on one Connection share one batched call. The Lorebook Keeper runs alone.
- **known** Advanced Memory is the closest match to our faults. A model finds scene ends (a check every 5 Messages). One call per closed scene writes the recap, the audience (the Characters present) and, if on, a 50–100 word first-person recollection for each Character present. All of this runs after the reply is sent. The next reply reads only saved records. It works in Roleplay chats only, and the per-Character rules work only in group chats where Characters reply one by one.
- **known** On the owner's view: Advanced Memory writes nothing for a Character that was absent. The prompt says "Never assign an absent character just because the scene is about them." The Relationship Tracker ("Do not gate tracking from scene presence") and LTM do update by name without a presence check. Memory Nag probably does (inferred). ME has no Persona memory: the Persona gets no recollection.
- **known** The leak check is a four-word copy test. It runs when a recollection or a combined summary is saved, not when a prompt is built. It misses a retold fact, and the maintainers say so.
- **known** ME records no story clock for memory. The World State Agent writes free-text `date` and `time` for each Swipe, with a prompt rule ("push time forward by a minute or two" per beat). A memory record gets its story time from a regex over the Message text or from a hand edit, shows "unknown (use message order)" when none exists, and never reads the World State value. I found no arithmetic on time (inferred).
- **known** The Card Evolution Auditor runs every 8 Messages and always needs a person's approval. An approved edit rewrites the shared Character row that every chat reads. The version row has no chat, Message or Swipe key. So the approval guards a shared card, which this map's per-Game change does not have. ME's own chat-local precedents are the About Me override (no approval), the Lorebook Keeper pin (`{messageId, swipeIndex}`, restore on delete) and the tracker rows for each Swipe.
- **known** Swipe, edit and delete get four different answers. The Keeper and the trackers follow the Swipe. The Advanced Memory prompt keeps one snapshot for each Swipe, but a finished recap survives a text edit and a Swipe, and can go stale. Chat Summary entries and Memory Nag memories follow nothing.
- **known** When memory passes its cap, Advanced Memory resets the live history to the start of the latest scene, combines the constant summaries in the background (over 70% of the budget), drops optional recall first, and throws an error rather than cut instructions. A failed writer keeps the previous state. A failed Director Secret Plot stops the generation.
- **known** The evidence is plumbing only: 21 Advanced Memory regression files (17,191 lines) with a scripted model, a changelog of fixed faults (scenes that never ended, a Helper that paid again on each try), and a few issues. **I don't know** of any measure of how well a memory finds the right fact, or of its token cost.
- **inferred** The Auditor sees 5 of every 8 Messages (default context 5, interval 8). The Lorebook Keeper has the same gap.
- **known** The check found 7 claims wrong or incomplete in the memory asset and 5 in the state asset. The main ones: the Lorebook Keeper runs every 8 Messages and alone; Memory Recall is injected into the prompt; trackers share one call; and the Card Evolution Auditor does not keep a Game apart from the authored card.
- **inferred** None of this shows that the ME design works well on long play. The Decision model fix (a 0.8 cut-off that never found a real scene change) shows that its scene detection needed real chats to get right.

**Coordinator check (2026-11-03).** Known: `character_card_versions` (`packages/server/src/db/schema/characters.ts:20`) keys a version by `character_id` only, with no chat, Message or Swipe key. Known: a `persona_card_versions` table exists at line 102 of the same file. Which code writes it, and whether an Agent can change a Persona card, is not read. [Decide what a card change of one Game holds and who writes it](07-card-change.md) needs this.
