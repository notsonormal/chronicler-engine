# What is known about long-term memory for LLMs, and what is still open

Ticket: [Find what is known about long-term memory for LLMs, and what is still open](../issues/02-llm-memory-research.md)

Scope: the field in general. The earlier survey ([how other tools keep long-term memory](../../prompt-flexibility/assets/long-term-memory.md), sections 2 and 6) covers ST Summarize, ST Vector Storage, AI Dungeon, NovelAI, Generative Agents, MemGPT, and a short note on LoCoMo, *Lost in the Middle* and RULER. This asset does not repeat it. Marinara Engine (ME) has one row. Ticket 01 covers ME in depth. The engine terms are those of `CONTEXT.md`: Game, Message, Swipe, Character, Persona, Narration, Cycle, Agent, Hook.

Labels: **known** (read in the primary source on the access date), **inferred** (my reasoning from a read), **guessed** (no source). A product claim is not a measured result. The text marks each product claim as "claim". A vendor paper is a measured result by an interested party, and the text says so.

Access date of every source: 2026-11-03, unless the Sources list says otherwise. Where I read only an abstract, the text says "abstract only".

Short answer (known unless marked):
- Twelve families exist (table in section 1). No family is solved.
- On public benchmarks, a full-context run beats every memory system on accuracy when the history fits. Memory systems win on cost and latency.
- Updating a changed fact, time, and what each Character knows are the weakest points in the sources.
- Only one family was built for who knows what: a filtered view for each reader. It worked in a short template test.
- I found no benchmark and no method for an in-story clock, and no benchmark for an interactive story with several Characters (inferred gap).

## 1. The families

How to read the table. "Writer" is the call or person that makes the memory. "Evidence" is a measured result when the cell says "measured", and a product claim when it says "claim". "Fit" is my reading for a long story with several Characters (inferred, not tested). Details of each measured result are in section 2.

| Family | What it stores | Writer, and when | How it is found | Cost | Evidence | Fit for several Characters (inferred) |
|---|---|---|---|---|---|---|
| A. Rolling or recursive summary | One text, rewritten from the old summary and the new turns | One model call after each chunk (session, N turns) | Always in the prompt; no search | One call per chunk; output has a fixed size | Measured: on DMR, recursive summarization 35.3% against 94.4% for full context (`gpt-4-turbo`; the first figure is as MemGPT reported). On LoCoMo, summary RAG did not help although recall was high | Weak. One voice. Old detail is lost at each rewrite. No place for who knows what |
| B. Hierarchical summary or tree (RAPTOR, ReadAgent, Re3 outlines, Kindroid cascaded) | Summaries at several levels; ReadAgent keeps "gist" pages with a link to the source text | Model calls that cluster and summarize (RAPTOR), or split into episodes and gist them (ReadAgent) | Tree search by embedding (RAPTOR); the model asks to open a page (ReadAgent) | Several calls per chunk; indexing cost grows with the text | Measured by the authors on document QA (RAPTOR +20 points on QuALITY with GPT-4; ReadAgent extends the context 3.5 to 20 times). No chat benchmark found | Medium. Good for plot level. A tree does not mark who saw what |
| C. Embedding retrieval of past text | The raw turns or chunks, with a vector | An embedding call per turn. No completion call | Top-k by similarity to the latest turn or a query | Embeddings only. Prompt grows by k chunks | Measured: LoCoMo RAG recall is high with big k, but accuracy peaks at a small k (5 observations) and falls when k grows. LongMemEval: finer units (rounds) beat sessions | Medium. Finds a scene. Needs the turn to name the entity. Weak for time and for "who" |
| D. Extracted facts with ADD, UPDATE, DELETE (Mem0) | Short fact sentences, with a vector | One extraction call for each message pair, then an update call against the 10 closest facts | Top-k by similarity | One extraction call for each message pair, and one update call for each fact (inferred from the design). Context about 1.8k tokens at question time | Measured (vendor): LoCoMo J 66.9, below full context 72.9. The authors report 91% lower p95 latency and over 90% fewer tokens than full context. LongMemEval: expanding keys with extracted facts gave +9.4 recall and +5.4 accuracy | Good for one entity at a time. A fact needs an owner field for several Characters. Overwrite loses the old value |
| E. Temporal knowledge graph (Zep, Graphiti, Mem0 graph variant, AriGraph) | Entities and facts as edges, with valid-from and valid-to. Raw episodes stay and link to the facts | Several calls for each message: entity extraction, resolution, fact extraction, dedup, time extraction, invalidation. Community summaries. | Hybrid: cosine, BM25 and graph walk, then rerank | The highest of the families I read. The paper gives latency and no cost for each message | Measured (vendor): LongMemEval_S +15.2% (`gpt-4o-mini`) and +18.5% (`gpt-4o`) over full context, with 90% lower latency; −17.7% on single-session-assistant questions. LoCoMo Zep number is disputed (58.44, 65.99 or 75.14) | Good for time and for change. Actors are entities, so a Character can own an edge. Heavy |
| F. Memory the model edits (MemGPT, Letta blocks, sleep-time agents) | Named blocks in the prompt, plus recall and archive stores | The generating model, through tool calls. A second "sleep-time" agent can rewrite blocks between turns | The blocks are always in the prompt; the archive is searched by a tool | Tool calls inside the main generation. A second model for sleep time | Measured: MemGPT 93.4% on DMR (as reported). Letta plain-file agent 74.0% on LoCoMo (vendor blog). Zep could not run MemGPT on LongMemEval | Medium. A block for each Character is easy. The writer can drift, and nothing checks it |
| G. Linked notes (A-MEM) | A note with keywords, tags, a description and links to other notes. Old notes can be updated by new ones | One call per new memory to write the note, find links and update neighbours | Embedding search, then follow links | Several calls per memory | Measured: the authors report gains on LoCoMo; in the Mem0 team's re-run A-Mem scored J 48.4, the lowest. Vendor against author, not reconciled | Unknown. Not designed for several Characters |
| H. Reflection (Generative Agents) | Observations plus higher-level reflections, in one stream for each agent | The agent writes observations. A reflection call runs when the importance sum passes a threshold | Score: recency, importance, relevance | An importance call for each memory plus reflection calls | Measured by ablation in the paper (earlier survey). I found no LoCoMo or LongMemEval number for it | Good by shape: one stream for each agent. No Narrator and no shared story memory |
| I. Narrative plan and state for generation (Re3, DOC, RecurrentGPT, NstAgent) | Plan or outline, Character descriptions, a dictionary of attributes, a short-term memory text, summaries | Model calls at each step (plan once, update after each passage) | Fixed slots in the prompt, plus retrieval of the Character descriptions | One to several calls for each passage | Measured: human preference for coherence over a rolling window (Re3 +14 points, DOC +22.5 over Re3, RecurrentGPT preferred over both). The Re3 Edit module had little effect | Good for plot. Re3 keeps one dictionary for each Character, which helps with facts about a Character, but not with who knows what |
| J. Agentic search over plain text or a tree (Letta filesystem, TunnelVision) | The raw chat in files, or a lorebook with a tree index | None, or a summary writer | The model searches with tools (grep, open, tree walk) | Several tool calls for each answer | Measured (vendor): Letta 74.0% on LoCoMo. TunnelVision: no result found | Medium. Needs a model that uses tools well |
| K. Hand-written or pinned memory (NovelAI Memory, Character.AI Story Memory and pins, Kindroid key memories and journal, ST Author's Note) | Text that a person writes or pins | The person | Always in the prompt, or by keyword | Tokens only | Claim only | Good as an override. Does not scale |
| L. Filtered view for each reader (character view, ME audience lists, Kindroid group memory) | The same memory with a list of who may see each item, or one memory for each reader | The writer marks the audience. The system filters at prompt time | A filter before the prompt | A filter has no cost. The marking costs a call | Measured: the character view cut a leak from 60.7 to 0 points in short template scenes. ME and Kindroid: claim | Direct. This is the only family built for who knows what |
| Control: full context | The whole history | None | None | The highest tokens. 26k tokens and 17 s p95 on LoCoMo | Measured: the best accuracy in the Mem0 and Zep papers when it fits (72.9 J on LoCoMo; 94.4 to 98.0% on DMR). LongMemEval_S: 30 to 60% below the oracle | The upper limit while the history fits |
| ME (Marinara Engine) | Scene records with an audience list, a summary and per-Character recollections, plus embeddings; separate lorebook entries from an agent | Scene check every 5 Messages, a summary call at scene end, an embedding per record | Markers in the preset; audience match | One cheap call per 5 Messages, one summary per scene | No measured result found. Details: ticket 01 and the earlier survey | By design, per Character. Unmeasured |

Not covered: memory in model weights (MemoryLLM, M+). The MemoryAgentBench authors say it is mostly academic and weaker than text memory (known). It does not apply to a hosted model.

### 1.1 One short paragraph for each family

**A. Rolling summary.** The Wang et al. paper (arXiv 2308.15022) feeds the old memory and the next chunk to the model and asks for a new memory. Known: the abstract says that it helps consistency on a public long-chat dataset and complements long-context and retrieval models. Known: LoCoMo and DMR show that a summary loses detail. ST Summarize and AI Dungeon belong here (earlier survey).

**B. Hierarchical summary.** RAPTOR clusters chunks, summarizes each cluster and repeats, then retrieves from every level. ReadAgent splits a text into episodes, writes a gist for each, and lets the model open the original page. Both were tested on document QA, not on chats (known: abstracts). Re3 uses the same shape inside a prompt: outlines of old sections, a summary of recent passages, and the last passage in full. Kindroid describes a "cascaded" memory that keeps less detail for older events (claim).

**C. Embedding retrieval.** The simplest strong baseline. LoCoMo and LongMemEval both test it. LongMemEval says that round-level units beat session-level units. LoCoMo says that retrieval over extracted observations beat retrieval over raw turns, and that more retrieved items hurt. ST Vector Storage and ME Memory Recall belong here.

**D. Extracted facts.** Mem0 extracts facts from each message pair, with the last 10 messages and a rolling summary as context. An LLM then picks ADD, UPDATE, DELETE or NOOP for each fact, against the 10 most similar stored facts. Known: this is a design in a vendor paper. Inferred: the DELETE rule is the weak point for a story, because a story needs the old value ("the debt was paid") as well as the new one.

**E. Temporal graph.** Zep keeps three layers: raw episodes, entities and facts with valid time, and community summaries. Graphiti is the open engine. Known: the paper reports lower latency and higher accuracy than full context on LongMemEval_S. Known: the tables are by a vendor, and the LoCoMo number for Zep is disputed (section 2.5). AriGraph (IJCAI 2025) builds a similar graph for an agent in a text game, with episodic nodes that link to the facts (known: abstract; I did not read the results).

**F. Self-edited memory.** MemGPT gives the model tools to move data between the prompt and storage. Letta calls the prompt part "memory blocks" with a size limit and a description. A "sleep-time" agent runs between turns and rewrites blocks. Known: the earlier survey covers MemGPT. Known: Letta's own blog reports that a plain-file agent reached 74.0% on LoCoMo. This is a vendor result and is not replicated.

**G. Linked notes.** A-MEM (NeurIPS 2025) builds a Zettelkasten: each note has keywords, tags and links, and a new note can change the text of old notes. Known: the abstract. The Mem0 paper's re-run gives A-Mem the lowest J score (48.38). I do not know why the two reports differ.

**H. Reflection.** Generative Agents (Park et al.) is in the earlier survey. LoCoMo built its dialogues with this architecture. Known: LoCoMo's text says that the generator agents use a session summary, observations and reflection.

**I. Narrative plan and state.** Re3, DOC, RecurrentGPT and NstAgent keep a plan and a state for the writer, and not for a reader's question. Section 3 covers them.

**J. Agentic search.** The model decides what to look up. Letta's file test and TunnelVision are the two examples. Known: the Letta number is a vendor claim. The MemoryAgentBench authors say that agentic memory systems stay limited by what retrieval can reach.

**K. Hand-written memory.** Every product in section 6 has a place where a person writes or pins text. Known: this is a claim of the products. Inferred: it is the one family that cannot be wrong about a fact, because a person wrote it. It covers only what the person thinks to write.

**L. Filtered view.** Section 4.
## 2. Evidence

### 2.0 The benchmarks in one table

| Benchmark | What it measures | Data | Questions come from | Score |
|---|---|---|---|---|
| LoCoMo (Maharana 2024) | Memory in very long two-person chats | 50 synthetic dialogues, human-edited | Persona and dated event graph; answers copied from the dialogue | F1; recall of the evidence turn |
| LongMemEval (Wu 2025) | Five abilities of a chat assistant, including update and time | 500 questions, each in a chat of 115k or 1.5M tokens | Human-written questions on an attribute ontology; evidence hidden in simulated sessions; filler sessions | LLM judge for each question type; Recall@k |
| MemoryAgentBench (Hu 2026) | Retrieval, learning at test time, long-range understanding, forgetting | Rebuilt long-context sets, plus EventQA and FactConsolidation | MQuAKE edit pairs for forgetting; a novel for events | Accuracy, F1 |
| DMR (MemGPT) | One fact from a short multi-session chat | 500 chats of about 60 messages | From Multi-Session Chat | LLM judge |
| FANToM (Kim 2023) | Who is aware of what | Multi-party chats where speakers join and leave | Facts turned into belief, answerability and access questions | Accuracy, and a strict "all question types" score |
| Curse of Knowledge (2026) | Whether a Character's speech uses what it cannot know | 480 twin scenes | Twin pairs that differ in one hidden fact | Net use: target minus foil |
| ConStory-Bench (2026) | Contradictions in a long generated story | 2,000 prompts, 8 to 10k words each | Not a question set. A checker finds contradictions | Errors per 10,000 words |

Known: the five benchmarks that ask questions differ in who writes the evidence. LoCoMo and ConStory use model output with human edits or an LLM checker. LongMemEval uses human questions and simulated evidence sessions. The Curse benchmark uses templates.

### 2.1 LongMemEval (known)

Source: Wu et al., arXiv 2410.10813 (ICLR 2025), and the official repository `xiaowu0162/LongMemEval`. Read 2026-11-03.

| Item | What the source says |
|---|---|
| Measures | Five abilities: information extraction, multi-session reasoning, knowledge update, temporal reasoning, abstention. |
| Size | 500 questions. Seven question types: single-session-user, single-session-assistant, single-session-preference, multi-session, knowledge-update, temporal-reasoning, and abstention. |
| Built by | An ontology of 164 user attributes in five groups. An LLM (Llama 3 70B Instruct) writes a background paragraph for an attribute and proposes seed question-answer pairs. Human experts filter and rewrite all questions. They split each answer into one or more evidence statements, with optional timestamps. |
| Evidence session | Each evidence statement goes into one task-oriented chat session, made by LLM self-chat. The simulated user states the fact in an indirect way (it asks about car insurance and reveals the new car). Humans check that the evidence is present, move it to varied positions, and rewrite time mentions in a colloquial way. |
| Haystack | Per question, the pipeline samples unrelated sessions and inserts the evidence sessions in the middle. Filler comes from self-chat sessions on other non-conflicting attributes, and from ShareGPT and UltraChat. All sessions get a plausible timestamp. |
| Standard sizes | `LongMemEval_S`: about 115k tokens, about 40 sessions. `LongMemEval_M`: 500 sessions, about 1.5M tokens. An oracle file holds the evidence sessions only. |
| Protocol | The system receives the sessions one at a time (online), then gets the question with its date. |
| Score | An LLM judge (`gpt-4o-2024-08-06`) with one prompt for each question type. The paper reports more than 97% agreement with human experts. The knowledge-update prompt accepts an answer that holds old information with the updated answer. The temporal prompt does not penalize an off-by-one error in days. The abstention prompt expects a refusal. Retrieval is scored apart: Recall@k and NDCG@k against labelled evidence turns (`has_answer`) and evidence sessions. |
| Controls in the paper | "Offline reading" (the whole history in the prompt) and "oracle" (evidence sessions only). |

Results (known, measured by the authors):

- Long-context LLMs lose 30% to 60% of accuracy on `LongMemEval_S`, compared to the oracle setting.
- A pilot with humans on 97 questions and short histories (3 to 6 sessions): offline reading with GPT-4o scores 0.918. ChatGPT memory (GPT-4o) scores 0.577. Coze (GPT-4o) scores 0.330. These are product systems in August 2024. The authors say ChatGPT overwrote key facts, and Coze often did not record facts given in an indirect way.
- Key expansion with extracted user facts raises Recall@k by 9.4% and QA accuracy by 5.4%. Time-aware query expansion raises recall on temporal questions by 6.8% to 11.3%. Chain-of-Note with a structured format raises accuracy by up to 10 points. Storing rounds (one user message and the answer) works better than whole sessions. Reducing the history to facts only loses information and hurts the total score, but helps multi-session questions.
- When the reader answers right, retrieval was right about 90% of the time. The other 10% are mostly knowledge-update questions, where the retriever found the new fact and missed the old one.

### 2.2 LoCoMo (known)

Source: Maharana et al., arXiv 2402.17753 (ACL 2024). Read 2026-11-03 (HTML version).

| Item | What the source says |
|---|---|
| Measures | Memory over very long chats between two people: question answering, event summarization and multimodal dialogue generation. |
| Size | 50 dialogues. About 300 turns and 9K tokens on average, over up to 35 sessions. |
| Built by | Two LLM agents (`gpt-3.5-turbo`). Each gets a persona (from MSC, expanded by the LLM) and a temporal event graph of up to 25 events over 6 to 12 months with causal links (`text-davinci-003`, in batches of 3). The agents follow the Generative Agents design: a session summary, observations and reflection. Before each session the agent also gets the events that fall between the last session and this one. Human annotators then edit the chats for long-range consistency. They edited about 15% of the turns. |
| Questions | Five types: single-hop, multi-hop (several sessions), temporal, open-domain (needs outside knowledge), adversarial (a false premise; the right answer is "unanswerable"). Answers are copied from the dialogue where possible. Each question carries the IDs of the turns that hold the answer. |
| Score | F1 after normalization (a partial match), for each type. Recall@k of the evidence turn for retrieval. Event summarization: FactScore-style precision and recall of atomic facts against the event graph. |
| Controls | No memory (context truncated), long-context (`gpt-3.5-turbo-16k`), and RAG with three units: raw dialog turns, observations, and session summaries. Human score. |

Results (known): `gpt-4-turbo` with a short context scores 32.4 overall. Humans score 87.9. Long-context models lose on adversarial questions (2.1 F1 for `gpt-3.5-turbo-16k`, against 70.2 for `gpt-4-turbo` with 4K). RAG over observations helps (about 5 points over the plain chat log with top 5). More retrieved items hurt. RAG over session summaries does not help although its recall is high; the authors blame information loss in the summary. Temporal questions are the hardest. I did not read the number of questions for each type (the appendix table did not come through).

### 2.3 Mem0 on LoCoMo: a vendor paper (known: what the paper says; claim: that Mem0 is best)

Source: Chhikara et al. (Mem0 team), arXiv 2504.19413. Read 2026-11-03.

- Setup: 10 LoCoMo conversations, about 26,000 tokens and about 200 questions each. (The LoCoMo paper says 9K tokens for the 50-dialogue set. The two sizes differ in the sources. I do not know why.) The authors drop the adversarial category, because the released data had no ground truth for it. Metrics: F1, BLEU-1, and an LLM-as-a-Judge score J, with 10 judge runs and the mean and one standard deviation. Token cost and latency are also reported.
- Overall J: full context 72.90; Mem0 graph variant 68.44; Mem0 66.88; Zep 65.99; LangMem 58.10; OpenAI memory 52.90; A-Mem 48.38; best RAG 60.97 (chunk 256, k=2). Memory tokens per question: full context 26,031; Mem0 1,764; Mem0 graph 3,616; Zep 3,911. Total p95 latency: full context 17.1 s; Mem0 1.44 s.
- Known: in this paper the full-context run scores higher than every memory system. The authors' case is cost: 91% lower p95 latency and more than 90% fewer tokens.
- Known: the paper says OpenAI memory scored below 15 on temporal questions because most generated memories had no timestamp.
- Caution (inferred): the authors sell Mem0, and they ran the baselines themselves. Section 2.5 shows a dispute about the Zep number.

### 2.4 Zep on DMR and LongMemEval: a vendor paper (known: what the paper says)

Source: Rasmussen et al. (Zep team), arXiv 2501.13956. Read 2026-11-03.

- DMR (Deep Memory Retrieval, from MemGPT, a 500-conversation subset of Multi-Session Chat): each conversation has 5 sessions of up to 12 messages (about 60 messages) and one question. An LLM judge compares the answer with the golden answer. Scores with `gpt-4-turbo`: recursive summarization 35.3% (as MemGPT reported), conversation summaries 78.6%, MemGPT 93.4% (as reported), full conversation 94.4%, Zep 94.8%. With `gpt-4o-mini`: summaries 88.0%, full conversation 98.0%, Zep 98.2%. The authors call DMR too small and too easy: the whole chat fits in the context, and a full-context run beats or equals the memory systems.
- LongMemEval_S (about 115k tokens): Zep raised accuracy by 15.2% (`gpt-4o-mini`) and by 18.5% (`gpt-4o`) over the full-context baseline, and cut latency by about 90%. Gains were largest on preference, multi-session and temporal questions. Zep lost 17.7% (`gpt-4o`) and 9.06% (`gpt-4o-mini`) on single-session-assistant questions. I could not read the tables of absolute scores in the HTML copy.
- Known: the authors could not run MemGPT on LongMemEval. They say the framework could not ingest an existing message history.

### 2.5 Disputed numbers, and a plain-file control (known: the disputes exist; the true Zep number is unknown)

- The Mem0 paper reports Zep with J = 65.99 on LoCoMo. Zep says the Mem0 run used a wrong Zep setup. Zep's blog "Is Mem0 Really SOTA in Agent Memory?" (read 2026-11-03) reports a corrected Zep score of 75.14 ± 0.17. An issue in `getzep/zep-papers` (#5) gives 58.44 for Zep, against an earlier Zep claim of 84. The third-party README `a40-labs/memory` says the reason was that the adversarial category counted in the numerator and not in the denominator. That README lists the three figures. I did not run any of it. Inferred: a LoCoMo number from a vendor is not comparable with another vendor's number, unless the setup is identical.
- Letta ran a plain-file agent on LoCoMo (blog "Benchmarking AI Agent Memory: Is a Filesystem All You Need?", 2025-08-12, read 2026-11-03). An agent with `gpt-4o-mini` that stores the chats in files and searches them with file tools (grep, search, open) scored 74.0%. Letta compares it with 68.5% that Mem0's graph variant reported. Letta also says it could not find how the Mem0 team loaded LoCoMo into MemGPT. Known: this is a claim of a vendor and not a replicated result. Inferred: LoCoMo does not separate a simple store from a complex memory.

### 2.6 MemoryAgentBench (known)

Source: Hu, Wang, McAuley, arXiv 2507.05257 (ICLR 2026). Read 2026-11-03.

| Item | What the source says |
|---|---|
| Measures | Four abilities of a memory agent: accurate retrieval, test-time learning, long-range understanding, selective forgetting. |
| Protocol | Incremental. The data go in as chunks inside simulated user messages that say "please memorize this". Then questions follow. One long input feeds many questions (for example five chats of about 355K tokens and 300 questions, rebuilt from LongMemEval). |
| Selective forgetting set (FactConsolidation) | Built from the counterfactual edit pairs of MQuAKE. Each pair holds a true fact and a rewritten contradicting fact. The new fact comes after the old one. Many pairs are joined into contexts of 6K, 32K, 64K and 262K tokens. The questions are single-hop ("which country was X created in?") or multi-hop. The prompt says: facts have serial numbers, newer facts have larger numbers, and the newest wins. |
| EventQA | The agent reads a novel. Then it gets up to five earlier events and must pick the next event among candidates. An automatic pipeline builds it. |
| Score | Accuracy (multiple choice, exact), F1 for summaries, Recall@5 for recommendation. |
| Result | All methods fail on multi-hop forgetting (at most 28%). Only long-context agents do fairly on single-hop. A reasoning model (`o4-mini`) scores 80.0 on the 6K multi-hop set and 14.0 on the 32K set. RAG and the product memory agents lose on tasks that need the whole context. |

Inferred: the FactConsolidation method (a fact, then a contradicting later fact, then a question on the new state) is cheap to copy for a story: "the debt was 740,000, then paid" is one such pair.

### 2.7 What the results say across families (inferred from 2.1 to 2.6)

- Full context scores highest when it fits (Mem0 paper, Zep paper). A memory system must beat it on cost, or on a history that does not fit.
- The gains of memory systems are largest on multi-session, preference and temporal questions (Zep, LongMemEval optimizations). They are smallest or negative on single-session questions (Zep: −17.7% on single-session-assistant).
- Vendor papers rank their own system first. The reported numbers for one system differ by 17 points across reports (Zep on LoCoMo).
- The number of runs is small in most papers. The Mem0 paper is the one that reports a standard deviation (10 judge runs).

## 3. Stories

Question: how do methods for long generated stories keep their facts and their plan? In all four, a model writes the whole story, and the memory serves the writer. This is close to our engine: the Narrator writes, and the memory must serve it.

| Method | Facts | Plan | Evidence |
|---|---|---|---|
| Re3 | Character descriptions with retrieval; an attribute dictionary with an entailment check | Setting, Characters and a numbered outline, made once | Human raters, +14 coherence, +20 relevance against a rolling window. Edit: little effect |
| DOC | Retrieved Character descriptions, plus one new inferred fact for each Character at each outline item | A hierarchical detailed outline; a controller keeps each passage to its outline item | Human raters, +22.5 coherence over Re3 |
| RecurrentGPT | Short-term memory text (rewritten each step) and a vector store of paragraph summaries | A plan of 3 to 5 sentences for the next paragraph, made at each step, and editable by a person | Human raters prefer it over rolling ChatGPT, Re3 and DOC; the gap grows with length |
| NstAgent | A structured narrative state of Characters, past events and future requirements | Future requirements in the state | Abstract only: consistency does not fall from 10K to 100K words |
| ConStory | A benchmark and checker, not a method | None | Errors grow with length |

**Re3** (Yang et al., arXiv 2210.06774, EMNLP 2022; read 2026-11-03). Known.
- Plan: a premise becomes a setting, up to three Characters with descriptions, and a numbered outline.
- Draft: each passage comes from a prompt that is rebuilt every step. It holds (1) "Relevant Context": the plan and the Character descriptions, chosen by dense retrieval for the latest passage, (2) outlines of earlier sections, (3) a short summary of the last few passages, (4) the current outline point, (5) the last passage verbatim. The Character list grows: a named-entity pass finds new entities in each passage, and the model writes a description.
- Rewrite: several continuations are reranked by two trained classifiers (coherence with the story, relevance to the outline point).
- Edit: the module keeps an attribute-value dictionary for each Character (age, job, relations). It extracts new facts from each passage, checks them against the dictionary with an entailment model, and edits the passage when a value conflicts. The authors say that checking all statement pairs gives "a sea of false positive contradictions", so they check only Character attributes.
- Measured: human raters (Mechanical Turk, pairwise, about 2000 to 2500 words, 100 premises) judged Re3 stories coherent more often by up to 14 points and premise-relevant by up to 20 points, against a rolling-window GPT-3 baseline. The ablation says the Plan and Rewrite modules are critical, and the Edit module "makes little difference". The generator was GPT-3 (175B); the authors report that a 13B generator was noticeably less coherent.

**DOC** (Yang et al., arXiv 2212.10077, ACL 2023; abstract and one passage read 2026-11-03). Known: a detailed outliner builds a hierarchical outline. A detailed controller steers each passage to its outline item. Human raters preferred DOC over a Re3 baseline: plot coherence +22.5 points, outline relevance +28.2, interestingness +20.7. DOC infers one new fact about each Character each time the Character appears in the outline, drops facts that an earlier fact entails, and uses the retrieved Character descriptions when it drafts the passage. The authors say Re3 "struggled" with Character development over time. I did not read the controller details.

**RecurrentGPT** (Zhou et al., arXiv 2305.13304, 2023; full text read 2026-11-03). Known:
- Each step writes one paragraph (200 to 400 words), a plan of 3 to 5 sentences for the next paragraph (three candidate plans), and an updated short-term memory (10 to 20 sentences, rewritten each step, it drops what is no longer relevant). The long-term memory holds a summary of each paragraph in a vector store. The plan is the query for the semantic search that puts a few paragraphs into the prompt.
- A person can read and edit the memory and the plan. The paper presents this as interpretable and interactive, and it shows a choose-a-plan interactive fiction as a use case.
- Measured: human pairwise comparison, 20 novels per genre, three raters, novels of 3000 to 6000 words, ChatGPT as the backbone for all methods. RecurrentGPT is preferred for coherence over rolling ChatGPT (about 85% to 15%), over Re3 (about 62% to 70% against about 25% to 38%), and over DOC (about 56% to 66%). The gap grows with length. Caveats (inferred): the authors re-implemented Re3 and DOC with ChatGPT and removed the DOC controller. The stories are short compared with a 100-Cycle Game. The metric is a reader's overall preference, and not a count of contradictions.

**ConStory-Bench and ConStory-Checker** (Li et al., ACL Findings 2026; project page read 2026-11-03). Known.
- Method: 2,000 prompts from seven public corpora (generation, continuation, expansion, completion), with outputs of 8,000 to 10,000 words. A checker (`o4-mini`) works in stages: it extracts spans per error category, pairs the spans and labels each pair Consistent or Contradictory, and writes an evidence chain with quoted text and character offsets. Metrics: CED (errors per 10,000 words) and GRR (a rank inside each prompt group).
- Taxonomy: 5 categories and 19 subtypes. Timeline and plot logic: absolute-time, duration and simultaneity contradictions, causeless effects, causal-logic violations, abandoned plot elements. Characterization: memory contradictions, knowledge contradictions, skill fluctuations, forgotten abilities. World-building: rule, social-norm and geography violations. Fact and detail: appearance, name and quantity mismatches. Narrative and style.
- Result: the best model (GPT-5 with reasoning) scores CED 0.113. The paper says that errors grow about linearly with output length, and that error-bearing spans have higher entropy and perplexity. I did not read the tables for every model.
- Inferred: this taxonomy has a "Duration" subtype (our "no story clock" fault), a "Knowledge contradictions" subtype (our "who knows what" fault) and "Quantitative mismatches" (the debt 740,000 and "eleven"). The checker method can score a stored story with a judge that quotes evidence.

**NstAgent** (arXiv 2609.35759, posted 2026-09-28, under review; abstract only). Known: a training-free agent that tracks a structured narrative state of Characters, past events and future requirements, for stories of 10K to 100K words. The authors say that consistency and quality do not drop noticeably as the story grows. I did not read the method or the numbers. Unknown: how it keeps the state.

**Inferred.** Three points carry over to our engine. (1) A plan or a short-term text that a person can read and edit is a published design (RecurrentGPT). (2) A per-Character attribute dictionary is a published design for facts about a Character (Re3), but its checker had too many false positives, and it had little effect. (3) The Re3, DOC and RecurrentGPT results are judgments by raters on stories of 2,000 to 6,000 words. They do not measure contradictions. None of them tests the length of a 100-Cycle Game.

## 4. Who knows what

| Method | What is kept apart | How | Evidence |
|---|---|---|---|
| Generative Agents (earlier survey) | One memory stream for each agent | Each agent writes observations from its own perceptions | Measured by ablation in the paper; leaks between agents not tested (inferred) |
| ME Advanced Memory (earlier survey, pinned commit) | Scene records with an audience list; one first-person recollection for each Character | The summary call names the audience. Recall passes a record only if the audience matches. A leak check compares private and wider text | Known: code read. No measured result |
| Kindroid group chat (claim) | Backstory and key memories for each Kindroid | "Each Kindroid draws from its own backstory, key memories, etc." A toggle "Share Short Term Memory" | Claim only |
| Re3 attribute dictionary | Facts for each Character | A dictionary of attribute and value for each Character, checked for contradictions | Measured: little effect (section 3). It tracks what is true of a Character, and not what a Character knows |
| Zep | The speaker is extracted as an entity ("ALWAYS extract the speaker/actor as the first node") | Facts are edges between entities. There is no access list | Known: prompt in the paper's appendix. Inferred: it can say who did a thing. It cannot say who knows it |
| ReverieMem, MENTOR (named by the Curse paper) | Facts tagged by which Characters can see them (ReverieMem). A separate memory for each role (MENTOR) | Filter at retrieval time | I did not read them |
| Character view (Curse paper) | What a Character can still observe | The system replaces each inaccessible event with its observable part, before the prompt | Measured: leak 60.7 to 0 points; permitted use +0.6 points (short template scenes) |

Known: LoCoMo says that long-context models "are especially prone to misassigning dialogs or events to the wrong speaker". Inferred: the engine's Narrator sees the whole history, and the fault "Carla knows about Louise's wax" is the same shape as the leak in the Curse paper.

Inferred, from FANToM and the Curse paper together: a Character's access can be derived from the Characters present in each Swipe state, as in FANToM (join and leave). The test of a candidate can then ask the list questions "who knows X?" and the content question "what does Carla say about X?". The first checks the memory. The second checks the prompt.

**FANToM** (Kim et al., EMNLP 2023; project page and abstract read 2026-11-03). Known.
- Method: multi-party conversations on one topic (pets, family). They start with two or three speakers. Speakers join and leave while the talk goes on. The speaker who was away comes back and does not know what the others said. This gives information asymmetry by design.
- Questions: from each fact, the authors build several questions that need the same reasoning, "who is aware of the information". They are BeliefQ (choice and free answer), AnswerabilityQ (list and yes/no: who can answer?), InfoAccessQ (list and yes/no: who has access to the information?). They also score a fact question and an "all question types" score that needs every question about one fact to be right. A control task has no information asymmetry.
- Result: all tested LLMs score far below humans, with or without chain-of-thought. Models do better on BeliefQ[Choice] than on the list questions, although the list questions are the prerequisite. The "all question types" score drops sharply. The authors call good single-question results "illusory". On list questions the typical error is to include a Character who is unaware.
- Inferred: the question forms "who has access?" and "who can answer?" are a model for the who-knows-what test of ticket 04, and the "all question types" rule gives a strict score.

**Hi-ToM** (Wu et al., Findings of EMNLP 2023; abstract only, 2026-11-03). Known: a benchmark of 1.2k higher-order theory-of-mind questions (what A thinks B thinks C believes), in short generated stories, some with deception. I did not read the results.

**The Curse of Knowledge in Role-Playing Language Models** (arXiv 2610.17327, posted 2026-10-12; a preprint, read in full on 2026-11-03). Known: what the paper reports. It is not peer reviewed (inferred).
- Method: 480 scenes in twin pairs. The twins differ only in the hidden fact (an object and a place). A rule-based scorer counts a reply that names its own version's value (target) and the twin's value (foil). The net use is target minus foil, in percentage points. It cancels guessing. A control makes the same content accessible (the character knows the language, or hears the speech) and measures "permitted use". Two annotators agreed with the scorer on 773 of 782 replies. Channels: unheard speech, speech in an unknown language, a hidden action, a private message.
- Result: 11 open-weight models (at most 40B parameters) use content that the character cannot hear or understand at 67 and 72 points above the baseline, although most of them state correctly what the character perceives. Five of six proprietary models rarely leak. Compliance depends on the input format: a change of only the state layout moved one open model from 0 to 62 points, and combined changes moved two proprietary models from near 0 to 13 and 21 points. Leaked facts persist after four more neutral exchanges and change where characters search (49 to 63 points). Writing the line as "the scene's author" leaks nearly as much as role-play. Thinking rarely helps.
- Fix that worked: a "character view" that the system builds outside the model. It drops each inaccessible event and replaces it with what the character can still observe ("Alex appears to put something away"). Leak fell from 60.7 to 0 points, and permitted use changed by +0.6 points. A prompt that tells the model to filter itself ("list only what X can perceive") cut the leak by 64.7% on average, but it works only when the list is clean, and the prompted self-filter also dropped permitted content (28.6 to 41.2 points in two channels). The authors' limits: short, template scenes with one hidden fact. Long scenes with many Characters were not tested.
- The paper cites two systems that I did not read: ReverieMem (tags the facts of a novel by which characters can see them, so retrieval gives a Character only facts in its view) and MENTOR (a separate memory for each role, so that information does not flow between roles). Known: the paper names them. Unknown: their results.

## 5. Time

| Method | How it records time and order | How it handles a fact that changes | Evidence |
|---|---|---|---|
| Zep / Graphiti | Two timelines. Event time T: when the fact held. Transaction time T′: when the system learned it. Each edge stores `t_valid`, `t_invalid`, and the two system timestamps. Each message carries a reference timestamp. The extractor uses it to turn "two weeks ago" into a date. | A new edge that contradicts an old edge sets the old edge's `t_invalid` to the new edge's `t_valid`. The old edge stays. New information wins. The context string shows "FACT (Date range: from - to)". | Known: design (paper). Measured: Zep gains on LongMemEval temporal questions; the table is not in the HTML copy I read. |
| Mem0 | Memory text, no required time field in the base design. | ADD, UPDATE, DELETE or NOOP, chosen by the LLM against the 10 most similar memories. The graph variant marks a relation obsolete and does not delete it, "to enable temporal reasoning". | Known: design. Measured (vendor): J 55.5 base and 58.1 graph on LoCoMo temporal questions. |
| LongMemEval methods | Each session has a timestamp, and the question has a date. | The knowledge-update type asks about the new state. | Measured: time-aware query expansion (index the time range of facts, and narrow the search by the time in the question) gave +6.8 to +11.3 points of recall on temporal questions with a strong LLM. |
| LoCoMo data | Sessions have dates. The event graph has dates and causal links. At generation time the agent gets the events that fall between the last session and the next. | Not tested as a type. | Measured: temporal questions are among the hardest. The paper says that models lag humans by 73% on them. |
| OpenAI memory (in the Mem0 test) | The extracted memories lacked timestamps. | Not tested. | Measured (vendor test): below 15 J on temporal questions, because most memories had no timestamp. |
| MemoryAgentBench | Facts carry serial numbers, and the prompt says that a larger number is newer. | The new fact contradicts the old one. The agent must use the newest. | Measured: multi-hop at most 28% for all methods. One reasoning model scored 80.0 at 6K tokens and 14.0 at 32K. |
| Re3 Edit | Not time. A dictionary of attribute values for each Character. | A new value that contradicts the old one is flagged. The new passage is edited to match the old value. | Measured: the Edit module "makes little difference". |
| Generative Agents (earlier survey) | Each memory has a creation time and a last-access time. Recency decays by 0.995 for each game hour. | No update rule. Old and new memories coexist and are ranked. | Measured by ablation in the paper (see the earlier survey). |
| MemoryBank | A forgetting curve (Ebbinghaus). Memory strength falls with elapsed time and rises on recall. | Forgets by decay. | Known: the abstract describes it. I did not read results. |
| Kindroid (claim) | Older events keep less detail. | Learned Context drops "the least important older items" when full. | Claim only. |

**Known.** Only Zep has a stored valid time for each fact in the sources I read, and only Zep's design keeps the old value after a change. Most other methods overwrite or delete. Two benchmarks test the update case: LongMemEval (knowledge update) and MemoryAgentBench (FactConsolidation).

**Known.** In the sources, "time" is the time of the chat: the real date of each session. I found no source that keeps a clock inside the fiction, for example "day 2, evening" that advances with each Narration. LoCoMo comes closest. Its dialogues are generated from an event graph with dates, and the agent sees the events between sessions. I do not know of a method that tracks in-story elapsed time in an interactive story.

**Inferred.** The debt example ("740,000, then paid") is a knowledge-update case. A method that keeps both values with a valid-time range answers "what was the debt on day 1?" and "what is it now?". A method that overwrites answers only the second.

## 6. Roleplay practice

### 6.1 SillyTavern extensions other than Summarize and Vector Storage (known: what the repositories say; no measured result found)

The GitHub API gave the last push date on 2026-11-03.

| Extension | What it does (from its README) | Last push |
|---|---|---|
| `aikohanasaki/SillyTavern-MemoryBooks` | The user marks a scene (or the extension picks one). A model summarizes it. The summary is saved as a lorebook entry with extracted keywords. Older chat messages can be hidden. Earlier memories can go back to the writer as context. Group chats are supported. | 2026-11-01 |
| `qvink/SillyTavern-MessageSummarize` | Summarizes each message on its own, instead of one rolling summary. Two tiers: short-term (recent message summaries, up to a token limit; older ones are dropped or promoted) and long-term (summaries of messages that the user marked by hand, up to a second limit). | 2026-07-29 |
| `Coneja-Chibi/TunnelVision` | Replaces lorebook keyword triggers. The model gets a tree index of the lorebook and walks it with tool calls. A second small "sidecar" model can pre-load the context before the main model writes. It can also write scene summaries and re-organize entries. | 2026-10-11 |
| `SpicyMarinara/rpg-companion-sillytavern` | Tracks stats, scene and Characters in a panel (state, not memory). The README marks it "deprecated" and says the work moved to Marinara Engine. | 2026-08-29 |

Inferred: these are the same families as in section 1: a summary (Message Summarize, Memory Books), retrieval by key (Memory Books, TunnelVision), a model that picks from an index (TunnelVision), and state in a panel (RPG Companion). None of them has a per-Character memory, as far as the READMEs show. I did not read the code. I found no benchmark for any of them.

### 6.2 Roleplay products: what the vendor says (claims, not measured results)

| Product | Claim (source, access date 2026-11-03) |
|---|---|
| Character.AI | The blog of 2026-05-21 ("Smarter Memory for Smarter Chats", `blog.character.ai/memory/`) names three parts. Story Memory: text that the user writes, plus messages that the user pins; "protected" from being tidied. Facts: auto-captured facts in separate tabs for the Persona, the Character and side characters; the user can edit, disable or remove them; paid feature; copied to a new chat on request. Memory Usage: a view of what fills the memory. The blog says older context is "tidied" in the background. It gives no method and no measurement. |
| Kindroid | The docs (`kindroid.ai/v2/docs/memory/`) name five systems. Persistent: backstory, key memories, recent chat. Cascaded memory: a hierarchical medium-term memory, with lower fidelity for older events (subscribers only). Learned Context: three capped fields (800, 1,200 and 800 characters) that the model keeps up to date; "drops the least important older items" when full. Long-term memory: automatic consolidation at intervals, recalled when an AI judges it relevant. Journal: user entries with up to 8 keyphrases, recalled by key match. The docs say that recalled long-term memory is "unreliable" and "may not be always recalled". The user can see which memories a reply used and can deprioritize one. In group chats each Kindroid "draws from its own backstory, key memories, etc." and not from the others'. Group settings let the user switch on "Share Short Term Memory". |
| Nomi | The vendor posts (2024-12-12, 2025-10-09) name short, medium and "infinite" long-term memory, a backstory, shared notes, an Identity Core (what the Nomi learns about itself), and a Mind Map (a view of the people, places and topics that a memory links). I read the titles and the lead text only. |
| Replika | The help center says some memories are visible and editable in a Memory tab, and others come from "a deeper system". It also says that chat history is shown for four months only. |

Known: all four are claims. The vendors give no benchmark. Inferred: the common shape is a mix of hand-written fields (Story Memory, key memories, journal), model-written facts with a size cap, and a retrieved long-term store. Two products let the user see and edit the memory, which matches the owner's UI goal. Kindroid is the one product in this list that says how memory works in a group: one memory for each Kindroid, and an option to share the short-term memory.

## 7. Open problems

Each item gives the source. "Known" means that the source says it. "Inferred" means that I drew it from the source.

1. **Updating and forgetting is unsolved.** MemoryAgentBench: all tested methods fail on multi-hop forgetting (at most 28% accuracy). Only long-context agents do fairly on single hop. A reasoning model falls from 80.0 at 6K tokens to 14.0 at 32K. Known.
2. **Time is the weakest question type.** LoCoMo says that temporal questions are the hardest, and models lag humans by 73% there. In the Mem0 test OpenAI memory fell below 15 J on temporal questions, because memories had no timestamps. LongMemEval needed a special time-aware query step to gain 7 to 11 points of recall. Known. Unknown: any method that keeps a clock inside a fiction.
3. **Full context often beats a memory system on accuracy.** In the Mem0 paper full context scores 72.9 and the best memory system scores 68.4. In the Zep paper full context equals or beats MemGPT and Zep on DMR. The case for memory is cost and latency, and long contexts degrade (LongMemEval: 30 to 60% below the oracle). Known. Inferred: for a 100-Cycle Game, the question is when the history no longer fits. The engine already cuts it (map Notes).
4. **A summary loses detail, and retrieval can miss the old value.** LoCoMo: summaries gave no gain although recall was high, and the authors blame information loss. LongMemEval: reducing the history to facts hurts the total score. In about 10% of right answers retrieval had found only the new value of a changed fact. Known.
5. **A reader still fails after correct retrieval.** LongMemEval: correct retrieval with a wrong answer makes 15% to 19% of all instances and 40% to 50% of the errors. The share is higher with a weaker reader. Chain-of-Note and a structured format gave up to 10 points. Known.
6. **The benchmarks are weak or disputed.** The Anatomy survey (arXiv 2602.19320) says that benchmarks are underscaled, metrics do not match semantic use, results depend on the backbone model, and the cost of memory upkeep is overlooked. Zep calls DMR too easy. Letta says that a plain-file agent beats specialized tools on LoCoMo, so LoCoMo may not separate designs. The Zep LoCoMo number is 58.44, 65.99 or 75.14 depending on who ran it. Known (each source is a vendor, except the survey).
7. **A Character's knowledge boundary is fragile.** FANToM: models do the list questions ("who knows?") much worse than the choice question, and the strict score drops. Curse of Knowledge: 11 open models leak at 67 to 72 points. Strong proprietary models mostly hold the boundary, but format changes break it. The fix that worked is to filter the input outside the model. The authors list short template scenes with one hidden fact as a limit. Known. Unknown: how this holds in a long story with many Characters.
8. **Consistency errors grow with length, and checking has false positives.** ConStory: errors grow about linearly with output length. Re3: comparing all statement pairs "can result in a sea of false positive contradictions", so its Edit module checks only Character attributes, and the Edit module "makes little difference". Known.
9. **No benchmark found for an interactive story with several Characters.** I found benchmarks for chats (LoCoMo, LongMemEval), for documents, for single long generations (ConStory) and for Character knowledge (FANToM, Curse). I found none with a player who acts, a Narrator and several Characters. DialSim (named in the LongMemEval paper) plays TV show characters, but I did not read it. Inferred: a story memory test has to be built, as ticket 04 plans.
10. **Product memory is not measured in public.** Character.AI, Kindroid, Nomi and Replika publish no benchmark in the sources I read. LongMemEval measured ChatGPT and Coze memory in a small human study (August 2024): ChatGPT (GPT-4o) 0.577 and Coze (GPT-4o) 0.330, against 0.918 for offline reading. Known. Roleplay products were not in that study.

## 8. Facts for the other tickets

### For ticket 04: make the memory test from the stored stories

All items below are known from the sources, except where marked inferred.

1. **Build the question set from labelled evidence.** LongMemEval stores for each question the evidence statements, the session IDs and the turn that holds the answer (`has_answer`). It then computes recall of the memory and accuracy of the answer apart. LoCoMo also stores the evidence turn IDs. Our stored stories give the same labels if ticket 03 lists the Message index where each fact first appears.
2. **Use question types, and score each type.** LongMemEval: single-session (user and assistant), preference, multi-session, knowledge update, temporal, abstention. LoCoMo: single-hop, multi-hop, temporal, open-domain, adversarial. Our three memory faults map to: a fact that left the history (single-hop and multi-hop), time (temporal), and who knows what (a FANToM-style access question).
3. **Use the same judge rules as LongMemEval for the tricky types.** One prompt for each type. The knowledge-update prompt accepts an answer that holds the old value as well as the new one. The temporal prompt does not penalize an off-by-one error in days. The paper reports more than 97% agreement of the judge with human experts. LoCoMo used F1 with normalization, and the Mem0 paper says F1 can score "born in July" high against "born in March". Use a rule or a short answer where possible, and a judge where not.
4. **Build abstention questions by changing a premise.** LongMemEval turns 30 questions into false-premise questions that must get "I don't know". Inferred: for who-knows-what, ask Carla about a fact that she never saw. The right answer is that she does not know. The twin method of the Curse paper gives a second way: two versions of the scene differ in one hidden fact, and a reply that uses the hidden fact of the other version is a guess. The net score removes guessing.
5. **Use the strict score for access questions.** FANToM asks the same fact in several forms (choice, list of who knows, list of who can answer, yes or no) and counts a fact as right only if all forms are right. Single-form scores overstate ability ("illusory").
6. **The controls the sources use.** No memory or truncated context (LoCoMo, MemoryAgentBench FIFO window). Full history (LongMemEval "offline reading"). Oracle: only the evidence (LongMemEval oracle file). The Mem0 paper adds RAG baselines with several chunk sizes. Inferred: the oracle control tells whether a question is solvable at all, and the no-memory control tells whether the question tests memory.
7. **Report noise.** The Mem0 paper ran the judge 10 times and gave the mean and one standard deviation. The MemoryAgentBench and Mem0 papers set temperature 0 where they could. Known: Zep's reply to Mem0 shows that the same system scored 58.44, 65.99 or 75.14 by setup. Inferred: ticket 04 should fix the prompt, the model and the setup of every candidate.
8. **Report cost.** Mem0: tokens of the memory in the prompt (1,764 for Mem0, 3,911 for Zep, 26,031 for full context), search latency and total latency, p50 and p95. The Anatomy survey says that upkeep cost is often overlooked.
9. **Insert the facts as a pair, to test a change.** MemoryAgentBench FactConsolidation: an old fact, a later contradicting fact, then a question on the new state; the prompt says later is newer. Inferred: a story pair is "the debt is 740,000 (Message A), the debt is paid (Message B), how much is owed now?"
10. **Check a whole story with a quoting checker.** ConStory-Checker extracts spans by category, pairs them, labels each pair, and quotes both spans with offsets. Inferred: this is the method of the judge in the Layout trials pilot, with a fixed taxonomy. Its categories include duration, knowledge and quantity contradictions.
11. **Caveat from the sources.** The questions of LongMemEval, LoCoMo and the Curse paper are synthetic or template-made. LoCoMo's answers are copied from the text, which favours exact match. I found no benchmark on a model that writes both sides of a story (inferred: our stored stories have this property).

### For ticket 05: choose the memory candidates to try

This asset does not choose. It gives the facts below.

1. **Cost per Cycle, from the designs.**
   - Rolling summary: one call per chunk (known).
   - Mem0: one extraction call for each message pair, one update call for each fact (inferred from the design).
   - Zep: several calls for each message (entity, resolution, fact, dedup, time, invalidation), plus community summaries (known).
   - ME: one cheap check call every 5 Messages, one summary call for each scene (earlier survey).
   - Embedding retrieval: embeddings only, no completion call (known).
   - Character view or audience filter: no model call at prompt time (known).
2. **Size in the prompt, measured.** Memory text per question on LoCoMo: Mem0 1,764 tokens, Mem0 graph 3,616, Zep 3,911, A-Mem 2,520, full context 26,031 (Mem0 paper, vendor).
3. **What each candidate can answer.** Summaries lose detail (LoCoMo, LongMemEval). Embedding retrieval depends on the match of the question (LongMemEval: key expansion with facts helped by 9.4 recall points). A fact store with DELETE loses the old value (inferred). A graph with valid time keeps both (Zep design). Only the filtered-view family targets who knows what.
4. **The simple baselines are strong.** Full context (72.9 J) beat every memory system in the Mem0 paper. Plain-file search scored 74.0% in the Letta blog. Round-level retrieval beat session-level in LongMemEval. Inferred: ticket 05 should include a plain retrieval candidate and the whole-story control.
5. **A fact written by a model needs a place for its source.** Re3 and Mem0 do not store the source Message. Zep keeps episodes and links each fact to its source (known). ME records keep their source Message IDs, and its lorebook entries pin the Swipe index (earlier survey). This matters for Retry and delete in our engine (inferred).
6. **Where a candidate writes, in the sources.** After each turn (Mem0, Zep, Re3), after each session (LoCoMo summaries, Mem0 summary refresh asynchronous), at scene end (ME, Memory Books), between turns by a second agent (Letta sleep-time), by the generating model itself (MemGPT, RecurrentGPT). Known: the Mem0 paper updates its summary asynchronously, so extraction does not wait for it.
7. **Whether a Character memory changes while the Character is absent.** I found no source that tests this. Kindroid's group setting "Update Individual Learned Context" lets a group chat update each Kindroid's own record, and "Share Short Term Memory" is a toggle (claim). ME writes a recollection only for Characters in the scene audience (earlier survey). Known: that is all I found. Unknown: the effect.

### For ticket 06: decide how the story keeps its time

1. **Known:** only Zep's design stores a valid-from and valid-to time on each fact, and keeps the old value after a change. Its time is real chat time plus a reference timestamp. It resolves relative dates ("two weeks ago") by that timestamp.
2. **Known:** LongMemEval gives each session a timestamp and the question a date. Its time-aware query expansion indexes the time range of facts and narrows the search by the time in the question (+6.8 to +11.3 recall points on temporal questions).
3. **Known:** LoCoMo generates dialogues from an event graph with dates, and gives the generating agent the events that fall between two sessions. It is the closest published design to a story clock. It is a data-generation design, and not a memory method.
4. **Known:** in the Mem0 test, memories without timestamps scored below 15 J on temporal questions.
5. **Known:** ConStory has duration, absolute-time and simultaneity subtypes in its taxonomy. This matches the "four days" and "two weeks" faults in the stories.
6. **I do not know** any method or benchmark that tracks elapsed time inside a fiction for an interactive story. This is a gap in what I found, not a proof that none exists.
7. **Inferred:** the two options that the sources suggest are (a) a clock value that the engine or a writer advances and stores per Swipe, so that the prompt carries "day 2, evening", and (b) a valid time on each stored fact in story time. The sources test neither in a story.

## 9. What I could not find

- The number of questions for each type in LoCoMo. The appendix table did not come through in the HTML copy.
- The absolute scores in the Zep paper for LongMemEval (tables missing in the HTML copy). I used the percentage gains that the text gives.
- The size difference between LoCoMo as the paper describes it (9K tokens a dialogue) and as the Mem0 paper uses it (about 26K tokens, 10 conversations). I do not know the reason.
- Any measured result for ME, Memory Books, Message Summarize, TunnelVision, Character.AI, Kindroid, Nomi or Replika.
- The methods and results of ReverieMem, MENTOR, DialSim, TimeChara and AriGraph (named or abstract only).
- The detail of the DOC controller, the NstAgent method, and the Hi-ToM results.
- Any method that keeps an in-story clock, and any benchmark for an interactive story with several Characters.
- I did not run any code, and I did not replicate any number.

## Sources

Access date: 2026-11-03 for all rows.

| Source | URL | Used for |
|---|---|---|
| Wu et al., *LongMemEval*, arXiv 2410.10813 (ICLR 2025), HTML v2 | https://arxiv.org/html/2410.10813v2 | 2.1, section 5 |
| LongMemEval repository (README, data format) | https://github.com/xiaowu0162/LongMemEval | 2.1 |
| Maharana et al., *LoCoMo*, arXiv 2402.17753 | https://arxiv.org/html/2402.17753 | 2.2, 4, 5 |
| Chhikara et al. (Mem0), arXiv 2504.19413 | https://arxiv.org/html/2504.19413 | 1, 2.3, 5 |
| Rasmussen et al. (Zep), arXiv 2501.13956 | https://arxiv.org/html/2501.13956 | 1, 2.4, 5 |
| Zep blog, "Is Mem0 Really SOTA in Agent Memory?" | https://www.getzep.com/blog/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/ | 2.5 |
| `getzep/zep-papers` issue 5 (Mem0 side) | https://github.com/getzep/zep-papers/issues/5 | 2.5 |
| `a40-labs/memory` README (third-party list of the three Zep figures) | https://github.com/a40-labs/memory | 2.5 |
| Letta blog, "Benchmarking AI Agent Memory: Is a Filesystem All You Need?" (2025-08-12) | https://www.letta.com/blog/benchmarking-ai-agent-memory/ | 2.5 |
| Letta blogs: sleep-time compute; memory blocks | https://www.letta.com/blog/sleep-time-compute/ , https://www.letta.com/blog/memory-blocks/ | 1 (snippets only) |
| Hu, Wang, McAuley, *MemoryAgentBench*, arXiv 2507.05257 (ICLR 2026), HTML v4 | https://arxiv.org/html/2507.05257v4 | 2.6, 5 |
| Xu et al., *A-MEM*, arXiv 2502.12110 (NeurIPS 2025) | https://arxiv.org/abs/2502.12110 | 1 (abstract only) |
| Wang et al., *Recursively Summarizing Enables Long-Term Dialogue Memory*, arXiv 2308.15022 | https://arxiv.org/abs/2308.15022 | 1 (abstract only) |
| Lee et al., *ReadAgent*, arXiv 2402.09727 | https://arxiv.org/abs/2402.09727 | 1 (abstract only) |
| Sarthi et al., *RAPTOR*, arXiv 2401.18059 | https://arxiv.org/abs/2401.18059 | 1 (abstract only) |
| Zhong et al., *MemoryBank*, arXiv 2305.10250 | https://arxiv.org/abs/2305.10250 | 5 (abstract only) |
| Anokhin et al., *AriGraph*, arXiv 2407.04363 (IJCAI 2025) | https://arxiv.org/abs/2407.04363 | 1 (search snippet only) |
| Yang et al., *Re3*, arXiv 2210.06774 (EMNLP 2022), HTML v3 | https://arxiv.org/html/2210.06774v3 | 3, 5 |
| Yang et al., *DOC*, arXiv 2212.10077 (ACL 2023) | https://arxiv.org/abs/2212.10077 and https://aclanthology.org/2023.acl-long.190.pdf | 3 (abstract and one passage) |
| Zhou et al., *RecurrentGPT*, arXiv 2305.13304 | https://arxiv.org/pdf/2305.13304v1.pdf | 3 |
| Li et al., *ConStory-Bench*, ACL Findings 2026, project page | https://picrew.github.io/constory-bench.github.io/ and https://aclanthology.org/2026.findings-acl.410/ | 3 |
| *NstAgent*, arXiv 2609.35759 (under review) | https://arxiv.org/abs/2609.35759 | 3 (abstract only) |
| Kim et al., *FANToM*, EMNLP 2023, project page | https://hyunw.kim/fantom/ and https://aclanthology.org/2023.emnlp-main.890/ | 4 |
| Wu et al., *Hi-ToM*, Findings of EMNLP 2023 | https://aclanthology.org/2023.findings-emnlp.717/ | 4 (abstract only, from a search snippet) |
| *The Curse of Knowledge in Role-Playing Language Models*, arXiv 2610.17327 (2026-10-12, preprint) | https://arxiv.org/html/2610.17327v1 | 4 |
| Jiang et al., *Anatomy of Agentic Memory*, arXiv 2602.19320 (survey) | https://arxiv.org/abs/2602.19320 | 7 (abstract only) |
| `aikohanasaki/SillyTavern-MemoryBooks` | https://github.com/aikohanasaki/SillyTavern-MemoryBooks | 6.1 (search snippets, GitHub API push date) |
| `qvink/SillyTavern-MessageSummarize` | https://github.com/qvink/SillyTavern-MessageSummarize | 6.1 (same) |
| `Coneja-Chibi/TunnelVision` | https://github.com/Coneja-Chibi/TunnelVision | 6.1 (same) |
| `SpicyMarinara/rpg-companion-sillytavern` | https://github.com/SpicyMarinara/rpg-companion-sillytavern | 6.1 (same) |
| Character.AI blog, "Smarter Memory for Smarter Chats" (2026-05-21) | https://blog.character.ai/memory/ | 6.2 |
| Kindroid Help Center: Memory; Groupchats | https://kindroid.ai/v2/docs/memory/ , https://kindroid.ai/v2/docs/groupchats/ | 6.2 (the group-chat page: search snippet only) |
| Nomi posts: Identity Core (2024-12-12), Mind Map 2.0 (2025-10-09), Nomipedia | https://nomi.ai/updates/introducing-the-nomi-identity-core-fostering-dynamic-and-authentic-identities/ , https://nomi.ai/updates/mind-map-2-0-bringing-nomi-memory-into-view/ | 6.2 (search snippets) |
| Replika help center: memory | https://help.replika.com/hc/en-us/articles/37208679176077-How-does-Replika-s-memory-work | 6.2 (search snippet) |
| The earlier survey (local) | `../../prompt-flexibility/assets/long-term-memory.md` | Generative Agents, MemGPT, ME, ST, AID, NovelAI facts |
