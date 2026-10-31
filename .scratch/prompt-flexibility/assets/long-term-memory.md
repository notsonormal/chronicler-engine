# How ST, ME and other tools keep long-term memory

Ticket: [Find how ST, ME and other tools keep long-term memory](../issues/13-long-term-memory.md)

Scope: the memory of events and the memories that belong to one Character. Long-term state is a second ticket, [Find how ST, ME and other tools keep long-term state](../issues/14-long-term-state.md). This asset names state only where a method stores memory and state in the same place.

Every claim is labelled **known** (read in the code, read in a stored request, or read in a provider document), **inferred** (reasoned from a read, without a direct measurement) or **guessed**.

Primary sources:

| Source | Pinned or read |
|---|---|
| `chronicler-engine` `src/` at the current tree | this session |
| SillyTavern at `06bde939fb1e9c4c8d8641d810f0a916b5bce127`, in `/tmp/research/st` | 2026-10-31 |
| Marinara Engine at `457e827a3d3291be978121f8c50c98c5dd9d644d`, in `/tmp/research/marinara` | 2026-10-31 |
| Stored calls of Game 6, `tmp/stage6/debug/chronicler_3060.db`, read-only | 2026-10-31 |
| Provider documents and papers (the last section lists them, with the read date) | 2026-10-31 |

# 1. What our engine loses today

## 1.1 The two cuts

**known** The Narrator role cuts the history twice. Both cuts are fixed in code, and neither has an author handle.

| Cut | Rule | Fires when | Code |
|---|---|---|---|
| 1. History cap | keep the last 16000 estimated tokens of the history text | the history text is longer than 64000 characters | `src/application/prompting/token_budget.rs:6`, `src/application/prompting/assembler.rs:327` |
| 2. Context fit | drop whole history lines from the front until the prompt fits the connection's context | the user half is longer than the connection's context minus the reply space | `src/application/prompting/utils/context.rs:8-46`, `:50-101` |

**known** The engine estimates one token for each four characters (`estimate_tokens`, `token_budget.rs:28-30`). Cut 1 therefore keeps exactly the last 64000 characters of the joined history. It cuts in the middle of a line (`truncate_to_budget`, `token_budget.rs:32-45`). Cut 2 drops whole lines only, and it writes the line `(History truncated to fit context window)` when it drops everything (`context.rs:71`, `:95`).

**known** Every history line carries a label. A Narration line starts with `Narrator: `, an Input line starts with the Persona name, and a System line starts with `System: ` (`assembler.rs:302-332`). A label costs characters in both cuts.

## 1.2 The stored calls of Game 6

**known** `tmp/stage6/debug/chronicler_3060.db` holds Game 6 (`Redmist Estate_2026-10-22_1`) with 50 Messages and 19 Narrator-role `llm_messages` rows. Each row stores the full user half of the request.

**known** The 50 Messages of Game 6 are a 20-Message story that the database repeats. Messages 81-100, 101-120 and 121-130 hold the same 20 texts, and the last 10 Messages end the third copy. The repetition keeps the size of one cycle unchanged, so the growth per cycle stays usable.

**known** The measured history block of the Narrator calls. A history entry is one line that starts with `Narrator: ` or with the Persona name.

| `llm_messages.id` | Time | History entries | History characters | Estimated tokens | Truncation marker |
|---|---|---|---|---|---|
| 129 | 2026-10-22 19:03 | 1 | 541 | 135 | no |
| 139 | 2026-10-22 19:04 | 6 | 5465 | 1366 | no |
| 151 | 2026-10-22 19:05 | 10 | 10021 | 2505 | no |
| 157 | 2026-10-22 19:06 | 12 | 12174 | 3043 | no |
| 160 | 2026-10-22 19:06 | 13 | 12848 | 3212 | no |
| 161 | 2026-10-25 23:21 | 13 | 12848 | 3212 | no |
| 164 | 2026-10-25 23:22 | 15 | 13910 | 3477 | no |
| 167 | 2026-10-25 23:22 | 17 | 14678 | 3669 | no |
| 170 | 2026-10-26 17:58 | 19 | 15537 | 3884 | no |
| 172 | 2026-10-26 21:11 | 50 | 42891 | 10722 | no |

**known** Every block from row 129 to row 170 is an exact prefix of the story, and none of them drops an entry. Row 161 repeats the block of row 160, because it is a second call on the same prompt. Row 172 holds all 50 Messages, and its block matches the joined story text to the last character.

**known** Rows 125 and 128 hold a 9-entry prefix of Game 5, an earlier Game of the same World. They are not part of this comparison.

**known** No Narrator call of the eight small Games under `tmp/db-backup-20261031/` reaches a cut either. Those Games hold 1 to 13 Messages, and their largest history block is 4597 characters.

## 1.3 The measured result

**known** Neither cut fires in the only long stored story. The history of Game 6 reaches 10722 estimated tokens at 50 Messages, which is 67 per cent of the 16000-token cap. Cut 2 does not fire either, because the largest user half (47447 characters, `llm_messages.id` 172) is below the 118648-character budget that an OpenRouter connection with the default 32768-token context leaves.

| Quantity | Value | Label |
|---|---|---|
| One 20-Message block, history characters | 16435 | known |
| One block, estimated tokens | 4108 | known |
| One block, cycles | 10 | known |
| Growth per cycle | 1643 characters, 411 estimated tokens | known |
| Cycle whose history first passes the 64000-character cap | 39 | inferred |
| Characters dropped at that cycle | 800 | inferred |

**Inferred, from the measured block.** The cap fires at cycle 39 of this World's prose, when the history holds about 64800 characters. A story whose Narration answers are shorter reaches the cap much later. A model that writes the 150-word target of `response_length` gives about 200 characters per Narration, so one cycle is about 600 characters and the cap holds about 106 cycles. The cap counts characters, not cycles, so the cycle count depends on how long the model writes.

## 1.4 What the first drop takes

**Inferred.** At cycle 39 the block keeps the last 64000 of 64800 characters. The cut removes 800 characters from the front:

1. Message 81, the opening Narration. Carla introduces herself as Julian's bodyguard and asks to borrow his phone. The whole line goes, 542 characters.
2. 250 characters of Message 82, the first Impersonate Input. Julian studies Carla and hands over the phone with the cracked corner. The kept text starts in the middle of that sentence.

**Inferred.** The next text to go holds the first meeting with each other Character. Message 86 is the Trigger-driven introduction of Gabriella at the portico. Messages 92-96 hold Jezebel's account of Bernard's last call and of the GBP 60000 discrepancy in the foundation accounts. A story that loses the front of its history loses its opening scene first, then its first promise, then its first object.

**Known.** Nothing replaces the dropped text. The block keeps no summary line, no marker of the missing Messages, and no reference to the story tables.

## 1.5 A small connection loses text much earlier

**Inferred.** Cut 2 depends on the connection's context size. The engine reads `max_context_tokens` from the connection, else the provider default (`src/domain/model/llm_backend.rs:70-75`: OpenRouter 32768, Ollama 8192, Mock 4096). `fit_messages_to_context` then takes the context, holds back 256 tokens and the reply space, and drops history lines from the front.

| Connection context | User-half budget | History characters that survive | Cut that fires first | Cycles before the cut, this World |
|---|---|---|---|---|
| 32768 (OpenRouter default) | 29662 tokens | cut 1 caps it at 64000 | cut 1 | about 39 |
| 8192 (Ollama default) | 5086 tokens | about 15800 | cut 2 | about 10 |
| 200000 (a long-context model, set in data JSON) | about 196894 tokens | cut 1 caps it at 64000 | cut 1 | about 39 |

**Known.** The non-history part of the user half is measured at 4556 characters in `llm_messages.id` 172 (47447 minus 42891). The 8192-token row subtracts that measured value from the budget.

**Consequence, inferred.** The engine's real limit is not one number. It is the smaller of the 16000-token history cap and the space that the connection leaves. An author who picks a small local connection moves the first loss to about cycle 10, and no part of the UI says so.

## 1.6 The rule with no data

**known** The Narrator preset of both Narrator Modes says: "Each Character is a separate entity with their own knowledge and memory. Characters only know what they have witnessed or been told." The text is in `data/prompt_presets/system/default.json` and `data/prompt_presets/system/if_default.json`, in the `instructions` field.

**known** No part of a call records what one Character has seen. The prompt carries the Character cards of the Characters present, the names of the others, and the same history for every call. A memory that belongs to one Character has no place to live and no field to travel in.

# 2. The methods

## 2.1 What each method stores, writes and costs

| Method | What it stores | Who writes it, and how often | Where it goes in the prompt | Cost per cycle |
|---|---|---|---|---|
| **ST Summarize** | one rolling summary text, stored on a Message's `extra.memory` field of the chat file (`memory/index.js:964-978`) | one extra model call. Default: every 10 new Messages, or every N words (`:122`, `:604-614`). The writer prompt is a plain text field (`:105`) | an extension prompt. The default position is `After Main Prompt / Story String`, which puts it before the history, role `system`, depth 2 (`:113-121`; `settings.html:114-142`). The template is `[Summary: {{summary}}]` (`:106`) | 1 call per 5 cycles. Input is the previous summary plus the Messages since. Output is 200 words by default (`:118`) |
| **ST Vector Storage** | an embedding for each chat Message text, in a local vector collection. Chunks of 400 characters (`vectors/index.js:94`) | an embedding call per new or changed text, 5 texts per batch (`:440`, `:469-487`). No completion call | the 3 most relevant Messages become a text block, inserted at the configured position and depth, default depth 2 (`:854`, `:92-93`). The default template is `Past events:\n{{text}}` (`:88`) | 1 embedding per changed Message, plus 1 query embedding per generation (`:93`) |
| **ST World Info timed effects** | a per-entry timer in the chat's `timedWorldInfo` metadata: `start` and `end` in Message counts (`world-info.js:604-611`, `:560-567`) | no model call. A rule: an activated entry with `sticky` N stays active for N Messages, then goes on `cooldown` for M Messages. `delay` blocks the first N Messages (`:513-540`, `:663-687`) | the entry text enters the prompt at its own position and depth, as any World Info entry does | no call. The tokens are the entry text |
| **ST Author's Note** | one hand-written text per chat | the user writes it. No model writes it | a depth injection, default 2, role `system` (`script.js:4485`) | tokens only. A place for a manual memory |
| **ST Data Bank** | embedded files and attachments, in a collection per source | an embedding per file chunk (`vectors/index.js:723-760`). No completion call | the 5 best chunks as `Related information:\n{{text}}` at depth 4 (`:103-107`) | embeddings only. It is file memory, not event memory |
| **ME Advanced Memory** | scene records with content, `startMessageId` and `endMessageId`, an audience list, a title, a timeline, a fingerprint and an embedding (`packages/shared/src/types/advanced-memory.ts:120-140`) | 1. a scene check call every 5 Messages, default (`advanced-memory.ts:3651-3655`). 2. a summary call when a scene closes. 3. an embedding per record. 4. an optional decision call (`advanced-memory-decisions.ts`). The summary call writes the recollections in the same JSON output (`advanced-memory.ts:1839`) | one marker for each part: `chat_summary`, `current_scene_summary`, `recalled_scenes`, `recalled_messages` (`prompt/marker-expander.ts:204-211`). The default preset puts `chat_summary` after the Persona and before `chat_history` (`db/default-preset.json`, section 8) | 1 cheap call per 5 Messages, 1 summary call per scene, embeddings. The memory budget is 10 per cent of the context by default (`summaryBudgetTokens`, `types/advanced-memory.ts:15-17`, `:40-45`) |
| **ME Chat Summary** | append-only summary entries, up to 200 automated entries (`shared/src/utils/chat-summary-entries.ts:13`) | one call per 5 new Messages (`roleplay-summary-runtime.ts:14`). The prompt says: "Write an appendable continuation. Do not rewrite or repeat the previous summary" (`shared/src/constants/agent-prompts.ts`) | into the `chat_summary` marker, or to the end of the system block when the preset has no marker (`prompt/assembler.ts:774-790`) | 1 call per 5 Messages. The output is short and additive |
| **ME Memory Recall** | chunks of 5 Messages, embedded, with a similarity threshold of 0.25 and a top count of 8 (`memory-recall.ts:18-35`) | an embedding per chunk. No completion call | read by agents and tools, not by a preset marker | embeddings only |
| **ME Long-term memory package** | a package-owned vault of "durable memories", built from chat summaries, character records and lorebooks | the package owns the writes. The host calls `recall()` before the generation and `recordPromptAccepted()` after (`generation/long-term-memory-runtime.ts:20-40`, `routes/generate.routes.ts:7428-7457`, `:8754-8772`) | the preset's runtime-agent marker if the preset has one, else a fallback injection (`generate.routes.ts:7437-7454`) | 1 recall per generation, with no model call in the host |
| **ME Lorebook Keeper** | a lorebook entry per durable story fact, with `sourceAgentId` and `sourceMessageRefs` (`shared/src/types/lorebook.ts:154-157`, `:265-280`) | an agent model call in the post-processing phase, when the agent is active (`services/agents/agent-pipeline.ts:523`) | a lorebook entry, at its own position and depth, by key match | 1 agent call per generation |
| **AI Dungeon: Memory System** | a summary of 6 Actions, embedded. A second, longer Story Summary | a summary call every 6 Actions. Auto Summarization refreshes the Story Summary every 15 Actions | Story Summary and Memory Bank are two of the Ordered Context parts, at positions 4 and 5 of 9 | 1 call per 6 Actions, plus 1 call per 15 Actions |
| **AI Dungeon: Plot Essentials** | one hand-written text per Adventure | the user writes it | part of the Required elements, before Story Cards | tokens only |
| **NovelAI: Memory and Author's Note** | two hand-written texts per story | the user writes them | Memory sits at the top of the context by default. Author's Note sits among the most recent text | tokens only |
| **Generative Agents** | a memory stream of natural-language observations, with a creation time and a last-access time | the agent writes observations from its own perceptions, and reflections from the stream | the top-ranked memories that fit the context | 1 importance score per memory object, plus a reflection call at an importance sum of 150 |
| **MemGPT / Letta** | a tiered store: main context, recall storage and archival storage, with memory blocks in the main context | the model itself pages data between the tiers, through function calls | the main context holds the blocks, and the model edits them | function calls inside the generation |

## 2.2 What an author can change

| Method | Writer prompt | Size | Place in the prompt | Depth | Manual edit | Off switch |
|---|---|---|---|---|---|---|
| ST Summarize | yes, a text field (`memory/index.js:105`) | yes, target words 25-1000, default 200 (`:118-120`) | yes: in-prompt, in-chat, before-prompt, nowhere (`settings.html:114-142`) | yes, 0-9999 | yes, typing in the summary box (`memory/index.js:943-945`) | yes: source set to none, or Pause |
| ST Vector Storage | yes, for the message summary step (`vectors/index.js:80`) | yes: chunk size 400, insert 3, protect 5 | yes, position and depth (`:89`, `:853-854`) | yes | no | yes, per chat |
| ST World Info | no writer | yes: sticky, cooldown, delay, probability, depth | yes, position and depth | yes | yes, the entry text | yes, per entry |
| ME Advanced Memory | yes: the summary prompt, with a settings key (`CHAT_SUMMARY_PROMPT_SETTINGS_KEY`) | yes: `summaryBudgetTokens`, `maxContextTokens` | yes: the four markers are preset sections, so the author moves and re-roles them | yes, the section role and the preset order | yes: a hand edit sets `manualOverride`, and the Fix job treats it as authoritative (`advanced-memory.ts:3407`) | yes, `enabled` |
| ME Chat Summary | yes: prompt and combine prompt | no direct size, only the entry count | yes, through the `chat_summary` marker | yes | yes, manual entries | yes |
| AI Dungeon | no | yes, through the Context allocation | no | no. Ordered context is fixed | yes, Story Summary and Plot Essentials | yes, two toggles |
| NovelAI | no | yes, through the context settings | yes, `Insertion Position`, `Insertion Order`, `Token Budget`, `Reserved Tokens` per entry | yes | yes | yes |
| Generative Agents | no | no | no. Retrieval decides | no | no | no |

**Known.** Only ST, ME and NovelAI give the author a place, a depth and a size for the memory text. AI Dungeon fixes the order of its context parts. Generative Agents and MemGPT have no author facing control at all, because they are research systems.

## 2.3 The token cost inside the prompt

**known** Each method caps its own share of the context. The cap is what an author or a system sets, and the memory text sits inside it on every call.

| Method | Cap inside the prompt | Source |
|---|---|---|
| ST Summarize | the target summary length, 200 words by default. About 1300 characters, or 325 estimated tokens, on every call | `memory/index.js:118` |
| ST Vector Storage | 3 retrieved Messages, each chunked at 400 characters. The block therefore holds about 1200 to 4000 characters | `vectors/index.js:92-94` |
| ME Advanced Memory | `summaryBudgetTokens`, 10 per cent of `maxContextTokens`. The default pair is 6500 of 65000 | `types/advanced-memory.ts:15-17`, `:40-45` |
| ME Memory Recall | 8 chunks of 5 Messages, filtered at a similarity of 0.25 | `memory-recall.ts:18-35` |
| AI Dungeon | the Story Summary sits inside the 70 per cent Required cap. The Memory Bank takes about 25 per cent of what remains after Required | AID, What goes into the Context |
| NovelAI | `Token Budget` and `Reserved Tokens` per entry, as a count or as a fraction of the context | NovelAI, Lorebook |

**Inferred.** A summary is the cheapest useful memory. ME caps it at 10 per cent of the context and AID at about 25 per cent of the dynamic space. Both keep the live history in the rest. Our Narrator call spends no tokens on memory today, so the first memory layer can take its budget from the space that the history does not use.

# 3. Memories that belong to a Character

**known** ME Advanced Memory is the only production method here with a per-Character model. The other production methods keep one memory for the whole chat. Two research systems come close, and the list is below the table.

| Part | Rule | Source |
|---|---|---|
| The owner list | `audienceCharacterIds`. An empty list means the Narrator alone, never all Characters | `types/advanced-memory.ts:128-130` |
| Who is chosen | the summary model returns `audience`, a list of names of the Characters present in the scene. The prompt says a Character who is "merely mentioned, remembered, discussed, or addressed while absent is NOT a participant" | `advanced-memory.ts:1839` |
| One Character's own memory | a `recollection` record: a first-person memory of 50-100 words, written in that Character's voice, from a specification in the same prompt. The prompt names what to skip: "A character can use only the messages they could know" | `advanced-memory.ts:259` (`RECOLLECTION_RULES`), `:5064-5080` |
| How it reaches a call | the recalled scene passes only when the audience matches. A single Character's call needs that Character in `record.audienceCharacterIds`. A merged call takes any present Character who remembers it, and the line names who does | `recallAudienceMatches`, `:4667-4677` |
| The line at the call | `${name}'s personal recollections from this scene: *text*` | `:5064-5080` |
| When a Character's knowledge starts | `knowledgeStarts`, one entry per Character. A null value confirms knowledge from the beginning. A missing value blocks the job until the user confirms | `types/advanced-memory.ts:25-27`, `advanced-memory.ts:3632-3633` |
| Hidden Messages | `autoMessageVisibility`, group chats only: the model hides each new Message from the Characters it finds absent | `types/advanced-memory.ts:24` |

**known** ME renders the memory separately for each reader and then checks for leaks. It normalises both sides and reports a run of four words that a private section shares with a wider section (`advanced-memory.ts:4428-4520`). It also removes a sentence that the Narrator's copy and another Character's copy both hold (`:4408-4420`).

**known** ST has no per-Character memory. The summary belongs to the chat. The `{{summary}}` macro and the injection carry one text for every participant.

**Known, from the same read.** The two methods that come closest outside ST and ME are:
- **MemGPT/Letta** keeps its memory blocks in the main context and lets the model rewrite them. One agent can hold several blocks, so one block per Character is possible. The paper does not model Characters.
- **Generative Agents** keeps one memory stream per agent, so a per-Character memory is the default shape. It has no Narrator and no shared memory.

**Inferred.** Our engine can host a per-Character memory with little new machinery. The Swipe state already names the Characters present (`domain/model/state/scene_state.rs`), and the prompt already gives full cards to those Characters only (`assembler.rs:216-263`, `builders/sections.rs:75-108`). A memory filtered by "Characters present" therefore follows the same rule as a card, and it follows a swipe for free.

# 4. Swipes, edits and deletes

The question: how does a memory stay true when a Message is swiped (retried), edited or deleted?

| Method | Swipe or Retry | Edit | Delete |
|---|---|---|---|
| **ST Summarize** | the summary lives on a Message's `extra.memory`. The code deletes that field when the last Message text differs from the stored hash, so a regenerate of the last Message clears its summary (`memory/index.js:456-465`). `getLatestMemoryFromChat` skips the last Message (`:357-372`), so a fresh summary is never read back | the same hash check clears it. The summary is a rolling rewrite, so the older summaries stay as text inside the newest one | a shorter chat reloads the latest summary from an earlier Message (`:452-455`) |
| **ST Vector Storage** | the index is keyed by the hash of the Message text (`vectors/index.js:463-469`). A new swipe text is inserted, and the old hash is deleted (`:469`) | the same path, because an edit changes the hash | the missing hash is deleted from the collection |
| **ST World Info timed effects** | a timer that has not advanced is removed: "chat not advanced" (`world-info.js:626-630`). A Retry does not advance the chat, so its sticky timer is rolled back | the timers do not change | the timers of a missing entry are removed at the end of their interval (`:633-638`) |
| **ME Advanced Memory** | a finished archive record keeps its source Message IDs and its index across "text edits, swipes, illustrations and live cutoffs" (`advanced-memory.ts:1193-1206`). Recall then reads the current Messages. Each Swipe also stores the prepared memory it was generated with, in its own extras, and the host validates that snapshot before reuse (`generation/advanced-memory-context.ts:27-52`, `:74-104`) | the same rule. A hand edit sets `manualOverride` and the record stays authoritative | a record whose covered Message is missing fails `recordValid` (`:1186-1200`) |
| **ME Lorebook Keeper** | `sourceMessageRefs` pins the swipe index. At injection time an entry whose pinned swipe is no longer active is excluded, and it comes back when the user swipes back (`services/lorebook/index.ts:1310-1346`). The type comment states the rule | a hand edit clears `sourceAgentId`, which makes the entry immune to the delete cascade (`types/lorebook.ts:265-280`) | the storage cascade reverts the entry to its pre-write snapshot or removes it (`:277-280`) |
| **ME Long-term memory package** | recall does not run on a regenerate (`routes/generate.routes.ts:7428`, `:3969`). A cached injection from the same prompt is reused instead (`:7326-7327`) | not read | not read |
| **AI Dungeon** | a Memory covers Actions that are at least 6 old, so a Retry or an undo of the last 6 Actions "without it impacting memories" (AID help, Memory System) | an edit of older text does not regenerate the summary. AID says so, and gives the cost of the summarization model as the reason | not read |
| **Generative Agents, MemGPT** | no read | no read | no read |

**Known, and the strongest single fact of this section.** ME gives the three cases three different answers:
1. A memory that is finished keeps its source IDs and survives an edit or a swipe. Recall reads the current text.
2. An agent-written lorebook entry is lazily excluded when its pinned swipe is not active, so a Retry hides it and a swipe back shows it.
3. A prepared prompt snapshot is stored per Swipe, so a Retry can reuse exactly what its sibling Swipe saw.

**Known.** ST gives one answer to two different cases. It deletes the summary of the last Message on any text change, and it keys the vector index by text hash. A Retry therefore costs a new summary or a new embedding.

**Inferred.** Our engine's Swipe state is the natural home for a per-Swipe memory snapshot, because every Swipe already holds its own state record (`domain/model/state/swipe_state.rs`). A memory that belongs to the Game, and not to a Swipe, needs a rule for a Retry. ME's `swipeIndex` pin is the cheapest such rule that is written down in code.

# 5. Caching

**known** A memory that changes on every cycle moves the prompt prefix and breaks the cache. The methods differ in where they put the memory text.

| Position | Methods | Effect on the cache |
|---|---|---|
| The system block, before the history | ST Summarize default (in-prompt), ME `chat_summary` marker before `chat_history` in the default preset, AI Dungeon Story Summary and Memory Bank, NovelAI Memory at the top | a rewrite of the summary changes the prefix on the cycle that rewrites it only. The rest of the prefix stays |
| After the history, at a depth | ST Author's Note, ST Vector Storage default depth 2, ME `recalled_scenes` and `recalled_messages` where the author puts them | the prefix survives, and only the tail differs. This is the cheap position for text that changes often |
| Inside the history, as reordered Messages | ST Vector Storage removes the retrieved Messages from the chat array and inserts them as a block (`vectors/index.js:823-855`) | the prefix changes on almost every cycle, because the block sits two Messages from the end and the set of Messages changes |
| A separate message | ME injects its memory parts as sections with their own role (`prompt/advanced-memory-prompt.ts:36-52`) | the prefix survives when the section keeps its place |

**Inferred.** Three rules follow for our engine:
1. A memory whose text changes on every cycle belongs after the history, near the Player Input.
2. A memory that changes rarely can sit before the history, in the system block.
3. A memory that is a retrieved set of Messages will break the prefix on most cycles. Place it late, or accept the break.

[Find how prompt caching works for our providers and our layout](../issues/12-prompt-caching.md) owns the cache rules of the providers. This asset does not restate them.

# 6. Evidence on models and long context

| Claim | Evidence | Label |
|---|---|---|
| A model uses the middle of a long context worse than the start or the end | Liu et al., *Lost in the Middle*: accuracy is "often highest" at the start or the end, and "significantly degrades" in the middle, "even for explicitly long-context models" | known |
| The drop is large | the same paper: GPT-3.5-Turbo scores 56.1 per cent closed-book, and multi-document QA falls by more than 20 points. In the worst case a 20-document setting scores below closed-book. Llama-2 13B base shows a 20-point best-to-worst gap | known |
| A stated window is not an effective window | Hsieh et al., *RULER*: "almost all models exhibit large performance drops as the context length increases. While these models all claim context sizes of 32K tokens or greater, only half of them can maintain satisfactory performance at the length of 32K" | known |
| Long-term conversational memory is hard | Maharana et al., *LoCoMo*: conversations of about 300 turns and 9K tokens over up to 35 sessions. "LLMs exhibit challenges in understanding lengthy conversations", and long-context or retrieval methods "still substantially lag behind human performance" | known |
| A memory system can beat a big window | Park et al., *Generative Agents*: the memory stream plus reflection plus retrieval drives the behaviour. The paper demonstrates by ablation that observation, planning and reflection each matter | known |
| A working memory with tiers is a proven design | Packer et al., *MemGPT*: "virtual context management", with data moved between fast and slow memory tiers | known |
| A cheap importance score is enough to rank memories | *Generative Agents*: an integer 1-10 from the model, assigned when the memory object is created. The retrieval score is a weighted sum of min-max scaled recency, importance and relevance, with all weights at 1. Recency decays by 0.995 per game hour | known |
| Reflection is a second write pass | *Generative Agents*: reflections are generated when the importance sum of the latest events passes 150. The agent reflected "roughly two or three times a day" | known |
| The length of one cycle decides when the cap fires | the measurement of section 1.3 | known, for this World |

**Inferred from the two evidence rows.** Our cap holds about 39 cycles of this World's prose, and about 106 cycles of a 150-word prose. A model reads the middle of that history worse than its ends. So the tail of a long Autoplay story is already weaker than its head, before the cap drops anything.

# 7. Facts for the other tickets

## For [Decide which parts of a Narrator-role call an author can change](../issues/06-the-layout-model.md)

1. **known** The history has a hard character cap of 64000 characters, and the fit to the context can cut it again. Neither has an author handle. An author cannot change the cap, the label cost, the cut marker or the cut direction.
2. **known** The user half has no memory layer. The order is Game State, Character cards, Persona, World Lore, History, the post-history splice, Player Input, Guide (`assembler.rs:165-184`).
3. **A place for memory, from the methods.** Every method that works places its memory in one of three spots: a block before the history (ST Summarize, ME `chat_summary`, AI Dungeon), a block after the history at a depth (ST Author's Note, ME `recalled_scenes`), or a replacement for the history itself.
4. **known** The cheapest first step is available today. The preset has a post-history half (`<writing_style>`, `<output_format>`), so a memory block can land after the history with no engine change to the order. The content must still come from somewhere.
5. **known** The cut marker text and the label cost are engine rules. Ticket 06 decides whether they become author fields.

## For [Decide which parts of a Quantifier or Options call an author can change](../issues/10-quantifier-options-layout.md)

1. **known** An Agent runs at one of two Hooks, the after-narration Hook and the end-of-Job Hook. The runner holds one Agent per Hook, and a second config replaces the first with a warning (`agents/runner.rs:23-64`). An unknown agent type is a config error (`:62`).
2. **known** A failed after-narration Agent gives the empty Quantifier result, and the Job continues (`runner.rs:71-83`). A failed end-of-Job Agent gives an error (`:84-101`). A memory writer at the after-narration Hook would inherit the first rule, which is quiet. A writer needs the second rule, or a third Hook.
3. **known** The Quantifier and the Options each build their own two halves, with no fitting and no shared layers (`agents/quantifier/prompt.rs`, `agents/options/prompt.rs`). A memory writer as a new Agent can do the same, and it needs no change to the Narrator assembly.
4. **The cost, from the methods.** ME splits the work in three: a cheap check call per 5 Messages, a summary call per scene, and recollections in the same summary call. ST uses one call per 10 Messages. AID uses one call per 6 Actions. One call per scene is the cheapest shape that still names the participants.
5. **known** The per-Character data already exists in the Swipe state, so a memory writer can read "Characters present" without new state.

## For [Shape the test setup and the trial](../issues/08-test-setup-and-trial.md)

1. **known** The reading pack can show what fell out of the window, because the engine stores the full request of each call in `llm_messages`. Two numbers are enough:
   - the count of Messages in the story that the `<ConversationHistory>` block does not hold.
   - the character offset of the front cut, and whether the retained text starts in the middle of a line.
2. **known** The two markers make the check mechanical: `(History truncated to fit context window)` for cut 2, and a first line that does not start with `Narrator: ` or the Persona name for cut 1.
3. **known** No stored story in the repository reaches a cut at its length. A trial must run about 39 cycles of this World's prose to show cut 1, or use a connection with an 8192-token context to show cut 2 at about cycle 10.
4. **known** A trial needs a memory field to compare two setups. Without one, the second setup differs only in the prose length of its model, and the cycle of the first loss moves with that length.
5. **known** The transcript is already a part of the reading pack. A memory layer adds a second artefact: the memory text that the call carried.

# 8. What I could not do

- **No paid model call, no server.** I read the stored requests only. The cycle count of 39 is inferred from the measured line sizes of Game 6, because no stored story in the repository reaches the 64000-character cap.
- **Game 6 is a repeated story.** Its 50 Messages are 20 Messages copied. The per-cycle growth is valid. The content of a cut at cycle 39 is inferred from the same copy, so the named texts come from the 20-Message original.
- **No read of AI Dungeon or NovelAI at run time.** Their facts come from the provider documents of 2026-10-31. The summary cadence and the context allocations are documented, and the internal prompts are not public.
- **No read of ME's decision model details.** `advanced-memory-decisions.ts` decides whether a recall is worth a call. I read its name, its defaults and its diagnostics type, and not its prompt.
- **I did not measure a real token count.** The engine estimates 4 characters per token. All token numbers here use that estimate, or the provider's own count where a document gives one.

# Sources

Provider documents:

| Source | URL | Read |
|---|---|---|
| AI Dungeon, What is the Memory System? | https://help.aidungeon.com/faq/the-memory-system | 2026-10-31 |
| AI Dungeon, What goes into the Context sent to the AI? | https://help.aidungeon.com/faq/what-goes-into-the-context-sent-to-the-ai | 2026-10-31 |
| AI Dungeon, What is Plot Essentials? | https://help.aidungeon.com/faq/plot-essentials | 2026-10-31 |
| NovelAI, Lorebook | https://docs.novelai.net/en/text/lorebook/ | 2026-10-31 |
| NovelAI, Story Settings (Memory, Author's Note) | https://docs.novelai.net/en/text/editor/storysettings/ | 2026-10-31 |
| NovelAI, Glossary (context) | https://docs.novelai.net/en/text/glossary/ | 2026-10-31 |

Papers:

| Paper | URL | Read |
|---|---|---|
| Park et al., *Generative Agents: Interactive Simulacra of Human Behavior*, arXiv 2304.03442 | https://arxiv.org/abs/2304.03442 | 2026-10-31 |
| Liu et al., *Lost in the Middle: How Language Models Use Long Contexts*, arXiv 2307.03172 | https://arxiv.org/abs/2307.03172 | 2026-10-31 |
| Packer et al., *MemGPT: Towards LLMs as Operating Systems*, arXiv 2310.08560 | https://arxiv.org/abs/2310.08560 | 2026-10-31 |
| Maharana et al., *Evaluating Very Long-Term Conversational Memory of LLM Agents* (LoCoMo), arXiv 2402.17753 | https://arxiv.org/abs/2402.17753 | 2026-10-31 |
| Hsieh et al., *RULER: What's the Real Context Size of Your Long-Context Language Models?*, arXiv 2404.06654 | https://arxiv.org/abs/2404.06654 | 2026-10-31 |

Source trees:

| Tree | Path |
|---|---|
| SillyTavern `06bde939` | `/tmp/research/st` |
| Marinara Engine `457e827a` | `/tmp/research/marinara` |
| Stored calls of Game 6 | `tmp/stage6/debug/chronicler_3060.db` |
| Small Games | `tmp/db-backup-20261031/*.db` |
