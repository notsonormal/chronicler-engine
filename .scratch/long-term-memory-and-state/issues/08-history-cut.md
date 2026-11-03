# Decide the size and the point of the history cut

Type: grilling
Status: resolved
Blocked by: —

## Question

How large is the history space of a Narrator-role call, and where does the history cut fall? Choose between a fixed number of tokens, a share of the real window of the model, and a logical point such as a scene end. Which `max_context_tokens` value does each Connection get, and is the history cap a code constant or a setting?

## Context

- **Why this comes first (owner, 2026-11-03).** The memory candidates ([Choose the memory candidates to try](05-memory-candidates.md)) depend on how much leaves the prompt, and when. The size of the history was never decided. [Cut the history in blocks](../../prompt-flexibility/issues/20-cut-the-history-in-blocks.md) changed *how* the history is cut (whole Messages, in blocks, for the cache), and kept the size.
- **Two separate limits (owner, 2026-11-03).** What the dashboard can scroll back to is not what the prompt holds. The scroll-back is a separate effort ([the seed](../../story-scroll-back/idea.md)). This ticket decides only the prompt.
- **The engine today (known).**

  | Limit | Value | Where |
  |---|---|---|
  | Story window in the dashboard | the newest 50 entries | `STORY_WINDOW_SIZE`, `src/application/games/view_query.rs` |
  | Storage | every Message | `messages` table |
  | Prompt history | at most 16,000 estimated tokens. A cut drops whole Messages until the rest fills 75 % | `MAX_HISTORY_TOKENS`, `src/application/prompting/token_budget.rs`; `KEPT_AFTER_CUT_PERCENT`, `src/domain/model/history_cut.rs` |
  | Connection window | 32,768 tokens for OpenCode Go, because the Connection sets no `max_context_tokens` | `src/domain/model/llm_backend.rs`, `data/settings.json` |

  In the 100-Cycle trial of the cut, a Message was about 206 tokens, a cut came every 9 to 11 Cycles and dropped 17 to 19 Messages. Narrator-role calls read 61 % cached between cuts, and about 0 % on a cut.
- **What other tools do (known, from earlier assets).**

  | Tool | Rule | Source |
  |---|---|---|
  | ST | Fills the history newest first until `openai_max_context − openai_max_tokens`. It drops the oldest Message first, on every call. No logical point. The default preset is 4095 and 300 | [ST and ME prompt configuration](../../prompt-flexibility/assets/st-me-prompt-configuration.md), "Token budget" |
  | ME | The default preset has `maxContext` 128,000. With Advanced Memory, the live history limit is `maxContextTokens` (65,000). Over it, the history resets to the start of the latest scene, then grows again. Memory gets 10 % of the context | [the ME asset](../assets/marinara-memory.md), section 7.2 |
  | AI Dungeon | Memory parts take about 25 % of the free space | [how other tools keep long-term memory](../../prompt-flexibility/assets/long-term-memory.md) |

- **The model and a long context (known).** RULER: only half of the models that claim 32K or more work well at 32K. *Lost in the Middle*: the middle of a long context is used worst. LongMemEval: at about 115K tokens, long-context models lose 30 to 60 % against the evidence alone. While the history fits, the full history beats every memory system ([the LLM memory research](../assets/llm-memory-research.md), section 2.7).
- **The facts for this model** ([the model context asset](../assets/model-context.md)).
  - **known** The window is 1M tokens, and the maximum output is 384K, on DeepSeek and on OpenCode Go. The 32,768 is the engine default, not a provider limit. Whether OpenCode Go enforces less: I don't know.
  - **I don't know** any score of this model at 16K to 128K. The vendor scores are at 1M, and non-think mode (our Connection) is much weaker there than think mode (MRCR 37.5 against 76.9).
  - **known** Local evidence: in the memory test, the whole-story arm put up to about 41K tokens of history in the stored request, and it answered 95 % of the kept questions ([the memory test](../assets/memory-test.md)). This tests recall, not use in play.
- **The cost (inferred, [the model context asset](../assets/model-context.md), section 4.3).** One 100-Cycle trial, Narrator-role calls, off-peak: 2.65M input tokens and $0.08 at 16K (86 % cached, the measured share of the `lt15` trial); 4.05M and $0.12 at 32K; 4.47M and $0.13 at 64K or 128K, which never cut in 100 Cycles. Peak hours double the price. This supersedes the "2.7 times" of [Prompt caching](../../prompt-flexibility/assets/prompt-caching.md), section 4.3.
- **The check trial (owner: not decided).** A 100-Cycle trial is about 41,000 tokens of history (200 Messages × 206 tokens, inferred). If the new space holds it, the check trial cannot test a memory of dropped entries. This ticket gives the size. The map keeps the trial length in "Not yet specified".

## Output

A resolution that gives the history space (the rule and its numbers), whether the cut falls at a logical point, the `max_context_tokens` value of each Connection in `data/settings.json`, and where the cap lives (code, Connection or preset). Create the build ticket for the change, and update [Choose the memory candidates to try](05-memory-candidates.md) with what leaves the history at the new size.

## Answer

Resolved on 2026-11-03 (owner). Every answer below is the owner's.

| # | Decision |
|---|---|
| Q1 | The cut falls at a size in tokens, as today. No logical point (a scene end) now. [Choose the memory candidates to try](05-memory-candidates.md) can add one later, on top of the size rule. |
| Q2 | The size is a field of the Connection. When the field is empty, a code default applies. The quality limit is a fact about the model, and a preset cannot change the model. |
| Q3 | `opencode-go-deepseek-v4.1-flash` gets `max_context_tokens` 1,000,000 in `data/settings.json`. The window field states the real window. The history size limits the history. |
| Q4, Q6 | The default is 65,536 tokens (64K), chosen now from the evidence that exists. A 100-Cycle story (about 41K tokens) fits whole, and the first cut comes at about Cycle 155 (inferred). The history space stays the smaller of the size and the room in the window, so an 8192-token Ollama window still gets about 1,600 tokens. |
| Q5, Q7 | The size of a trial is separate from the default. The trials of this map, and the check trial of the Destination, run 100 Cycles at 16,000 tokens. That size matches the stored stories, the baseline counts and the stored requests of the memory test, and about 5 cuts happen in such a trial. |
| Q8 | A `history_tokens` field in the trial setup file. `trial.py` writes it on the Connection of the probe server before the run, and the pack shows it. |
| Q9 | A "History (tokens)" field in the Settings Connection form, under "Context window (tokens)", with the placeholder "Empty uses the default (65,536)". The stored key is `max_history_tokens`. |
| Q10 | A cut still drops the history to 75 % of the space (`KEPT_AFTER_CUT_PERCENT`). The memory design can change it later. |

- **Known evidence for the default.** The model has a 1M-token window on DeepSeek and on OpenCode Go. No benchmark scores it at 16K to 128K in non-think mode. The whole-story arm of the memory test answered 95 % of the recall questions with about 41K tokens of history. One 100-Cycle trial of Narrator-role calls costs about $0.13 off-peak at 64K, against $0.08 at 16K (inferred, [the model context asset](../assets/model-context.md)).
- **Build ticket:** [Make the history size a Connection field](09-history-size-field.md).
