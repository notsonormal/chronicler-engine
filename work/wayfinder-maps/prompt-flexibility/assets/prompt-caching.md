# Prompt caching: the measured prefix, the stake and the cheap rules

Scope: the Narrator role (Narration, Event Narration, Impersonate), the Quantifier and the Options.
The connection test is out of scope.

Every claim is **known** (read in the code, or read in a stored row), **inferred** (reasoned from
a measurement without a direct read) or **guessed**. Sources are at the end.

The owner's rule for this ticket: caching is a balance, not a goal. This asset looks for cheap
rules that stop a cache break on every call. It does not look for the best possible hit rate.

## 1. Method

- **known** The measured Game is Game 6 of `tmp/stage6/debug/chronicler_3060.db`: World
  "Redmist Estate", Persona "Julian", a 50-Message story played by Autoplay on 2026-10-22,
  2026-10-25 and 2026-10-26. The `llm_messages` table of that DB holds 50 rows: 11 Narration,
  8 Impersonate, 1 Event Narration and 30 Quantifier calls. Some Message rows in that copy are
  duplicated. The `llm_messages` rows are not, so they give the call sequence.
- **known** No Options call is in that Game. Game 6 has `options_always_on = 0` (`games` table).
  The other 8 DBs of `tmp/db-backup-20261031/` hold 4 Options rows, and no two of them come from
  the same Game. So this asset measures no Options pair.
- **known** All reads were read-only URI opens. No row was written.
- **known** Each pair below is two calls of the same kind, one after the other, with no failed
  call between them. A pair means "the same Game asks the same question again".
- **known** The shared prefix is measured as the byte prefix of `system_prompt + user_prompt`,
  which is the request as the model sees it. Two messages cost one byte of separator, so the
  figures are short by that byte. That error is below 0.1%.
- **known** The token count of a stored prompt is in `usage.prompt_tokens` of its
  `raw_response_json`. The measured ratio is 4.5 to 4.7 characters per token for the Narrator
  role and 4.0 for the Quantifier.

## 2. The measured shared prefix, per call kind

Today, in the current layout. "Share" is the prefix divided by the new call's characters.

| Call kind | Pairs measured | Shared prefix | Share of the new call | Where the first difference falls |
|---|---|---|---|---|
| Narration | 8 | 6304 to 6725 chars | 23.5% to 38.7%, mean 27.8% | inside `GameState` (byte 20 to 30), else inside `KnownCharacters` (byte 13 to 454) |
| Impersonate | 6 | 3238 to 3593 chars | 16.9% to 40.0%, mean 26.8% | inside `GameState` (byte 20 to 30), else inside `KnownCharacters` (byte 13) |
| Quantifier | 4 | 2196 chars | 24.2% to 37.2% | inside `<available_character_ids>`, at the first `<Character>` line |
| Event Narration | 0 | — | — | one call only, so there is no pair |
| Options | 0 | — | — | no two Options calls of one Game |

- **known** The shared prefix in every pair is the system message plus at most 460 bytes of the
  user message. So the engine caches the system message only.
- **known** The system message was byte-identical in every measured pair: 8 Narration and 6
  Impersonate. The preset text and the world `global_rules` do not change inside a Game.
- **known** The Narration system message is 6274 chars (about 1360 tokens). The Impersonate system
  message is 3208 chars (about 700 tokens). The Quantifier system message is 2959 chars.
- **known** The story grows only at its end. In every measured pair the older `ConversationHistory`
  block is a byte prefix of the newer one. The 16000-token history cut was never active in Game 6
  (the largest history block is 42936 chars, and the cut starts at 64000 chars).
- **known** The provider agrees. `usage.prompt_tokens_details.cached_tokens` is above zero on 7 of
  the 19 successful Narrator calls of Game 6: 512, 1024, 1280, 512, 512, 1280, 1280. It is 0 on
  the other 12. The hit share per call is 11.8% to 27.3%.
- **inferred** The measured hits (512 to 1280 tokens) are the whole system message rounded down,
  because the system message is 700 to 1360 tokens. DeepSeek reads its cache in blocks and calls
  it best-effort, so a hit need not reach the full block. The stored rows do not show the block
  size, so I do not know the exact rounding rule.

## 3. The three causes of the break today

The first difference falls at the start of the user message in every pair. Three causes produce it.

| Cause | Where | What it does | Label |
|---|---|---|---|
| The card list order is random | `KnownCharacters` (Narrator), `<available_character_ids>` (Quantifier) | The order of the cards differs between two calls, so the first card line differs | known |
| The presence marks change | `KnownCharacters` holds `(in room)` / `(elsewhere)` for each card | A Character who enters or leaves the room changes a card line | known |
| The volatile blocks come first | `GameState` is the first block of the user message | A new Room changes byte 20, so the 6000 to 43000 chars of history after it are lost | known |

**Cause 1, the card order.** `known`: `src/application/pipeline/job.rs:255` builds
`Vec<CharacterCard>` from `bundle.characters.values()`, and `bundle.characters` is a
`HashMap<String, CharacterCard>` (`src/domain/model/agent.rs:87`). The same line exists at
`src/application/agents/quantifier/utils/orchestration.rs:30`. A `HashMap` iterates in a
per-instance random order, so each call lists the cards in a new order.

`known`: the stored rows show it. The first `KnownCharacters` entry is Carla in call 161, Lisette
in call 164, and Gabriella in call 167. The first `KnownNpcs` entry moves the same way in the
earlier rows. The Quantifier shows it too: its `<available_character_ids>` order differs between
calls 130 and 132, and between 132 and 134.

**Cause 2, the presence marks.** `known`: `builders/sections.rs:81-85` writes `(in room)` or
`(elsewhere)` beside every card name. The same fact is already in `CharactersPresent`, which lists
the full cards of the Characters in the Room. So the marks carry no information that the prompt
does not carry twice.

**Cause 3, the order.** `known`: the layer order is GameState, the cards, Persona, WorldLore,
History, the style and format sections, PlayerInput (`assembler.rs:171-178`). GameState is 135 to
147 chars, so a change there is cheap to write and expensive to cache.

Two more instabilities, which the measured pairs did not show but the code allows:

- **known** `<CharactersPresent>` keeps the order that the Quantifier returned, because
  `game_rules.rs:67` collects `result.present` in its own order and never sorts it. A Quantifier
  answer with the same set in a new order changes the block.
- **known** The two history cuts slide on every call once they are active.
  `assembler.rs:327` cuts the history to 16000 estimated tokens as a raw character cut, so it can
  cut inside a line. `prompting/utils/context.rs:50-101` then drops whole lines from the oldest
  end until the prompt fits the context. Both move the first byte of the history.

## 4. The stake

### 4.1 Input tokens per cycle, measured

`known`: one Autoplay Cycle of Game 6 makes four model calls: Impersonate, Quantifier, Narration,
Quantifier. A fired Trigger adds an Event Narration and its Quantifier. Prices are the `usage.cost`
that the provider reported.

| Cycle | Calls that answered | Input tokens | Cached tokens | Output tokens | Cost |
|---|---|---|---|---|---|
| 1 | 4 | 7439 | 0 | 342 | $0.003792 |
| 2 | 6 (a Trigger fired) | 15226 | 0 | 844 | $0.007668 |
| 3 | 2 (both Quantifiers failed) | 7152 | 1536 | 447 | $0.004161 |
| 4 | 2 (both Quantifiers failed) | 7994 | 1792 | 541 | $0.004613 |
| 5 | 2 (both Quantifiers failed) | 9132 | 0 | 466 | $0.006472 |
| 6 | 2 (both Quantifiers failed) | 9984 | 1792 | 392 | $0.005980 |

- **known** Each Narrator call grows by a few hundred tokens per cycle. The Narration call grew by
  392, 593 and 383 tokens across 142 to 148 to 154 to 160. The Impersonate call grew by 450 and 469
  tokens across 139 to 145 to 151 to 157.
- **known** The ten Quantifier calls of cycles 3 to 6 failed with HTTP 400 and cost nothing. A
  measured Quantifier call takes 1257 to 2127 tokens, so the two of a cycle add about 2700 at cycle
  1 and about 3700 by cycle 4.
- **inferred** One complete cycle therefore adds about 1300 to 1400 input tokens in total.
- **known** The whole 6-cycle run cost $0.0327 and sent 56927 input tokens.

### 4.2 The price of one input token depends on the upstream host

The one model id `deepseek/deepseek-v4-flash` was served by three upstream hosts in this DB. The
stored `usage.cost_details.upstream_inference_prompt_cost` gives the price of each.

| Upstream host that OpenRouter reported | Measured input price | Calls | Cached tokens in that call |
|---|---|---|---|
| Wafer | $0.700 per 1M | most Narrator calls | 0, 512, 1024 or 1280 |
| Azure | $0.210 per 1M | 161, 164, 167 (Narrator) | 0, 0, 1280 |
| Cloudflare | $0.440 per 1M | 172 (Impersonate) | 0 |
| Azure, `openai/gpt-4o-mini` | $0.150 per 1M | every Quantifier call | 0 |

- **known** A cache hit costs 0.1 of the input price on OpenRouter for DeepSeek. Call 160 read 1280
  of its 5661 tokens from the cache and paid $0.0031947 for the prompt. Call 154 read none and paid
  $0.0036946 for 5278 tokens. Both divide to exactly $0.700 per 1M uncached, with a cached token at
  one tenth.
- **known** So the same work cost 3.3 times more on one upstream host than on another. The routing
  changes the price more than the cache does today.
- **known** The current `data/settings.json` points the narration and the quantifier roles at
  `opencode-go-deepseek-v4.1-flash`, and the measured Game used OpenRouter connections. So the
  dollar figures below carry the Wafer price of the measured connection. I do not know the price of
  the current connection, because no stored row of it carries a priced cost.

### 4.3 One 100-cycle trial, inferred

`inferred`: the prompt grows by 7400 + 1400 x (n - 1) tokens at cycle n. That line starts at the
measured 7439 tokens of complete cycle 1 and uses the growth of about 1350 tokens per cycle. The engine caps the call at `max_context_tokens` minus `max_tokens`
minus the safety margin. The connections of Game 6 set no `max_context_tokens`, so the engine uses
32768 for OpenRouter (`llm_backend.rs:70-76`). The cap is therefore 32768 - 2048 - 256 = 30464
input tokens, and it binds at cycle 18.

This table uses the Wafer price of $0.700 per 1M, and `cost = (1 - 0.9 x hit share) x price`.

| Case | Hit share | Total input tokens | Cost of 100 cycles |
|---|---|---|---|
| No cache at all | 0% | 2.85M | $1.99 |
| Today, measured behaviour | 7.4% mean | 2.85M | $1.86 |
| Cheap rules 1 to 3, no cut rule | 95% until the cap, 10% after | 2.85M | $1.64 |
| Cheap rules 1 to 4, the cut rule included | 95% until the cap, 91% after | 2.85M | $0.35 |

- **inferred** The cut rule is the important one for a long trial. Above the cap the history is cut
  on every call. The history start then moves on every call, so the hit falls back to the blocks
  before the history, which are about 10% of the capped prompt. A cut that moves in steps keeps the
  hit near 95% between cuts.
- **inferred** If ticket 07 raises `max_context_tokens` to the provider's real context, the same
  trial sends about 7.7M input tokens and no cut binds until cycle 60. The cache rules then save
  more money, not less.
- **known** The measured hit share today is 7.4% of the Narrator input tokens, averaged over all 19
  successful Narrator calls of Game 6. The other 12 calls read nothing from the cache.

### 4.4 Latency

- **known** On the cloud connections, the stored rows measure wall time but not prefill time.
  `created_at` of the next row minus `created_at` of this row gives 5.2 to 6.6 s for a Narration
  call of 3000 to 3600 input tokens and 150 to 300 output tokens, and 1.3 to 2.8 s for a Quantifier
  call of 1300 to 2100 input tokens. Some gaps contain a human pause, so these figures are upper
  bounds.
- **inferred** The cache removes the prefill of the hit part. The rows cannot separate prefill from
  decode, network and queue time, so I cannot state the saved seconds. I do not know it.
- **known** DeepSeek states that a cache hit "reduces service latency" without a figure.
- **inferred** For local Ollama the prefill is the whole cost of a long prompt, and there is no
  network. Third-party `llama-bench` runs of `gemma-4-26B-A4B-it` report 5951 prompt tokens/s on an
  RTX 5070 Ti and 12176 on an RTX 5090. At 5951 tokens/s a 10000-token prompt costs 1.7 s of
  prefill and a 30000-token prompt costs 5.0 s. A 95% hit removes almost all of it. These are not
  measurements on this machine.

## 5. Our connections: automatic, or opt-in

All of our connections use the one chat-completions provider
(`src/adapters/driven/llm/providers/chat_completions.rs`), so one request shape serves all of them.

| Connection (`data/settings.json`) | Provider | Cache without a request change | Where a hit shows in `usage` |
|---|---|---|---|
| `openrouter-gpt-4o-mini`, `openrouter-euryale`, `openrouter-hanami`, `mistral-nemo`, `deepseek-v4-flash` | OpenRouter | Yes, for the models whose upstream caches | `usage.prompt_tokens_details.cached_tokens` |
| `opencode-go-deepseek-v4.1-flash` | OpenCode Go | I don't know the rule. The response shape shows a cache that is billed | `usage.prompt_tokens_details.cached_tokens`, and `usage.opencode.tokens.cache_read` |
| `ollama-gemma-4-26B`, `ollama-gemma-4-e2b` | Ollama | Yes, per slot, on by default | `usage.prompt_tokens_details.cached_tokens` on `/v1`, and `prompt_eval_cached_count` on `/api/chat` |

- **known** OpenRouter: "Most providers automatically enable prompt caching". For DeepSeek, "prompt
  caching with DeepSeek is automated and does not require any additional configuration", and cache
  reads cost 0.1 of the input price. A hit appears in `usage.prompt_tokens_details.cached_tokens`.
- **known** OpenRouter routing loses the cache in our layout. Its sticky routing pins the upstream
  host after a cached request, but its default key is a hash of the first system message and the
  **first non-system message**. Our first non-system message is the whole user prompt, and it
  changes on every call. Sticky routing also expires after 10 minutes of inactivity.
- **known** OpenRouter accepts a `session_id` in the request body, or an `x-session-id` header, or
  the OpenAI `prompt_cache_key` field, as the sticky key. With one, "sticky routing activates on any
  successful request". We send none of them (`transport/utils/request.rs:68-118`).
- **known** DeepSeek direct: caching is "enabled by default for all users", with no code change. A
  hit shows in `usage.prompt_cache_hit_tokens` and `prompt_cache_miss_tokens`. A cache hit costs
  $0.006 per 1M input tokens at peak and the miss costs $0.30, so a hit is 50 times cheaper. The
  cache lives "a few hours to a few days".
- **known** OpenCode Go: I found no documentation for this endpoint, so I do not know its cache
  rule. Its stored answer shows that it reports one. Both calls of
  `tmp/db-backup-20261031/chronicler_3150.db`, written 2026-10-31, carry
  `usage.prompt_tokens_details.cached_tokens`, `cache_write_tokens` and `cache_creation_input_tokens`,
  and an `opencode.tokens` block with `cache_read`, `cache_write_5m` and `cache_write_1h`. Both calls
  read 0 from the cache. They are a Narrator call and a Quantifier call of one Job, so they are not
  a pair of the same kind.
- **inferred** The `cache_write_5m` and `cache_write_1h` fields are the Anthropic cache-write TTLs.
  So the endpoint probably bills an explicit cache with those two lifetimes. I did not read that in
  a document.
- **known** OpenCode Go reports `usage.cost` as 0. The real cost of the same two calls is in
  `usage.opencode.allowance`, `0.00029124` and `0.00012348`, against a `list_cost` of `0.0004854`
  and `0.0002058`. A cost total that reads `usage.cost` alone reports this connection as free.
- **known** Ollama and llama.cpp: `llama-server` has `--cache-prompt` on by default ("whether to
  enable prompt caching (default: enabled)"). It reuses "whatever prefix the next request shares
  with it". Ollama's OpenAI-compatible layer returns `usage.prompt_tokens` = the whole prompt and
  `usage.prompt_tokens_details.cached_tokens` = `PromptEvalCachedCount`
  (`openai/openai.go:76-82`, `:266-271`).
- **known** Ollama keeps a model in memory for 5 minutes by default (`keep_alive`). It unloads the
  model after that, and the KV cache goes with it. An Autoplay run makes a call every few seconds,
  so this does not bite inside a run.
- **known** Ollama sets the context length from the VRAM at start, with 4k below 24 GiB, 32k from
  24 GiB and 256k from 48 GiB. Our two Ollama connections set no `max_context_tokens`, so the
  engine budgets 8192 (`llm_backend.rs:70-76`). A prompt above the server's context is truncated,
  and a truncated prompt breaks the prefix.

### ST and ME, one line each

- **known** SillyTavern: opt-in, Claude only. `claude.cachingAtDepth` defaults to `-1` (off) and
  `claude.enableSystemPromptCache` defaults to `false` (`default/config.yaml:360`, `:368`). It puts
  `cache_control: {type: "ephemeral"}` on a message at a depth counted from the newest one
  (`src/prompt-converters.js:985-1006`). On one OpenRouter path it also sends an `x-session-affinity`
  header, an HMAC of the chat id, to pin the route (`src/endpoints/backends/chat-completions.js:2474`).
- **known** Marinara Engine: opt-in per connection. `enable_caching` defaults to `false`,
  `caching_at_depth` defaults to 5, and the extended TTL defaults to false
  (`packages/server/src/db/schema/connections.ts:66-72`). It applies `cache_control` only in its
  Anthropic provider (`services/llm/providers/anthropic.provider.ts:490-510`) and records
  `cachedPromptTokens` from `usage.cache_read_input_tokens` (`:627-631`).
- **known** Neither app orders the prompt for the cache. Both treat it as an optional provider
  feature, which matches the owner's "balance, not a goal".

## 6. The cheap rules

Each rule is small and independent. The prefix figures come from the same measured pairs, with the
layer bytes re-ordered in the measurement script so that only the named change is applied.

| # | Rule | Estimated shared prefix for a Narration call | Measured effect | What it costs an author |
|---|---|---|---|---|
| 1 | Sort the card list by id, in the Narrator and the Quantifier | 81.6% to 96.9% of the call in the 8 Narration pairs, against 23.5% to 38.7% today | Narration: mean 27.8% to 84.1%. Quantifier: 24% to 37% becomes 66% to 70%, and 33% when the Room also changed | Nothing. The card order carries no meaning |
| 2 | Take `(in room)` / `(elsewhere)` out of `KnownCharacters` | +1000 to +1300 chars on a pair where a Character moved but the Room did not. No change when no Character moved | +6 to +13 points of share on those pairs | Nothing. `CharactersPresent` already states who is in the Room |
| 3 | Order the blocks: system, WorldLore, Persona, KnownCharacters, CharactersPresent, GameState, style, format, History, PlayerInput | a Room change keeps 38.5% of call 170, against 22.2% without the reorder | No change in the 3 current-tag pairs, because the Room held there. It sets what a Room change costs | If ticket 06 lets an author order the blocks, the author must know this rule. A preset that moves a volatile block to the front gives a break on every call |
| 4 | Quantize the history cut so that it moves in steps, not on every call | keeps the hit near 95% above the context cap (inferred) | Today the cut slides every call, so the hit there is 10% (inferred) | None. It is a code rule |
| 5 | Send a stable `session_id` per Game to OpenRouter | does not change the prefix. It keeps the same upstream host (inferred) | The same call cost $0.700 or $0.210 per 1M depending on the host (known) | None. It is one field and the Game id is the natural value |
| 6 | Keep the system message the same for a Game | about 6274 chars, over 20% of the call | Already true. All 18 measured pairs share the `system` message byte for byte | None today. A per-call timestamp or a per-call author name in `<role>` would break it |

Order and effect:

- **known** Rules 1 to 3 moved the Narration prefix from a mean of 27.8% to a mean of 84.6% of the
  call, and the Impersonate prefix from 26.8% to 74.0%. The three current-tag Narration pairs
  (161 to 164, 164 to 167, 167 to 170) reach 95.5% to 96.7%, because no Room changed in them.
- **inferred** A Room change keeps the part before GameState, which is 10881 of the 28273 chars of
  call 170 (38.5%). The next block is the history, so a Room change loses the history and the tail.
- **inferred** Rules 1 to 3 alone take a 100-cycle trial from $1.86 to $1.64, because the context
  cap binds at cycle 18 and the sliding cut then brings the hit back to 10%. Rule 4 takes the same
  trial to $0.35. Rule 5 keeps the route on the cheaper host and also protects the cache.
- **inferred** The order of the cheap rules by value is 1, then 4, then 3, then 5, then 2. Rule 1
  comes first because a random card order breaks the cache even in an otherwise good layout.

### One conflict to name

- **known** `CharactersPresent` follows the Quantifier's own order (`game_rules.rs:67`), so sorting
  it is a rule too. Sorting it changes what the model sees when the Quantifier returns a new order,
  but not the set of Characters. It is the same kind of rule as rule 1.

## 7. Facts for the later tickets

For [Decide which parts of a Narrator-role call an author can change](../issues/06-the-layout-model.md):

- **An order rule with its reason.** "The parts that change less often come first." The reason: a
  provider cache reads only a prefix, so the first changed byte ends the hit, and the history is
  the largest part of a long call. Measured: GameState at byte 20 of the user message ends the hit
  at 6304 to 6725 chars of a 16608 to 28271 char call.
- **A second order rule.** "The new story text comes last." The reason: the history only grows at
  its end, so a history before the tail keeps its whole length in the prefix. Measured: the older
  history block was a byte prefix of the newer one in all 8 Narration pairs.
- **What a layout model must allow.** Model A (an ordered template of blocks) can carry rules 1 to
  3 as the default order, and an author who reorders gets a written warning. Model B and model C
  cannot express rule 3 at all, so the engine would fix the order and the author would have no
  handle on it. That is a cheaper choice, and it is not a loss for the author.
- **A part that must stay engine-fixed for the cache:** the system half is one message for the
  whole Game. A per-call value inside `<role>`, `<instructions>` or `<global_rules>` drops the hit
  from 95% to 0% on every call.

For [Shape the test setup and the trial](../issues/08-test-setup-and-trial.md):

- **Record `usage.prompt_tokens_details.cached_tokens` per call.** All three of our providers report
  it at that path, with one number for OpenRouter and Ollama, and `prompt_cache_hit_tokens` for
  DeepSeek direct.
- **Read `usage.opencode.allowance` for an OpenCode Go trial.** Its `usage.cost` is 0, so a trial
  on the current narration connection reports no cost.
- **No schema change is needed now.** `raw_response_json` already holds the whole response,
  `usage` included, so a reading pack can total cached tokens and cost today. The parser in
  `src/adapters/driven/llm/transport/utils/response.rs:28-36` reads only `completion_tokens` and
  `reasoning_tokens`, and nothing stores a cached count in a column.
- **A trial's cost is not reproducible until the route is pinned.** The same model cost $0.700,
  $0.210 or $0.440 per 1M input tokens depending on the upstream host that OpenRouter chose. A
  cost limit per trial needs the host in the report.
- **A trial that runs past the context cap measures the cut, not the story.** At 32768 context the
  cap binds at about cycle 18 of a 100-cycle trial, so most of the trial runs on a trimmed history.
  Report the cap with the trial.
- **The measured figures for a cost limit.** Four calls per cycle, 7439 input tokens at cycle 1,
  about +1350 input tokens per cycle, and $0.0038 for cycle 1 on the Wafer price.

## 8. Sources

Provider facts, with the date I read each one (2026-10-31):

| Source | URL | The facts I took from it |
|---|---|---|
| DeepSeek, Context Caching | https://api-docs.deepseek.com/guides/kv_cache | Caching on by default. A hit needs a full cache prefix unit. The `usage` fields `prompt_cache_hit_tokens` and `prompt_cache_miss_tokens`. The cache lives hours to days |
| DeepSeek, Models & Pricing | https://api-docs.deepseek.com/quick_start/pricing.md | deepseek-flash: cache hit $0.003 off-peak and $0.006 peak per 1M, cache miss $0.15 and $0.30. The legacy name `deepseek-v4-flash` still serves DeepSeek-V4.1-Flash |
| OpenRouter, Prompt Caching | https://openrouter.ai/docs/guides/best-practices/prompt-caching | Most providers cache automatically. Provider sticky routing and its key. `session_id`, `x-session-id` and `prompt_cache_key`. `cached_tokens` and `cache_write_tokens`. DeepSeek cache reads at 0.1x. The Gemini tip to keep the first part of the messages fixed |
| Ollama, FAQ | https://docs.ollama.com/faq | `keep_alive` default 5 minutes, and its values |
| Ollama, Context length | https://docs.ollama.com/context-length | The default context length by VRAM: 4k, 32k, 256k |
| Ollama, Usage | https://docs.ollama.com/api/usage | `prompt_eval_count`, `prompt_eval_cached_count`, `prompt_eval_duration` |
| Ollama, `openai/openai.go` | https://github.com/ollama/ollama/blob/main/openai/openai.go | `usage.prompt_tokens_details.cached_tokens` is filled from `PromptEvalCachedCount` |
| llama.cpp, server README | https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md | `--cache-prompt` default enabled, `--cache-reuse`, `-c/--ctx-size` |
| llama.cpp, KV cache reuse tutorial | https://github.com/ggml-org/llama.cpp/discussions/13606 | A slot is reused when the prompt is similar enough, `-sps` default 0.5 |
| Algol Labs, Gemma 4 26B on a consumer GPU | https://algollabs.com/blog/gemma4-bfcl | 5951 prompt tokens/s at pp2048 on an RTX 5070 Ti |
| llamabench.ai, gemma-4-26B-A4B-it | https://llamabench.ai/?mode=tune&model=gemma-4-26b-a4b-it | 12176 prompt tokens/s on an RTX 5090 |
| SillyTavern source | `/tmp/research/st` at `06bde939fb1e9c4c8d8641d810f0a916b5bce127` | `cachingAtDepth` and `enableSystemPromptCache` defaults, `cache_control` at depth, `x-session-affinity` |
| Marinara Engine source | `/tmp/research/marinara` at `457e827a3d3291be978121f8c50c98c5dd9d644d` | The connection cache fields and their defaults, the Anthropic-only `cache_control` |

Engine facts come from `src/`, and the measured figures come from the read-only DB opens named in
section 1.

## 9. Open questions

- I do not know the OpenCode Go cache rule. Its stored answer reports cached tokens, but the two
  stored calls of that connection do not share a prompt, so nothing shows whether a second call of
  the same kind would hit. One measured pair would answer it.
- I do not know the exact block rule of the DeepSeek cache. The stored hits are 512, 1024 and 1280
  tokens, and the system messages are 700 and 1360 tokens, so I cannot derive the rounding from
  these numbers.
- I could not measure an Options pair, because no Game in the DB copies ran Options twice. The
  Options prompt is short (965 to 970 chars of system text and 1680 to 3579 chars of user text), so
  a cache gain there is small. That is inferred from its size, not measured.
- I could not measure the prefill time on any connection separately from the decode and network
  time. The stored rows have one timestamp per call.
- I do not know how an author-ordered layout (ticket 06, model A) would affect the hit in practice.
  A measured run of two orders on one Game would answer it, and that run is a trial.
