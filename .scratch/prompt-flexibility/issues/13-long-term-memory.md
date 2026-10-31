# Find how ST, ME and other tools keep long-term memory

Type: research
Status: resolved
Blocked by: —

## Question

When a story grows past the history window, what does our engine lose today? How do SillyTavern, Marinara Engine and other interactive-fiction tools keep what happened earlier, as one shared memory and as memories that belong to one Character? Which call writes each memory, where does it go in a prompt, what can an author change, and how does it stay true when a Message is swiped, retried, edited or deleted?

## Context

- **Owner's view (this session).** The engine has no long-term memory. Long play and Autoplay need it: memories added to Characters, and long-term state that changes. It is a large unknown and context for most decisions of this map. [Find how ST, ME and other tools keep long-term state](14-long-term-state.md) covers the state. This ticket covers the memory of events.
- **The history cut (known, from [List what reaches the model in each call](01-what-reaches-the-model.md)).** It is fixed in code: a 16000-token character cut, then older lines are dropped to fit the context (`src/application/prompting/token_budget.rs`, `utils/context.rs`). Nothing replaces the dropped text.
- **A rule with no data under it (known).** The Narrator preset says "Each Character is a separate entity with their own knowledge and memory. Characters only know what they have witnessed or been told." No part of a call records what each Character has seen.
- **Where a writer could run in our engine (known, `CONTEXT.md`).** An Agent runs at a Hook: the after-narration Hook or the end-of-Job Hook (`src/application/agents/`). The Quantifier and the Options Agent are the two Agents today. The Swipe state belongs to one Swipe, so state on it follows a swipe.
- **Start points in ST and ME (known: these paths exist at the pinned commits).**
  - ST: `public/scripts/extensions/memory/` (Summarize), `public/scripts/extensions/vectors/` (Vector Storage), the Data Bank, and the World Info `sticky`, `cooldown` and recursion settings.
  - ME: `packages/server/src/services/advanced-memory*.ts`, `memory-recall*.ts`, `chat-summary/`, the markers `chat_summary`, `current_scene_summary`, `recalled_scenes` and `recalled_messages`, the Memory Nag agent, and lorebook entries that an agent writes from a Message (`SourceMessageRef` with `swipeIndex`).
  - Earlier notes: `docs/external_applications/sillytavern_prompt_system.md` (summarization and hierarchical memory) and `.scratch/_closed/rust-rebuild-build/assets/generation-shape/marinara-agents.md`.
- **Outside ST and ME.** Interactive-fiction tools: AI Dungeon (Memory, the Memory Bank, Story Cards, Plot Essentials) and NovelAI (Memory, Lorebook, Author's Note). Methods with primary sources: rolling or recursive summaries, retrieval of past Messages by embedding, the memory stream and reflection of Generative Agents (Park et al.), and the memory tiers of MemGPT / Letta. Evidence on how well models use a long history ("lost in the middle", the effective context against the stated window), and a benchmark for memory over a long conversation (for example LoCoMo).

Points to cover:

1. **Our loss, measured (the outside check).** Use the dev DB (see `ENVIRONMENT.md` for its place) or [the story transcript](../assets/autoplay-story-transcript.md). At which cycle does the cut first drop text from a Narration call? What did the dropped text hold: the opening, the first meeting with each Character, a promise, an object? Read the stored requests. Do not estimate.
2. **For each method:** what it stores, which call writes it (an extra model call or a rule), how often, where it goes in the prompt, its cost per cycle in calls and tokens, and what an author or a user can set (the writer prompt, the size, the place, the depth, a manual edit).
3. **Memories that belong to a Character.** Which method keeps what one Character saw, heard or felt apart from what the narrator knows? How does it choose which Character gets a memory, and how does a memory reach a call only when that Character is present?
4. **Swipes and edits.** How does each method keep its memory true when a Message is swiped, retried, edited or deleted? ME's `swipeIndex` is one answer.
5. **Caching.** A memory that changes moves the prompt prefix. Note where each method puts it. [Find how prompt caching works for our providers and our layout](12-prompt-caching.md) covers the cache.

## Output

An asset `assets/long-term-memory.md` with:

- the measured loss in our engine
- a table of methods: what is stored, who writes it, where it goes, its cost, its author handles, its per-Character support and its Swipe behaviour
- facts for [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md) (the history cut and a place for memory), [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md) (a memory writer as an Agent at a Hook) and [Shape the test setup and the trial](08-test-setup-and-trial.md) (show in the reading pack what fell out of the window)

This ticket finds facts. It does not design a memory layer. The answer must give the owner what they need to decide whether this map designs memory or only keeps room for it.

## Answer

Asset: [How ST, ME and other tools keep long-term memory](../assets/long-term-memory.md)

- **No stored story loses text, so the cycle of the first loss is computed, not observed.** In Game 6 (50 Messages) the history reaches 42891 characters, or 10722 estimated tokens by the engine's own 4-characters-per-token rule. The 16000-token cap is at 64000 characters, so no stored call drops any text. The eight small Games under `tmp/db-backup-20261031/` do not reach a cut either. [known]
- **The first drop comes at cycle 39 of this World's prose.** One 20-Message block measures 16435 characters and 4108 estimated tokens, so one cycle adds 1643 characters and 411 tokens. The cap then fires at cycle 39, and the 800 dropped characters are the whole opening Narration and the head of the first Impersonate Input. [inferred from measured line sizes]
- **The connection can matter more than the history cap.** `fit_messages_to_context` cuts again to the connection's context. An 8192-token connection (the Ollama default) leaves about 15800 history characters and loses text at about cycle 10. An author can move the first loss by 29 cycles by changing the connection, and no UI field says so. [inferred]
- **The two cuts are the only losses.** Cut 1 is a character cut at 16000 tokens that can start mid-line. Cut 2 drops whole lines and writes `(History truncated to fit context window)`. Neither has an author handle, and nothing replaces the dropped text.
- **ME Advanced Memory is the only production method with a per-Character memory.** Its records carry `audienceCharacterIds`, and an empty list means the Narrator alone. The same summary call writes one first-person `recollection` of 50-100 words per present Character. `knowledgeStarts` fixes where each Character's knowledge begins. ST keeps one summary for the whole chat.
- **The methods split into four prompt positions.** Before the history (ST Summarize's default, ME's `chat_summary` marker, AI Dungeon Story Summary and Memory Bank), after the history at a depth (ST Author's Note, ST Vector Storage at depth 2, ME `recalled_scenes`), in place of the history (a retrieved set of Messages), or as a separate message.
- **ME gives swipes, edits and deletes three different answers.** A finished scene record keeps its source IDs and survives both. An agent-written lorebook entry pins `swipeIndex` and is excluded at injection time when that swipe is not active, then comes back on a swipe back. Each Swipe also stores the prepared memory snapshot it was generated with. ST deletes the summary of the last Message on any text change and keys its vector index by text hash.
- **One cheap call per scene is the cheapest shape that still names the participants.** ME uses a scene check call every 5 Messages plus one summary call per scene. ST uses one call per 10 Messages. AI Dungeon uses one per 6 Actions. Generative Agents adds an importance score per memory and a reflection pass at an importance sum of 150.
- **The token cost of a memory is a cap, not a guess.** ME caps memory at 10 per cent of the context (6500 of 65000). AI Dungeon keeps the Story Summary inside a 70 per cent Required cap and gives the Memory Bank about 25 per cent of the rest. NovelAI sets `Token Budget` and `Reserved Tokens` per entry.

Facts the later tickets need:

- **Ticket 06.** The user half has no memory layer, and its order is Game State, Character cards, Persona, World Lore, History, the post-history splice, Player Input, Guide (`assembler.rs:165-184`). The cheapest place for a memory block today is the post-history half, which already carries `<writing_style>` and `<output_format>`. The 64000-character cap, the label cost and the cut marker are engine rules for ticket 06 to keep or expose.
- **Ticket 10.** The runner holds one Agent per Hook and an unknown agent type is a config error (`agents/runner.rs:23-64`). A failed after-narration Agent falls back to the empty Quantifier result and the Job continues (`:71-83`), while a failed end-of-Job Agent returns an error (`:84-101`). A memory writer needs the error path, or a third Hook. The Characters present are already in the Swipe state, so a writer needs no new state.
- **Ticket 08.** The reading pack can report the loss from the stored request alone. Count the Messages that the `<ConversationHistory>` block does not hold, and give the character offset of the front cut. A trial needs about 39 cycles of this World's prose to show cut 1, or an 8192-token connection to show cut 2 at about cycle 10. A trial that compares two setups needs a memory field, because otherwise only the prose length differs.

Open question for the map: the cycle count depends on how long the model writes, so a trial needs a fixed `response_length` and a fixed connection context, or the two setups differ in a second variable.
