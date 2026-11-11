# Model context: window, long-context quality and price of `deepseek-v4.1-flash`

Scope: the Connection `opencode-go-deepseek-v4.1-flash`, used for the Narrator role.
Research only. No engine change, no trial, no model call, no `.env` read.
Labels: **known** (read in a source or stored row), **inferred**, **guessed**. Web sources were read on 2026-11-03.

## 1. Short answer

- **known** The window is 1M tokens. The maximum output is 384K. DeepSeek and OpenCode state the same values. I found no smaller window on OpenCode Go.
- **known** The 32,768-token window is the engine default, not a provider limit. A history space above about 30K needs a larger `max_context_tokens` on the Connection.
- **I don't know** any score of this model on Fiction.LiveBench, RULER, NoLiMa or OpenAI MRCR at 16K, 32K, 64K or 128K. I found none. The Fiction.LiveBench page needs JavaScript.
- **known** Only vendor numbers exist, and none is at our lengths. DeepSeek V4-Flash scores 37.5 on MRCR at 1M in non-think mode, and 76.9 in high-think mode. Our Connection sends `reasoning_effort: none`.
- **known** OpenCode Go and DeepSeek direct charge the same: $0.15 per 1M input (miss), $0.003 (hit), $0.60 output, off-peak. All rates double at peak hours.
- **inferred** A 100-Cycle trial sends 2.65M Narrator-role input tokens at a 16K history space, 4.05M at 32K, and 4.47M at 64K or 128K.
- **inferred** The cache hit share changes the cost about 5 times (0 % to 86 % hit). The space changes it at most 1.6 times.
- **known** A 100-Cycle story is about 41K tokens. So 64K and 128K never cut in one trial.

## 2. The window

| Source | Window | Max output | Label |
|---|---|---|---|
| DeepSeek API docs, `deepseek-flash` (`DeepSeek-V4.1-Flash-1017`) | 1M | 384K | known |
| OpenCode data page, `deepseek/deepseek-v4.1-flash` | 1M | 384K | known |
| pi.dev list, provider `opencode-go` | 1,000,000 | 384,000 | known |

- **known** The Go docs page lists the model and its prices. It states no window.
- **inferred** The pi.dev entry copies OpenCode metadata. It is not an OpenCode document.
- **I don't know** whether OpenCode Go enforces a smaller limit. I found no public metadata endpoint that needs no key.
- **known** The tech report: training lengths are 4K, 16K, 64K, then 1M. Sparse attention starts at the 64K stage.

## 3. How well it uses a long context

| Measure | Model and mode | Score | Length | Type |
|---|---|---|---|---|
| OpenAI MRCR (MMR) | V4-Flash non-think / high / max | 37.5 / 76.9 / 78.7 | 1M | vendor |
| CorpusQA (accuracy) | V4-Flash non-think / high / max | 15.5 / 59.3 / 60.5 | 1M | vendor |
| LongBench-V2, 1-shot, base models | V3.2 / V4-Flash / V4-Pro | 40.2 / 44.7 / 51.5 | not stated | vendor |
| MRCR 8-needle | V4-**Pro**-Max | above 0.82 to 256K, 0.59 at 1M | 256K, 1M | vendor, Pro |
| AA-LCR v1.1 | V4-Flash, setting not stated | 74.3 | guessed about 100K | aggregator |

- **known** The report says retrieval is "highly stable within a 128K context window" and degrades beyond it. That is for V4-Pro. It gives no Flash curve, and I did not read its chart.
- **inferred** Non-think mode is weaker at long range than think mode (37.5 against 76.9 at 1M). Our Connection uses non-think mode. So the think-mode curves may overstate our case. I have no non-think score below 1M.
- **known** MRCR and CorpusQA test retrieval. They do not test story comprehension.
- **I don't know** any Fiction.LiveBench score for V4 or V4.1. Searches found no table.
- **known** Older DeepSeek results exist only as Reddit summaries (2025), with no numbers. V3 0324 "retains its performance up to 60k tokens". R1 "ranks 1st at 60k tokens, but drops sharply at 120k". The V3.2 chat variant is "massively worse" than its reasoner.
- **inferred** The older results agree with the MRCR gap: the non-think variant is the weak one.
- **guessed** A history space up to 32K is low risk for this model. A space of 64K or 128K is untested in non-think mode. Only a trial can settle it.

## 4. The price

### 4.1 Rates (per 1M tokens)

| Channel | Input (miss) | Input (hit) | Output |
|---|---|---|---|
| OpenCode Go and DeepSeek direct, off-peak | $0.15 | $0.003 | $0.60 |
| OpenCode Go and DeepSeek direct, peak | $0.30 | $0.006 | $1.20 |

- **known** Peak is 01:00 to 04:00 and 06:00 to 10:00 UTC, Monday to Friday. Cache write has no price.
- **correction** The earlier notes give $0.15 as "the price" of this endpoint. That is the off-peak price. A trial at peak hours costs twice as much.
- **known** OpenCode Go reports `usage.cost` as 0. The real figures are `usage.opencode.list_cost` and `usage.opencode.allowance`. The Go docs say `allowance` is what the request counts against the plan limit. In `tmp/lt15/new/trial-1/calls.jsonl` the sums give allowance = 0.6 × list cost, for the Narrator role ($0.0482 of $0.0804) and for the Quantifier. Why, I don't know.

### 4.2 Assumptions

| Input | Value | Label |
|---|---|---|
| Tokens per Message | 206 (issue 20) | known |
| Messages per Cycle | 2 | known |
| Narrator-role calls per Cycle | 2 (Narration, Impersonate) | known |
| Fixed prompt outside the history | about 2,100 (Narration), 1,140 (Impersonate) | inferred |
| Output | 34,087 tokens over 200 calls (measured) | known |
| Cut rule | at the space, drop to 75 % (`HistoryCut`) | known |

- **inferred** I fitted the fixed prompt from `tmp/lt15/new/trial-1/calls.jsonl`: prompt tokens of 199 Narrator-role calls against history characters. A fit by Message count gives 2,318 and 1,126. The error is about 10 %.
- **known** In that file a fit gives 175 tokens per Message, not 206. I use 206 as ordered. With 175 the input falls by 3 % at 16K, 9 % at 32K and 14 % at 64K and above.
- **known** The Quantifier does not use the history space (window of 4). Its list cost in the same file is $0.0356 for 200 calls. I add it as a constant.
- **inferred** The model fits the stored trial. At 16K it gives 7 cuts, the first in Cycle 39 (stored: 7 cuts, first in Cycle 40). It gives 2.65M input tokens (stored: 2.53M). At 86 % hit it gives $0.0802 (stored list cost: $0.0804).

### 4.3 Result: one 100-Cycle trial

Narrator-role calls. List cost at off-peak prices, output included ($0.020).

| History space | Input tokens | Cuts (first Cycle) | Cost, 0 % hit | 61 % hit | 86 % hit |
|---|---|---|---|---|---|
| 16K | 2.65M | 7 (39) | $0.42 | $0.18 | $0.08 |
| 32K | 4.05M | 2 (78) | $0.63 | $0.27 | $0.12 |
| 64K | 4.47M | 0 | $0.69 | $0.29 | $0.13 |
| 128K | 4.47M | 0 | $0.69 | $0.29 | $0.13 |

- **inferred** Add $0.036 for the Quantifier to each cell. Double the trial at peak hours.
- **known** The two hit shares are measured. 61 % is the share between cuts in the pf20 trial (issue 20). 86 % is the Narrator-role share of the `lt15` trial (2,178,304 of 2,534,338 input tokens).
- **guessed** A larger space keeps the same hit share. Fewer cuts may raise it. A longer prompt may lower it. I have no measurement above 16K.
- **inferred** The 64K and 128K rows are equal because the story ends at about 41K tokens. The first cut would come at about Cycle 155 (64K) and Cycle 311 (128K).
- **inferred** A space of 32K or more needs a new `MAX_HISTORY_TOKENS` and a larger `max_context_tokens`. At 32K the space, 2.1K fixed prompt, 2,048 output and the margin pass the 32,768 default.

## 5. Open questions

- I do not know any Fiction.LiveBench figure for V4.1 Flash. A person can read the page in a browser.
- I do not know whether OpenCode Go enforces the 1M window. A 200K-token call would show it.
- I do not know the cache hit share above 16K of history. One trial at 32K would measure it.

## Sources

| Source | URL |
|---|---|
| DeepSeek API docs, Models & Pricing | https://api-docs.deepseek.com/quick_start/pricing |
| OpenCode Go docs | https://dev.opencode.ai/docs/go/ |
| OpenCode data, DeepSeek V4.1 Flash | https://opencode.ai/data/deepseek/deepseek-v4-1-flash |
| pi.dev model list, `opencode-go` | https://pi.dev/models/opencode-go/deepseek-v4-1-flash |
| DeepSeek V4 tech report | https://arxiv.org/html/2606.19348 |
| DeepSeek V4 Flash model card | https://huggingface.co/deepseek-ai/DeepSeek-V4-Flash |
| Hugging Face blog, DeepSeek V4 (MRCR 8-needle) | https://huggingface.co/blog/deepseekv4 |
| Benchmark Atlas, V4-Flash (AA-LCR, aggregator) | https://atlas.kevinhu.io/models/deepseek-v4-flash |
| r/LocalLLaMA, Fiction.LiveBench chart (secondary) | https://www.reddit.com/r/LocalLLaMA/comments/1kcghky/chart_of_medium_to_longcontext_fictonlivebench/ |
| r/SillyTavernAI, V3.2-exp (search snippet only) | https://www.reddit.com/r/SillyTavernAI/comments/1nu7ybk/deepseek_v32exp_context_comprehension_on/ |
| Local: `tmp/lt15/new/trial-1/calls.jsonl`, issue 20 | read 2026-11-03 |
